## Review (Pass 2)

### 1. Blockers

None. All Pass 1 follow-ups landed correctly:

- **p95 skip rationale comment** at `check_budgets.py:25-27`. Reads cleanly.
- **CLI audit note** at `check_budgets.py:319-330`. Threaded via `report_skipped_p95=True` only from `main()` (line 58), so direct CLI runs *and* `sifr_verify areas run` (which shells out to `check_budgets.py` via `runner.py:173,199`) both surface it; in-process self-tests stay quiet — correct separation.
- **Sample-count / samples_ms validation** at `check_budgets.py:243-250`. The bool exclusion is right (`isinstance(True, int)` is True in Python), positive-integer guard, list-shape, length match, and numeric-non-bool sample contents all covered. I verified `baselines.json` (65 entries) and every result-bearing negative seed already satisfy the new shape — no false positives.
- **Boundary coverage** at `check_budgets.py:378-379`. `spike_p95_below_threshold` forces 19 samples and passes; `spike_p95_at_threshold` forces 20 and fails. Both branches of `should_enforce_p95` are now locked in.
- **Median-still-enforced on 5-sample case** at `check_budgets.py:377` — uses `check-project-004-project-graph` (confirmed `sample_count: 5` in baselines), so it proves median is independent of the new gate.
- **Ordering of new validation vs malformed seed:** `budget_malformed_result.json` is missing `p95_ms` but has valid `sample_count: 5` / matching `samples_ms`, so it still fails on the existing `missing metric` diagnostic — refactor didn't regress that path.

### 2. Non-blocking suggestions

- **Redundant p95 spike test.** `spike_twenty_sample_p95` (line 367) and `spike_p95_at_threshold` (line 379) produce the same effective result — the baseline already has `sample_count: 20` for `interactive-tooling-foundation-002-warm-diagnostics-query`, so `force_sample_count(...,20)` is a no-op. Either drop line 367 (the boundary test subsumes it) or fold the boundary tests next to it for grouping.
- **Orphan seed file.** `verification/areas/performance/negative_seeds/budget_p95_regression_result.json` is no longer referenced from anywhere. Pass 1 flagged this as harmless; still worth deleting in this PR to avoid the next reader wondering whether it should be wired in.
- **`int(result["sample_count"])` cast** at `check_budgets.py:316` is defensive — validation already guarantees a positive int. Minor; harmless either way.
- **Audit note legibility.** `id:count` is parser-friendly but a bit terse for a human glancing at CI logs. `id (sample_count=N)` would scan slightly better; not worth blocking on.
- **Comment wording.** The "scheduler-bound" framing at line 25-27 is technically right but might be opaque to a future reader; "with <20 samples the nearest-rank p95 collapses to the max sample, so it tracks scheduler noise rather than the workload" is the longer-form version.
- **`spike_metric` / `force_sample_count` raise `BudgetError`** for "case not found" — these are programmer-invariant violations, not budget failures. Using `AssertionError` (or just `assert`) would be more idiomatic, but `BudgetError` propagates through the self-test harness the same way, so functionally fine.

### 3. Merge recommendation

**Approve / ready to merge.** The Pass 1 visibility ask is addressed (skipped-p95 audit note surfaces in both CLI and area-runner paths), boundary and median-still-enforced regressions are locked in, and the new shape validation defends the precondition that `should_enforce_p95` depends on. The only cleanup I'd consider doing in this same PR is deleting the orphaned `budget_p95_regression_result.json` and dropping the duplicate `spike_twenty_sample_p95` self-test line — both are trivial and avoid leaving the next reviewer with the same "is this still wired in?" question.
