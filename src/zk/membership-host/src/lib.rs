//! LP-0016 RISC0 host.
//!
//! Builds an `ExecutorEnv` from `PublicInputs` + `PrivateInputs`, asks the
//! default prover for a receipt against the membership-guest ELF, and exposes
//! a verifier that checks the receipt against the expected image id and the
//! public-inputs commitment.
//!
//! Without the `risc0` feature the crate compiles as a stub so editor
//! tooling and the workspace-wide `cargo check` keep working without the
//! RISC0 toolchain.

#[cfg(feature = "risc0")]
use risc0_statement::{PrivateInputs, PublicInputs};

#[cfg(feature = "risc0")]
pub mod real {
    use super::*;
    use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

    pub fn prove(
        public: &PublicInputs,
        private: &PrivateInputs,
        guest_elf: &[u8],
    ) -> Result<Receipt, ProveError> {
        let public_bytes = postcard::to_allocvec(public).map_err(ProveError::env)?;
        let private_bytes = postcard::to_allocvec(private).map_err(ProveError::env)?;
        let env = ExecutorEnv::builder()
            .write_frame(&public_bytes)
            .write_frame(&private_bytes)
            .build()
            .map_err(ProveError::env)?;
        let prover = default_prover();
        let info = prover.prove(env, guest_elf).map_err(ProveError::env)?;
        Ok(info.receipt)
    }

    pub fn verify(
        receipt: &Receipt,
        image_id: [u32; 8],
        expected_commitment: &[u8; 32],
    ) -> Result<(), ProveError> {
        receipt.verify(image_id).map_err(ProveError::env)?;
        // The guest commits the 32-byte public-inputs commitment.
        let journal = journal_commitment(receipt)?;
        if &journal != expected_commitment {
            return Err(ProveError::CommitmentMismatch);
        }
        Ok(())
    }

