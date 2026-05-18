# RISC0 membership proof

The production guest proves the post statement described in `SPEC.md`.

The host must expose:

```rust
prove_post(private_inputs, public_inputs) -> Receipt
verify_post_receipt(receipt, public_inputs) -> Result<()>
receipt_to_protocol(receipt, image_id, public_inputs_hash) -> ZkReceipt::Risc0
```

`lp0016-membership-host` exposes `receipt_to_protocol` behind `--features risc0`.
The returned protocol receipt carries the journal hash, image ID bytes, and
serialized RISC0 receipt bytes so the app/SDK can replace the local mock
receipt without changing the post envelope format.

The demo video must show `RISC0_DEV_MODE=0` for the final submission.
