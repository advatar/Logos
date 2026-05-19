#!/usr/bin/env python3
"""Run the local LP-0016 submission gate and write reproducible evidence."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUT = ROOT / "dist" / "submission"


def command_text(cmd: list[str]) -> str:
    return " ".join(cmd)


def run_step(
    *,
    name: str,
    cmd: list[str],
    out_dir: Path,
    required: bool = True,
    env: dict[str, str] | None = None,
    parse_json: bool = False,
) -> dict:
    started = time.time()
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)

    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        env=merged_env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    elapsed = time.time() - started
    log_path = out_dir / f"{name}.log"
    log_path.write_text(proc.stdout)

    parsed = None
    if parse_json and proc.stdout.strip():
        try:
            parsed = json.loads(proc.stdout)
        except json.JSONDecodeError:
            parsed = {"parse_error": "stdout was not valid JSON"}

    return {
        "name": name,
        "command": command_text(cmd),
        "required": required,
        "returncode": proc.returncode,
        "ok": proc.returncode == 0,
        "elapsed_seconds": round(elapsed, 3),
        "log": str(log_path.relative_to(ROOT)),
        "json": parsed,
    }


def build_steps(skip_slow: bool) -> list[dict]:
    python_suite = [
        "python3",
        "-m",
        "unittest",
        "scripts/test_protocol.py",
        "scripts/test_basecamp_package.py",
        "scripts/test_runtime_checks.py",
        "scripts/test_success_criteria.py",
        "scripts/test_phase_closure.py",
    ]
    steps: list[dict] = [
        {"name": "python_demo", "cmd": ["python3", "scripts/demo_e2e.py"]},
        {"name": "python_success_suite", "cmd": python_suite},
        {"name": "rust_build", "cmd": ["cargo", "build", "--workspace"]},
        {"name": "rust_tests", "cmd": ["cargo", "test", "--workspace"]},
        {"name": "shell_demo", "cmd": ["scripts/demo_e2e.sh"]},
        {
            "name": "risc0_dev_mode_zero_demo",
            "cmd": ["scripts/demo_e2e.sh"],
            "env": {"RISC0_DEV_MODE": "0"},
        },
        {
            "name": "risc0_host_feature",
            "cmd": [
                "rustup",
                "run",
                "stable",
                "cargo",
                "check",
                "--manifest-path",
                "zk/membership-host/Cargo.toml",
                "--features",
                "risc0",
            ],
        },
        {
            "name": "risc0_proof_performance",
            "cmd": [
                "python3",
                "scripts/check_risc0_proof_performance.py",
                "--run-prover",
                "--fail-on-blocked",
            ],
            "parse_json": True,
        },
        {
            "name": "lez_guest_check",
            "cmd": ["cargo", "+stable", "check", "--manifest-path", "methods/guest/Cargo.toml"],
        },
        {
            "name": "lez_guest_build",
            "cmd": ["bash", "-lc", "cd methods && cargo risczero build --manifest-path guest/Cargo.toml"],
        },
        {
            "name": "localnet_evidence",
            "cmd": ["python3", "scripts/collect_localnet_evidence.py"],
            "parse_json": True,
        },
        {"name": "lean_build", "cmd": ["lake", "build"], "cwd": ROOT / "lean"},
        {
            "name": "basecamp_package",
            "cmd": ["scripts/package_basecamp.sh", "dist/basecamp"],
        },
        {
            "name": "lez_runtime_diagnostic",
            "cmd": ["python3", "scripts/check_lez_runtime.py"],
            "required": False,
            "parse_json": True,
        },
        {
            "name": "basecamp_runtime_diagnostic",
            "cmd": ["python3", "scripts/check_basecamp_inspector.py"],
            "required": False,
            "parse_json": True,
        },
        {
            "name": "cu_diagnostic",
            "cmd": ["scripts/measure_cu.sh"],
            "required": False,
            "parse_json": True,
        },
        {
            "name": "live_network_deploy_diagnostic",
            "cmd": ["python3", "scripts/check_live_network_deploy.py"],
            "required": False,
            "parse_json": True,
        },
    ]
    if skip_slow:
        skip_names = {
            "rust_build",
            "rust_tests",
            "risc0_dev_mode_zero_demo",
            "risc0_host_feature",
            "risc0_proof_performance",
            "lez_guest_build",
            "lean_build",
        }
        steps = [step for step in steps if step["name"] not in skip_names]
    return steps


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--skip-slow", action="store_true")
    parser.add_argument(
        "--strict-runtime",
        action="store_true",
        help="fail if LEZ/Basecamp/CU diagnostics report blocked",
    )
    args = parser.parse_args()

    out_dir = args.out
    out_dir.mkdir(parents=True, exist_ok=True)
    steps_dir = out_dir / "logs"
    steps_dir.mkdir(parents=True, exist_ok=True)

    results: list[dict] = []
    for spec in build_steps(args.skip_slow):
        cwd = spec.pop("cwd", ROOT)
        if cwd != ROOT:
            cmd = ["bash", "-lc", f"cd {cwd} && {command_text(spec['cmd'])}"]
        else:
            cmd = spec["cmd"]
        result = run_step(
            name=spec["name"],
            cmd=cmd,
            out_dir=steps_dir,
            required=spec.get("required", True),
            env=spec.get("env"),
            parse_json=spec.get("parse_json", False),
        )
        results.append(result)
        status = "ok" if result["ok"] else "failed"
        print(f"{result['name']}: {status} ({result['elapsed_seconds']}s)")

    required_ok = all(step["ok"] for step in results if step["required"])
    runtime_ready = True
    if args.strict_runtime:
        for step in results:
            parsed = step.get("json")
            if isinstance(parsed, dict) and parsed.get("status") == "blocked":
                runtime_ready = False

    evidence = {
        "generated_at_unix": int(time.time()),
        "root": str(ROOT),
        "policy": "local integration evidence; GitHub Actions is intentionally not required",
        "strict_runtime": args.strict_runtime,
        "required_ok": required_ok,
        "runtime_ready": runtime_ready,
        "ok": required_ok and runtime_ready,
        "steps": results,
    }
    evidence_path = out_dir / "evidence.json"
    evidence_path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
    print(f"evidence: {evidence_path}")

    return 0 if evidence["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
