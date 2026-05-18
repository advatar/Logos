use serde::{Deserialize, Serialize};

use crate::{digest, encrypt, encode_share, Ciphertext, Hash32, Scalar, SharePublicKey, ThresholdPublicKey};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumConfig {
    pub forum_id: Hash32,
    pub k: u8,
    pub n: u8,
    pub moderators: Vec<ModeratorIdentity>,
    pub mod_set_version: u64,
    /// Forum-wide threshold-decryption public key `Y = s·G`.
    pub threshold_public_key: ThresholdPublicKey,
}

impl ForumConfig {
    /// Canonical 32-byte hash of the threshold public key. Bound into every
    /// transcript that depends on the threshold-key configuration.
    pub fn threshold_public_key_hash(&self) -> Hash32 {
        self.threshold_public_key.hash()
    }

    pub fn find_moderator(&self, id: &ModeratorId) -> Option<&ModeratorIdentity> {
        self.moderators.iter().find(|m| &m.id == id)
    }

    pub fn share_public_key(&self, idx: u32) -> Option<&SharePublicKey> {
        self.moderators
            .iter()
            .map(|m| &m.share_public_key)
            .find(|spk| spk.idx == idx)
    }
}

/// A moderator's stable forum identity: opaque id, Ed25519 verifying key, and
/// threshold-decryption share public key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeratorIdentity {
    pub id: ModeratorId,
    pub verifying_key: [u8; 32],
    pub share_public_key: SharePublicKey,
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
    pub ciphertext: Ciphertext,
    pub ciphertext_hash: Hash32,
    pub share_commitment: Hash32,
    pub retro_tag: Hash32,
    pub zk_receipt: MockZkReceipt,
}

impl AnonymousPostEnvelope {
    /// Build a post envelope with a real threshold-ElGamal ciphertext under
    /// the forum's threshold public key. The ZK receipt is still a
    /// `MockZkReceipt`; the RISC0 path will replace that without changing the
    /// rest of the envelope.
    pub fn build(
        forum: &ForumConfig,
        member: &MemberSecret,
        content_id: Hash32,
        nonce: Vec<u8>,
    ) -> Self {
        let post_id = digest("post-id", &[&forum.forum_id, &content_id, &nonce]);
        let share = crate::share_for(&forum.forum_id, &member.coeffs, &content_id, &nonce);
        let share_commitment = crate::share_commitment(&forum.forum_id, &content_id, &nonce, share);
        let plaintext = encode_share(share);
        let ciphertext = encrypt(&forum.threshold_public_key, &plaintext, &post_id);
        let ciphertext_hash = ciphertext.hash();
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
                &forum.threshold_public_key_hash(),
            ],
        );
        let commitment = member.commitment(&forum.forum_id);
        Self {
            forum_id: forum.forum_id,
            post_id,
            content_id,
            post_nonce: nonce,
            proof_public_inputs_hash: public_inputs_hash,
            ciphertext,
            ciphertext_hash,
            share_commitment,
            retro_tag,
            zk_receipt: MockZkReceipt {
                public_inputs_hash,
                hidden_commitment_for_local_model: commitment,
                valid: true,
            },
        }
    }

    /// Domain seed used by every moderator's DLEQ proof for partial
    /// decryptions of this post. Binding to `(forum_id, post_id)` prevents
    /// proof reuse across posts and across forums.
    pub fn dleq_domain_seed(&self) -> Hash32 {
        dleq_domain_seed_for(&self.forum_id, &self.post_id)
    }
}

pub fn dleq_domain_seed_for(forum_id: &Hash32, post_id: &Hash32) -> Hash32 {
    digest("partial-dleq-domain", &[forum_id, post_id])
}
