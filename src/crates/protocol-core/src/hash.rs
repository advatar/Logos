use sha2::{Digest, Sha256};

use crate::field::Scalar;

pub type Hash32 = [u8; 32];

/// Canonical LP-0016 transcript hash: `lp0016:<domain>:v1 || u32(count) || repeated(u32(len) || bytes)`.
///
/// Every consensus byte string in the protocol is built this way; never use
/// `serde_json`/`bincode` for transcript bytes.
pub fn digest(domain: &str, parts: &[&[u8]]) -> Hash32 {
    let mut h = Sha256::new();
    h.update(b"lp0016:");
    h.update(domain.as_bytes());
    h.update(b":v1");
    h.update((parts.len() as u32).to_be_bytes());
    for part in parts {
        h.update((part.len() as u32).to_be_bytes());
        h.update(part);
    }
    h.finalize().into()
}

/// 64-byte uniform hash for wide reduction into the scalar field.
///
/// Implemented by hashing `(domain, 0, parts)` and `(domain, 1, parts)` and
/// concatenating, both using the canonical [`digest`] framing. This avoids
/// pulling SHA-512 into the dependency tree while still giving negligible bias.
fn digest64(domain: &str, parts: &[&[u8]]) -> [u8; 64] {
    let mut buf = [0u8; 64];
    let dom0 = [&domain.as_bytes()[..], b"|0"].concat();
    let dom1 = [&domain.as_bytes()[..], b"|1"].concat();
    let half0 = digest(core::str::from_utf8(&dom0).expect("ascii domain"), parts);
    let half1 = digest(core::str::from_utf8(&dom1).expect("ascii domain"), parts);
    buf[..32].copy_from_slice(&half0);
    buf[32..].copy_from_slice(&half1);
    buf
}

/// Domain-separated hash-to-scalar for the Ristretto255 scalar field.
pub fn hash_to_field(domain: &str, parts: &[&[u8]]) -> Scalar {
    let wide = digest64(domain, parts);
    Scalar::from_bytes_wide(&wide)
}

pub fn hex8(hash: &Hash32) -> String {
    hex::encode(&hash[..4])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_to_field_is_deterministic() {
        let a = hash_to_field("test", &[b"abc", b"def"]);
        let b = hash_to_field("test", &[b"abc", b"def"]);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_to_field_domain_separates() {
        let a = hash_to_field("alpha", &[b"x"]);
        let b = hash_to_field("beta", &[b"x"]);
        assert_ne!(a, b);
    }
}
