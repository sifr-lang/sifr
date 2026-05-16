

## m35.1 Review: Baseline Benchmark Suite

### SATISFIED

---

### Validation Summary

The milestone satisfies all definition-of-done criteria from `internal_docs/phases/35_performance_benchmarking_and_budgets.md`.

---

### Critical Review Findings

**1. Manifest Compliance — PASS**

All 7 required groups present with correct counts:
- `check-single-file`: 10 ✓
- `check-project`: 5 ✓
- `build-single-file`: 10 ✓
- `build-project`: 5 ✓
- `incremental-local-loop`: 5 ✓
- `interactive-tooling-foundation`: 5 ✓
- `phase27-non-regression`: 5 ✓

Lexicographic sorting, uniqueness constraints, and version fields all correct. Phase 34 overlap confirmed via `phase34_source_id` on 18 build-single-file and check-single-file entries.

**2. Baseline Reproducibility — PASS**

`verification/performance/baselines.json` captures:
- Host metadata (OS, architecture, toolchain, compiler fingerprint, Cargo.lock hash)
- All 45 manifest cases with sample arrays
- Complete metrics: `median_ms`, `p95_ms`, `mad_ms`, `coefficient_variation`, `peak_rss_bytes`
- Max CV: 0.091581 (within default 0.10 limit)
- Evidence at `target/performance/evidence/bench-1778968427-69591.json` confirms the runner executed and produced matching output.

**3. Runner Completeness — PASS**

`run_benchmarks.py` emits complete JSON under `target/performance/`:
- Every result includes `metrics` with all 5 required fields
- `frontend-query` results include `cache.hits`, `cache.misses`, `diagnostics_count`
- `timed_out` field present in all results
- Run reports written to `target/performance/evidence/<run_id>.json` for reproducibility

**4. Negative Validation — PASS**

5 negative seeds prove the runner detects failure modes:
- `malformed_manifest.json` → "missing required fields"
- `missing_input_manifest.json` → "input path does not exist"
- `timeout_result.json` → "timed out benchmark"
- `missing_metric_result.json` → "missing metric"
- `unstable_result.json` → "unstable"

`--self-test` confirms all seeds fail with expected diagnostics.

**5. Frontend Query Cases Use Canonical API — PASS**

`crates/sifr_frontend/src/bin/frontend_query_bench.rs` exercises `sifr_frontend::FrontendContext` directly:
- `load_single_file()`, `load_project()` for context creation
- `parse_module()`, `lower_module()`, `diagnostics_for_module()`, `analysis_for_module()`, `diagnostics_for_project()`, `analysis_for_project()` for query execution
- `update_module_source()` for incremental scenarios
- `module_graph()` and `source_map()` for graph/sourcemap inspection
- All scenarios align with documented Phase 35 API contract

No parallel semantics path detected. The binary invokes `cargo build -q -p sifr_frontend --bin frontend_query_bench` and reads JSON from stdout.

**6. Baseline/Result ID Equality — PASS**

Evidence confirms 45/45 manifest cases have corresponding baseline entries. Maximum coefficient of variation across all cases is 0.091581, well within stability limits.

---

### Non-Blocking Follow-ups

1. **`interactive-tooling-foundation-002-warm-diagnostics-query`** has CV of 0.091581 (near the 0.10 boundary). The baseline is valid per the default stability limit, but future budget threshold derivation should account for this case's higher variance if tighter budgets are applied.

2. **`SourceMapView::text_position_to_span`** and `span_to_text_range` in `sifr_frontend/src/lib.rs:214-230` return `None` — these are documented as Phase 36 responsibilities. No action needed now, but tracking is appropriate.

3. **LSP budget id reservations** in `lsp_query_budget_ids.md` are in place for Phase 36 extension.

---

### Verdict

**m35.1: SATISFIED**

All DoD criteria met: baselines versioned and reproducible, runner emits stable JSON, positive validation proves 45 cases run, negative validation proves malformed manifest/missing input/timeout/missing metric/high-variance failures produce actionable diagnostics, manifest satisfies group counts and Phase 34 overlap, frontend-query cases use canonical `sifr_frontend` API.
