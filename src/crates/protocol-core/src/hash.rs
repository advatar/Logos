use sha2::{Digest, Sha256};

use crate::field::{F, FIELD_PRIME};

pub type Hash32 = [u8; 32];

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

pub fn hash_to_field(domain: &str, parts: &[&[u8]]) -> F {
    let bytes = digest(domain, parts);
    let mut acc: u128 = 0;
    for b in bytes {
        acc = ((acc << 8) + b as u128) % FIELD_PRIME as u128;
    }
    let v = acc as u64;
    if v == 0 { F::ONE } else { F(v) }
}

pub fn hex8(hash: &Hash32) -> String {
    hex::encode(&hash[..4])
}
