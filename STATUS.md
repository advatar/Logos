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
- Dependency install / runtime unblock pass: https://github.com/advatar/Logos/issues/30
- Final local submission readiness pass: https://github.com/advatar/Logos/issues/31

## Active Final Local Submission Readiness Pass

- [x] Replace GitHub Actions as an acceptance dependency with a reproducible local integration gate and submission evidence artifacts: `src/scripts/local_submission_gate.py`.
- [x] Make Basecamp runtime artifact discovery durable and documented rather than relying on transient `/tmp` fallback paths.
- [x] Re-check the LEZ deploy/CU path and either close the deployable guest gap or preserve exact local-runtime blockers in the submission gate.
- [x] Update README/status/success tracking so evaluators can run the local gate and see the remaining human submission artifact.
- [x] Run local verification, commit, push, and update issue #31.

## Active Dependency Install / Runtime Unblock Pass

- [x] Re-run dependency diagnostics and identify installable blockers.
- [x] Install or fetch matching Logos circuits expected by the LEZ probe.
- [x] Install or fetch Basecamp inspector dependencies where possible (`logos-qt-mcp`, Basecamp source/runtime, inspector-enabled app, design-system QML, package tooling).
- [x] Re-run runtime diagnostics and local build/test after dependency changes.
- [x] Commit, push, and update issue #30 with what was installed and what still needs system-level/manual action.

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
- [x] Extended CI with shell demo smoke coverage, structured CU script output, Basecamp static checks, and a required RISC0 feature check on stable Rust.
- [x] Installed and verified RISC0 tooling locally via `cargo-risczero 3.0.5`, `rzup 0.5.1`, `r0vm 3.0.5`, `cpp 2024.1.5`, and RISC0 Rust `1.94.1`.
- [x] Installed `logos-scaffold` and converted `src/scaffold.toml` to the current scaffold schema with the pinned LEZ commit.
- [x] Installed `logos-blockchain-circuits v0.4.2` into `~/.logos-blockchain-circuits`; `logos-scaffold setup` now satisfies the pinned LEZ probe.
- [x] Added a deployable `lez-framework` guest at `src/methods/guest/src/bin/lp0016_registry.rs`, switched `src/scaffold.toml` to `framework.kind = "lez-framework"`, and verified `cargo +stable check --manifest-path methods/guest/Cargo.toml`.
- [x] Ran `logos-scaffold build idl`; it now writes `src/registry/idl/lp0016_registry.json` from the hidden `__lssa_idl_print` test.
- [x] Built the LEZ registry RISC0 guest binary with `cd src/methods && cargo risczero build --manifest-path guest/Cargo.toml`; image/program id `d4e94a8c0e642f6a440882a69ec0ce20148d343e5369e9b8a7702d108ddd01ec` is recorded in `src/registry/program_ids/localnet.txt`.
- [x] Submitted the built registry program to a live local sequencer with `logos-scaffold deploy lp0016_registry --program-path methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin --json`.
- [x] Packaged the Basecamp UI as a real `ui_qml` LGX (`scripts/package_basecamp.sh`) and changed `Main.qml` to an embeddable `Item` root for Basecamp's `QQuickWidget` loader.
- [x] Boot-tested the official macOS `logos-basecamp` v0.1.1 runtime from the signed/notarized DMG.
- [x] Built an inspector-enabled Basecamp app locally without Nix, rebuilt matching `package_manager` and `main_ui` plugins, installed the design-system QML imports, and passed the LP-0016 QML inspector click-through.

## External Blockers

- [ ] CU measurement for `register_member` / `slash_member`: local registry deploy submission works, but current scaffold/wallet exposes no custom deployed-program invoke command or CU reporting path for these LP-0016 instructions. `scripts/measure_cu.sh` now reports this exact narrowed blocker as JSON after deployment submission.
- [ ] LEZ devnet/testnet proof: `registry/program_ids/localnet.txt` is recorded, but `registry/program_ids/devnet.txt` and `registry/program_ids/testnet.txt` still require live network deployment.
- [ ] Basecamp inspector runtime artifacts: durable env/cache discovery is implemented, but this shell currently lacks `logos-qt-mcp`, a built `LogosBasecamp` binary, and design-system QML paths unless supplied through `LOGOS_BASECAMP_CACHE` or explicit env vars.
- [ ] Full RISC0 proof generation from the app flow: feature-gated host checks pass and generated receipt bytes can now be converted/attached to `ZkReceipt::Risc0`; the remaining blocker is producing the final guest image and serialized receipt from the application flow rather than the current local host feature test.
- [ ] Narrated video demo: must be recorded by the builder and linked from the README before final submission.

## Current Verification

