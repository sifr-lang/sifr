#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
cd "$REPO_ROOT"
uv run --project verification --locked python verification/areas/sysroot_release/native_candidate_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_capacity_tests.py
