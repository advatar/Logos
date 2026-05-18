#!/usr/bin/env bash
set -euo pipefail

LOGOS_SCAFFOLD="${LOGOS_SCAFFOLD:-logos-scaffold}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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

if [[ ! -f "$ROOT/scaffold.toml" ]]; then
  cat <<'JSON'
{"status":"blocked","measurement":"lez_compute_units","reason":"scaffold.toml is missing","required_files":["scaffold.toml"]}
JSON
  exit 0
fi

LEZ_PATH="$(
  awk '
    /^\[repos\.lez\]/ { in_lez=1; next }
    /^\[/ { in_lez=0 }
    in_lez && $1 == "path" {
      value=$0
      sub(/^[^=]*= */, "", value)
      gsub(/^"|"$/, "", value)
      print value
      exit
    }
  ' "$ROOT/scaffold.toml"
)"

if [[ -z "$LEZ_PATH" ]]; then
  cat <<'JSON'
{"status":"blocked","measurement":"lez_compute_units","reason":"scaffold.toml does not define repos.lez.path","required_files":["scaffold.toml"]}
JSON
  exit 0
fi

if [[ "$LEZ_PATH" != /* ]]; then
  LEZ_PATH="$ROOT/$LEZ_PATH"
fi

SEQUENCER_BIN="$LEZ_PATH/target/release/sequencer_service"
WALLET_BIN="$LEZ_PATH/target/release/wallet"

if [[ ! -x "$SEQUENCER_BIN" || ! -x "$WALLET_BIN" ]]; then
  cat <<JSON
{"status":"blocked","measurement":"lez_compute_units","reason":"LEZ sequencer/wallet binaries are unavailable; run logos-scaffold setup after installing the logos-blockchain-circuits release","required_artifacts":["$SEQUENCER_BIN","$WALLET_BIN","registry/program_ids/devnet.txt","registry/program_ids/testnet.txt","docs/performance.md CU table"]}
JSON
  exit 0
fi

RUNTIME_CHECK="$(python3 "$ROOT/scripts/check_lez_runtime.py" 2>/dev/null || true)"
if [[ -n "$RUNTIME_CHECK" ]] && printf '%s\n' "$RUNTIME_CHECK" | grep -q '"status": "blocked"'; then
  printf '%s\n' "$RUNTIME_CHECK"
  exit 0
fi

if [[ ! -d "$ROOT/methods/guest/src/bin" ]]; then
  cat <<JSON
{"status":"blocked","measurement":"lez_compute_units","reason":"LEZ runtime binaries are available, but lp0016_registry has no deployable LEZ guest under methods/guest/src/bin yet","available_artifacts":["$SEQUENCER_BIN","$WALLET_BIN"],"required_artifacts":["methods/guest/src/bin/lp0016_registry.rs","registry/program_ids/devnet.txt","registry/program_ids/testnet.txt","docs/performance.md CU table"]}
JSON
  exit 0
fi

LOCALNET_STATUS="$("$LOGOS_SCAFFOLD" localnet status --json 2>/dev/null || true)"
if ! printf '%s\n' "$LOCALNET_STATUS" | grep -q '"ready": true'; then
  cat <<JSON
{"status":"blocked","measurement":"lez_compute_units","reason":"LEZ binaries are available, but the localnet is not ready","available_artifacts":["$SEQUENCER_BIN","$WALLET_BIN"],"required_commands":["logos-scaffold localnet start","logos-scaffold doctor","logos-scaffold deploy lp0016_registry --json"]}
JSON
  exit 0
fi

cat <<'JSON'
{"status":"blocked","measurement":"lez_compute_units","reason":"LEZ localnet is ready, but compute-unit capture still needs a deployable lp0016_registry guest and scaffold invoke/CU reporting path","required_commands":["logos-scaffold deploy lp0016_registry --json","logos-scaffold invoke register_member","logos-scaffold invoke slash_member"],"required_artifacts":["registry/program_ids/devnet.txt","registry/program_ids/testnet.txt","docs/performance.md CU table"]}
JSON
