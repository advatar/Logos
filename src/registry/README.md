# Registry integration

The registry crate is the executable Rust boundary for the intended LEZ program. It mirrors the IDL and is covered by workspace tests, but it is not yet a deployable LEZ guest.

`logos-scaffold` v0.1.1 is installed and the pinned LEZ `sequencer_service` / `wallet` binaries can be built after installing `logos-blockchain-circuits`. The current scaffold IDL generator only runs for the `lez-framework` project shape and expects generated client crates, while this repo still uses the default scaffold shape to keep normal workspace builds passing.

Remaining deploy gap: add a `methods/guest/src/bin/lp0016_registry.rs` LEZ guest or migrate this crate to the `lez-framework` template without breaking `cargo build --workspace`.

- `create_forum`
- `register_member`
- `slash_member`

The registry stores commitments and revocations. It does not store forum content.
