#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."

for script in "${REPO_ROOT}"/verification/distribution/*.sh; do
  case "${script}" in
    */common.sh)
      continue
      ;;
  esac
  echo "Running ${script#${REPO_ROOT}/}"
  "${script}"
done
