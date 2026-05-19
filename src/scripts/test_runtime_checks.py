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

        self.assertNotIn("logos_qt_mcp", blocker_ids)
        self.assertNotIn("basecamp_app", blocker_ids)
        self.assertNotIn("nix", blocker_ids)
        self.assertNotIn("logos_design_system", blocker_ids)

    def test_local_submission_gate_exists_and_records_local_policy(self):
        gate = ROOT / "scripts" / "local_submission_gate.py"
        text = gate.read_text()

        self.assertIn("local integration evidence", text)
        self.assertIn("strict-runtime", text)
        self.assertIn("scripts/check_lez_runtime.py", text)
        self.assertIn("scripts/check_basecamp_inspector.py", text)


if __name__ == "__main__":
    unittest.main()
