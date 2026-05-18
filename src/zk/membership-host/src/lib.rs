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
        let env = ExecutorEnv::builder()
            .write(public)
            .map_err(ProveError::env)?
            .write(private)
            .map_err(ProveError::env)?
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
        let journal: [u8; 32] = receipt.journal.decode().map_err(ProveError::env)?;
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
        let journal: [u8; 32] = receipt.journal.decode().map_err(ProveError::env)?;
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
}
