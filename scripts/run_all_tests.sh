#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run_all_tests.sh [options]

Run local-first validation for the selected lane.

Lanes:
  quick   Fast local signal.
  pr      Authoritative merge gate (default).
  nightly Broad hardening and full-corpus signal.
  release Highest-confidence local qualification gate.

Legacy aliases:
  full    Alias for `pr`
  stress  Alias for `release`

Options:
  --profile <quick|pr|nightly|release|full|stress>  Validation lane (default: pr)
  --help                         Show this help

Any remaining arguments are forwarded to scripts/run_e2e_pass.sh.
EOF
}

PROFILE="${SIFR_TEST_PROFILE:-pr}"
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}/.."

LANE_EXPORTS="$(python3 "${SCRIPT_DIR}/validation_lane.py" shell --profile "${PROFILE}")"
eval "${LANE_EXPORTS}"

PROFILE="${CANONICAL_PROFILE}"

echo "Running local-first validation"
echo "  profile=${PROFILE}"
echo "  lane=${LANE_NAME}"
echo "  budget=warm<=${WARM_TARGET_MINUTES}m cold<=${COLD_TARGET_MINUTES}m"
echo "  policy=thermal:${THERMAL_POLICY} memory:${MEMORY_POLICY}"
echo "Running HIR maintainability guardrails"
python3 "${SCRIPT_DIR}/check_hir_maintainability_guardrails.py"

echo "Running sifr_driver maintainability guardrails"
python3 "${SCRIPT_DIR}/check_sifr_driver_maintainability_guardrails.py"

echo "Running unit tests and non-pass e2e tests (cargo test -p sifr -- --skip test_e2e_pass)"
cargo test -p sifr -- --skip test_e2e_pass

if [[ "${RUN_FRONTEND_MODE_PARITY}" == "1" ]]; then
  echo "Running frontend mode parity matrix"
  bash "${SCRIPT_DIR}/run_frontend_mode_parity_matrix.sh"
fi

if [[ "${RUN_PHASE23_GRAPH_ISOLATION}" == "1" ]]; then
  echo "Running phase 23 graph/isolation matrix"
  bash "${SCRIPT_DIR}/run_phase23_graph_isolation_matrix.sh"
fi

if [[ "${RUN_PHASE24_HIR_ANALYSIS}" == "1" ]]; then
  echo "Running phase 24 HIR analysis consolidation matrix"
  bash "${SCRIPT_DIR}/run_phase24_hir_analysis_consolidation_matrix.sh"
fi

if [[ "${RUN_PHASE25_CFG_FLOW}" == "1" ]]; then
  echo "Running phase 25 CFG/flow activation matrix"
  bash "${SCRIPT_DIR}/run_phase25_cfg_flow_activation_matrix.sh"
fi

echo "Running e2e pass suite"
E2E_ARGS=(
  --profile "${E2E_PROFILE}"
  --mode "${E2E_MODE}"
  --sifr-jobs "${E2E_SIFR_JOBS}"
  --rust-jobs "${E2E_RUST_JOBS}"
  --run-jobs "${E2E_RUN_JOBS}"
  --cargo-build-jobs "${E2E_CARGO_BUILD_JOBS}"
)
if [[ -n "${E2E_FIXTURE_MANIFEST}" ]]; then
  E2E_ARGS+=(--fixture-manifest "${E2E_FIXTURE_MANIFEST}")
fi
if [[ "${E2E_DISABLE_CACHE}" == "1" ]]; then
  E2E_ARGS+=(--no-cache)
fi
bash "${SCRIPT_DIR}/run_e2e_pass.sh" "${E2E_ARGS[@]}" ${FORWARD_ARGS[@]+"${FORWARD_ARGS[@]}"}

if [[ "${RUN_HARDENING}" == "1" ]]; then
  echo "Running phase 29 verification hardening suites"
  HARDENING_ARGS=(--profile "${PROFILE}")
  IFS=',' read -r -a HARDENING_SUITE_ARRAY <<< "${HARDENING_SUITES}"
  for suite in "${HARDENING_SUITE_ARRAY[@]}"; do
    if [[ -n "${suite}" ]]; then
      HARDENING_ARGS+=(--suite "${suite}")
    fi
  done
  python3 "${SCRIPT_DIR}/run_verification_hardening.py" "${HARDENING_ARGS[@]}"
fi

if [[ "${RUN_E2E_REPORT_DETERMINISM}" == "1" ]]; then
  echo "Running e2e report determinism check"
  bash "${SCRIPT_DIR}/check_e2e_report_determinism.sh" --profile "${PROFILE}"
fi

if [[ "${RUN_E2E_SEQUENTIAL_PARALLEL_EQUIVALENCE}" == "1" ]]; then
  echo "Running e2e sequential-vs-parallel equivalence check"
  bash "${SCRIPT_DIR}/check_e2e_sequential_parallel_equivalence.sh" --profile "${PROFILE}"
fi
