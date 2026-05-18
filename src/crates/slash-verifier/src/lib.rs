//! Library half of the slash-verifier CLI. The CLI itself is in `main.rs`.

use anyhow::{anyhow, Result};
use protocol_core::{slash, ForumConfig, Hash32, ModerationCertificate, RegistryState, SlashResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub forum: ForumConfig,
    pub registry: RegistryState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashBundleFile {
    pub certificates: Vec<ModerationCertificate>,
}

/// Run the slash verification pipeline against a parsed snapshot + bundle.
///
/// Returns the [`SlashResult`] on success; does not mutate the input snapshot
/// (the caller's registry is cloned internally so this is safe to call on
/// archived snapshots).
pub fn verify(snapshot: &RegistrySnapshot, bundle: &SlashBundleFile) -> Result<SlashResult> {
    let k = snapshot.forum.k as usize;
    if bundle.certificates.len() != k {
        return Err(anyhow!(
            "bundle has {} certificates but forum K = {}",
            bundle.certificates.len(),
            snapshot.forum.k
        ));
    }
    let mut registry = snapshot.registry.clone();
    slash(&mut registry, &snapshot.forum, &bundle.certificates)
        .map_err(|e| anyhow!("slash verification failed: {e}"))
}

/// Convenience: take the recovered commitment as 32 bytes.
pub fn revoked_commitment(result: &SlashResult) -> Hash32 {
    result.commitment
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol_core::{
        create_vote, digest, statement_for, AnonymousPostEnvelope, DevThresholdOracle, MemberSecret,
        ModerationCertificate, ModeratorId, ModeratorSecret,
    };

    fn moderator(name: &str) -> ModeratorSecret {
        let seed: [u8; 32] = digest("mod-seed", &[name.as_bytes()]);
        ModeratorSecret::from_seed(ModeratorId(name.to_string()), &seed)
    }

    fn build_snapshot_and_bundle() -> (RegistrySnapshot, SlashBundleFile) {
        let mods = [moderator("alice"), moderator("bob"), moderator("carol")];
        let forum = ForumConfig {
            forum_id: digest("forum", &[b"slash-verifier-test"]),
            k: 2,
            n: 2,
            moderators: mods.iter().map(ModeratorSecret::identity).collect(),
            mod_set_version: 1,
            threshold_public_key_hash: digest("threshold-pk", &[b"slash-verifier-test"]),
        };
        let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"seed");
        let mut registry = RegistryState::default();
        registry.register(member.commitment(&forum.forum_id)).unwrap();
        let mut oracle = DevThresholdOracle::default();

        let mut certs = Vec::new();
        for i in 0..2u8 {
            let content_id = digest("content", &[&[i]]);
            let (post, share) = AnonymousPostEnvelope::new_dev(&forum, &member, content_id, vec![i]);
            oracle.remember(post.ciphertext_hash, share);
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
            certs.push(ModerationCertificate {
                statement: st,
                votes,
                revealed_share: oracle.decrypt(&post.ciphertext_hash).unwrap(),
            });
        }
        (
            RegistrySnapshot { forum, registry },
            SlashBundleFile { certificates: certs },
        )
    }

    #[test]
    fn verify_accepts_valid_bundle() {
        let (snapshot, bundle) = build_snapshot_and_bundle();
        let result = verify(&snapshot, &bundle).unwrap();
        assert!(snapshot.registry.is_active(&result.commitment));
    }

    #[test]
    fn verify_rejects_wrong_k() {
        let (snapshot, mut bundle) = build_snapshot_and_bundle();
        bundle.certificates.pop();
        assert!(verify(&snapshot, &bundle).is_err());
    }

    #[test]
    fn json_round_trip() {
        let (snapshot, bundle) = build_snapshot_and_bundle();
        let s_json = serde_json::to_string(&snapshot).unwrap();
        let b_json = serde_json::to_string(&bundle).unwrap();
        let s2: RegistrySnapshot = serde_json::from_str(&s_json).unwrap();
        let b2: SlashBundleFile = serde_json::from_str(&b_json).unwrap();
        assert!(verify(&s2, &b2).is_ok());
    }
}
