#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run_validation_contract_matrix.sh [--suite <name>] [--help]

Run the declarative validation-contract harness.

Options:
  --suite <name>  Contract suite filter. Can be repeated.
  --help          Show this help.
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."
SUITE_FILTERS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --suite)
      SUITE_FILTERS+=("${2:-}")
      shift 2
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

SUITE_FILTER_CSV=""
if [[ "${#SUITE_FILTERS[@]}" -gt 0 ]]; then
  SUITE_FILTER_CSV="$(IFS=,; echo "${SUITE_FILTERS[*]}")"
fi

echo "Running validation contract matrix"
if [[ -n "${SUITE_FILTER_CSV}" ]]; then
  echo "  suites=${SUITE_FILTER_CSV}"
else
  echo "  suites=all"
fi

(
  cd "${REPO_ROOT}"
  SIFR_VALIDATION_CONTRACT_SUITE_FILTER="${SUITE_FILTER_CSV}" \
    cargo test -p sifr --test validation_contracts test_validation_contract_matrix -- --ignored --nocapture
)
