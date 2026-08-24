#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

python3 "${REPO_ROOT}/scripts/check_github_action_pins.py" --self-test
python3 "${REPO_ROOT}/scripts/check_github_action_pins.py"
