#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: verification/runner/e2e/check_report_determinism.sh [--profile <merge|nightly|release>] [--help]

Run the e2e pass suite twice and assert the emitted report signature is identical.
EOF
}

PROFILE="release"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

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
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

PROFILE="$(uv run --project "${REPO_ROOT}/verification" --locked python -m sifr_verify profiles profile --profile "${PROFILE}")"
if [[ "${PROFILE}" == "create-pr" ]]; then
  echo "determinism checks are not part of the create-pr lane; use merge, nightly, or release" >&2
  exit 2
fi
extract_signature() {
  local run_id="$1"
  local log_file
  log_file="$(mktemp "${TMPDIR:-/tmp}/sifr-e2e-determinism-${run_id}.XXXXXX")"

  (
    cd "${REPO_ROOT}"
    bash "${SCRIPT_DIR}/run_e2e_pass.sh" --profile "${PROFILE}"
  ) 2>&1 | tee "${log_file}" >/dev/null

  local signature
  signature="$(
    grep -Eo '\[sifr-e2e\] report_signature=[0-9a-f]+' "${log_file}" \
      | tail -n1 \
      | sed 's/.*=//'
  )"
  if [[ -z "${signature}" ]]; then
    echo "missing report signature in run ${run_id}; see ${log_file}" >&2
    exit 1
  fi

  echo "${signature}"
}

echo "Running deterministic-report check"
echo "  profile=${PROFILE}"

SIG_A="$(extract_signature run1)"
SIG_B="$(extract_signature run2)"

if [[ "${SIG_A}" != "${SIG_B}" ]]; then
  echo "report signature mismatch: run1=${SIG_A}, run2=${SIG_B}" >&2
  exit 1
fi

echo "deterministic report signature confirmed: ${SIG_A}"
