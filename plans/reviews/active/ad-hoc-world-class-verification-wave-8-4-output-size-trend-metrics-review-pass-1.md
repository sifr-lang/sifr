## Findings

### 1. (Non-blocking) `build-project-*` cases will undercount emitted Rust source
`verification/areas/performance/run_benchmarks.py:500-515` only reads `output_dir/sifr_output/src/main.rs`. For project builds (`build-project-001-additional-modules` through `-005-project-graph`), `sifr build` materialises a multi-file tree (`src/main.rs` + nested `helpers/mod.rs`, `helpers/value.rs`, etc. — confirmed in `crates/sifr_driver/src/tests/project_build_check.rs:131`). The trend metric will silently capture only the main module while presenting under the name "emitted_rust_lines/bytes" implying total emitted source. Validation only exercised `build-single-file-001-break-continue` so this gap was not caught. Self-consistent across runs, but the field contract is broader than the implementation. Walk `output_dir/sifr_output/src/**/*.rs` (or scope the contract in a comment/docstring) before this gets adopted into a baseline.

### 2. (Non-blocking) Silent OSError on a build with no `main.rs`
`collect_build_size_metrics` (run_benchmarks.py:500-515) swallows OSError and leaves the metric `None`, indistinguishable from a non-build case in downstream reports. The build expected-exit-code check earlier in `run_case` would catch a build failure, so this only matters if the layout drifts but the build still exits 0 — at which point silently reporting `None` masks a contract drift. Worth a short inline comment that None on a `mode == "build"` case means the emit layout drifted.

### 3. (Non-blocking) `baseline["baseline_captured_at_unix"]` is accessed without a defensive guard
`trend_reports.py:97` raises `KeyError` rather than `TrendReportError` if a baseline result omits that field. Pre-existing behavior carried over from the pre-split code — not a regression, but visible in the freshly-split surface and worth a follow-up.

### 4. (No blocker) Backward compatibility verified
- `metric_snapshot` uses `.get()` and `metric_delta` uses `nullable_delta`/`nullable_percent_delta`, so the checked-in baseline with `emitted_rust_*: null` and `generated_binary_bytes: null` flows through cleanly — confirmed by inspecting `target/performance/wave-8-4-build.trend.json` (current populated, baseline null, delta null).
- `check_trend_policy.py:275-279` already validates `tracked_optional_metrics` as "present, possibly null" — unchanged by this slice.
- `check_budgets.py` only reads `median_ms`/`p95_ms`/`peak_rss_bytes`, so size metrics in captured baselines don't disturb budget gating.

### 5. (No blocker) Non-build benchmarks correctly keep size fields null
`run_case` (run_benchmarks.py:354) gates `collect_build_size_metrics` on `case.raw.get("mode") == "build"`; frontend-query (line 402) and lsp-query (line 444) explicitly merge `SIZE_METRIC_DEFAULTS`; `check` and `fmt-check` command cases fall through the else and also use `SIZE_METRIC_DEFAULTS`. Verified against the manifest's `check-*`, `frontend-query-*`, `lsp-query-*`, and `formatter-*` (fmt-check) cases.

### 6. (No blocker) Trend semantics preserved across the split
- Stable-id comparison: `baselines_by_id` dict at trend_reports.py:30-34. ✓
- Missing baseline raises loudly: `TrendReportError(f"...missing benchmark ids: {missing_baselines}")` at trend_reports.py:45 (now exercised by the new self-test). ✓
- Sample-count-below-baseline classification: trend_reports.py:139-140. ✓
- Local trend deltas non-blocking: `local_trend_delta_blocking: False`, `local_blocking: False`. ✓
- Review gating only for `regression_outside_noise`: trend_reports.py:47-51, 115. ✓
- `main()` catches both `BenchmarkError` and `TrendReportError` (run_benchmarks.py:164), so the split does not silently leak exceptions.

### 7. (No blocker) Import path works in all current invocations
`from trend_reports import ...` resolves via `sys.path[0]` (the script's own directory) for every existing call site — `runner.py`, `manifest.json`, `check_phase36_closeout.py`, and direct CLI use — which all invoke `run_benchmarks.py` by absolute path with `sys.executable`.

### 8. (No blocker) Guardrails and tracker
- File sizes: 750 / 251 lines, both under the 900-line cap.
- Self-test now covers size deltas, missing-baseline error, and required-metric numeric check — matches the suggestion deferred in the Wave 8.3 review.
- Tracker entry for Wave 8.4 (`plans/issues/active/...:1457-1462`) accurately describes the slice and the Status banner was advanced to "Wave 8.1 through Wave 8.3 ... merged through PR #2639" — correct framing for this open slice.

---

**No blockers remain.** Findings 1 and 2 are quality follow-ups (especially #1, since project-build size metrics will be misleading once a reference run captures them). The slice is safe to PR as-is; #1 should be tracked as a known scope before any approved reference run bakes the partial number into a baseline. No further review round needed.
