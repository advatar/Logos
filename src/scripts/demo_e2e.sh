#!/usr/bin/env bash
set -euo pipefail

# Local dependency-free protocol demo. The production version of this script
# should start a LEZ standalone sequencer and run with RISC0_DEV_MODE=0.
python3 "$(dirname "$0")/demo_e2e.py"
