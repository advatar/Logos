# Local Submission Evidence

GitHub Actions is intentionally not part of the acceptance path for this
submission because hosted jobs are blocked before startup by account
billing/spending limits. The reproducible gate is local:

```bash
cd src
scripts/local_submission_gate.py
```

The gate writes `dist/submission/evidence.json` plus per-command logs under
`dist/submission/logs/`. The required local steps include the Python lifecycle
demo, success-criteria tests, Rust workspace build/tests, RISC0 host feature
check, LEZ guest check/build, Lean build, and Basecamp package build.

Runtime diagnostics are included as non-required evidence steps:

```bash
python3 scripts/check_lez_runtime.py --pretty
python3 scripts/check_basecamp_inspector.py --pretty
scripts/measure_cu.sh
```

The final local pass built the LEZ guest binary at
`methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin` and
submitted it to a local sequencer. The local program image ID is recorded in
`registry/program_ids/localnet.txt`.

Remaining submission artifacts that require live external or human action:

- `registry/program_ids/devnet.txt`
- `registry/program_ids/testnet.txt`
- CU numbers for `register_member` and `slash_member` once custom invoke/CU
  reporting is exposed by scaffold/wallet or the generated client.
- A narrated video demo link in the README.
