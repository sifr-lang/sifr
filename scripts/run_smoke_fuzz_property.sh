#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run_smoke_fuzz_property.sh [--help]

Run always-on smoke property and fuzz-style parser-extractor checks.
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
