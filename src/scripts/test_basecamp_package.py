import json
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP_DIR = ROOT / "app" / "basecamp-forum"


class BasecampPackageTests(unittest.TestCase):
    def test_manifest_matches_basecamp_ui_qml_contract(self):
        manifest = json.loads((APP_DIR / "manifest.json").read_text())
        metadata = json.loads((APP_DIR / "metadata.json").read_text())

        self.assertEqual(manifest["manifestVersion"], "0.1.0")
        self.assertEqual(manifest["type"], "ui_qml")
        self.assertEqual(metadata["type"], "ui_qml")
        self.assertEqual(metadata["pluginType"], "qml")
        self.assertEqual(metadata["main"], "Main.qml")
        self.assertIn("darwin-arm64", manifest["main"])

    def test_qml_root_can_embed_in_basecamp_qquickwidget(self):
        qml = (APP_DIR / "Main.qml").read_text()
        first_object = next(
            line.strip()
            for line in qml.splitlines()
            if line.strip() and not line.startswith("import ")
        )

        self.assertEqual(first_object, "Item {")
        self.assertNotIn("ApplicationWindow", qml)

    def test_package_script_creates_lgx_with_platform_variant(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = subprocess.check_output(
                [str(ROOT / "scripts" / "package_basecamp.sh"), tmp],
                text=True,
                cwd=ROOT,
            ).strip()

            package = Path(out)
            self.assertTrue(package.exists())
            with tarfile.open(package, "r:gz") as archive:
                names = set(archive.getnames())

            self.assertIn("./manifest.json", names)
            self.assertIn("./variants/darwin-arm64/Main.qml", names)
            self.assertIn("./variants/darwin-arm64/metadata.json", names)

    def test_qml_inspector_click_through_spec_exists(self):
        test_file = APP_DIR / "ui-tests.mjs"
        script = test_file.read_text()

        self.assertIn("click through the full moderation flow", script)
        self.assertIn("receipt cannot prove non-membership", script)


if __name__ == "__main__":
    unittest.main()