- `cd src && python3 scripts/demo_e2e.py`: passed.
- `cd src && python3 -m unittest scripts/test_protocol.py scripts/test_basecamp_package.py scripts/test_runtime_checks.py scripts/test_success_criteria.py scripts/test_phase_closure.py`: passed, 38 tests.
- `cd src && python3 -m unittest scripts/test_phase_closure.py`: passed, 9 tests.
- `cd src && python3 -m json.tool docs/success_criteria.json`: passed.
- `cd src && cargo build --workspace`: passed.
- `cd src && cargo test --workspace`: passed, 49 Rust tests across workspace crates plus doc-tests.
- `cd src && cargo test --manifest-path zk/membership-host/Cargo.toml`: passed.
- `cd src && rustup run stable cargo check --manifest-path zk/membership-host/Cargo.toml --features risc0`: passed.
- `cd src && scripts/demo_e2e.sh`: passed, including registry JSON emission and `slash-verifier` CLI verification.
- `cd src && RISC0_DEV_MODE=0 scripts/demo_e2e.sh`: passed, including feature-gated RISC0 host tests under `cargo +stable`.
- `cd src/lean && lake build`: passed.
- `cd src && ./scripts/package_basecamp.sh dist/basecamp`: passed and emitted `lp0016-anon-forum-demo.lgx`.
- `cd src && DYLD_LIBRARY_PATH=/tmp/logos-package-install/lib /tmp/logos-package-install/bin/lgx verify /tmp/lp0016-basecamp/lp0016-anon-forum-demo.lgx`: passed, package structure valid and unsigned.
- `cd src && LOGOS_QT_MCP=/tmp/logos-qt-mcp-inspect LOGOS_BASECAMP_APP=/tmp/logos-basecamp-build/LogosBasecamp QML2_IMPORT_PATH=/tmp/logos-design-system/src/qml QML_IMPORT_PATH=/tmp/logos-design-system/src/qml DYLD_LIBRARY_PATH=/tmp/modules/package_manager:/tmp/plugins/main_ui:/opt/homebrew/opt/gettext/lib:/opt/homebrew/opt/icu4c/lib:/tmp/logos-package-manager-install/lib:/tmp/logos-package-install/lib:/tmp/logos-liblogos-install/lib node app/basecamp-forum/ui-tests.mjs --ci /tmp/logos-basecamp-build/LogosBasecamp`: passed, 2 tests.
- `cd src && python3 scripts/check_lez_runtime.py --pretty`: passed and reported `ready` while the local sequencer TCP listener was running; scaffold pid status was stale but TCP listener readiness was true.
- `cd src && ./scripts/measure_cu.sh`: passed script execution, submitted the registry deploy through wallet mode, and emitted structured blocked JSON for the remaining custom invoke/CU reporting gap.
- `cd src && cargo +stable check --manifest-path methods/guest/Cargo.toml`: passed.
- `cd src/methods && cargo risczero build --manifest-path guest/Cargo.toml`: passed and emitted `methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin`.
- `cd src && ~/.cargo/bin/logos-scaffold build idl`: passed and wrote `registry/idl/lp0016_registry.json`.
- `cd src && ~/.cargo/bin/logos-scaffold deploy lp0016_registry --program-path methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin --json`: passed and returned `{"status":"submitted","program":"lp0016_registry","tx":null}`.
- `cd src && scripts/local_submission_gate.py`: passed and wrote `dist/submission/evidence.json` with all required local steps green; optional Basecamp runtime and CU diagnostics still report the external artifact/custom invoke blockers.
- `cd src && cargo risczero --version`: passed, `cargo-risczero 3.0.5`.
- `cd src && ~/.cargo/bin/rzup --version && ~/.cargo/bin/r0vm --version`: passed, `rzup 0.5.1` and `risc0-r0vm 3.0.5`.
- `cd src && ~/.cargo/bin/logos-scaffold doctor`: reported 13 PASS, 4 WARN, 0 FAIL with localnet not running; remaining WARNs are LEZ cache working tree dirty plus sequencer/localnet reachability.
- `logos-basecamp v0.1.1` DMG boot smoke: previously passed; runtime reached "Logos Core started successfully".

## Notes

- The dependency-free Python simulator intentionally remains a structural reference using dev crypto; production crypto lives in Rust.
- The Lean Shamir module currently exposes a `ShamirSystem` proof contract rather than a Mathlib-backed concrete finite-field polynomial development. It is `sorry`-free and keeps downstream theorem names stable.
- Generated local artifacts remain ignored: `target/`, `.lake/`, `.scaffold/`, and Python `__pycache__/`.
- Nix was not installed in this pass because the macOS installer requires sudo/root and `sudo -n true` fails without a password; the Basecamp path is unblocked with manually built artifacts instead.
