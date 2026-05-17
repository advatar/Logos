import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

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
    interpolate_coeffs,
    retroactively_link_posts,
    share_for,
    slash,
    verify_certificate,
    verify_post,
)


def mod(mid: str) -> ModeratorKey:
    return ModeratorKey(mid, digest("test-moderator-secret", mid.encode()))


class ProtocolTests(unittest.TestCase):
    def setUp(self):
        moderators = {m.moderator_id: m for m in [mod("a"), mod("b"), mod("c")]}
        self.forum = ForumConfig(forum_id=digest("forum", b"test"), k=2, n=2, moderators=moderators)
        self.registry = Registry(self.forum)
        self.oracle = ThresholdOracle()
        self.member = MemberSecret.from_seed(self.forum.forum_id, self.forum.k, b"member")
        self.registry.register(self.member.commitment(self.forum.forum_id))

    def test_shamir_interpolation_recovers_coefficients(self):
        shares = [share_for(self.forum.forum_id, self.member.coeffs, b"c1", b"n1"), share_for(self.forum.forum_id, self.member.coeffs, b"c2", b"n2")]
        self.assertEqual(tuple(interpolate_coeffs(shares)), self.member.coeffs)

    def test_post_verifies_before_revocation(self):
        post = build_post(self.registry, self.oracle, self.member, b"content", b"nonce")
        self.assertTrue(verify_post(self.registry, post))

    def test_partial_certificate_rejected(self):
        post = build_post(self.registry, self.oracle, self.member, b"bad", b"nonce")
        reason = digest("reason", b"x")
        votes = [create_vote(self.forum, "a", post, reason)]
        with self.assertRaises(ProtocolError):
            aggregate_certificate(self.forum, self.oracle, post, reason, votes)

    def test_certificate_verifies_with_n_votes(self):
        post = build_post(self.registry, self.oracle, self.member, b"bad", b"nonce")
        reason = digest("reason", b"x")
        votes = [create_vote(self.forum, "a", post, reason), create_vote(self.forum, "b", post, reason)]
        cert = aggregate_certificate(self.forum, self.oracle, post, reason, votes)
        self.assertTrue(verify_certificate(self.forum, cert))

    def test_slash_revokes_and_blocks_future_post(self):
        reason = digest("reason", b"x")
        certs = []
        posts = []
        for i in range(2):
            post = build_post(self.registry, self.oracle, self.member, f"bad-{i}".encode(), f"nonce-{i}".encode())
            posts.append(post)
            votes = [create_vote(self.forum, "a", post, reason), create_vote(self.forum, "b", post, reason)]
            certs.append(aggregate_certificate(self.forum, self.oracle, post, reason, votes))
        result = slash(self.registry, certs)
        self.assertIn(result.commitment, self.registry.revoked)
        with self.assertRaises(ProofGenerationError):
            build_post(self.registry, self.oracle, self.member, b"after", b"nonce-after")
        linked = retroactively_link_posts(self.forum.forum_id, result.reconstructed_coeffs, posts)
        self.assertEqual(len(linked), 2)

    def test_cross_forum_replay_rejected(self):
        post = build_post(self.registry, self.oracle, self.member, b"bad", b"nonce")
        other_forum = ForumConfig(forum_id=digest("forum", b"other"), k=2, n=2, moderators=self.forum.moderators)
        other_registry = Registry(other_forum)
        self.assertFalse(verify_post(other_registry, post))


if __name__ == "__main__":
    unittest.main()
