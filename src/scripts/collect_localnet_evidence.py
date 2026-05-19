#!/usr/bin/env python3
"""Collect evaluator-facing localnet evidence without GitHub CI."""

from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

import check_lez_runtime


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUT = ROOT / "dist" / "submission" / "localnet_evidence.json"


def run(cmd: list[str], *, env: dict[str, str] | None = None, timeout: int = 120) -> dict:
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
        timeout=timeout,
        check=False,
    )
    return {
        "command": " ".join(cmd),
        "returncode": proc.returncode,
        "ok": proc.returncode == 0,
        "elapsed_seconds": round(time.time() - started, 3),
        "output": proc.stdout,
    }


def wait_for_port(port: int, timeout: float = 15.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if check_lez_runtime.tcp_listener_ready(port):
            return True
        time.sleep(0.25)
    return False


def start_sequencer(log_path: Path) -> tuple[subprocess.Popen | None, dict]:
    config = check_lez_runtime.load_scaffold()
    raw_lez_path = config.get("repos", {}).get("lez", {}).get("path")
    if not raw_lez_path:
        return None, {"ok": False, "reason": "scaffold.toml does not define repos.lez.path"}
    lez_path = Path(raw_lez_path)
    if not lez_path.is_absolute():
        lez_path = ROOT / lez_path

    binary = lez_path / "target" / "release" / "sequencer_service"
    config_path = lez_path / "sequencer" / "service" / "configs" / "debug" / "sequencer_config.json"
    if not binary.exists():
        return None, {"ok": False, "reason": f"sequencer binary missing: {binary}"}
    if not config_path.exists():
        return None, {"ok": False, "reason": f"sequencer config missing: {config_path}"}

    log_path.parent.mkdir(parents=True, exist_ok=True)
    log = log_path.open("w")
    proc = subprocess.Popen(
        [str(binary), str(config_path)],
        cwd=lez_path,
        stdout=log,
        stderr=subprocess.STDOUT,
        text=True,
    )
    log.close()
    return proc, {
        "ok": True,
        "pid": proc.pid,
        "command": f"{binary} {config_path}",
        "log": str(log_path.relative_to(ROOT)),
    }


def stop_process(proc: subprocess.Popen | None) -> None:
    if proc is None or proc.poll() is not None:
        return
    proc.send_signal(signal.SIGINT)
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)


def tail(path: Path, max_bytes: int = 6000) -> str:
    if not path.exists():
        return ""
    data = path.read_bytes()
    return data[-max_bytes:].decode(errors="replace")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--keep-running", action="store_true")
    args = parser.parse_args()

    args.out.parent.mkdir(parents=True, exist_ok=True)
    log_path = args.out.parent / "logs" / "localnet_sequencer.log"

    config = check_lez_runtime.load_scaffold()
    port = check_lez_runtime.configured_localnet_port(config) or 3040
    existing_listener = check_lez_runtime.tcp_listener_ready(port)
    proc: subprocess.Popen | None = None
    started = {"ok": True, "existing_listener": True} if existing_listener else {}
    if not existing_listener:
        proc, started = start_sequencer(log_path)
        if not started.get("ok"):
            evidence = {
                "status": "blocked",
                "target": "localnet_evidence",
                "started": started,
                "blockers": [{"id": "localnet_start", "reason": started.get("reason", "unknown")}],
            }
            args.out.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
            print(json.dumps(evidence, sort_keys=True))
            return 1

    try:
        ready = wait_for_port(port)
        runtime_report = json.loads(
            subprocess.check_output(
                ["python3", "scripts/check_lez_runtime.py"],
                cwd=ROOT,
                text=True,
            )
        )
        guest_bin = ROOT / "methods" / "target" / "riscv32im-risc0-zkvm-elf" / "docker" / "lp0016_registry.bin"
        deploy = run(
            [
                check_lez_runtime.command_path("logos-scaffold") or "logos-scaffold",
                "deploy",
                "lp0016_registry",
                "--program-path",
                str(guest_bin),
                "--json",
            ],
            timeout=60,
        )
        demo = run(["scripts/demo_e2e.sh"], env={"RISC0_DEV_MODE": "0"}, timeout=180)
        cu = run(["scripts/measure_cu.sh"], timeout=60)

        deploy_submitted = '"status":"submitted"' in deploy["output"] or '"status": "submitted"' in deploy["output"]
        evidence = {
            "status": "ready" if ready and deploy_submitted and demo["ok"] else "blocked",
            "target": "localnet_evidence",
            "localnet_port": port,
            "started": started,
            "runtime": runtime_report,
            "guest_binary": str(guest_bin.relative_to(ROOT)),
            "program_image_id": (ROOT / "registry" / "program_ids" / "localnet.txt").read_text().strip(),
            "deploy": deploy,
            "risc0_dev_mode_zero_demo": demo,
            "cu_diagnostic": cu,
            "sequencer_log_tail": tail(log_path),
            "blockers": [],
        }
        if evidence["status"] != "ready":
            if not ready:
                evidence["blockers"].append({"id": "localnet_ready", "reason": "port did not become ready"})
            if not deploy_submitted:
                evidence["blockers"].append({"id": "deploy", "reason": "deploy did not return submitted"})
            if not demo["ok"]:
                evidence["blockers"].append({"id": "risc0_dev_mode_zero_demo", "reason": "demo command failed"})

        args.out.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
        print(json.dumps(evidence, sort_keys=True))
        return 0 if evidence["status"] == "ready" else 1
    finally:
        if proc is not None and not args.keep_running:
            stop_process(proc)


if __name__ == "__main__":
    raise SystemExit(main())
