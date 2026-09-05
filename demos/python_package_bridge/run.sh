#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AREA_ROOT="${REPO_ROOT}/verification/areas/python_interop"

uv run --project "${AREA_ROOT}" --locked python -c 'import biip'
MARKER_FILE="$(mktemp)"
trap 'rm -f "${MARKER_FILE}"' EXIT
SIFR_PACKAGE_BRIDGE_DEMO_MARKER_FILE="${MARKER_FILE}" cargo test \
  --manifest-path "${REPO_ROOT}/Cargo.toml" \
  -p sifr_driver \
  archived_biip_bridge_builds_and_runs_without_checkout_or_extraction \
  -- --ignored --nocapture

sed -n '1p' "${MARKER_FILE}"
