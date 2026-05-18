use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    commitment_for, interpolate_coeffs, verify_certificate, AnonymousPostEnvelope, ForumConfig,
    Hash32, ModerationCertificate, ProtocolError, Result, Scalar, VerifiedZkReceipt,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryState {
    pub registered: BTreeSet<Hash32>,
    pub revoked: BTreeSet<Hash32>,
}

impl RegistryState {
    pub fn register(&mut self, commitment: Hash32) -> Result<()> {
        if self.revoked.contains(&commitment) {
            return Err(ProtocolError::AlreadyRevoked);
        }
        self.registered.insert(commitment);
        Ok(())
    }

    pub fn is_active(&self, commitment: &Hash32) -> bool {
        self.registered.contains(commitment) && !self.revoked.contains(commitment)
    }

    pub fn revoke(&mut self, commitment: Hash32) -> Result<()> {
        if !self.registered.contains(&commitment) {
            return Err(ProtocolError::UnregisteredCommitment);
        }
        self.revoked.insert(commitment);
        Ok(())
    }

    /// Merkle root over `registered`. Bound into the post public-inputs hash
    /// so the RISC0 guest (Phase 4) can verify membership without trusting the
    /// caller. The set is implicitly sorted by [`crate::root_from_set`].
    pub fn membership_root(&self) -> Hash32 {
        crate::root_from_set(self.registered.iter().copied())
    }

    /// Merkle root over `revoked`. Used the same way as
    /// [`Self::membership_root`] for non-membership proofs in the guest.
    pub fn revocation_root(&self) -> Hash32 {
        crate::root_from_set(self.revoked.iter().copied())
    }
}

