# Status

## Cleared

- [x] Improve repository `.gitignore` coverage for Rust, Python, editor, and OS artifacts.
- [x] Move CI workflow to root `.github/workflows/ci.yml` with `src` working-directory defaults.
- [x] Verify the Python simulator demo and unit tests.
- [x] Verify the Rust workspace build and tests.
- [x] Verify the registry simulator binary.
- [x] Verify the Lean scaffold build.
- [x] Commit and push the completed cleanup and verification state.

Tracking issue (cleanup): https://github.com/advatar/Logos/issues/1

## Close-the-gaps plan

Goal: bring the repo from "protocol-state-machine reference + boundary stubs" to the production target described in `src/SPEC.md` and `REPO.md`. The work is sequenced so the workspace keeps building under plain `cargo` at every step; toolchain-dependent layers (RISC0, LEZ/SPEL, Basecamp) are added behind feature flags / separate workspaces so unblocked layers ship independently.

Tracking issue: https://github.com/advatar/Logos/issues/2.

### Phase 1 — Production crypto in `protocol-core` ✅

- [x] Add `curve25519-dalek` and `ed25519-dalek` as workspace dependencies (default-features off, only what we need).
- [x] Replace the `F = 2^61 - 1` field with `curve25519_dalek::Scalar` behind a `Scalar` newtype.
- [x] Implement `hash_to_field` returning `Scalar` via wide-reduction (two SHA-256 invocations concatenated, kept on SHA-256 per `SPEC.md §2`).
- [x] Migrate `shamir::{eval_poly, interpolate_coeffs, share_for, retro_tag, commitment_for}` to the new scalar type.
- [x] Replace `dev_sign` placeholder with `ed25519_dalek::Signature`; `ForumConfig.moderators` carries `ModeratorIdentity { id, verifying_key }` and `verify_vote` validates via `VerifyingKey::verify`.
- [x] `threshold_public_key_hash` added to `ForumConfig`, `CertificateStatement`, and the post public-inputs hash, matching `SPEC.md §3`.
- [x] Property tests: Shamir round-trip, interpolation rejects duplicate x, certificate replay across forums rejected, certificate replay across threshold-key rotations rejected (Python).

Open follow-ups for this phase:
- [ ] Add a Rust property test for `mod_set_version` rotation rejection (Python has the equivalent for threshold-key rotation; Rust covers cross-forum but not mod-set-version yet).
- [ ] Add a fuzz target around `interpolate_coeffs` for malformed share inputs.

### Phase 2 — Threshold ElGamal + DLEQ ✅ (DKG deferred)

- [x] New module `protocol-core::threshold` with `ThresholdPublicKey`, per-moderator `ShareSecretKey` / `SharePublicKey`, hybrid encryption of the 64-byte `(x, y)` payload (`KEM = H("kem", rY)` keying a SHA-256 counter-mode KDF), partial decryption `D_i = s_i · C1`, Chaum–Pedersen DLEQ proof binding `D_i` to its committed `S_i = s_i · G`, Lagrange-at-zero aggregator that recovers the plaintext from N verified partials.
- [x] SHA-256 counter-mode KDF (no extra deps).
- [x] `DevThresholdOracle` removed. `AnonymousPostEnvelope` carries the real `Ciphertext`; `ModerationCertificate` carries `Vec<PartialDecryption>` (DLEQ-proven). `verify_certificate` checks each partial against the moderator's `share_public_key`, rejects duplicate indices, and `cert.revealed_share(forum)` aggregates trustlessly. `slash` no longer trusts an input share — it recomputes from the partials.
- [x] Unit tests: dealer-shares threshold property (Lagrange-at-zero recovers the master secret), end-to-end encrypt/partial-decrypt/aggregate round trip, DLEQ rejects wrong-key partials, fewer-than-threshold partials do not recover, JSON round trip, cross-mod-set-version cert rejection.

Deferred for follow-up:
- [ ] Real DKG (e.g. Pedersen) replacing `DealerShares::trusted`. The trusted-dealer is fine for tests/demos and clearly labeled, but production needs no single party that ever sees `s`.
- [ ] Batch DLEQ verification for large `N` (current code verifies one at a time).

