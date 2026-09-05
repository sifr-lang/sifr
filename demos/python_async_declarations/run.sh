#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AREA_ROOT="${REPO_ROOT}/verification/areas/python_interop"
REPORT="${REPO_ROOT}/target/verification/areas/python_interop/python-async-declarations.latest.json"

uv run --project "${AREA_ROOT}" --locked python \
  "${AREA_ROOT}/runner/run.py" \
  --async-declaration-examples \
  --report "${REPORT}"

jq -r '.cases[0].stdout' "${REPORT}" | sed -n '1p'
