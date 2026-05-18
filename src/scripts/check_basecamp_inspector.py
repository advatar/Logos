#!/usr/bin/env python3
"""Report whether the Basecamp QML inspector click-through harness is runnable."""

from __future__ import annotations

import argparse
import json
import os
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP_DIR = ROOT / "app" / "basecamp-forum"


def first_existing(paths: list[Path]) -> Path | None:
    for path in paths:
        if path.exists():
            return path
    return None


def build_report() -> dict:
    basecamp_dir = Path(
        os.environ.get("LOGOS_BASECAMP_DIR", "/tmp/logos-basecamp-inspect")
    )
    qt_mcp = Path(
        os.environ.get("LOGOS_QT_MCP", str(basecamp_dir / "result-mcp"))
    )
    app_binary_value = os.environ.get("LOGOS_BASECAMP_APP")
    app_binary = Path(app_binary_value) if app_binary_value else None
    test_file = APP_DIR / "ui-tests.mjs"
    packaged_app = first_existing(
        [
            ROOT / "dist" / "basecamp" / "lp0016-anon-forum-demo.lgx",
            Path("/tmp/lp0016-basecamp/lp0016-anon-forum-demo.lgx"),
        ]
    )

    blockers: list[dict[str, str]] = []
    if not basecamp_dir.exists():
        blockers.append(
            {
                "id": "basecamp_source",
                "reason": f"Basecamp source checkout not found at {basecamp_dir}",
                "next": "set LOGOS_BASECAMP_DIR or clone logos-co/logos-basecamp",
            }
        )
    if not (qt_mcp / "test-framework" / "framework.mjs").exists():
        blockers.append(
            {
                "id": "logos_qt_mcp",
                "reason": f"QML inspector test framework not found under {qt_mcp}",
                "next": "in the Basecamp checkout, run nix build .#logos-qt-mcp -o result-mcp",
            }
        )
    if shutil.which("node") is None:
        blockers.append(
            {
                "id": "node",
                "reason": "Node.js is not available for the inspector test runner",
                "next": "install Node.js or use the Nix integration-test target",
            }
        )
    if shutil.which("nix") is None:
        blockers.append(
            {
                "id": "nix",
                "reason": "Nix is not available to build Basecamp and logos-qt-mcp hermetically",
                "next": "install Nix with flakes enabled, or provide LOGOS_QT_MCP and LOGOS_BASECAMP_APP",
            }
        )
    if app_binary is None or not os.access(app_binary, os.X_OK):
        blockers.append(
            {
                "id": "basecamp_app",
                "reason": "LOGOS_BASECAMP_APP does not point to an executable LogosBasecamp binary",
                "next": "build Basecamp and set LOGOS_BASECAMP_APP=/path/to/LogosBasecamp",
            }
        )
    if not test_file.exists():
        blockers.append(
            {
                "id": "lp0016_ui_test",
                "reason": "LP-0016 inspector test file is missing",
                "next": "add app/basecamp-forum/ui-tests.mjs",
            }
        )

    return {
        "status": "ready" if not blockers else "blocked",
        "target": "basecamp_qml_inspector",
        "basecamp_dir": str(basecamp_dir),
        "logos_qt_mcp": str(qt_mcp),
        "basecamp_app": str(app_binary) if app_binary else None,
        "lp0016_ui_test": str(test_file),
        "packaged_app": str(packaged_app) if packaged_app else None,
        "blockers": blockers,
        "ready_commands": [
            "scripts/package_basecamp.sh /tmp/lp0016-basecamp",
            "cd $LOGOS_BASECAMP_DIR && nix build .#logos-qt-mcp -o result-mcp",
            "LOGOS_QT_MCP=$LOGOS_BASECAMP_DIR/result-mcp node app/basecamp-forum/ui-tests.mjs --ci $LOGOS_BASECAMP_APP",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pretty", action="store_true", help="pretty-print JSON")
    args = parser.parse_args()
    report = build_report()
    print(json.dumps(report, indent=2 if args.pretty else None, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
