#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'EOF'
Usage: scripts/clone_subrepos.sh [--remote] [--status]

Initialize and update Sifr submodules from .gitmodules.

Options:
  --remote  Update submodules to their configured remote branch tips.
  --status  Print recursive submodule status after updating.
  --help    Show this help.
EOF
}

UPDATE_REMOTE=0
PRINT_STATUS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote)
      UPDATE_REMOTE=1
      shift
      ;;
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

cd "${REPO_ROOT}"

git submodule sync --recursive

if [[ "${UPDATE_REMOTE}" == "1" ]]; then
  git submodule update --init --recursive --remote
else
  git submodule update --init --recursive
fi

if [[ "${PRINT_STATUS}" == "1" ]]; then
  git submodule status --recursive
fi
