# Performance plan

The prize target is post-proof generation under 10 seconds on a standard laptop.

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

The current statement (`crates/risc0-statement::verify`) costs:

- 1 SHA-256 for `member_commitment`;
- O(log N) SHA-256 calls for the membership Merkle path (depth depends on registered set size);
- O(log R) SHA-256 calls for revocation non-membership, where `R` is the revoked set size;
- 1 SHA-256 + 1 polynomial evaluation of degree `< K` for the Shamir share;
- 1 SHA-256 for `share_commitment`;
- 1 SHA-256 for `retro_tag`;
- 1 SHA-256 + 1 Ristretto255 scalar multiplication + 1 Ristretto255 point multiplication + 1 SHA-256 KDF + 64-byte XOR for the threshold-ElGamal re-encryption check;
- 1 SHA-256 for the threshold-public-key hash check.

The dominant cost is expected to be the Ristretto255 point multiplications inside the ciphertext binding. If proving exceeds the 10-second budget, the documented fallback (see `src/SPEC.md §5`) is to move the ciphertext binding outside the receipt and bind it via the threshold-decryption transcript at moderation time instead.

## Development-simulator numbers

`cargo run -p registry-sim` and `python3 scripts/demo_e2e.py` are correctness oracles, not perf metrics. Their wall-clock numbers do not reflect production proving costs.

`scripts/measure_cu.sh` emits structured JSON and now distinguishes the scaffold stages:

- missing `logos-scaffold`;
- missing LEZ `sequencer_service` / `wallet` binaries;
- localnet not ready;
- no deployable `methods/guest/src/bin/lp0016_registry.rs` guest yet;
- localnet ready but CU capture still blocked on the deploy/invoke reporting path.

The current repository has the LEZ binaries after `logos-scaffold setup`, but it still reports `{"status":"blocked","measurement":"lez_compute_units",...}` until the registry has a deployable LEZ guest and program IDs for devnet/testnet are recorded.
