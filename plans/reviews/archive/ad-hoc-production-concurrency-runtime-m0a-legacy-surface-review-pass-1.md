## Review: milestone_concurrency_runtime_0a — FAIL

The diff makes good progress (legacy `lib/sifr/{asyncio,concurrent,subprocess,threading}.sifr` removed, sources.rs trimmed, new `SIFR-IMPORT-0009` diagnostic with native-namespace replacement, negative fixtures for all nine legacy names, async-concurrency model doc rewritten, golden manifest unblocked). But there are concrete blockers before opening the PR:

### Blockers

1. **Local create-pr validation not run / not recorded.** `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:319-321` still says `M0a targeted local validation: Pending.`, with no `scripts/run_all_tests.sh --profile create-pr` report cited. AGENTS.md explicitly requires "Before considering any task done, run local validation on your changes" and the same ledger states "Record local validation for each milestone before opening its PR". → Run `scripts/run_all_tests.sh --profile create-pr` and record the evidence (and `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `python3 scripts/check_hir_maintainability_guardrails.py`) in that section.

2. **Two e2e fail fixtures are byte-identical and both still listed in the discovery directory.** `crates/sifr/tests/e2e/fail/thread_pool_executor_non_send_rejected.sifr` and `crates/sifr/tests/e2e/fail/thread_pool_submit_unannotated_rejected.sifr` are now literally:
   ```
   # expect-error: SIFR-IMPORT-0009
   from sifr.concurrent import ThreadPoolExecutor
   ```
   `verification/stdlib/concurrency_runtime_m0a_legacy_surface_traceability.md` only points to `thread_pool_submit_unannotated_rejected.sifr` for the `sifr.concurrent` row, so `thread_pool_executor_non_send_rejected.sifr` is now an unreferenced duplicate. e2e fixtures are discovered lexicographically, so both will run as identical-name checks and the snapshot/insta sort order will look noisy. → Delete `thread_pool_executor_non_send_rejected.sifr` (and check no manifest still references the old name) or repurpose it. The same goes for whether `async_popen_unsupported.sifr` and `subprocess_non_string_cmd.sifr` should be merged into `legacy_sifr_subprocess_removed.sifr` — they all test exactly the same `SIFR-IMPORT-0009 on sifr.subprocess`.

3. **`reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-1.md` is a 0-byte file** committed alongside the diff. Either fill it with the review content (or delete it before opening the PR). An empty review artifact in `reviews/` will mislead future reviewers tracking the planning-review chain in the execution ledger.

4. **Compiler still carries a public-facing `sifr.asyncio` veneer code path that is now permanently dead.** With `sifr.asyncio` removed from `STDLIB_SOURCES` and its early registration deleted from `imports.rs`, `LowerCtx.asyncio_compat_imports` (`crates/sifr_lowering/src/lower/mod_context.rs:93,154`) is never populated. That makes the following unreachable:
   - `lower_asyncio_compat_call` (`crates/sifr_lowering/src/lower/task_calls.rs:20-…`) and its dispatch in `crates/sifr_lowering/src/lower/expressions/core_and_calls.rs:14,310`
   - The entire `crates/sifr_lowering/src/lower/asyncio_run_entrypoint.rs` (133 lines)
   - The `asyncio_compat_imports.get(...)` branch in `crates/sifr_lowering/src/lower/async_with.rs:70`

   The M0a phase contract is explicit: "Move any evidence-only helpers behind internal test namespaces; they must not be reachable as public sifr.* modules" and "Prove production APIs do not depend on legacy surfaces before M1 starts." Leaving this dead lowering in tree contradicts the latter. → Either delete the asyncio-veneer lowering plus its callsites and the `asyncio_compat_imports` field, or move it behind an explicit internal-test-only module and document that in `verification/stdlib/concurrency_runtime_m0a_legacy_surface_traceability.md` under "Implementation evidence".

### Non-blocking but worth fixing before PR

- `demos/{subprocess,system_tools,additional_modules,stdlib_fixes}/emitted.rs` still contain `// --- stdlib: sifr.subprocess ---` boilerplate from before. They are checked-in artifacts (excluded from the file-size guardrail but used by Phase 34 generated-code-quality producer caches). Regenerate them so the committed snapshots match the new `main.sifr`, or note the staleness so reviewers don't think the demos still depend on `sifr.subprocess`.
- `verification/stdlib/concurrency_runtime_m0a_legacy_surface_traceability.md` lists the legacy-row "Regression fixture" for `sifr.concurrent` as `thread_pool_submit_unannotated_rejected.sifr` — once the duplicate (blocker #2) is removed, repoint this row at `legacy_sifr_concurrent_futures_removed.sifr` or a dedicated `legacy_sifr_concurrent_removed.sifr` for cohesion with the other legacy rows.
- Several fail fixtures lost their original SIFR-TYPE-0002 / SIFR-ASYNC-0005 / SIFR-NAME-0004 coverage when they were rewritten as SIFR-IMPORT-0009 cases. Those original code paths (TYPE-0002 ThreadPool result-type mismatch, ASYNC-0005 unannotated `executor.submit`, NAME-0004 missing-member) may still exist for non-legacy modules; confirm they keep a representative fixture elsewhere, or note the coverage move in the traceability doc.

### Spot-check confirmations (passed)

- No demo, e2e pass fixture, golden program, or validation-lane manifest still imports any of the 9 legacy `sifr.*` modules (confirmed via grep across `demos/`, `crates/sifr/tests/e2e/pass/`, `verification/`, `scripts/`). `verification/validation_lanes/{create_pr,merge}_e2e_manifest.json` had `thread_pool_executor_basic`, `threading_compat_basic`, `subprocess_completed_process`, and `stdlib_subprocess` removed, and the deleted pass fixtures are not referenced from any other live manifest.
- `process_runtime_and_platform.sifr` expected/actual lengths (18 vs 18) line up after the three subprocess actuals were dropped (`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr:117-127`).
- `internal_docs/async_concurrency_model.md` no longer describes `sifr.asyncio` as a supported veneer; the only remaining mentions are in the "Removed Compatibility Veneers" migration table and the explicit "not a public compatibility veneer" sentence.
- `SIFR-IMPORT-0009` is registered correctly in `crates/sifr_diagnostics/src/codes/registry.rs:36`, `parsing_names_and_types.rs:261-277`, `docs/errors/diagnostic-codes.md:64`, `internal_docs/diagnostic_codes.md:106`, and the auto-generated `docs/errors/SIFR-IMPORT-0009.md`. The lowering path is wired so any `from sifr.<legacy> import …` skips externals lookup and routes through `report_unknown_stdlib_module` → `unsupported_legacy_stdlib_module` → emits `SIFR-IMPORT-0009` with `legacy_module`, `suggested_module`, `imported_names`, `reason`.

### Verdict

FAIL — primarily because (1) no create-pr validation evidence has been recorded, (2) duplicate/redundant negative fixtures need cleanup, (3) an empty review artifact is staged, and (4) substantial dead `sifr.asyncio` veneer lowering remains in `sifr_lowering` and contradicts the M0a "production APIs do not depend on legacy surfaces" gate. Fix those, then re-run `scripts/run_all_tests.sh --profile create-pr` and record the evidence under the M0a section of the execution ledger.
