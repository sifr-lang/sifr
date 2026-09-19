#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
python3 "${REPO_ROOT}/verification/areas/distribution_release/native_package_checks.py"
