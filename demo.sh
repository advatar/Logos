#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/src"
exec python3 scripts/local_submission_gate.py "$@"
