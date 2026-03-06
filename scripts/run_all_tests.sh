#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run_all_tests.sh [options]

Run local-first validation for the selected profile.

Profiles:
  quick   Fast local signal.
  full    Authoritative local gate (default).
  stress  High-contention parity check.

Options:
  --profile <quick|full|stress>  Validation profile (default: full)
  --help                         Show this help

Any remaining arguments are forwarded to scripts/run_e2e_pass.sh.
EOF
}

PROFILE="${SIFR_TEST_PROFILE:-full}"
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
    *)
      FORWARD_ARGS+=("$1")
      shift
      ;;
  esac
done

case "${PROFILE}" in
  quick|full|stress)
    ;;
  *)
    echo "unsupported profile: ${PROFILE}" >&2
    usage >&2
    exit 2
    ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}/.."

echo "Running local-first validation"
echo "  profile=${PROFILE}"
echo "Running HIR maintainability guardrails"
python3 "${SCRIPT_DIR}/check_hir_maintainability_guardrails.py"

echo "Running unit tests and non-pass e2e tests (cargo test -p sifr -- --skip test_e2e_pass)"
cargo test -p sifr -- --skip test_e2e_pass

echo "Running frontend mode parity matrix"
bash "${SCRIPT_DIR}/run_frontend_mode_parity_matrix.sh"

echo "Running phase 23 graph/isolation matrix"
bash "${SCRIPT_DIR}/run_phase23_graph_isolation_matrix.sh"

echo "Running phase 24 HIR analysis consolidation matrix"
bash "${SCRIPT_DIR}/run_phase24_hir_analysis_consolidation_matrix.sh"

echo "Running e2e pass suite"
bash "${SCRIPT_DIR}/run_e2e_pass.sh" --profile "${PROFILE}" "${FORWARD_ARGS[@]}"
