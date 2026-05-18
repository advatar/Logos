use std::ffi::c_char;

use moderation_sdk::{storage_namespace, StorageKind};
use protocol_core::digest;

static FLOW_JSON: &str = r#"{"screens":["forum","register","post","moderate","vote","certificate","history","slash","rejected"],"backend":"moderation-sdk"}"#;
static FLOW_JSON_NUL: &[u8] = b"{\"screens\":[\"forum\",\"register\",\"post\",\"moderate\",\"vote\",\"certificate\",\"history\",\"slash\",\"rejected\"],\"backend\":\"moderation-sdk\"}\0";

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_full_flow_json() {
        let json = flow_json_for_tests();
        assert!(json.contains("certificate"));
        assert!(json.contains("rejected"));
        assert!(json.contains("moderation-sdk"));
    }

    #[test]
    fn probes_sdk_namespaces() {
        let namespaces = sdk_namespace_probe();
        assert!(namespaces[0].starts_with("post/"));
        assert!(namespaces[1].starts_with("vote/"));
        assert!(namespaces[2].starts_with("cert/"));
        assert!(namespaces[3].starts_with("slash/"));
    }
}
