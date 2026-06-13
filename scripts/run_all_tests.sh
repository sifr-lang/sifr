#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORIGINAL_ARGS=("$@")

usage() {
  cat <<'EOF'
Usage: scripts/run_all_tests.sh [options]

Run local-first validation for the selected profile.

Profiles:
  create-pr Fast local create-PR signal.
  merge     Authoritative merge gate (default).
  nightly Broad hardening and full-corpus signal.
  release Highest-confidence local qualification gate.

Options:
  --profile <create-pr|merge|nightly|release>  Validation profile (default: merge)
  --help                         Show this help

Any remaining arguments are forwarded to scripts/run_e2e_pass.sh.
EOF
}

PROFILE="${SIFR_TEST_PROFILE:-merge}"
FORWARD_ARGS=()
MIN_UV_VERSION="${SIFR_MIN_UV_VERSION:-0.9.28}"

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

  uv run --project "${SCRIPT_DIR}/../verification" --locked python -m sifr_verify reports summarize \
    --profile "${PROFILE}" \
    --log "${LATEST_LOG_FILE}" \
    --time-file "${LATEST_TIME_FILE}" \
    --json-out "${JSON_FILE}" || true
  rm -f "${LOG_FILE}" "${TIME_FILE}"
  exit "${STATUS}"
fi

cd "${SCRIPT_DIR}/.."

LANE_EXPORTS="$(uv run --project "${SCRIPT_DIR}/../verification" --locked python -m sifr_verify profiles shell --profile "${PROFILE}")"
eval "${LANE_EXPORTS}"

PROFILE="${RESOLVED_PROFILE}"

echo "Running local-first validation"
echo "  profile=${PROFILE}"
echo "  lane=${LANE_NAME}"
echo "  budget=warm<=${WARM_TARGET_MINUTES}m cold<=${COLD_TARGET_MINUTES}m"
echo "  policy=thermal:${THERMAL_POLICY} memory:${MEMORY_POLICY}"

now_ms() {
  python3 -c 'import time; print(time.monotonic_ns() // 1_000_000)'
}

timed_step() {
  local name="$1"
  shift
  local start_ms
  local end_ms
  local elapsed_ms
  local status
  start_ms="$(now_ms)"
  set +e
  (set -euo pipefail; "$@")
  status=$?
  set -e
  end_ms="$(now_ms)"
  elapsed_ms=$((end_ms - start_ms))
  if [[ "${status}" -eq 0 ]]; then
    echo "[sifr-lane-step] name=${name} elapsed_ms=${elapsed_ms} status=pass"
  else
    echo "[sifr-lane-step] name=${name} elapsed_ms=${elapsed_ms} status=fail"
  fi
  return "${status}"
}

run_core_guardrails() {
  echo "Running lowering maintainability guardrails"
  python3 "${SCRIPT_DIR}/check_hir_maintainability_guardrails.py"

  echo "Running file-size guardrails"
  python3 "${SCRIPT_DIR}/check_file_size_guardrails.py"

  echo "Running Cursor hygiene guardrails"
  python3 "${SCRIPT_DIR}/check_cursor_hygiene.py"
  python3 "${SCRIPT_DIR}/check_cursor_hygiene.py" --self-test

  echo "Running source crate dependency-direction guardrail"
  python3 "${SCRIPT_DIR}/check_source_crate_dependency_direction.py"
  python3 "${SCRIPT_DIR}/check_source_crate_dependency_direction.py" --self-test

  echo "Running TypeScript-Go architecture transfer M1 guardrails"
  python3 "${SCRIPT_DIR}/../verification/tooling/check_typescript_go_m1_guardrails.py"
  python3 "${SCRIPT_DIR}/../verification/tooling/check_typescript_go_m1_guardrails.py" --self-test

  echo "Running sifr_driver maintainability guardrails"
  python3 "${SCRIPT_DIR}/check_sifr_driver_maintainability_guardrails.py"

  echo "Running package-manager guardrails"
  python3 "${SCRIPT_DIR}/check_package_manager_guardrails.py"
}

run_diagnostic_contracts() {
  echo "Running diagnostics area contract checks"
  uv run --project "${SCRIPT_DIR}/../verification" --locked \
    python -m sifr_verify areas run --area diagnostics --suite contracts

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
}

run_frontend_syntax_guardrails() {
  echo "Running Phase 35 frontend and syntax guardrails"
  python3 "${SCRIPT_DIR}/../verification/performance/check_ruff_fork_update_contract.py"
  python3 "${SCRIPT_DIR}/../verification/performance/check_split_brain_guardrail.py"
  python3 "${SCRIPT_DIR}/../verification/performance/check_split_brain_guardrail.py" --self-test
  python3 "${SCRIPT_DIR}/../verification/performance/check_frontend_cache_contract.py"
}

