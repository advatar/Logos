//! Threshold ElGamal encryption over Ristretto255 with DLEQ partial-decryption proofs.
//!
//! The forum holds a single threshold public key `Y = s·G`; the secret `s` is
//! Shamir-shared across the moderator set as `s_i = P(i)` for a degree
//! `N - 1` polynomial `P` with `P(0) = s`. Posts encrypt their Shamir-strike
//! share `(x, y)` (64 bytes) under `Y`; any `N` moderators can recover the
//! plaintext by publishing partial decryptions `D_i = s_i · C1` together with
//! Chaum–Pedersen DLEQ proofs binding `D_i` to their committed public share
//! `S_i = s_i · G`.
//!
//! Distributed key generation (DKG) is out of scope for this module — the
//! `DealerShares` constructor is a trusted-dealer simulation suitable for
//! tests and local demos. Production deployments must replace
//! [`DealerShares::trusted`] with a proper DKG (e.g. Pedersen DKG) before
//! security review.

use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::ristretto::RistrettoPoint;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::{digest, eval_poly, hash_to_field, Hash32, ProtocolError, Result, Scalar, Share};

/// Plaintext payload encrypted under the threshold key: serialized `(x, y)`.
pub const PLAINTEXT_LEN: usize = 64;

pub type Plaintext = [u8; PLAINTEXT_LEN];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdPublicKey(pub(crate) RistrettoPoint);

impl ThresholdPublicKey {
    pub fn point(&self) -> RistrettoPoint {
        self.0
    }

    pub fn hash(&self) -> Hash32 {
        digest("threshold-pk", &[&self.0.compress().to_bytes()])
    }
}

impl Serialize for ThresholdPublicKey {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        ser_point(&self.0, s)
    }
}

impl<'de> Deserialize<'de> for ThresholdPublicKey {
    fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        de_point(de).map(ThresholdPublicKey)
    }
}

#[derive(Debug, Clone)]
pub struct ShareSecretKey {
    pub idx: u32,
    pub(crate) secret: Scalar,
}

