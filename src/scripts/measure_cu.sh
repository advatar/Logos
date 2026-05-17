#!/usr/bin/env bash
set -euo pipefail
cat <<'MSG'
CU measurement placeholder.

Production steps:
  1. logos-scaffold localnet start
  2. logos-scaffold deploy lp0016_registry
  3. run register_member and capture sequencer CU output
  4. run slash_member with K certificates and capture CU output
  5. write results to docs/performance.md
MSG
