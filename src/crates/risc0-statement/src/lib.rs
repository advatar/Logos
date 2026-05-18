//! Pure LP-0016 membership-and-post statement.
//!
//! This crate holds the *exact same check function* that the RISC0 guest
//! runs and that a CPU-side test harness verifies — same code, no
//! duplication. The guest links this crate, reads its public + private
//! inputs from `risc0_zkvm::guest::env`, calls [`verify`], and commits the
//! public-inputs hash. A non-zkvm caller can call the same function to
//! sanity-check inputs before proving.
//!
//! Public inputs (committed to the receipt):
//!
//! ```text
//! forum_id, K, membership_root, revocation_root, content_id, post_nonce,
//! threshold_public_key_hash, ciphertext_hash, retro_tag, share_commitment
//! ```
//!
//! Private inputs (passed only to the prover):
//!
//! ```text
//! polynomial coefficients
//! membership Merkle path proving member_commitment ∈ membership_root
//! encryption nonce_seed used to produce ciphertext_hash deterministically
//! ```
//!
//! Revocation non-membership is proved with predecessor/successor witnesses
//! against the sorted revocation Merkle tree.

use protocol_core::{
    commitment_for, digest, encrypt, eval_poly, hash_to_field, merkle_verify_membership, retro_tag,
    share_commitment as share_commitment_for, ThresholdPublicKey,
};
use protocol_core::{
    merkle_verify_non_membership, Hash32, MerklePath, NonMembershipProof, ProtocolError, Scalar,
    Share,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicInputs {
    pub forum_id: Hash32,
    pub k: u8,
    pub membership_root: Hash32,
    pub revocation_root: Hash32,
    pub content_id: Hash32,
    pub post_nonce: Vec<u8>,
    pub threshold_public_key_hash: Hash32,
    pub ciphertext_hash: Hash32,
    pub retro_tag: Hash32,
    pub share_commitment: Hash32,
}

impl PublicInputs {
    /// Same framing as `AnonymousPostEnvelope::build` so the receipt
    /// commitment hash is comparable.
    pub fn commitment(&self) -> Hash32 {
        let k_bytes = [self.k];
        digest(
            "proof-public-inputs",
            &[
                &self.forum_id,
                &k_bytes,
                &self.content_id,
                &self.post_nonce,
                &self.ciphertext_hash,
                &self.share_commitment,
                &self.retro_tag,
                &self.threshold_public_key_hash,
                &self.membership_root,
                &self.revocation_root,
            ],
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInputs {
    pub coeffs: Vec<Scalar>,
    pub membership_path: MerklePath,
    pub revocation_non_membership: NonMembershipProof,
    pub threshold_public_key: ThresholdPublicKey,
    pub encryption_nonce_seed: Vec<u8>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StatementError {
    #[error("member commitment is not in membership_root")]
    NotInMembershipTree,
    #[error("member commitment is in revocation_root or the non-membership proof is invalid")]
    BadRevocationNonMembership,
    #[error("share x-coordinate is not derived from public inputs")]
    BadShareX,
    #[error("share_commitment does not match the derived (x, y)")]
    BadShareCommitment,
    #[error("retro_tag does not match the polynomial")]
    BadRetroTag,
    #[error("ciphertext_hash does not bind the derived encryption of (x, y)")]
    BadCiphertextHash,
    #[error("threshold_public_key_hash does not match the threshold key")]
    BadThresholdKeyHash,
    #[error("public-inputs commitment does not match the computed framing")]
    BadCommitment,
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}

/// Validate that the private inputs satisfy the public statement. The RISC0
/// guest is this function plus an `env::commit(&inputs.commitment())` call;
/// CPU callers use it directly.
pub fn verify(public: &PublicInputs, private: &PrivateInputs) -> Result<(), StatementError> {
    if private.threshold_public_key.hash() != public.threshold_public_key_hash {
        return Err(StatementError::BadThresholdKeyHash);
    }
    let commitment = commitment_for(&public.forum_id, public.k, &private.coeffs);
    if !merkle_verify_membership(
        &public.membership_root,
        &commitment,
        &private.membership_path,
    ) {
        return Err(StatementError::NotInMembershipTree);
    }
    if !merkle_verify_non_membership(
        &public.revocation_root,
        &commitment,
        &private.revocation_non_membership,
    ) {
        return Err(StatementError::BadRevocationNonMembership);
    }
    let x = hash_to_field(
        "share-x",
        &[&public.forum_id, &public.content_id, &public.post_nonce],
    );
    let y = eval_poly(&private.coeffs, x);
    let derived_share_commitment = share_commitment_for(
        &public.forum_id,
        &public.content_id,
        &public.post_nonce,
        Share { x, y },
    );
    if derived_share_commitment != public.share_commitment {
        return Err(StatementError::BadShareCommitment);
    }
    let derived_retro = retro_tag(
        &public.forum_id,
        &private.coeffs,
        &public.content_id,
        &public.post_nonce,
    );
    if derived_retro != public.retro_tag {
        return Err(StatementError::BadRetroTag);
    }
    let plaintext = protocol_core::encode_share(Share { x, y });
    let post_id = digest(
        "post-id",
        &[&public.forum_id, &public.content_id, &public.post_nonce],
    );
    let ciphertext = encrypt(&private.threshold_public_key, &plaintext, &post_id);
    if ciphertext.hash() != public.ciphertext_hash {
        return Err(StatementError::BadCiphertextHash);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use protocol_core::{
        AnonymousPostEnvelope, DealerShares, ForumConfig, MemberSecret, ModeratorId,
        ModeratorIdentity, ModeratorSecret, RegistryState,
    };

    fn forum() -> (ForumConfig, MemberSecret, RegistryState) {
        let dealer = DealerShares::pedersen_dkg(2, 3, b"r0-stmt-dealer");
        let names = ["alice", "bob", "carol"];
        let mods: Vec<ModeratorSecret> = names
            .iter()
            .zip(dealer.share_secret_keys.iter())
            .map(|(n, sk)| {
                let seed: [u8; 32] = digest("mod-sign-seed", &[n.as_bytes()]);
                ModeratorSecret::new(
                    ModeratorId((*n).into()),
                    SigningKey::from_bytes(&seed),
                    sk.clone(),
                )
            })
            .collect();
        let identities: Vec<ModeratorIdentity> =
            mods.iter().map(ModeratorSecret::identity).collect();
        let forum = ForumConfig {
            forum_id: digest("forum", &[b"r0-stmt"]),
            k: 2,
            n: 2,
            moderators: identities,
            mod_set_version: 1,
            threshold_public_key: dealer.threshold_public_key,
        };
        let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"seed");
        let mut registry = RegistryState::default();
        registry
            .register(member.commitment(&forum.forum_id))
            .unwrap();
        (forum, member, registry)
    }

    fn build_inputs(
        forum: &ForumConfig,
        registry: &RegistryState,
        member: &MemberSecret,
    ) -> (PublicInputs, PrivateInputs, AnonymousPostEnvelope) {
        let content_id = digest("content", &[b"x"]);
        let nonce = vec![1u8, 2, 3];
        let post = AnonymousPostEnvelope::build(forum, registry, member, content_id, nonce.clone());
        let public = PublicInputs {
            forum_id: forum.forum_id,
            k: forum.k,
            membership_root: registry.membership_root(),
            revocation_root: registry.revocation_root(),
            content_id,
            post_nonce: nonce,
            threshold_public_key_hash: forum.threshold_public_key_hash(),
            ciphertext_hash: post.ciphertext_hash,
            retro_tag: post.retro_tag,
            share_commitment: post.share_commitment,
        };
        let leaves: Vec<Hash32> = registry.registered.iter().copied().collect();
        let commitment = member.commitment(&forum.forum_id);
        let path = protocol_core::merkle_prove_membership(&leaves, &commitment).unwrap();
        let revoked: Vec<Hash32> = registry.revoked.iter().copied().collect();
        let private = PrivateInputs {
            coeffs: member.coeffs.clone(),
            membership_path: path,
            revocation_non_membership: protocol_core::merkle_prove_non_membership(
                &revoked,
                &commitment,
            )
            .unwrap(),
            threshold_public_key: forum.threshold_public_key,
            encryption_nonce_seed: post.post_id.to_vec(),
        };
        (public, private, post)
    }

    #[test]
    fn verify_accepts_real_inputs() {
        let (forum, member, registry) = forum();
        let (public, private, post) = build_inputs(&forum, &registry, &member);
        verify(&public, &private).unwrap();
        // The same commitment framing as the post envelope must match.
        assert_eq!(public.commitment(), post.proof_public_inputs_hash);
    }

    #[test]
    fn verify_rejects_tampered_membership_root() {
        let (forum, member, registry) = forum();
        let (mut public, private, _post) = build_inputs(&forum, &registry, &member);
        public.membership_root[0] ^= 0xff;
        assert_eq!(
            verify(&public, &private).unwrap_err(),
            StatementError::NotInMembershipTree
        );
    }

    #[test]
    fn verify_rejects_wrong_coeffs() {
        let (forum, member, registry) = forum();
        let (public, mut private, _post) = build_inputs(&forum, &registry, &member);
        private.coeffs[0] += Scalar::ONE;
        let err = verify(&public, &private).unwrap_err();
        assert!(matches!(
            err,
            StatementError::NotInMembershipTree | StatementError::BadShareCommitment
        ));
    }

    #[test]
    fn verify_rejects_swapped_threshold_key() {
        let (forum, member, registry) = forum();
        let other = DealerShares::pedersen_dkg(2, 3, b"other-dealer");
        let (public, mut private, _post) = build_inputs(&forum, &registry, &member);
        private.threshold_public_key = other.threshold_public_key;
        assert_eq!(
            verify(&public, &private).unwrap_err(),
            StatementError::BadThresholdKeyHash
        );
    }

    #[test]
    fn verify_rejects_revoked_member_root() {
        let (forum, member, registry) = forum();
        let (mut public, private, _post) = build_inputs(&forum, &registry, &member);
        let commitment = member.commitment(&forum.forum_id);
        public.revocation_root = protocol_core::root_from_set([commitment]);
        assert_eq!(
            verify(&public, &private).unwrap_err(),
            StatementError::BadRevocationNonMembership
        );
    }
}
