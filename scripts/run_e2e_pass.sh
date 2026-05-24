#!/usr/bin/env bash

set -euo pipefail

PROFILE="${SIFR_E2E_PROFILE:-pr}"
SIFR_JOBS_OVERRIDE=""
RUST_JOBS_OVERRIDE=""
RUN_JOBS_OVERRIDE=""
CARGO_BUILD_JOBS_OVERRIDE=""
DISABLE_CACHE_OVERRIDE=""
CACHE_DIR_OVERRIDE=""
FIXTURE_MANIFEST_OVERRIDE=""
FIXTURE_MANIFEST_DEFAULT=""

usage() {
  cat <<'EOF'
Usage: scripts/run_e2e_pass.sh [options]

Profiles:
  quick   Fast local signal; bounded parallel workers; cache enabled.
  pr      Authoritative merge gate; representative corpus; cache enabled.
  nightly Broad pass-corpus lane; cache enabled.
  release Highest-confidence local gate; cache enabled.

Legacy aliases:
  full    Alias for `pr`
  stress  Alias for `release`

Options:
  --profile <quick|pr|nightly|release|full|stress> Local execution profile (default: pr)
  --sifr-jobs <n>              Parallel Sifr compile workers
  --rust-jobs <n>              Parallel group build workers
  --run-jobs <n>               Parallel group run workers
  --cargo-build-jobs <n>       Cargo jobs per generated group build
  --cache-dir <path>           e2e cache directory (default: target/sifr_e2e_cache/<profile>)
  --fixture-manifest <path>    JSON manifest listing the selected e2e pass fixtures
  --no-cache                   Disable the e2e cache for this run
  --help                       Show this help
EOF
}

canonicalize_profile() {
  python3 "${SCRIPT_DIR}/validation_lane.py" canonical-profile --profile "$1"
}

set_profile_defaults() {
  local profile="$1"
  case "${profile}" in
    quick)
      SIFR_JOBS="2"
      RUST_JOBS="2"
      RUN_JOBS="2"
      CARGO_BUILD_JOBS="1"
      DISABLE_CACHE="0"
      FIXTURE_MANIFEST_DEFAULT="verification/validation_lanes/quick_e2e_manifest.json"
      ;;
    pr)
      SIFR_JOBS="4"
      RUST_JOBS="3"
      RUN_JOBS="3"
      CARGO_BUILD_JOBS="1"
      DISABLE_CACHE="0"
      FIXTURE_MANIFEST_DEFAULT="verification/validation_lanes/pr_e2e_manifest.json"
      ;;
    nightly|release)
      SIFR_JOBS="6"
      RUST_JOBS="4"
      RUN_JOBS="4"
      CARGO_BUILD_JOBS="1"
      DISABLE_CACHE="0"
      FIXTURE_MANIFEST_DEFAULT=""
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
    --fixture-manifest)
      FIXTURE_MANIFEST_OVERRIDE="${2:-}"
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROFILE="$(canonicalize_profile "${PROFILE}")"
set_profile_defaults "${PROFILE}"

SIFR_JOBS="${SIFR_E2E_SIFR_JOBS:-${SIFR_JOBS}}"
RUST_JOBS="${SIFR_E2E_RUST_JOBS:-${RUST_JOBS}}"
RUN_JOBS="${SIFR_E2E_RUN_JOBS:-${RUN_JOBS}}"
CARGO_BUILD_JOBS="${SIFR_E2E_CARGO_BUILD_JOBS:-${CARGO_BUILD_JOBS}}"
DISABLE_CACHE="${SIFR_E2E_DISABLE_CACHE:-${DISABLE_CACHE}}"
CACHE_DIR="${SIFR_E2E_CACHE_DIR:-target/sifr_e2e_cache/${PROFILE}}"
FIXTURE_MANIFEST="${SIFR_E2E_FIXTURE_MANIFEST:-${FIXTURE_MANIFEST_DEFAULT}}"

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
if [[ -n "${FIXTURE_MANIFEST_OVERRIDE}" ]]; then
  FIXTURE_MANIFEST="${FIXTURE_MANIFEST_OVERRIDE}"
fi
if [[ "${CACHE_DIR}" != /* ]]; then
  CACHE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)/${CACHE_DIR}"
fi
if [[ -n "${FIXTURE_MANIFEST}" && "${FIXTURE_MANIFEST}" != /* ]]; then
  FIXTURE_MANIFEST="$(cd "${SCRIPT_DIR}/.." && pwd)/${FIXTURE_MANIFEST}"
fi

echo "Running e2e pass suite"
echo "  profile=${PROFILE}"
echo "  sifr_jobs=${SIFR_JOBS}"
echo "  rust_jobs=${RUST_JOBS}"
echo "  run_jobs=${RUN_JOBS}"
echo "  cargo_build_jobs=${CARGO_BUILD_JOBS}"
echo "  disable_cache=${DISABLE_CACHE}"
echo "  cache_dir=${CACHE_DIR}"
if [[ -n "${FIXTURE_MANIFEST}" ]]; then
  echo "  fixture_manifest=${FIXTURE_MANIFEST}"
fi

SIFR_E2E_PROFILE="${PROFILE}" \
SIFR_E2E_SIFR_JOBS="${SIFR_JOBS}" \
SIFR_E2E_RUST_JOBS="${RUST_JOBS}" \
SIFR_E2E_RUN_JOBS="${RUN_JOBS}" \
SIFR_E2E_CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS}" \
SIFR_E2E_DISABLE_CACHE="${DISABLE_CACHE}" \
SIFR_E2E_CACHE_DIR="${CACHE_DIR}" \
SIFR_E2E_FIXTURE_MANIFEST="${FIXTURE_MANIFEST}" \
cargo test -p sifr --test e2e test_e2e_pass -- --nocapture
