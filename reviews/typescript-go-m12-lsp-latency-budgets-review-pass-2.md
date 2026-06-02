I have all the data I need. Let me complete the review.

## Review of M12 LSP Latency Budgets — Pass 2

**Verdict: SATISFIED**

All three pass-1 blockers are addressed.

### Blocker 1 — Baseline CV instability: FIXED

`verification/performance/lsp_query_bench.py:210-235` now amortizes jitter by running `inner_repetitions` operations inside each timed sample and dividing the wall time, while cold start at `lsp_query_bench.py:196-207` correctly keeps process-per-sample. The combination of `warmups=3, measured=20, inner_repetitions=10–20` (manifest.json:69-84) collapses all `lsp-query` baseline CVs in `baselines.json` below `default_stability_limit=0.10`:

```
lsp-query-010-rename             cv 0.077  (was 0.587)
lsp-query-011-semantic-tokens    cv 0.070  (was —)
lsp-query-017-generated-rust...  cv 0.062  (was 0.154)
lsp-query-004-workspace-diag     cv 0.056  (was 0.108)
lsp-query-014-type-hierarchy     cv 0.050  (was 0.202)
…all others < 0.05
```

`run_benchmarks.py:527-531` would now accept these as a fresh `--capture-baseline`. The refresh workflow in `internal_docs/performance_budgets.md:54-57` no longer fails.

### Blocker 2 — didOpen unmeasured: FIXED (with a doc nit)

`lsp_query_bench.py:286-287` dispatches `lsp.did_open_diagnostics` to `run_did_open_diagnostics`, which is now mapped by `manifest.json:84` to `lsp-query-018-did-open-diagnostics` with budget id `perf.lsp.document_sync.did_open`. Baseline (`baselines.json:2010-2052`), budget (`budgets.json:1190-1208`), and the M12 mapping table in `verification/performance/lsp_query_budget_ids.md:90` all match. The guardrail (`check_typescript_go_m1_guardrails.py:125-165`) enforces both the 18-scenario set and the 18-budget-id set, including `perf.lsp.document_sync.did_open`.

Non-blocking nit: prose in `internal_docs/performance_budgets.md:90-91`, `internal_docs/typescript_go_architecture_transfer_m12_lsp_latency_budgets.md:35-37`, and the `rationale` in `budgets.json:1201` all say didOpen covers "diagnostics publication **and a pull diagnostics request**". The implementation in `lsp_query_bench.py:238-267` only issues `notify(didOpen)` + `wait_for_notification(publishDiagnostics)` — no `textDocument/diagnostic` pull. Either drop "and a pull diagnostics request" from those three places, or add a pull to `run_did_open_diagnostics` mirroring `run_document_diagnostics`.

### Blocker 3 — Loose thresholds and undocumented policy: FIXED

Thresholds are now baseline-derived with an explicit floor and SLO cap. The formula in `budgets.json:1317-1319`:
```
lsp_query_median_ms: min(slo_median, max(baseline_median * 3, baseline_median + 5ms))
lsp_query_p95_ms:    min(slo_p95,    max(baseline_p95 * 4,    baseline_p95 + 10ms))
lsp_query_cache:     not enforced until LSP exports stable per-request cache counters
```
and the same rule is spelled out in prose in `internal_docs/performance_budgets.md:81-98`. Compared to pass 1, the tightening is real:

```
perf.lsp.hover.symbol           was 100ms  → 5.088ms median   (60×)
perf.lsp.type_hierarchy.symbol  was 250ms  → 5.077ms median   (49×)
perf.lsp.diagnostics.workspace  was 1000ms → 5.324ms median   (188× tighter)
perf.lsp.cold_start.workspace   was 100ms  → 13.488ms median  (3× baseline)
perf.lsp.generated_rust_preview was 750ms  → 23.946ms median  (3× baseline)
```

Per-entry `rationale` strings (`budgets.json:861, 881, 901, 921, ...`) now name the family the threshold applies to instead of repeating a boilerplate. The `policy: "lsp-query"` value is documented end-to-end.

### Note items from pass 1

- **Subset coverage in `scripts/run_all_tests.sh`** — fixed. `lsp-query-003-diagnostics` is now in the PR/nightly/release subset (line 200) and in the `--profile quick` smoke (line 214), so the merge gate actually exercises an LSP code path instead of comparing baselines to themselves.
- **Reserved-but-unused namespaces** — fixed. `verification/performance/lsp_query_budget_ids.md:107-111` now explicitly tags un-mapped reservations as "intentionally deferred".
- **`cache_misses` is bogus** — still emitted as `iterations` by `lsp_query_bench.py:296-298`. Harmless for M12 because every `lsp-query` budget in `budgets.json` ships with `cache: {}` and the policy explicitly defers cache enforcement until real counters exist; if a future budget ever sets `max_misses`, it will be a tautological gate. Recommend dropping the field from the bench output rather than re-emitting a known-wrong value.

### Residual non-blocking risks

1. **didOpen prose mismatch (above)** — three docs claim a pull that the code doesn't issue.
2. **Negative budget seeds remain pre-M12.** `check_budgets.py:332-345` passes `allow_subset=True` on the regression/malformed seeds because the seeds in `verification/performance/negative_seeds/` still contain only 48 cases (`lsp-query-001` only, no LSP-002 through 018). The negative tests therefore can't catch a future accidental shrink of the 17 new LSP budgets. Regenerating the four `budget_*_regression_result.json` seeds against the current `baselines.json` would remove the need for `allow_subset` on those self-tests.
3. **6-line fixture (`verification/performance/query_projects/lsp/main.sifr`)** — sub-millisecond baselines are dominated by JSON-RPC framing, so the +5ms / +10ms additive floor is what catches real regressions. A multi-hundred-line fixture before M14/M17 would let the multiplicative `×3 / ×4` rule become the binding constraint instead of the additive floor.
4. **`slo_median` / `slo_p95` values aren't enumerated.** The docs reference an SLO cap but never list per-family caps; the only place an SLO actually binds in the current numbers is `perf.lsp.request_families` (50/100 ms). If a future tighter cap is intended, it should be listed in `budgets.json` derivation alongside the multipliers so reviewers can verify the cap matches the policy intent.

None of the residual items contradict the M12 closeout ("protocol-level editor latency is enforced per request family") — they're hygiene/clarity follow-ups.
