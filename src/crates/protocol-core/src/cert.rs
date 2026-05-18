use std::collections::BTreeSet;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{digest, ForumConfig, Hash32, ModeratorId, ModeratorIdentity, ProtocolError, Result, Share};

/// 64-byte Ed25519 signature serialized as a fixed array for canonical bytes.
type SignatureBytes = [u8; 64];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateStatement {
    pub forum_id: Hash32,
    pub post_id: Hash32,
    pub content_id: Hash32,
    pub proof_public_inputs_hash: Hash32,
    pub ciphertext_hash: Hash32,
    pub reason_hash: Hash32,
    pub mod_set_version: u64,
    pub k: u8,
    pub n: u8,
    pub threshold_public_key_hash: Hash32,
}

impl CertificateStatement {
    pub fn hash(&self) -> Hash32 {
        digest(
            "certificate-statement",
            &[
                &self.forum_id,
                &self.post_id,
                &self.content_id,
                &self.proof_public_inputs_hash,
                &self.ciphertext_hash,
                &self.reason_hash,
                &self.mod_set_version.to_be_bytes(),
                &[self.k],
                &[self.n],
                &self.threshold_public_key_hash,
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationVote {
    pub moderator_id: ModeratorId,
    pub statement_hash: Hash32,
    /// 64-byte Ed25519 signature over `statement_hash`.
    #[serde(with = "serde_signature_bytes")]
    pub signature: SignatureBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationCertificate {
    pub statement: CertificateStatement,
    pub votes: Vec<ModerationVote>,
    pub revealed_share: Share,
}

/// A moderator's local signing material. Production storage should keep this
/// behind hardware key isolation; this type provides the protocol interface.
#[derive(Debug)]
pub struct ModeratorSecret {
    pub id: ModeratorId,
    signing_key: SigningKey,
}

impl ModeratorSecret {
    pub fn from_seed(id: ModeratorId, seed: &[u8; 32]) -> Self {
        Self { id, signing_key: SigningKey::from_bytes(seed) }
    }

    pub fn identity(&self) -> ModeratorIdentity {
        ModeratorIdentity {
            id: self.id.clone(),
            verifying_key: self.signing_key.verifying_key().to_bytes(),
        }
    }

    fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }
}

pub fn statement_for(
    forum: &ForumConfig,
    post_id: Hash32,
    content_id: Hash32,
    proof_public_inputs_hash: Hash32,
    ciphertext_hash: Hash32,
    reason_hash: Hash32,
) -> CertificateStatement {
    CertificateStatement {
        forum_id: forum.forum_id,
        post_id,
        content_id,
        proof_public_inputs_hash,
        ciphertext_hash,
        reason_hash,
        mod_set_version: forum.mod_set_version,
        k: forum.k,
        n: forum.n,
        threshold_public_key_hash: forum.threshold_public_key_hash,
    }
}

pub fn create_vote(
    forum: &ForumConfig,
    moderator: &ModeratorSecret,
    statement: &CertificateStatement,
) -> Result<ModerationVote> {
    let identity = moderator.identity();
    if !forum.moderators.iter().any(|m| m == &identity) {
        return Err(ProtocolError::InvalidModerator);
    }
    let statement_hash = statement.hash();
    let signature = moderator.sign(&statement_hash);
    Ok(ModerationVote {
        moderator_id: moderator.id.clone(),
        statement_hash,
        signature: signature.to_bytes(),
    })
}

pub fn verify_vote(forum: &ForumConfig, vote: &ModerationVote, statement_hash: &Hash32) -> bool {
    if &vote.statement_hash != statement_hash {
        return false;
    }
    let Some(identity) = forum.moderators.iter().find(|m| m.id == vote.moderator_id) else {
        return false;
    };
    let Ok(vk) = VerifyingKey::from_bytes(&identity.verifying_key) else {
        return false;
    };
    let signature = Signature::from_bytes(&vote.signature);
    vk.verify(statement_hash, &signature).is_ok()
}

pub fn verify_certificate(forum: &ForumConfig, cert: &ModerationCertificate) -> Result<()> {
    let st = &cert.statement;
    if st.forum_id != forum.forum_id
        || st.k != forum.k
        || st.n != forum.n
        || st.mod_set_version != forum.mod_set_version
        || st.threshold_public_key_hash != forum.threshold_public_key_hash
    {
        return Err(ProtocolError::InvalidCertificate);
    }
    let distinct: BTreeSet<_> = cert.votes.iter().map(|v| v.moderator_id.clone()).collect();
    if distinct.len() < forum.n as usize {
        return Err(ProtocolError::PartialCertificate);
    }
    let h = st.hash();
    for vote in &cert.votes {
        if !verify_vote(forum, vote, &h) {
            return Err(ProtocolError::InvalidVoteStatement);
        }
    }
    Ok(())
}

mod serde_signature_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 64], ser: S) -> Result<S::Ok, S::Error> {
        if ser.is_human_readable() {
            ser.serialize_str(&hex::encode(bytes))
        } else {
            bytes.serialize(ser)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<[u8; 64], D::Error> {
        use serde::de::Error;
        if de.is_human_readable() {
            let s = String::deserialize(de)?;
            let raw = hex::decode(&s).map_err(D::Error::custom)?;
            raw.try_into().map_err(|_| D::Error::custom("signature must be 64 bytes"))
        } else {
            let raw = <Vec<u8>>::deserialize(de)?;
            raw.try_into().map_err(|_| D::Error::custom("signature must be 64 bytes"))
        }
    }
}
