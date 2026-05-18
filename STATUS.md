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

### Phase 2 — Threshold ElGamal + DLEQ

- [ ] New module `protocol-core::threshold` with: distributed key generation output (`ThresholdPublicKey`, per-moderator `ShareSecretKey`/`SharePublicKey`), hybrid encryption of `(x, y)` payload (`KEM = H("kem", rY)` ⊕ XOF), partial decryption `D_i = s_i C1`, DLEQ proof that `D_i` matches `SharePublicKey_i` with the same exponent, aggregator that interpolates over public-key shares to recover `D = sC1` and decrypts.
- [ ] Add `ChaCha20-XOF`-style KDF on top of `Sha256` keyed by `KEM` for the hybrid leg (no extra deps; SHA-256 counter mode is fine).
- [ ] Replace `DevThresholdOracle` in `moderation-sdk` with a `ThresholdSession` that holds the public threshold key and the local moderator's share key; `aggregate_certificate` collects partial decryptions + DLEQ proofs and verifies them.
- [ ] Add unit tests: encrypt-then-threshold-decrypt round trip, DLEQ rejects a partial decryption with a mismatched exponent, fewer than N partials fail aggregation.

### Phase 3 — Registry roots + Merkle paths

- [ ] Add a binary Merkle tree module (`protocol-core::merkle`) over `Hash32` with deterministic node hashing (`digest("merkle-node", left, right)`).
- [ ] Extend `RegistryState` with `membership_root` and `revocation_root` recomputed on every mutation; expose `prove_membership` and `prove_non_membership` (sparse-Merkle path).
- [ ] Bump the canonical public inputs hash to include `membership_root`, `revocation_root`, and `threshold_public_key_hash` per `SPEC.md §5`.
- [ ] Tests: membership proof verifies against the current root, non-membership proof verifies against the revocation root, stale proof rejected after revoke.

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
