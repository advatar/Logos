use serde::{Deserialize, Serialize};

use crate::{digest, Hash32, Scalar, Share};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumConfig {
    pub forum_id: Hash32,
    pub k: u8,
    pub n: u8,
    pub moderators: Vec<ModeratorIdentity>,
    pub mod_set_version: u64,
    /// Hash of the threshold-decryption public key used by this forum.
    /// Bound into certificate statements and the post public-inputs hash.
    pub threshold_public_key_hash: Hash32,
}

/// A moderator's stable forum identity: opaque id plus Ed25519 public key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeratorIdentity {
    pub id: ModeratorId,
    /// Canonical 32-byte Ed25519 verifying key.
    pub verifying_key: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ModeratorId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberSecret {
    pub coeffs: Vec<Scalar>,
}

impl MemberSecret {
    pub fn from_seed(forum_id: &Hash32, k: u8, seed: &[u8]) -> Self {
        let mut coeffs = Vec::with_capacity(k as usize);
        for i in 0..k {
            let idx = [i];
            coeffs.push(crate::hash_to_field("member-coeff", &[forum_id, seed, &idx]));
        }
        Self { coeffs }
    }

    pub fn commitment(&self, forum_id: &Hash32) -> Hash32 {
        crate::commitment_for(forum_id, self.coeffs.len() as u8, &self.coeffs)
    }
}

/// Development-only ZK receipt stand-in. Replaced by a RISC0 receipt once the
/// `risc0-verify` feature is enabled (see `STATUS.md → Phase 4`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockZkReceipt {
    pub public_inputs_hash: Hash32,
    /// Development-only; real receipts do not reveal the commitment.
    pub hidden_commitment_for_local_model: Hash32,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymousPostEnvelope {
    pub forum_id: Hash32,
    pub post_id: Hash32,
    pub content_id: Hash32,
    pub post_nonce: Vec<u8>,
    pub proof_public_inputs_hash: Hash32,
    pub ciphertext_hash: Hash32,
    pub share_commitment: Hash32,
    pub retro_tag: Hash32,
    pub zk_receipt: MockZkReceipt,
}

impl AnonymousPostEnvelope {
    pub fn new_dev(
        forum: &ForumConfig,
        member: &MemberSecret,
        content_id: Hash32,
        nonce: Vec<u8>,
    ) -> (Self, Share) {
        let post_id = digest("post-id", &[&forum.forum_id, &content_id, &nonce]);
        let share = crate::share_for(&forum.forum_id, &member.coeffs, &content_id, &nonce);
        let share_commitment = crate::share_commitment(&forum.forum_id, &content_id, &nonce, share);
        let ciphertext_hash = digest(
            "dev-threshold-ciphertext",
            &[&forum.forum_id, &post_id, &share.x.to_bytes(), &share.y.to_bytes()],
        );
        let retro_tag = crate::retro_tag(&forum.forum_id, &member.coeffs, &content_id, &nonce);
        let k_bytes = [forum.k];
        let public_inputs_hash = digest(
            "proof-public-inputs",
            &[
                &forum.forum_id,
                &k_bytes,
                &content_id,
                &nonce,
                &ciphertext_hash,
                &share_commitment,
                &retro_tag,
                &forum.threshold_public_key_hash,
            ],
        );
        let commitment = member.commitment(&forum.forum_id);
        (
            Self {
                forum_id: forum.forum_id,
                post_id,
                content_id,
                post_nonce: nonce,
                proof_public_inputs_hash: public_inputs_hash,
                ciphertext_hash,
                share_commitment,
                retro_tag,
                zk_receipt: MockZkReceipt {
                    public_inputs_hash,
                    hidden_commitment_for_local_model: commitment,
                    valid: true,
                },
            },
            share,
        )
    }
}
