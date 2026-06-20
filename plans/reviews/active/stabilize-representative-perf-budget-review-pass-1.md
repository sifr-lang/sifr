## Review

### 1. Blockers
None — implementation is correct.

- `should_enforce_p95` is referenced at `check_budgets.py:276` and defined later at `check_budgets.py:305`. Fine — Python resolves at call time.
- Bool-vs-int check in `validate_results_shape` (line 239) correctly excludes `True`/`False` from passing as positive integers.
- Self-test coverage proves both directions: 5-sample p95 spike does **not** fail (`check_budgets.py:353–354`), 20-sample p95 spike does (`check_budgets.py:343`). The chosen seed benchmarks back this up — `check-project-004-project-graph` has `sample_count: 5` and `interactive-tooling-foundation-002-warm-diagnostics-query` has `sample_count: 20` in baselines, so the seeds aren't accidentally testing the wrong branch.
- Old `budget_p95_regression_result.json` seed is no longer referenced and not used elsewhere — harmless dead file, not a blocker.

### 2. Gate-policy risk (real, worth raising)

This is the part to weigh before merging — not a bug, but a soft policy shift:

- **Silent scope.** 39 of 65 baselines (`check_budgets.py` data) have `sample_count: 5`, so this patch quietly drops p95 enforcement for ~60% of the budgeted suite. The CLI still prints `performance budget check passed` with no indication that p95 was skipped for any benchmark.
- **No per-benchmark opt-in.** The decision is a single global constant `MIN_P95_SAMPLE_COUNT = 20` (line 25) with no comment explaining why 20, and no per-budget switch (e.g. `enforce_p95: false` in `budgets.json`). When someone later adds a benchmark with `measured: 10`, p95 silently won't gate it.
- **Root-cause alternative not addressed.** The manifest hard-codes `measured: 5` for the noisy cases. A more durable fix would be: bump those cases to `measured: 20` in the representative suite, or switch the noisy ones to a robust statistic (trimmed mean / MAD-bounded). This patch lowers the gate sensitivity instead of fixing the sample size. Median is still enforced, so this is defensible — but it is a policy decision worth being explicit about.
- **Missing visibility hook.** A one-line `print` or stderr note listing benchmarks whose p95 was skipped would turn a silent waiver into an auditable one without changing behaviour.

### 3. Non-blocking suggestions

- Log the list of skipped-p95 benchmarks in `check_budgets`/`compare_result` so operators see what was waived this run.
- Add a boundary self-test for `sample_count == 19` (skipped) vs `sample_count == 20` (enforced) to lock the threshold.
- Add a self-test that median regression on a 5-sample seed still fails — proves median enforcement isn't accidentally tied to the same gate.
- `spike_five_sample_p95` and `spike_twenty_sample_p95` (`check_budgets.py:443–456`) are near-duplicates — could be one helper parameterised by id.
- Consider validating `len(samples_ms) == sample_count` here (mirrors `check_trend_policy.py:266–267`) since the new policy depends on `sample_count` being truthful. Trend policy catches it elsewhere, so this is defence-in-depth, not required.
- A comment near `MIN_P95_SAMPLE_COUNT` explaining "below 20 samples, nearest-rank p95 collapses to max and becomes scheduler-noise-bound" would document the why for the next person.

### 4. Merge recommendation

**Merge — conditional on filing the visibility follow-up.** The patch is correct, validation passes, and it unblocks the gate without weakening median/RSS/timeout/cache enforcement. But the silent waiver of p95 for 60% of benchmarks is a real footgun; I'd want either a one-line skip log in this PR or a tracked follow-up to (a) surface skipped p95s and (b) revisit whether to raise the noisy benchmarks to `measured: 20` rather than relax the gate. Without that, the next "p95 looks fine" claim will be hard to trust.
