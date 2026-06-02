The review found three blocking issues plus several notes. The implementation is structurally complete (manifest, budgets, baselines, doc, guardrail) but has gaps that contradict M12 goals.

## Blocking findings

### 1. Baselines violate `default_stability_limit = 0.10` for 11 of the 16 new LSP cases
`verification/performance/manifest.json:5` sets `default_stability_limit: 0.10` and `run_benchmarks.py:527-531` (`validate_baseline_capture`) raises `BenchmarkError` when a baseline's `coefficient_variation` exceeds the per-case limit (which defaults to 0.10 in `run_benchmarks.py:67-68`). The checked-in `baselines.json` contains:

| id | cv | × limit |
| --- | --- | --- |
| lsp-query-010-rename | 0.587611 | 5.9× |
| lsp-query-015-code-actions | 0.545131 | 5.5× |
| lsp-query-012-inlay-hints | 0.511379 | 5.1× |
| lsp-query-016-formatting | 0.427661 | 4.3× |
| lsp-query-006-hover | 0.202927 | 2.0× |
| lsp-query-014-type-hierarchy | 0.201859 | 2.0× |
| lsp-query-007-signature-help | 0.168432 | 1.7× |
| lsp-query-017-generated-rust-preview | 0.154251 | 1.5× |
| lsp-query-009-references | 0.122778 | 1.2× |
| lsp-query-005-completion | 0.113404 | 1.1× |
| lsp-query-004-workspace-diagnostics | 0.108497 | 1.1× |

The user's reported validations all skipped `--capture-baseline`, which is the only path that enforces this. The documented refresh workflow in `internal_docs/performance_budgets.md:52-57` (`run_benchmarks.py --capture-baseline`) now fails immediately, leaving the baseline frozen — any future tightening of `sifr lsp` cannot be recorded without bypassing the runner.

Either add per-case `stability_limit` overrides to each affected entry in `manifest.json` (with rationale in the M12 doc explaining why these scenarios are inherently noisy), or rework `lsp_query_bench.py` to amortize jitter (e.g., loop the request N× inside the timed region) so the per-iteration sample isn't dominated by JSON-RPC framing variance.

### 2. `run_did_open_diagnostics` is dead code and didOpen latency is unmeasured
`verification/performance/lsp_query_bench.py:231-259` defines `run_did_open_diagnostics` and `main()` (line 272-273) dispatches scenario `lsp.did_open_diagnostics`, but no manifest case uses that scenario. Meanwhile `verification/performance/lsp_query_budget_ids.md:90` claims `perf.lsp.diagnostics.document` covers both `lsp-did-open-diagnostics` and `lsp-did-change-diagnostics`, yet `lsp-query-003-diagnostics` only invokes `run_document_diagnostics` (didChange + pull). The same claim is repeated in the M12 doc ("Document diagnostics measure a warm edit notification through diagnostics publication"). Either wire `lsp.did_open_diagnostics` to a new manifest case (or to `lsp-query-003-diagnostics` as a paired phase), or remove the unreachable function and tighten the mapping table/doc to say "didChange only".

### 3. Per-request thresholds are 90×–2980× looser than baselines; the `lsp-query` policy is undocumented
The new budgets are effectively static ceilings rather than baseline-derived guards:

```
perf.lsp.type_hierarchy.symbol         baseline 0.090 ms   threshold 250 ms   2778×
perf.lsp.references.workspace_symbol   baseline 0.168 ms   threshold 500 ms   2976×
perf.lsp.diagnostics.workspace         baseline 0.379 ms   threshold 1000 ms  2638×
perf.lsp.hover.symbol                  baseline 0.083 ms   threshold 100 ms   1205×
…
perf.lsp.generated_rust_preview.document  baseline 8.355   threshold 750      90×
```

Issue scope says "guide later work" — a 50× regression in hover (5ms) or definition latency would still pass. The `derivation` block in `budgets.json:1324-1331` only describes `command-default` and `frontend-query-edit-loop`; the new `policy: "lsp-query"` is silent on how its thresholds were chosen, and the per-entry rationale strings just say "M12 budget for representative …". If these are intentional Phase 36 SLO ceilings rather than regression budgets, that must be stated explicitly in `internal_docs/typescript_go_architecture_transfer_m12_lsp_latency_budgets.md` and in `budgets.json` derivation rules — and a separate per-baseline guard (or `stability_limit`/MAD-based ratchet) is needed before LSP regressions can fail CI. As written, M12's stated closeout ("protocol-level editor latency is enforced per request family") is technically true but operationally toothless.

## Notes

- **`check_budgets.py` self-test loosened.** `verification/performance/check_budgets.py:332-345` now passes `allow_subset=True` on the regression and malformed-result seeds. Necessary because the static seeds only contain results for the pre-M12 manifest, but it makes the seeds blind to a future accidental case-list shrink. Regenerate the seeds to include the 16 new LSP cases so the negative tests stay exhaustive.
- **`cache_misses` is bogus.** `lsp_query_bench.py:282-284` emits `cache_hits=0, cache_misses=iterations` unconditionally. Every new LSP budget sets `cache.max_misses == iterations`, so the cache gate is a no-op. Pre-existing for `lsp.request_families`, but M12 multiplies the surface — either compute real cache counters from the server or drop the field from `lsp-query` entries.
- **Fixture is too small to be representative.** `verification/performance/query_projects/lsp/main.sifr` is 6 lines, which is why most baselines are sub-millisecond. Per-request ceilings calibrated on this corpus measure protocol overhead, not realistic analysis-on-edit cost; consider adding a multi-hundred-line fixture before later milestones rely on these numbers.
- **`run_all_tests.sh` budget subset doesn't include any LSP case** (`scripts/run_all_tests.sh:191-201`). Live execution of the new `perf.lsp.*` benchmarks happens only when a developer manually passes `--groups lsp-query`. The merge gate just compares baseline-to-budget, which is tautological. Adding even one representative LSP case (e.g., `lsp-query-003-diagnostics`) to `run_performance_budget_subset` would make the gate actually exercise the new path.
- **Reserved-but-unused ids.** `lsp_query_budget_ids.md:18-41` reserves namespaces (`perf.lsp.definition.local_symbol`, `perf.lsp.rename.prepare`, `perf.lsp.semantic_tokens.delta`, `perf.lsp.completion.auto_import`, `perf.lsp.document_sync.*`, `perf.lsp.transport.*`) that no scenario covers. Reasonable to keep reserved, but tag them "deferred" so the table doesn't imply M12 covers them.

Not satisfied — please address the three blocking findings (stability_limit violations, didOpen unmeasured, ceiling-vs-baseline transparency) before merge.
