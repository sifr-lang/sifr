#!/usr/bin/env bash

set -euo pipefail

PROFILE="${SIFR_E2E_PROFILE:-full}"
MODE_OVERRIDE=""
SIFR_JOBS_OVERRIDE=""
RUST_JOBS_OVERRIDE=""
RUN_JOBS_OVERRIDE=""
CARGO_BUILD_JOBS_OVERRIDE=""
DISABLE_CACHE_OVERRIDE=""
CACHE_DIR_OVERRIDE=""

usage() {
  cat <<'EOF'
Usage: scripts/run_e2e_pass.sh [options]

Profiles:
  quick   Fast local signal; bounded parallel workers; cache enabled.
  full    Authoritative local gate; balanced parallel workers; cache enabled.
  stress  High-contention check; compare mode + cache disabled.

Options:
  --profile <quick|full|stress> Local execution profile (default: full)
  --mode <legacy|new|compare>  Runner mode (default: new)
  --sifr-jobs <n>              Parallel Sifr compile workers
  --rust-jobs <n>              Parallel group build workers
  --run-jobs <n>               Parallel group run workers
  --cargo-build-jobs <n>       Cargo jobs per generated group build
  --cache-dir <path>           e2e cache directory (default: target/sifr_e2e_cache/<profile>)
  --no-cache                   Disable the e2e cache for this run
  --help                       Show this help
EOF
}

set_profile_defaults() {
  local profile="$1"
  case "${profile}" in
    quick)
      MODE="new"
      SIFR_JOBS="2"
      RUST_JOBS="2"
      RUN_JOBS="2"
      CARGO_BUILD_JOBS="1"
      DISABLE_CACHE="0"
      ;;
    full)
      MODE="new"
      SIFR_JOBS="6"
      RUST_JOBS="4"
      RUN_JOBS="4"
      CARGO_BUILD_JOBS="1"
      DISABLE_CACHE="0"
      ;;
    stress)
      MODE="compare"
      SIFR_JOBS="8"
      RUST_JOBS="6"
      RUN_JOBS="6"
      CARGO_BUILD_JOBS="1"
      DISABLE_CACHE="1"
      ;;
    *)
      echo "unsupported profile: ${profile}" >&2
      usage >&2
      exit 2
      ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --mode)
      MODE_OVERRIDE="${2:-}"
      shift 2
      ;;
    --sifr-jobs)
      SIFR_JOBS_OVERRIDE="${2:-}"
      shift 2
      ;;
    --rust-jobs)
      RUST_JOBS_OVERRIDE="${2:-}"
      shift 2
      ;;
    --run-jobs)
      RUN_JOBS_OVERRIDE="${2:-}"
      shift 2
      ;;
    --cargo-build-jobs)
      CARGO_BUILD_JOBS_OVERRIDE="${2:-}"
      shift 2
      ;;
    --cache-dir)
      CACHE_DIR_OVERRIDE="${2:-}"
      shift 2
      ;;
    --no-cache)
      DISABLE_CACHE_OVERRIDE="1"
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

set_profile_defaults "${PROFILE}"

MODE="${SIFR_E2E_RUNNER_MODE:-${MODE}}"
SIFR_JOBS="${SIFR_E2E_SIFR_JOBS:-${SIFR_JOBS}}"
RUST_JOBS="${SIFR_E2E_RUST_JOBS:-${RUST_JOBS}}"
RUN_JOBS="${SIFR_E2E_RUN_JOBS:-${RUN_JOBS}}"
CARGO_BUILD_JOBS="${SIFR_E2E_CARGO_BUILD_JOBS:-${CARGO_BUILD_JOBS}}"
DISABLE_CACHE="${SIFR_E2E_DISABLE_CACHE:-${DISABLE_CACHE}}"
CACHE_DIR="${SIFR_E2E_CACHE_DIR:-target/sifr_e2e_cache/${PROFILE}}"

if [[ -n "${MODE_OVERRIDE}" ]]; then
  MODE="${MODE_OVERRIDE}"
fi
if [[ -n "${SIFR_JOBS_OVERRIDE}" ]]; then
  SIFR_JOBS="${SIFR_JOBS_OVERRIDE}"
fi
if [[ -n "${RUST_JOBS_OVERRIDE}" ]]; then
  RUST_JOBS="${RUST_JOBS_OVERRIDE}"
fi
if [[ -n "${RUN_JOBS_OVERRIDE}" ]]; then
  RUN_JOBS="${RUN_JOBS_OVERRIDE}"
fi
if [[ -n "${CARGO_BUILD_JOBS_OVERRIDE}" ]]; then
  CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS_OVERRIDE}"
fi
if [[ -n "${DISABLE_CACHE_OVERRIDE}" ]]; then
  DISABLE_CACHE="${DISABLE_CACHE_OVERRIDE}"
fi
if [[ -n "${CACHE_DIR_OVERRIDE}" ]]; then
  CACHE_DIR="${CACHE_DIR_OVERRIDE}"
fi

echo "Running e2e pass suite"
echo "  profile=${PROFILE}"
echo "  mode=${MODE}"
echo "  sifr_jobs=${SIFR_JOBS}"
echo "  rust_jobs=${RUST_JOBS}"
echo "  run_jobs=${RUN_JOBS}"
echo "  cargo_build_jobs=${CARGO_BUILD_JOBS}"
echo "  disable_cache=${DISABLE_CACHE}"
echo "  cache_dir=${CACHE_DIR}"

SIFR_E2E_PROFILE="${PROFILE}" \
SIFR_E2E_RUNNER_MODE="${MODE}" \
SIFR_E2E_SIFR_JOBS="${SIFR_JOBS}" \
SIFR_E2E_RUST_JOBS="${RUST_JOBS}" \
SIFR_E2E_RUN_JOBS="${RUN_JOBS}" \
SIFR_E2E_CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS}" \
SIFR_E2E_DISABLE_CACHE="${DISABLE_CACHE}" \
SIFR_E2E_CACHE_DIR="${CACHE_DIR}" \
cargo test -p sifr --test e2e test_e2e_pass -- --nocapture
