use std::collections::BTreeSet;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{
    aggregate_decrypt, decode_share, digest, dleq_domain_seed_for, verify_partial,
    AnonymousPostEnvelope, Ciphertext, ForumConfig, Hash32, ModeratorId, ModeratorIdentity,
    PartialDecryption, ProtocolError, Result, Share, SharePublicKey, ShareSecretKey,
};

/// 64-byte Ed25519 signature serialized as a fixed array for canonical bytes.
type SignatureBytes = [u8; 64];

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
    pub threshold_public_key_hash: Hash32,
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
                &self.threshold_public_key_hash,
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationVote {
    pub moderator_id: ModeratorId,
    pub statement_hash: Hash32,
    /// 64-byte Ed25519 signature over `statement_hash`.
    #[serde(with = "serde_signature_bytes")]
    pub signature: SignatureBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationCertificate {
    pub statement: CertificateStatement,
    pub votes: Vec<ModerationVote>,
    pub ciphertext: Ciphertext,
    pub partial_decryptions: Vec<PartialDecryption>,
}

impl ModerationCertificate {
    /// Aggregate the partial decryptions and recover the encoded Shamir share.
    /// Assumes [`verify_certificate`] has already validated the partials.
    pub fn revealed_share(&self, forum: &ForumConfig) -> Result<Share> {
        let pk_shares: Vec<SharePublicKey> = self
            .partial_decryptions
            .iter()
            .map(|pd| {
                forum
                    .share_public_key(pd.idx)
                    .copied()
                    .ok_or(ProtocolError::InvalidModerator)
            })
            .collect::<Result<_>>()?;
        let plaintext = aggregate_decrypt(&self.ciphertext, &self.partial_decryptions, &pk_shares)?;
        decode_share(&plaintext)
    }
}

/// A moderator's local signing + threshold-share secret material.
#[derive(Debug)]
pub struct ModeratorSecret {
    pub id: ModeratorId,
    signing_key: SigningKey,
    pub share_secret_key: ShareSecretKey,
}

impl ModeratorSecret {
    pub fn new(id: ModeratorId, signing_key: SigningKey, share_secret_key: ShareSecretKey) -> Self {
        Self {
            id,
            signing_key,
            share_secret_key,
        }
    }

    pub fn from_seed_and_share(
        id: ModeratorId,
        seed: &[u8; 32],
        share_secret_key: ShareSecretKey,
    ) -> Self {
        Self {
            id,
            signing_key: SigningKey::from_bytes(seed),
            share_secret_key,
        }
    }

    pub fn identity(&self) -> ModeratorIdentity {
        ModeratorIdentity {
            id: self.id.clone(),
            verifying_key: self.signing_key.verifying_key().to_bytes(),
            share_public_key: self.share_secret_key.public(),
        }
    }

    fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Produce a partial decryption of the post's ciphertext, with a DLEQ
    /// proof bound to the post's domain seed.
    pub fn partial_decrypt(&self, post: &AnonymousPostEnvelope) -> PartialDecryption {
        let pk_share = self.share_secret_key.public();
        let domain_seed = post.dleq_domain_seed();
        crate::partial_decrypt(
            &self.share_secret_key,
            &post.ciphertext,
            &pk_share,
            &domain_seed,
        )
    }
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
        threshold_public_key_hash: forum.threshold_public_key_hash(),
    }
}

pub fn create_vote(
    forum: &ForumConfig,
    moderator: &ModeratorSecret,
    statement: &CertificateStatement,
) -> Result<ModerationVote> {
    let identity = moderator.identity();
    if !forum.moderators.iter().any(|m| m == &identity) {
        return Err(ProtocolError::InvalidModerator);
    }
    let statement_hash = statement.hash();
    let signature = moderator.sign(&statement_hash);
    Ok(ModerationVote {
        moderator_id: moderator.id.clone(),
        statement_hash,
        signature: signature.to_bytes(),
    })
}

pub fn verify_vote(forum: &ForumConfig, vote: &ModerationVote, statement_hash: &Hash32) -> bool {
    if &vote.statement_hash != statement_hash {
        return false;
    }
    let Some(identity) = forum.find_moderator(&vote.moderator_id) else {
        return false;
    };
    let Ok(vk) = VerifyingKey::from_bytes(&identity.verifying_key) else {
        return false;
    };
    let signature = Signature::from_bytes(&vote.signature);
    vk.verify(statement_hash, &signature).is_ok()
}

pub fn verify_certificate(forum: &ForumConfig, cert: &ModerationCertificate) -> Result<()> {
    let st = &cert.statement;
    if st.forum_id != forum.forum_id
        || st.k != forum.k
        || st.n != forum.n
        || st.mod_set_version != forum.mod_set_version
        || st.threshold_public_key_hash != forum.threshold_public_key_hash()
    {
        return Err(ProtocolError::InvalidCertificate);
    }
    if cert.ciphertext.hash() != st.ciphertext_hash {
        return Err(ProtocolError::InvalidCertificate);
    }
    let distinct: BTreeSet<_> = cert.votes.iter().map(|v| v.moderator_id.clone()).collect();
    if distinct.len() < forum.n as usize {
        return Err(ProtocolError::PartialCertificate);
    }
    if cert.partial_decryptions.len() < forum.n as usize {
        return Err(ProtocolError::PartialCertificate);
    }
    let h = st.hash();
    for vote in &cert.votes {
        if !verify_vote(forum, vote, &h) {
            return Err(ProtocolError::InvalidVoteStatement);
        }
    }
    let domain_seed = dleq_domain_seed_for(&st.forum_id, &st.post_id);
    let mut seen_idx = BTreeSet::new();
    for pd in &cert.partial_decryptions {
        let pk_share = forum
            .share_public_key(pd.idx)
            .ok_or(ProtocolError::InvalidModerator)?;
        if !verify_partial(pd, &cert.ciphertext, pk_share, &domain_seed) {
            return Err(ProtocolError::InvalidCertificate);
        }
        if !seen_idx.insert(pd.idx) {
            return Err(ProtocolError::InvalidCertificate);
        }
    }
    Ok(())
}

mod serde_signature_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        bytes: &[u8; 64],
        ser: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        if ser.is_human_readable() {
            ser.serialize_str(&hex::encode(bytes))
        } else {
            bytes.serialize(ser)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        de: D,
    ) -> std::result::Result<[u8; 64], D::Error> {
        use serde::de::Error;
        if de.is_human_readable() {
            let s = String::deserialize(de)?;
            let raw = hex::decode(&s).map_err(D::Error::custom)?;
            raw.try_into()
                .map_err(|_| D::Error::custom("signature must be 64 bytes"))
        } else {
            let raw = <Vec<u8>>::deserialize(de)?;
            raw.try_into()
                .map_err(|_| D::Error::custom("signature must be 64 bytes"))
        }
    }
}
