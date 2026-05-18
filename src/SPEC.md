# LP-0016 concrete implementation spec

This document fills the gaps called out by the developer review.

## 1. Toolchain and versions

Production target:

- Rust edition: 2021
- Rust toolchain: `1.82.0`
- Cargo resolver: `2`
- RISC Zero: `cargo-risczero 3.0.5`, `rzup 0.5.1`, `r0vm 3.0.5`, and RISC0 Rust `1.94.1` have been installed locally; final pinning must still match the LEZ proving stack.
- LEZ: `logos-scaffold 0.1.1` with `scaffold.toml` pinned to LEZ commit `35d8df0d031315219f94d1546ceb862b0e5b208f`.
- SPEL/IDL: `logos-scaffold build idl` is available, but the installed generator only emits IDL for the `lez-framework` project shape. This repo still keeps the registry as a plain Rust crate plus hand-written IDL until a deployable LEZ guest or full `lez-framework` migration lands.
- Basecamp: official `logos-basecamp` v0.1.1 macOS runtime boots locally; `app/basecamp-forum` packages as a `ui_qml` LGX via `scripts/package_basecamp.sh`.

Local simulator target:

- Python 3.10+
- no third-party Python dependencies

## 2. Concrete crypto choices

### Production choices

- Shamir field: Ristretto255 scalar field (`curve25519-dalek::Scalar`).
- Moderator signatures: Ed25519 (`ed25519-dalek`), with canonical transcript signing.
- Hash/transcript: SHA-256 with explicit domain separation and length-delimited fields. Use BLAKE3 only for non-consensus cache keys.
- Membership commitment:

```text
member_commitment = H("lp0016/member/v1", forum_id, K, polynomial_coefficients)
```

- Post share:

```text
x = H_to_scalar("lp0016/share-x/v1", forum_id, content_id, post_nonce)
y = P(x)
```

- Retroactive link tag:

```text
retro_tag = H("lp0016/retro/v1", forum_id, polynomial_coefficients, content_id, post_nonce)
```

- Threshold encryption: threshold ElGamal over Ristretto255.
  - Forum has threshold public key `Y = sG`.
  - Each moderator holds a Shamir key share `s_i`.
  - Post encrypts the Shamir strike share `(x, y)` with hybrid ElGamal:

```text
C1 = rG
K  = H("lp0016/kem/v1", rY)
C2 = encode(x, y) XOR KDF(K, len(encode(x, y)))
```

  - Moderator partial decryptions are `D_i = s_i C1`.
  - Aggregation interpolates partial decryptions to recover `D = sC1`, derives the same KDF key, and decrypts `(x, y)`.
  - Each moderator decryption share carries a DLEQ proof that `D_i` uses the same secret exponent as the moderator's threshold public share.

- ZK proof system: RISC Zero zkVM receipt for the membership/post statement.

### Implementation status

