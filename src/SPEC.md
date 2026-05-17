# LP-0016 concrete implementation spec

This document fills the gaps called out by the developer review.

## 1. Toolchain and versions

Production target:

- Rust edition: 2021
- Rust toolchain: `1.82.0`
- Cargo resolver: `2`
- RISC Zero: install with `rzup install`; pin the final repo to the RISC Zero version used by the current LEZ tag. The placeholder manifests document `risc0-zkvm = 3.0.5` because the public docs currently expose 3.x APIs, but final pinning must match LEZ.
- LEZ: pin by git tag in `scaffold.toml` after `logos-scaffold setup`.
- SPEL: generated via `logos-scaffold build idl` or `logos-scaffold spel -- <command>`.
- Basecamp: run via `logos-scaffold basecamp launch <profile>` after module registration.

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

### Development choices in this starter repo

The Rust/Python local model uses a small prime field `2^61 - 1`, SHA-256 transcripts, mock receipts, and mock threshold decryption. This lets tests exercise the protocol state machine quickly. Replace with the production choices above before security review.

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
