#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
cd "$REPO_ROOT"
uv run --project verification --locked python verification/areas/sysroot_release/native_candidate_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_capacity_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_preparation_storage_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_capacity_diagnostics_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_runtime_observer_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_runtime_storage_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_runtime_diagnostic_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_recovery_tests.py
uv run --project verification --locked python verification/areas/sysroot_release/native_source_dependencies_tests.py
