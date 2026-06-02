# Performance Budgets

Phase 35 performance policy is local-first and versioned under `verification/performance/`.

## Files

- `manifest.json` defines the benchmark corpus and stable budget ids.
- `baselines.json` records the approved m35.1 baseline run for every manifest case.
- `budgets.json` records thresholds derived from the checked-in baselines.
- `waivers.json` records active temporary performance waivers.
- `run_benchmarks.py` executes benchmarks and emits evidence under `target/performance/evidence/`.
- `check_budgets.py` compares benchmark results against budgets and validates waivers.

## Commands

Run the schema and negative-seed checks:

```bash
python3 verification/performance/run_benchmarks.py --validate-only
python3 verification/performance/run_benchmarks.py --self-test
python3 verification/performance/check_budgets.py --self-test
```

Run a fast representative benchmark smoke:

```bash
python3 verification/performance/run_benchmarks.py --sample-scale smoke
```

Run the budget gate against checked-in baselines:

```bash
python3 verification/performance/check_budgets.py
```

`scripts/run_all_tests.sh --profile quick` runs manifest validation, benchmark
negative seeds, budget/waiver negative seeds, the checked-in baseline budget
gate, and a minimal frontend-query smoke. `--profile pr`, `nightly`, and
`release` run the same schema and negative checks, execute a reviewed
representative subset with the manifest sample counts into
`target/performance/<profile>.budget.latest.json`, and run `check_budgets.py
--allow-subset` against that result file. If that measured subset misses a
budget, the lane reruns the same subset up to four more times with unchanged
thresholds into `target/performance/<profile>.budget.retry-<attempt>.latest.json`;
one measured attempt must pass. This keeps performance budgets hard while
avoiding one noisy local host window from deciding the merge gate. The subset
covers single-file check, project check, single-file build, project build,
incremental cache behavior, interactive diagnostics, and Phase 27
diagnostic/exit-code non-regression.
Full-corpus benchmark execution and baseline refresh remain explicit:

Refresh baselines intentionally after review:

```bash
python3 verification/performance/run_benchmarks.py --capture-baseline
python3 verification/performance/check_budgets.py
```

## Threshold Rules

Command benchmarks use:

- median latency: `max(baseline_median * 1.10, baseline_median + 25ms)`
- p95 latency: `max(baseline_p95 * 1.15, baseline_p95 + 50ms)`
- peak RSS: `max(baseline_peak_rss * 1.10, baseline_peak_rss + 32MiB)`

Command benchmark RSS is measured per command invocation with `/usr/bin/time` when available (`-l` on macOS, `-v` on Linux). Python `RUSAGE_CHILDREN` is used only as a fallback because it is process-cumulative on some platforms and can otherwise contaminate later samples with earlier validation work.

Frontend-query and local edit-loop benchmarks use stricter latency thresholds:

- median latency: `max(baseline_median * 1.05, baseline_median + 2ms)`
- p95 latency: `max(baseline_p95 * 1.10, baseline_p95 + 5ms)`
- peak RSS uses the same 10% / 32MiB rule as command benchmarks.
- cases with baseline cache hits must keep at least the baseline hit count.
- m36.5 added `lsp-query-001-request-families` with budget id
  `perf.lsp.request_families`; M12 keeps that case as aggregate protocol smoke
  coverage only and adds per-request `lsp-query-*` cases with concrete
  `perf.lsp.*` budget ids. These cases execute `lsp_query_bench.py` through
  `sifr lsp --stdio` and are validated by the same manifest/budget gate.

LSP query budgets use their own policy because they measure editor-observed
JSON-RPC operations instead of in-process compiler queries:

- `lsp-query-001-request-families` is smoke coverage only, with fixed broad
  thresholds for aggregate protocol reachability.
- cold start measures subprocess spawn through `initialize`.
- warm request-family cases reuse one initialized server with an open document
  and record the average operation time across `inner_repetitions` inside each
  measured sample to reduce stdio and scheduler jitter.
- didOpen diagnostics measures document sync through diagnostics publication in
  a fresh warm session.
- per-request latency thresholds are derived from the recorded baseline and then
  capped by the request-family editor SLO: median uses
  `min(slo_median, max(baseline_median * 3, baseline_median + 5ms))`; p95 uses
  `min(slo_p95, max(baseline_p95 * 4, baseline_p95 + 10ms))`.
- LSP budget entries intentionally do not enforce cache-hit or cache-miss
  thresholds until the LSP server exports real cache counters; protocol latency
  remains enforced through median, p95, RSS, and timeout checks.

The Phase 35 frontend-query budgets measure compiler-service operations below
the protocol layer, such as warm diagnostics queries, source-map lookups, and
local edit-loop invalidation. M12 LSP budgets measure the editor-observed
JSON-RPC path on top of those same analysis APIs: cold start includes process
spawn and initialize, document diagnostics include notification-to-publication
plus pull diagnostics, and warm request-family cases reuse an initialized LSP
session with an open document. A regression in a frontend-query budget usually
points at compiler-service work; a regression in an LSP budget can also come
from protocol conversion, scheduling, request dispatch, or command plumbing.

Timeouts are hard failures. Missing results, unknown ids, malformed metric payloads, and cache-miss regressions are hard failures.

## Waivers

Waivers are allowed only for temporary performance threshold regressions. They must include:

- `id`
- `owner`
- `issue`
- `created`
- `expires`
- `benchmark_ids`
- `budget_ids`
- `override`
- `rationale`
- `removal_criteria`

Waivers may override `median_ms`, `p95_ms`, `peak_rss_bytes`, or `cache_hits`. They may not suppress timeouts, missing or malformed results, unknown ids, stale-cache/correctness failures, diagnostic drift, split-brain semantics, or panic-safety failures.

Expired waivers, ownerless waivers, issue-less waivers, unknown benchmark/budget references, and non-performance overrides fail `check_budgets.py`.
