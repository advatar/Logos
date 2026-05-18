#!/usr/bin/env python3
"""Run a dependency-free LP-0016 end-to-end protocol simulation."""

from lp0016_sim import (
    ForumConfig,
    MemberSecret,
    ModeratorKey,
    ProofGenerationError,
    ProtocolError,
    Registry,
    ThresholdOracle,
    aggregate_certificate,
    build_post,
    create_vote,
    digest,
    hex8,
    retroactively_link_posts,
    slash,
    verify_post,
)


def mod(mid: str) -> ModeratorKey:
    return ModeratorKey(mid, digest("demo-moderator-secret", mid.encode()))


def run_forum_a() -> None:
    print("\n=== Forum A: K=2 strikes, N=2-of-3 moderators ===")
    moderators = {m.moderator_id: m for m in [mod("alice"), mod("bob"), mod("carol")]}
    forum = ForumConfig(
        forum_id=digest("forum", b"forum-a"),
        k=2,
        n=2,
        moderators=moderators,
        threshold_public_key_hash=digest("threshold-pk", b"forum-a"),
    )
    registry = Registry(forum)
    oracle = ThresholdOracle()

    member = MemberSecret.from_seed(forum.forum_id, forum.k, b"member-1-seed")
    commitment = member.commitment(forum.forum_id)
    registry.register(commitment)
    print(f"registered member commitment: {hex8(commitment)}…")

    posts = [
        build_post(registry, oracle, member, b"post:hello", b"nonce-1"),
        build_post(registry, oracle, member, b"post:bad-1", b"nonce-2"),
        build_post(registry, oracle, member, b"post:bad-2", b"nonce-3"),
    ]
    for p in posts:
        assert verify_post(registry, p)
    print("3 anonymous posts verified; public envelopes do not reveal the commitment")

    reason = digest("reason", b"rule-violation")

    try:
        one_vote = [create_vote(forum, "alice", posts[1], reason)]
        aggregate_certificate(forum, oracle, posts[1], reason, one_vote)
        raise AssertionError("partial certificate unexpectedly succeeded")
    except ProtocolError as exc:
        print(f"partial certificate rejected: {exc}")

    certs = []
    for post in posts[1:3]:
        votes = [create_vote(forum, "alice", post, reason), create_vote(forum, "bob", post, reason)]
        cert = aggregate_certificate(forum, oracle, post, reason, votes)
        certs.append(cert)
        print(f"certificate for post {hex8(post.post_id)}… revealed share x={cert.revealed_share[0]}")

    result = slash(registry, certs)
    print(f"slash reconstructed commitment: {hex8(result.commitment)}…")
    print(f"commitment revoked: {result.commitment in registry.revoked}")

    linked = retroactively_link_posts(forum.forum_id, result.reconstructed_coeffs, posts)
    print(f"retroactively linked posts from slashed member: {[hex8(p.post_id) for p in linked]}")

    try:
        build_post(registry, oracle, member, b"post:after-slash", b"nonce-4")
        raise AssertionError("post after revocation unexpectedly succeeded")
    except ProofGenerationError as exc:
        print(f"post after slash rejected before consuming nullifier: {exc}")


def run_forum_b() -> None:
    print("\n=== Forum B: independent parameters K=3, N=1-of-2 moderators ===")
    moderators = {m.moderator_id: m for m in [mod("dave"), mod("erin")]}
    forum = ForumConfig(
        forum_id=digest("forum", b"forum-b"),
        k=3,
        n=1,
        moderators=moderators,
        threshold_public_key_hash=digest("threshold-pk", b"forum-b"),
    )
    registry = Registry(forum)
    oracle = ThresholdOracle()
    member = MemberSecret.from_seed(forum.forum_id, forum.k, b"member-2-seed")
    registry.register(member.commitment(forum.forum_id))

    posts = [build_post(registry, oracle, member, f"b-post-{i}".encode(), f"b-nonce-{i}".encode()) for i in range(3)]
    reason = digest("reason", b"forum-b-rule")
    certs = []
    for post in posts:
        votes = [create_vote(forum, "dave", post, reason)]
        certs.append(aggregate_certificate(forum, oracle, post, reason, votes))
    result = slash(registry, certs)
    print(f"Forum B slash succeeded with independent commitment {hex8(result.commitment)}…")


def main() -> None:
    run_forum_a()
    run_forum_b()
    print("\nLocal demo completed successfully.")


if __name__ == "__main__":
    main()