impl ShareSecretKey {
    pub fn public(&self) -> SharePublicKey {
        SharePublicKey { idx: self.idx, point: RISTRETTO_BASEPOINT_POINT * self.secret.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharePublicKey {
    pub idx: u32,
    pub(crate) point: RistrettoPoint,
}

impl Serialize for SharePublicKey {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        (self.idx, ser_point_bytes(&self.point)).serialize(s)
    }
}

impl<'de> Deserialize<'de> for SharePublicKey {
    fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        let (idx, bytes): (u32, [u8; 32]) = Deserialize::deserialize(de)?;
        let point = decompress(bytes).map_err(serde::de::Error::custom)?;
        Ok(SharePublicKey { idx, point })
    }
}

#[derive(Debug, Clone)]
pub struct DealerShares {
    pub secret: Scalar,
    pub threshold_public_key: ThresholdPublicKey,
    pub share_secret_keys: Vec<ShareSecretKey>,
    pub share_public_keys: Vec<SharePublicKey>,
}

impl DealerShares {
    /// Generate a trusted-dealer key set: degree `threshold - 1` polynomial
    /// with `total` evaluation points indexed `1..=total`. Replace with a real
    /// DKG before production.
    pub fn trusted(threshold: u32, total: u32, seed: &[u8]) -> Self {
        assert!(threshold >= 1 && threshold <= total, "invalid threshold/total");
        let mut coeffs = Vec::with_capacity(threshold as usize);
        for i in 0..threshold {
            coeffs.push(hash_to_field("dealer-poly-coeff", &[seed, &i.to_be_bytes()]));
        }
        let secret = coeffs[0];
        let threshold_public_key = ThresholdPublicKey(RISTRETTO_BASEPOINT_POINT * secret.0);
        let mut share_secret_keys = Vec::with_capacity(total as usize);
        let mut share_public_keys = Vec::with_capacity(total as usize);
        for j in 1..=total {
            let x = Scalar::from_u64(j as u64);
            let s_j = eval_poly(&coeffs, x);
            let sk = ShareSecretKey { idx: j, secret: s_j };
            share_public_keys.push(sk.public());
            share_secret_keys.push(sk);
        }
        Self { secret, threshold_public_key, share_secret_keys, share_public_keys }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ciphertext {
    #[serde(serialize_with = "ser_ciphertext_c1", deserialize_with = "de_ciphertext_c1")]
    pub c1: RistrettoPoint,
    pub c2: Vec<u8>,
}

impl Ciphertext {
    pub fn hash(&self) -> Hash32 {
        digest("threshold-ciphertext", &[&self.c1.compress().to_bytes(), &self.c2])
    }
}

/// Hybrid encryption: ephemeral C1 = rG, KEM = SHA256("kem" || r·Y), C2 = m XOR KDF(KEM).
///
/// The ephemeral scalar `r` is derived deterministically from `nonce_seed` so
/// callers can reproduce the same ciphertext (matters for the post public-inputs
/// hash). Pass a fresh, unpredictable `nonce_seed` per post.
pub fn encrypt(pk: &ThresholdPublicKey, plaintext: &Plaintext, nonce_seed: &[u8]) -> Ciphertext {
    let r = hash_to_field("threshold-encrypt-r", &[nonce_seed]);
    let c1 = RISTRETTO_BASEPOINT_POINT * r.0;
    let shared = pk.0 * r.0;
    let pad = kdf(&shared, PLAINTEXT_LEN);
    let mut c2 = vec![0u8; PLAINTEXT_LEN];
    for i in 0..PLAINTEXT_LEN {
        c2[i] = plaintext[i] ^ pad[i];
    }
    Ciphertext { c1, c2 }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialDecryption {
    pub idx: u32,
    #[serde(serialize_with = "ser_ciphertext_c1", deserialize_with = "de_ciphertext_c1")]
    pub d: RistrettoPoint,
    pub dleq: DleqProof,
}

/// Chaum–Pedersen DLEQ: proves `log_G(S) == log_C1(D)` with the same exponent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DleqProof {
    pub c: Scalar,
    pub z: Scalar,
}

pub fn partial_decrypt(
    sk: &ShareSecretKey,
    ciphertext: &Ciphertext,
    pk_share: &SharePublicKey,
    domain_seed: &[u8],
) -> PartialDecryption {
    let d = ciphertext.c1 * sk.secret.0;
    let dleq = dleq_prove(&sk.secret, &ciphertext.c1, &pk_share.point, &d, domain_seed);
    PartialDecryption { idx: sk.idx, d, dleq }
}

pub fn verify_partial(
    pd: &PartialDecryption,
    ciphertext: &Ciphertext,
    pk_share: &SharePublicKey,
    domain_seed: &[u8],
) -> bool {
    pd.idx == pk_share.idx
        && dleq_verify(&pk_share.point, &ciphertext.c1, &pd.d, &pd.dleq, domain_seed)
}

/// Aggregate `N` verified partial decryptions and recover the plaintext.
/// `pk_shares` must contain the public-share entries matching each partial
/// (same order, same indices). Caller has already run `verify_partial` on each.
pub fn aggregate_decrypt(
    ciphertext: &Ciphertext,
    partials: &[PartialDecryption],
    pk_shares: &[SharePublicKey],
) -> Result<Plaintext> {
    if partials.len() != pk_shares.len() {
        return Err(ProtocolError::InvalidCertificate);
    }
    if partials.is_empty() {
        return Err(ProtocolError::PartialCertificate);
    }
    // We need to interpolate D_i values at x = 0 using their idx values as x
    // coordinates. Use the existing Shamir interpolation: treat (x_i, D_i) as
    // a polynomial in the *group*; the scalar-coefficient identity carries
    // through because each D_i = s_i · C1 and s_i = P(idx_i).
    //
    // Equivalent direct construction: D = sum_i lambda_i(0) · D_i where
    // lambda_i(0) is the Lagrange basis at zero evaluated against the share
    // indices. Compute Lagrange coefficients in the scalar field, then take
    // the group multi-scalar combination.
    let xs: Vec<Scalar> = partials.iter().map(|p| Scalar::from_u64(p.idx as u64)).collect();
    let lambdas = lagrange_zero(&xs)?;
    let mut acc = RistrettoPoint::default();
    for (lambda, pd) in lambdas.iter().zip(partials.iter()) {
        acc += pd.d * lambda.0;
    }
    let pad = kdf(&acc, PLAINTEXT_LEN);
    if ciphertext.c2.len() != PLAINTEXT_LEN {
        return Err(ProtocolError::InvalidCertificate);
    }
    let mut out = [0u8; PLAINTEXT_LEN];
    for i in 0..PLAINTEXT_LEN {
        out[i] = ciphertext.c2[i] ^ pad[i];
    }
    Ok(out)
}

/// Encode a Shamir strike share `(x, y)` as the 64-byte threshold plaintext.
pub fn encode_share(share: Share) -> Plaintext {
    let mut out = [0u8; PLAINTEXT_LEN];
    out[..32].copy_from_slice(&share.x.to_bytes());
    out[32..].copy_from_slice(&share.y.to_bytes());
    out
}

pub fn decode_share(plaintext: &Plaintext) -> Result<Share> {
    let x_bytes: [u8; 32] = plaintext[..32].try_into().expect("32 bytes");
    let y_bytes: [u8; 32] = plaintext[32..].try_into().expect("32 bytes");
    let x = Scalar::from_canonical_bytes(x_bytes).ok_or(ProtocolError::InvalidCertificate)?;
    let y = Scalar::from_canonical_bytes(y_bytes).ok_or(ProtocolError::InvalidCertificate)?;
    Ok(Share { x, y })
}

fn dleq_prove(
    secret: &Scalar,
    base2: &RistrettoPoint,
    pub1: &RistrettoPoint,
    pub2: &RistrettoPoint,
    domain_seed: &[u8],
) -> DleqProof {
    // Sigma protocol for `log_G(pub1) = log_{base2}(pub2) = secret`.
    // Commitment: pick k, A1 = kG, A2 = k·base2.
    // Challenge: c = H(transcript || pub1 || pub2 || A1 || A2).
    // Response: z = k + c·secret.
    let k = hash_to_field(
        "dleq-nonce",
        &[
            domain_seed,
            &secret.to_bytes(),
            &base2.compress().to_bytes(),
            &pub1.compress().to_bytes(),
            &pub2.compress().to_bytes(),
        ],
    );
    let a1 = RISTRETTO_BASEPOINT_POINT * k.0;
    let a2 = *base2 * k.0;
    let c = dleq_challenge(base2, pub1, pub2, &a1, &a2, domain_seed);
    let z = k + c * *secret;
    DleqProof { c, z }
}

fn dleq_verify(
    pub1: &RistrettoPoint,
    base2: &RistrettoPoint,
    pub2: &RistrettoPoint,
    proof: &DleqProof,
    domain_seed: &[u8],
) -> bool {
    // Recompute A1 = z·G - c·pub1 and A2 = z·base2 - c·pub2, then verify the
    // challenge.
    let a1 = RISTRETTO_BASEPOINT_POINT * proof.z.0 - *pub1 * proof.c.0;
    let a2 = *base2 * proof.z.0 - *pub2 * proof.c.0;
    let expected = dleq_challenge(base2, pub1, pub2, &a1, &a2, domain_seed);
    expected == proof.c
}

fn dleq_challenge(
    base2: &RistrettoPoint,
    pub1: &RistrettoPoint,
    pub2: &RistrettoPoint,
    a1: &RistrettoPoint,
    a2: &RistrettoPoint,
    domain_seed: &[u8],
) -> Scalar {
    hash_to_field(
        "dleq-challenge",
        &[
            domain_seed,
            &base2.compress().to_bytes(),
            &pub1.compress().to_bytes(),
            &pub2.compress().to_bytes(),
            &a1.compress().to_bytes(),
            &a2.compress().to_bytes(),
        ],
    )
}

fn lagrange_zero(xs: &[Scalar]) -> Result<Vec<Scalar>> {
    let n = xs.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let xi = xs[i];
        let mut num = Scalar::ONE;
        let mut den = Scalar::ONE;
        for j in 0..n {
            if i == j {
                continue;
            }
            let xj = xs[j];
            if xj == xi {
                return Err(ProtocolError::DuplicateShareX);
            }
            num = num * (Scalar::ZERO - xj);
            den = den * (xi - xj);
        }
        out.push(num / den);
    }
    Ok(out)
}

fn kdf(shared: &RistrettoPoint, len: usize) -> Vec<u8> {
    // SHA-256 in counter mode, salt = "lp0016:threshold-kdf:v1".
    let shared_bytes = shared.compress().to_bytes();
    let mut out = Vec::with_capacity(len);
    let mut counter: u32 = 0;
    while out.len() < len {
        let mut h = Sha256::new();
        h.update(b"lp0016:threshold-kdf:v1");
        h.update(&shared_bytes);
        h.update(counter.to_be_bytes());
        out.extend_from_slice(&h.finalize());
        counter += 1;
    }
    out.truncate(len);
    out
}

// --- serde helpers ----------------------------------------------------------

fn ser_point<S: Serializer>(point: &RistrettoPoint, s: S) -> std::result::Result<S::Ok, S::Error> {
    let bytes = point.compress().to_bytes();
    if s.is_human_readable() {
        s.serialize_str(&hex::encode(bytes))
    } else {
        s.serialize_bytes(&bytes)
    }
}

fn ser_point_bytes(point: &RistrettoPoint) -> [u8; 32] {
    point.compress().to_bytes()
}

fn de_point<'de, D: Deserializer<'de>>(de: D) -> std::result::Result<RistrettoPoint, D::Error> {
    use serde::de::Error;
    if de.is_human_readable() {
        let s = String::deserialize(de)?;
        let raw = hex::decode(&s).map_err(D::Error::custom)?;
        let bytes: [u8; 32] = raw.try_into().map_err(|_| D::Error::custom("point must be 32 bytes"))?;
        decompress(bytes).map_err(D::Error::custom)
    } else {
        let raw = <Vec<u8>>::deserialize(de)?;
        let bytes: [u8; 32] = raw.try_into().map_err(|_| D::Error::custom("point must be 32 bytes"))?;
        decompress(bytes).map_err(D::Error::custom)
    }
}

fn ser_ciphertext_c1<S: Serializer>(point: &RistrettoPoint, s: S) -> std::result::Result<S::Ok, S::Error> {
    ser_point(point, s)
}

fn de_ciphertext_c1<'de, D: Deserializer<'de>>(de: D) -> std::result::Result<RistrettoPoint, D::Error> {
    de_point(de)
}

fn decompress(bytes: [u8; 32]) -> std::result::Result<RistrettoPoint, &'static str> {
    curve25519_dalek::ristretto::CompressedRistretto(bytes)
        .decompress()
        .ok_or("non-canonical ristretto point")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest;

