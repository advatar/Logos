#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd "$script_dir/.." && pwd)"

python3 "$script_dir/demo_e2e.py"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

(
  cd "$repo_dir"
  LP0016_SIM_JSON_DIR="$tmp_dir" cargo run -p registry-sim --quiet
  cargo run -p slash-verifier --quiet -- verify \
    --registry "$tmp_dir/registry.json" \
    --bundle "$tmp_dir/bundle.json"
)

if [[ "${RISC0_DEV_MODE:-1}" == "0" ]]; then
  if ! cargo risczero --version >/dev/null 2>&1; then
    echo "RISC0_DEV_MODE=0 requested but cargo-risczero is not installed" >&2
    exit 1
  fi
  (
    cd "$repo_dir/zk/membership-host"
    cargo +stable test --features risc0
  )
fi
