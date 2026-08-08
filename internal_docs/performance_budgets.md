# Performance Budgets

frontend query architecture performance policy is local-first and versioned under `verification/areas/performance/`.

## Files

- `manifest.json` defines the verification area suites.
- `data/benchmark_manifest.json` defines the benchmark corpus and stable budget ids.
- `data/baselines.json` records the approved performance baseline run for every manifest case.
- `data/budgets.json` records thresholds derived from the checked-in baselines.
- `data/waivers.json` records active temporary performance waivers.
- `run_benchmarks.py` executes benchmarks and emits evidence under `target/performance/evidence/`.
- `check_budgets.py` compares benchmark results against budgets and validates waivers.

## Commands

Run the schema and negative-seed checks:

```bash
python3 verification/areas/performance/run_benchmarks.py --validate-only
python3 verification/areas/performance/run_benchmarks.py --self-test
python3 verification/areas/performance/check_budgets.py --self-test
```

Run a fast representative benchmark smoke:

```bash
python3 verification/areas/performance/run_benchmarks.py --sample-scale smoke
```

Run the budget gate against checked-in baselines:

```bash
python3 verification/areas/performance/check_budgets.py
```

`scripts/run_all_tests.sh --profile create-pr` runs manifest validation, benchmark
negative seeds, budget/waiver negative seeds, the checked-in baseline budget
gate, and a minimal frontend-query smoke. `--profile merge`, `nightly`, and
`release` run the same schema and negative checks, execute a reviewed
representative subset with the manifest sample counts into
`target/performance/representative.budget.latest.json` for `merge` and
`target/performance/full.budget.latest.json` for `nightly` and `release`, and
run `check_budgets.py --allow-subset` against that result file. The subset
covers single-file check, project check, single-file build, project build,
incremental cache behavior, interactive diagnostics, and diagnostic architecture
diagnostic/exit-code non-regression.

Representative and full budget producers require a controlled host window
before measuring. Admission requires three consecutive quiet snapshots on AC
power on macOS, no thermal or CPU-power warning, no competing Cargo, rustc,
benchmark, or Git indexing process, normalized one-minute load at or below
`0.85`, no more than `50%` of one logical CPU consumed by unrelated processes,
and a stable fixed-work CPU-throughput calibration. The unrelated-CPU limit is
intentionally independent of logical CPU count because the governed command
benchmarks are sensitive to contention for one performance core. The report
records load, power, thermal state, direct CPU frequencies when the host exposes
them, the unprivileged throughput proxy otherwise, external CPU consumption and
its top processes, memory-pressure counters, and the compiler/generated-artifact
cache state. Per-case monitoring applies the same external-CPU limit and records
pressure that appears after admission. The benchmark runner and its descendants
and ancestors are excluded, so measured work and its command launcher cannot
reject the run. On macOS, `ps` supplies recent decayed CPU use. On Linux, `ps`
supplies process-lifetime average CPU use; controlled Linux evidence must record
that limitation until the runner provides interval sampling there. On hosts
without direct frequency telemetry, the throughput calibration runs only during
admission because running it inside a measured case would contaminate the
samples; the case coefficient of variation rejects frequency-driven in-window
instability.
Competing-work classification uses the actual executable or Python module/script
identity, not arbitrary shell argument text that merely mentions Cargo or a
benchmark. A top-level `git fetch` is not rejected by itself; its CPU/disk-heavy
`index-pack`, clone, or submodule work is rejected when that executable or
subcommand appears. If all three attempts are rejected, their snapshots and
reasons are persisted under the run's `control-failures/` directory before the
producer exits.

Each case must produce samples whose coefficient of variation is within its
manifest `stability_limit` (default `0.10`). An unstable or host-contaminated
case is discarded and retried up to two times. Before each retry, the producer
must reacquire the same complete controlled-host admission window; it does not
spend another attempt inside known continuing contention. Exhausting the three
controlled attempts fails the producer as host instability. Stable samples are
compared to the unchanged governed budgets, so a uniform seeded slowdown still
fails. The budget checker treats sample instability as non-waiverable.

The profile adapter invalidates the fixed `*.budget.latest.json` path before
production and binds producer and checker with a unique invocation id. If the
producer fails, the checker is not run. A failed benchmark invocation therefore
cannot feed a prior run to the budget diagnostic.

Full-corpus benchmark execution and baseline refresh remain explicit. Budget
baselines and trend baselines are separate governed artifacts. Updating
`data/baselines.json` changes blocking thresholds and uses the reviewed budget
workflow:

Refresh baselines intentionally after review:

```bash
python3 verification/areas/performance/run_benchmarks.py --capture-baseline
python3 verification/areas/performance/check_budgets.py
```

Updating `data/trend/current.json` does not change blocking budgets. It requires
an owner-approved reference run from a clean exact commit, the complete
manifest with manifest sample counts, and controlled-host admission and
per-case monitoring:

```bash
SIFR_VALIDATION_PROFILE=approved-reference \
SIFR_THERMAL_POLICY=controlled-host \
python3 verification/areas/performance/run_benchmarks.py \
  --capture-trend-baseline \
  --require-controlled-host \
  --reference-approval compiler/performance
python3 verification/areas/performance/check_trend_policy.py
```

The trend snapshot is written only after the full run succeeds and passes the
same stability validation used for budget capture. Its `reference_capture`
receipt binds the clean source commit, invocation and run ids, controlled-host
policy, observation counts, and the SHA-256 of the raw evidence under
`target/performance/evidence/`. Per-case transient host snapshots remain in the
raw evidence rather than bloating the checked-in trend snapshot. Expired
metadata or freshness deferrals are removed only by this successful fresh
capture; extending their dates is not a refresh.

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
- LSP protocol layer added `lsp-query-001-request-families` with budget id
  `perf.lsp.request_families`; LSP latency budget keeps that case as aggregate protocol smoke
  coverage only and adds per-request `lsp-query-*` cases with concrete
  `perf.lsp.*` budget ids. These cases execute `lsp_query_bench.py` through
  `sifr lsp --stdio` and are validated by the same manifest/budget gate. Cold
  start, workspace diagnostics, references, and rename run against
  `verification/areas/performance/query_projects/lsp_workspace/`, a multi-file
  workspace fixture, so workspace-shaped protocol costs are not measured only
  on the single-file smoke fixture.

Every `lsp-query` manifest case must declare `workspace_mode` as either
`isolated` or `package`; manifest validation cross-checks that choice against
the source directory's sibling `sifr.toml`. Isolated warm/cold cases execute in
a deterministic minimal temporary locked Sifr/Cargo package so upward package
discovery cannot attach them to the repository workspace. Package cases run the
checked-in `lsp_workspace` fixture in place. The didOpen diagnostics sync case
retains its package-less temporary directory because its measured operation is
document publication rather than package analysis.

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

The frontend query architecture frontend-query budgets measure compiler-service operations below
the protocol layer, such as warm diagnostics queries, source-map lookups, and
local edit-loop invalidation. LSP latency budget LSP budgets measure the editor-observed
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
