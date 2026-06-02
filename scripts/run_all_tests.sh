#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORIGINAL_ARGS=("$@")

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

if [[ -z "${SIFR_LANE_REPORT_CAPTURED:-}" ]]; then
  REPORT_DIR="${SCRIPT_DIR}/../target/validation_lane_reports"
  mkdir -p "${REPORT_DIR}"
  LOG_FILE="$(mktemp "${REPORT_DIR}/lane.${PROFILE}.log.XXXXXX")"
  TIME_FILE="$(mktemp "${REPORT_DIR}/lane.${PROFILE}.time.XXXXXX")"
  LATEST_LOG_FILE="${REPORT_DIR}/${PROFILE}.latest.log"
  LATEST_TIME_FILE="${REPORT_DIR}/${PROFILE}.latest.time"
  JSON_FILE="${REPORT_DIR}/${PROFILE}.latest.json"

  set +e
  /usr/bin/time -l -o "${TIME_FILE}" \
    env SIFR_LANE_REPORT_CAPTURED=1 \
    bash "$0" "${ORIGINAL_ARGS[@]}" \
    > >(tee "${LOG_FILE}") 2>&1
  STATUS=$?
  set -e

  cp "${LOG_FILE}" "${LATEST_LOG_FILE}"
  cp "${TIME_FILE}" "${LATEST_TIME_FILE}"

  python3 "${SCRIPT_DIR}/validation_lane_report.py" summarize \
    --profile "${PROFILE}" \
    --log "${LATEST_LOG_FILE}" \
    --time-file "${LATEST_TIME_FILE}" \
    --json-out "${JSON_FILE}" || true
  rm -f "${LOG_FILE}" "${TIME_FILE}"
  exit "${STATUS}"
fi

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

echo "Running file-size guardrails"
python3 "${SCRIPT_DIR}/check_file_size_guardrails.py"

echo "Running source crate dependency-direction guardrail"
python3 "${SCRIPT_DIR}/check_source_crate_dependency_direction.py"

echo "Running TypeScript-Go architecture transfer M1 guardrails"
python3 "${SCRIPT_DIR}/../verification/tooling/check_typescript_go_m1_guardrails.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_typescript_go_m1_guardrails.py" --self-test

echo "Running sifr_driver maintainability guardrails"
python3 "${SCRIPT_DIR}/check_sifr_driver_maintainability_guardrails.py"

echo "Running package-manager guardrails"
python3 "${SCRIPT_DIR}/check_package_manager_guardrails.py"

echo "Running diagnostic schema sync check"
python3 "${SCRIPT_DIR}/check_diagnostic_schema_sync.py"

echo "Running diagnostic docs sync check"
python3 "${SCRIPT_DIR}/check_diagnostic_docs_sync.py"

echo "Running diagnostic code coverage check"
python3 "${SCRIPT_DIR}/check_diagnostic_code_coverage.py"

echo "Running diagnostic baseline hygiene check"
python3 "${SCRIPT_DIR}/check_diagnostic_baseline_hygiene.py"

echo "Running diagnostic cancel usage check"
python3 "${SCRIPT_DIR}/check_diagnostic_cancel_usage.py"

echo "Running diagnostic transport cleanup check"
python3 "${SCRIPT_DIR}/check_diagnostic_transport_cleanup.py"

echo "Running diagnostic presentation contract check"
python3 "${SCRIPT_DIR}/../verification/tooling/check_diagnostic_presentation_contract.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_diagnostic_presentation_contract.py" --self-test

echo "Running diagnostic source canonicalization contract check"
python3 "${SCRIPT_DIR}/../verification/tooling/check_diagnostic_source_canonicalization_contract.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_diagnostic_source_canonicalization_contract.py" --self-test

echo "Running Phase 35 frontend and syntax guardrails"
python3 "${SCRIPT_DIR}/../verification/performance/check_ruff_fork_update_contract.py"
python3 "${SCRIPT_DIR}/../verification/performance/check_split_brain_guardrail.py"
python3 "${SCRIPT_DIR}/../verification/performance/check_split_brain_guardrail.py" --self-test
python3 "${SCRIPT_DIR}/../verification/performance/check_frontend_cache_contract.py"

echo "Running Developer Tooling Checks"
python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_contract_lock.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_contract_lock.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_dependency_boundaries.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_dependency_boundaries.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_lsp_split_brain.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_lsp_split_brain.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_linter_diagnostic_class.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_linter_diagnostic_class.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension_contract.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension_contract.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_contract.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_contract.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_phase_manifests.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_phase_manifests.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_ast_coverage.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_ast_coverage.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_rule_suppression_contract.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_rule_suppression_contract.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_contract.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_contract.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_coherence.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_coherence.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_split_brain.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_split_brain.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/run_tooling_parity.py"
python3 "${SCRIPT_DIR}/../verification/tooling/run_tooling_parity.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_completion_quality.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_completion_quality.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_smoke.py"
python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_smoke.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_stress.py"
python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_stress.py" --self-test
git submodule update --init verification/sifr-large-lsp-verification
python3 "${SCRIPT_DIR}/../verification/sifr-large-lsp-verification/tools/generate_corpus.py" check
python3 "${SCRIPT_DIR}/../verification/tooling/lsp_large_session.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/lsp_large_session.py" --mode smoke --require-submodule
python3 "${SCRIPT_DIR}/../verification/tooling/check_editor_assets.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_editor_assets.py" --self-test
python3 "${SCRIPT_DIR}/../verification/tooling/check_phase36_closeout.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_phase36_closeout.py" --self-test

