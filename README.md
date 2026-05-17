# Logos

LP-0016 anonymous forum starter implementation.

The current starter app lives in `src/`. It includes the Rust protocol workspace, Python simulator, Lean proof scaffold, LEZ/SPEL and RISC0 stubs, and a minimal Basecamp placeholder app. See `src/README.md` and `REPO.md` for the detailed repository notes.

Verified local commands:

```bash
cd src
python3 scripts/demo_e2e.py
python3 -m unittest scripts/test_protocol.py
cargo build --workspace
cargo test --workspace
cargo run -p registry-sim
cd lean && lake build
```
