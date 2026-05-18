# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repo orientation

This is the LP-0016 anonymous forum starter — a forum-agnostic moderation library plus a Basecamp app that does off-chain posting/moderation with on-chain slash only at revocation time. The "working" code today is the local protocol simulator (Python) and the pure Rust state-machine model; LEZ, SPEL, RISC0, and Basecamp directories are deliberate boundary stubs.

All source lives under `src/` — almost every build/test command runs from there, not the repo root. The CI workflow at `.github/workflows/ci.yml` uses `working-directory: src` for every job.

Authoritative docs (do not duplicate when answering questions — read these instead):

- `src/SPEC.md` — concrete implementation decisions: toolchain pins, production vs. dev crypto choices, canonical transcript serialization, LEZ/SPEL/Basecamp targets, RISC0 circuit statement, acceptance criteria, threat model.
- `REPO.md` — narrative description of the original starter delivery and the prize requirements.
- `STATUS.md` — current active task list, verification results, and a placeholder inventory describing which components are still stubs vs. real implementations.
- `AGENTS.md` — working rules (see "Working rules" below).
- `src/docs/protocol.md`, `api.md`, `threat-model.md`, `performance.md` — protocol-level reference.

## Common commands

Run from `src/` unless noted:

```bash
# Python simulator (dependency-free, Python 3.10+)
python3 scripts/demo_e2e.py
python3 -m unittest scripts/test_protocol.py
python3 -m unittest scripts/test_protocol.py TestProtocol.<method>   # single test

# Rust workspace (toolchain pinned to 1.82.0 via rust-toolchain.toml)
cargo build --workspace
cargo test --workspace
cargo test -p protocol-core                  # single crate
cargo run -p registry-sim                    # local registry simulation binary
cargo run -p slash-verifier -- <args>        # slash-bundle verifier CLI

# Lean 4 proofs
cd lean && lake build
```

Note: `clap` is pinned to `=4.5.50` in `Cargo.toml` so the workspace builds under Rust 1.82.0 — don't bump it without also bumping the toolchain.

## Working rules (from AGENTS.md)

- After assessing a request, update `STATUS.md` tasks **before** implementing, and open a GitHub issue with the plan.
- `git add` + commit + push after creating or editing files; verify builds locally before claiming completion.
- Never stage, commit, or alter files unrelated to the current task — other agents may be working in this repo.
- When tasks are unchecked, complete them one by one after passing tests; don't stop mid-list.
- Add unit tests when adding new functionality.
- Only ask the user when a decision is blocking progress; otherwise keep going.

## Architecture

The protocol implements an **anonymous forum with K-strike slashing**: each member registers a hidden Shamir polynomial commitment; each post leaks one encrypted Shamir share; N-of-M moderators threshold-decrypt only offending shares; K certificates from the same member reconstruct the polynomial, recompute the commitment, and trigger on-chain revocation + retroactive linkability of *only* that member's prior posts. The full construction is in `src/docs/protocol.md`.

### Three parallel implementations of the same state machine

1. **`src/scripts/lp0016_sim.py`** — the runnable reference. End-to-end Python simulator with no third-party deps. `demo_e2e.py` exercises registration → post → partial-cert rejection → N-of-M certificate → K-strike slash → revocation → post rejection after revocation → retroactive linkability → two independent forum instances with different `(K, N, M)`. Treat this as the executable spec.

2. **Rust workspace under `src/crates/`** — the production implementation target. Layered so the pure protocol has no Logos/LEZ/RISC0/Basecamp dependencies:
   - `protocol-core` (no_std-friendly, deps: `sha2`, `serde`, `thiserror`, `hex`): `field`, `shamir`, `hash`, `cert`, `state`, `types`. Models the slash/certificate/Shamir state machine. **The other crates and the Lean proofs must preserve this state machine's behavior.**
   - `moderation-sdk` (depends only on `protocol-core` + `anyhow`/`serde_json`): forum-agnostic façade with `OffchainStore` trait + `MemoryStore`. Production should implement `OffchainStore` for Logos Storage/Delivery; UI must go through this surface, not poke `protocol-core` directly.
   - `registry-sim` — local binary that simulates the LEZ registry program for tests/demos.
   - `slash-verifier` — CLI shell for slash-bundle verification.