echo "Running Performance Budget Checks"
python3 "${SCRIPT_DIR}/../verification/performance/run_benchmarks.py" --validate-only
python3 "${SCRIPT_DIR}/../verification/performance/run_benchmarks.py" --self-test
python3 "${SCRIPT_DIR}/../verification/performance/check_budgets.py"
python3 "${SCRIPT_DIR}/../verification/performance/check_budgets.py" --self-test
run_performance_budget_subset() {
  local perf_results="$1"
  python3 "${SCRIPT_DIR}/../verification/performance/run_benchmarks.py" \
    --case check-single-file-001-arithmetic \
    --case check-project-004-project-graph \
    --case build-single-file-001-break-continue \
    --case build-project-001-additional-modules \
    --case formatter-corpus-001-project-check \
    --case formatter-large-file-001-check \
    --case incremental-local-loop-001-unchanged-file-update \
    --case interactive-tooling-foundation-002-warm-diagnostics-query \
    --case lsp-query-003-diagnostics \
    --case phase27-non-regression-002-json-diagnostic-schema \
    --json-out "${perf_results}"
  python3 "${SCRIPT_DIR}/../verification/performance/check_budgets.py" \
    --results "${perf_results}" \
    --allow-subset
}
if [[ "${PROFILE}" == "quick" ]]; then
  python3 "${SCRIPT_DIR}/../verification/performance/run_benchmarks.py" \
    --sample-scale smoke \
    --case formatter-corpus-001-project-check \
    --case formatter-large-file-001-check \
    --case incremental-local-loop-001-unchanged-file-update \
    --case interactive-tooling-foundation-002-warm-diagnostics-query \
    --case lsp-query-003-diagnostics
else
  PERF_RESULTS="target/performance/${PROFILE}.budget.latest.json"
  if ! run_performance_budget_subset "${PERF_RESULTS}"; then
    PERFORMANCE_PASSED=0
    for attempt in 2 3 4 5; do
      RETRY_PERF_RESULTS="target/performance/${PROFILE}.budget.retry-${attempt}.latest.json"
      echo "performance budget subset failed; retrying attempt ${attempt}/5 with unchanged thresholds"
      if run_performance_budget_subset "${RETRY_PERF_RESULTS}"; then
        PERFORMANCE_PASSED=1
        break
      fi
    done
    if [[ "${PERFORMANCE_PASSED}" != "1" ]]; then
      echo "performance budget subset failed after 5 attempts with unchanged thresholds" >&2
      exit 1
    fi
  fi
fi

echo "Running verification hardening script self-tests"
python3 "${SCRIPT_DIR}/run_verification_hardening.py" --self-test

echo "Running distribution validation"
bash "${SCRIPT_DIR}/run_distribution_validation.sh"

if [[ "${PROFILE}" == "pr" || "${PROFILE}" == "nightly" || "${PROFILE}" == "release" ]]; then
  echo "Running Generated Code Quality Checks"
  bash "${SCRIPT_DIR}/../verification/generated_code_quality/generated_code_quality_corpus.sh"
  bash "${SCRIPT_DIR}/../verification/generated_code_quality/generated_code_quality_panic_scan.sh"
  bash "${SCRIPT_DIR}/../verification/generated_code_quality/generated_code_quality_rustfmt.sh"
  bash "${SCRIPT_DIR}/../verification/generated_code_quality/generated_code_quality_clippy.sh"
  bash "${SCRIPT_DIR}/../verification/generated_code_quality/generated_code_quality_determinism.sh"
  bash "${SCRIPT_DIR}/../verification/generated_code_quality/generated_code_quality_demos.sh"
fi

echo "Running sifr_diagnostics tests"
cargo test -p sifr_diagnostics

echo "Running sifr_hir tests"
cargo test -p sifr_hir -- --skip test_e2e_pass

echo "Running sifr_syntax tests"
cargo test -p sifr_syntax

echo "Running sifr_frontend tests"
cargo test -p sifr_frontend

echo "Running sifr_analysis tests"
cargo test -p sifr_analysis

echo "Running sifr_lsp tests"
cargo test -p sifr_lsp

echo "Running sifr_package tests"
cargo test -p sifr_package

echo "Running unit tests and non-pass e2e tests (cargo test -p sifr -- --skip test_e2e_pass)"
cargo test -p sifr -- --skip test_e2e_pass

echo "Running sifr_driver library tests"
cargo test -p sifr_driver --lib

if [[ -n "${CONTRACT_SUITES}" ]]; then
  echo "Running validation contract matrix suites"
  CONTRACT_ARGS=()
  IFS=',' read -r -a CONTRACT_SUITE_ARRAY <<< "${CONTRACT_SUITES}"
  for suite in "${CONTRACT_SUITE_ARRAY[@]}"; do
    if [[ -n "${suite}" ]]; then
      CONTRACT_ARGS+=(--suite "${suite}")
    fi
  done
  bash "${SCRIPT_DIR}/run_validation_contract_matrix.sh" "${CONTRACT_ARGS[@]}"
fi

echo "Running e2e pass suite"
E2E_ARGS=(
  --profile "${E2E_PROFILE}"
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
