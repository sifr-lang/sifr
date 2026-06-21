#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MIN_UV_VERSION="${SIFR_MIN_UV_VERSION:-0.9.28}"

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
  --emit-plan                                 Print the selected profile execution plan and exit
  --help                                      Show this help

Any remaining arguments are forwarded to the verification-owned e2e pass runner.
EOF
}

PROFILE="${SIFR_TEST_PROFILE:-merge}"
EMIT_PLAN=0
FORWARD_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
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
    *)
      FORWARD_ARGS+=("$1")
      shift
      ;;
  esac
done

version_gte() {
  python3 - "$1" "$2" <<'PY'
import re
import sys


def parse(value: str) -> tuple[int, ...]:
    parts = [int(part) for part in re.findall(r"\d+", value)]
    return tuple(parts)


actual = parse(sys.argv[1])
minimum = parse(sys.argv[2])
width = max(len(actual), len(minimum))
actual += (0,) * (width - len(actual))
minimum += (0,) * (width - len(minimum))
raise SystemExit(0 if actual >= minimum else 1)
PY
}

require_uv() {
  if ! command -v uv >/dev/null 2>&1; then
    cat >&2 <<EOF
error: uv ${MIN_UV_VERSION} or newer is required for verification tooling.
Install uv and re-run this facade; see verification/README.md.
EOF
    exit 2
  fi

  local uv_version
  uv_version="$(uv --version | awk '{print $2}')"
  if ! version_gte "${uv_version}" "${MIN_UV_VERSION}"; then
    cat >&2 <<EOF
error: uv ${MIN_UV_VERSION} or newer is required for verification tooling; found ${uv_version}.
Upgrade uv and re-run this facade; see verification/README.md.
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
  exec uv run --project "${SCRIPT_DIR}/../verification" --locked \
    python -m sifr_verify profiles run --profile "${PROFILE}" -- "${FORWARD_ARGS[@]}"
fi

exec uv run --project "${SCRIPT_DIR}/../verification" --locked \
  python -m sifr_verify profiles run --profile "${PROFILE}"
