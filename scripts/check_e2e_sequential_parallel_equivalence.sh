#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/check_e2e_sequential_parallel_equivalence.sh [--profile <merge|nightly|release>] [--help]

Run the e2e pass suite with sequential and parallel worker settings and assert report signature equivalence.
EOF
}

PROFILE="release"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

PROFILE="$(python3 "${SCRIPT_DIR}/validation_lane.py" profile --profile "${PROFILE}")"
if [[ "${PROFILE}" == "create-pr" ]]; then
  echo "sequential-vs-parallel equivalence is not part of the create-pr lane; use merge, nightly, or release" >&2
  exit 2
fi
REPO_ROOT="${SCRIPT_DIR}/.."

extract_signature() {
  local mode="$1"
  local log_file
  log_file="$(mktemp "${TMPDIR:-/tmp}/sifr-e2e-eq-${mode}.XXXXXX")"

  (
    cd "${REPO_ROOT}"
    if [[ "${mode}" == "sequential" ]]; then
      bash "${SCRIPT_DIR}/run_e2e_pass.sh" \
        --profile "${PROFILE}" \
        --sifr-jobs 1 \
        --rust-jobs 1 \
        --run-jobs 1 \
        --cargo-build-jobs 1 \
        --no-cache
    else
      bash "${SCRIPT_DIR}/run_e2e_pass.sh" --profile "${PROFILE}" --no-cache
    fi
  ) 2>&1 | tee "${log_file}" >/dev/null

  local signature
  signature="$(
    grep -Eo '\[sifr-e2e\] report_signature=[0-9a-f]+' "${log_file}" \
      | tail -n1 \
      | sed 's/.*=//'
  )"
  if [[ -z "${signature}" ]]; then
    echo "missing report signature for mode ${mode}; see ${log_file}" >&2
    exit 1
  fi

  echo "${signature}"
}

echo "Running sequential-vs-parallel e2e equivalence check"
echo "  profile=${PROFILE}"

SEQ_SIG="$(extract_signature sequential)"
PAR_SIG="$(extract_signature parallel)"

if [[ "${SEQ_SIG}" != "${PAR_SIG}" ]]; then
  echo "report signature mismatch: sequential=${SEQ_SIG}, parallel=${PAR_SIG}" >&2
  exit 1
fi

echo "sequential-vs-parallel report signature confirmed: ${SEQ_SIG}"
