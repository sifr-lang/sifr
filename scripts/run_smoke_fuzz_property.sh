#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run_smoke_fuzz_property.sh [--help]

Run deterministic local smoke property/fuzz checks.
EOF
}

if [[ $# -gt 0 ]]; then
  case "$1" in
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
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}/.."

echo "Running smoke property checks"
cargo test -p sifr --test e2e smoke_property -- --nocapture

echo "Running smoke fuzz checks"
cargo test -p sifr --test e2e smoke_fuzz -- --nocapture

echo "Running phase 29 property and fuzz-smoke verification suites"
python3 "${SCRIPT_DIR}/run_verification_hardening.py" --profile quick --suite property --suite fuzz-smoke
