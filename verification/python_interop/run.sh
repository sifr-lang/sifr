#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_BIN="${PYTHON_BIN:-python3}"

if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  echo "error: python3 is required for python interop verification" >&2
  exit 1
fi

exec "${PYTHON_BIN}" "${SCRIPT_DIR}/runner/run.py" "$@"
