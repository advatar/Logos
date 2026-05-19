#!/usr/bin/env python3
"""Report whether the Basecamp QML inspector click-through harness is runnable."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP_DIR = ROOT / "app" / "basecamp-forum"
DEFAULT_EVIDENCE = ROOT / "dist" / "submission" / "basecamp_inspector_evidence.json"


def cache_roots() -> list[Path]:
    roots: list[Path] = []
    explicit = os.environ.get("LOGOS_BASECAMP_CACHE")
    if explicit:
        roots.append(Path(explicit))
    xdg = os.environ.get("XDG_CACHE_HOME")
    if xdg:
        roots.append(Path(xdg) / "logos-basecamp")
    roots.extend(
        [
            ROOT / ".scaffold" / "cache" / "basecamp",
            Path.home() / ".cache" / "logos-basecamp",
            Path.home() / "Library" / "Caches" / "logos-basecamp",
        ]
    )
    # Legacy scratch paths from early local build passes. These remain last so
    # durable env/cache locations win when present.
    roots.extend([Path("/tmp/logos-basecamp-inspect"), Path("/tmp")])
    seen: set[Path] = set()
    unique: list[Path] = []
    for root in roots:
        expanded = root.expanduser()
        if expanded not in seen:
            seen.add(expanded)
            unique.append(expanded)
    return unique


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


def design_system_root_from_qml(qml: Path) -> Path:
    if qml.name == "qml" and qml.parent.name == "src":
        return qml.parent.parent
    return qml


def with_env_path(env: dict[str, str], name: str, path: Path) -> None:
    existing = env.get(name)
    env[name] = f"{path}{os.pathsep}{existing}" if existing else str(path)


def output_tail(output: str, limit: int = 4000) -> str:
    return output[-limit:] if len(output) > limit else output


def inspector_evidence_path() -> Path:
    return Path(
        os.environ.get("LOGOS_BASECAMP_INSPECTOR_EVIDENCE", str(DEFAULT_EVIDENCE))
    ).expanduser()


def load_inspector_evidence(
    evidence_path: Path,
    *,
    app_binary: Path | None,
    qt_mcp: Path,
    design_system_qml: Path | None,
) -> dict:
    if not evidence_path.exists():
        return {"status": "missing", "path": str(evidence_path)}

    try:
        evidence = json.loads(evidence_path.read_text())
    except json.JSONDecodeError as err:
        return {
            "status": "invalid",
            "path": str(evidence_path),
            "reason": f"JSON parse failed: {err}",
        }

    expected = {
        "basecamp_app": str(app_binary) if app_binary else None,
        "logos_qt_mcp": str(qt_mcp),
        "design_system_qml": str(design_system_qml) if design_system_qml else None,
    }
    mismatches = {
        key: {"expected": value, "actual": evidence.get(key)}
        for key, value in expected.items()
        if value is not None and evidence.get(key) != value
    }
    if evidence.get("status") == "passed" and not mismatches:
        return {
            "status": "accepted",
            "path": str(evidence_path),
            "generated_at_unix": evidence.get("generated_at_unix"),
        }
    return {
        "status": "stale",
        "path": str(evidence_path),
        "evidence_status": evidence.get("status"),
        "mismatches": mismatches,
    }


def run_inspector_click_through(
    *,
    app_binary: Path,
    qt_mcp: Path,
    design_system_qml: Path,
    test_file: Path,
    timeout_seconds: int,
    evidence_path: Path,
) -> dict:
    node = shutil.which("node")
    if node is None:
        return {
            "status": "failed",
            "reason": "Node.js is not available",
            "returncode": None,
        }

    env = os.environ.copy()
    env["LOGOS_QT_MCP"] = str(qt_mcp)
    env["LOGOS_BASECAMP_APP"] = str(app_binary)
    env["LOGOS_DESIGN_SYSTEM_ROOT"] = str(design_system_root_from_qml(design_system_qml))
    with_env_path(env, "QML_IMPORT_PATH", design_system_qml)
    with_env_path(env, "QML2_IMPORT_PATH", design_system_qml)

    command = [node, str(test_file), "--ci", str(app_binary)]
    started = time.time()
    try:
        proc = subprocess.run(
            command,
            cwd=ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=timeout_seconds,
            check=False,
        )
        output = proc.stdout
        returncode = proc.returncode
        timed_out = False
    except subprocess.TimeoutExpired as err:
        output = err.stdout or ""
        returncode = None
        timed_out = True

    elapsed = round(time.time() - started, 3)
    result = {
        "status": "failed" if timed_out or returncode != 0 else "passed",
        "command": " ".join(command),
        "returncode": returncode,
        "elapsed_seconds": elapsed,
        "timed_out": timed_out,
        "stdout_tail": output_tail(output),
    }
    if result["status"] == "passed":
        evidence_path.parent.mkdir(parents=True, exist_ok=True)
        evidence = {
            "status": "passed",
            "target": "basecamp_qml_inspector",
            "generated_at_unix": int(time.time()),
            "command": result["command"],
            "elapsed_seconds": elapsed,
            "basecamp_app": str(app_binary),
            "logos_qt_mcp": str(qt_mcp),
            "design_system_qml": str(design_system_qml),
            "stdout_tail": result["stdout_tail"],
        }
        evidence_path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
        result["evidence_path"] = str(evidence_path)
    return result


def build_report(*, verify_click_through: bool = False, timeout_seconds: int = 25) -> dict:
    basecamp_dir = Path(
        os.environ.get("LOGOS_BASECAMP_DIR", str(cache_roots()[0] / "logos-basecamp"))
    )
    roots = cache_roots()
    qt_mcp_value = os.environ.get("LOGOS_QT_MCP")
    qt_mcp_candidates = [
        Path(qt_mcp_value) if qt_mcp_value else None,
        basecamp_dir / "result-mcp",
    ]
    for root in roots:
        qt_mcp_candidates.extend(
            [
                root / "logos-qt-mcp",
                root / "qt-mcp",
                root / "result-mcp",
                root / "logos-basecamp" / "result-mcp",
            ]
        )
    qt_mcp = first_qt_mcp([path for path in qt_mcp_candidates if path is not None])
    if qt_mcp is None:
        qt_mcp = Path(qt_mcp_value) if qt_mcp_value else basecamp_dir / "result-mcp"

    app_binary_value = os.environ.get("LOGOS_BASECAMP_APP")
    app_binary_candidates = [
        Path(app_binary_value) if app_binary_value else None,
        basecamp_dir / "result" / "bin" / "LogosBasecamp",
        basecamp_dir / "result" / "LogosBasecamp.app" / "Contents" / "MacOS" / "LogosBasecamp",
        Path("/Applications/LogosBasecamp.app/Contents/MacOS/LogosBasecamp"),
        Path.home() / "Applications" / "LogosBasecamp.app" / "Contents" / "MacOS" / "LogosBasecamp",
    ]
    for root in roots:
        app_binary_candidates.extend(
            [
                root / "LogosBasecamp",
                root / "logos-basecamp" / "LogosBasecamp",
                root / "logos-basecamp-build" / "LogosBasecamp",
                root / "logos-basecamp-runtime" / "LogosBasecamp.app" / "Contents" / "MacOS" / "LogosBasecamp",
                root / "logos-basecamp-actions-app" / "LogosBasecamp.app" / "Contents" / "MacOS" / "LogosBasecamp",
                root / "logos-basecamp" / "result" / "bin" / "LogosBasecamp",
                root / "logos-basecamp" / "result" / "LogosBasecamp.app" / "Contents" / "MacOS" / "LogosBasecamp",
            ]
        )
    app_binary = first_executable([path for path in app_binary_candidates if path is not None])
    if app_binary is None and app_binary_value:
        app_binary = Path(app_binary_value)
    test_file = APP_DIR / "ui-tests.mjs"
    packaged_app = first_existing(
        [
            ROOT / "dist" / "basecamp" / "lp0016-anon-forum-demo.lgx",
            *[
                root / "basecamp-package" / "lp0016-anon-forum-demo.lgx"
                for root in roots
            ],
        ]
    )
    design_system_value = os.environ.get("LOGOS_DESIGN_SYSTEM_ROOT")
    design_system_candidates = [
        Path(design_system_value) if design_system_value else None,
        Path(design_system_value) / "src" / "qml" if design_system_value else None,
        Path(design_system_value) / "lib" if design_system_value else None,
    ]
    for root in roots:
        design_system_candidates.extend(
            [
                root / "logos-design-system" / "src" / "qml",
                root / "logos-design-system" / "lib",
                root / "design-system" / "src" / "qml",
                root / "design-system",
            ]
        )
    design_system_qml = first_design_system_qml(
        [path for path in design_system_candidates if path is not None]
    )
    nix_available = shutil.which("nix") is not None
    evidence_path = inspector_evidence_path()

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

    artifact_blockers = list(blockers)
    click_through: dict[str, object] = {
        "status": "not_run",
        "reason": "pass --run-click-through to launch Basecamp and verify the QML inspector harness",
    }
    inspector_evidence = load_inspector_evidence(
        evidence_path,
        app_binary=app_binary,
        qt_mcp=qt_mcp,
        design_system_qml=design_system_qml,
    )
    if not artifact_blockers:
        if verify_click_through:
            click_through = run_inspector_click_through(
                app_binary=app_binary,
                qt_mcp=qt_mcp,
                design_system_qml=design_system_qml,
                test_file=test_file,
                timeout_seconds=timeout_seconds,
                evidence_path=evidence_path,
            )
            if click_through["status"] == "passed":
                inspector_evidence = {
                    "status": "accepted",
                    "path": str(evidence_path),
                    "generated_at_unix": int(time.time()),
                }
            else:
                blockers.append(
                    {
                        "id": "basecamp_inspector",
                        "reason": "Basecamp artifacts are present, but the QML inspector click-through did not pass",
                        "next": "use an inspector-enabled Basecamp build, then rerun scripts/check_basecamp_inspector.py --run-click-through --pretty",
                    }
                )
        elif inspector_evidence["status"] == "accepted":
            click_through = {
                "status": "evidence_accepted",
                "path": inspector_evidence["path"],
            }
        else:
            blockers.append(
                {
                    "id": "basecamp_inspector",
                    "reason": "Basecamp artifacts are present, but no successful QML inspector click-through evidence matches this app/framework/design-system set",
                    "next": "run scripts/check_basecamp_inspector.py --run-click-through --pretty with an inspector-enabled Basecamp build",
                }
            )

    design_root = (
        design_system_root_from_qml(design_system_qml) if design_system_qml else "$HOME/.cache/logos-basecamp/logos-design-system"
    )
    app_for_command = app_binary if app_binary else Path("$HOME/.cache/logos-basecamp/LogosBasecamp")
    qt_for_command = qt_mcp if qt_mcp else Path("$HOME/.cache/logos-basecamp/logos-qt-mcp")
    qml_for_command = (
        design_system_qml if design_system_qml else Path("$HOME/.cache/logos-basecamp/logos-design-system/src/qml")
    )
    return {
        "status": "ready" if not blockers else "blocked",
        "target": "basecamp_qml_inspector",
        "artifact_status": "ready" if not artifact_blockers else "blocked",
        "cache_roots": [str(root) for root in roots],
        "basecamp_dir": str(basecamp_dir),
        "logos_qt_mcp": str(qt_mcp),
        "basecamp_app": str(app_binary) if app_binary else None,
        "nix": shutil.which("nix"),
        "lp0016_ui_test": str(test_file),
        "packaged_app": str(packaged_app) if packaged_app else None,
        "design_system_qml": str(design_system_qml) if design_system_qml else None,
        "inspector_evidence": inspector_evidence,
        "click_through": click_through,
        "blockers": blockers,
        "ready_commands": [
            "scripts/package_basecamp.sh dist/basecamp",
            "python3 scripts/check_basecamp_inspector.py --run-click-through --pretty",
            f"LOGOS_QT_MCP={qt_for_command} LOGOS_BASECAMP_APP={app_for_command} LOGOS_DESIGN_SYSTEM_ROOT={design_root} QML_IMPORT_PATH={qml_for_command} QML2_IMPORT_PATH={qml_for_command} node app/basecamp-forum/ui-tests.mjs --ci {app_for_command}",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pretty", action="store_true", help="pretty-print JSON")
    parser.add_argument(
        "--run-click-through",
        action="store_true",
        help="launch Basecamp and require the QML inspector click-through to pass",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=25,
        help="seconds to wait for the click-through run when --run-click-through is set",
    )
    args = parser.parse_args()
    report = build_report(
        verify_click_through=args.run_click_through,
        timeout_seconds=args.timeout,
    )
    print(json.dumps(report, indent=2 if args.pretty else None, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
