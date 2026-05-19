#![no_main]

use borsh::{BorshDeserialize, BorshSerialize};
use lez_framework::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

type Hash32 = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
struct ForumStateWire {
    forum_id: Hash32,
    k: u8,
    n: u8,
    mod_set_version: u64,
    moderator_count: u16,
    threshold_public_key_hash: Hash32,
    minimum_stake: u64,
    membership_root: Hash32,
    revocation_root: Hash32,
    registered_count: u64,
    revoked_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
struct MemberRecordWire {
    forum_id: Hash32,
    member_commitment: Hash32,
    stake_amount: u64,
    registered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
struct RevocationRecordWire {
    forum_id: Hash32,
    member_commitment: Hash32,
    slashed_at: u64,
    slash_bundle_hash: Hash32,
}

fn digest(domain: &str, parts: &[&[u8]]) -> Hash32 {
    let mut h = Sha256::new();
    h.update(b"lp0016-lez-registry:v1:");
    h.update(domain.as_bytes());
    for part in parts {
        let len = (part.len() as u32).to_be_bytes();
        h.update(len);
        h.update(part);
    }
    h.finalize().into()
}

fn decode<T: BorshDeserialize>(account: &AccountWithMetadata, index: usize) -> Result<T, LezError> {
    borsh::from_slice(account.account.data.as_ref()).map_err(|e| LezError::DeserializationError {
        account_index: index,
        message: e.to_string(),
    })
}

fn encode<T: BorshSerialize>(value: &T) -> Result<nssa_core::account::Data, LezError> {
    borsh::to_vec(value)
        .map_err(|e| LezError::SerializationError {
            message: e.to_string(),
        })?
        .try_into()
        .map_err(
            |e: nssa_core::account::data::DataTooBigError| LezError::SerializationError {
                message: e.to_string(),
            },
        )
}

fn post_owned(
    mut account: AccountWithMetadata,
    data: nssa_core::account::Data,
) -> AccountPostState {
    account.account.data = data;
    AccountPostState::new_claimed_if_default(account.account)
}

fn next_root(domain: &str, prior: &Hash32, value: &Hash32, count: u64) -> Hash32 {
    digest(domain, &[prior, value, &count.to_be_bytes()])
}

#[lez_program]
mod lp0016_registry {
    #[allow(unused_imports)]
    use super::*;

    #[instruction]
    pub fn create_forum(
        #[account(init, pda = [literal("forum"), arg("forum_id")])]
        forum_state: AccountWithMetadata,
        #[account(signer)] authority: AccountWithMetadata,
        forum_id: Hash32,
        k: u8,
        n: u8,
        moderator_count: u16,
        mod_set_version: u64,
        threshold_public_key_hash: Hash32,
        minimum_stake: u64,
    ) -> LezResult {
        if k == 0 || n == 0 || u16::from(n) > moderator_count {
            return Err(LezError::custom(1, "invalid LP-0016 threshold parameters"));
        }

        let empty_root = digest("empty-root", &[&forum_id]);
        let state = ForumStateWire {
            forum_id,
            k,
            n,
            mod_set_version,
            moderator_count,
            threshold_public_key_hash,
            minimum_stake,
            membership_root: empty_root,
            revocation_root: empty_root,
            registered_count: 0,
            revoked_count: 0,
        };

        Ok(LezOutput::states_only(vec![
            post_owned(forum_state, encode(&state)?),
            AccountPostState::new(authority.account.clone()),
        ]))
    }

    #[instruction]
    pub fn register_member(
        #[account(mut, pda = [literal("forum"), arg("forum_id")])] forum_state: AccountWithMetadata,
        #[account(init, pda = [literal("member"), arg("forum_id"), arg("member_commitment")])]
        member_record: AccountWithMetadata,
        #[account(signer)] member_authority: AccountWithMetadata,
        forum_id: Hash32,
        member_commitment: Hash32,
        stake_amount: u64,
        registered_at: u64,
    ) -> LezResult {
        let mut state: ForumStateWire = decode(&forum_state, 0)?;
        if state.forum_id != forum_id {
            return Err(LezError::custom(2, "forum account does not match forum_id"));
        }
        if stake_amount < state.minimum_stake {
            return Err(LezError::InsufficientBalance {
                available: stake_amount as u128,
                requested: state.minimum_stake as u128,
            });
        }

        state.registered_count =
            state
                .registered_count
                .checked_add(1)
                .ok_or_else(|| LezError::Overflow {
                    operation: "registered_count + 1".to_string(),
                })?;
        state.membership_root = next_root(
            "membership-root",
            &state.membership_root,
            &member_commitment,
            state.registered_count,
        );

        let record = MemberRecordWire {
            forum_id,
            member_commitment,
            stake_amount,
            registered_at,
        };

        Ok(LezOutput::states_only(vec![
            post_owned(forum_state, encode(&state)?),
            post_owned(member_record, encode(&record)?),
            AccountPostState::new(member_authority.account.clone()),
        ]))
    }

    #[instruction]
    pub fn slash_member(
        #[account(mut, pda = [literal("forum"), arg("forum_id")])] forum_state: AccountWithMetadata,
        #[account(mut, pda = [literal("member"), arg("forum_id"), arg("member_commitment")])]
        member_record: AccountWithMetadata,
        #[account(init, pda = [literal("revoked"), arg("forum_id"), arg("member_commitment")])]
        revocation_record: AccountWithMetadata,
        #[account(signer)] submitter: AccountWithMetadata,
        forum_id: Hash32,
        member_commitment: Hash32,
        slash_bundle_hash: Hash32,
        slashed_at: u64,
    ) -> LezResult {
        let mut state: ForumStateWire = decode(&forum_state, 0)?;
        let member: MemberRecordWire = decode(&member_record, 1)?;
        if state.forum_id != forum_id || member.forum_id != forum_id {
            return Err(LezError::custom(3, "forum mismatch in slash"));
        }
        if member.member_commitment != member_commitment {
            return Err(LezError::custom(4, "member commitment mismatch in slash"));
        }
        if slash_bundle_hash == [0u8; 32] {
            return Err(LezError::custom(5, "slash bundle hash is required"));
        }

        state.revoked_count =
            state
                .revoked_count
                .checked_add(1)
                .ok_or_else(|| LezError::Overflow {
                    operation: "revoked_count + 1".to_string(),
                })?;
        state.revocation_root = next_root(
            "revocation-root",
            &state.revocation_root,
            &member_commitment,
            state.revoked_count,
        );

        let record = RevocationRecordWire {
            forum_id,
            member_commitment,
            slashed_at,
            slash_bundle_hash,
        };

        Ok(LezOutput::states_only(vec![
            post_owned(forum_state, encode(&state)?),
            AccountPostState::new(member_record.account.clone()),
            post_owned(revocation_record, encode(&record)?),
            AccountPostState::new(submitter.account.clone()),
        ]))
    }
}