    #[test]
    fn dealer_shares_satisfy_threshold() {
        let dealer = DealerShares::trusted(2, 3, b"test-seed");
        // The interpolation at x=0 of any two share secrets must equal the master secret.
        let xs: Vec<Scalar> = dealer.share_secret_keys[..2].iter().map(|s| Scalar::from_u64(s.idx as u64)).collect();
        let lambdas = lagrange_zero(&xs).unwrap();
        let mut recovered = Scalar::ZERO;
        for (l, sk) in lambdas.iter().zip(dealer.share_secret_keys[..2].iter()) {
            recovered += *l * sk.secret;
        }
        assert_eq!(recovered, dealer.secret);
    }

    #[test]
    fn encrypt_aggregate_round_trip() {
        let dealer = DealerShares::trusted(2, 3, b"rt-seed");
        let share = Share { x: Scalar::from_u64(42), y: Scalar::from_u64(99) };
        let plaintext = encode_share(share);
        let ct = encrypt(&dealer.threshold_public_key, &plaintext, b"nonce-0");
        let domain_seed = digest("post-id", &[b"unit-1"]);
        let pds: Vec<_> = dealer.share_secret_keys[..2]
            .iter()
            .zip(dealer.share_public_keys[..2].iter())
            .map(|(sk, pk)| partial_decrypt(sk, &ct, pk, &domain_seed))
            .collect();
        for (pd, pk) in pds.iter().zip(dealer.share_public_keys[..2].iter()) {
            assert!(verify_partial(pd, &ct, pk, &domain_seed));
        }
        let plain = aggregate_decrypt(&ct, &pds, &dealer.share_public_keys[..2]).unwrap();
        let recovered = decode_share(&plain).unwrap();
        assert_eq!(recovered, share);
    }

