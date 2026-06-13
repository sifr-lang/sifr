## Review: milestone_concurrency_runtime_0a — PASS

All four pass-1 blockers from `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-1.md` are remediated in the staged diff, and the rest of the M0a requirements (public-surface removal, structured `SIFR-IMPORT-0009` emission, veneer-free native task lowering, doc/manifest cleanup, ledger validation evidence) are met. The diff is ready to ship as the M0a PR.

### Pass-1 blockers — verified remediated

1. **Local create-pr validation recorded.** `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:319-327` now records `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_stdlib legacy_concurrency_runtime_modules_are_not_embedded_public_sources`, `cargo test -p sifr_lowering unsupported_legacy_stdlib_module_has_import_code_and_replacement_args`, `cargo test -p sifr_driver --lib` (140 tests), `cargo test -p sifr test_e2e_fail`, and `scripts/run_all_tests.sh --profile create-pr` with the report at `target/validation_lane_reports/create-pr.latest.json` (pass=5, skip=2 golden; 70 e2e pass; advisory: warm wall-time only). The report file exists on disk and shows all e2e/generated-code/platform-golden buckets at `status: pass`.

2. **Byte-identical duplicate negative fixtures removed.** `crates/sifr/tests/e2e/fail/thread_pool_executor_non_send_rejected.sifr`, `thread_pool_submit_unannotated_rejected.sifr`, and `subprocess_non_string_cmd.sifr` are deleted in the staged diff. The remaining renamed legacy fixtures (`async_popen_unsupported.sifr`, `asyncio_*`, `concurrent_future_result_type_rejected.sifr`, `process_pool_not_available.sifr`) each import a *distinct* member name from the same removed module, so they exercise different `imported_names` payloads and are not byte-identical with the new `legacy_sifr_*_removed.sifr` set.

3. **Empty pass-1 review artifact filled.** `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-1.md` is 7,277 bytes and contains the full pass-1 review body (verdict, blockers, non-blocking notes, spot-check confirmations).

4. **Dead `sifr.asyncio` veneer lowering removed end-to-end.**
   - `crates/sifr_lowering/src/lower/asyncio_run_entrypoint.rs` is deleted from disk.
   - `mod.rs` no longer declares `mod asyncio_run_entrypoint` or `use asyncio_run_entrypoint::function_uses_asyncio_run_entrypoint`.
   - `mod_impl.rs:160-165` and `typing_and_functions/annotations_and_function_lowering.rs:438-440` collapse `effective_is_async` to `func.is_async`; no more `asyncio.run` entrypoint inference.
   - `LowerCtx.asyncio_compat_imports` is gone from `mod_context.rs` (verified by grep across `crates/sifr_lowering/src`).
   - `lower_asyncio_compat_call` and its `expressions/core_and_calls.rs` dispatch are gone; `task_calls.rs` no longer carries the compat path.
   - `async_with.rs` no longer references `asyncio_compat_imports.get(...)`.
   - The only remaining `asyncio` strings under `crates/sifr_lowering/src` are in `name_import_diagnostics_tests.rs:90-101`, which asserts the new `SIFR-IMPORT-0009` diagnostic — i.e., negative coverage, not a code path.

### Other requested checks — verified