tooling_enabled() {
  local suite="$1"
  [[ ",${TOOLING_SUITES}," == *",full,"* || ",${TOOLING_SUITES}," == *",${suite},"* ]]
}

run_developer_tooling_checks() {
  echo "Running Developer Tooling Checks"
  echo "  suites=${TOOLING_SUITES:-none}"
  if tooling_enabled static; then
    python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_contract_lock.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_contract_lock.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_dependency_boundaries.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_tooling_dependency_boundaries.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_lsp_split_brain.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_lsp_split_brain.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_linter_diagnostic_class.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_linter_diagnostic_class.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_rule_suppression_contract.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_rule_suppression_contract.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_completion_quality.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_completion_quality.py" --self-test
  fi
  if tooling_enabled formatter; then
    python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_contract.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_contract.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_phase_manifests.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_phase_manifests.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_ast_coverage.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_formatter_ast_coverage.py" --self-test
  fi
  if tooling_enabled analysis; then
    python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_contract.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_contract.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_coherence.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_snapshot_coherence.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_split_brain.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_analysis_split_brain.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/run_tooling_parity.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/run_tooling_parity.py" --self-test
  fi
  if tooling_enabled lsp-smoke; then
    python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_smoke.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_smoke.py" --self-test
  fi
  if tooling_enabled editor-release; then
    python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension_contract.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension_contract.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_vscode_extension.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/check_editor_assets.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_editor_assets.py" --self-test
  fi
  if tooling_enabled lsp-stress; then
    python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_stress.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/lsp_protocol_stress.py" --self-test
    git submodule update --init verification/sifr-large-lsp-verification
    python3 "${SCRIPT_DIR}/../verification/sifr-large-lsp-verification/tools/generate_corpus.py" check
    python3 "${SCRIPT_DIR}/../verification/tooling/lsp_large_session.py" --self-test
    python3 "${SCRIPT_DIR}/../verification/tooling/lsp_large_session.py" --mode smoke --require-submodule
  fi
  if tooling_enabled phase-closeout; then
    python3 "${SCRIPT_DIR}/../verification/tooling/check_phase36_closeout.py"
    python3 "${SCRIPT_DIR}/../verification/tooling/check_phase36_closeout.py" --self-test
  fi
}

run_performance_budget_checks() {
  echo "Running Performance Budget Checks"
  echo "  mode=${PERFORMANCE_BUDGET_MODE}"
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
  if [[ "${PERFORMANCE_BUDGET_MODE}" == "smoke" ]]; then
    python3 "${SCRIPT_DIR}/../verification/performance/run_benchmarks.py" \
      --sample-scale smoke \
      --case formatter-corpus-001-project-check \
      --case formatter-large-file-001-check \
      --case incremental-local-loop-001-unchanged-file-update \
      --case interactive-tooling-foundation-002-warm-diagnostics-query \
      --case lsp-query-003-diagnostics
  elif [[ "${PERFORMANCE_BUDGET_MODE}" == "representative" || "${PERFORMANCE_BUDGET_MODE}" == "full" ]]; then
    PERF_RESULTS="target/performance/${PROFILE}.budget.latest.json"
    run_performance_budget_subset "${PERF_RESULTS}"
  else
    echo "Skipping performance benchmark execution for lane ${PROFILE}"
  fi
}

run_verification_hardening_self_tests() {
  echo "Running verification hardening script self-tests"
  python3 "${SCRIPT_DIR}/run_verification_hardening.py" --self-test
}

run_verification_runner_foundation_checks() {
  echo "Running verification runner foundation checks"
  uv lock --project "${SCRIPT_DIR}/../verification" --check
  uv run --project "${SCRIPT_DIR}/../verification" --locked python -m sifr_verify --self-test
}

run_distribution_checks() {
  if [[ "${DISTRIBUTION_MODE}" == "none" ]]; then
    echo "Skipping distribution validation for lane ${PROFILE}"
    return 0
  fi
  echo "Running distribution validation"
  echo "  mode=${DISTRIBUTION_MODE}"
  bash "${SCRIPT_DIR}/run_distribution_validation.sh"
}