    #[test]
    fn dleq_rejects_wrong_secret() {
        let dealer = DealerShares::trusted(2, 3, b"bad-seed");
        let share = Share { x: Scalar::from_u64(1), y: Scalar::from_u64(2) };
        let plaintext = encode_share(share);
        let ct = encrypt(&dealer.threshold_public_key, &plaintext, b"n");
        let domain_seed = digest("post-id", &[b"u"]);
        // Use moderator 1's secret but claim it's moderator 2's public share.
        let mut tampered = partial_decrypt(&dealer.share_secret_keys[0], &ct, &dealer.share_public_keys[0], &domain_seed);
        tampered.idx = dealer.share_public_keys[1].idx;
        assert!(!verify_partial(&tampered, &ct, &dealer.share_public_keys[1], &domain_seed));
    }

    #[test]
    fn fewer_than_threshold_does_not_recover() {
        let dealer = DealerShares::trusted(2, 3, b"few-seed");
        let share = Share { x: Scalar::from_u64(7), y: Scalar::from_u64(11) };
        let plaintext = encode_share(share);
        let ct = encrypt(&dealer.threshold_public_key, &plaintext, b"n");
        let domain_seed = digest("post-id", &[b"f"]);
        let pds: Vec<_> = dealer.share_secret_keys[..1]
            .iter()
            .zip(dealer.share_public_keys[..1].iter())
            .map(|(sk, pk)| partial_decrypt(sk, &ct, pk, &domain_seed))
            .collect();
        // Aggregation against the single share does not interpolate to the right group element.
        let plain = aggregate_decrypt(&ct, &pds, &dealer.share_public_keys[..1]).unwrap();
        let bad = decode_share(&plain);
        // It will *try* to decode 64 bytes; the result may be a valid scalar pair
        // but it will not equal the original share.
        if let Ok(s) = bad {
            assert_ne!(s, share);
        }
    }

    #[test]
    fn ciphertext_json_round_trip() {
        let dealer = DealerShares::trusted(2, 3, b"json-seed");
        let share = Share { x: Scalar::from_u64(13), y: Scalar::from_u64(17) };
        let plaintext = encode_share(share);
        let ct = encrypt(&dealer.threshold_public_key, &plaintext, b"n");
        let s = serde_json::to_string(&ct).unwrap();
        let back: Ciphertext = serde_json::from_str(&s).unwrap();
        assert_eq!(ct, back);
    }
}
