# Logos

LP-0016 anonymous forum starter implementation.

The current starter app lives in `src/`. It includes the Rust protocol workspace, Python simulator, Lean proof modules, LEZ/SPEL registry crate, feature-gated RISC0 host/guest crates, and a Basecamp flow harness. See `src/README.md` and `REPO.md` for the detailed repository notes.

Verified local commands:

```bash
cd src
scripts/local_submission_gate.py
python3 scripts/collect_localnet_evidence.py
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

Submission evidence:

- Local gate: `cd src && scripts/local_submission_gate.py`
- Local sequencer deploy/RISC0 evidence: `cd src && python3 scripts/collect_localnet_evidence.py`
- RISC0 proof performance: `cd src && python3 scripts/check_risc0_proof_performance.py --run-prover --fail-on-blocked`
- Evidence JSON: `src/dist/submission/evidence.json`
- Localnet evidence JSON: `src/dist/submission/localnet_evidence.json`
- RISC0 proof performance JSON: `src/dist/submission/risc0_proof_performance.json`
- Localnet registry image ID: `dd914ffd8202da7c363d0aa7d9ad6222d1638b79f63a13f5dd24109896817e30`
- Program ID files: `src/registry/program_ids/`

The local lifecycle covers forum creation, anonymous registration, anonymous
posting, N-of-M moderation, K-certificate slash, revocation, and retroactive
linking only for the slashed member. Basecamp packaging is included in the
local gate; full Basecamp click-through needs the external Basecamp runtime,
`logos-qt-mcp`, and Logos design-system QML artifacts.
