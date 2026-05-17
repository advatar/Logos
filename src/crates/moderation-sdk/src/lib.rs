//! Forum-agnostic moderation SDK facade.
//!
//! Production adapters should implement [`OffchainStore`] for Logos Storage and
//! Delivery. The in-memory store is enough for local tests and demos.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use protocol_core::*;

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
        self.data.get(id).cloned().ok_or_else(|| anyhow!("missing object: {id}"))
    }

    fn list(&self, namespace: &str) -> Result<Vec<String>> {
        Ok(self
            .data
            .keys()
            .filter(|k| k.starts_with(namespace))
            .cloned()
            .collect())
    }
}

pub struct ForumSdk<S: OffchainStore> {
    pub forum: ForumConfig,
    pub registry: RegistryState,
    pub store: S,
    pub dev_oracle: DevThresholdOracle,
}

impl<S: OffchainStore> ForumSdk<S> {
    pub fn new(forum: ForumConfig, store: S) -> protocol_core::Result<Self> {
        if forum.k == 0 {
            return Err(ProtocolError::InvalidK);
        }
        if forum.n == 0 {
            return Err(ProtocolError::InvalidN);
        }
        if forum.n as usize > forum.moderators.len() {
            return Err(ProtocolError::ThresholdTooLarge);
        }
        Ok(Self { forum, registry: RegistryState::default(), store, dev_oracle: DevThresholdOracle::default() })
    }

    pub fn register_member(&mut self, member: &MemberSecret) -> protocol_core::Result<Hash32> {
        let commitment = member.commitment(&self.forum.forum_id);
        self.registry.register(commitment)?;
        Ok(commitment)
    }

    pub fn build_post(&mut self, member: &MemberSecret, content_id: Hash32, nonce: Vec<u8>) -> protocol_core::Result<AnonymousPostEnvelope> {
        let commitment = member.commitment(&self.forum.forum_id);
        if !self.registry.is_active(&commitment) {
            return Err(ProtocolError::CommitmentNotActive);
        }
        let (post, share) = AnonymousPostEnvelope::new_dev(&self.forum, member, content_id, nonce);
        self.dev_oracle.remember(post.ciphertext_hash, share);
        Ok(post)
    }

    pub fn persist_post(&mut self, post: &AnonymousPostEnvelope) -> Result<String> {
        let bytes = serde_json::to_vec(post)?;
        self.store.put("post", bytes)
    }

    pub fn create_moderation_vote(
        &self,
        moderator_id: ModeratorId,
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
        create_vote(&self.forum, moderator_id, &st)
    }

    pub fn aggregate_certificate(
        &self,
        post: &AnonymousPostEnvelope,
        reason_hash: Hash32,
        votes: Vec<ModerationVote>,
    ) -> protocol_core::Result<ModerationCertificate> {
        let st = statement_for(
            &self.forum,
            post.post_id,
            post.content_id,
            post.proof_public_inputs_hash,
            post.ciphertext_hash,
            reason_hash,
        );
        let distinct = votes.iter().map(|v| v.moderator_id.clone()).collect::<std::collections::BTreeSet<_>>();
        if distinct.len() < self.forum.n as usize {
            return Err(ProtocolError::PartialCertificate);
        }
        let share = self.dev_oracle.decrypt(&post.ciphertext_hash).ok_or(ProtocolError::InvalidCertificate)?;
        let cert = ModerationCertificate { statement: st, votes, revealed_share: share };
        verify_certificate(&self.forum, &cert)?;
        Ok(cert)
    }

    pub fn submit_slash(&mut self, certificates: &[ModerationCertificate]) -> protocol_core::Result<SlashResult> {
        slash(&mut self.registry, &self.forum, certificates)
    }
}