### Phase 3 — Registry roots + Merkle paths ✅ (non-membership proofs deferred)

- [x] `protocol-core::merkle`: sorted, de-duplicating binary Merkle tree over `Hash32`. Domain separation between leaves (`digest("merkle-leaf", &[leaf])`) and internal nodes (`digest("merkle-node", left, right)`) so a leaf hash cannot pose as a subtree root. `root_from_set`, `prove_membership`, `verify_membership`, `MerklePath`.
- [x] `RegistryState::membership_root()` / `revocation_root()` derive from the current `BTreeSet`s on demand. The empty root is a distinct domain-separated constant.
- [x] `AnonymousPostEnvelope::build` now takes `&RegistryState` and binds both `membership_root` and `revocation_root` into the public-inputs hash alongside `threshold_public_key_hash`. The envelope also stores the roots in the clear so off-chain verifiers can check freshness.
- [x] Tests: empty-set root is canonical, root deterministic across insertion order, singleton root is the domain-separated leaf (not the raw leaf), tampered path is rejected, membership round trips for every leaf, registry roots change on register/revoke.

Deferred for Phase 4 (RISC0):
- [ ] Non-membership proofs against the revocation root. Encoding depends on the circuit shape (sparse Merkle tree vs. indexed Merkle tree); pick once the guest is being written so the in-circuit cost is the deciding factor.
- [ ] Incremental root updates. Today both roots are recomputed from scratch on every read. Fine at starter scale; revisit if `registered` grows past a few thousand entries.

### Phase 4 — RISC0 guest + host

- [ ] New crate `crates/risc0-statement` (no_std) holding the pure check function used in both the guest and a `cfg(test)` host harness — same code, no duplication.
- [ ] `src/zk/membership-guest`: real RISC0 guest using `risc0-zkvm`, reading public/private inputs, calling `risc0-statement::check`, committing the public-inputs hash.
- [ ] `src/zk/membership-host`: real RISC0 host that builds an `ExecutorEnv`, produces a receipt, and verifies it.
- [ ] In `protocol-core`, replace `MockZkReceipt` with a `ZkReceipt` enum: `Mock` (cfg-feature `dev-mocks`) and `Risc0 { receipt_bytes, image_id }` (cfg-feature `risc0-verify`). The verifier checks `Risc0` receipts via the host crate when `risc0-verify` is on.
- [ ] Keep the default workspace build on `dev-mocks` so contributors without `cargo-risczero` can still build & test.
- [ ] `RISC0_DEV_MODE=0 scripts/demo_e2e.sh` runs against the real guest once the toolchain is installed (documented prerequisite).

### Phase 5 — LEZ/SPEL registry program

- [ ] Move `src/registry/lez-program-stub` to `src/registry/lp0016-registry` and add SPEL-shaped annotations behind a `spel` feature so the crate keeps compiling without `logos-scaffold`. Annotations cover: `#[lez_program]`, `#[instruction]` on `create_forum/register_member/slash_member`, `#[account]` on `ForumState/MemberRecord/RevocationRecord/ModeratorSetVersion`.
- [ ] Generate a hand-written IDL at `src/registry/idl/lp0016_registry.json` matching the eventual `logos-scaffold build idl` output shape, so downstream tooling can be wired before the real generator is available. CI lint compares this against a `gen-idl` test that round-trips the structs.
- [ ] Persist account types using the canonical transcript serialization (not `serde_json`) so on-chain bytes are deterministic.

### Phase 6 — Storage, retry, and slash-verifier CLI

- [~] `moderation-sdk` namespaces post storage under `post/<forum_id_hex>/...`. Remaining: `cert/`, `vote/`, `slash/` namespaces and a uniform helper.
- [ ] Add a `RetryQueue` trait with an in-memory implementation; wrap every `OffchainStore::put` and the slash submission path through it.
- [x] `crates/slash-verifier`: real CLI that loads a `RegistrySnapshot` + `SlashBundleFile` from JSON, runs `verify_certificate` + `slash`, prints the recovered commitment, or exits non-zero on failure. Library half is independently unit-tested.
- [ ] Smoke test the CLI from `scripts/demo_e2e.sh` (needs `registry-sim` to also emit JSON snapshots).

