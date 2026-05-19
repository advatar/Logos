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
- Finish remaining hackathon submission blockers locally: https://github.com/advatar/Logos/issues/32
- RISC0 membership guest proof-performance finish pass: https://github.com/advatar/Logos/issues/33
- Official LEZ local sequencer quickstart evidence: https://github.com/advatar/Logos/issues/34
- Top-level README blocker and Lean proof-surface documentation: https://github.com/advatar/Logos/issues/35
- Optional Noir proof-circuit icing: https://github.com/advatar/Logos/issues/36
- Narrated submission video generation: https://github.com/advatar/Logos/issues/37
- Clean-shell Basecamp runtime artifacts: https://github.com/advatar/Logos/issues/38
- Repository MIT license update: https://github.com/advatar/Logos/issues/39
- README inline submission video link: https://github.com/advatar/Logos/issues/40

## Active README Inline Video Link Pass

- [x] Locate the README submission video reference.
- [x] Change the plain video path into a clickable inline Markdown link.
- [x] Run focused local verification, commit, push, and update issue #40.

## Active Repository MIT License Pass

- [x] Confirm the repository has no top-level `LICENSE` file and identify repo-owned Rust license metadata.
- [x] Add a root MIT `LICENSE` file.
- [x] Align repo-owned Cargo license metadata with MIT while leaving vendored dependency licenses untouched.
- [x] Run focused local verification, commit, push, and update issue #39.

## Active Clean-Shell Basecamp Runtime Pass

- [x] Reproduce the clean-shell artifact check against durable cache paths.
- [x] Confirm the public release/action Basecamp app artifacts are durable but do not expose the QML inspector endpoint needed by the click-through harness.
- [x] Tighten `scripts/check_basecamp_inspector.py` so path-ready runtime artifacts are not reported as a full inspector-ready pass without successful click-through evidence.
- [x] Update `README.md` to distinguish clean-shell artifact discovery from the remaining inspector-enabled app requirement.
- [x] Run local verification, commit, push, and update issue #38.

## Active Narrated Submission Video Pass

- [x] Assess local video tooling and open submission issues.
- [x] Install or use a local video encoder path.
- [x] Add a reproducible narrated-video generator and produce the MP4 artifact.
- [x] Update README/status/issues so the video is no longer an open blocker.
- [x] Run local verification, commit, push, and close/update all issues that this pass resolves.

## Active Optional Noir Proof-Circuit Pass

- [x] Confirm no Noir package or `nargo` tool is currently present in the repo/shell.
- [x] Add an optional Noir circuit and structured local diagnostic.
- [x] Document Noir as additive icing in `README.md`, separate from the RISC0 submission path.
- [x] Expand Noir documentation with a dedicated docs page and top-level README highlight.
- [x] Run local verification, commit, push, and update issue #36.

## Active README Blocker And Lean Documentation Pass

- [x] Assess current top-level README, blocker state, and Lean proof modules.
- [x] Add explicit remaining blocker documentation to `README.md`.
- [x] Add concise explanation of how Lean 4 is used in the solution.
- [x] Run local verification, commit, push, and update issue #35.

## Active Official LEZ Local Sequencer Evidence Pass

- [x] Confirm the official LEZ wallet quickstart documents standalone local sequencer usage at `localhost:3040` rather than a public devnet/testnet RPC endpoint.
- [x] Update submission docs and readiness diagnostics to cite the official local sequencer path.
- [x] Run local verification, commit, push, and update issue #34.

## Active RISC0 Membership Guest Proof-Performance Pass

- [x] Pin the membership guest dependency graph so RISC0 guest-builder rustc 1.88 can build it.
- [x] Add/update local diagnostics for RISC0 proof-performance readiness.
- [x] Run local verification, commit, push, and update SC-PERF-01 tracking.

## Active Finish Remaining Submission Blockers Pass

