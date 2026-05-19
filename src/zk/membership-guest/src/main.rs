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
extern crate alloc;

#[cfg(feature = "risc0")]
risc0_zkvm::guest::entry!(main);

#[cfg(feature = "risc0")]
fn main() {
    use risc0_statement::{PrivateInputs, PublicInputs};
    use risc0_zkvm::guest::env;

    let public_bytes = read_frame_bytes();
    let private_bytes = read_frame_bytes();
    let public: PublicInputs =
        postcard::from_bytes(&public_bytes).expect("failed to decode public inputs");
    let private: PrivateInputs =
        postcard::from_bytes(&private_bytes).expect("failed to decode private inputs");
    risc0_statement::verify(&public, &private).expect("LP-0016 statement check failed");
    let commit_hash = public.commitment();
    env::commit_slice(&commit_hash);
}

#[cfg(feature = "risc0")]
fn read_frame_bytes() -> alloc::vec::Vec<u8> {
    use risc0_zkvm::guest::env;

    let mut len = 0u32;
    env::read_slice(core::slice::from_mut(&mut len));
    let mut bytes = alloc::vec![0u8; len as usize];
    env::read_slice(&mut bytes);
    bytes
}

#[cfg(not(feature = "risc0"))]
fn main() {}
