## Review: milestone_concurrency_runtime_0a — PASS

All four pass-1 blockers from `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-1.md` are remediated in the current working tree, and the M0a phase contract (public legacy CPython-shaped modules removed/diagnosed, native task lowering veneer-free, fixtures + traceability + validation evidence in place) is satisfied. The working tree is ready to open the M0a PR.

### Pass-1 blockers — verified remediated

1. **Create-pr validation recorded.** `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:319-327` now records the full M0a matrix: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_stdlib legacy_concurrency_runtime_modules_are_not_embedded_public_sources`, `cargo test -p sifr_lowering unsupported_legacy_stdlib_module_has_import_code_and_replacement_args`, `cargo test -p sifr_driver --lib` (140 tests), `cargo test -p sifr test_e2e_fail`, and `scripts/run_all_tests.sh --profile create-pr`. Report `target/validation_lane_reports/create-pr.latest.json` exists and shows every `lane_steps` entry at `status: pass`; `create-pr.latest.log:1440` shows `[platform-golden] summary pass=5 skip=2`; the e2e_pass_suite step is `70 passed, 0 failed`; the only advisory is `warm wall-time budget exceeded`. Re-running `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `python3 scripts/check_hir_maintainability_guardrails.py`, `python3 scripts/check_file_size_guardrails.py`, `python3 scripts/check_diagnostic_docs_sync.py`, and the two targeted tests locally all PASS against the working tree.

2. **Byte-identical duplicate negative fixtures removed.** `crates/sifr/tests/e2e/fail/thread_pool_executor_non_send_rejected.sifr`, `thread_pool_submit_unannotated_rejected.sifr`, and `subprocess_non_string_cmd.sifr` are deleted from disk and from the lane manifests. The repurposed historical fixtures (`async_popen_unsupported.sifr`, `asyncio_create_task_outside_scope_rejected.sifr`, `asyncio_loop_policy_not_supported.sifr`, `asyncio_run_requires_coroutine.sifr`, `asyncio_transport_protocol_not_supported.sifr`, `concurrent_future_result_type_rejected.sifr`, `process_pool_not_available.sifr`) each import a distinct member name (e.g. `Popen`, `create_task`, `get_event_loop_policy`, `run`, `BaseTransport`, `Future, ThreadPoolExecutor`, `ProcessPoolExecutor`) and therefore exercise different `imported_names` payloads — they are not byte-identical with the new `legacy_sifr_*_removed.sifr` set.

3. **Empty pass-1 review artifact filled.** `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-1.md` is 7,277 bytes and contains the full pass-1 review body (verdict, blockers, non-blocking notes, spot-check confirmations).

4. **Dead `sifr.asyncio` veneer lowering removed end-to-end.**
   - `crates/sifr_lowering/src/lower/asyncio_run_entrypoint.rs` is deleted; `mod.rs` no longer declares it.
   - `mod_impl.rs:160-164` and `typing_and_functions/annotations_and_function_lowering.rs:439` collapse `effective_is_async` to `func.is_async`; no `asyncio.run` entrypoint inference remains.
   - `LowerCtx.asyncio_compat_imports` is gone — grep across `crates/sifr_lowering/src` returns zero hits for `asyncio_compat`, `asyncio_run_entrypoint`, or `lower_asyncio_compat_call`.
   - `expressions/core_and_calls.rs`, `task_calls.rs`, and `async_with.rs` no longer carry the compat dispatch.
   - The only remaining `asyncio` strings under `crates/sifr_lowering/src` are negative-coverage assertions in `name_import_diagnostics_tests.rs`, not code paths.

### M0a contract — verified

