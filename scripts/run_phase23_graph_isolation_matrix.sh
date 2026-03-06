#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."
cd "${REPO_ROOT}"

PHASE23_DIR="demos/m23_5_graph_isolation_regression_matrix_demo"
SINGLE_MAIN="${PHASE23_DIR}/single_file/main.sifr"
MULTI_MAIN="${PHASE23_DIR}/main.sifr"
CLOSURE_NEG_MAIN="${PHASE23_DIR}/negative_cases/reachable_parse_error/main.sifr"
CYCLE_MAIN="${PHASE23_DIR}/negative_cases/module_cycle/main.sifr"
PARITY_NEG_TEST_DIR="demos/m23_3_project_test_discovery_parity_contract_demo/negative_cases/reachable_parse_error"
PARALLEL_RUN_A="demos/m23_4_invocation_scoped_temp_workspace_isolation_demo/parallel_runs/a/main.sifr"
PARALLEL_RUN_B="demos/m23_4_invocation_scoped_temp_workspace_isolation_demo/parallel_runs/b/main.sifr"
PARALLEL_TEST_A="demos/m23_4_invocation_scoped_temp_workspace_isolation_demo/parallel_tests/a"
PARALLEL_TEST_B="demos/m23_4_invocation_scoped_temp_workspace_isolation_demo/parallel_tests/b"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

run_capture() {
  local label="$1"
  local expected_exit="$2"
  shift 2

  local out_file="${TMP_DIR}/${label}.out"
  local err_file="${TMP_DIR}/${label}.err"

  set +e
  "$@" >"${out_file}" 2>"${err_file}"
  local exit_code=$?
  set -e

  if [[ "${exit_code}" -ne "${expected_exit}" ]]; then
    echo "[phase23-matrix] ${label}: expected exit ${expected_exit}, got ${exit_code}" >&2
    echo "[phase23-matrix] command: $*" >&2
    echo "[phase23-matrix] stderr:" >&2
    cat "${err_file}" >&2
    exit 1
  fi
}

echo "Running phase 23 graph and isolation regression matrix"

echo "  row=single_file_layout_smoke"
run_capture single_check 0 cargo run -q -p sifr -- check "${SINGLE_MAIN}"
run_capture single_build 0 cargo run -q -p sifr -- build "${SINGLE_MAIN}" --output "${TMP_DIR}/single_build"
run_capture single_run 0 cargo run -q -p sifr -- run "${SINGLE_MAIN}"
if ! grep -Fq "single-file-m23_5" "${TMP_DIR}/single_run.out"; then
  echo "[phase23-matrix] single_file run output missing expected marker" >&2
  cat "${TMP_DIR}/single_run.out" >&2
  exit 1
fi

echo "  row=multi_file_import_closure_and_test"
run_capture multi_check 0 cargo run -q -p sifr -- check "${MULTI_MAIN}"
run_capture multi_build 0 cargo run -q -p sifr -- build "${MULTI_MAIN}" --output "${TMP_DIR}/multi_build"
run_capture multi_run 0 cargo run -q -p sifr -- run "${MULTI_MAIN}"
run_capture multi_test 0 cargo run -q -p sifr -- test "${PHASE23_DIR}"
if ! grep -Fq "m23_5 graph and isolation regression matrix demo:" "${TMP_DIR}/multi_run.out"; then
  echo "[phase23-matrix] multi_file run output missing demo header" >&2
  cat "${TMP_DIR}/multi_run.out" >&2
  exit 1
fi
if ! grep -Fq "55" "${TMP_DIR}/multi_run.out"; then
  echo "[phase23-matrix] multi_file run output missing expected numeric value" >&2
  cat "${TMP_DIR}/multi_run.out" >&2
  exit 1
fi

echo "  row=reachable_parse_error_contract"
run_capture closure_neg_check 1 cargo run -q -p sifr -- check "${CLOSURE_NEG_MAIN}"
run_capture closure_neg_build 1 cargo run -q -p sifr -- build "${CLOSURE_NEG_MAIN}" --output "${TMP_DIR}/closure_neg_build"
run_capture closure_neg_run 1 cargo run -q -p sifr -- run "${CLOSURE_NEG_MAIN}"
run_capture closure_neg_test 1 cargo run -q -p sifr -- test "${PARITY_NEG_TEST_DIR}"
if ! grep -Fq "[helper] failed to parse" "${TMP_DIR}/closure_neg_check.err"; then
  echo "[phase23-matrix] closure negative check missing helper parse diagnostic" >&2
  cat "${TMP_DIR}/closure_neg_check.err" >&2
  exit 1