- [x] Reassess every remaining open success-criteria issue and close anything now covered by local evidence.
- [x] Add concrete submission/demo artifacts for evaluator handoff without relying on GitHub CI: `src/scripts/collect_localnet_evidence.py`.
- [x] Attempt to convert local sequencer, Basecamp, RISC0, CU, and devnet/testnet blockers into executable local proof or precise final blockers.
- [x] Run local verification, commit, push, and update issue #32 plus affected success-criteria issues.

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
- [x] Extended local and optional workflow coverage with shell demo smoke coverage, structured CU script output, Basecamp static checks, and a required RISC0 feature check on stable Rust.
- [x] Installed and verified RISC0 tooling locally via `cargo-risczero 3.0.5`, `rzup 0.5.1`, `r0vm 3.0.5`, `cpp 2024.1.5`, and RISC0 Rust `1.94.1`.
- [x] Installed `logos-scaffold` and converted `src/scaffold.toml` to the current scaffold schema with the pinned LEZ commit.
- [x] Installed `logos-blockchain-circuits v0.4.2` into `~/.logos-blockchain-circuits`; `logos-scaffold setup` now satisfies the pinned LEZ probe.
- [x] Added a deployable `lez-framework` guest at `src/methods/guest/src/bin/lp0016_registry.rs`, switched `src/scaffold.toml` to `framework.kind = "lez-framework"`, and verified `cargo +stable check --manifest-path methods/guest/Cargo.toml`.
- [x] Ran `logos-scaffold build idl`; it now writes `src/registry/idl/lp0016_registry.json` from the hidden `__lssa_idl_print` test.
- [x] Built the LEZ registry RISC0 guest binary with `cd src/methods && cargo risczero build --manifest-path guest/Cargo.toml`; image/program id `dd914ffd8202da7c363d0aa7d9ad6222d1638b79f63a13f5dd24109896817e30` is recorded in `src/registry/program_ids/localnet.txt`.
- [x] Submitted the built registry program to a live local sequencer with `logos-scaffold deploy lp0016_registry --program-path methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin --json`.
- [x] Packaged the Basecamp UI as a real `ui_qml` LGX (`scripts/package_basecamp.sh`) and changed `Main.qml` to an embeddable `Item` root for Basecamp's `QQuickWidget` loader.
- [x] Boot-tested the official macOS `logos-basecamp` v0.1.1 runtime from the signed/notarized DMG.
- [x] Built an inspector-enabled Basecamp app locally without Nix, rebuilt matching `package_manager` and `main_ui` plugins, installed the design-system QML imports, and passed the LP-0016 QML inspector click-through.
- [x] Added `scripts/collect_localnet_evidence.py`, which starts the local sequencer directly when scaffold state is stale, deploys the registry guest, runs `RISC0_DEV_MODE=0 scripts/demo_e2e.sh`, writes `dist/submission/localnet_evidence.json`, and passes locally.
- [x] Added `scripts/check_risc0_proof_performance.py`, fixed the RISC0 host/guest input boundary with framed postcard bytes, and measured the real `RISC0_DEV_MODE=0` membership proof at `6.053s`, under the 10-second target.

## External Blockers

- [ ] CU measurement for `register_member` / `slash_member`: local registry deploy submission works, but current scaffold/wallet exposes no custom deployed-program invoke command or CU reporting path for these LP-0016 instructions. `scripts/measure_cu.sh` now reports this exact narrowed blocker as JSON after deployment submission.
- [ ] LEZ public devnet/testnet proof if reviewers insist on public network endpoints: the current official LEZ wallet quickstart documents standalone local sequencer usage at `localhost:3040`, and our `registry/program_ids/localnet.txt` plus `scripts/collect_localnet_evidence.py` cover that public developer path. `scripts/check_live_network_deploy.py` still reports exact missing endpoint/program-ID blockers for separate public devnet/testnet deployments: `LOGOS_LEZ_DEVNET_URL`, `LOGOS_LEZ_TESTNET_URL`, `registry/program_ids/devnet.txt`, and `registry/program_ids/testnet.txt`.
- [ ] Basecamp inspector click-through evidence: clean-shell artifact discovery now resolves durable cache/env paths for `logos-qt-mcp`, Basecamp runtime binaries, and Logos design-system QML imports. The public `logos-basecamp` v0.1.1 DMG and current action-built app are durable runtime artifacts, but they do not expose the QML inspector endpoint used by `app/basecamp-forum/ui-tests.mjs`. `scripts/check_basecamp_inspector.py` therefore reports `artifact_status=ready` while keeping `status=blocked` until an inspector-enabled Basecamp build passes `--run-click-through`.

## Current Verification

