#!/usr/bin/env python3
"""Report whether the local tree can deploy and measure the LP-0016 LEZ guest."""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_EXPECTED_CIRCUITS_VERSION = "v0.4.2"


def command_path(name: str) -> str | None:
    found = shutil.which(name)
    if found:
        return found
    cargo_bin = Path.home() / ".cargo" / "bin" / name
    if cargo_bin.exists():
        return str(cargo_bin)
    return None


def load_scaffold() -> dict:
    path = ROOT / "scaffold.toml"
    if not path.exists():
        return {}
    config: dict = {}
    section: list[str] = []
    for raw_line in path.read_text().splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].split(".")
            cursor = config
            for part in section:
                cursor = cursor.setdefault(part, {})
            continue
        if "=" not in line:
            continue
        key, value = [part.strip() for part in line.split("=", 1)]
        value = value.strip('"')
        cursor = config
        for part in section:
            cursor = cursor.setdefault(part, {})
        cursor[key] = value
    return config


def expected_circuits_version() -> str:
    env_value = os.environ.get("LOGOS_BLOCKCHAIN_CIRCUITS_EXPECTED_VERSION")
    if env_value:
        return env_value
    for source in sorted(
        (Path.home() / ".cargo" / "git" / "checkouts").glob(
            "logos-blockchain-*/**/zk/circuits/utils/src/lib.rs"
        ),
        reverse=True,
    ):
        text = source.read_text(errors="ignore")
        match = re.search(r'EXPECTED_CIRCUITS_VERSION:\s*&str\s*=\s*"([^"]+)"', text)
        if match:
            return match.group(1)
    return DEFAULT_EXPECTED_CIRCUITS_VERSION


def circuits_version(circuits_dir: Path) -> str | None:
    version_file = circuits_dir / "VERSION"
    if not version_file.exists():
        return None
    return version_file.read_text().strip()


def localnet_ready(logos_scaffold: str | None) -> bool:
    if not logos_scaffold:
        return False
    try:
        proc = subprocess.run(
            [logos_scaffold, "localnet", "status", "--json"],
            cwd=ROOT,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired):
        return False
    return '"ready": true' in proc.stdout


def build_report() -> dict:
    config = load_scaffold()
    framework_kind = config.get("framework", {}).get("kind")
    lez_path_value = config.get("repos", {}).get("lez", {}).get("path")
    lez_path = Path(lez_path_value) if lez_path_value else None
    if lez_path is not None and not lez_path.is_absolute():
        lez_path = ROOT / lez_path

    circuits_dir = Path(
        os.environ.get("LOGOS_BLOCKCHAIN_CIRCUITS", Path.home() / ".logos-blockchain-circuits")
    )
    expected_version = expected_circuits_version()
    actual_version = circuits_version(circuits_dir)

    logos_scaffold = command_path("logos-scaffold")
    cargo_risczero = command_path("cargo-risczero")
    guest = ROOT / "methods" / "guest" / "src" / "bin" / "lp0016_registry.rs"
    sequencer = lez_path / "target" / "release" / "sequencer_service" if lez_path else None
    wallet = lez_path / "target" / "release" / "wallet" if lez_path else None

    blockers: list[dict[str, str]] = []
    if not logos_scaffold:
        blockers.append(
            {
                "id": "logos_scaffold",
                "reason": "logos-scaffold is not installed or not on PATH",
                "next": "cargo install logos-scaffold or set LOGOS_SCAFFOLD to the binary path",
            }
        )
    if not cargo_risczero:
        blockers.append(
            {
                "id": "cargo_risczero",
                "reason": "cargo-risczero is not installed",
                "next": "rzup install && cargo risczero --version",
            }
        )
    if framework_kind != "lez-framework":
        blockers.append(
            {
                "id": "framework_kind",
                "reason": f"scaffold.toml framework.kind is {framework_kind!r}, not 'lez-framework'",
                "next": "migrate once methods/guest builds without breaking the Rust 1.82 workspace",
            }
        )
    if not guest.exists():
        blockers.append(
            {
                "id": "guest_source",
                "reason": "deployable LEZ guest source is missing",
                "next": "add methods/guest/src/bin/lp0016_registry.rs from the lez-framework program",
            }
        )
    if actual_version != expected_version:
        blockers.append(
            {
                "id": "circuits_version",
                "reason": f"logos-blockchain-circuits is {actual_version!r}, expected {expected_version!r}",
                "next": "install the matching circuits release or set LOGOS_BLOCKCHAIN_CIRCUITS",
            }
        )
    if sequencer is None or not os.access(sequencer, os.X_OK):
        blockers.append(
            {
                "id": "sequencer_binary",
                "reason": "LEZ sequencer_service binary is missing",
                "next": "run logos-scaffold setup after circuits match",
            }
        )
    if wallet is None or not os.access(wallet, os.X_OK):
        blockers.append(
            {
                "id": "wallet_binary",
                "reason": "LEZ wallet binary is missing",
                "next": "run logos-scaffold setup after circuits match",
            }
        )

    ready = not blockers and localnet_ready(logos_scaffold)
    if not blockers and not ready:
        blockers.append(
            {
                "id": "localnet",
                "reason": "LEZ prerequisites are present but localnet is not ready",
                "next": "logos-scaffold localnet start && logos-scaffold doctor",
            }
        )

    return {
        "status": "ready" if ready else "blocked",
        "target": "lez_runtime",
        "root": str(ROOT),
        "framework_kind": framework_kind,
        "logos_scaffold": logos_scaffold,
        "cargo_risczero": cargo_risczero,
        "circuits_dir": str(circuits_dir),
        "expected_circuits_version": expected_version,
        "actual_circuits_version": actual_version,
        "guest_source": str(guest),
        "lez_path": str(lez_path) if lez_path else None,
        "localnet_ready": ready,
        "blockers": blockers,
        "ready_commands": [
            "logos-scaffold localnet start",
            "logos-scaffold deploy lp0016_registry --json",
            "logos-scaffold invoke register_member --json",
            "logos-scaffold invoke slash_member --json",
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
