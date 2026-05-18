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


def first_qt_mcp(paths: list[Path]) -> Path | None:
    for path in paths:
        if (path / "test-framework" / "framework.mjs").exists():
            return path
    return None


def first_executable(paths: list[Path]) -> Path | None:
    for path in paths:
        if os.access(path, os.X_OK):
            return path
    return None


def first_design_system_qml(paths: list[Path]) -> Path | None:
    for path in paths:
        if (
            (path / "Logos" / "Theme" / "qmldir").exists()
            and (path / "Logos" / "Controls" / "qmldir").exists()
        ):
            return path
    return None


def build_report() -> dict:
    basecamp_dir = Path(
        os.environ.get("LOGOS_BASECAMP_DIR", "/tmp/logos-basecamp-inspect")
    )
    qt_mcp_value = os.environ.get("LOGOS_QT_MCP")
    qt_mcp_candidates = [
        Path(qt_mcp_value) if qt_mcp_value else None,
        basecamp_dir / "result-mcp",
        Path("/tmp/logos-qt-mcp-inspect"),
    ]
    qt_mcp = first_qt_mcp([path for path in qt_mcp_candidates if path is not None])
    if qt_mcp is None:
        qt_mcp = Path(qt_mcp_value) if qt_mcp_value else basecamp_dir / "result-mcp"

    app_binary_value = os.environ.get("LOGOS_BASECAMP_APP")
    app_binary_candidates = [
        Path(app_binary_value) if app_binary_value else None,
        basecamp_dir / "result" / "bin" / "LogosBasecamp",
        basecamp_dir / "result" / "LogosBasecamp.app" / "Contents" / "MacOS" / "LogosBasecamp",
        Path("/tmp/logos-basecamp-build/LogosBasecamp"),
        Path("/tmp/logos-basecamp-runtime/LogosBasecamp.app/Contents/MacOS/LogosBasecamp"),
        Path("/Applications/LogosBasecamp.app/Contents/MacOS/LogosBasecamp"),
        Path.home() / "Applications" / "LogosBasecamp.app" / "Contents" / "MacOS" / "LogosBasecamp",
    ]
    app_binary = first_executable([path for path in app_binary_candidates if path is not None])
    if app_binary is None and app_binary_value:
        app_binary = Path(app_binary_value)
    test_file = APP_DIR / "ui-tests.mjs"
    packaged_app = first_existing(
        [
            ROOT / "dist" / "basecamp" / "lp0016-anon-forum-demo.lgx",
            Path("/tmp/lp0016-basecamp/lp0016-anon-forum-demo.lgx"),
        ]
    )
    design_system_value = os.environ.get("LOGOS_DESIGN_SYSTEM_ROOT")
    design_system_candidates = [
        Path(design_system_value) if design_system_value else None,
        Path(design_system_value) / "src" / "qml" if design_system_value else None,
        Path(design_system_value) / "lib" if design_system_value else None,
        basecamp_dir.parent / "logos-design-system" / "src" / "qml",
        Path("/tmp/logos-design-system/src/qml"),
    ]
    design_system_qml = first_design_system_qml(
        [path for path in design_system_candidates if path is not None]
    )
    nix_available = shutil.which("nix") is not None

    blockers: list[dict[str, str]] = []
    if not basecamp_dir.exists() and (qt_mcp is None or app_binary is None):
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
    if not nix_available and not (
        (qt_mcp / "test-framework" / "framework.mjs").exists()
        and app_binary is not None
        and os.access(app_binary, os.X_OK)
    ):
        blockers.append(
            {
                "id": "nix",
                "reason": "Nix is not available to build missing Basecamp/logos-qt-mcp artifacts hermetically",
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
    if design_system_qml is None:
        blockers.append(
            {
                "id": "logos_design_system",
                "reason": "Logos design-system QML imports are not available",
                "next": "clone logos-co/logos-design-system or set LOGOS_DESIGN_SYSTEM_ROOT",
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
        "nix": shutil.which("nix"),
        "lp0016_ui_test": str(test_file),
        "packaged_app": str(packaged_app) if packaged_app else None,
        "design_system_qml": str(design_system_qml) if design_system_qml else None,
        "blockers": blockers,
        "ready_commands": [
            "scripts/package_basecamp.sh /tmp/lp0016-basecamp",
            "LOGOS_QT_MCP=/tmp/logos-qt-mcp-inspect LOGOS_BASECAMP_APP=/tmp/logos-basecamp-build/LogosBasecamp QML2_IMPORT_PATH=/tmp/logos-design-system/src/qml QML_IMPORT_PATH=/tmp/logos-design-system/src/qml node app/basecamp-forum/ui-tests.mjs --ci /tmp/logos-basecamp-build/LogosBasecamp",
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
