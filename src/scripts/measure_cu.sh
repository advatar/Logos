#!/usr/bin/env bash
set -euo pipefail

LOGOS_SCAFFOLD="${LOGOS_SCAFFOLD:-logos-scaffold}"
if ! command -v "$LOGOS_SCAFFOLD" >/dev/null 2>&1; then
  if [[ -x "$HOME/.cargo/bin/logos-scaffold" ]]; then
    LOGOS_SCAFFOLD="$HOME/.cargo/bin/logos-scaffold"
  fi
fi

if ! command -v "$LOGOS_SCAFFOLD" >/dev/null 2>&1; then
  cat <<'JSON'
{"status":"blocked","measurement":"lez_compute_units","reason":"logos-scaffold is not installed","required_commands":["logos-scaffold localnet start","logos-scaffold deploy lp0016_registry","logos-scaffold invoke register_member","logos-scaffold invoke slash_member"]}
JSON
  exit 0
fi

cat <<'JSON'
{"status":"blocked","measurement":"lez_compute_units","reason":"LEZ sequencer/wallet binaries are unavailable; run logos-scaffold setup after installing the logos-blockchain-circuits release","required_artifacts":[".scaffold/cache/repos/lez/<pin>/target/release/sequencer_service",".scaffold/cache/repos/lez/<pin>/target/release/wallet","registry/program_ids/devnet.txt","registry/program_ids/testnet.txt","docs/performance.md CU table"]}
JSON