fi
if ! grep -Fq "helper.sifr" "${TMP_DIR}/closure_neg_test.err"; then
  echo "[phase23-matrix] closure negative test missing helper parse diagnostic" >&2
  cat "${TMP_DIR}/closure_neg_test.err" >&2
  exit 1
fi

echo "  row=cycle_diagnostic_stability"
run_capture cycle_check_first 1 cargo run -q -p sifr -- check "${CYCLE_MAIN}"
run_capture cycle_check_second 1 cargo run -q -p sifr -- check "${CYCLE_MAIN}"
if ! diff -u "${TMP_DIR}/cycle_check_first.err" "${TMP_DIR}/cycle_check_second.err" >/dev/null; then
  echo "[phase23-matrix] cycle diagnostics changed between repeated runs" >&2
  diff -u "${TMP_DIR}/cycle_check_first.err" "${TMP_DIR}/cycle_check_second.err" >&2 || true
  exit 1
fi
EXPECTED_CYCLE="module dependency cycle detected: a -> b -> c -> a; import chain: a imports b, b imports c, c imports a"
if ! grep -Fq "${EXPECTED_CYCLE}" "${TMP_DIR}/cycle_check_first.err"; then
  echo "[phase23-matrix] cycle diagnostics missing canonical path" >&2
  cat "${TMP_DIR}/cycle_check_first.err" >&2
  exit 1
fi

echo "  row=parallel_invocation_isolation"
(
  cargo run -q -p sifr -- run "${PARALLEL_RUN_A}" >"${TMP_DIR}/parallel_run_a.out" 2>"${TMP_DIR}/parallel_run_a.err"
  echo $? >"${TMP_DIR}/parallel_run_a.exit"
) &
run_a_pid=$!
(
  cargo run -q -p sifr -- run "${PARALLEL_RUN_B}" >"${TMP_DIR}/parallel_run_b.out" 2>"${TMP_DIR}/parallel_run_b.err"
  echo $? >"${TMP_DIR}/parallel_run_b.exit"
) &
run_b_pid=$!
wait "${run_a_pid}"
wait "${run_b_pid}"
if [[ "$(cat "${TMP_DIR}/parallel_run_a.exit")" -ne 0 ]]; then
  echo "[phase23-matrix] parallel run A failed" >&2
  cat "${TMP_DIR}/parallel_run_a.err" >&2
  exit 1
fi
if [[ "$(cat "${TMP_DIR}/parallel_run_b.exit")" -ne 0 ]]; then
  echo "[phase23-matrix] parallel run B failed" >&2
  cat "${TMP_DIR}/parallel_run_b.err" >&2
  exit 1
fi
if ! grep -Fq "parallel-run-a" "${TMP_DIR}/parallel_run_a.out"; then
  echo "[phase23-matrix] parallel run A output mismatch" >&2
  cat "${TMP_DIR}/parallel_run_a.out" >&2
  exit 1
fi
if ! grep -Fq "parallel-run-b" "${TMP_DIR}/parallel_run_b.out"; then
  echo "[phase23-matrix] parallel run B output mismatch" >&2
  cat "${TMP_DIR}/parallel_run_b.out" >&2
  exit 1
fi
(
  cargo run -q -p sifr -- test "${PARALLEL_TEST_A}" >"${TMP_DIR}/parallel_test_a.out" 2>"${TMP_DIR}/parallel_test_a.err"
  echo $? >"${TMP_DIR}/parallel_test_a.exit"
) &
test_a_pid=$!
(
  cargo run -q -p sifr -- test "${PARALLEL_TEST_B}" >"${TMP_DIR}/parallel_test_b.out" 2>"${TMP_DIR}/parallel_test_b.err"
  echo $? >"${TMP_DIR}/parallel_test_b.exit"
) &
test_b_pid=$!
wait "${test_a_pid}"
wait "${test_b_pid}"
if [[ "$(cat "${TMP_DIR}/parallel_test_a.exit")" -ne 0 ]]; then
  echo "[phase23-matrix] parallel test A failed" >&2
  cat "${TMP_DIR}/parallel_test_a.err" >&2
  exit 1
fi
if [[ "$(cat "${TMP_DIR}/parallel_test_b.exit")" -ne 0 ]]; then
  echo "[phase23-matrix] parallel test B failed" >&2
  cat "${TMP_DIR}/parallel_test_b.err" >&2
  exit 1
fi

echo "Phase 23 graph and isolation regression matrix: PASS"
