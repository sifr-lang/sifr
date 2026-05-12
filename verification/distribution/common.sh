#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SITE_INSTALL_ROOT="${SIFR_SITE_INSTALL_ROOT:-/Users/yaseralnajjar/work/sifr/sifr-blog-website/apps/sifr-site/public/install}"
DISPATCH_BASE_URL="${SIFR_INSTALL_BASE_URL:-file://${SITE_INSTALL_ROOT}}"

run_dispatcher() {
  local dispatcher="$1"
  shift
  SIFR_INSTALL_BASE_URL="${DISPATCH_BASE_URL}" \
    SIFR_DISPATCH_TRACE=1 \
    sh "${SITE_INSTALL_ROOT}/${dispatcher}" "$@"
}

require_success_contains() {
  local expected="$1"
  shift
  local output
  output="$("$@" 2>&1)"
  if [[ "${output}" != *"${expected}"* ]]; then
    echo "expected output to contain: ${expected}" >&2
    echo "--- output ---" >&2
    echo "${output}" >&2
    exit 1
  fi
}

require_failure_contains() {
  local expected="$1"
  shift
  local output
  set +e
  output="$("$@" 2>&1)"
  local status=$?
  set -e
  if [[ ${status} -eq 0 ]]; then
    echo "expected command to fail" >&2
    echo "--- output ---" >&2
    echo "${output}" >&2
    exit 1
  fi
  if [[ "${output}" != *"${expected}"* ]]; then
    echo "expected failure output to contain: ${expected}" >&2
    echo "--- output ---" >&2
    echo "${output}" >&2
    exit 1
  fi
}

make_dispatcher_fixture() {
  local target_root="$1"
  local alpha_version="${2:-0.1.0-alpha.1}"
  local beta_version="${3:-0.1.0-beta.1}"
  mkdir -p "${target_root}/versions"
  "${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
    --install-root "${target_root}" \
    --alpha-version "${alpha_version}" \
    --beta-version "${beta_version}" \
    --base-url "file://${target_root}" >/dev/null
}
