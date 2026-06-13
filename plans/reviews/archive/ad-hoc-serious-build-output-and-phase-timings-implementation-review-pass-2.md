All pass-1 recommendations are addressed and verified in-tree.

**Pass-1 recommended fixes — landed:**
1. Cached `sifr build` contradiction resolved — `plans/issues/active/ad-hoc-serious-build-output-and-phase-timings.md:102-103` now says `sifr build` materializes into the caller-provided output dir and does not use the cache; deferral noted in change log at line 282.
2. `parse_compiled_binary` removed from `verification/areas/algorithmic_compatibility/corpora/leetcode/benchmarks/bench.py` (grep shows zero hits).

**Pass-1 cleanups — landed:**
- `CachedBinaryArtifact::cache_status_line()` gone (zero hits in `crates/`). `BuildReport::cache_hit()` is now the source of truth at `crates/sifr_driver/src/build/report.rs:114-116`, populated correctly from `cache_entry.report().cache_hit()` at `entrypoint.rs:303` for the cached path and from a literal `false` at `entrypoint.rs:226` for the un-cached `sifr build` path — consistent with the architecture decision.

**Pass-1 testing gaps — addressed:**
- Project mode: `build_output_project_mode_reports_import_closure_counts` (`tests/build_output_contracts.rs:96-119`) writes `main.sifr` + `helper.sifr` and asserts `mode: project`, `Parsing import closure (2 modules)`, `Analyzing 2 modules`.
- Machine-format warning-success: real fix, not just a test. `BuildReport.frontend_diagnostics` now carries `Vec<RenderedDiagnostic>` (`report.rs:55`); `build_rooted_entrypoint_binary_with_report` and `build_cached_rooted_entrypoint_binary` snapshot them via `plan.frontend_diagnostics()` (data form, no side-effect emit) at `entrypoint.rs:205,275`; the CLI renders through `emit_report_frontend_diagnostics` which routes via `render_diagnostics(..., diagnostic_format)` at `diagnostic_rendering_and_run.rs:98-106`, so `json|compact` no longer leaks human-shaped lines on success-with-warnings. `build_output_compact_warning_success_emits_diagnostics_without_progress` (`build_output_contracts.rs:180-209`) covers the regression with a known-real `SIFR-TYPE-0901` integer-multiply warning.

**Other constraints holding:** file-size cap fine; `print_stderr`/`print_stdout` lints unaffected; stderr-only progress still verified by `build_output_default_is_phase_aware_and_stderr_only`; existing `emit_project_frontend_diagnostics` survives only on the `sifr emit` path (`entrypoint.rs:188`), which is unrelated to build/run rendering.

**Verdict:** no further review rounds needed before PR validation. Ship after the planned `cargo test -p sifr -- --skip test_e2e_pass` rerun and `scripts/run_all_tests.sh --profile create-pr`.
