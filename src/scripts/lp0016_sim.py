"""Dependency-free LP-0016 local simulator.

This file is deliberately small and readable. It models the protocol state
machine and lifecycle, not production cryptography.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from hashlib import sha256
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

FIELD_PRIME = (1 << 61) - 1
Hash32 = bytes
FieldElem = int
ShamirShare = Tuple[FieldElem, FieldElem]


class ProtocolError(Exception):
    pass


class ProofGenerationError(ProtocolError):
    pass


def _u64(n: int) -> bytes:
    return int(n).to_bytes(8, "big", signed=False)


def _frame(domain: str, parts: Iterable[bytes]) -> bytes:
    out = b"lp0016:" + domain.encode("utf-8") + b":v1"
    parts = list(parts)
    out += len(parts).to_bytes(4, "big")
    for part in parts:
        out += len(part).to_bytes(4, "big") + part
    return out


def digest(domain: str, *parts: bytes) -> Hash32:
    return sha256(_frame(domain, parts)).digest()


def hash_to_field(domain: str, *parts: bytes) -> FieldElem:
    n = int.from_bytes(digest(domain, *parts), "big") % FIELD_PRIME
    return n if n != 0 else 1


def field_add(a: int, b: int) -> int:
    return (a + b) % FIELD_PRIME


def field_sub(a: int, b: int) -> int:
    return (a - b) % FIELD_PRIME


def field_mul(a: int, b: int) -> int:
    return (a * b) % FIELD_PRIME


def field_inv(a: int) -> int:
    if a % FIELD_PRIME == 0:
        raise ZeroDivisionError("zero has no inverse")
    return pow(a, FIELD_PRIME - 2, FIELD_PRIME)


def field_div(a: int, b: int) -> int:
    return field_mul(a, field_inv(b))


def eval_poly(coeffs: Sequence[FieldElem], x: FieldElem) -> FieldElem:
    acc = 0
    power = 1
    for c in coeffs:
        acc = field_add(acc, field_mul(c, power))
        power = field_mul(power, x)
    return acc


def interpolate_coeffs(shares: Sequence[ShamirShare]) -> List[FieldElem]:
    """Return coefficients of the unique degree < len(shares) polynomial."""
    if len(shares) == 0:
        raise ProtocolError("need at least one share")
    xs = [x for x, _ in shares]
    if len(set(xs)) != len(xs):
        raise ProtocolError("duplicate Shamir x-coordinate")

    n = len(shares)
    result = [0] * n

    for i, (xi, yi) in enumerate(shares):
        # Build numerator polynomial product_{j != i} (X - xj)
        basis = [1]
        denom = 1
        for j, (xj, _) in enumerate(shares):
            if i == j:
                continue
            new_basis = [0] * (len(basis) + 1)
            for d, coeff in enumerate(basis):
                new_basis[d] = field_sub(new_basis[d], field_mul(coeff, xj))
                new_basis[d + 1] = field_add(new_basis[d + 1], coeff)
            basis = new_basis
            denom = field_mul(denom, field_sub(xi, xj))
        scale = field_div(yi, denom)
        for d, coeff in enumerate(basis):
            result[d] = field_add(result[d], field_mul(scale, coeff))

    return result


def coeffs_to_bytes(coeffs: Sequence[int]) -> bytes:
    return b"".join(_u64(c) for c in coeffs)


def commitment_for(forum_id: bytes, k: int, coeffs: Sequence[int]) -> Hash32:
    return digest("member", forum_id, _u64(k), coeffs_to_bytes(coeffs))


def retro_tag_for(forum_id: bytes, coeffs: Sequence[int], content_id: bytes, post_nonce: bytes) -> Hash32:
    return digest("retro", forum_id, coeffs_to_bytes(coeffs), content_id, post_nonce)


def share_for(forum_id: bytes, coeffs: Sequence[int], content_id: bytes, post_nonce: bytes) -> ShamirShare:
    x = hash_to_field("share-x", forum_id, content_id, post_nonce)
    y = eval_poly(coeffs, x)
    return x, y


def hex8(b: bytes) -> str:
    return b.hex()[:8]


@dataclass(frozen=True)
class ModeratorKey:
    moderator_id: str
    secret: bytes

    def sign(self, statement_hash: bytes) -> bytes:
        return digest("dev-moderator-signature", self.secret, statement_hash)

    def verify(self, statement_hash: bytes, signature: bytes) -> bool:
        return self.sign(statement_hash) == signature


@dataclass(frozen=True)
class ForumConfig:
    forum_id: bytes
    k: int
    n: int
    moderators: Dict[str, ModeratorKey]
    mod_set_version: int = 1
    # Hash of the threshold-decryption public key. Bound into certificate
    # statements and the post public-inputs hash so a certificate produced for
    # one threshold-key configuration cannot be replayed against another.
    threshold_public_key_hash: bytes = b"\x00" * 32

    def __post_init__(self) -> None:
        if self.k < 1:
            raise ValueError("K must be >= 1")
        if self.n < 1:
            raise ValueError("N must be >= 1")
        if self.n > len(self.moderators):
            raise ValueError("N cannot exceed M")


@dataclass(frozen=True)
class MemberSecret:
    coeffs: Tuple[FieldElem, ...]

    @classmethod
    def from_seed(cls, forum_id: bytes, k: int, seed: bytes) -> "MemberSecret":
        coeffs = tuple(hash_to_field("member-coeff", forum_id, seed, _u64(i)) for i in range(k))
        return cls(coeffs)

    def commitment(self, forum_id: bytes) -> Hash32:
        return commitment_for(forum_id, len(self.coeffs), self.coeffs)


@dataclass
class Registry:
    forum: ForumConfig
    registered: set[Hash32] = field(default_factory=set)
    revoked: set[Hash32] = field(default_factory=set)

    def register(self, commitment: Hash32) -> None:
        if commitment in self.revoked:
            raise ProtocolError("cannot register a revoked commitment")
        self.registered.add(commitment)

    def is_active(self, commitment: Hash32) -> bool:
        return commitment in self.registered and commitment not in self.revoked

    def revoke(self, commitment: Hash32) -> None:
        if commitment not in self.registered:
            raise ProtocolError("cannot revoke an unregistered commitment")
        self.revoked.add(commitment)


@dataclass(frozen=True)
class FakeZkReceipt:
    """Opaque local stand-in for a RISC0 receipt.

    The hidden commitment is present only so the local simulator can model
    verifier behavior without implementing a ZK proof system.
    """

    public_inputs_hash: Hash32
    hidden_commitment: Hash32
    valid: bool = True

    def verify(self, registry: Registry) -> bool:
        return self.valid and registry.is_active(self.hidden_commitment)


@dataclass(frozen=True)
class AnonymousPostEnvelope:
    forum_id: bytes
    post_id: bytes
    content_id: bytes
    post_nonce: bytes
    proof_public_inputs_hash: Hash32
    ciphertext_hash: Hash32
    share_commitment: Hash32
    retro_tag: Hash32
    zk_receipt: FakeZkReceipt


@dataclass(frozen=True)
class ThresholdOracle:
    """Development-only threshold-decryption oracle.

    Production code replaces this with Ristretto255 threshold ElGamal and DLEQ
    partial-decryption proofs.
    """

    shares_by_ciphertext: Dict[Hash32, ShamirShare] = field(default_factory=dict)

    def encrypt_share(self, forum: ForumConfig, post_id: bytes, share: ShamirShare) -> Hash32:
        x, y = share
        ciphertext_hash = digest("dev-threshold-ciphertext", forum.forum_id, post_id, _u64(x), _u64(y))
        self.shares_by_ciphertext[ciphertext_hash] = share
        return ciphertext_hash

    def threshold_decrypt(self, ciphertext_hash: Hash32, signer_ids: Sequence[str], threshold_n: int) -> ShamirShare:
        if len(set(signer_ids)) < threshold_n:
            raise ProtocolError("not enough distinct moderators for threshold decrypt")
        try:
            return self.shares_by_ciphertext[ciphertext_hash]
        except KeyError as exc:
            raise ProtocolError("unknown ciphertext") from exc


def build_post(
    registry: Registry,
    oracle: ThresholdOracle,
    member: MemberSecret,
    content_id: bytes,
    post_nonce: bytes,
) -> AnonymousPostEnvelope:
    forum = registry.forum
    commitment = member.commitment(forum.forum_id)
    if not registry.is_active(commitment):
        raise ProofGenerationError("member is not registered or has been revoked")

    post_id = digest("post-id", forum.forum_id, content_id, post_nonce)
    share = share_for(forum.forum_id, member.coeffs, content_id, post_nonce)
    x, y = share
    share_commitment = digest("share", forum.forum_id, content_id, post_nonce, _u64(x), _u64(y))
    ciphertext_hash = oracle.encrypt_share(forum, post_id, share)
    retro_tag = retro_tag_for(forum.forum_id, member.coeffs, content_id, post_nonce)
    public_hash = digest(
        "proof-public-inputs",
        forum.forum_id,
        _u64(forum.k),
        content_id,
        post_nonce,
        ciphertext_hash,
        share_commitment,
        retro_tag,
        forum.threshold_public_key_hash,
    )
    return AnonymousPostEnvelope(
        forum_id=forum.forum_id,
        post_id=post_id,
        content_id=content_id,
        post_nonce=post_nonce,
        proof_public_inputs_hash=public_hash,
        ciphertext_hash=ciphertext_hash,
        share_commitment=share_commitment,
        retro_tag=retro_tag,
        zk_receipt=FakeZkReceipt(public_hash, commitment),
    )


def verify_post(registry: Registry, post: AnonymousPostEnvelope) -> bool:
    if post.forum_id != registry.forum.forum_id:
        return False
    return post.zk_receipt.public_inputs_hash == post.proof_public_inputs_hash and post.zk_receipt.verify(registry)


@dataclass(frozen=True)
class CertificateStatement:
    forum_id: bytes
    post_id: bytes
    content_id: bytes
    proof_public_inputs_hash: Hash32
    ciphertext_hash: Hash32
    reason_hash: Hash32
    mod_set_version: int
    k: int
    n: int
    threshold_public_key_hash: Hash32

    def hash(self) -> Hash32:
        return digest(
            "certificate-statement",
            self.forum_id,
            self.post_id,
            self.content_id,
            self.proof_public_inputs_hash,
            self.ciphertext_hash,
            self.reason_hash,
            _u64(self.mod_set_version),
            _u64(self.k),
            _u64(self.n),
            self.threshold_public_key_hash,
        )


@dataclass(frozen=True)
class ModerationVote:
    moderator_id: str
    statement_hash: Hash32
    signature: bytes


@dataclass(frozen=True)
class ModerationCertificate:
    statement: CertificateStatement
    votes: Tuple[ModerationVote, ...]
    revealed_share: ShamirShare


@dataclass(frozen=True)
class SlashResult:
    commitment: Hash32
    reconstructed_coeffs: Tuple[FieldElem, ...]


def statement_for(forum: ForumConfig, post: AnonymousPostEnvelope, reason_hash: Hash32) -> CertificateStatement:
    return CertificateStatement(
        forum_id=forum.forum_id,
        post_id=post.post_id,
        content_id=post.content_id,
        proof_public_inputs_hash=post.proof_public_inputs_hash,
        ciphertext_hash=post.ciphertext_hash,
        reason_hash=reason_hash,
        mod_set_version=forum.mod_set_version,
        k=forum.k,
        n=forum.n,
        threshold_public_key_hash=forum.threshold_public_key_hash,
    )


def create_vote(forum: ForumConfig, moderator_id: str, post: AnonymousPostEnvelope, reason_hash: Hash32) -> ModerationVote:
    key = forum.moderators[moderator_id]
    statement_hash = statement_for(forum, post, reason_hash).hash()
    return ModerationVote(moderator_id, statement_hash, key.sign(statement_hash))


def verify_vote(forum: ForumConfig, vote: ModerationVote, statement_hash: Hash32) -> bool:
    key = forum.moderators.get(vote.moderator_id)
    if key is None:
        return False
    return vote.statement_hash == statement_hash and key.verify(statement_hash, vote.signature)


def aggregate_certificate(
    forum: ForumConfig,
    oracle: ThresholdOracle,
    post: AnonymousPostEnvelope,
    reason_hash: Hash32,
    votes: Sequence[ModerationVote],
) -> ModerationCertificate:
    statement = statement_for(forum, post, reason_hash)
    statement_hash = statement.hash()
    signer_ids = [v.moderator_id for v in votes]
    if len(set(signer_ids)) < forum.n:
        raise ProtocolError("partial certificate: fewer than N distinct moderators")
    for vote in votes:
        if not verify_vote(forum, vote, statement_hash):
            raise ProtocolError(f"invalid moderator vote from {vote.moderator_id}")
    revealed_share = oracle.threshold_decrypt(post.ciphertext_hash, signer_ids, forum.n)
    return ModerationCertificate(statement, tuple(votes), revealed_share)


def verify_certificate(forum: ForumConfig, cert: ModerationCertificate) -> bool:
    st = cert.statement
    if st.forum_id != forum.forum_id or st.k != forum.k or st.n != forum.n:
        return False
    if st.mod_set_version != forum.mod_set_version:
        return False
    if st.threshold_public_key_hash != forum.threshold_public_key_hash:
        return False
    signer_ids = [v.moderator_id for v in cert.votes]
    if len(set(signer_ids)) < forum.n:
        return False
    statement_hash = st.hash()
    return all(verify_vote(forum, vote, statement_hash) for vote in cert.votes)


def slash(registry: Registry, certificates: Sequence[ModerationCertificate]) -> SlashResult:
    forum = registry.forum
    if len(certificates) != forum.k:
        raise ProtocolError(f"slash requires exactly K={forum.k} certificates")
    if not all(verify_certificate(forum, cert) for cert in certificates):
        raise ProtocolError("invalid certificate in slash bundle")

    shares = [cert.revealed_share for cert in certificates]
    if len({x for x, _ in shares}) != len(shares):
        raise ProtocolError("slash bundle contains duplicate Shamir x-coordinate")

    coeffs = tuple(interpolate_coeffs(shares))
    commitment = commitment_for(forum.forum_id, forum.k, coeffs)
    if not registry.is_active(commitment):
        raise ProtocolError("reconstructed commitment is not active in registry")
    registry.revoke(commitment)
    return SlashResult(commitment, coeffs)


def retroactively_link_posts(forum_id: bytes, coeffs: Sequence[int], posts: Sequence[AnonymousPostEnvelope]) -> List[AnonymousPostEnvelope]:
    linked: List[AnonymousPostEnvelope] = []
    for post in posts:
        expected = retro_tag_for(forum_id, coeffs, post.content_id, post.post_nonce)
        if expected == post.retro_tag:
            linked.append(post)
    return linked
