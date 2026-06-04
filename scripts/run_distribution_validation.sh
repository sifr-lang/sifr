#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."

now_ms() {
  python3 -c 'import time; print(time.monotonic_ns() // 1_000_000)'
}

for script in "${REPO_ROOT}"/verification/distribution/*.sh; do
  case "${script}" in
    */common.sh)
      continue
      ;;
  esac
  echo "Running ${script#${REPO_ROOT}/}"
  start_ms="$(now_ms)"
  set +e
  "${script}"
  status=$?
  set -e
  end_ms="$(now_ms)"
  elapsed_ms=$((end_ms - start_ms))
  case_name="${script#${REPO_ROOT}/verification/distribution/}"
  case_name="${case_name%.sh}"
  if [[ "${status}" -eq 0 ]]; then
    echo "[sifr-case-timing] bucket=distribution case=${case_name} elapsed_ms=${elapsed_ms} status=pass"
  else
    echo "[sifr-case-timing] bucket=distribution case=${case_name} elapsed_ms=${elapsed_ms} status=fail"
    exit "${status}"
  fi
done
