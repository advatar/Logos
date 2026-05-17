use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    commitment_for, interpolate_coeffs, verify_certificate, AnonymousPostEnvelope, ForumConfig,
    Hash32, ModerationCertificate, ProtocolError, Result, Share,
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
}

#[derive(Debug, Clone, Default)]
pub struct DevThresholdOracle {
    shares: HashMap<Hash32, Share>,
}

impl DevThresholdOracle {
    pub fn remember(&mut self, ciphertext_hash: Hash32, share: Share) {
        self.shares.insert(ciphertext_hash, share);
    }

    pub fn decrypt(&self, ciphertext_hash: &Hash32) -> Option<Share> {
        self.shares.get(ciphertext_hash).copied()
    }
}

pub fn verify_post(registry: &RegistryState, forum: &ForumConfig, post: &AnonymousPostEnvelope) -> Result<()> {
    if post.forum_id != forum.forum_id {
        return Err(ProtocolError::InvalidCertificate);
    }
    if post.zk_receipt.public_inputs_hash != post.proof_public_inputs_hash || !post.zk_receipt.valid {
        return Err(ProtocolError::InvalidCertificate);
    }
    if !registry.is_active(&post.zk_receipt.hidden_commitment_for_local_model) {
        return Err(ProtocolError::CommitmentNotActive);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashResult {
    pub commitment: Hash32,
    pub reconstructed_coeffs: Vec<crate::F>,
}

pub fn slash(registry: &mut RegistryState, forum: &ForumConfig, certificates: &[ModerationCertificate]) -> Result<SlashResult> {
    if certificates.len() != forum.k as usize {
        return Err(ProtocolError::WrongSlashCertificateCount);
    }
    for cert in certificates {
        verify_certificate(forum, cert)?;
    }
    let shares: Vec<_> = certificates.iter().map(|c| c.revealed_share).collect();
    let coeffs = interpolate_coeffs(&shares)?;
    let commitment = commitment_for(&forum.forum_id, forum.k, &coeffs);
    if !registry.is_active(&commitment) {
        return Err(ProtocolError::CommitmentNotActive);
    }
    registry.revoke(commitment)?;
    Ok(SlashResult { commitment, reconstructed_coeffs: coeffs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_vote, digest, statement_for, AnonymousPostEnvelope, MemberSecret, ModerationCertificate, ModeratorId};

    fn forum() -> ForumConfig {
        ForumConfig {
            forum_id: digest("forum", &[b"unit"]),
            k: 2,
            n: 2,
            moderators: vec![ModeratorId("alice".into()), ModeratorId("bob".into()), ModeratorId("carol".into())],
            mod_set_version: 1,
        }
    }

    #[test]
    fn slash_revokes_member() {
        let forum = forum();
        let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"seed");
        let mut registry = RegistryState::default();
        registry.register(member.commitment(&forum.forum_id)).unwrap();
        let mut oracle = DevThresholdOracle::default();

        let mut certs = Vec::new();
        for i in 0..2u8 {
            let content_id = digest("content", &[&[i]]);
            let (post, share) = AnonymousPostEnvelope::new_dev(&forum, &member, content_id, vec![i]);
            oracle.remember(post.ciphertext_hash, share);
            verify_post(&registry, &forum, &post).unwrap();
            let reason = digest("reason", &[b"rule"]);
            let st = statement_for(&forum, post.post_id, post.content_id, post.proof_public_inputs_hash, post.ciphertext_hash, reason);
            let votes = vec![
                create_vote(&forum, ModeratorId("alice".into()), &st).unwrap(),
                create_vote(&forum, ModeratorId("bob".into()), &st).unwrap(),
            ];
            certs.push(ModerationCertificate { statement: st, votes, revealed_share: oracle.decrypt(&post.ciphertext_hash).unwrap() });
        }

        let result = slash(&mut registry, &forum, &certs).unwrap();
        assert!(registry.revoked.contains(&result.commitment));
    }
}
