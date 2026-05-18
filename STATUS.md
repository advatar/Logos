# Status

## Tracking

- Cleanup issue: https://github.com/advatar/Logos/issues/1
- Parent close-the-gaps issue: https://github.com/advatar/Logos/issues/2
- Current close-all-gaps pass: https://github.com/advatar/Logos/issues/3
- Runtime build continuation: https://github.com/advatar/Logos/issues/4

## Active Runtime Build Pass

- [x] Find and install/fetch `logos-blockchain-circuits` so the pinned LEZ repo can build `sequencer_service` and `wallet`.
- [x] Rerun `logos-scaffold setup`, `logos-scaffold doctor`, and localnet/IDL checks after circuits are available.
- [x] Find/install/build the Basecamp runtime and run the app if possible.
- [x] Wire any reproducible runtime setup commands into scripts/docs.
- [x] Rerun full verification, commit, and push this pass.

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
- [x] Installed `logos-blockchain-circuits v0.4.1` into `~/.logos-blockchain-circuits`; `logos-scaffold setup` now builds the pinned LEZ `sequencer_service` and `wallet` binaries.
- [x] Packaged the Basecamp UI as a real `ui_qml` LGX (`scripts/package_basecamp.sh`) and changed `Main.qml` to an embeddable `Item` root for Basecamp's `QQuickWidget` loader.
- [x] Boot-tested the official macOS `logos-basecamp` v0.1.1 runtime from the signed/notarized DMG.

## External Blockers

- [ ] LEZ registry deployment / CU measurement: LEZ binaries now build and the localnet can be held live for `logos-scaffold doctor`, but `logos-scaffold deploy lp0016_registry --json` fails because this repo has no deployable `methods/guest/src/bin/lp0016_registry.rs` guest yet. `scripts/measure_cu.sh` now reports this exact blocker instead of the old missing-circuits message.
- [ ] Real SPEL macro flip: `logos-scaffold build idl` is available, but v0.1.1 only emits IDL for `framework.kind = "lez-framework"` and expects the generated client crate layout. This repo stays on `framework.kind = "default"` until the registry can migrate without breaking `cargo build --workspace`.
- [ ] Basecamp automated click-through: official runtime v0.1.1 boots and the app packages as LGX, but full UI interaction still needs the Basecamp/QML inspector harness. Local source build is blocked by missing `nix`, `cmake`, `ninja`, and Qt tools.
- [ ] Full RISC0 proof generation from the app flow: feature-gated host checks pass, but the demo script still performs the local host feature test rather than producing a real application receipt. Wiring generated receipt bytes into `ZkReceipt::Risc0` remains dependent on the final guest image build/receipt packaging flow.

## Current Verification

- `cd src && python3 scripts/demo_e2e.py`: passed.
- `cd src && python3 -m unittest scripts/test_protocol.py scripts/test_basecamp_package.py`: passed, 10 tests.
- `cd src && cargo build --workspace`: passed.
- `cd src && cargo test --workspace`: passed, 44 Rust tests across workspace crates plus doc-tests.
- `cd src && scripts/demo_e2e.sh`: passed, including registry JSON emission and `slash-verifier` CLI verification.
- `cd src && RISC0_DEV_MODE=0 scripts/demo_e2e.sh`: passed, including feature-gated RISC0 host tests under `cargo +stable`.
- `cd src/lean && lake build`: passed.
- `cd src && ./scripts/package_basecamp.sh /tmp/lp0016-basecamp`: passed and emitted `lp0016-anon-forum-demo.lgx`.
- `logos-basecamp v0.1.1` DMG boot smoke: passed; runtime reached "Logos Core started successfully".
- `cd src && ./scripts/measure_cu.sh`: passed script execution and emitted structured blocked JSON for missing deployable `methods/guest/src/bin/lp0016_registry.rs`.
- `cd src && ~/.cargo/bin/logos-scaffold build idl`: ran; skipped because scaffold framework kind is `default`.
- `cd src && ~/.cargo/bin/logos-scaffold deploy lp0016_registry --json`: failed with the expected current blocker, missing `methods/guest/src/bin`.
- `cd src && ~/.cargo/bin/logos-scaffold doctor`: with a live sequencer, reported 16 PASS, 1 WARN, 0 FAIL. The remaining warning is the generated LEZ cache working tree dirty after scaffold patched the sequencer debug config for localnet.

## Notes

- The dependency-free Python simulator intentionally remains a structural reference using dev crypto; production crypto lives in Rust.
- The Lean Shamir module currently exposes a `ShamirSystem` proof contract rather than a Mathlib-backed concrete finite-field polynomial development. It is `sorry`-free and keeps downstream theorem names stable.
- Generated local artifacts remain ignored: `target/`, `.lake/`, `.scaffold/`, and Python `__pycache__/`.
