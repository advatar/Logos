use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{digest, ForumConfig, Hash32, ModeratorId, ProtocolError, Result, Share};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateStatement {
    pub forum_id: Hash32,
    pub post_id: Hash32,
    pub content_id: Hash32,
    pub proof_public_inputs_hash: Hash32,
    pub ciphertext_hash: Hash32,
    pub reason_hash: Hash32,
    pub mod_set_version: u64,
    pub k: u8,
    pub n: u8,
}

impl CertificateStatement {
    pub fn hash(&self) -> Hash32 {
        digest(
            "certificate-statement",
            &[
                &self.forum_id,
                &self.post_id,
                &self.content_id,
                &self.proof_public_inputs_hash,
                &self.ciphertext_hash,
                &self.reason_hash,
                &self.mod_set_version.to_be_bytes(),
                &[self.k],
                &[self.n],
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationVote {
    pub moderator_id: ModeratorId,
    pub statement_hash: Hash32,
    /// Development placeholder. Production uses Ed25519 signature bytes.
    pub signature: Hash32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationCertificate {
    pub statement: CertificateStatement,
    pub votes: Vec<ModerationVote>,
    pub revealed_share: Share,
}

pub fn dev_sign(moderator_id: &ModeratorId, statement_hash: &Hash32) -> Hash32 {
    digest("dev-moderator-signature", &[moderator_id.0.as_bytes(), statement_hash])
}

pub fn statement_for(
    forum: &ForumConfig,
    post_id: Hash32,
    content_id: Hash32,
    proof_public_inputs_hash: Hash32,
    ciphertext_hash: Hash32,
    reason_hash: Hash32,
) -> CertificateStatement {
    CertificateStatement {
        forum_id: forum.forum_id,
        post_id,
        content_id,
        proof_public_inputs_hash,
        ciphertext_hash,
        reason_hash,
        mod_set_version: forum.mod_set_version,
        k: forum.k,
        n: forum.n,
    }
}

pub fn create_vote(forum: &ForumConfig, moderator_id: ModeratorId, statement: &CertificateStatement) -> Result<ModerationVote> {
    if !forum.moderators.contains(&moderator_id) {
        return Err(ProtocolError::InvalidModerator);
    }
    let statement_hash = statement.hash();
    let signature = dev_sign(&moderator_id, &statement_hash);
    Ok(ModerationVote { moderator_id, statement_hash, signature })
}

pub fn verify_vote(forum: &ForumConfig, vote: &ModerationVote, statement_hash: &Hash32) -> bool {
    forum.moderators.contains(&vote.moderator_id)
        && &vote.statement_hash == statement_hash
        && vote.signature == dev_sign(&vote.moderator_id, statement_hash)
}

pub fn verify_certificate(forum: &ForumConfig, cert: &ModerationCertificate) -> Result<()> {
    let st = &cert.statement;
    if st.forum_id != forum.forum_id || st.k != forum.k || st.n != forum.n || st.mod_set_version != forum.mod_set_version {
        return Err(ProtocolError::InvalidCertificate);
    }
    let distinct: BTreeSet<_> = cert.votes.iter().map(|v| v.moderator_id.clone()).collect();
    if distinct.len() < forum.n as usize {
        return Err(ProtocolError::PartialCertificate);
    }
    let h = st.hash();
    for vote in &cert.votes {
        if !verify_vote(forum, vote, &h) {
            return Err(ProtocolError::InvalidVoteStatement);
        }
    }
    Ok(())
}
