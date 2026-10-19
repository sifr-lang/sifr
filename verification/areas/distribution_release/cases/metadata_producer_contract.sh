#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
python3 "${REPO_ROOT}/scripts/distribution/metadata_artifact_checks.py"
echo "metadata producer/packager contract: PASS"
