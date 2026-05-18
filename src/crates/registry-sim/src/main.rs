use ed25519_dalek::SigningKey;
use moderation_sdk::{ForumSdk, MemoryStore};
use protocol_core::*;
use slash_verifier::{RegistrySnapshot, SlashBundleFile};

fn build_moderator(name: &str, share: ShareSecretKey) -> ModeratorSecret {
    let seed: [u8; 32] = digest("mod-sign-seed", &[name.as_bytes()]);
    ModeratorSecret::new(
        ModeratorId(name.to_string()),
        SigningKey::from_bytes(&seed),
        share,
    )
}

fn main() -> anyhow::Result<()> {
    let dealer = DealerShares::pedersen_dkg(2, 3, b"registry-sim-dealer");
    let names = ["alice", "bob", "carol"];
    let mods: Vec<ModeratorSecret> = names
        .iter()
        .zip(dealer.share_secret_keys.iter())
        .map(|(n, sk)| build_moderator(n, sk.clone()))
        .collect();

    let forum = ForumConfig {
        forum_id: digest("forum", &[b"registry-sim"]),
        k: 2,
        n: 2,
        moderators: mods.iter().map(ModeratorSecret::identity).collect(),
        mod_set_version: 1,
        threshold_public_key: dealer.threshold_public_key,
    };
    let mut sdk = ForumSdk::new(forum, MemoryStore::default())?;
    let member = MemberSecret::from_seed(&sdk.forum.forum_id, sdk.forum.k, b"member-seed");
    let commitment = sdk.register_member(&member)?;
    println!("registered commitment {}…", hex8(&commitment));

    let mut certs = Vec::new();
    for i in 0..2u8 {
        let post = sdk.build_post(&member, digest("content", &[&[i]]), vec![i])?;
        sdk.persist_post(&post)?;
        let reason = digest("reason", &[b"rule"]);
        let votes = vec![
            sdk.create_moderation_vote(&mods[0], &post, reason)?,
            sdk.create_moderation_vote(&mods[1], &post, reason)?,
        ];
        let partials = vec![
            mods[0].partial_decrypt(&post),
            mods[1].partial_decrypt(&post),
        ];
        certs.push(sdk.aggregate_certificate(&post, reason, votes, partials)?);
    }

    if let Ok(dir) = std::env::var("LP0016_SIM_JSON_DIR") {
        std::fs::create_dir_all(&dir)?;
        let snapshot = RegistrySnapshot {
            forum: sdk.forum.clone(),
            registry: sdk.registry.clone(),
        };
        let bundle = SlashBundleFile {
            certificates: certs.clone(),
        };
        std::fs::write(
            std::path::Path::new(&dir).join("registry.json"),
            serde_json::to_vec_pretty(&snapshot)?,
        )?;
        std::fs::write(
            std::path::Path::new(&dir).join("bundle.json"),
            serde_json::to_vec_pretty(&bundle)?,
        )?;
    }

    let slash = sdk.submit_slash(&certs)?;
    println!(
        "slash reconstructed and revoked {}…",
        hex8(&slash.commitment)
    );
    Ok(())
}