- **Public legacy modules unreachable.** `lib/sifr/{asyncio,concurrent,subprocess,threading}.sifr` are deleted; `crates/sifr_stdlib/src/sources.rs` no longer embeds them. The `legacy_concurrency_runtime_modules_are_not_embedded_public_sources` test (`crates/sifr_stdlib/src/lib.rs:362-387`) enforces this for all nine names (`sifr.asyncio`, `sifr.queue`, `sifr.subprocess`, `sifr.concurrent`, `sifr.concurrent.futures`, `sifr.contextlib`, `sifr.multiprocessing`, `sifr.threading`, `sifr.warnings`).
- **`SIFR-IMPORT-0009` wired with native replacement args.** Registered in `crates/sifr_diagnostics/src/codes/registry.rs:36` with `Severity::Error`; full entry in `parsing_names_and_types.rs:261-276` with args `legacy_module`, `suggested_module`, `imported_names`, `reason`. Emission path: `imports::report_unknown_stdlib_module` → `sifr_stdlib::unsupported_legacy_stdlib_module` → `import_diagnostics::unsupported_legacy_stdlib_module`. `crates/sifr_stdlib/src/lib.rs:180-224` maps each of the nine modules to a concrete Sifr-native replacement (`sifr.task`, `sifr.process`, `sifr.runtime`, `sifr.sync`, `sifr.ipc`, `sifr.resource`).
- **Negative fixtures cover all nine legacy modules.** `crates/sifr/tests/e2e/fail/legacy_sifr_{asyncio,subprocess,concurrent,concurrent_futures,queue,multiprocessing,threading,contextlib,warnings}_removed.sifr` each assert `SIFR-IMPORT-0009`. The traceability table at `verification/stdlib/concurrency_runtime_m0a_legacy_surface_traceability.md` points to the canonical fixture per row.
- **Platform golden gate active.** `verification/platform/golden/legacy_sifr_runtime_surfaces_removed.sifr` imports all nine legacy modules. `verification/platform/golden/manifest.json:62-88` expects exit 1 with `SIFR-IMPORT-0009` plus the six native replacement namespaces and now has `blocked_until: []` (gate unblocked). Lane log confirms `pass=5 skip=2`.
- **Demos, e2e pass fixtures, lane manifests, and doc surfaces clean.** Grep across `demos/`, `crates/sifr/tests/e2e/pass/`, `lib/`, and `verification/validation_lanes/` finds no live import of any removed module. The four `demos/*/emitted.rs` snapshots were regenerated and no longer carry `// --- stdlib: sifr.subprocess ---` boilerplate. `internal_docs/async_concurrency_model.md:86` no longer describes `sifr.asyncio` as a supported veneer.
- **Native task lowering intact.** `task_calls.rs` lowers `task.{sleep,timeout,gather,race,select,spawn_blocking}` directly without any `sifr.asyncio` compat path or `asyncio.run` entrypoint inference. `cargo test -p sifr test_e2e_fail` and the targeted unit tests pass.
- **Execution ledger consistent.** Milestone Checklist (line 31) marks M0a `[x]`; Planning Reviews lines 226-231 record this M0a review chain (pass-1 FAIL → pass-2 PASS); Implementation PRs (line 302) says `local implementation prepared; PR pending`; Validation Evidence (lines 319-327) records the M0a matrix.

### Non-blocking risks

- `crates/sifr_lowering/src/lower/imports.rs:39-44` keeps `deferred_module_reason` arms for `"sifr.contextlib"` and `"sifr.warnings"`. Both are also in `unsupported_legacy_stdlib_module` and `report_unknown_stdlib_module` checks the legacy table first, so these arms are unreachable today. Not wrong, just dead — safe to fold into a follow-up.
- `typing_and_functions/annotations_and_function_lowering.rs:439` keeps `let effective_is_async = func.is_async;` — a no-op rename left over from the deleted entrypoint inference. Harmless; collapse on the next pass through this file.
- The repurposed historical fail fixtures (`async_popen_unsupported.sifr`, `asyncio_*`, `concurrent_future_result_type_rejected.sifr`, `process_pool_not_available.sifr`) now each assert only `SIFR-IMPORT-0009` with distinct imported members. They are not byte-identical with the new `legacy_sifr_*_removed.sifr` set, but at the family level the e2e fail suite is denser than it strictly needs to be. Their original SIFR-TYPE-0002 / SIFR-ASYNC-0005 / SIFR-NAME-0004 coverage moved away from `sifr.concurrent`/`sifr.subprocess`; once M1 native APIs land, those error codes should regain a representative fixture against `sifr.task`/`sifr.runtime`. Track this with the M1 traceability doc rather than blocking M0a.
- `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-2.md` and this pass-3 file are untracked in git — the M0a PR commit should `git add reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-{2,3}.md` so the ledger's reference to pass-2 is not a dangling link. Not a contract blocker, but worth tidying before pushing the PR.

### Verdict

PASS. The four pass-1 blockers are fixed in the working tree, the M0a contract is satisfied, and the recorded create-pr lane report meets every numeric expectation (`pass=5 skip=2` golden, `70 passed, 0 failed` e2e pass, warm wall-time advisory only).

### M1 readiness

M1 (Structured Async Runtime) can start after **all** of the following:

1. The M0a PR is opened from this working tree, reviewed, and merged.
2. The execution ledger's `Implementation PRs` line for M0a is updated with the merged PR URL.
3. The Pending Reviews section's `Post-M0 external review` gate completes — either a recorded external review returning `PASS`, or the five-working-day fallback procedure recorded with attempted review, open questions, conservative self-review, and no unresolved blocking questions (per the ledger contract at lines 234-236).

Conditions 1–2 are mechanical follow-ups to opening the PR. Condition 3 is the explicit M0 gate and is independent of M0a — it cannot be satisfied by this review.
