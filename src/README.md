# LP-0016 Anonymous Forum — starter implementation

This repository is a concrete starting point for LP-0016: anonymous forums with threshold moderation and membership revocation.

It contains:

- a Rust workspace for the protocol core, moderation SDK surface, registry simulator, and slash verifier CLI;
- a runnable Python end-to-end simulator that exercises registration, anonymous posting, N-of-M moderation, K-strike slash, revocation, and retroactive linkability;
- Lean 4 proof scaffolding for the core state-machine invariants;
- placeholder LEZ/SPEL/RISC0/Basecamp integration directories with the API boundaries the production work should fill.

The Python simulator is intentionally dependency-free and works in a clean environment with Python 3.10+:

```bash
python3 scripts/demo_e2e.py
python3 -m unittest scripts/test_protocol.py
```

The Rust code is designed as the implementation target once Rust, LEZ, SPEL, and RISC Zero toolchains are installed:

```bash
cargo test --workspace
cargo run -p registry-sim
```

## Security status

This is not a final prize submission. The repo fixes the previously underspecified engineering decisions and implements the core protocol state machine, but the following components are still development adapters:

- `MockZkReceipt` is a local model of the RISC0 receipt.
- threshold encryption is represented as a test/development oracle in the simulator.
- LEZ/SPEL/Basecamp integrations are stubs with documented interfaces.
- Lean files currently prove basic threshold/state-machine lemmas and list the stronger Shamir proof targets.

The production path is documented in `SPEC.md` and `docs/protocol.md`.

## Repository layout

```text
crates/protocol-core       Pure protocol state machine: field, Shamir, certs, slash
crates/moderation-sdk      Forum-agnostic SDK facade and storage abstraction
crates/registry-sim        Local registry simulation binary
crates/slash-verifier      CLI shell for slash-bundle verification
scripts/demo_e2e.py        Runnable local end-to-end demo
scripts/test_protocol.py   Unit tests for the local simulator
lean/                      Lean 4 invariant scaffold
registry/lez-program-stub  LEZ/SPEL registry boundary stub
zk/                        RISC0 guest/host boundary stubs
app/basecamp-forum         Minimal Basecamp UI placeholder
```

## Immediate next steps for the developer

1. Install the Logos toolchain and generate a fresh scaffold project with `logos-scaffold`.
2. Replace the registry stub with a SPEL-annotated LEZ program.
3. Replace `MockZkReceipt` with the RISC0 guest/host proof pair.
4. Replace dev threshold encryption with Ristretto255 threshold ElGamal and DLEQ share proofs.
5. Wire `moderation-sdk` storage and messaging traits to Logos Storage/Delivery.
6. Use the Basecamp app only through the SDK surface; do not add forum-specific assumptions to the library.

## License

MIT or Apache-2.0 at your option.
