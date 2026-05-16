

I have completed my thorough review. Here is my assessment:

---

## **SATISFIED** for m35.2.

---

### Blockers: None.

### Detailed Review Findings

**1. Budget Derivation Rules — ENCODED**

`verification/performance/budgets.json` contains a `derivation` section that documents both rule sets:

- **Command default** (median: `max(baseline * 1.10, baseline + 25ms)`, p95: `max(baseline * 1.15, baseline + 50ms)`, RSS: `max(baseline * 1.10, baseline + 32MiB)`) is explicitly defined and applied to all command-group cases (e.g., `check-single-file-001-arithmetic`: baseline median 1212.854ms → threshold 1334.139ms = `max(1334.1394, 1237.854)`).
- **Frontend-query/edit-loop stricter rule** (median: `max(baseline * 1.05, baseline + 2ms)`, p95: `max(baseline * 1.10, baseline + 5ms)`, same RSS floor) is documented and applied to `incremental-*` and `interactive-tooling-*` cases (e.g., `perf.interactive.warm_diagnostics_query`: baseline median 0.16ms → threshold 2.16ms = `max(0.168, 2.16)` — correctly using the +2ms floor, not the +25ms command floor).

The `policy` field in each budget entry correctly tags cases as `command-default` or `frontend-query-edit-loop`. The derivation is sound for both groups.

**2. Default Rules Encoded — CONFIRMED**

All 45 manifest cases have corresponding budget entries. Three cases (`perf.incremental.unchanged_file_update`, `perf.interactive.warm_diagnostics_query`, `perf.interactive.unchanged_file_update`) carry `cache.min_hits: 2300` enforcing the edit-loop cache-hit floor from the policy doc. The `budget_median_regression_result.json` and `budget_p95_regression_result.json` seeds each inject one regression while leaving all other entries at clean baseline values, correctly testing regression detection without cross-contamination.

**3. `check_budgets` Rejection Coverage — COMPLETE**

| Failure type | How rejected | Actionable diagnostic |
|---|---|---|
| Median regression | `compare_result` → `format_failure` with `median_ms regression` | ✓ |
| p95 regression | `compare_result` → `format_failure` with `p95_ms regression` | ✓ |
| RSS regression | `compare_result` → `format_failure` with `peak_rss_bytes regression` | ✓ |
| Timeout | `compare_result` returns immediately with `timed_out: true` | ✓ |
| Missing result | `check_budgets` loop detects `result is None` → `"missing result"` | ✓ |
| Unknown benchmark id | `validate_results_shape` checks `result_id in cases` → `"unknown benchmark id"` | ✓ |
| Malformed metric | `validate_results_shape` loops required fields → `"missing metric"` | ✓ |
| Expired waiver | `validate_waivers` compares `expires < today` → `"expired"` | ✓ |
| Malformed waiver (empty owner) | `validate_waivers` calls `require_string` → `"owner"` | ✓ |
| Correctness waiver (non-performance override) | `validate_waivers` checks `override.keys ⊆ ALLOWED_WAIVER_OVERRIDE_KEYS` → `"non-performance"` | ✓ |

The self-test (`--self-test`) covers all 9 negative seeds plus the positive active-waiver suppressing median regression. The `format_failure` output includes case id, budget id, metric name, measured value, threshold, and `waiver_status` — fully actionable.

**4. Waiver Non-Suppression Enforcement — CORRECT**

`ALLOWED_WAIVER_OVERRIDE_KEYS = {"median_ms", "p95_ms", "peak_rss_bytes", "cache_hits"}` — explicitly excludes `timeout`, `cache_misses`, and any correctness/non-performance fields. The `correctness_waiver.json` seed uses `override: {"timeout": true}` which the `validate_waivers` logic rejects at schema-validation time (before any budget comparison runs), so correctness failures are blocked at the door, not at the comparison step.

In `compare_result`:
- `timed_out` failures pass `waiverable=False` → `format_failure` produces `not_waiverable` status.
- `cache_misses` violations pass `waiverable=False` → same.
- Median/p95/RSS and `cache_hits` violations pass `waiverable=True` (default) → `has_waiver()` is consulted.

The split-brain protection is at two levels: schema validation rejects non-performance override keys, and runtime `compare_result` never consults waivers for timeouts or cache-miss violations.

**5. Positive Path — PASSES**

`check_budgets` against `baselines.json` (all 45 clean entries) passes without errors. The `run_self_test` internal call to `check_budgets(manifest, budgets, active_waiver, median_regression)` proves that a valid waiver does suppress a seeded median regression — the full waiver lifecycle is exercised.

**6. Documentation — COMPLETE**

`internal_docs/performance_budgets.md` covers:
- File inventory (manifest, baselines, budgets, waivers, runners, checker).
- Threshold derivation formulas for both command and frontend-query/edit-loop groups.
- Explicit statement that timeouts, missing/malformed results, unknown ids, and cache-miss regressions are hard failures.
- Waiver override field enumeration with the same exclusion set as `ALLOWED_WAIVER_OVERRIDE_KEYS`.

---

### Non-Blocking Follow-Ups (not required for m35.2 completion)

1. **Corpus floor**: The phase contract requires "at least 3 negative budget/waiver seeds". Current count is 9 (`budget_*` × 7 + `*waiver*` × 4 = 11, with overlap). More than sufficient, but no action needed.

2. **Cache miss waiverability**: The phase doc says "cache-miss regressions are hard failures" and the code correctly implements `waiverable=False`. The `ALLOWED_WAIVER_OVERRIDE_KEYS` includes `cache_hits` (for the minimum-hit floor) but not `cache_misses`. This is the correct design but worth noting: the 3 cache-hit-floor budgets enforce a minimum, and cache-miss violations are unconditional hard failures — the two cache dimensions are asymmetrically waiverable.

3. **M35.3 dependency**: `scripts/run_all_tests.sh` integration is still pending (m35.3 scope). The `check_budgets.py` standalone gate is complete; the CI lane integration is a separate milestone.

---

**Verdict**: `m35.2` is fully implemented against the contract. `check_budgets.py` accepts clean baselines, rejects all seeded failure classes with actionable diagnostics, validates waiver lifecycle end-to-end, and correctly prevents waivers from suppressing timeout/correctness/cache-staleness failures.
