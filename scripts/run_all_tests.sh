#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run_all_tests.sh [options]

Run all unit tests for the sifr package and then the e2e pass suite.

Any arguments are forwarded to scripts/run_e2e_pass.sh.
EOF
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}/.."

echo "Running unit tests (cargo test -p sifr)"
cargo test -p sifr

echo "Running e2e pass suite"
bash "${SCRIPT_DIR}/run_e2e_pass.sh" "$@"
