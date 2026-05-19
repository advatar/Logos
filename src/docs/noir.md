# Optional Noir Proof Circuit

Noir is included as an extra proof artifact for reviewers who want a compact
ACIR/Nargo view of the anonymous-post relation. It is intentionally additive:
RISC0 remains the submitted zkVM proof path, and Lean 4 remains the formal
invariant proof surface.

## What It Proves

The circuit in `noir/post_binding/src/main.nr` models the post-binding shape:

- the prover knows private values `member_secret` and `opening`;
- the public `registered_commitment` opens to that private secret;
- the public `post_nullifier` is deterministically tied to the forum domain and
  post nonce;
- the public `post_retro_tag` is deterministically tied to the slash domain and
  post nonce.

That is the core relation needed by the app: a post can be checked against a
registered member commitment without revealing the member secret, while public
tags remain deterministic enough for verification and later slash/linking logic.

## What It Does Not Replace

The Noir circuit is deliberately small. It does not implement production
hashing, Merkle membership, revocation non-membership, threshold ElGamal, DLEQ
proofs, or LEZ settlement. Those pieces live in the Rust/RISC0 path:

- Rust implements protocol state, crypto, registry logic, and SDK boundaries.
- RISC0 proves the membership/non-revocation statement used for submission
  evidence.
- Lean 4 checks the protocol-level threshold, slash, revocation, and Shamir
  theorem surfaces.

Noir is the icing: a concise proof-circuit companion that makes the private
binding relation easy to inspect and run.

## Files

- `noir/post_binding/Nargo.toml`: Noir package manifest.
- `noir/post_binding/src/main.nr`: circuit and tests.
- `noir/post_binding/Prover.toml`: sample witness/public-input assignment.
- `scripts/check_noir_icing.py`: structured diagnostic used by the local gate.

## Commands

Install Noir/Nargo with the official `noirup` path when needed:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup
```

Run the circuit tests:

```bash
cd src/noir/post_binding
nargo test
```

Run the repo diagnostic:

```bash
cd src
python3 scripts/check_noir_icing.py --pretty
```

The diagnostic also checks the standard `~/.nargo/bin/nargo` install location,
so it works in non-interactive shells where `.zshrc` has not refreshed `PATH`.

## Current Local Result

On this machine, `nargo 1.0.0-beta.21` compiles and runs the package:

```text
[lp0016_post_binding] Running 2 test functions
[lp0016_post_binding] Testing accepts_consistent_post_binding ... ok
[lp0016_post_binding] Testing rejects_bad_nullifier ... ok
[lp0016_post_binding] 2 tests passed
```

The local submission gate includes `noir_icing_diagnostic` as an optional
evidence step, and it is currently ready.