- **Public legacy modules unreachable.** `crates/sifr_stdlib/src/sources.rs` has no entries for `sifr.{asyncio,subprocess,concurrent,concurrent.futures,queue,multiprocessing,threading,contextlib,warnings}`. The companion `lib/sifr/{asyncio,concurrent,subprocess,threading}.sifr` files are deleted. The `legacy_concurrency_runtime_modules_are_not_embedded_public_sources` test in `crates/sifr_stdlib/src/lib.rs:362-387` enforces this for all nine names.
- **SIFR-IMPORT-0009 wired with native replacement args.** Registered in `crates/sifr_diagnostics/src/codes/registry.rs:36-37` with `Severity::Error`; full entry in `parsing_names_and_types.rs:261-276` with args `legacy_module`, `suggested_module`, `imported_names`, `reason`. Emission path: `imports::report_unknown_stdlib_module` → `sifr_stdlib::unsupported_legacy_stdlib_module` → `import_diagnostics::unsupported_legacy_stdlib_module`. `crates/sifr_stdlib/src/lib.rs:165-224` maps each of the nine legacy modules to the correct Sifr-native replacement (`sifr.task`, `sifr.process`, `sifr.runtime`, `sifr.sync`, `sifr.ipc`, `sifr.resource`).
- **Native task lowering veneer-free.** `task_calls.rs` lowers `task.{sleep,timeout,gather,race,select,spawn_blocking}` directly without any `sifr.asyncio` compat path or `asyncio.run` entrypoint inference.
- **Demos, e2e pass fixtures, golden programs, and manifests are clean.** Grep across `lib/`, `demos/`, `crates/sifr/tests/e2e/pass/`, `verification/validation_lanes/`, and `scripts/` finds no live import of any removed module. The four `demos/*/emitted.rs` snapshots no longer carry `// --- stdlib: sifr.subprocess ---` markers. `verification/validation_lanes/{create_pr,merge}_e2e_manifest.json` does not reference any deleted pass fixture.
- **Platform golden gate covers all nine names.** `verification/platform/golden/legacy_sifr_runtime_surfaces_removed.sifr` imports all nine legacy modules; `verification/platform/golden/manifest.json:62-88` expects exit 1 with `SIFR-IMPORT-0009` and `sifr.task / sifr.process / sifr.runtime / sifr.sync / sifr.ipc / sifr.resource` in the diagnostic output, depends on `milestone_concurrency_runtime_0a`, and `blocked_until` is empty (gate unblocked).
- **Negative fixtures per removed module.** `crates/sifr/tests/e2e/fail/legacy_sifr_{asyncio,subprocess,concurrent,concurrent_futures,queue,multiprocessing,threading,contextlib,warnings}_removed.sifr` each assert `SIFR-IMPORT-0009`. Coverage matches the M0a traceability table.
- **Traceability + docs updated.** `verification/stdlib/concurrency_runtime_m0a_legacy_surface_traceability.md` lists all nine removed modules with native directions and fixture paths, and explicitly records the lowering-side removals. `internal_docs/async_concurrency_model.md:86` no longer describes `sifr.asyncio` as a supported veneer; the only remaining mentions are the removal/migration sections.
- **Execution ledger.** `Milestone Checklist` (line 31) marks M0a `[x]`; `Validation Evidence` section (lines 319-327) records the validation matrix; `Implementation PRs` (line 296) says "local implementation prepared; PR pending"; `Planning Reviews` includes the pass-1 review as the M0 implementation review entry. Ledger state is internally consistent with the diff.

### Non-blocking polish (no action required to open the PR)

- `crates/sifr_lowering/src/lower/imports.rs:38-50` keeps `deferred_module_reason` entries for `"sifr.contextlib"` and `"sifr.warnings"`. Both are also in `sifr_stdlib::unsupported_legacy_stdlib_module`, and `report_unknown_stdlib_module` checks the legacy table first, so these two arms are unreachable. They aren't wrong (the messages still say "rejected; use ..."), just dead — safe to remove in a follow-up.
- The pre-existing `async_popen_unsupported.sifr`, `asyncio_*`, `concurrent_future_result_type_rejected.sifr`, and `process_pool_not_available.sifr` fixtures now each assert only `SIFR-IMPORT-0009` with a different imported member. They aren't byte-identical with the new `legacy_sifr_*_removed.sifr` set, but at the family level they overlap; consolidating once the M1 native APIs land (so the original TYPE/ASYNC/NAME coverage can move to fixtures that exercise `sifr.task`/`sifr.runtime`) would tighten the e2e fail suite.
- Pass-1's non-blocking note on `demos/*/emitted.rs` is resolved — the regenerated snapshots in the staged diff no longer carry stale `sifr.subprocess` boilerplate.

### Verdict

PASS. No concrete blocker to opening the M0a PR.
