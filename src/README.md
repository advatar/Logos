# LP-0016 Anonymous Forum — starter implementation

This repository is a concrete starting point for LP-0016: anonymous forums with threshold moderation and membership revocation.

It contains:

- a Rust workspace for the protocol core, moderation SDK surface, registry simulator, and slash verifier CLI;
- a runnable Python end-to-end simulator that exercises registration, anonymous posting, N-of-M moderation, K-strike slash, revocation, and retroactive linkability;
- Lean 4 proof modules for the core state-machine invariants and Shamir/slash theorem surfaces;
- LEZ/SPEL/RISC0/Basecamp integration directories with feature-gated or deterministic local harnesses where the external runtime is not available.

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

- `ZkReceipt::Mock` is used by local tests until the external RISC0 proving toolchain is installed.
- threshold encryption is represented as a test/development oracle only in the dependency-free Python simulator; Rust uses threshold ElGamal with DLEQ partials.
- SPEL/LEZ deployment still requires a deployable `methods/guest/src/bin/lp0016_registry.rs` guest or a migration to the `lez-framework` scaffold shape.
- Basecamp has an embeddable `ui_qml` package harness plus Rust core-module bridge; the official Basecamp runtime boots locally, but full automated click-through still needs the Basecamp/QML inspector test harness.

The production path is documented in `SPEC.md` and `docs/protocol.md`.

## Repository layout

```text
crates/protocol-core       Pure protocol state machine: field, Shamir, certs, slash
crates/moderation-sdk      Forum-agnostic SDK facade and storage abstraction
crates/registry-sim        Local registry simulation binary
crates/slash-verifier      CLI shell for slash-bundle verification
scripts/demo_e2e.py        Runnable local end-to-end demo
scripts/test_protocol.py   Unit tests for the local simulator
lean/                      Lean 4 invariant and theorem-surface modules
registry/lp0016-registry   LEZ/SPEL registry crate and hand-written IDL
zk/                        RISC0 guest/host boundary stubs
app/basecamp-forum         Basecamp QML flow and core-module bridge
```

## Immediate next steps for the developer

1. Add the deployable LEZ guest wrapper for `lp0016_registry` or migrate the registry crate to the `lez-framework` scaffold shape.
2. Capture `register_member` / `slash_member` CU numbers after deploy/invoke reporting is available.
3. Wire generated RISC0 receipt bytes from the app flow into `ZkReceipt::Risc0`.
4. Wire `moderation-sdk` storage and messaging traits to Logos Storage/Delivery.
5. Run the packaged Basecamp LGX through the QML inspector harness.

## License

MIT or Apache-2.0 at your option.
