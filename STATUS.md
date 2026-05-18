# Status

## Tracking

- Cleanup issue: https://github.com/advatar/Logos/issues/1
- Parent close-the-gaps issue: https://github.com/advatar/Logos/issues/2
- Current close-all-gaps pass: https://github.com/advatar/Logos/issues/3
- Runtime build continuation: https://github.com/advatar/Logos/issues/4
- No-Noir final blocker pass: https://github.com/advatar/Logos/issues/5
- Success criteria proof tracker: https://github.com/advatar/Logos/issues/6
- Per-criterion success issues: https://github.com/advatar/Logos/issues/7 through https://github.com/advatar/Logos/issues/28
- Phase 2-9 closure verification: https://github.com/advatar/Logos/issues/29

## Active Phase 2-9 Closure Verification Pass

- [x] Verify every requested Phase 2-9 follow-up against source and tests.
- [x] Add executable phase-closure tests/diagnostics for regressions: `src/scripts/test_phase_closure.py`.
- [x] Update `STATUS.md` to distinguish completed local phase work from real external blockers.
- [x] Run verification, commit, push, and update issue #29.

## Active Success Criteria Tracking Pass

- [x] Create one GitHub issue for each LP-0016 success criterion from `HACK.md`.
- [x] Add a tracked success-criteria matrix that maps every criterion to its issue and proof tests.
- [x] Add or tighten unit tests so local criteria have executable proof and external criteria have explicit readiness diagnostics.
- [x] Run verification for the local proof suite.
- [x] Commit, push, and update GitHub issues (#6-#28).

## Active No-Noir Final Blocker Pass

- [x] Determine the smallest deployable LEZ guest / `lez-framework` migration path that keeps the Rust 1.82 workspace green. The generated `lez-framework` template needs a workspace-lints patch, then the current local build stops on `logos-blockchain-circuits v0.4.1` versus the framework's `v0.4.2` requirement.
- [x] Add local scripts/tests for LEZ guest build/deploy/CU capture or record exact scaffold blockers. `scripts/check_lez_runtime.py` now reports framework-kind, guest-source, circuits-version, binary, and localnet blockers as JSON; `scripts/measure_cu.sh` delegates to it before claiming CU readiness.
- [x] Improve Basecamp QML inspector click-through automation or record the exact missing harness/tooling. `app/basecamp-forum/ui-tests.mjs` now clicks through the LP-0016 flow when `logos-qt-mcp` and a Basecamp app binary are available; `scripts/check_basecamp_inspector.py` reports missing runtime pieces as JSON.
- [x] Wire app-flow RISC0 receipt bytes into the boundary where feasible while keeping local mock demos available. `protocol-core`, `moderation-sdk`, `lp0016-membership-host`, and the Basecamp core module now expose serialized receipt byte attachment/conversion paths.
- [x] Run full verification for the local pieces that can run without the missing external runtimes.
- [x] Commit, push, and update GitHub issues (#2, #3, #4, #5).

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

- [ ] LEZ registry deployment / CU measurement: LEZ binaries from the previous pinned setup exist, but the deployable path is still blocked by three concrete checks: `scaffold.toml` is still `framework.kind = "default"`, this repo has no `methods/guest/src/bin/lp0016_registry.rs`, and the installed `~/.logos-blockchain-circuits` is `v0.4.1` while the current `lez-framework` probe expects `v0.4.2`. `scripts/check_lez_runtime.py` and `scripts/measure_cu.sh` now report those blockers as JSON.
- [ ] Real SPEL macro flip: `logos-scaffold build idl` is available, but v0.1.1 only emits IDL for `framework.kind = "lez-framework"` and expects the generated client crate layout. This repo stays on `framework.kind = "default"` until the registry can migrate without breaking `cargo build --workspace`.
- [ ] Basecamp automated click-through: official runtime v0.1.1 boots and the app packages as LGX. The LP-0016 click-through spec now exists, but running it is blocked locally by missing `logos-qt-mcp`, missing `nix`, and no `LOGOS_BASECAMP_APP` executable path. `scripts/check_basecamp_inspector.py` reports the exact missing pieces.
- [ ] Full RISC0 proof generation from the app flow: feature-gated host checks pass and generated receipt bytes can now be converted/attached to `ZkReceipt::Risc0`; the remaining blocker is producing the final guest image and serialized receipt from the application flow rather than the current local host feature test.

## Current Verification

- `cd src && python3 scripts/demo_e2e.py`: passed.
- `cd src && python3 -m unittest scripts/test_protocol.py scripts/test_basecamp_package.py scripts/test_runtime_checks.py scripts/test_success_criteria.py scripts/test_phase_closure.py`: passed, 36 tests.
- `cd src && python3 -m unittest scripts/test_phase_closure.py`: passed, 9 tests.
- `cd src && python3 -m json.tool docs/success_criteria.json`: passed.
- `cd src && cargo build --workspace`: passed.
- `cd src && cargo test --workspace`: passed, 49 Rust tests across workspace crates plus doc-tests.
- `cd src && cargo test --manifest-path zk/membership-host/Cargo.toml`: passed.
- `cd src && scripts/demo_e2e.sh`: passed, including registry JSON emission and `slash-verifier` CLI verification.
- `cd src && RISC0_DEV_MODE=0 scripts/demo_e2e.sh`: passed, including feature-gated RISC0 host tests under `cargo +stable`.
- `cd src/lean && lake build`: passed.
- `cd src && ./scripts/package_basecamp.sh /tmp/lp0016-basecamp`: passed and emitted `lp0016-anon-forum-demo.lgx`.
- `cd src && ./scripts/measure_cu.sh`: passed script execution and emitted structured blocked JSON for `framework.kind = "default"`, missing `methods/guest/src/bin/lp0016_registry.rs`, and circuits `v0.4.1` versus expected `v0.4.2`.
- `cd src && python3 scripts/check_lez_runtime.py && python3 scripts/check_basecamp_inspector.py`: passed script execution; both intentionally reported blocked JSON for missing external runtime pieces.
- `cd src && cargo risczero --version`: passed, `cargo-risczero 3.0.5`.
- `cd src && ~/.cargo/bin/rzup --version && ~/.cargo/bin/r0vm --version`: passed, `rzup 0.5.1` and `risc0-r0vm 3.0.5`.
- `cd src && ~/.cargo/bin/logos-scaffold build idl`: ran; skipped because scaffold framework kind is `default`.
- `cd src && ~/.cargo/bin/logos-scaffold deploy lp0016_registry --json`: failed with the expected current blocker, missing `methods/guest/src/bin`.
- `cd src && ~/.cargo/bin/logos-scaffold doctor`: reported 13 PASS, 4 WARN, 0 FAIL with localnet not running; remaining WARNs are LEZ cache working tree dirty plus sequencer/localnet reachability.
- `logos-basecamp v0.1.1` DMG boot smoke: previously passed; runtime reached "Logos Core started successfully".

## Notes

- The dependency-free Python simulator intentionally remains a structural reference using dev crypto; production crypto lives in Rust.
- The Lean Shamir module currently exposes a `ShamirSystem` proof contract rather than a Mathlib-backed concrete finite-field polynomial development. It is `sorry`-free and keeps downstream theorem names stable.
- Generated local artifacts remain ignored: `target/`, `.lake/`, `.scaffold/`, and Python `__pycache__/`.