- Rust `protocol-core`: **production** Ristretto255 scalar field, **production** Ed25519 moderator signatures, and **production** threshold ElGamal over Ristretto255 with Chaum–Pedersen DLEQ partial-decryption proofs (`ThresholdPublicKey`, `ShareSecretKey`/`SharePublicKey`, `Ciphertext`, `PartialDecryption`, `DleqProof`). `ForumConfig` carries the actual threshold public key (its hash is derived for transcripts). `AnonymousPostEnvelope` carries the real `Ciphertext`. `ModerationCertificate` carries `Vec<PartialDecryption>`; `verify_certificate` validates every DLEQ against the moderator's `share_public_key` and the slash verifier aggregates trustlessly. Canonical transcript framing matches §3.
- Rust `protocol-core`: a sorted binary Merkle tree (`protocol-core::merkle`) over `Hash32` with leaf/node domain separation, used by `RegistryState::membership_root()` and `revocation_root()`. Both roots are bound into the post public-inputs hash alongside `threshold_public_key_hash` per §5. Revocation non-membership is encoded as predecessor/successor Merkle membership proofs with sorted-index adjacency checks, and `risc0-statement` enforces it.
- Rust `lp0016-registry`: the LEZ registry boundary with `create_forum` / `register_member` / `slash_member` instructions and `ForumState` / `MemberRecord` / `RevocationRecord` / `StakePolicy` account types. SPEL annotations remain doc-style today (`// #[lez_program]` etc.) because `logos-scaffold 0.1.1` generates IDL only for the `lez-framework` project shape. The hand-written `src/registry/idl/lp0016_registry.json` is the source-of-truth until a deployable `methods/guest/src/bin/lp0016_registry.rs` guest or full `lez-framework` migration lands.
- Rust `risc0-statement`: the pure check function the RISC0 guest runs, with CPU-side tests verifying it accepts real inputs and rejects tampered membership root / wrong coefficients / swapped threshold key. `PublicInputs::commitment()` uses the same `proof-public-inputs` framing as the post envelope so receipts are pin-compatible.
- Rust `lp0016-membership-guest` and `lp0016-membership-host`: real `risc0-zkvm` integration, gated behind a `risc0` feature so the default workspace build doesn't require `cargo-risczero`. The guest reads `PublicInputs` + `PrivateInputs` from `env`, runs `risc0_statement::verify`, and commits the public-inputs hash; the host builds an `ExecutorEnv`, asks `default_prover` for a receipt, and verifies it against the expected image id.
- Rust `protocol-core`: `ZkReceipt` now supports both `Mock` and `Risc0 { receipt_bytes, image_id, journal }` variants. Local tests still use `ZkReceipt::Mock`; full `Risc0` receipt verification is exercised through the feature-gated host crate when the RISC0 toolchain is installed. Threshold setup uses a Pedersen-style DKG transcript API (`DealerShares::pedersen_dkg`) instead of a single trusted dealer polynomial.
- Python `lp0016_sim.py`: stays on the dev field (`2^61 - 1`), the dev SHA-256-derived moderator signature, and the `ThresholdOracle` HashMap. SPEC.md keeps it dependency-free; the production crypto lives in Rust. Python mirrors every protocol-level transcript binding the Rust core enforces (forum_id, mod_set_version, threshold_public_key_hash, K, N), so changes to the protocol-level transcript must update both.

## 3. Serialization and domain separation

Consensus and certificate bytes must not use `serde_json`, `bincode`, or map iteration order. Use an explicit canonical transcript:

```text
lp0016:<domain>:v1 || field_count || repeated(len(field) || field_bytes)
```

Rules:

- all integers are big-endian fixed width;
- all byte arrays are length-prefixed with `u32`;
- forum IDs, content IDs, post IDs, and hashes are fixed 32-byte arrays;
- certificate statements include every replay-sensitive value:
  - `forum_id`
  - `content_id`
  - `post_id`
  - `post_proof_public_inputs_hash`
  - `ciphertext_hash`
  - `reason_hash`
  - `mod_set_version`
  - `K`
  - `N`
  - threshold encryption key hash.

## 4. LEZ/SPEL/Basecamp targets

LEZ registry responsibilities:

```text
create_forum(forum_id, K, N, moderator_keys, threshold_public_key, stake_policy)
register_member(forum_id, member_commitment, stake_account)
slash_member(forum_id, slash_bundle)
```

State accounts:

```text
ForumState
MemberRecord
RevocationRecord
ModeratorSetVersion
```

SPEL IDL must be generated from annotations around these instructions. The final repo should include:

```text
registry/idl/lp0016_registry.json
registry/program_ids/devnet.txt
registry/program_ids/testnet.txt
```

Basecamp app scope:

- create forum;
- register member;
- post text content;
- moderator dashboard for pending posts;
- cast moderation vote;
- aggregate certificate;
- view moderation history;
- submit slash;
- show rejected post after revocation.

No feeds, search, reputation, reactions, or social graph.

## 5. RISC0 circuit statement