run_generated_code_quality_checks() {
  if [[ "${GENERATED_CODE_QUALITY_MODE}" == "none" ]]; then
    echo "Skipping Generated Code Quality Checks for lane ${PROFILE}"
    return 0
  fi
  echo "Running Generated Code Quality Checks"
  echo "  mode=${GENERATED_CODE_QUALITY_MODE}"
  local shared_root="target/sifr_generated_code_quality/${PROFILE}.shared"
  rm -rf "${shared_root}"
  export SIFR_GCQ_SHARED_ROOT="${shared_root}"
  case "${GENERATED_CODE_QUALITY_MODE}" in
    smoke|representative|full)
      uv run --project "${SCRIPT_DIR}/../verification" --locked \
        python -m sifr_verify areas run \
          --area generated_code_quality \
          --suite "${GENERATED_CODE_QUALITY_MODE}" \
          --hardening-summary
      ;;
    *)
      echo "unsupported generated-code quality mode: ${GENERATED_CODE_QUALITY_MODE}" >&2
      return 2
      ;;
  esac
}

run_crate_tests() {
  echo "Running crate tests"
  echo "  mode=${CRATE_TEST_MODE}"

  echo "Running sifr_diagnostics tests"
  cargo test -p sifr_diagnostics

  echo "Running sifr_lowering tests"
  cargo test -p sifr_lowering -- --skip test_e2e_pass

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

  echo "Running sifr_stdlib tests"
  cargo test -p sifr_stdlib

  echo "Running sifr_runtime tests"
  cargo test -p sifr_runtime

  echo "Running sifr_runtime HTTP feature tests"
  cargo test -p sifr_runtime --features http

  if [[ "${CRATE_TEST_MODE}" == "smoke" ]]; then
    echo "Running sifr CLI unit tests"
    cargo test -p sifr --bin sifr
  elif [[ "${CRATE_TEST_MODE}" == "full" ]]; then
    echo "Running unit tests and non-pass e2e tests (cargo test -p sifr -- --skip test_e2e_pass)"
    cargo test -p sifr -- --skip test_e2e_pass
  else
    echo "unsupported crate test mode: ${CRATE_TEST_MODE}" >&2
    return 2
  fi

  echo "Running sifr_driver library tests"
  cargo test -p sifr_driver --lib
}

run_validation_contract_suites() {
  if [[ -n "${CONTRACT_SUITES}" ]]; then
    echo "Running validation contract area suites"
    CORE_LANGUAGE_CONTRACT_ARGS=()
    PROJECT_WORKSPACE_CONTRACT_ARGS=()
    IFS=',' read -r -a CONTRACT_SUITE_ARRAY <<< "${CONTRACT_SUITES}"
    for suite in "${CONTRACT_SUITE_ARRAY[@]}"; do
      if [[ -n "${suite}" ]]; then
        if [[ "${suite}" == "integer_dtype_contract" ||
              "${suite}" == "phase24_hir_analysis" ||
              "${suite}" == "phase25_cfg_flow" ]]; then
          CORE_LANGUAGE_CONTRACT_ARGS+=(--suite "${suite}")
        elif [[ "${suite}" == "frontend_mode_parity" ||
                "${suite}" == "phase23_graph_isolation" ]]; then
          PROJECT_WORKSPACE_CONTRACT_ARGS+=(--suite "${suite}")
        else
          echo "unknown validation contract suite: ${suite}" >&2
          return 2
        fi
      fi
    done
    if [[ "${#CORE_LANGUAGE_CONTRACT_ARGS[@]}" -gt 0 ]]; then
      uv run --project "${SCRIPT_DIR}/../verification" --locked \
        python -m sifr_verify areas run --area core_language "${CORE_LANGUAGE_CONTRACT_ARGS[@]}"
    fi
    if [[ "${#PROJECT_WORKSPACE_CONTRACT_ARGS[@]}" -gt 0 ]]; then
      uv run --project "${SCRIPT_DIR}/../verification" --locked \
        python -m sifr_verify areas run --area project_workspace "${PROJECT_WORKSPACE_CONTRACT_ARGS[@]}"
    fi
  fi
}

run_platform_golden_suite() {
  echo "Running platform golden fixtures"
  bash "${SCRIPT_DIR}/run_platform_golden.sh"
}

run_e2e_pass_suite() {
  echo "Running e2e pass suite"
  E2E_ARGS=(
    --profile "${E2E_PROFILE}"
    --sifr-jobs "${E2E_SIFR_JOBS}"
    --rust-jobs "${E2E_RUST_JOBS}"
    --run-jobs "${E2E_RUN_JOBS}"
    --cargo-build-jobs "${E2E_CARGO_BUILD_JOBS}"
  )
  if [[ -n "${E2E_MAX_GROUP_FIXTURES}" ]]; then
    E2E_ARGS+=(--max-group-fixtures "${E2E_MAX_GROUP_FIXTURES}")
  fi
  if [[ -n "${E2E_FIXTURE_MANIFEST}" ]]; then
    E2E_ARGS+=(--fixture-manifest "${E2E_FIXTURE_MANIFEST}")
  fi
  if [[ "${E2E_DISABLE_CACHE}" == "1" ]]; then
    E2E_ARGS+=(--no-cache)
  fi
  bash "${SCRIPT_DIR}/run_e2e_pass.sh" "${E2E_ARGS[@]}" ${FORWARD_ARGS[@]+"${FORWARD_ARGS[@]}"}
}

