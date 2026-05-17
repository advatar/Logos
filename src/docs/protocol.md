# Protocol specification

## Overview

A forum instance has independent parameters:

```text
forum_id
K                    revocation threshold
N                    moderation threshold
M                    number of designated moderators
moderator_keys       Ed25519 verification keys
threshold_key        threshold ElGamal public key
membership_root      registered commitments
revocation_root      revoked commitments
```

A member samples a hidden polynomial of degree `< K`.

```text
P(X) = a0 + a1 X + ... + a(K-1) X^(K-1)
member_commitment = H("lp0016/member/v1", forum_id, K, coeffs(P))
```

The commitment is registered and staked. The coefficients remain private.

## Anonymous posting

For each content item:

```text
x = H_to_scalar("lp0016/share-x/v1", forum_id, content_id, post_nonce)
y = P(x)
```

The post includes:

```text
ciphertext      threshold encryption of encode(x, y)
share_commitment = H("lp0016/share/v1", forum_id, content_id, post_nonce, x, y)
retro_tag       = H("lp0016/retro/v1", forum_id, coeffs(P), content_id, post_nonce)
zk_receipt      proves registered, non-revoked membership and consistency
```

The public post envelope does not reveal the member commitment or the share `(x, y)`.

## Moderation certificates

A moderation vote signs the canonical certificate statement:

```text
forum_id
content_id
post_id
post_proof_public_inputs_hash
ciphertext_hash
reason_hash
mod_set_version
K
N
threshold_key_hash
```

A moderation certificate is valid if:

1. it contains at least N distinct moderator votes;
2. every signer belongs to the moderator set for `mod_set_version`;
3. every vote signs the same statement;
4. the threshold decryption transcript proves the decrypted share matches the post ciphertext.

The certificate reveals exactly one Shamir point `(x, y)` for the offending post.

## Slash

A slash bundle contains K valid moderation certificates. The registry verifier:

1. verifies every certificate;
2. checks the K revealed shares have distinct x-coordinates;
3. interpolates the unique degree `< K` polynomial through those shares;
4. recomputes `member_commitment`;
5. checks the commitment is registered and not revoked;
6. marks the commitment revoked and releases/slashes the stake according to the forum policy.

## Unlinkability argument

Before slash, observers see only public post envelopes. They do not see the member commitment, the Shamir share, or the member polynomial. Each post uses a fresh `content_id`/`post_nonce` pair, so its encrypted share and retro tag are unlinkable under the hash, threshold encryption, and ZK assumptions.

A single moderation certificate reveals one Shamir point. Fewer than K distinct points are insufficient to determine a degree `< K` polynomial: for any candidate constant term, there exists a polynomial of degree `< K` that matches the observed points. Therefore, fewer than K certificates do not identify a registered commitment in the ideal Shamir model.

Upon slash, K certificates reconstruct the member polynomial. Anyone can then recompute the slashed member's `retro_tag` values for historical posts and link that member's prior posts. No other member's polynomial is reconstructed, so no other member's anonymity is affected.

## Moderator trust model

Moderators decide whether content violates forum rules. The cryptographic protocol does not judge content. It enforces that:

- no single moderator can act unless `N = 1` for that forum;
- every certificate has a public, auditable set of moderator votes;
- certificate evidence is replay-protected by forum ID and moderator-set version;
- revocation requires K valid certificates for the same registered commitment.

## Replay protection

Every signed/proven object is domain-separated by forum-specific values. A post proof, vote, certificate, or slash bundle for one forum must not verify in another forum.

## Revocation behavior

After slash, the commitment is added to the revocation list. A later post proof from the same hidden polynomial must fail because the RISC0 guest checks non-membership in the revocation root.

## Development model in this repository

The local simulator keeps the same state-machine shape, but replaces the cryptographic receipt and threshold decryption with deterministic mocks so the lifecycle can be tested without the full Logos stack. Production code must replace those mocks before submission.
