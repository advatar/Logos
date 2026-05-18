# SDK API reference

The moderation SDK is forum-agnostic: it operates on opaque post envelopes keyed by `content_id` and `post_id` and never assumes a content shape.

## Core types (`protocol-core`)

```rust
pub struct ForumConfig {
    pub forum_id: Hash32,
    pub k: u8,
    pub n: u8,
    pub moderators: Vec<ModeratorIdentity>,
    pub mod_set_version: u64,
    pub threshold_public_key: ThresholdPublicKey,   // Ristretto255 point
}

pub struct ModeratorIdentity {
    pub id: ModeratorId,
    pub verifying_key: [u8; 32],                    // Ed25519
    pub share_public_key: SharePublicKey,           // threshold-ElGamal share
}

pub struct AnonymousPostEnvelope {
    pub forum_id: Hash32,
    pub post_id: Hash32,
    pub content_id: Hash32,
    pub post_nonce: Vec<u8>,
    pub proof_public_inputs_hash: Hash32,
    pub ciphertext: Ciphertext,                     // threshold-ElGamal
    pub ciphertext_hash: Hash32,
    pub share_commitment: Hash32,
    pub retro_tag: Hash32,
    pub membership_root: Hash32,
    pub revocation_root: Hash32,
    pub zk_receipt: MockZkReceipt,                  // → ZkReceipt::Risc0 once Phase 4 ships
}

pub struct ModerationCertificate {
    pub statement: CertificateStatement,
    pub votes: Vec<ModerationVote>,                 // ≥ N distinct Ed25519 signatures
    pub ciphertext: Ciphertext,
    pub partial_decryptions: Vec<PartialDecryption>, // ≥ N DLEQ-proven partials
}
```

`ModerationCertificate` has no `revealed_share` field. Call `cert.revealed_share(forum)` to aggregate the partials trustlessly; the slash verifier does this internally.

## Moderator material

```rust
pub struct ModeratorSecret {
    pub id: ModeratorId,
    signing_key: SigningKey,                        // Ed25519
    pub share_secret_key: ShareSecretKey,           // threshold-ElGamal share s_i
}

impl ModeratorSecret {
    pub fn new(id, signing_key, share_secret_key) -> Self;
    pub fn from_seed_and_share(id, seed: &[u8; 32], share_secret_key) -> Self;
    pub fn identity(&self) -> ModeratorIdentity;
    pub fn partial_decrypt(&self, post: &AnonymousPostEnvelope) -> PartialDecryption;
}
```

## Forum SDK (`moderation-sdk::ForumSdk`)

```rust
fn new(forum: ForumConfig, store: S) -> Result<Self>;
fn register_member(&mut self, member: &MemberSecret) -> Result<Hash32>;
fn build_post(&mut self, member, content_id, nonce) -> Result<AnonymousPostEnvelope>;
fn persist_post(&mut self, post: &AnonymousPostEnvelope) -> Result<String>;
fn create_moderation_vote(&self, mod: &ModeratorSecret, post, reason_hash) -> Result<ModerationVote>;
fn aggregate_certificate(
    &self,
    post: &AnonymousPostEnvelope,
    reason_hash: Hash32,
    votes: Vec<ModerationVote>,
    partial_decryptions: Vec<PartialDecryption>,
) -> Result<ModerationCertificate>;
fn submit_slash(&mut self, certs: &[ModerationCertificate]) -> Result<SlashResult>;
```

## Storage trait (`moderation-sdk::OffchainStore`)

```rust
pub trait OffchainStore {
    fn put(&mut self, namespace: &str, bytes: Vec<u8>) -> Result<String>;
    fn get(&self, id: &str) -> Result<Vec<u8>>;
    fn list(&self, namespace: &str) -> Result<Vec<String>>;
}
```

In-memory implementation: `MemoryStore`. Production must implement this trait against Logos Storage + Delivery, with a retry queue around `put` and the slash submission path (tracked in `STATUS.md → Phase 6`).

Namespacing (current + planned):

```text
post/<forum_id_hex>/<id>        // implemented
cert/<forum_id_hex>/<id>        // planned
vote/<forum_id_hex>/<post>/<m>  // planned
slash/<forum_id_hex>/<id>       // planned
```

## RISC0 statement (`risc0-statement`)

```rust
pub struct PublicInputs { forum_id, k, membership_root, revocation_root,
    content_id, post_nonce, threshold_public_key_hash, ciphertext_hash,
    retro_tag, share_commitment }

pub struct PrivateInputs { coeffs, membership_path, threshold_public_key,
    encryption_nonce_seed, non_membership_witness }

pub fn verify(public: &PublicInputs, private: &PrivateInputs) -> Result<(), StatementError>;
```

`PublicInputs::commitment()` uses the same `proof-public-inputs` framing as `AnonymousPostEnvelope::build`, so the receipt's journal entry matches what the certificate statement binds.

## Slash verifier CLI

```sh
slash-verifier verify --registry snapshot.json --bundle bundle.json
slash-verifier schema   # print JSON schemas
```

The library half (`slash_verifier::verify`) is independently unit-tested and can be embedded into other tooling.
