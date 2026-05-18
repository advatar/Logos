use moderation_sdk::{ForumSdk, MemoryStore};
use protocol_core::*;

fn moderator(name: &str) -> ModeratorSecret {
    let seed: [u8; 32] = digest("mod-seed", &[name.as_bytes()]);
    ModeratorSecret::from_seed(ModeratorId(name.to_string()), &seed)
}

fn main() -> anyhow::Result<()> {
    let mods = ["alice", "bob", "carol"].map(moderator);
    let forum = ForumConfig {
        forum_id: digest("forum", &[b"registry-sim"]),
        k: 2,
        n: 2,
        moderators: mods.iter().map(ModeratorSecret::identity).collect(),
        mod_set_version: 1,
        threshold_public_key_hash: digest("threshold-pk", &[b"registry-sim"]),
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
        certs.push(sdk.aggregate_certificate(&post, reason, votes)?);
    }

    let slash = sdk.submit_slash(&certs)?;
    println!("slash reconstructed and revoked {}…", hex8(&slash.commitment));
    Ok(())
}
