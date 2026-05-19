import json
import subprocess
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
    retroactively_link_posts,
    slash,
    verify_certificate,
    verify_post,
)


ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = ROOT.parent
CRITERIA_PATH = ROOT / "docs" / "success_criteria.json"

EXPECTED_IDS = {
    "SC-FUNC-01",
    "SC-FUNC-02",
    "SC-FUNC-03",
    "SC-FUNC-04",
    "SC-FUNC-05",
    "SC-FUNC-06",
    "SC-FUNC-07",
    "SC-FUNC-08",
    "SC-FUNC-09",
    "SC-USE-01",
    "SC-USE-02",
    "SC-REL-01",
    "SC-REL-02",
    "SC-REL-03",
    "SC-PERF-01",
    "SC-PERF-02",
    "SC-SUP-01",
    "SC-SUP-02",
    "SC-SUP-03",
    "SC-SUP-04",
    "SC-SUP-05",
    "SC-SUP-06",
}

EXTERNAL_IDS = {
    "SC-FUNC-08",
    "SC-FUNC-09",
    "SC-USE-02",
    "SC-PERF-01",
    "SC-PERF-02",
    "SC-SUP-01",
    "SC-SUP-02",
    "SC-SUP-03",
    "SC-SUP-04",
    "SC-SUP-05",
    "SC-SUP-06",
}


def mod(mid: str) -> ModeratorKey:
    return ModeratorKey(mid, digest("success-moderator-secret", mid.encode()))


def forum(k: int = 2, n: int = 2, suffix: bytes = b"a") -> ForumConfig:
    moderators = {m.moderator_id: m for m in [mod("a"), mod("b"), mod("c")]}
    return ForumConfig(
        forum_id=digest("forum", b"success", suffix),
        k=k,
        n=n,
        moderators=moderators,
        threshold_public_key_hash=digest("threshold-pk", suffix),
    )


def certificate_for(
    cfg: ForumConfig,
    registry: Registry,
    oracle: ThresholdOracle,
    member: MemberSecret,
    idx: int,
):
    post = build_post(
        registry,
        oracle,
        member,
        f"content-{idx}".encode(),
        f"nonce-{idx}".encode(),
    )
    reason = digest("reason", b"rule")
    votes = [
        create_vote(cfg, "a", post, reason),
        create_vote(cfg, "b", post, reason),
    ][: cfg.n]
    cert = aggregate_certificate(cfg, oracle, post, reason, votes)
    return post, cert


class SuccessCriteriaBehaviorTests(unittest.TestCase):
    def setUp(self):
        self.forum = forum()
        self.registry = Registry(self.forum)
        self.oracle = ThresholdOracle()
        self.member = MemberSecret.from_seed(self.forum.forum_id, self.forum.k, b"member")
        self.other = MemberSecret.from_seed(self.forum.forum_id, self.forum.k, b"other")
        self.registry.register(self.member.commitment(self.forum.forum_id))
        self.registry.register(self.other.commitment(self.forum.forum_id))

    def test_registration_posts_and_unlinkability(self):
        posts = [
            build_post(self.registry, self.oracle, self.member, b"content-a", b"nonce-a"),
            build_post(self.registry, self.oracle, self.member, b"content-b", b"nonce-b"),
            build_post(self.registry, self.oracle, self.other, b"content-c", b"nonce-c"),
        ]

        self.assertTrue(all(verify_post(self.registry, post) for post in posts))
        self.assertEqual(len({post.post_id for post in posts}), len(posts))
        self.assertEqual(len({post.retro_tag for post in posts}), len(posts))
        self.assertEqual(len({post.proof_public_inputs_hash for post in posts}), len(posts))

    def test_slash_links_only_the_slashed_members_posts(self):
        slashed_posts_and_certs = [
            certificate_for(self.forum, self.registry, self.oracle, self.member, idx)
            for idx in range(self.forum.k)
        ]
        other_post = build_post(self.registry, self.oracle, self.other, b"other", b"nonce")

        result = slash(self.registry, [cert for _, cert in slashed_posts_and_certs])
        linked = retroactively_link_posts(
            self.forum.forum_id,
            result.reconstructed_coeffs,
            [post for post, _ in slashed_posts_and_certs] + [other_post],
        )

        self.assertEqual(
            {post.post_id for post in linked},
            {post.post_id for post, _ in slashed_posts_and_certs},
        )
        self.assertNotIn(other_post.post_id, {post.post_id for post in linked})

    def test_threshold_certificate_rejects_fewer_than_n(self):
        post = build_post(self.registry, self.oracle, self.member, b"bad", b"nonce")
        reason = digest("reason", b"rule")
        one_vote = [create_vote(self.forum, "a", post, reason)]

        with self.assertRaises(ProtocolError):
            aggregate_certificate(self.forum, self.oracle, post, reason, one_vote)

        two_votes = one_vote + [create_vote(self.forum, "b", post, reason)]
        cert = aggregate_certificate(self.forum, self.oracle, post, reason, two_votes)
        self.assertTrue(verify_certificate(self.forum, cert))

    def test_k_certificates_revoke_and_block_future_posts(self):
        certs = [
            certificate_for(self.forum, self.registry, self.oracle, self.member, idx)[1]
            for idx in range(self.forum.k)
        ]

        result = slash(self.registry, certs)

        self.assertIn(result.commitment, self.registry.revoked)
        with self.assertRaises(ProofGenerationError):
            build_post(self.registry, self.oracle, self.member, b"after", b"slash")

    def test_forums_are_independently_parameterized(self):
        forum_a = forum(k=2, n=2, suffix=b"a")
        forum_b = forum(k=3, n=1, suffix=b"b")

        self.assertEqual((forum_a.k, forum_a.n), (2, 2))
        self.assertEqual((forum_b.k, forum_b.n), (3, 1))
        self.assertNotEqual(forum_a.forum_id, forum_b.forum_id)

    def test_proof_generation_failure_is_retryable_without_state_change(self):
        before_registered = set(self.registry.registered)
        before_revoked = set(self.registry.revoked)
        unregistered = MemberSecret.from_seed(self.forum.forum_id, self.forum.k, b"new")

        with self.assertRaisesRegex(ProofGenerationError, "not registered|revoked"):
            build_post(self.registry, self.oracle, unregistered, b"content", b"nonce")

        self.assertEqual(self.registry.registered, before_registered)
        self.assertEqual(self.registry.revoked, before_revoked)


class SuccessCriteriaMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.matrix = json.loads(CRITERIA_PATH.read_text())

    def test_success_matrix_has_all_criteria_issues_and_proofs(self):
        criteria = self.matrix["criteria"]
        ids = {entry["id"] for entry in criteria}

        self.assertEqual(ids, EXPECTED_IDS)
        self.assertEqual(len(criteria), len(EXPECTED_IDS))
        for entry in criteria:
            self.assertRegex(entry["issue"], r"^https://github.com/advatar/Logos/issues/\d+$")
            self.assertTrue(entry["criterion"])
            self.assertTrue(entry["proofs"], entry["id"])
            self.assertIn(
                entry["current_status"],
                {
                    "local_proof",
                    "runtime_ready_check",
                    "external_runtime_required",
                    "submission_artifact_required",
                },
            )

    def test_proof_paths_exist_for_local_tests_and_diagnostics(self):
        checked_kinds = {"unit", "doc", "rust", "integration", "diagnostic"}
        for entry in self.matrix["criteria"]:
            for proof in entry["proofs"]:
                if proof["kind"] not in checked_kinds:
                    continue
                path = ROOT / proof["path"]
                self.assertTrue(path.exists(), f"{entry['id']} proof path missing: {path}")

    def test_external_runtime_criteria_have_diagnostics_or_artifacts(self):
        criteria_by_id = {entry["id"]: entry for entry in self.matrix["criteria"]}
        for criterion_id in EXTERNAL_IDS:
            proof_kinds = {proof["kind"] for proof in criteria_by_id[criterion_id]["proofs"]}
            self.assertTrue(
                {"diagnostic", "artifact", "integration"} & proof_kinds,
                criterion_id,
            )

    def test_protocol_docs_record_retroactive_linkability_boundary(self):
        protocol = (ROOT / "docs" / "protocol.md").read_text().lower()

        self.assertIn("historical posts", protocol)
        self.assertIn("no other member", protocol)
        self.assertIn("anonymity is affected", protocol)

    def test_sdk_api_docs_cover_forum_agnostic_surface(self):
        api = (ROOT / "docs" / "api.md").read_text()

        for symbol in [
            "register_member",
            "build_post",
            "attach_risc0_receipt",
            "create_moderation_vote",
            "aggregate_certificate",
            "submit_slash",
            "OffchainStore",
            "RetryQueue",
        ]:
            self.assertIn(symbol, api)

    def test_readme_names_e2e_usage_and_remaining_address_artifacts(self):
        readme = (ROOT / "README.md").read_text()

        for phrase in [
            "LEZ",
            "Basecamp",
            "program IDs",
            "register",
            "posting",
            "moderating",
            "slash",
        ]:
            self.assertIn(phrase, readme)

    def test_local_submission_gate_runs_success_criteria_and_runtime_checks(self):
        ci = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text()
        gate = (ROOT / "scripts" / "local_submission_gate.py").read_text()
        localnet = (ROOT / "scripts" / "collect_localnet_evidence.py").read_text()

        self.assertIn("test_success_criteria.py", ci)
        self.assertIn("test_runtime_checks.py", ci)
        self.assertIn("measure_cu.sh", ci)
        self.assertIn("test_success_criteria.py", gate)
        self.assertIn("test_runtime_checks.py", gate)
        self.assertIn("measure_cu.sh", gate)
        self.assertIn("RISC0_DEV_MODE", localnet)
        self.assertIn("localnet_evidence", localnet)

    def test_runtime_diagnostics_are_structured(self):
        for script, target in [
            ("check_lez_runtime.py", "lez_runtime"),
            ("check_basecamp_inspector.py", "basecamp_qml_inspector"),
            ("check_live_network_deploy.py", "live_lez_deployment"),
        ]:
            output = subprocess.check_output(
                ["python3", str(ROOT / "scripts" / script)],
                cwd=ROOT,
                text=True,
            )
            report = json.loads(output)
            self.assertIn(report["status"], {"ready", "blocked"})
            self.assertEqual(report["target"], target)
            self.assertIsInstance(report["blockers"], list)


if __name__ == "__main__":
    unittest.main()