- `cd src && python3 -m unittest scripts.test_runtime_checks.RuntimeCheckTests.test_submission_video_is_documented_and_reproducible && cargo build --workspace`: passed.
- `cd src && cargo metadata --no-deps --format-version 1 >/tmp/logos-license-metadata.json && cargo build --workspace`: passed.
- `cd src && cargo check --manifest-path zk/membership-host/Cargo.toml`: passed.
- `cd src && rustup run stable cargo check --manifest-path zk/membership-guest/Cargo.toml && cargo +stable check --manifest-path methods/guest/Cargo.toml`: passed.
- `cd src && cargo test --workspace`: passed, 50 Rust tests across workspace crates plus doc-tests.
- `cd src && python3 -m unittest scripts/test_runtime_checks.py scripts/test_success_criteria.py scripts/test_phase_closure.py`: passed, 33 tests.
- `cd src && python3 scripts/check_basecamp_inspector.py --pretty`: passed script execution and reported `artifact_status=ready`, `status=blocked`, and missing matching inspector evidence for the durable public Basecamp runtime.
- `cd src && python3 scripts/check_basecamp_inspector.py --run-click-through --timeout 22 --pretty`: passed script execution and reported the narrowed public-runtime blocker, `Inspector not available at localhost:3768 after 15000ms`.
- `cd src && scripts/local_submission_gate.py`: passed and wrote `dist/submission/evidence.json` with all required local steps green; optional Basecamp remains blocked only on inspector-enabled click-through evidence.
- `cd src && python3 scripts/demo_e2e.py`: passed.
- `cd src && python3 -m unittest scripts/test_protocol.py scripts/test_basecamp_package.py scripts/test_runtime_checks.py scripts/test_success_criteria.py scripts/test_phase_closure.py`: passed, 40 tests.
- `cd src && python3 -m unittest scripts/test_runtime_checks.py scripts/test_success_criteria.py`: passed, 22 tests.
- `cd src && python3 -m unittest scripts/test_phase_closure.py`: passed, 9 tests.
- `cd src && python3 -m json.tool docs/success_criteria.json`: passed.
- `cd src && cargo build --workspace`: passed.
- `cd src && cargo test --workspace`: passed, 49 Rust tests across workspace crates plus doc-tests.
- `cd src && cargo test --manifest-path zk/membership-host/Cargo.toml`: passed.
- `cd src && rustup run stable cargo check --manifest-path zk/membership-host/Cargo.toml --features risc0`: passed.
- `cd src && cargo +stable check --manifest-path zk/membership-guest/Cargo.toml --features risc0`: passed.
- `cd src && python3 scripts/check_risc0_proof_performance.py --run-prover --fail-on-blocked`: passed, built image id words `[deb26c51, 1ccf4402, 6aea9e95, b3349864, 9d157525, e5fd593a, 0dafbd64, 470e5af3]`, and measured `proof_seconds=6.053`.
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
- `cd src && python3 scripts/collect_localnet_evidence.py`: passed and wrote `dist/submission/localnet_evidence.json` with local sequencer runtime `ready`, deploy submission ok, and `RISC0_DEV_MODE=0` demo ok.
- `cd src && python3 scripts/check_live_network_deploy.py`: passed script execution and reports structured blockers for missing `LOGOS_LEZ_DEVNET_URL`, `LOGOS_LEZ_TESTNET_URL`, `registry/program_ids/devnet.txt`, and `registry/program_ids/testnet.txt`.
- `cd src && python3 scripts/check_noir_icing.py --pretty`: passed with `nargo 1.0.0-beta.21` from `~/.nargo/bin`; the optional Noir post-binding circuit ran 2 tests.
- `cd src/noir/post_binding && ~/.nargo/bin/nargo test`: passed, 2 Noir tests.
- `cd src && python3 scripts/make_submission_video.py`: passed and wrote `../submission/lp0016-demo.mp4`, a 1280x720 H.264/AAC narrated video of about 119 seconds.
- `ffprobe -v error -show_entries format=duration,size -of json submission/lp0016-demo.mp4`: passed, duration `119.163084`, size `2337882`.
- `cd src && scripts/local_submission_gate.py`: passed and wrote `dist/submission/evidence.json` with all required local steps green, including `risc0_proof_performance` and `localnet_evidence`; optional Noir diagnostic is ready, while optional Basecamp runtime, live-network, and CU diagnostics still report the external artifact/custom invoke blockers.
- `cd src && cargo risczero --version`: passed, `cargo-risczero 3.0.5`.
- `cd src && ~/.cargo/bin/rzup --version && ~/.cargo/bin/r0vm --version`: passed, `rzup 0.5.1` and `risc0-r0vm 3.0.5`.
- `cd src && ~/.cargo/bin/logos-scaffold doctor`: reported 13 PASS, 4 WARN, 0 FAIL with localnet not running; remaining WARNs are LEZ cache working tree dirty plus sequencer/localnet reachability.
- `logos-basecamp v0.1.1` DMG boot smoke: previously passed; runtime reached "Logos Core started successfully".

## Notes

- The dependency-free Python simulator intentionally remains a structural reference using dev crypto; production crypto lives in Rust.
- The Lean Shamir module currently exposes a `ShamirSystem` proof contract rather than a Mathlib-backed concrete finite-field polynomial development. It is `sorry`-free and keeps downstream theorem names stable.
- Generated local artifacts remain ignored: `target/`, `.lake/`, `.scaffold/`, and Python `__pycache__/`.
- Nix was not installed in this pass because the macOS installer requires sudo/root and `sudo -n true` fails without a password; the Basecamp path is unblocked with manually built artifacts instead.
