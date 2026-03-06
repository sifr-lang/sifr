#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."
cd "${REPO_ROOT}"

M25_3_MAIN="demos/m25_3_canonical_flow_truth_queries_demo/main.sifr"
M25_4_MAIN="demos/m25_4_diagnostics_and_consumer_integration_demo/main.sifr"
M25_5_DIR="demos/m25_5_cfg_flow_activation_regression_matrix_demo"
M25_5_MAIN="${M25_5_DIR}/main.sifr"
M25_5_NEG_MAIN="${M25_5_DIR}/negative_cases/reachable_type_error/main.sifr"

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
    echo "[phase25-matrix] ${label}: expected exit ${expected_exit}, got ${exit_code}" >&2
    echo "[phase25-matrix] command: $*" >&2
    echo "[phase25-matrix] stderr:" >&2
    cat "${err_file}" >&2
    exit 1
  fi
}

echo "Running phase 25 CFG/flow activation regression matrix"

echo "  row=canonical_query_paths"
run_capture m25_3_run 0 cargo run -q -p sifr -- run "${M25_3_MAIN}"
if ! grep -Fq "m25_3 canonical flow truth queries demo:" "${TMP_DIR}/m25_3_run.out"; then
  echo "[phase25-matrix] m25_3 output missing expected header" >&2
  cat "${TMP_DIR}/m25_3_run.out" >&2
  exit 1
fi

echo "  row=diagnostics_consumer_cfg_integration"
run_capture m25_4_run 0 cargo run -q -p sifr -- run "${M25_4_MAIN}"
if ! grep -Fq "m25_4 diagnostics and consumer integration demo:" "${TMP_DIR}/m25_4_run.out"; then
  echo "[phase25-matrix] m25_4 output missing expected header" >&2
  cat "${TMP_DIR}/m25_4_run.out" >&2
  exit 1
fi
if ! grep -Fq "unreachable statement at block index 2 was ignored" "${TMP_DIR}/m25_4_run.err"; then
  echo "[phase25-matrix] m25_4 warning missing expected unreachable diagnostic" >&2
  cat "${TMP_DIR}/m25_4_run.err" >&2
  exit 1
fi

echo "  row=matrix_fixture_full_modes"
run_capture m25_5_check 0 cargo run -q -p sifr -- check "${M25_5_MAIN}"
run_capture m25_5_build 0 cargo run -q -p sifr -- build "${M25_5_MAIN}" --output "${TMP_DIR}/m25_5_build"
run_capture m25_5_run 0 cargo run -q -p sifr -- run "${M25_5_MAIN}"
run_capture m25_5_test 0 cargo run -q -p sifr -- test "${M25_5_DIR}"
if ! grep -Fq "m25_5 cfg flow activation regression matrix demo:" "${TMP_DIR}/m25_5_run.out"; then
  echo "[phase25-matrix] m25_5 output missing expected header" >&2
  cat "${TMP_DIR}/m25_5_run.out" >&2
  exit 1
fi
if ! grep -Fq "8" "${TMP_DIR}/m25_5_run.out" || ! grep -Fq "42" "${TMP_DIR}/m25_5_run.out" || ! grep -Fq "9" "${TMP_DIR}/m25_5_run.out"; then
  echo "[phase25-matrix] m25_5 output missing expected regression values" >&2
  cat "${TMP_DIR}/m25_5_run.out" >&2
  exit 1
fi

echo "  row=cfg_shape_and_query_repeat_determinism"
run_capture m25_cfg_repeat 0 cargo test -q -p sifr_hir cfg::tests::cfg_repeat_run_matrix_is_deterministic -- --nocapture

echo "  row=negative_reachable_type_error_parity"
run_capture m25_5_neg_check 1 cargo run -q -p sifr -- check "${M25_5_NEG_MAIN}"
run_capture m25_5_neg_build 1 cargo run -q -p sifr -- build "${M25_5_NEG_MAIN}" --output "${TMP_DIR}/m25_5_neg_build"
run_capture m25_5_neg_run 1 cargo run -q -p sifr -- run "${M25_5_NEG_MAIN}"
if ! diff -u "${TMP_DIR}/m25_5_neg_check.err" "${TMP_DIR}/m25_5_neg_build.err" >/dev/null; then
  echo "[phase25-matrix] negative check/build diagnostics diverged" >&2
  diff -u "${TMP_DIR}/m25_5_neg_check.err" "${TMP_DIR}/m25_5_neg_build.err" >&2 || true
  exit 1
fi
if ! diff -u "${TMP_DIR}/m25_5_neg_check.err" "${TMP_DIR}/m25_5_neg_run.err" >/dev/null; then
  echo "[phase25-matrix] negative check/run diagnostics diverged" >&2
  diff -u "${TMP_DIR}/m25_5_neg_check.err" "${TMP_DIR}/m25_5_neg_run.err" >&2 || true
  exit 1
fi
EXPECTED_NEG="type error: return type mismatch: expected 'int', got 'str'"
if ! grep -Fq "${EXPECTED_NEG}" "${TMP_DIR}/m25_5_neg_check.err"; then
  echo "[phase25-matrix] negative diagnostics missing expected type error" >&2
  cat "${TMP_DIR}/m25_5_neg_check.err" >&2
  exit 1
fi

echo "  row=negative_diagnostic_stability"
run_capture m25_5_neg_check_first 1 cargo run -q -p sifr -- check "${M25_5_NEG_MAIN}"
run_capture m25_5_neg_check_second 1 cargo run -q -p sifr -- check "${M25_5_NEG_MAIN}"
if ! diff -u "${TMP_DIR}/m25_5_neg_check_first.err" "${TMP_DIR}/m25_5_neg_check_second.err" >/dev/null; then
  echo "[phase25-matrix] negative diagnostics changed across repeated runs" >&2
  diff -u "${TMP_DIR}/m25_5_neg_check_first.err" "${TMP_DIR}/m25_5_neg_check_second.err" >&2 || true
  exit 1
fi

echo "Phase 25 CFG/flow activation regression matrix: PASS"
