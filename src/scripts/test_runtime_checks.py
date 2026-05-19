import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class RuntimeCheckTests(unittest.TestCase):
    def run_json_script(self, name: str) -> dict:
        output = subprocess.check_output(
            ["python3", str(ROOT / "scripts" / name)],
            cwd=ROOT,
            text=True,
        )
        return json.loads(output)

    def test_lez_runtime_check_reports_structured_blockers(self):
        report = self.run_json_script("check_lez_runtime.py")

        self.assertIn(report["status"], {"ready", "blocked"})
        self.assertEqual(report["target"], "lez_runtime")
        self.assertIsInstance(report["blockers"], list)
        self.assertIn("deploy lp0016_registry", " ".join(report["ready_commands"]))

    def test_basecamp_inspector_check_reports_structured_blockers(self):
        report = self.run_json_script("check_basecamp_inspector.py")

        self.assertIn(report["status"], {"ready", "blocked"})
        self.assertEqual(report["target"], "basecamp_qml_inspector")
        self.assertIn("cache_roots", report)
        self.assertTrue(any(".cache/logos-basecamp" in root for root in report["cache_roots"]))
        self.assertIsInstance(report["blockers"], list)
        self.assertTrue(report["lp0016_ui_test"].endswith("ui-tests.mjs"))

    def test_basecamp_inspector_accepts_manual_artifacts_without_nix(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            basecamp_dir = root / "basecamp"
            qt_mcp = root / "qt-mcp"
            app = root / "LogosBasecamp"
            design_system = root / "design-system"
            basecamp_dir.mkdir()
            (qt_mcp / "test-framework").mkdir(parents=True)
            (qt_mcp / "test-framework" / "framework.mjs").write_text("")
            (design_system / "Logos" / "Theme").mkdir(parents=True)
            (design_system / "Logos" / "Controls").mkdir(parents=True)
            (design_system / "Logos" / "Theme" / "qmldir").write_text("")
            (design_system / "Logos" / "Controls" / "qmldir").write_text("")
            app.write_text("#!/usr/bin/env sh\nexit 0\n")
            app.chmod(0o755)

            env = os.environ.copy()
            env["LOGOS_BASECAMP_DIR"] = str(basecamp_dir)
            env["LOGOS_QT_MCP"] = str(qt_mcp)
            env["LOGOS_BASECAMP_APP"] = str(app)
            env["LOGOS_DESIGN_SYSTEM_ROOT"] = str(design_system)
            output = subprocess.check_output(
                ["python3", str(ROOT / "scripts" / "check_basecamp_inspector.py")],
                cwd=ROOT,
                env=env,
                text=True,
            )
            report = json.loads(output)
            blocker_ids = {blocker["id"] for blocker in report["blockers"]}

        self.assertEqual(report["artifact_status"], "ready")
        self.assertNotIn("logos_qt_mcp", blocker_ids)
        self.assertNotIn("basecamp_app", blocker_ids)
        self.assertNotIn("nix", blocker_ids)
        self.assertNotIn("logos_design_system", blocker_ids)
        self.assertIn("basecamp_inspector", blocker_ids)

    def test_basecamp_inspector_accepts_matching_clickthrough_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            basecamp_dir = root / "basecamp"
            qt_mcp = root / "qt-mcp"
            app = root / "LogosBasecamp"
            design_system = root / "design-system"
            evidence = root / "evidence.json"
            basecamp_dir.mkdir()
            (qt_mcp / "test-framework").mkdir(parents=True)
            (qt_mcp / "test-framework" / "framework.mjs").write_text("")
            (design_system / "Logos" / "Theme").mkdir(parents=True)
            (design_system / "Logos" / "Controls").mkdir(parents=True)
            (design_system / "Logos" / "Theme" / "qmldir").write_text("")
            (design_system / "Logos" / "Controls" / "qmldir").write_text("")
            app.write_text("#!/usr/bin/env sh\nexit 0\n")
            app.chmod(0o755)
            evidence.write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "basecamp_app": str(app),
                        "logos_qt_mcp": str(qt_mcp),
                        "design_system_qml": str(design_system),
                    }
                )
            )

            env = os.environ.copy()
            env["LOGOS_BASECAMP_DIR"] = str(basecamp_dir)
            env["LOGOS_QT_MCP"] = str(qt_mcp)
            env["LOGOS_BASECAMP_APP"] = str(app)
            env["LOGOS_DESIGN_SYSTEM_ROOT"] = str(design_system)
            env["LOGOS_BASECAMP_INSPECTOR_EVIDENCE"] = str(evidence)
            output = subprocess.check_output(
                ["python3", str(ROOT / "scripts" / "check_basecamp_inspector.py")],
                cwd=ROOT,
                env=env,
                text=True,
            )
            report = json.loads(output)

        self.assertEqual(report["status"], "ready")
        self.assertEqual(report["artifact_status"], "ready")
        self.assertEqual(report["inspector_evidence"]["status"], "accepted")
        self.assertEqual(report["click_through"]["status"], "evidence_accepted")
        self.assertEqual(report["blockers"], [])

    def test_local_submission_gate_exists_and_records_local_policy(self):
        gate = ROOT / "scripts" / "local_submission_gate.py"
        text = gate.read_text()

        self.assertIn("local integration evidence", text)
        self.assertIn("strict-runtime", text)
        self.assertIn("collect_localnet_evidence.py", text)
        self.assertIn("check_risc0_proof_performance.py", text)
        self.assertIn("scripts/check_lez_runtime.py", text)
        self.assertIn("scripts/check_basecamp_inspector.py", text)
        self.assertIn("scripts/check_noir_icing.py", text)

    def test_risc0_proof_performance_check_reports_structured_status(self):
        report = self.run_json_script("check_risc0_proof_performance.py")

        self.assertIn(report["status"], {"ready", "blocked"})
        self.assertEqual(report["target"], "risc0_proof_performance")
        self.assertIsInstance(report["blockers"], list)
        self.assertIn("threshold_seconds", report)

    def test_localnet_evidence_script_starts_direct_sequencer(self):
        script = ROOT / "scripts" / "collect_localnet_evidence.py"
        text = script.read_text()

        self.assertIn("sequencer_service", text)
        self.assertIn("RISC0_DEV_MODE", text)
        self.assertIn("localnet_evidence", text)
        self.assertIn("deploy", text)

    def test_live_network_check_reports_official_local_sequencer_context(self):
        report = self.run_json_script("check_live_network_deploy.py")

        self.assertIn("official_local_sequencer", report)
        self.assertIn("localhost:3040", report["official_local_sequencer"]["note"])
        self.assertEqual(report["official_local_sequencer"]["rpc"], "http://127.0.0.1:3040")
        self.assertIn("quickstart", report["official_local_sequencer"]["doc"])
        self.assertIn("reviewer_acceptance_note", report)

    def test_noir_icing_check_reports_structured_status(self):
        report = self.run_json_script("check_noir_icing.py")

        self.assertIn(report["status"], {"ready", "blocked"})
        self.assertEqual(report["target"], "noir_icing")
        self.assertEqual(report["package"], "noir/post_binding")
        self.assertIn("RISC0 remains", report["note"])
        self.assertIn("installation", report["docs"])
        self.assertIsInstance(report["blockers"], list)

    def test_submission_video_is_documented_and_reproducible(self):
        readme = (ROOT.parent / "README.md").read_text()
        script = (ROOT / "scripts" / "make_submission_video.py").read_text()
        criteria = json.loads((ROOT / "docs" / "success_criteria.json").read_text())
        video_entry = next(item for item in criteria["criteria"] if item["id"] == "SC-SUP-06")

        self.assertIn("submission/lp0016-demo.mp4", readme)
        self.assertIn("submission/lp0016-demo-poster.jpg", readme)
        self.assertIn("[![LP-0016 narrated demo video first frame]", readme)
        self.assertIn("scripts/make_submission_video.py", readme)
        self.assertIn("ffmpeg", script)
        self.assertIn("say", script)
        self.assertIn("localnet_evidence", script)
        self.assertEqual(video_entry["current_status"], "local_artifact")
        self.assertTrue(any(proof["path"] == "../submission/lp0016-demo.mp4" for proof in video_entry["proofs"]))


if __name__ == "__main__":
    unittest.main()
