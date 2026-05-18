#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

UV_REMOTE="${UV_REMOTE:-https://github.com/astral-sh/uv.git}"
UV_REVISION="${UV_REVISION:-d19f1cd498202e04da70224573bbd5b79b94a726}"
UV_DIR="${UV_DIR:-${REPO_ROOT}/third_party/uv}"

usage() {
  cat <<'EOF'
Usage: scripts/prepare_uv_reference.sh [--status]

Clone or update the optional external uv reference checkout used for Phase 37
audits and adapter planning. Production uv crate pins belong in Cargo.toml and
Cargo.lock; this ignored checkout exists only so agents and reviewers can inspect
the exact upstream source without vendoring it.

Environment overrides:
  UV_REMOTE     uv git remote URL
  UV_REVISION   pinned uv commit SHA
  UV_DIR        destination checkout directory

Options:
  --status      Print checkout status after preparation.
  --help        Show this help.
EOF
}

PRINT_STATUS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --status)
      PRINT_STATUS=1
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

mkdir -p "$(dirname "${UV_DIR}")"

if [[ ! -d "${UV_DIR}/.git" ]]; then
  git clone "${UV_REMOTE}" "${UV_DIR}"
fi

git -C "${UV_DIR}" fetch --tags origin
git -C "${UV_DIR}" checkout --detach "${UV_REVISION}"

if [[ "${PRINT_STATUS}" == "1" ]]; then
  git -C "${UV_DIR}" describe --tags --always --dirty
  git -C "${UV_DIR}" rev-parse HEAD
fi
