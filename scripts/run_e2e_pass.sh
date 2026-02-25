#!/usr/bin/env bash

set -euo pipefail

MODE="${SIFR_E2E_RUNNER_MODE:-new}"
SIFR_JOBS="${SIFR_E2E_SIFR_JOBS:-6}"
RUST_JOBS="${SIFR_E2E_RUST_JOBS:-4}"
RUN_JOBS="${SIFR_E2E_RUN_JOBS:-4}"
CARGO_BUILD_JOBS="${SIFR_E2E_CARGO_BUILD_JOBS:-1}"
DISABLE_CACHE="${SIFR_E2E_DISABLE_CACHE:-0}"

usage() {
  cat <<'EOF'
Usage: scripts/run_e2e_pass.sh [options]

Options:
  --mode <legacy|new|compare>  Runner mode (default: new)
  --sifr-jobs <n>              Parallel Sifr compile workers
  --rust-jobs <n>              Parallel group build workers
  --run-jobs <n>               Parallel group run workers
  --cargo-build-jobs <n>       Cargo jobs per generated group build
  --no-cache                   Disable the e2e cache for this run
  --help                       Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      MODE="${2:-}"
      shift 2
      ;;
    --sifr-jobs)
      SIFR_JOBS="${2:-}"
      shift 2
      ;;
    --rust-jobs)
      RUST_JOBS="${2:-}"
      shift 2
      ;;
    --run-jobs)
      RUN_JOBS="${2:-}"
      shift 2
      ;;
    --cargo-build-jobs)
      CARGO_BUILD_JOBS="${2:-}"
      shift 2
      ;;
    --no-cache)
      DISABLE_CACHE=1
      shift
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

echo "Running e2e pass suite"
echo "  mode=${MODE}"
echo "  sifr_jobs=${SIFR_JOBS}"
echo "  rust_jobs=${RUST_JOBS}"
echo "  run_jobs=${RUN_JOBS}"
echo "  cargo_build_jobs=${CARGO_BUILD_JOBS}"
echo "  disable_cache=${DISABLE_CACHE}"

SIFR_E2E_RUNNER_MODE="${MODE}" \
SIFR_E2E_SIFR_JOBS="${SIFR_JOBS}" \
SIFR_E2E_RUST_JOBS="${RUST_JOBS}" \
SIFR_E2E_RUN_JOBS="${RUN_JOBS}" \
SIFR_E2E_CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS}" \
SIFR_E2E_DISABLE_CACHE="${DISABLE_CACHE}" \
cargo test -p sifr --test e2e test_e2e_pass -- --nocapture