run_hardening_suites() {
  if [[ "${RUN_HARDENING}" == "1" ]]; then
    echo "Running phase 29 verification hardening suites"
    HARDENING_ARGS=(--profile "${PROFILE}")
    DIAGNOSTICS_HARDENING=0
    PROJECT_WORKSPACE_HARDENING=0
    REGRESSION_HARDENING_ARGS=()
    FUZZ_PROPERTY_HARDENING_ARGS=()
    IFS=',' read -r -a HARDENING_SUITE_ARRAY <<< "${HARDENING_SUITES}"
    for suite in "${HARDENING_SUITE_ARRAY[@]}"; do
      if [[ -n "${suite}" ]]; then
        if [[ "${suite}" == "diagnostics" ]]; then
          DIAGNOSTICS_HARDENING=1
        elif [[ "${suite}" == "project" ]]; then
          PROJECT_WORKSPACE_HARDENING=1
        elif [[ "${suite}" == "fixedbugs" || "${suite}" == "crashes" ]]; then
          REGRESSION_HARDENING_ARGS+=(--suite "${suite}")
        elif [[ "${suite}" == "property" || "${suite}" == "fuzz-smoke" ]]; then
          FUZZ_PROPERTY_HARDENING_ARGS+=(--suite "${suite}")
        else
          HARDENING_ARGS+=(--suite "${suite}")
        fi
      fi
    done
    if [[ "${DIAGNOSTICS_HARDENING}" == "1" ]]; then
      uv run --project "${SCRIPT_DIR}/../verification" --locked \
        python -m sifr_verify areas run --area diagnostics --suite baselines --hardening-summary
    fi
    if [[ "${PROJECT_WORKSPACE_HARDENING}" == "1" ]]; then
      uv run --project "${SCRIPT_DIR}/../verification" --locked \
        python -m sifr_verify areas run --area project_workspace --suite baselines --hardening-summary
    fi
    if [[ "${#REGRESSION_HARDENING_ARGS[@]}" -gt 0 ]]; then
      uv run --project "${SCRIPT_DIR}/../verification" --locked \
        python -m sifr_verify areas run --area regression "${REGRESSION_HARDENING_ARGS[@]}" --hardening-summary
    fi
    if [[ "${#FUZZ_PROPERTY_HARDENING_ARGS[@]}" -gt 0 ]]; then
      uv run --project "${SCRIPT_DIR}/../verification" --locked \
        python -m sifr_verify areas run --area fuzz_property "${FUZZ_PROPERTY_HARDENING_ARGS[@]}" --hardening-summary
    fi
    if [[ "${#HARDENING_ARGS[@]}" -gt 2 ]]; then
      python3 "${SCRIPT_DIR}/run_verification_hardening.py" "${HARDENING_ARGS[@]}"
    fi
  fi
}

run_extra_e2e_checks() {
  if [[ "${RUN_E2E_REPORT_DETERMINISM}" == "1" ]]; then
    echo "Running e2e report determinism check"
    bash "${SCRIPT_DIR}/check_e2e_report_determinism.sh" --profile "${PROFILE}"
  fi

  if [[ "${RUN_E2E_SEQUENTIAL_PARALLEL_EQUIVALENCE}" == "1" ]]; then
    echo "Running e2e sequential-vs-parallel equivalence check"
    bash "${SCRIPT_DIR}/check_e2e_sequential_parallel_equivalence.sh" --profile "${PROFILE}"
  fi
}

timed_step core_guardrails run_core_guardrails
timed_step diagnostic_contracts run_diagnostic_contracts
timed_step frontend_syntax_guardrails run_frontend_syntax_guardrails
timed_step developer_tooling_checks run_developer_tooling_checks
timed_step performance_budget_checks run_performance_budget_checks
timed_step verification_hardening_self_tests run_verification_hardening_self_tests
timed_step verification_runner_foundation run_verification_runner_foundation_checks
timed_step distribution_validation run_distribution_checks

if [[ "${GENERATED_CODE_QUALITY_MODE}" != "none" ]]; then
  timed_step generated_code_quality_checks run_generated_code_quality_checks
fi

timed_step crate_tests run_crate_tests

timed_step validation_contract_matrix run_validation_contract_suites

timed_step platform_golden run_platform_golden_suite

timed_step e2e_pass_suite run_e2e_pass_suite

timed_step verification_hardening_suites run_hardening_suites
timed_step extra_e2e_checks run_extra_e2e_checks
