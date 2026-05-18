use std::ffi::c_char;

use moderation_sdk::{storage_namespace, ForumSdk, MemoryStore, StorageKind};
use protocol_core::{
    digest, AnonymousPostEnvelope, DealerShares, ForumConfig, Hash32, MemberSecret, ModeratorId,
    ModeratorSecret,
};

static FLOW_JSON: &str = r#"{"screens":["forum","register","post","moderate","vote","certificate","history","slash","rejected"],"backend":"moderation-sdk","risc0_receipt_boundary":"bytes"}"#;
static FLOW_JSON_NUL: &[u8] = b"{\"screens\":[\"forum\",\"register\",\"post\",\"moderate\",\"vote\",\"certificate\",\"history\",\"slash\",\"rejected\"],\"backend\":\"moderation-sdk\",\"risc0_receipt_boundary\":\"bytes\"}\0";

#[no_mangle]
pub extern "C" fn lp0016_basecamp_flow_json() -> *const c_char {
    FLOW_JSON_NUL.as_ptr() as *const c_char
}

pub fn flow_json_for_tests() -> &'static str {
    FLOW_JSON
}

pub fn sdk_namespace_probe() -> [String; 4] {
    let forum_id = digest("basecamp-core-probe", &[b"forum"]);
    [
        storage_namespace(StorageKind::Post, &forum_id),
        storage_namespace(StorageKind::Vote, &forum_id),
        storage_namespace(StorageKind::Cert, &forum_id),
        storage_namespace(StorageKind::Slash, &forum_id),
    ]
}

pub fn bind_risc0_receipt_for_app(
    post: &mut AnonymousPostEnvelope,
    expected_public_inputs_hash: Hash32,
    image_id: [u8; 32],
    journal: Hash32,
    receipt_bytes: Vec<u8>,
) -> protocol_core::Result<()> {
    if post.proof_public_inputs_hash != expected_public_inputs_hash {
        return Err(protocol_core::ProtocolError::InvalidCertificate);
    }
    post.attach_risc0_receipt(image_id, journal, receipt_bytes)
}

pub fn risc0_receipt_probe_for_tests() -> protocol_core::Result<AnonymousPostEnvelope> {
    let dealer = DealerShares::pedersen_dkg(1, 1, b"basecamp-risc0-probe-dkg");
    let moderator = ModeratorSecret::from_seed_and_share(
        ModeratorId("basecamp".into()),
        &digest("basecamp-mod-seed", &[b"risc0"]),
        dealer.share_secret_keys[0].clone(),
    );
    let forum = ForumConfig {
        forum_id: digest("forum", &[b"basecamp-risc0-probe"]),
        k: 1,
        n: 1,
        moderators: vec![moderator.identity()],
        mod_set_version: 1,
        threshold_public_key: dealer.threshold_public_key,
    };
    let member = MemberSecret::from_seed(&forum.forum_id, forum.k, b"member");
    let mut sdk = ForumSdk::new(forum, MemoryStore::default())?;
    sdk.register_member(&member)?;
    let mut post = sdk.build_post(
        &member,
        digest("content", &[b"app-flow"]),
        b"nonce".to_vec(),
    )?;
    let journal = post.proof_public_inputs_hash;
    sdk.attach_risc0_receipt(&mut post, [5u8; 32], journal, b"receipt-bytes".to_vec())?;
    Ok(post)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_full_flow_json() {
        let json = flow_json_for_tests();
        assert!(json.contains("certificate"));
        assert!(json.contains("rejected"));
        assert!(json.contains("moderation-sdk"));
        assert!(json.contains("risc0_receipt_boundary"));
    }

    #[test]
    fn probes_sdk_namespaces() {
        let namespaces = sdk_namespace_probe();
        assert!(namespaces[0].starts_with("post/"));
        assert!(namespaces[1].starts_with("vote/"));
        assert!(namespaces[2].starts_with("cert/"));
        assert!(namespaces[3].starts_with("slash/"));
    }

    #[test]
    fn binds_risc0_receipt_bytes_for_app_flow() {
        let post = risc0_receipt_probe_for_tests().unwrap();
        assert!(matches!(
            post.zk_receipt
                .verify_public_inputs(&post.proof_public_inputs_hash)
                .unwrap(),
            protocol_core::VerifiedZkReceipt::Risc0
        ));
    }

    #[test]
    fn rejects_receipt_for_wrong_app_flow_hash() {
        let mut post = risc0_receipt_probe_for_tests().unwrap();
        let wrong = digest("wrong", &[b"hash"]);
        let journal = post.proof_public_inputs_hash;
        assert_eq!(
            bind_risc0_receipt_for_app(&mut post, wrong, [6u8; 32], journal, b"receipt".to_vec(),)
                .unwrap_err(),
            protocol_core::ProtocolError::InvalidCertificate
        );
    }
}
