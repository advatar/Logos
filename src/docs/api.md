# SDK API reference

The moderation SDK is forum-agnostic. It works with abstract content IDs and opaque post envelopes.

## Main types

```rust
ForumParams {
    forum_id: ForumId,
    strike_threshold_k: u8,
    moderation_threshold_n: u8,
    moderators: Vec<ModeratorKey>,
    threshold_public_key: ThresholdPublicKey,
}

AnonymousPostEnvelope {
    forum_id: ForumId,
    post_id: PostId,
    content_id: ContentId,
    proof_public_inputs_hash: Hash32,
    ciphertext_hash: Hash32,
    retro_tag: Hash32,
    zk_receipt: ReceiptBytes,
}

ModerationCertificate {
    statement: CertificateStatement,
    votes: Vec<ModerationVote>,
    revealed_share: ShamirShare,
    decryption_transcript: DecryptionTranscript,
}

SlashBundle {
    forum_id: ForumId,
    certificates: Vec<ModerationCertificate>,
}
```

## Core calls

```rust
create_forum(params) -> ForumHandle
register_member(forum, member_secret, stake) -> RegistrationReceipt
build_post_proof(forum, member_secret, content_id) -> AnonymousPostEnvelope
verify_post(forum, envelope, registry_snapshot) -> Result<()>
create_moderation_vote(forum, moderator_key, post, reason_hash) -> ModerationVote
aggregate_certificate(forum, post, votes, decryption_shares) -> ModerationCertificate
verify_certificate(forum, cert) -> Result<()>
find_slash_bundle(certs, K) -> Option<SlashBundle>
submit_slash(forum, bundle) -> SlashReceipt
```

## Storage interface

```rust
pub trait OffchainStore {
    fn put(&mut self, namespace: &str, bytes: Vec<u8>) -> Result<String>;
    fn get(&self, id: &str) -> Result<Vec<u8>>;
    fn list(&self, namespace: &str) -> Result<Vec<String>>;
}
```
