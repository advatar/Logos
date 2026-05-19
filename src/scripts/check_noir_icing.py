#!/usr/bin/env python3
"""Report optional Noir proof-circuit readiness."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "noir" / "post_binding"
INSTALL_DOC = "https://noir-lang.org/docs/getting_started/noir_installation"
TEST_DOC = "https://noir-lang.org/docs/tooling/tests"


def find_nargo() -> str | None:
    path = shutil.which("nargo")
    if path:
        return path
    noirup_path = Path.home() / ".nargo" / "bin" / "nargo"
    if noirup_path.exists():
        return str(noirup_path)
    return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pretty", action="store_true")
    args = parser.parse_args()

    nargo = find_nargo()
    blockers: list[dict[str, str]] = []
    test_result: dict[str, object] | None = None

    if not PACKAGE.exists():
        blockers.append(
            {
                "id": "noir_package",
                "reason": f"Noir package not found at {PACKAGE.relative_to(ROOT)}",
                "next": "restore noir/post_binding with Nargo.toml and src/main.nr",
            }
        )
    if not nargo:
        blockers.append(
            {
                "id": "nargo",
                "reason": "nargo is not installed or not on PATH",
                "next": "install Noir with: curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash && noirup",
            }
        )

    if nargo and PACKAGE.exists():
        proc = subprocess.run(
            [nargo, "test"],
            cwd=PACKAGE,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        test_result = {
            "command": "nargo test",
            "returncode": proc.returncode,
            "ok": proc.returncode == 0,
            "output": proc.stdout[-4000:],
        }
        if proc.returncode != 0:
            blockers.append(
                {
                    "id": "nargo_test",
                    "reason": "nargo test failed for noir/post_binding",
                    "next": "fix Noir circuit or update the package to match installed Nargo",
                }
            )

    report = {
        "status": "ready" if not blockers else "blocked",
        "target": "noir_icing",
        "package": str(PACKAGE.relative_to(ROOT)),
        "nargo": nargo,
        "blockers": blockers,
        "test": test_result,
        "docs": {
            "installation": INSTALL_DOC,
            "tests": TEST_DOC,
        },
        "ready_commands": [
            "export PATH=\"$HOME/.nargo/bin:$PATH\"",
            "cd noir/post_binding && nargo test",
            "python3 scripts/check_noir_icing.py --pretty",
        ],
        "note": (
            "Noir is an optional ACIR proof-circuit artifact for the anonymous-post "
            "binding shape. RISC0 remains the production submission proof path."
        ),
    }
    print(json.dumps(report, indent=2 if args.pretty else None, sort_keys=True))
    return 0 if test_result is None or test_result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