### Phase 7 — Lean proofs

- [ ] `lean/AnonymousForum/Field.lean`: a generic `Field` typeclass with the operations Shamir needs; instantiate over `Nat` arithmetic mod `p` (concrete development field) so the proofs are runnable.
- [ ] `lean/AnonymousForum/Shamir.lean`: `eval`, `interpolate`, and the Lagrange-correctness theorem `lagrange_reconstructs_original_polynomial`.
- [ ] `lean/AnonymousForum/Slash.lean`: state and prove `slash_sound`: `verifySlash cfg registry bundle = some commitment → commitment ∈ registry.members ∧ commitment ∉ registry.revoked ∧ bundle.certs.length = cfg.K`.
- [ ] `lean/AnonymousForum/Domain.lean`: domain-separation theorem — a certificate produced with `forum_id = A` does not verify against `forum_id = B`; same for `mod_set_version` mismatch.
- [ ] Remove the `ShamirTargets.lean` `sorry`-free placeholder once the above theorems land.

### Phase 8 — Basecamp app

- [ ] Replace `src/app/basecamp-forum/Main.qml` with a real 9-screen flow: create forum, register, post, moderator dashboard, vote, aggregate certificate, history, slash, post-revocation rejection view.
- [ ] Add `core-module/` with a Rust → C ABI bridge over `moderation-sdk`, packaged according to Basecamp module conventions.
- [ ] Add a `Main.qml` test harness that drives the screens deterministically against `MemoryStore` for CI.
- [ ] Document the `logos-scaffold basecamp launch` entry point once a tag is pinned.

### Phase 9 — Docs, CI, perf, demo

- [ ] `src/docs/protocol.md`, `api.md`, `threat-model.md`, `performance.md`: rewrite to match the production crypto, Merkle accounts, threshold encryption protocol, and RISC0 statement actually implemented.
- [ ] Extend `.github/workflows/ci.yml` with: RISC0 guest build (optional matrix using `dtolnay/install` for the toolchain), Basecamp QML lint, slash-verifier smoke test, IDL round-trip test.
- [ ] `src/scripts/measure_cu.sh`: implement against the local sequencer once available; until then, output a structured "N/A — needs LEZ" so CI can grep for completion.
- [ ] Acceptance walk-through documented in `src/docs/demo.md` matching `SPEC.md §7`.

### What remains explicitly out-of-scope locally

- Deploying the LEZ program to devnet/testnet and capturing real CU numbers.
- Running Basecamp end-to-end against a real Logos Storage / Delivery node.
- Recording the narrated demo video.

These need infrastructure beyond a developer laptop and are tracked separately under the parent issue.

## Verification results (baseline)

- `cd src && python3 scripts/demo_e2e.py`: passed.
- `cd src && python3 -m unittest scripts/test_protocol.py`: passed, 6 tests.
- `cd src && cargo build --workspace`: passed with Rust 1.82.0 after pinning `clap` to `=4.5.50`.
- `cd src && cargo test --workspace`: passed, including `protocol-core` unit tests and doc-tests.
- `cd src && cargo run -p registry-sim`: passed.
- `cd src/lean && lake build`: passed.

## Placeholder inventory (superseded by the Close-the-gaps plan above, kept for reference)

- `src/registry/lez-program-stub/`: LEZ/SPEL registry boundary stub, pending a generated SPEL-annotated LEZ program.
- `src/zk/membership-guest/` and `src/zk/membership-host/`: RISC0 guest/host placeholders, pending real membership and post receipts.
- `src/app/basecamp-forum/`: minimal Basecamp QML placeholder, pending real SDK-backed forum workflows.
- `src/scripts/measure_cu.sh`: compute-unit measurement placeholder, pending a deployed localnet/testnet flow.
- `src/scaffold.toml`: placeholder scaffold configuration, pending actual `logos-scaffold` initialization and pinned LEZ/SPEL/Basecamp commits.
- `src/lean/AnonymousForum/ShamirTargets.lean`: next proof targets, not complete production verification.
- Development crypto adapters: mock receipts, mock threshold decryption, small local field, and placeholder certificate signature bytes remain to be replaced by the production choices documented in `src/SPEC.md`.
