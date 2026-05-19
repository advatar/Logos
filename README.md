# Logos

LP-0016 anonymous forum starter implementation.

The current starter app lives in `src/`. It includes the Rust protocol workspace, Python simulator, Lean proof modules, LEZ/SPEL registry crate, feature-gated RISC0 host/guest crates, and a Basecamp flow harness. See `src/README.md` and `REPO.md` for the detailed repository notes.

Verified local commands:

```bash
cd src
scripts/local_submission_gate.py
python3 scripts/demo_e2e.py
python3 -m unittest scripts/test_protocol.py
cargo build --workspace
cargo test --workspace
cargo run -p registry-sim
cd lean && lake build
```

GitHub Actions is not the acceptance gate for this repository because hosted
jobs are blocked before startup by account billing/spending limits. Use the
local submission gate and `src/docs/submission.md` for hackathon evidence.
