RESULT: PASS

# Review: Concurrency Runtime M4 Process Handle Boundary Diagnostics - Pass 2

Branch: `codex/concurrency-runtime-m4-pipe-handle-boundaries`

## Pass-1 follow-up status

- **Empty pass-1 review placeholder**: resolved. `reviews/ad-hoc-production-concurrency-runtime-m4-process-handle-boundaries-review-pass-1.md` is now a populated review artifact (70 lines, RESULT: PASS) with scope verification, findings, and recommendation.
- **Missing share-safety fail fixture**: resolved. `crates/sifr/tests/e2e/fail/process_pipe_reader_shared_rejected.sifr` was added and re-checked locally — it emits `SIFR-OWN-0012` via `validate_shared_constructor` -> `non_share_safe_reason` -> `process_handle_type_label_by_name`, pinning the central-classifier claim against silent regressions if the wrapper-vs-handle ordering is ever refactored.
- **Out-of-scope HTTP/network files**: still present in the working tree but unstaged/untracked. They must not be staged for this PR; staging hygiene is the only remaining ask.

## Scope verified in this pass

- Central lowering change in `crates/sifr_lowering/src/lower/task_scope_calls.rs` (536 lines, well under the 900-line cap).
- Six new fail fixtures under `crates/sifr/tests/e2e/fail/` covering `SIFR-OWN-0010` (3x), `SIFR-OWN-0011`, `SIFR-OWN-0012`, and `SIFR-TYPE-0002`.
- Doc updates in `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` and `verification/stdlib/concurrency_runtime_m4_process_traceability.md`.
- Two review artifacts: pass-1 populated; pass-2 (this file).

## Findings

### Blockers
None.

### High (must fix before PR, but does not invalidate the wave)

- **Working-tree out-of-scope HTTP/network changes are still present.** They must not be staged or committed with this M4 process-handle PR:
  - Modified: `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
  - Modified: `issues/ad-hoc-production-network-http-platform-substrate.md`
  - Untracked: `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-1.md`
  - Untracked: `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-2.md`

  These belong to the network/HTTP platform substrate phase. Stage only the in-scope files: `crates/sifr_lowering/src/lower/task_scope_calls.rs`, the six new `crates/sifr/tests/e2e/fail/process_*.sifr` fixtures, the two listed docs, and the two pass-1/pass-2 review artifacts.

### Medium
None.

### Low (non-blocking)

- **`cargo test -p sifr -- test_e2e_pass` still not in the captured validation list.** `scripts/run_all_tests.sh --profile create-pr` did run the create-pr e2e pass suite (`105 passed, 0 failed, cache_hits=23/27`), which is the authoritative gate signal. The execution ledger correctly records the broad non-lane probe as failing in unrelated existing text/I/O and bytes conversion fixtures and explicitly does not accept it as the wave gate. This is honest scoping, not a regression introduced by this wave.

- **Bare-name class matching collision surface (carried over).** `process_handle_type_label_by_name` still matches on the unqualified class name (`Child`, `AsyncChild`, `PipeReader`, `PipeWriter`), consistent with the existing pattern used by `is_share_safe_sync_wrapper`, `sync_guard_type_label_by_name`, and `class_has_non_send_marker`. `Child` is materially more likely to collide with a user-defined class than `Shared`/`LockGuard`/`SemaphorePermit`. Not a regression introduced by this wave; long-term consider namespacing stdlib-owned class names in the classifier.

## Verification of pass-2 review goals

1. **Pass-1 review artifact populated.** Verified. Pass-1 review file now contains the full review record and a clear PASS recommendation.

2. **Share-safety fail fixture added.** Verified. `process_pipe_reader_shared_rejected.sifr` was added with `# expect-error: SIFR-OWN-0012`. Direct local check produces:
   - `error[SIFR-OWN-0012]: Shared cannot publish `reader` of type `PipeReader` because `PipeReader` is a process pipe reader handle; ...`
   This exercises the `non_share_safe_reason_inner` early-return at `task_scope_calls.rs:296-298` and closes the previously-flagged gap.

3. **All six fail fixtures emit the intended diagnostic codes.** Re-verified by direct `cargo run -q -p sifr -- check ...` on each fixture:
   - `process_pipe_reader_task_boundary_rejected.sifr` -> `SIFR-OWN-0010` (`scope.spawn()` moving `PipeReader`).
   - `process_async_child_task_boundary_rejected.sifr` -> `SIFR-OWN-0010` (`scope.spawn()` moving `AsyncChild`).
   - `process_pipe_writer_spawn_blocking_capture_rejected.sifr` -> `SIFR-OWN-0010` (`task.spawn_blocking()` capturing `PipeWriter`).
   - `process_child_spawn_cpu_return_rejected.sifr` -> `SIFR-TYPE-0002` (`task.spawn_cpu()` returning `Child`).
   - `process_pipe_reader_channel_element_rejected.sifr` -> `SIFR-OWN-0011` (`ChannelSender[PipeReader].send`).
   - `process_pipe_reader_shared_rejected.sifr` -> `SIFR-OWN-0012` (`Shared(reader)`).

4. **Validation evidence refreshed.** The execution ledger entry for "M4 process handle boundary diagnostics targeted local validation" records `cargo fmt`, `cargo fmt --check`, `git diff --check`, both guardrail scripts, `cargo check -p sifr_lowering -p sifr_driver -p sifr --quiet`, direct fixture checks for all six new fixtures, `cargo test -p sifr test_e2e_fail` (`431 fail tests completed`), the broad non-lane `test_e2e_pass` probe (transparently labeled as failing in unrelated text/I/O fixtures and not accepted as a wave gate), and `scripts/run_all_tests.sh --profile create-pr` (PASS; `105 passed, 0 failed, cache_hits=23/27`). Wall-time and warm-cache advisories are recorded as advisories, not failures.

5. **Doc honesty.** `verification/stdlib/concurrency_runtime_m4_process_traceability.md` adds the six new fail fixtures to the fail-suite validation row (`process_pipe_reader_task_boundary_rejected`, `process_async_child_task_boundary_rejected`, `process_pipe_writer_spawn_blocking_capture_rejected`, `process_child_spawn_cpu_return_rejected`, `process_pipe_reader_channel_element_rejected`, `process_pipe_reader_shared_rejected`). Follow-up boundaries continue to list public async owned pipes, top-level async kill/terminate helper shape, cancellation-safe observation, termination escalation, non-Unix signal status evidence, parent cancellation, scoped supervision (`TaskGroup.spawn_process`), and full subprocess text mode closeout as still-open M4 work — no false claim that this wave delivers them.

6. **No user-triggerable panic paths.** The new classifier branch is a pure pattern match on class name returning `Option<&'static str>`. No `.unwrap()`, no `.expect()`, no `assert!` introduced. Diagnostic emission paths in `ownership_diagnostics` and `expression_diagnostics` are unchanged.

7. **File-size guardrail.** `crates/sifr_lowering/src/lower/task_scope_calls.rs` is at 536 lines (under the 900-line cap). The execution ledger records `python3 scripts/check_file_size_guardrails.py` -> PASS over 2202 files and `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.

8. **Phase scope honesty.** Execution ledger adds `M4 process handle boundary diagnostics: in progress.` to the PR list, an in-progress implementation block, and a targeted validation block. No claim of merge or completion is made.

## Recommendation

PASS. The only outstanding item is staging hygiene: keep the four unrelated network/HTTP files (two modified `issues/...` and two untracked review pass artifacts) out of this PR's commit set. Once staged correctly, the wave is ready to open as a PR.
