# Protocol specification

This document describes the protocol as implemented in `protocol-core`. Cross-references in code: `src/SPEC.md` for the production-target spec; `src/crates/risc0-statement/src/lib.rs` for the exact statement the RISC0 guest proves.

## Forum parameters

```text
forum_id                   32 bytes; SHA-256 digest of the forum's namespace
K                          revocation threshold (one slash needs K certificates)
N                          moderation threshold (one certificate needs N votes)
moderators                 Vec<ModeratorIdentity>  // Ed25519 verifying key + threshold share public key
mod_set_version            monotonically increases across moderator-set rotations
threshold_public_key       Ristretto255 point Y = sG, where s is Shamir-shared across moderators
membership_root            Merkle root over registered commitments (sorted, leaf/node domain-separated)
revocation_root            Merkle root over revoked commitments (same shape as membership_root)
```

Every signed or proven object is bound to these via the canonical transcript framing `lp0016:<domain>:v1 || u32(field_count) || u32(len) || bytes`. `serde_json` is never used for consensus bytes.

## Membership

Each member samples a hidden polynomial of degree `< K` over the Ristretto255 scalar field:

```text
P(X) = a₀ + a₁X + … + a_{K-1}X^{K-1}
member_commitment = digest("member", forum_id, K, coeffs(P))
```

`MemberSecret::from_seed` derives the coefficients deterministically via `hash_to_field` so a member can recover their commitment from a single backup seed. The on-chain registry stores `member_commitment` (plus `stake_amount` and `registered_at`); the coefficients never leave the member's device.

## Anonymous posting

For each content item:

```text
post_id          = digest("post-id", forum_id, content_id, post_nonce)
x                = hash_to_field("share-x", forum_id, content_id, post_nonce)
y                = eval(P, x)
share_commitment = digest("share", forum_id, content_id, post_nonce, x, y)
retro_tag        = digest("retro", forum_id, coeffs(P), content_id, post_nonce)
ciphertext       = ThresholdElGamal::encrypt(threshold_public_key, encode(x, y), nonce_seed=post_id)
ciphertext_hash  = digest("threshold-ciphertext", c1, c2)
proof_public_inputs_hash = digest(
    "proof-public-inputs",
    forum_id, [K], content_id, post_nonce,
    ciphertext_hash, share_commitment, retro_tag,
    threshold_public_key_hash, membership_root, revocation_root,
)
```

The post envelope binds all of the above. The ZK receipt (RISC0) commits to `proof_public_inputs_hash`; nothing else about the member is revealed.

The threshold-ElGamal hybrid encryption uses `KEM = SHA-256("kem" || rY)` keying a SHA-256 counter-mode KDF; the 64-byte `(x, y)` plaintext is XOR'd against the keystream. `r` is derived deterministically from `nonce_seed` so the post envelope's `ciphertext_hash` is reproducible by the RISC0 guest, which is what binds the encryption inside the receipt.

## Moderation certificates

A moderation vote signs the canonical certificate statement hash:

```text
forum_id
post_id
content_id
proof_public_inputs_hash
ciphertext_hash
reason_hash
mod_set_version
K
N
threshold_public_key_hash
```

Each moderator also publishes a **partial decryption** `D_i = s_i · C1` with a Chaum–Pedersen DLEQ proof that `log_G(S_i) = log_{C1}(D_i)`. The DLEQ is bound to a per-post domain seed `digest("partial-dleq-domain", forum_id, post_id)` so a partial decryption cannot be replayed across posts.

A moderation certificate carries:

```text
statement                   CertificateStatement (see above)
votes                       N distinct Ed25519 signatures over statement.hash()
ciphertext                  the post's Ciphertext (binds to statement.ciphertext_hash)
partial_decryptions         N PartialDecryption { idx, D_i, DLEQ }
```

`verify_certificate(forum, cert)` checks:

1. Statement parameters match the forum (forum_id, K, N, mod_set_version, threshold_public_key_hash).
2. The statement hashes `cert.ciphertext`.
3. At least `N` distinct moderator ids signed `statement.hash()`.
4. At least `N` partial decryptions, no duplicate indices.
5. Each Ed25519 signature verifies against the moderator's `verifying_key` in `forum.moderators`.
6. Each partial decryption's DLEQ verifies against the moderator's `share_public_key` and the per-post domain seed.

`cert.revealed_share(forum)` aggregates the partial decryptions via Lagrange-at-zero, KDFs the recovered group element, decrypts the 64-byte payload, and decodes `(x, y)`. The slash verifier does **not** trust an aggregator-supplied share — it always recomputes.

## Slash

A slash bundle is `Vec<ModerationCertificate>` of length exactly `K`. The verifier:

1. Verifies every certificate independently.
2. Aggregates each cert's partials to recover its `(x, y)` share.
3. Rejects duplicate x-coordinates.
4. Interpolates the unique degree-`< K` polynomial.
5. Computes `commitment = digest("member", forum_id, K, coeffs)`.
6. Confirms the commitment is registered and not revoked.
7. Writes a `RevocationRecord` and advances `revocation_root`.

## Unlinkability argument

Before slash, observers see only public post envelopes. Each post commits to an encrypted Shamir share `(x, y)` and a `retro_tag` derived from the unknown polynomial; the ZK receipt proves consistency without revealing `(x, y)` or the polynomial.

A single moderation certificate reveals one Shamir point. Fewer than `K` distinct points are insufficient to determine a degree-`< K` polynomial: for any candidate constant term, there exists a polynomial of degree `< K` matching the observed points. Therefore fewer than `K` certificates do not identify a registered commitment in the ideal Shamir model.

Upon slash, `K` certificates reconstruct the member polynomial. Anyone can then recompute that member's `retro_tag` for historical posts and link the slashed member's prior content. No other member's polynomial is reconstructed, so no other member's anonymity is affected.

## Replay protection

Every signed or proven object is domain-separated by `forum_id`, and where applicable by `mod_set_version` and `threshold_public_key_hash`. Cross-forum, cross-mod-set, and cross-threshold-key replays are rejected by `verify_certificate`. The `idl_matches_handwritten_file` and `cross_forum_certificate_is_rejected` / `cross_mod_set_version_rejected` tests pin this.

## Revocation behaviour

After slash, the commitment is added to `revocation_root`. A later post envelope carries the *new* `revocation_root` in its public-inputs hash; the RISC0 guest's statement (`crates/risc0-statement`) verifies non-membership against `revocation_root` using predecessor/successor Merkle proofs and sorted-index adjacency checks.

## Development model in this repository

- The **Rust** core (`protocol-core`, `moderation-sdk`, `lp0016-registry`, `risc0-statement`) implements the protocol with production crypto: Ristretto255 scalar field, Ed25519 signatures, threshold ElGamal + Chaum–Pedersen DLEQ, Merkle roots, revocation non-membership proofs, a `ZkReceipt::{Mock,Risc0}` envelope, and a Pedersen-style DKG transcript API. Local tests use mock receipts; full RISC0 proving requires the external RISC0 toolchain.
- The **Python** simulator (`scripts/lp0016_sim.py`) keeps the same protocol state-machine shape but stays on the small dev field `2^61 - 1` and the SHA-256-derived dev moderator signature; SPEC.md keeps it dependency-free. It is the executable structural reference; commitment bytes will not match Rust because the fields differ.
