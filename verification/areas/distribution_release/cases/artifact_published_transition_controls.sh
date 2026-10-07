#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
cd "$REPO_ROOT"
uv run --project verification --locked python verification/areas/sysroot_release/published_transition_tests.py

uv run --project verification --locked python verification/areas/sysroot_release/legacy_transition_inventory_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_dependency_preparation_tests.py
