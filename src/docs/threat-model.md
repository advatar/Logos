# Threat model

## In scope

- malicious members posting after revocation — blocked by the revocation-root binding in `proof_public_inputs_hash` (Phase 4 RISC0 guest enforces non-membership);
- a coalition of fewer than `N` moderators issuing a certificate — `verify_certificate` rejects under both the vote count and the DLEQ partial-decryption count;
- a coalition of `N` moderators forging a `revealed_share` for an unrelated post — blocked: the slash verifier ignores any aggregator-supplied share and recomputes via `aggregate_decrypt`, verifying each partial's DLEQ proof against the moderator's committed `share_public_key`;
- slash submitters combining unrelated certificates — `slash` rejects bundles with duplicate Shamir x-coordinates and rejects bundles that interpolate to a commitment not present in the registry;
- cross-forum replay of any signed/proven object — every transcript binds `forum_id`;
- cross-mod-set-version replay — `mod_set_version` is in the certificate statement;
- cross-threshold-key replay — `threshold_public_key_hash` is in the certificate statement and the post public-inputs hash;
- replaying a partial decryption from one post against another — DLEQ proofs bind the per-post domain seed `digest("partial-dleq-domain", forum_id, post_id)`;
- storage / messaging outages — handled by retry queues at the SDK boundary (planned, see `STATUS.md → Phase 6`).

## Out of scope

- content-based deanonymization (writing style, posting time, network metadata);
- moderator collusion at or above the threshold (N for certificates, K-of-many for slash) — this is a policy parameter, not a cryptographic property;
- endpoint compromise (a moderator's signing key or share secret leaking from their device);
- side channels during local proof generation;
- denial of service against Logos network services;
- a faulty distributed key generation — today `DealerShares::trusted` is a single-trusted-party stand-in; Pedersen DKG is a Phase 2 follow-up.

## Security assumptions

- SHA-256 is collision-resistant and preimage-resistant for both transcripts and the threshold KDF.
- Ed25519 signatures are existentially unforgeable.
- Ristretto255 discrete-log is hard; Chaum–Pedersen DLEQ proofs are sound.
- The hybrid threshold-ElGamal scheme is IND-CPA under the gap-DH assumption on Ristretto255.
- RISC0 receipts are sound and zero-knowledge for the published statement (`crates/risc0-statement`).
- The LEZ runtime executes the registry program faithfully (i.e. the operator does not deviate from the published IDL).

## Newly defended versus the starter spec

- **Trustless slash aggregation.** Earlier versions of this repo had `ModerationCertificate.revealed_share` as an aggregator-supplied field; the slash verifier had to trust whoever assembled the cert. The current design carries DLEQ-proven partial decryptions and recomputes the share inside `verify_certificate` / `slash`. The aggregator role is removed from the trust boundary.
- **Threshold-key rotation safety.** `threshold_public_key_hash` is bound into every transcript, so a certificate produced before a threshold-key rotation cannot be replayed afterward.
- **Merkle-root freshness.** The post envelope carries `membership_root` and `revocation_root` in the clear *and* binds them into the public-inputs hash, giving offline verifiers a freshness check.
