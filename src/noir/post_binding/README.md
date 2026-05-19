# LP-0016 Noir Post Binding Circuit

This optional Noir circuit is the "icing" proof artifact. It is not the
production proof path; RISC0 remains the submitted zkVM path. The circuit gives
reviewers a compact ACIR/Nargo view of the anonymous-post binding shape:

- the prover knows a private `member_secret` and `opening`;
- the public `registered_commitment` opens to that secret;
- the public `post_nullifier` is bound to the forum domain and nonce;
- the public `post_retro_tag` is bound to the slash domain and nonce.

The arithmetic is intentionally small and dependency-free so `nargo test` can
run quickly. Production hashing, membership roots, revocation roots, and
threshold decryption stay in the Rust/RISC0 implementation.

Run:

```bash
cd src/noir/post_binding
nargo test
```

Or use the repository diagnostic:

```bash
cd src
python3 scripts/check_noir_icing.py --pretty
```
