#!/usr/bin/env python3
"""Measure LP-0016 RISC0 membership proof performance locally."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUT = ROOT / "dist" / "submission" / "risc0_proof_performance.json"
GUEST_ELF = (
    ROOT
    / "zk"
    / "membership-guest"
    / "target"
    / "riscv32im-risc0-zkvm-elf"
    / "docker"
    / "lp0016-membership-guest.bin"
)
PROOF_SECONDS_RE = re.compile(r"lp0016_risc0_proof_seconds=([0-9]+(?:\.[0-9]+)?)")
IMAGE_WORDS_RE = re.compile(r"lp0016_risc0_image_id_words=([^\n]+)")


def command_text(cmd: list[str]) -> str:
    return " ".join(cmd)


def run_command(
    *,
    cmd: list[str],
    log_path: Path,
    env: dict[str, str] | None = None,
    timeout: int,
) -> dict:
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    started = time.time()
    try:
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            env=merged_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
            timeout=timeout,
        )
        output = proc.stdout
        returncode = proc.returncode
        timed_out = False
    except subprocess.TimeoutExpired as err:
        output = (err.stdout or "") + (err.stderr or "")
        returncode = 124
        timed_out = True

    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(output)
    return {
        "command": command_text(cmd),
        "returncode": returncode,
        "ok": returncode == 0,
        "timed_out": timed_out,
        "elapsed_seconds": round(time.time() - started, 3),
        "log": str(log_path.relative_to(ROOT)),
    }


def tail(path: Path, lines: int = 20) -> str:
    if not path.exists():
        return ""
    return "\n".join(path.read_text(errors="replace").splitlines()[-lines:])


def write_report(out_path: Path | None, report: dict) -> None:
    if out_path is None:
        return
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")


def build_report(args: argparse.Namespace) -> tuple[dict, int]:
    out_path: Path | None = args.out
    logs_dir = (out_path.parent if out_path else DEFAULT_OUT.parent) / "logs"
    blockers: list[dict] = []
    steps: list[dict] = []

    report: dict = {
        "target": "risc0_proof_performance",
        "threshold_seconds": args.threshold_seconds,
        "risc0_dev_mode": "0",
        "guest_elf": str(GUEST_ELF.relative_to(ROOT)),
        "status": "blocked",
        "blockers": blockers,
        "steps": steps,
        "ready_commands": [
            "cargo +stable risczero build --manifest-path zk/membership-guest/Cargo.toml --features risc0",
            (
                "LP0016_MEMBERSHIP_GUEST_ELF="
                f"{GUEST_ELF.relative_to(ROOT)} RISC0_DEV_MODE=0 "
                "cargo +stable test --release --manifest-path zk/membership-host/Cargo.toml "
                "--features risc0 proves_sample_membership_with_guest_elf -- --nocapture"
            ),
        ],
    }

    if not args.run_prover:
        blockers.append(
            {
                "id": "prover_not_run",
                "reason": "pass --run-prover to build the guest ELF and time a real RISC0 proof",
            }
        )
        write_report(out_path, report)
        return report, 0

    build_cmd = [
        "cargo",
        "+stable",
        "risczero",
        "build",
        "--manifest-path",
        "zk/membership-guest/Cargo.toml",
        "--features",
        "risc0",
    ]
    build = run_command(
        cmd=build_cmd,
        log_path=logs_dir / "risc0_membership_guest_build.log",
        timeout=args.timeout_seconds,
    )
    build["name"] = "guest_build"
    steps.append(build)
    if not build["ok"]:
        blockers.append(
            {
                "id": "guest_build",
                "reason": "RISC0 membership guest build failed",
                "log_tail": tail(ROOT / build["log"]),
            }
        )
        write_report(out_path, report)
        return report, 1 if args.fail_on_blocked else 0
    if not GUEST_ELF.exists():
        blockers.append(
            {
                "id": "guest_elf",
                "reason": f"guest build did not emit {GUEST_ELF.relative_to(ROOT)}",
            }
        )
        write_report(out_path, report)
        return report, 1 if args.fail_on_blocked else 0

    host_cmd = [
        "cargo",
        "+stable",
        "test",
        "--release",
        "--manifest-path",
        "zk/membership-host/Cargo.toml",
        "--features",
        "risc0",
        "proves_sample_membership_with_guest_elf",
        "--",
        "--nocapture",
    ]
    proof = run_command(
        cmd=host_cmd,
        log_path=logs_dir / "risc0_membership_host_proof.log",
        env={
            "LP0016_MEMBERSHIP_GUEST_ELF": str(GUEST_ELF),
            "RISC0_DEV_MODE": "0",
        },
        timeout=args.timeout_seconds,
    )
    proof["name"] = "host_prove"
    steps.append(proof)
    proof_output = (ROOT / proof["log"]).read_text(errors="replace") if Path(ROOT / proof["log"]).exists() else ""
    seconds_match = PROOF_SECONDS_RE.search(proof_output)
    image_match = IMAGE_WORDS_RE.search(proof_output)
    if seconds_match:
        report["proof_seconds"] = float(seconds_match.group(1))
    if image_match:
        report["image_id_words"] = image_match.group(1).strip()

    if not proof["ok"]:
        blockers.append(
            {
                "id": "host_prove",
                "reason": "RISC0 host proof test failed",
                "log_tail": tail(ROOT / proof["log"]),
            }
        )
    elif not seconds_match:
        blockers.append(
            {
                "id": "missing_measurement",
                "reason": "host proof test passed but did not print lp0016_risc0_proof_seconds",
            }
        )
    elif report["proof_seconds"] > args.threshold_seconds:
        blockers.append(
            {
                "id": "proof_time",
                "reason": (
                    f"RISC0 proof took {report['proof_seconds']:.3f}s, "
                    f"above the {args.threshold_seconds:.3f}s target"
                ),
            }
        )
    else:
        report["status"] = "ready"

    write_report(out_path, report)
    exit_code = 0
    if blockers and args.fail_on_blocked:
        exit_code = 1
    return report, exit_code


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-prover", action="store_true")
    parser.add_argument("--fail-on-blocked", action="store_true")
    parser.add_argument("--threshold-seconds", type=float, default=10.0)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    if args.out is None and args.run_prover:
        args.out = DEFAULT_OUT

    report, exit_code = build_report(args)
    sys.stdout.write(json.dumps(report, indent=2, sort_keys=True) + "\n")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
