## Findings

### Concern 1 — Project-build emitted Rust under-counting: **Resolved**
`verification/areas/performance/run_benchmarks.py:500-523` now does:
- `rust_files = sorted(source_dir.rglob("*.rs"))` — walks the full `output_dir/sifr_output/src` tree, not just `main.rs`.
- Sums `len(splitlines())` and `len(encode("utf-8"))` across **every** emitted `.rs` file.

The smoke validation confirms the fix end-to-end: `build-project-001-additional-modules` now reports `emitted_rust_lines=3769` vs `build-single-file-001-break-continue` at `15`, which is only possible if nested module files under `helpers/` etc. are being included. The pass-1 worry that the field would silently capture only `main.rs` for project builds is gone.

### Concern 2 — Silent OSError leaving metrics null: **Resolved**
`collect_build_size_metrics` no longer swallows errors. Three explicit `BenchmarkError` raises cover the layout-drift cases:
- `run_benchmarks.py:506-507` — no `.rs` files emitted → raise.
- `run_benchmarks.py:513-514` — any emitted `.rs` file unreadable → raise (with chained `from error`).
- `run_benchmarks.py:521-522` — release binary `stat()` fails → raise.

`metrics["generated_binary_bytes"]` is now only ever populated from a successful `stat()`; there is no path through `collect_build_size_metrics` for a `mode == "build"` case that returns a dict containing `None`. Layout drift will fail loudly during the case, which is the contract the pass-1 finding asked for — stronger than the suggested inline comment.

### Spot-check — no collateral regressions
- `run_case` (line 354) still gates `collect_build_size_metrics` on `case.raw.get("mode") == "build"`; non-build paths (frontend-query at line 402, lsp-query at line 444, plus `check`/`fmt-check` falling through) still merge `SIZE_METRIC_DEFAULTS` (all-null), so the schema contract on non-build cases is unchanged.
- `dict(SIZE_METRIC_DEFAULTS)` at line 504 copies the module-level dict before mutation, so the defaults dict can't be poisoned across cases.
- `executable_name("sifr_output")` is the same helper used for `sifr`/`frontend_query_bench`, so the Windows `.exe` suffix is handled consistently.
- `shared_build_dir` is computed once outside the sample loop (line 335) and reused for both the sample commands and the size-metric collection — no mismatch risk between the directory the build wrote to and the directory we scan.

---

**No blockers remain. No further review round needed.** Both pass-1 non-blocking concerns are fully addressed by the new implementation, and the smoke-run line/byte counts independently confirm the multi-file walk is doing what it claims.
