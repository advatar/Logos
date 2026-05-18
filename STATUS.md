# Status

## Tracking

- Cleanup issue: https://github.com/advatar/Logos/issues/1
- Parent close-the-gaps issue: https://github.com/advatar/Logos/issues/2
- Current close-all-gaps pass: https://github.com/advatar/Logos/issues/3

## Cleared In This Pass

- [x] Replaced trusted-dealer-only threshold setup with an auditable Pedersen-style DKG transcript API: `DealerShares::pedersen_dkg`, `PedersenDkgTranscript`, and transcript tamper tests.
- [x] Added revocation-tree non-membership proofs using predecessor/successor Merkle witnesses with sorted-index adjacency checks.
- [x] Enforced revocation non-membership in `crates/risc0-statement`.
- [x] Replaced post-envelope `MockZkReceipt` usage with `ZkReceipt::{Mock,Risc0}` and a common `verify_public_inputs` surface.
- [x] Closed Phase 6 storage gaps: `post/`, `vote/`, `cert/`, `slash/` namespaces plus `RetryQueue` / `MemoryRetryQueue`.
- [x] Added slash-verifier CLI smoke coverage through `scripts/demo_e2e.sh` using JSON emitted by `registry-sim`.
- [x] Added Lean `Shamir.lean` and `Slash.lean` theorem surfaces, building locally without `sorry`.
- [x] Replaced the single-screen Basecamp placeholder with a deterministic 9-screen QML flow.
- [x] Added `app/basecamp-forum/core-module`, a Rust C ABI bridge point over `moderation-sdk`.
- [x] Extended CI with shell demo smoke coverage, structured CU script output, Basecamp static checks, and an optional RISC0 feature check on stable Rust.
- [x] Installed and verified RISC0 tooling locally via `cargo-risczero 3.0.5`, `rzup 0.5.1`, `r0vm 3.0.5`, `cpp 2024.1.5`, and RISC0 Rust `1.94.1`.
- [x] Installed `logos-scaffold` and converted `src/scaffold.toml` to the current scaffold schema with the pinned LEZ commit.

## External Blockers

- [ ] LEZ localnet/devnet CU measurement: `logos-scaffold setup` fails while building the pinned LEZ repo because `logos-blockchain-circuits` is not installed. The build script requests either `LOGOS_BLOCKCHAIN_CIRCUITS` or `~/.logos-blockchain-circuits`. Until that artifact is present, `sequencer_service`, `wallet`, `logos-scaffold localnet start`, real CU capture, and deployment are blocked.
- [ ] Real SPEL macro flip: `logos-scaffold` is installed and the repo has a valid scaffold config, but the LEZ build failure above prevents generating/validating real SPEL macro output. The Rust registry crate and hand-written IDL remain the checked local source of truth.
- [ ] Basecamp runtime launch: no Basecamp executable is installed in this environment. QML/static flow and Rust core-module tests are present; runtime launch still needs the Basecamp package.
- [ ] Full RISC0 proof generation from the app flow: feature-gated host checks pass, but the demo script still performs the local host feature test rather than producing a real application receipt. Wiring generated receipt bytes into `ZkReceipt::Risc0` remains dependent on the final guest image build/receipt packaging flow.

## Current Verification

- `cd src && python3 scripts/demo_e2e.py`: passed.
- `cd src && python3 -m unittest scripts/test_protocol.py`: passed, 7 tests.
- `cd src && cargo build --workspace`: passed.
- `cd src && cargo test --workspace`: passed, 44 Rust tests across workspace crates plus doc-tests.
- `cd src && scripts/demo_e2e.sh`: passed, including registry JSON emission and `slash-verifier` CLI verification.
- `cd src && RISC0_DEV_MODE=0 scripts/demo_e2e.sh`: passed, including feature-gated RISC0 host tests under `cargo +stable`.
- `cd src/lean && lake build`: passed.
- `cd src && ./scripts/measure_cu.sh`: passed script execution and emitted structured blocked JSON for missing LEZ binaries/circuits.
- `cd src && ~/.cargo/bin/logos-scaffold doctor`: parsed the scaffold config and passed repo/pin checks, then failed only on missing LEZ `sequencer_service` and `wallet` binaries caused by the circuits blocker above.

## Notes

- The dependency-free Python simulator intentionally remains a structural reference using dev crypto; production crypto lives in Rust.
- The Lean Shamir module currently exposes a `ShamirSystem` proof contract rather than a Mathlib-backed concrete finite-field polynomial development. It is `sorry`-free and keeps downstream theorem names stable.
- Generated local artifacts remain ignored: `target/`, `.lake/`, `.scaffold/`, and Python `__pycache__/`.
