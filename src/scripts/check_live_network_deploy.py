#!/usr/bin/env python3
"""Report live LEZ devnet/testnet deployment readiness."""

from __future__ import annotations

import json
import os
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def program_id(path: Path) -> str | None:
    if not path.exists():
        return None
    value = path.read_text().strip()
    return value or None


def main() -> int:
    devnet_url = os.environ.get("LOGOS_LEZ_DEVNET_URL")
    testnet_url = os.environ.get("LOGOS_LEZ_TESTNET_URL")
    devnet_id = program_id(ROOT / "registry" / "program_ids" / "devnet.txt")
    testnet_id = program_id(ROOT / "registry" / "program_ids" / "testnet.txt")

    blockers: list[dict[str, str]] = []
    if not devnet_url:
        blockers.append(
            {
                "id": "devnet_endpoint",
                "reason": "LOGOS_LEZ_DEVNET_URL is not set",
                "next": "set LOGOS_LEZ_DEVNET_URL to the live LEZ devnet sequencer RPC URL",
            }
        )
    if not testnet_url:
        blockers.append(
            {
                "id": "testnet_endpoint",
                "reason": "LOGOS_LEZ_TESTNET_URL is not set",
                "next": "set LOGOS_LEZ_TESTNET_URL to the live LEZ testnet sequencer RPC URL",
            }
        )
    if not devnet_id:
        blockers.append(
            {
                "id": "devnet_program_id",
                "reason": "registry/program_ids/devnet.txt is missing or empty",
                "next": "deploy lp0016_registry to devnet and record the returned program/image ID",
            }
        )
    if not testnet_id:
        blockers.append(
            {
                "id": "testnet_program_id",
                "reason": "registry/program_ids/testnet.txt is missing or empty",
                "next": "deploy lp0016_registry to testnet and record the returned program/image ID",
            }
        )

    report = {
        "status": "ready" if not blockers else "blocked",
        "target": "live_lez_deployment",
        "devnet_url_set": bool(devnet_url),
        "testnet_url_set": bool(testnet_url),
        "devnet_program_id": devnet_id,
        "testnet_program_id": testnet_id,
        "localnet_program_id": program_id(ROOT / "registry" / "program_ids" / "localnet.txt"),
        "blockers": blockers,
        "ready_commands": [
            "export LOGOS_LEZ_DEVNET_URL=<devnet sequencer rpc>",
            "export LOGOS_LEZ_TESTNET_URL=<testnet sequencer rpc>",
            "set wallet sequencer_addr to the target network",
            "logos-scaffold deploy lp0016_registry --program-path methods/target/riscv32im-risc0-zkvm-elf/docker/lp0016_registry.bin --json",
            "write returned IDs to registry/program_ids/devnet.txt and registry/program_ids/testnet.txt",
        ],
    }
    print(json.dumps(report, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
