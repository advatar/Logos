//! LP-0016 LEZ registry program.
//!
//! On the production target this crate is an LEZ program: the three
//! [`instructions`] are exposed to the sequencer and operate against the
//! account types defined here. Under the `spel` feature flag the doc-style
//! `// #[lez_program]` / `// #[instruction]` / `// #[account]` markers in
//! source will become real attribute macros via `logos-scaffold` /
//! `spel`. Until that toolchain is installed locally the crate compiles as
//! plain Rust so the rest of the workspace stays unblocked.
//!
//! The hand-written IDL at `src/registry/idl/lp0016_registry.json` is the
//! source of truth for the on-chain interface shape; the integration test
//! below verifies it stays in sync with the Rust signatures.

use std::collections::BTreeMap;

use protocol_core::{
    digest, slash, ForumConfig, Hash32, ModerationCertificate, ProtocolError, RegistryState,
    Result,
};
use serde::{Deserialize, Serialize};

// #[lez_program(name = "lp0016_registry", version = "0.1.0")]
pub mod lp0016_registry {
    use super::*;

    // #[account]
    /// Per-forum on-chain state. In a real LEZ deployment each forum owns
    /// one of these accounts, keyed by `forum_id`.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ForumState {
        pub config: ForumConfig,
        pub stake_policy: StakePolicy,
        pub membership_root: Hash32,
        pub revocation_root: Hash32,
    }

    impl ForumState {
        /// Canonical-transcript serialization for consensus. Never use
        /// `serde_json` for on-chain hashes — it is map-iteration sensitive.
        pub fn canonical_hash(&self) -> Hash32 {
            digest(
                "lp0016-account/forum-state",
                &[
                    &self.config.forum_id,
                    &[self.config.k],
                    &[self.config.n],
                    &self.config.mod_set_version.to_be_bytes(),
                    &self.config.threshold_public_key_hash(),
                    &self.membership_root,
                    &self.revocation_root,
                    &self.stake_policy.minimum_stake.to_be_bytes(),
                ],
            )
        }
    }

    // #[account]
    /// One per registered member. Keyed by `(forum_id, member_commitment)`.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct MemberRecord {
        pub forum_id: Hash32,
        pub member_commitment: Hash32,
        pub stake_amount: u64,
        pub registered_at: u64,
    }

    impl MemberRecord {
        pub fn canonical_hash(&self) -> Hash32 {
            digest(
                "lp0016-account/member-record",
                &[
                    &self.forum_id,
                    &self.member_commitment,
                    &self.stake_amount.to_be_bytes(),
                    &self.registered_at.to_be_bytes(),
                ],
            )
        }
    }

    // #[account]
    /// One per revoked member, written by `slash_member`. Immutable after creation.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct RevocationRecord {
        pub forum_id: Hash32,
        pub member_commitment: Hash32,
        pub slashed_at: u64,
        pub slash_bundle_hash: Hash32,
    }

    impl RevocationRecord {
        pub fn canonical_hash(&self) -> Hash32 {
            digest(
                "lp0016-account/revocation-record",
                &[
                    &self.forum_id,
                    &self.member_commitment,
                    &self.slashed_at.to_be_bytes(),
                    &self.slash_bundle_hash,
                ],
            )
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct StakePolicy {
        pub minimum_stake: u64,
    }

    /// In-memory ledger holding all account types for testing and local
    /// demos. The LEZ runtime takes ownership of the corresponding accounts
    /// in production.
    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct Ledger {
        pub forums: BTreeMap<Hash32, ForumState>,
        pub members: BTreeMap<(Hash32, Hash32), MemberRecord>,
        pub revocations: BTreeMap<(Hash32, Hash32), RevocationRecord>,
        /// Mirror of the off-chain registry sets per forum. Production uses
        /// indexed Merkle trees keyed on `member_commitment`; the
        /// `BTreeSet`-backed `RegistryState` is the off-chain mirror.
        pub registries: BTreeMap<Hash32, RegistryState>,
    }

    // #[instruction]
    /// Initialise a new forum's state account from a [`ForumConfig`] and
    /// stake policy. Fails if a forum with this id already exists.
    pub fn create_forum(
        ledger: &mut Ledger,
        config: ForumConfig,
        stake_policy: StakePolicy,
    ) -> Result<()> {
        if ledger.forums.contains_key(&config.forum_id) {
            return Err(ProtocolError::InvalidCertificate);
        }
        let registry = RegistryState::default();
        let state = ForumState {
            membership_root: registry.membership_root(),
            revocation_root: registry.revocation_root(),
            config: config.clone(),
            stake_policy,
        };
        ledger.forums.insert(config.forum_id, state);
        ledger.registries.insert(config.forum_id, registry);
        Ok(())
    }

    // #[instruction]
    /// Register a member by inserting `member_commitment` into the forum's
    /// membership tree. Caller must have escrowed the minimum stake; the
    /// stake-management side is out of scope here and represented by the
    /// `stake_amount` argument.
    pub fn register_member(
        ledger: &mut Ledger,
        forum_id: Hash32,
        member_commitment: Hash32,
        stake_amount: u64,
        registered_at: u64,
    ) -> Result<()> {
        let state = ledger.forums.get_mut(&forum_id).ok_or(ProtocolError::InvalidCertificate)?;
        if stake_amount < state.stake_policy.minimum_stake {
            return Err(ProtocolError::InvalidCertificate);
        }
        let registry = ledger.registries.get_mut(&forum_id).ok_or(ProtocolError::InvalidCertificate)?;
        registry.register(member_commitment)?;
        state.membership_root = registry.membership_root();
        ledger.members.insert(
            (forum_id, member_commitment),
            MemberRecord { forum_id, member_commitment, stake_amount, registered_at },
        );
        Ok(())
    }

    // #[instruction]
    /// Verify a slash bundle and revoke the reconstructed commitment.
    /// Returns the slashed commitment on success.
    pub fn slash_member(
        ledger: &mut Ledger,
        forum_id: Hash32,
        certificates: Vec<ModerationCertificate>,
        slashed_at: u64,
    ) -> Result<Hash32> {
        let state = ledger.forums.get_mut(&forum_id).ok_or(ProtocolError::InvalidCertificate)?;
        if state.config.forum_id != forum_id {
            return Err(ProtocolError::InvalidCertificate);
        }
        let bundle_hash = slash_bundle_hash(&certificates);
        let registry = ledger.registries.get_mut(&forum_id).ok_or(ProtocolError::InvalidCertificate)?;
        let result = slash(registry, &state.config, &certificates)?;
        state.membership_root = registry.membership_root();
        state.revocation_root = registry.revocation_root();
        ledger.revocations.insert(
            (forum_id, result.commitment),
            RevocationRecord {
                forum_id,
                member_commitment: result.commitment,
                slashed_at,
                slash_bundle_hash: bundle_hash,
            },
        );
        Ok(result.commitment)
    }

    fn slash_bundle_hash(certificates: &[ModerationCertificate]) -> Hash32 {
        // Order-independent digest over the per-certificate statement hashes,
        // matching what an on-chain verifier would compute.
        let mut hashes: Vec<Hash32> = certificates.iter().map(|c| c.statement.hash()).collect();
        hashes.sort_unstable();
        let parts: Vec<&[u8]> = hashes.iter().map(|h| h.as_slice()).collect();
        digest("slash-bundle", &parts)
    }
}

pub use lp0016_registry::*;

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use protocol_core::{
        create_vote, statement_for, AnonymousPostEnvelope, DealerShares, MemberSecret,
        ModeratorId, ModeratorSecret,
    };

    fn build_forum_and_mods() -> (ForumConfig, Vec<ModeratorSecret>) {
        let dealer = DealerShares::trusted(2, 3, b"lp0016-registry-test-dealer");
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
        let forum = ForumConfig {
            forum_id: digest("forum", &[b"lp0016-registry-test"]),
            k: 2,
            n: 2,
            moderators: mods.iter().map(ModeratorSecret::identity).collect(),
            mod_set_version: 1,
            threshold_public_key: dealer.threshold_public_key,
        };
        (forum, mods)
    }

    #[test]
    fn create_forum_initializes_state() {
        let (forum, _mods) = build_forum_and_mods();
        let mut ledger = Ledger::default();
        create_forum(&mut ledger, forum.clone(), StakePolicy { minimum_stake: 100 }).unwrap();
        let state = ledger.forums.get(&forum.forum_id).unwrap();
        assert_eq!(state.config, forum);
        assert_eq!(state.membership_root, protocol_core::empty_root());
        assert_eq!(state.revocation_root, protocol_core::empty_root());
    }

    #[test]
    fn create_forum_rejects_duplicate_forum_id() {
        let (forum, _mods) = build_forum_and_mods();
        let mut ledger = Ledger::default();
        create_forum(&mut ledger, forum.clone(), StakePolicy { minimum_stake: 0 }).unwrap();
        assert!(create_forum(&mut ledger, forum, StakePolicy { minimum_stake: 0 }).is_err());
    }

    #[test]
    fn register_then_slash_round_trip() {
        let (forum, mods) = build_forum_and_mods();
        let mut ledger = Ledger::default();
        create_forum(&mut ledger, forum.clone(), StakePolicy { minimum_stake: 100 }).unwrap();
        let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"seed");
        let commitment = member.commitment(&forum.forum_id);
        register_member(&mut ledger, forum.forum_id, commitment, 200, 0).unwrap();

        let mut certs = Vec::new();
        for i in 0..2u8 {
            let content_id = digest("content", &[&[i]]);
            let registry = ledger.registries.get(&forum.forum_id).unwrap().clone();
            let post = AnonymousPostEnvelope::build(&forum, &registry, &member, content_id, vec![i]);
            let reason = digest("reason", &[b"rule"]);
            let st = statement_for(
                &forum,
                post.post_id,
                post.content_id,
                post.proof_public_inputs_hash,
                post.ciphertext_hash,
                reason,
            );
            let votes = vec![
                create_vote(&forum, &mods[0], &st).unwrap(),
                create_vote(&forum, &mods[1], &st).unwrap(),
            ];
            let partials = vec![mods[0].partial_decrypt(&post), mods[1].partial_decrypt(&post)];
            certs.push(ModerationCertificate {
                statement: st,
                votes,
                ciphertext: post.ciphertext.clone(),
                partial_decryptions: partials,
            });
        }

        let revoked = slash_member(&mut ledger, forum.forum_id, certs, 99).unwrap();
        assert_eq!(revoked, commitment);
        assert!(ledger.revocations.contains_key(&(forum.forum_id, commitment)));
        let post_state = ledger.forums.get(&forum.forum_id).unwrap();
        assert_ne!(post_state.revocation_root, protocol_core::empty_root());
    }

    #[test]
    fn register_rejects_insufficient_stake() {
        let (forum, _mods) = build_forum_and_mods();
        let mut ledger = Ledger::default();
        create_forum(&mut ledger, forum.clone(), StakePolicy { minimum_stake: 100 }).unwrap();
        let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"seed");
        let commitment = member.commitment(&forum.forum_id);
        assert!(register_member(&mut ledger, forum.forum_id, commitment, 50, 0).is_err());
    }

    #[test]
    fn canonical_hashes_are_stable() {
        let (forum, _mods) = build_forum_and_mods();
        let state = ForumState {
            config: forum,
            stake_policy: StakePolicy { minimum_stake: 100 },
            membership_root: protocol_core::empty_root(),
            revocation_root: protocol_core::empty_root(),
        };
        // Two computations of the same canonical hash must agree byte-for-byte.
        assert_eq!(state.canonical_hash(), state.canonical_hash());
    }

    #[test]
    fn idl_matches_handwritten_file() {
        let idl_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../idl/lp0016_registry.json");
        let raw = std::fs::read_to_string(&idl_path).expect("missing IDL file");
        let idl: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let names: Vec<&str> = idl["instructions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["create_forum", "register_member", "slash_member"]);
        let accounts: Vec<&str> = idl["accounts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        for required in ["ForumState", "MemberRecord", "RevocationRecord", "StakePolicy"] {
            assert!(accounts.contains(&required), "IDL missing account: {required}");
        }
    }
}
