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

    pub fn prove(public: &PublicInputs, private: &PrivateInputs, guest_elf: &[u8]) -> Result<Receipt, ProveError> {
        let env = ExecutorEnv::builder()
            .write(public).map_err(ProveError::env)?
            .write(private).map_err(ProveError::env)?
            .build().map_err(ProveError::env)?;
        let prover = default_prover();
        let info = prover.prove(env, guest_elf).map_err(ProveError::env)?;
        Ok(info.receipt)
    }

    pub fn verify(receipt: &Receipt, image_id: [u32; 8], expected_commitment: &[u8; 32]) -> Result<(), ProveError> {
        receipt.verify(image_id).map_err(ProveError::env)?;
        // The guest commits the 32-byte public-inputs commitment.
        let journal: [u8; 32] = receipt
            .journal
            .decode()
            .map_err(ProveError::env)?;
        if &journal != expected_commitment {
            return Err(ProveError::CommitmentMismatch);
        }
        Ok(())
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ProveError {
        #[error("risc0 runtime: {0}")]
        Risc0(String),
        #[error("receipt commits a different public-inputs hash than expected")]
        CommitmentMismatch,
    }

    impl ProveError {
        fn env<E: core::fmt::Display>(e: E) -> Self {
            Self::Risc0(e.to_string())
        }
    }
}

#[cfg(not(feature = "risc0"))]
pub fn prove_unavailable() -> ! {
    panic!("lp0016-membership-host built without the `risc0` feature; install cargo-risczero and rebuild with --features risc0");
}

// Re-export the statement so callers don't have to depend on it twice.
pub use risc0_statement;
