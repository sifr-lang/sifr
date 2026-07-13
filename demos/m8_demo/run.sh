#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AREA_ROOT="${REPO_ROOT}/verification/areas/python_interop"
REPORT="${REPO_ROOT}/target/verification/areas/python_interop/m8-demo.latest.json"

uv run --project "${AREA_ROOT}" --locked python \
  "${AREA_ROOT}/runner/run.py" \
  --async-context-examples \
  --report "${REPORT}"

jq -r '.cases[0].stdout' "${REPORT}" \
  | grep -F -m1 'sifr-python-interop:async-context:'
