import json
import re
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = ROOT.parent


def read(relative: str) -> str:
    return (ROOT / relative).read_text()


class PhaseClosureTests(unittest.TestCase):
    def test_phase_2_pedersen_dkg_replaces_trusted_dealer(self):
        threshold = read("crates/protocol-core/src/threshold.rs")
        exports = read("crates/protocol-core/src/lib.rs")

        self.assertIn("pub fn pedersen_dkg", threshold)
        self.assertIn("pub struct PedersenDkgTranscript", threshold)
        self.assertIn("pub struct PedersenDkgContribution", threshold)
        self.assertIn("fn pedersen_dkg_transcript_rejects_tampered_share", threshold)
        self.assertIn("PedersenDkgTranscript", exports)
        self.assertNotIn("DealerShares::trusted", threshold)

    def test_phase_3_revocation_non_membership_is_enforced(self):
        merkle = read("crates/protocol-core/src/merkle.rs")
        statement = read("crates/risc0-statement/src/lib.rs")

        for symbol in [
            "pub struct NonMembershipProof",
            "pub fn prove_non_membership",
            "pub fn verify_non_membership",
            "non_membership_proof_round_trips_between_neighbors",
        ]:
            self.assertIn(symbol, merkle)
        self.assertIn("pub revocation_non_membership: NonMembershipProof", statement)
        self.assertIn("merkle_verify_non_membership", statement)
        self.assertIn("verify_rejects_revoked_member_root", statement)

    def test_phase_4_risc0_receipt_bytes_cross_protocol_and_app_boundaries(self):
        protocol_types = read("crates/protocol-core/src/types.rs")
        sdk = read("crates/moderation-sdk/src/lib.rs")
        host = read("zk/membership-host/src/lib.rs")
        basecamp_core = read("app/basecamp-forum/core-module/src/lib.rs")
        demo = read("scripts/demo_e2e.sh")

        for text in [protocol_types, sdk, basecamp_core]:
            self.assertIn("ZkReceipt::Risc0", text)
            self.assertIn("receipt_bytes", text)
        self.assertIn("protocol_core::ZkReceipt::risc0", host)
        self.assertIn("receipt_bytes", host)
        self.assertIn("pub fn attach_risc0_receipt", protocol_types)
        self.assertIn("pub fn attach_risc0_receipt", sdk)
        self.assertIn('RISC0_DEV_MODE:-1}" == "0"', demo)
        self.assertIn("cargo +stable test --features risc0", demo)
        self.assertNotIn("MockZkReceipt::", protocol_types)

    def test_phase_5_spel_macro_gap_is_explicit_until_lez_framework_migration(self):
        registry = read("registry/lp0016-registry/src/lib.rs")
        registry_manifest = read("registry/lp0016-registry/Cargo.toml")
        scaffold = read("scaffold.toml")
        status = (REPO_ROOT / "STATUS.md").read_text()
        repo = (REPO_ROOT / "REPO.md").read_text()

        self.assertIn("// #[lez_program", registry)
        self.assertIn("spel = []", registry_manifest)
        self.assertIn('kind = "default"', scaffold)
        self.assertIn("Real SPEL macro flip", status)
        self.assertIn("LEZ/SPEL registry remains the main placeholder boundary", repo)

    def test_phase_6_storage_namespaces_and_retry_queue_are_in_sdk(self):
        sdk = read("crates/moderation-sdk/src/lib.rs")

        for symbol in [
            "pub trait RetryQueue",
            "pub struct MemoryRetryQueue",
            "pub enum StorageKind",
            "pub fn storage_namespace",
            "StorageKind::Post",
            "StorageKind::Vote",
            "StorageKind::Cert",
            "StorageKind::Slash",
            "RetryTask::Put",
            "RetryTask::SubmitSlash",
            "namespaces_are_partitioned_by_kind_and_forum",
            "failed_put_is_queued_for_retry",
        ]:
            self.assertIn(symbol, sdk)

    def test_phase_7_lean_shamir_and_slash_surfaces_build_without_sorry(self):
        lean_files = [
            ROOT / "lean" / "AnonymousForum" / "Shamir.lean",
            ROOT / "lean" / "AnonymousForum" / "Slash.lean",
            ROOT / "lean" / "AnonymousForum" / "ShamirTargets.lean",
        ]
        combined = "\n".join(path.read_text() for path in lean_files)

        self.assertIn("theorem lagrange_reconstructs_original_polynomial", combined)
        self.assertIn("lagrangeSound", combined)
        self.assertIn("theorem slash_sound", combined)
        self.assertIn("theorem slash_bundle_certificate_thresholds", combined)
        self.assertIsNone(re.search(r"\bsorry\b", combined))

    def test_phase_8_basecamp_qml_app_and_clickthrough_spec_exist(self):
        main_qml = read("app/basecamp-forum/Main.qml")
        ui_tests = read("app/basecamp-forum/ui-tests.mjs")
        package = read("scripts/package_basecamp.sh")
        manifest = read("app/basecamp-forum/manifest.json")
        inspector = read("scripts/check_basecamp_inspector.py")

        for screen in [
            "Forum",
            "Register",
            "Post",
            "Moderate",
            "Vote",
            "Certificate",
            "History",
            "Slash",
            "Rejected",
        ]:
            self.assertIn(f'"{screen}"', main_qml)
        self.assertIn("click through the full moderation flow", ui_tests)
        self.assertIn("direct sidebar navigation reaches slash screen", ui_tests)
        self.assertIn("ui_qml", manifest)
        self.assertIn("lp0016-anon-forum-demo.lgx", package)
        self.assertIn("logos_qt_mcp", inspector)

    def test_phase_9_ci_risc0_and_cu_readiness_entries_exist(self):
        ci = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text()
        measure = read("scripts/measure_cu.sh")

        self.assertIn("risc0-feature", ci)
        self.assertIn("--features risc0", ci)
        self.assertIn("./scripts/measure_cu.sh", ci)
        self.assertIn("check_lez_runtime.py", measure)

    def test_external_runtime_blockers_are_structured_and_traceable(self):
        for script, target in [
            ("check_lez_runtime.py", "lez_runtime"),
            ("check_basecamp_inspector.py", "basecamp_qml_inspector"),
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
            if report["status"] == "blocked":
                self.assertTrue(report["blockers"])
                for blocker in report["blockers"]:
                    self.assertTrue(blocker["id"])
                    self.assertTrue(blocker["reason"])
                    self.assertTrue(blocker["next"])


if __name__ == "__main__":
    unittest.main()
