#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AREA_ROOT="${REPO_ROOT}/verification/areas/python_interop"
REPORT="${REPO_ROOT}/target/verification/areas/python_interop/m9-demo.latest.json"

uv run --project "${AREA_ROOT}" --locked python \
  "${AREA_ROOT}/runner/run.py" \
  --callback-examples \
  --report "${REPORT}"

jq -r '.cases[].stdout' "${REPORT}" | grep -F 'sifr-python-interop:callback:'
