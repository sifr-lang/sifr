#!/usr/bin/env bash

set -euo pipefail

RUNS="${SIFR_E2E_BENCH_RUNS:-7}"
P50_MAX="${SIFR_E2E_BENCH_P50_MS:-90000}"
P95_MAX="${SIFR_E2E_BENCH_P95_MS:-110000}"
CV_MAX="${SIFR_E2E_BENCH_CV_MAX:-0.15}"
TIMING_FILE="target/sifr_e2e_bench_ms.txt"

run_once() {
  local label="$1"
  local start_ms
  local end_ms
  local elapsed_ms

  start_ms="$(date +%s%3N)"
  if ! cargo test -p sifr --test e2e test_e2e_pass -- --nocapture >"${TIMING_FILE}.${label}.log" 2>&1; then
    echo "test_e2e_pass failed in sample ${label}" >&2
    cat "${TIMING_FILE}.${label}.log" >&2
    exit 1
  fi
  end_ms="$(date +%s%3N)"
  elapsed_ms=$((end_ms - start_ms))
  echo "$elapsed_ms"
}

echo "== e2e throughput benchmark =="
echo "samples: ${RUNS} (warm cache protocol; first warm run discarded)"
mkdir -p "$(dirname "${TIMING_FILE}")"

echo "running warm cache warmup"
run_once warm > /dev/null

rm -f "${TIMING_FILE}"
for idx in $(seq 1 "${RUNS}"); do
  sample_ms="$(run_once "${idx}")"
  echo "${sample_ms}" >> "${TIMING_FILE}"
  echo "run ${idx}: ${sample_ms}ms"
done

python - <<'PY' "${TIMING_FILE}" "${RUNS}" "${P50_MAX}" "${P95_MAX}" "${CV_MAX}"
import math
import statistics
import sys
from pathlib import Path

timing_path = Path(sys.argv[1])
expected_samples = int(sys.argv[2])
p50_max = float(sys.argv[3])
p95_max = float(sys.argv[4])
cv_max = float(sys.argv[5])

values = [
    int(line.strip())
    for line in timing_path.read_text().splitlines()
    if line.strip()
]

if len(values) != expected_samples:
    print(f"expected {expected_samples} benchmark samples, got {len(values)}")
    sys.exit(1)

values.sort()
n = len(values)

def percentile(values, q):
    idx = max(0, math.ceil(n * q) - 1)
    idx = min(n - 1, idx)
    return values[idx]

p50 = percentile(values, 0.50)
p95 = percentile(values, 0.95)
mean = sum(values) / n
cv = (statistics.pstdev(values) / mean) if mean else 0.0

print(f"p50_ms={p50}")
print(f"p95_ms={p95}")
print(f"coefficient_of_variation={cv:.3f}")

print("samples_ms=", ",".join(str(value) for value in values))

if p50 > p50_max or p95 > p95_max or cv > cv_max:
    print(
        "throughput gate failed: "
        f"p50={p50} (max {p50_max}), "
        f"p95={p95} (max {p95_max}), "
        f"cv={cv:.3f} (max {cv_max})"
    )
    sys.exit(1)

print("throughput gate passed")
PY
