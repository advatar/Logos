import json
import subprocess
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
        self.assertIsInstance(report["blockers"], list)
        self.assertTrue(report["lp0016_ui_test"].endswith("ui-tests.mjs"))


if __name__ == "__main__":
    unittest.main()
