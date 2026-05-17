use serde::{Deserialize, Serialize};

use crate::{digest, hash_to_field, ProtocolError, Result, F, Hash32};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Share {
    pub x: F,
    pub y: F,
}

pub fn eval_poly(coeffs: &[F], x: F) -> F {
    let mut acc = F::ZERO;
    let mut power = F::ONE;
    for c in coeffs {
        acc += *c * power;
        power *= x;
    }
    acc
}

pub fn interpolate_coeffs(shares: &[Share]) -> Result<Vec<F>> {
    if shares.is_empty() {
        return Ok(vec![]);
    }
    for i in 0..shares.len() {
        for j in (i + 1)..shares.len() {
            if shares[i].x == shares[j].x {
                return Err(ProtocolError::DuplicateShareX);
            }
        }
    }

    let n = shares.len();
    let mut result = vec![F::ZERO; n];

    for (i, si) in shares.iter().enumerate() {
        let mut basis = vec![F::ONE];
        let mut denom = F::ONE;
        for (j, sj) in shares.iter().enumerate() {
            if i == j {
                continue;
            }
            let mut next = vec![F::ZERO; basis.len() + 1];
            for (degree, coeff) in basis.iter().copied().enumerate() {
                next[degree] -= coeff * sj.x;
                next[degree + 1] += coeff;
            }
            basis = next;
            denom *= si.x - sj.x;
        }
        let scale = si.y / denom;
        for (degree, coeff) in basis.into_iter().enumerate() {
            result[degree] += scale * coeff;
        }
    }

    Ok(result)
}

pub fn coeffs_to_bytes(coeffs: &[F]) -> Vec<u8> {
    let mut out = Vec::with_capacity(coeffs.len() * 8);
    for c in coeffs {
        out.extend_from_slice(&c.to_be_bytes());
    }
    out
}

pub fn commitment_for(forum_id: &Hash32, k: u8, coeffs: &[F]) -> Hash32 {
    let k_bytes = [k];
    let coeff_bytes = coeffs_to_bytes(coeffs);
    digest("member", &[forum_id, &k_bytes, &coeff_bytes])
}

pub fn share_for(forum_id: &Hash32, coeffs: &[F], content_id: &Hash32, nonce: &[u8]) -> Share {
    let x = hash_to_field("share-x", &[forum_id, content_id, nonce]);
    let y = eval_poly(coeffs, x);
    Share { x, y }
}

pub fn share_commitment(forum_id: &Hash32, content_id: &Hash32, nonce: &[u8], share: Share) -> Hash32 {
    digest(
        "share",
        &[forum_id, content_id, nonce, &share.x.to_be_bytes(), &share.y.to_be_bytes()],
    )
}

pub fn retro_tag(forum_id: &Hash32, coeffs: &[F], content_id: &Hash32, nonce: &[u8]) -> Hash32 {
    let coeff_bytes = coeffs_to_bytes(coeffs);
    digest("retro", &[forum_id, &coeff_bytes, content_id, nonce])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation_recovers_coefficients() {
        let coeffs = vec![F::from(5), F::from(7), F::from(11)];
        let shares = [
            Share { x: F::from(1), y: eval_poly(&coeffs, F::from(1)) },
            Share { x: F::from(2), y: eval_poly(&coeffs, F::from(2)) },
            Share { x: F::from(3), y: eval_poly(&coeffs, F::from(3)) },
        ];
        assert_eq!(interpolate_coeffs(&shares).unwrap(), coeffs);
    }
}
