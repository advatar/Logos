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
check, real RISC0 proof-performance measurement, LEZ guest check/build, Lean
build, and Basecamp package build.

Runtime diagnostics are included as non-required evidence steps:

```bash
python3 scripts/collect_localnet_evidence.py
python3 scripts/check_risc0_proof_performance.py --run-prover --fail-on-blocked
python3 scripts/check_lez_runtime.py --pretty
python3 scripts/check_basecamp_inspector.py --pretty
python3 scripts/check_live_network_deploy.py
scripts/measure_cu.sh
```

The final local pass built the LEZ guest binary at
`methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin` and
submitted it to a local sequencer. `scripts/collect_localnet_evidence.py`
starts the local sequencer directly when scaffold's localnet state is stale,
deploys the registry guest, runs `RISC0_DEV_MODE=0 scripts/demo_e2e.sh`, and
writes `dist/submission/localnet_evidence.json`.

This is not an invented fallback network. The current official LEZ wallet
quickstart tells developers to run the sequencer in standalone mode locally and
connect wallet traffic to `localhost:3040`:

https://github.com/logos-co/logos-docs/blob/main/docs/apps/wallet/journeys/quickstart-for-the-logos-execution-zone-wallet.md

That public documentation does not publish separate devnet/testnet sequencer RPC
URLs. If reviewers accept the official standalone sequencer as the current LEZ
developer target, `localnet_evidence.json` and `registry/program_ids/localnet.txt`
are the relevant deployment evidence. If reviewers require separate public
devnet/testnet networks, the missing artifacts remain the RPC URLs and
`registry/program_ids/devnet.txt` / `registry/program_ids/testnet.txt`.

The RISC0 proof-performance diagnostic builds the membership guest and runs the
real host prover with `RISC0_DEV_MODE=0`. The latest local gate run reported
`proof_seconds: 6.053`, under the 10-second target, and wrote
`dist/submission/risc0_proof_performance.json`.

The local program image ID is recorded in `registry/program_ids/localnet.txt`:

```text
dd914ffd8202da7c363d0aa7d9ad6222d1638b79f63a13f5dd24109896817e30
```

Remaining submission artifacts that require live external or human action:

- Public devnet/testnet RPC URLs and `registry/program_ids/devnet.txt` /
  `registry/program_ids/testnet.txt`, only if the reviewer requires separate
  public networks instead of the official standalone local sequencer path.
- CU numbers for `register_member` and `slash_member` once custom invoke/CU
  reporting is exposed by scaffold/wallet or the generated client.
- A narrated video demo link in the README.
