# Status

## Active Tasks

- [x] Improve repository `.gitignore` coverage for Rust, Python, editor, and OS artifacts.
- [x] Move CI workflow to root `.github/workflows/ci.yml` with `src` working-directory defaults.
- [x] Verify the Python simulator demo and unit tests.
- [x] Verify the Rust workspace build and tests.
- [x] Verify the registry simulator binary.
- [x] Verify the Lean scaffold build.
- [x] Commit and push the completed cleanup and verification state.

Tracking issue: https://github.com/advatar/Logos/issues/1

## Verification Results

- `cd src && python3 scripts/demo_e2e.py`: passed.
- `cd src && python3 -m unittest scripts/test_protocol.py`: passed, 6 tests.
- `cd src && cargo build --workspace`: passed with Rust 1.82.0 after pinning `clap` to `=4.5.50`.
- `cd src && cargo test --workspace`: passed, including `protocol-core` unit tests and doc-tests.
- `cd src && cargo run -p registry-sim`: passed.
- `cd src/lean && lake build`: passed.

## Placeholder Inventory

Based on `REPO.md` and the repository files, this is still a starter implementation rather than a complete LP-0016 prize submission. The working code is the local protocol simulator and Rust protocol model; the following areas remain placeholders or scaffolding:

- `src/registry/lez-program-stub/`: LEZ/SPEL registry boundary stub, pending a generated SPEL-annotated LEZ program.
- `src/zk/membership-guest/` and `src/zk/membership-host/`: RISC0 guest/host placeholders, pending real membership and post receipts.
- `src/app/basecamp-forum/`: minimal Basecamp QML placeholder, pending real SDK-backed forum workflows.
- `src/scripts/measure_cu.sh`: compute-unit measurement placeholder, pending a deployed localnet/testnet flow.
- `src/scaffold.toml`: placeholder scaffold configuration, pending actual `logos-scaffold` initialization and pinned LEZ/SPEL/Basecamp commits.
- `src/lean/AnonymousForum/ShamirTargets.lean`: next proof targets, not complete production verification.
- Development crypto adapters: mock receipts, mock threshold decryption, small local field, and placeholder certificate signature bytes remain to be replaced by the production choices documented in `src/SPEC.md`.