    pub fn receipt_to_protocol(
        receipt: &Receipt,
        image_id: [u32; 8],
        expected_commitment: &[u8; 32],
    ) -> Result<protocol_core::ZkReceipt, ProveError> {
        verify(receipt, image_id, expected_commitment)?;
        let journal = journal_commitment(receipt)?;
        let receipt_bytes = bincode::serialize(receipt).map_err(ProveError::env)?;
        protocol_core::ZkReceipt::risc0(
            *expected_commitment,
            image_id_words_to_bytes(image_id),
            journal,
            receipt_bytes,
        )
        .map_err(ProveError::protocol)
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ProveError {
        #[error("risc0 runtime: {0}")]
        Risc0(String),
        #[error("receipt commits a different public-inputs hash than expected")]
        CommitmentMismatch,
        #[error("protocol receipt rejected: {0}")]
        Protocol(String),
    }

    impl ProveError {
        fn env<E: core::fmt::Display>(e: E) -> Self {
            Self::Risc0(e.to_string())
        }

        fn protocol<E: core::fmt::Display>(e: E) -> Self {
            Self::Protocol(e.to_string())
        }
    }

    fn journal_commitment(receipt: &Receipt) -> Result<[u8; 32], ProveError> {
        if receipt.journal.bytes.len() != 32 {
            return Err(ProveError::env(format!(
                "journal length {} did not equal 32 bytes",
                receipt.journal.bytes.len()
            )));
        }
        let mut journal = [0u8; 32];
        journal.copy_from_slice(&receipt.journal.bytes);
        Ok(journal)
    }
}

pub fn image_id_words_to_bytes(image_id: [u32; 8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (chunk, word) in out.chunks_exact_mut(4).zip(image_id) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    out
}

#[cfg(not(feature = "risc0"))]
pub fn prove_unavailable() -> ! {
    panic!("lp0016-membership-host built without the `risc0` feature; install cargo-risczero and rebuild with --features risc0");
}

// Re-export the statement so callers don't have to depend on it twice.
pub use risc0_statement;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_risc0_image_id_words_to_protocol_bytes() {
        let bytes = image_id_words_to_bytes([
            0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f, 0x10111213, 0x14151617, 0x18191a1b,
            0x1c1d1e1f,
        ]);
        assert_eq!(&bytes[0..4], &[3, 2, 1, 0]);
        assert_eq!(&bytes[28..32], &[31, 30, 29, 28]);
    }

    #[cfg(feature = "risc0")]
    fn sample_inputs() -> (
        risc0_statement::PublicInputs,
        risc0_statement::PrivateInputs,
    ) {
        use protocol_core::{
            digest, merkle_prove_membership, merkle_prove_non_membership, AnonymousPostEnvelope,
            DealerShares, ForumConfig, Hash32, MemberSecret, ModeratorId, ModeratorIdentity,
            RegistryState,
        };

        let dealer = DealerShares::pedersen_dkg(2, 3, b"r0-host-prove");
        let moderators: Vec<ModeratorIdentity> = dealer
            .share_public_keys
            .iter()
            .enumerate()
            .map(|(idx, share_public_key)| ModeratorIdentity {
                id: ModeratorId(format!("m{}", idx + 1)),
                verifying_key: digest("mod-vk", &[&[idx as u8]]),
                share_public_key: share_public_key.clone(),
            })
            .collect();
        let forum = ForumConfig {
            forum_id: digest("forum", &[b"r0-host-prove"]),
            k: 2,
            n: 2,
            moderators,
            mod_set_version: 1,
            threshold_public_key: dealer.threshold_public_key,
        };
        let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"member-seed");
        let mut registry = RegistryState::default();
        registry
            .register(member.commitment(&forum.forum_id))
            .unwrap();

        let content_id = digest("content", &[b"proof-perf"]);
        let nonce = b"proof-perf-nonce".to_vec();
        let post = AnonymousPostEnvelope::build(&forum, &registry, &member, content_id, nonce);
        let public = risc0_statement::PublicInputs {
            forum_id: forum.forum_id,
            k: forum.k,
            membership_root: registry.membership_root(),
            revocation_root: registry.revocation_root(),
            content_id,
            post_nonce: post.post_nonce.clone(),
            threshold_public_key_hash: forum.threshold_public_key_hash(),
            ciphertext_hash: post.ciphertext_hash,
            retro_tag: post.retro_tag,
            share_commitment: post.share_commitment,
        };
        let commitment = member.commitment(&forum.forum_id);
        let leaves: Vec<Hash32> = registry.registered.iter().copied().collect();
        let revoked: Vec<Hash32> = registry.revoked.iter().copied().collect();
        let private = risc0_statement::PrivateInputs {
            coeffs: member.coeffs.clone(),
            membership_path: merkle_prove_membership(&leaves, &commitment).unwrap(),
            revocation_non_membership: merkle_prove_non_membership(&revoked, &commitment).unwrap(),
        };
        assert_eq!(public.commitment(), post.proof_public_inputs_hash);
        (public, private)
    }

    #[test]
    #[cfg(feature = "risc0")]
    fn proves_sample_membership_with_guest_elf() {
        let Some(guest_elf_path) = std::env::var_os("LP0016_MEMBERSHIP_GUEST_ELF") else {
            eprintln!("LP0016_MEMBERSHIP_GUEST_ELF not set; skipping real RISC0 proof test");
            return;
        };
        let guest_elf = std::fs::read(&guest_elf_path)
            .unwrap_or_else(|err| panic!("failed to read {:?}: {err}", guest_elf_path));
        let image_id: [u32; 8] = risc0_zkvm::compute_image_id(&guest_elf).unwrap().into();
        let (public, private) = sample_inputs();
        risc0_statement::verify(&public, &private).unwrap();

        let started = std::time::Instant::now();
        let receipt = real::prove(&public, &private, &guest_elf).unwrap();
        let elapsed = started.elapsed();
        let commitment = public.commitment();
        real::verify(&receipt, image_id, &commitment).unwrap();

        println!("lp0016_risc0_proof_seconds={:.3}", elapsed.as_secs_f64());
        println!("lp0016_risc0_image_id_words={:08x?}", image_id);
    }
}