Guest public inputs:

```text
forum_id
K
membership_root
revocation_root
content_id
post_nonce
threshold_public_key_hash
ciphertext_hash
retro_tag
share_commitment
```

Guest private inputs:

```text
polynomial_coefficients
membership_merkle_path
revocation_nonmembership_path
encryption_randomness
```

Guest checks:

```text
member_commitment = H(member domain, forum_id, K, polynomial_coefficients)
member_commitment is in membership_root
member_commitment is not in revocation_root
x = H_to_scalar(share-x domain, forum_id, content_id, post_nonce)
y = eval(polynomial_coefficients, x)
share_commitment = H(share domain, forum_id, content_id, post_nonce, x, y)
ciphertext_hash matches encryption of encode(x, y)
retro_tag = H(retro domain, forum_id, polynomial_coefficients, content_id, post_nonce)
```

Performance plan:

1. implement the guest using only SHA-256, scalar arithmetic, and Merkle path checks;
2. benchmark on a laptop with `RISC0_DEV_MODE=0` before implementing the UI;
3. keep encryption verification outside the receipt if proving cost exceeds 10 seconds, but then include a binding ciphertext/share commitment verified by the threshold-decryption transcript;
4. document cycle counts and wall-clock times in `docs/performance.md`.

## 6. Storage and networking assumptions

The SDK never assumes a forum content shape. It stores opaque envelopes:

```text
post/<forum_id>/<post_id>              -> AnonymousPostEnvelope
cert/<forum_id>/<certificate_id>       -> ModerationCertificate
vote/<forum_id>/<post_id>/<moderator>  -> ModerationVote
slash/<forum_id>/<slash_id>            -> SlashBundle
```

`moderation-sdk` implements these namespaces with `storage_namespace(StorageKind, forum_id)` and wraps failed `put` operations plus slash submission records in a `RetryQueue`.

The SDK exposes a trait:

```rust
pub trait OffchainStore {
    fn put(&mut self, namespace: &str, bytes: Vec<u8>) -> Result<String>;
    fn get(&self, id: &str) -> Result<Vec<u8>>;
    fn list(&self, namespace: &str) -> Result<Vec<String>>;
}
```

A retry queue is mandatory for storage/messaging failures. The local implementation uses memory storage; production uses Logos Storage and Delivery.

## 7. Acceptance criteria for done

A final submission is done only when all of these are true:

- `cargo test --workspace` passes.
- `lake build` passes with no `sorry` in the claimed proof files.
- `RISC0_DEV_MODE=0 scripts/demo_e2e.sh` works against a local LEZ sequencer.
- `logos-scaffold build idl` emits a registry IDL.
- two testnet forum instances exist with different `(K, N, M)` parameters.
- CU cost for `register_member` and `slash_member` is measured and documented.
- Basecamp app executes the full flow without CLI interaction.
- CI runs Rust tests, Lean build, RISC0 guest build, local sequencer integration test, and app smoke test.
- README includes deployed program IDs and a narrated demo video link.

## 8. Threat model summary

Assumptions:

- SHA-256 is collision-resistant and preimage-resistant.
- Ed25519 signatures are unforgeable.
- Ristretto255 discrete log is hard.
- Threshold ElGamal and DLEQ share proofs are sound.
- RISC Zero receipts are sound and zero-knowledge for the statement used.
- LEZ validates the registry program faithfully.

Threats addressed:

- fewer than N moderators cannot create a valid certificate;
- fewer than K certificates do not reconstruct the member polynomial;
- slash affects only the registered commitment reconstructed from K shares;
- cross-forum replay is rejected by transcript domain separation;
- post proofs after revocation fail because the revocation root is checked in the receipt.

Threats not addressed:

- moderator collusion above threshold;
- social deanonymization from content style;
- malicious UI hiding moderation history;
- side channels in local proof generation;
- unavailable or censored Logos Storage/Delivery peers beyond retry/queue handling.
