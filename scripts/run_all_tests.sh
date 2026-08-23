#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UV_PROJECT_FILE="${SCRIPT_DIR}/../verification/pyproject.toml"

usage() {
  cat <<'EOF'
Usage: scripts/run_all_tests.sh [options]

Run local-first validation for the selected profile.

Profiles:
  create-pr Fast local create-PR signal.
  merge     Authoritative merge gate (default).
  nightly   Broad hardening and full-corpus signal.
  release   Highest-confidence local qualification gate.
  python-interop-live
            Explicit opt-in Python interop container/runtime policy and live examples.

Options:
  --profile <name>                            Validation profile (default: merge)
  --release-report-out <path>                 Write immutable release evidence (release only)
  --emit-plan                                 Print the selected profile execution plan and exit
  --help                                      Show this help

Any remaining arguments are forwarded to the verification-owned e2e pass runner.
EOF
}

PROFILE="${SIFR_TEST_PROFILE:-merge}"
EMIT_PLAN=0
FORWARD_ARGS=()
RELEASE_REPORT_OUT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      [[ $# -ge 2 && -n "${2:-}" ]] || { echo "error: --profile requires a value" >&2; exit 2; }
      PROFILE="${2:-}"
      shift 2
      ;;
    --help)
      usage
      exit 0
      ;;
    --emit-plan)
      EMIT_PLAN=1
      shift
      ;;
    --release-report-out)
      [[ $# -ge 2 && -n "${2:-}" ]] || { echo "error: --release-report-out requires a path" >&2; exit 2; }
      RELEASE_REPORT_OUT="${2:-}"
      shift 2
      ;;
    *)
      FORWARD_ARGS+=("$1")
      shift
      ;;
  esac
done

required_uv_version() {
  python3 - "${UV_PROJECT_FILE}" <<'PY'
import re
import sys
import tomllib

with open(sys.argv[1], "rb") as project_file:
    requirement = tomllib.load(project_file)["tool"]["uv"]["required-version"]
if re.fullmatch(r"==\d+\.\d+\.\d+", requirement) is None:
    raise SystemExit("verification uv required-version must be an exact == pin")
print(requirement.removeprefix("=="))
PY
}

require_uv() {
  local expected_version
  expected_version="$(required_uv_version)"

  if ! command -v uv >/dev/null 2>&1; then
    cat >&2 <<EOF
error: uv ${expected_version} is required for verification tooling.
Install uv and re-run this facade; see verification/README.md.
EOF
    exit 2
  fi

  local uv_version
  uv_version="$(uv --version | awk '{print $2}')"
  if [[ "${uv_version}" != "${expected_version}" ]]; then
    cat >&2 <<EOF
error: uv ${expected_version} is required for verification tooling; found ${uv_version}.
Install the exact uv release and re-run this facade; see verification/README.md.
EOF
    exit 2
  fi
}

require_uv

if [[ "${EMIT_PLAN}" -eq 1 ]]; then
  exec uv run --project "${SCRIPT_DIR}/../verification" --locked \
    python -m sifr_verify profiles plan --profile "${PROFILE}"
fi

if [[ "${#FORWARD_ARGS[@]}" -gt 0 ]]; then
  profile_args=(profiles run --profile "${PROFILE}")
  if [[ -n "${RELEASE_REPORT_OUT}" ]]; then
    profile_args+=(--release-report-out "${RELEASE_REPORT_OUT}")
  fi
  exec uv run --project "${SCRIPT_DIR}/../verification" --locked \
    python -m sifr_verify "${profile_args[@]}" -- "${FORWARD_ARGS[@]}"
fi

if [[ -n "${RELEASE_REPORT_OUT}" ]]; then
  exec uv run --project "${SCRIPT_DIR}/../verification" --locked \
    python -m sifr_verify profiles run --profile "${PROFILE}" \
    --release-report-out "${RELEASE_REPORT_OUT}"
fi

exec uv run --project "${SCRIPT_DIR}/../verification" --locked \
  python -m sifr_verify profiles run --profile "${PROFILE}"