3. **`src/lean/AnonymousForum/`** — Lean 4 proofs of the protocol state machine (`Basic.lean` definitions, `Invariants.lean` no-`sorry` invariant proofs, `ShamirTargets.lean` next-target theorems). Lean covers the formal protocol/state-machine layer only — cryptographic primitives (hashes, signatures, threshold ElGamal, RISC0 receipts) are stated as assumptions, not proved.

### Boundary stubs (intentional placeholders — see `STATUS.md` placeholder inventory before touching)

- `src/registry/lez-program-stub/` — LEZ/SPEL registry boundary. Will be replaced by a SPEL-annotated LEZ program generated via `logos-scaffold build idl`.
- `src/zk/membership-guest/` and `src/zk/membership-host/` — RISC0 guest/host placeholders. Production circuit statement (public/private inputs, checks, perf plan) is in `src/SPEC.md §5`.
- `src/app/basecamp-forum/` — minimal Basecamp QML placeholder. Must only call `moderation-sdk`, never `protocol-core` directly.
- `src/scripts/measure_cu.sh` — compute-unit measurement, pending deployed localnet/testnet.
- `src/scaffold.toml` — placeholder; real LEZ/SPEL/Basecamp commits get pinned after `logos-scaffold setup`.

### Dev vs. production crypto

Rust `protocol-core` is on the production targets for three primitives:

- **Ristretto255 scalar field** (`curve25519_dalek::Scalar`), hashed-to-field via SHA-256 wide reduction (two domain-separated halves).
- **Ed25519 moderator signatures** (`ed25519_dalek::SigningKey`/`VerifyingKey`), signing the canonical statement hash. `ForumConfig.moderators` holds `ModeratorIdentity { id, verifying_key, share_public_key }`.
- **Threshold ElGamal + Chaum–Pedersen DLEQ** (`protocol-core::threshold`): `ThresholdPublicKey`, per-moderator `ShareSecretKey`/`SharePublicKey`, hybrid encryption of the 64-byte `(x, y)` payload with SHA-256 KDF, partial decryptions with DLEQ proofs bound to the post's domain seed, Lagrange-at-zero aggregator. `AnonymousPostEnvelope` carries the real `Ciphertext`; `ModerationCertificate` carries `Vec<PartialDecryption>` (DLEQ-proven). `cert.revealed_share(forum)` aggregates trustlessly; `slash` no longer trusts an input share.

Registry state binds **Merkle roots** for membership and revocation. `RegistryState::membership_root()` and `revocation_root()` derive from the current sets via `protocol-core::merkle` (sorted, de-duplicating, leaf/node domain-separated). `AnonymousPostEnvelope::build` takes the registry and binds both roots into the public-inputs hash. Non-membership proofs against the revocation root are deferred until the RISC0 guest fixes the in-circuit encoding.

Still dev/mock and pending replacement (`STATUS.md` tracks):

- `MockZkReceipt` — stand-in for the RISC0 membership/post receipt.
- `DealerShares::trusted` — single-trusted-party DKG. Fine for tests/demos; production needs Pedersen DKG so no party ever sees `s`.

The Python simulator stays on the dev field (`2^61 - 1`), the dev `ModeratorKey` SHA-256-derived signature, and the `ThresholdOracle` HashMap stand-in. It's the executable structural reference. Python mirrors all transcript bindings the Rust core enforces (forum_id, mod_set_version, threshold_public_key_hash, K/N), so a protocol-level change must update both. Commitment *bytes* will not match between Python and Rust for the same seed because the fields differ — that's expected.

### Cross-cutting invariants worth preserving

- **Canonical transcript** (`src/SPEC.md §3`): consensus bytes use the explicit `lp0016:<domain>:v1 || field_count || …` framing — never `serde_json`, `bincode`, or map-iteration order. Certificate statements must include every replay-sensitive field listed there.
- **Forum-agnostic SDK**: nothing in `moderation-sdk` or `protocol-core` may assume a particular content shape; posts are opaque envelopes keyed by `forum_id`/`post_id` in storage.
- **Domain separation**: every signed/proved statement includes `forum_id` + `mod_set_version` to block cross-forum and cross-version replay.
