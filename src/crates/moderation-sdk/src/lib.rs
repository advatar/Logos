//! Forum-agnostic moderation SDK facade.
//!
//! Production adapters should implement [`OffchainStore`] for Logos Storage and
//! Delivery. The in-memory store is enough for local tests and demos.

use std::collections::{BTreeMap, VecDeque};

use anyhow::{anyhow, Result};
use protocol_core::*;
use serde::{Deserialize, Serialize};

pub trait OffchainStore {
    fn put(&mut self, namespace: &str, bytes: Vec<u8>) -> Result<String>;
    fn get(&self, id: &str) -> Result<Vec<u8>>;
    fn list(&self, namespace: &str) -> Result<Vec<String>>;
}

#[derive(Default)]
pub struct MemoryStore {
    seq: u64,
    data: BTreeMap<String, Vec<u8>>,
}

impl OffchainStore for MemoryStore {
    fn put(&mut self, namespace: &str, bytes: Vec<u8>) -> Result<String> {
        self.seq += 1;
        let id = format!("{namespace}/{}", self.seq);
        self.data.insert(id.clone(), bytes);
        Ok(id)
    }

    fn get(&self, id: &str) -> Result<Vec<u8>> {
        self.data
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("missing object: {id}"))
    }

    fn list(&self, namespace: &str) -> Result<Vec<String>> {
        let prefix = format!("{namespace}/");
        Ok(self
            .data
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryTask {
    Put {
        namespace: String,
        bytes: Vec<u8>,
    },
    SubmitSlash {
        forum_id: Hash32,
        commitment: Hash32,
    },
}

pub trait RetryQueue {
    fn push(&mut self, task: RetryTask) -> Result<()>;
    fn pop(&mut self) -> Option<RetryTask>;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Default)]
pub struct MemoryRetryQueue {
    tasks: VecDeque<RetryTask>,
}

impl RetryQueue for MemoryRetryQueue {
    fn push(&mut self, task: RetryTask) -> Result<()> {
        self.tasks.push_back(task);
        Ok(())
    }

    fn pop(&mut self) -> Option<RetryTask> {
        self.tasks.pop_front()
    }

    fn len(&self) -> usize {
        self.tasks.len()
    }
}

pub struct ForumSdk<S: OffchainStore, Q: RetryQueue = MemoryRetryQueue> {
    pub forum: ForumConfig,
    pub registry: RegistryState,
    pub store: S,
    pub retry_queue: Q,
}

impl<S: OffchainStore> ForumSdk<S, MemoryRetryQueue> {
    pub fn new(forum: ForumConfig, store: S) -> protocol_core::Result<Self> {
        Self::with_retry_queue(forum, store, MemoryRetryQueue::default())
    }
}

impl<S: OffchainStore, Q: RetryQueue> ForumSdk<S, Q> {
    pub fn with_retry_queue(
        forum: ForumConfig,
        store: S,
        retry_queue: Q,
    ) -> protocol_core::Result<Self> {
        if forum.k == 0 {
            return Err(ProtocolError::InvalidK);
        }
        if forum.n == 0 {
            return Err(ProtocolError::InvalidN);
        }
        if forum.n as usize > forum.moderators.len() {
            return Err(ProtocolError::ThresholdTooLarge);
        }
        Ok(Self {
            forum,
            registry: RegistryState::default(),
            store,
            retry_queue,
        })
    }

    pub fn register_member(&mut self, member: &MemberSecret) -> protocol_core::Result<Hash32> {
        let commitment = member.commitment(&self.forum.forum_id);
        self.registry.register(commitment)?;
        Ok(commitment)
    }

    pub fn build_post(
        &mut self,
        member: &MemberSecret,
        content_id: Hash32,
        nonce: Vec<u8>,
    ) -> protocol_core::Result<AnonymousPostEnvelope> {
        let commitment = member.commitment(&self.forum.forum_id);
        if !self.registry.is_active(&commitment) {
            return Err(ProtocolError::CommitmentNotActive);
        }
        Ok(AnonymousPostEnvelope::build(
            &self.forum,
            &self.registry,
            member,
            content_id,
            nonce,
        ))
    }

    pub fn persist_post(&mut self, post: &AnonymousPostEnvelope) -> Result<String> {
        let bytes = serde_json::to_vec(post)?;
        self.put_with_retry(&storage_namespace(StorageKind::Post, &post.forum_id), bytes)
    }

    pub fn persist_vote(&mut self, forum_id: &Hash32, vote: &ModerationVote) -> Result<String> {
        let bytes = serde_json::to_vec(vote)?;
        self.put_with_retry(&storage_namespace(StorageKind::Vote, forum_id), bytes)
    }

    pub fn persist_certificate(&mut self, cert: &ModerationCertificate) -> Result<String> {
        let bytes = serde_json::to_vec(cert)?;
        self.put_with_retry(
            &storage_namespace(StorageKind::Cert, &cert.statement.forum_id),
            bytes,
        )
    }

    pub fn persist_slash_bundle(
        &mut self,
        forum_id: &Hash32,
        certificates: &[ModerationCertificate],
    ) -> Result<String> {
        let record = SlashBundleRecord {
            certificates: certificates.to_vec(),
        };
        let bytes = serde_json::to_vec(&record)?;
        self.put_with_retry(&storage_namespace(StorageKind::Slash, forum_id), bytes)
    }

    pub fn create_moderation_vote(
        &self,
        moderator: &ModeratorSecret,
        post: &AnonymousPostEnvelope,
        reason_hash: Hash32,
    ) -> protocol_core::Result<ModerationVote> {
        let st = statement_for(
            &self.forum,
            post.post_id,
            post.content_id,
            post.proof_public_inputs_hash,
            post.ciphertext_hash,
            reason_hash,
        );
        create_vote(&self.forum, moderator, &st)
    }

    /// Aggregate a certificate from the moderators' votes and partial
    /// decryptions. Caller supplies one [`PartialDecryption`] per moderator
    /// (caller is responsible for collecting them from the moderator clients).
    pub fn aggregate_certificate(
        &self,
        post: &AnonymousPostEnvelope,
        reason_hash: Hash32,
        votes: Vec<ModerationVote>,
        partial_decryptions: Vec<PartialDecryption>,
    ) -> protocol_core::Result<ModerationCertificate> {
        let st = statement_for(
            &self.forum,
            post.post_id,
            post.content_id,
            post.proof_public_inputs_hash,
            post.ciphertext_hash,
            reason_hash,
        );
        let cert = ModerationCertificate {
            statement: st,
            votes,
            ciphertext: post.ciphertext.clone(),
            partial_decryptions,
        };
        verify_certificate(&self.forum, &cert)?;
        Ok(cert)
    }

    pub fn submit_slash(&mut self, certificates: &[ModerationCertificate]) -> Result<SlashResult> {
        let result = slash(&mut self.registry, &self.forum, certificates)
            .map_err(|e| anyhow!("slash submission rejected: {e}"))?;
        let forum_id = self.forum.forum_id;
        if let Err(err) = self.persist_slash_bundle(&forum_id, certificates) {
            self.retry_queue.push(RetryTask::SubmitSlash {
                forum_id,
                commitment: result.commitment,
            })?;
            return Err(err);
        }
        Ok(result)
    }

    fn put_with_retry(&mut self, namespace: &str, bytes: Vec<u8>) -> Result<String> {
        match self.store.put(namespace, bytes.clone()) {
            Ok(id) => Ok(id),
            Err(err) => {
                self.retry_queue.push(RetryTask::Put {
                    namespace: namespace.to_string(),
                    bytes,
                })?;
                Err(err)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageKind {
    Post,
    Vote,
    Cert,
    Slash,
}

pub fn storage_namespace(kind: StorageKind, forum_id: &Hash32) -> String {
    let prefix = match kind {
        StorageKind::Post => "post",
        StorageKind::Vote => "vote",
        StorageKind::Cert => "cert",
        StorageKind::Slash => "slash",
    };
    format!("{prefix}/{}", hex::encode(forum_id))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashBundleRecord {
    pub certificates: Vec<ModerationCertificate>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;

    #[derive(Default)]
    struct FailingStore;

    impl OffchainStore for FailingStore {
        fn put(&mut self, _namespace: &str, _bytes: Vec<u8>) -> Result<String> {
            bail!("injected put failure")
        }

        fn get(&self, _id: &str) -> Result<Vec<u8>> {
            bail!("not implemented")
        }

        fn list(&self, _namespace: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    fn forum() -> (ForumConfig, MemberSecret) {
        let dealer = DealerShares::pedersen_dkg(1, 1, b"sdk-test-dkg");
        let moderator = ModeratorSecret::from_seed_and_share(
            ModeratorId("alice".into()),
            &protocol_core::digest("sdk-mod-seed", &[b"alice"]),
            dealer.share_secret_keys[0].clone(),
        );
        let forum = ForumConfig {
            forum_id: protocol_core::digest("forum", &[b"sdk"]),
            k: 1,
            n: 1,
            moderators: vec![moderator.identity()],
            mod_set_version: 1,
            threshold_public_key: dealer.threshold_public_key,
        };
        let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"member");
        (forum, member)
    }

    #[test]
    fn namespaces_are_partitioned_by_kind_and_forum() {
        let forum_id = protocol_core::digest("forum", &[b"namespaces"]);
        assert!(storage_namespace(StorageKind::Post, &forum_id).starts_with("post/"));
        assert!(storage_namespace(StorageKind::Vote, &forum_id).starts_with("vote/"));
        assert!(storage_namespace(StorageKind::Cert, &forum_id).starts_with("cert/"));
        assert!(storage_namespace(StorageKind::Slash, &forum_id).starts_with("slash/"));
    }

    #[test]
    fn failed_put_is_queued_for_retry() {
        let (forum, member) = forum();
        let mut sdk =
            ForumSdk::with_retry_queue(forum, FailingStore, MemoryRetryQueue::default()).unwrap();
        sdk.register_member(&member).unwrap();
        let post = sdk
            .build_post(
                &member,
                protocol_core::digest("content", &[b"post"]),
                b"nonce".to_vec(),
            )
            .unwrap();
        assert!(sdk.persist_post(&post).is_err());
        assert_eq!(sdk.retry_queue.len(), 1);
        match sdk.retry_queue.pop().unwrap() {
            RetryTask::Put { namespace, .. } => assert!(namespace.starts_with("post/")),
            other => panic!("unexpected retry task: {other:?}"),
        }
    }
}
