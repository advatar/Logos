//! LP-0016 RISC0 guest.
//!
//! When built with `--features risc0` and the RISC0 toolchain, this binary
//! becomes the guest ELF whose receipt is committed by the post envelope.
//! Without the feature the binary is a no-op so `cargo check` works.
//!
//! The guest reads `PublicInputs` then `PrivateInputs` from `env`, runs
//! `risc0_statement::verify`, then commits the public-inputs commitment.

#![cfg_attr(feature = "risc0", no_main)]
#![cfg_attr(feature = "risc0", no_std)]

#[cfg(feature = "risc0")]
risc0_zkvm::guest::entry!(main);

#[cfg(feature = "risc0")]
fn main() {
    use risc0_statement::{PrivateInputs, PublicInputs};
    use risc0_zkvm::guest::env;

    let public: PublicInputs = env::read();
    let private: PrivateInputs = env::read();
    risc0_statement::verify(&public, &private).expect("LP-0016 statement check failed");
    let commit_hash = public.commitment();
    env::commit(&commit_hash);
}

#[cfg(not(feature = "risc0"))]
fn main() {}
