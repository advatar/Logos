# RISC0 membership proof

The production guest proves the post statement described in `SPEC.md`.

The host must expose:

```rust
prove_post(private_inputs, public_inputs) -> Receipt
verify_post_receipt(receipt, public_inputs) -> Result<()>
```

The demo video must show `RISC0_DEV_MODE=0` for the final submission.
