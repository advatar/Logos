//! LEZ/SPEL registry boundary stub.
//!
//! Replace this with the current SPEL annotations after generating a project
//! with `logos-scaffold`. The intended IDL contains the three instructions
//! below.

use protocol_core::{ForumConfig, Hash32, ModerationCertificate, RegistryState, Result, slash};

pub struct ForumAccount {
    pub config: ForumConfig,
    pub registry: RegistryState,
}

// #[lez_program]
pub mod lp0016_registry {
    use super::*;

    // #[instruction]
    pub fn create_forum(config: ForumConfig) -> ForumAccount {
        ForumAccount { config, registry: RegistryState::default() }
    }

    // #[instruction]
    pub fn register_member(account: &mut ForumAccount, member_commitment: Hash32) -> Result<()> {
        account.registry.register(member_commitment)
    }

    // #[instruction]
    pub fn slash_member(account: &mut ForumAccount, certificates: Vec<ModerationCertificate>) -> Result<Hash32> {
        let result = slash(&mut account.registry, &account.config, &certificates)?;
        Ok(result.commitment)
    }
}