pub fn verify_post(
    registry: &RegistryState,
    forum: &ForumConfig,
    post: &AnonymousPostEnvelope,
) -> Result<()> {
    if post.forum_id != forum.forum_id {
        return Err(ProtocolError::InvalidCertificate);
    }
    if post.membership_root != registry.membership_root()
        || post.revocation_root != registry.revocation_root()
    {
        return Err(ProtocolError::InvalidCertificate);
    }
    if post.ciphertext.hash() != post.ciphertext_hash {
        return Err(ProtocolError::InvalidCertificate);
    }
    match post
        .zk_receipt
        .verify_public_inputs(&post.proof_public_inputs_hash)?
    {
        VerifiedZkReceipt::Mock {
            hidden_commitment_for_local_model,
        } => {
            if !registry.is_active(&hidden_commitment_for_local_model) {
                return Err(ProtocolError::CommitmentNotActive);
            }
        }
        VerifiedZkReceipt::Risc0 => {}
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashResult {
    pub commitment: Hash32,
    pub reconstructed_coeffs: Vec<Scalar>,
}

pub fn slash(
    registry: &mut RegistryState,
    forum: &ForumConfig,
    certificates: &[ModerationCertificate],
) -> Result<SlashResult> {
    if certificates.len() != forum.k as usize {
        return Err(ProtocolError::WrongSlashCertificateCount);
    }
    let mut shares = Vec::with_capacity(certificates.len());
    for cert in certificates {
        verify_certificate(forum, cert)?;
        shares.push(cert.revealed_share(forum)?);
    }
    let coeffs = interpolate_coeffs(&shares)?;
    let commitment = commitment_for(&forum.forum_id, forum.k, &coeffs);
    if !registry.is_active(&commitment) {
        return Err(ProtocolError::CommitmentNotActive);
    }
    registry.revoke(commitment)?;
    Ok(SlashResult {
        commitment,
        reconstructed_coeffs: coeffs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_vote, digest, statement_for, DealerShares, MemberSecret, ModerationCertificate,
        ModeratorId, ModeratorSecret,
    };
    use ed25519_dalek::SigningKey;

    struct TestSetup {
        forum: ForumConfig,
        mods: Vec<ModeratorSecret>,
    }

    fn test_setup() -> TestSetup {
        let dealer = DealerShares::pedersen_dkg(2, 3, b"forum-seed");
        let names = ["alice", "bob", "carol"];
        let mods: Vec<ModeratorSecret> = names
            .iter()
            .zip(dealer.share_secret_keys.iter())
            .map(|(name, share)| {
                let seed: [u8; 32] = digest("mod-sign-seed", &[name.as_bytes()]);
                ModeratorSecret::new(
                    ModeratorId((*name).into()),
                    SigningKey::from_bytes(&seed),
                    share.clone(),
                )
            })
            .collect();
        let forum = ForumConfig {
            forum_id: digest("forum", &[b"unit"]),
            k: 2,
            n: 2,
            moderators: mods.iter().map(ModeratorSecret::identity).collect(),
            mod_set_version: 1,
            threshold_public_key: dealer.threshold_public_key,
        };
        TestSetup { forum, mods }
    }

    fn build_cert(
        setup: &TestSetup,
        registry: &RegistryState,
        member: &MemberSecret,
        idx: u8,
    ) -> (AnonymousPostEnvelope, ModerationCertificate) {
        let content_id = digest("content", &[&[idx]]);
        let post =
            AnonymousPostEnvelope::build(&setup.forum, registry, member, content_id, vec![idx]);
        let reason = digest("reason", &[b"rule"]);
        let st = statement_for(
            &setup.forum,
            post.post_id,
            post.content_id,
            post.proof_public_inputs_hash,
            post.ciphertext_hash,
            reason,
        );
        let votes = vec![
            create_vote(&setup.forum, &setup.mods[0], &st).unwrap(),
            create_vote(&setup.forum, &setup.mods[1], &st).unwrap(),
        ];
        let partials = vec![
            setup.mods[0].partial_decrypt(&post),
            setup.mods[1].partial_decrypt(&post),
        ];
        let cert = ModerationCertificate {
            statement: st,
            votes,
            ciphertext: post.ciphertext.clone(),
            partial_decryptions: partials,
        };
        (post, cert)
    }

    #[test]
    fn slash_revokes_member() {
        let setup = test_setup();
        let member = MemberSecret::from_seed(&setup.forum.forum_id, setup.forum.k, b"seed");
        let mut registry = RegistryState::default();
        registry
            .register(member.commitment(&setup.forum.forum_id))
            .unwrap();

        let (_post0, cert0) = build_cert(&setup, &registry, &member, 0);
        let (_post1, cert1) = build_cert(&setup, &registry, &member, 1);
        verify_certificate(&setup.forum, &cert0).unwrap();
        verify_certificate(&setup.forum, &cert1).unwrap();
        let result = slash(&mut registry, &setup.forum, &[cert0, cert1]).unwrap();
        assert!(registry.revoked.contains(&result.commitment));
    }

    #[test]
    fn cross_forum_certificate_is_rejected() {
        let setup = test_setup();
        let mut forum_b = setup.forum.clone();
        forum_b.forum_id = digest("forum", &[b"other"]);
        let member = MemberSecret::from_seed(&setup.forum.forum_id, setup.forum.k, b"seed");
        let registry = RegistryState::default();
        let (_post, cert) = build_cert(&setup, &registry, &member, 0);
        assert_eq!(
            verify_certificate(&forum_b, &cert).unwrap_err(),
            ProtocolError::InvalidCertificate
        );
    }

    #[test]
    fn cross_mod_set_version_rejected() {
        let setup = test_setup();
        let mut bumped = setup.forum.clone();
        bumped.mod_set_version += 1;
        let member = MemberSecret::from_seed(&setup.forum.forum_id, setup.forum.k, b"seed");
        let registry = RegistryState::default();
        let (_post, cert) = build_cert(&setup, &registry, &member, 0);
        assert_eq!(
            verify_certificate(&bumped, &cert).unwrap_err(),
            ProtocolError::InvalidCertificate
        );
    }

    #[test]
    fn registry_roots_change_on_register_and_revoke() {
        let mut registry = RegistryState::default();
        let empty = registry.membership_root();
        assert_eq!(empty, crate::empty_root());
        registry.register(digest("c", &[b"1"])).unwrap();
        assert_ne!(registry.membership_root(), empty);
        let before = registry.revocation_root();
        registry.revoke(digest("c", &[b"1"])).unwrap();
        assert_ne!(registry.revocation_root(), before);
    }
}
