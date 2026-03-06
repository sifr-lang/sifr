#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."
cd "${REPO_ROOT}"

POSITIVE_DIR="demos/m22_4_parity_regression_matrix_demo"
POSITIVE_MAIN="${POSITIVE_DIR}/main.sifr"
NEGATIVE_DIR="demos/m22_4_parity_regression_matrix_demo/negative_cases/type_error_project"
NEGATIVE_MAIN="${NEGATIVE_DIR}/main.sifr"

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
    echo "[parity-matrix] ${label}: expected exit ${expected_exit}, got ${exit_code}" >&2
    echo "[parity-matrix] command: $*" >&2
    echo "[parity-matrix] stderr:" >&2
    cat "${err_file}" >&2
    exit 1
  fi
}

echo "Running frontend mode parity matrix"

echo "  row=positive_project"
run_capture positive_check 0 cargo run -q -p sifr -- check "${POSITIVE_MAIN}"
run_capture positive_build 0 cargo run -q -p sifr -- build "${POSITIVE_MAIN}" --output "${TMP_DIR}/positive_build"
run_capture positive_run 0 cargo run -q -p sifr -- run "${POSITIVE_MAIN}"
run_capture positive_test 0 cargo run -q -p sifr -- test "${POSITIVE_DIR}"

if ! grep -q "m22_4 parity regression matrix demo:" "${TMP_DIR}/positive_run.out"; then
  echo "[parity-matrix] positive_run missing demo header" >&2
  cat "${TMP_DIR}/positive_run.out" >&2
  exit 1
fi
if ! grep -q "8" "${TMP_DIR}/positive_run.out"; then
  echo "[parity-matrix] positive_run missing expected numeric output" >&2
  cat "${TMP_DIR}/positive_run.out" >&2
  exit 1
fi

echo "  row=negative_project_type_error"
run_capture negative_check 1 cargo run -q -p sifr -- check "${NEGATIVE_MAIN}"
run_capture negative_build 1 cargo run -q -p sifr -- build "${NEGATIVE_MAIN}" --output "${TMP_DIR}/negative_build"
run_capture negative_run 1 cargo run -q -p sifr -- run "${NEGATIVE_MAIN}"
run_capture negative_test 1 cargo run -q -p sifr -- test "${NEGATIVE_DIR}"

if ! diff -u "${TMP_DIR}/negative_check.err" "${TMP_DIR}/negative_build.err" >/dev/null; then
  echo "[parity-matrix] check/build diagnostics diverged for equivalent frontend failure" >&2
  diff -u "${TMP_DIR}/negative_check.err" "${TMP_DIR}/negative_build.err" >&2 || true
  exit 1
fi
if ! diff -u "${TMP_DIR}/negative_check.err" "${TMP_DIR}/negative_run.err" >/dev/null; then
  echo "[parity-matrix] check/run diagnostics diverged for equivalent frontend failure" >&2
  diff -u "${TMP_DIR}/negative_check.err" "${TMP_DIR}/negative_run.err" >&2 || true
  exit 1
fi

EXPECTED_NEG="type error: [helper] return type mismatch: expected 'int', got 'str'"
if ! grep -Fq "${EXPECTED_NEG}" "${TMP_DIR}/negative_check.err"; then
  echo "[parity-matrix] negative check diagnostics missing expected frontend message" >&2
  cat "${TMP_DIR}/negative_check.err" >&2
  exit 1
fi
if ! grep -Fq "${EXPECTED_NEG}" "${TMP_DIR}/negative_test.err"; then
  echo "[parity-matrix] negative test diagnostics missing expected frontend message" >&2
  cat "${TMP_DIR}/negative_test.err" >&2
  exit 1
fi

echo "Frontend mode parity matrix: PASS"
