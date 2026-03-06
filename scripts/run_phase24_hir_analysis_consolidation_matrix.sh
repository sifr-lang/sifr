#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."
cd "${REPO_ROOT}"

M24_2_MAIN="demos/m24_2_semantic_query_layer_standardization_demo/main.sifr"
M24_3_MAIN="demos/m24_3_control_flow_effect_query_unification_demo/main.sifr"
M24_4_MAIN="demos/m24_4_analysis_emission_boundary_hardening_demo/main.sifr"
M24_5_DIR="demos/m24_5_analysis_consolidation_regression_matrix_demo"
M24_5_MAIN="${M24_5_DIR}/main.sifr"
M24_5_NEG_MAIN="${M24_5_DIR}/negative_cases/mixed_block_type_error/main.sifr"

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
    echo "[phase24-matrix] ${label}: expected exit ${expected_exit}, got ${exit_code}" >&2
    echo "[phase24-matrix] command: $*" >&2
    echo "[phase24-matrix] stderr:" >&2
    cat "${err_file}" >&2
    exit 1
  fi
}

echo "Running phase 24 HIR analysis consolidation regression matrix"

echo "  row=nested_conditionals_call_detection"
run_capture m24_2_run 0 cargo run -q -p sifr -- run "${M24_2_MAIN}"
if ! grep -Fq "m24_2 semantic query layer standardization demo:" "${TMP_DIR}/m24_2_run.out"; then
  echo "[phase24-matrix] m24_2 run output missing expected demo header" >&2
  cat "${TMP_DIR}/m24_2_run.out" >&2
  exit 1
fi

echo "  row=control_flow_effect_query_paths"
run_capture m24_3_run 0 cargo run -q -p sifr -- run "${M24_3_MAIN}"
if ! grep -Fq "99" "${TMP_DIR}/m24_3_run.out"; then
  echo "[phase24-matrix] m24_3 run output missing expected fallback value" >&2
  cat "${TMP_DIR}/m24_3_run.out" >&2
  exit 1
fi

echo "  row=analysis_boundary_consumers"
run_capture m24_4_run 0 cargo run -q -p sifr -- run "${M24_4_MAIN}"
if ! grep -Fq "33" "${TMP_DIR}/m24_4_run.out"; then
  echo "[phase24-matrix] m24_4 run output missing expected value" >&2
  cat "${TMP_DIR}/m24_4_run.out" >&2
  exit 1
fi

echo "  row=matrix_fixture_full_modes"
run_capture m24_5_check 0 cargo run -q -p sifr -- check "${M24_5_MAIN}"
run_capture m24_5_build 0 cargo run -q -p sifr -- build "${M24_5_MAIN}" --output "${TMP_DIR}/m24_5_build"
run_capture m24_5_run 0 cargo run -q -p sifr -- run "${M24_5_MAIN}"
run_capture m24_5_test 0 cargo run -q -p sifr -- test "${M24_5_DIR}"
if ! grep -Fq "m24_5 analysis consolidation regression matrix demo:" "${TMP_DIR}/m24_5_run.out"; then
  echo "[phase24-matrix] m24_5 run output missing expected demo header" >&2
  cat "${TMP_DIR}/m24_5_run.out" >&2
  exit 1
fi
if ! grep -Fq "20" "${TMP_DIR}/m24_5_run.out" || ! grep -Fq "45" "${TMP_DIR}/m24_5_run.out"; then
  echo "[phase24-matrix] m24_5 run output missing expected numeric values" >&2
  cat "${TMP_DIR}/m24_5_run.out" >&2
  exit 1
fi

echo "  row=negative_mixed_block_parity"
run_capture m24_5_neg_check 1 cargo run -q -p sifr -- check "${M24_5_NEG_MAIN}"
run_capture m24_5_neg_build 1 cargo run -q -p sifr -- build "${M24_5_NEG_MAIN}" --output "${TMP_DIR}/m24_5_neg_build"
run_capture m24_5_neg_run 1 cargo run -q -p sifr -- run "${M24_5_NEG_MAIN}"
if ! diff -u "${TMP_DIR}/m24_5_neg_check.err" "${TMP_DIR}/m24_5_neg_build.err" >/dev/null; then
  echo "[phase24-matrix] negative check/build diagnostics diverged" >&2
  diff -u "${TMP_DIR}/m24_5_neg_check.err" "${TMP_DIR}/m24_5_neg_build.err" >&2 || true
  exit 1
fi
if ! diff -u "${TMP_DIR}/m24_5_neg_check.err" "${TMP_DIR}/m24_5_neg_run.err" >/dev/null; then
  echo "[phase24-matrix] negative check/run diagnostics diverged" >&2
  diff -u "${TMP_DIR}/m24_5_neg_check.err" "${TMP_DIR}/m24_5_neg_run.err" >&2 || true
  exit 1
fi
EXPECTED_NEG="type error: return type mismatch: expected 'int', got 'str'"
if ! grep -Fq "${EXPECTED_NEG}" "${TMP_DIR}/m24_5_neg_check.err"; then
  echo "[phase24-matrix] negative diagnostics missing expected type error" >&2
  cat "${TMP_DIR}/m24_5_neg_check.err" >&2
  exit 1
fi

echo "  row=negative_diagnostic_stability"
run_capture m24_5_neg_check_first 1 cargo run -q -p sifr -- check "${M24_5_NEG_MAIN}"
run_capture m24_5_neg_check_second 1 cargo run -q -p sifr -- check "${M24_5_NEG_MAIN}"
if ! diff -u "${TMP_DIR}/m24_5_neg_check_first.err" "${TMP_DIR}/m24_5_neg_check_second.err" >/dev/null; then
  echo "[phase24-matrix] negative diagnostics changed across repeated runs" >&2
  diff -u "${TMP_DIR}/m24_5_neg_check_first.err" "${TMP_DIR}/m24_5_neg_check_second.err" >&2 || true
  exit 1
fi

echo "Phase 24 HIR analysis consolidation regression matrix: PASS"
