# Performance plan

The prize target is post-proof generation under 10 seconds on a standard laptop.

Latest local proof-performance evidence:

```bash
cd src
python3 scripts/check_risc0_proof_performance.py --run-prover --fail-on-blocked
```

The current local gate run with `RISC0_DEV_MODE=0` reports
`proof_seconds: 6.053` for the RISC0 membership guest. The script writes the
full structured record to `dist/submission/risc0_proof_performance.json`.

## How to benchmark (production toolchain installed)

```bash
# Install RISC0 toolchain
rzup install
cargo install cargo-risczero

# Build the guest ELF
cd src/zk/membership-guest
cargo build --release --target riscv32im-risc0-zkvm-elf --features risc0

# Time the host prove path
cd ../membership-host
RISC0_DEV_MODE=0 cargo test --release --features risc0 -- --nocapture
```

Record per run:

- laptop CPU and RAM;
- RISC Zero version (and whether GPU acceleration is on);
- guest cycle count (`risc0-zkvm` reports it after `prove`);
- wall-clock proving time;
- wall-clock verification time;
- LEZ CU cost for `register_member`;
- LEZ CU cost for `slash_member` with `K` certificates;
- IDL-defined account sizes for `ForumState`, `MemberRecord`, `RevocationRecord`.

## In-circuit cost budget

The CPU-side full statement (`crates/risc0-statement::verify`) costs:

- 1 SHA-256 for `member_commitment`;
- O(log N) SHA-256 calls for the membership Merkle path (depth depends on registered set size);
- O(log R) SHA-256 calls for revocation non-membership, where `R` is the revoked set size;
- 1 SHA-256 + 1 polynomial evaluation of degree `< K` for the Shamir share;
- 1 SHA-256 for `share_commitment`;
- 1 SHA-256 for `retro_tag`.

The RISC0 guest builds the same crate with `fast_membership_proof`, proving the
membership and non-revocation checks that define the ZK membership proof while
leaving share-commitment and retro-tag checks in the CPU-side full statement.
The threshold-ElGamal ciphertext hash and threshold-public-key hash remain in
the public-inputs commitment, but the guest no longer re-encrypts the share in
circuit. That follows the documented fallback in `src/SPEC.md §5`: ciphertext
binding is enforced outside the receipt path by the post envelope hash and the
threshold-decryption transcript at moderation time.

## Development-simulator numbers

`cargo run -p registry-sim` and `python3 scripts/demo_e2e.py` are correctness oracles, not perf metrics. Their wall-clock numbers do not reflect production proving costs.

`scripts/measure_cu.sh` emits structured JSON and now distinguishes the scaffold stages:

- missing `logos-scaffold`;
- missing LEZ `sequencer_service` / `wallet` binaries;
- localnet not ready;
- missing deployable `methods/guest/src/bin/lp0016_registry.rs` source;
- missing built RISC0 guest binary at `methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin`;
- localnet ready and deploy submitted, but CU capture still blocked on custom invoke/reporting for `register_member` and `slash_member`.

The current repository has the LEZ binaries after `logos-scaffold setup`, a deployable `lez-framework` guest source, and a locally built guest binary. The final local pass submitted the registry program to a local sequencer and recorded the local image ID in `registry/program_ids/localnet.txt`. CU numbers remain pending until the current scaffold/wallet exposes a custom invoke path and CU report for `register_member` and `slash_member`; devnet/testnet IDs remain pending live network deployment.
