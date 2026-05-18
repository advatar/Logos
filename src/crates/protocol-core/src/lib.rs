//! Pure LP-0016 protocol model.
//!
//! This crate intentionally has no LEZ, RISC0, Basecamp, or Logos Storage
//! dependency. It models the slash/certificate/Shamir state machine that the
//! production integrations must preserve.

// Transitive pin to keep Rust 1.82.0 compat — base64ct 1.7+ needs edition 2024.
use base64ct as _;

pub mod cert;
pub mod field;
pub mod hash;
pub mod shamir;
pub mod state;
pub mod types;

pub use cert::*;
pub use field::*;
pub use hash::*;
pub use shamir::*;
pub use state::*;
pub use types::*;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("K must be at least 1")]
    InvalidK,
    #[error("N must be at least 1")]
    InvalidN,
    #[error("N cannot exceed M")]
    ThresholdTooLarge,
    #[error("not enough distinct moderators")]
    PartialCertificate,
    #[error("invalid moderator")]
    InvalidModerator,
    #[error("invalid vote statement")]
    InvalidVoteStatement,
    #[error("invalid certificate")]
    InvalidCertificate,
    #[error("slash requires exactly K certificates")]
    WrongSlashCertificateCount,
    #[error("duplicate Shamir x-coordinate")]
    DuplicateShareX,
    #[error("commitment is not active")]
    CommitmentNotActive,
    #[error("commitment is already revoked")]
    AlreadyRevoked,
    #[error("unregistered commitment")]
    UnregisteredCommitment,
}

pub type Result<T> = core::result::Result<T, ProtocolError>;
