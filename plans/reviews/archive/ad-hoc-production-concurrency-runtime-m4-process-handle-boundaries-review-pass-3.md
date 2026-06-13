RESULT: PASS

# Review: Concurrency Runtime M4 Process Handle Boundary Diagnostics - Pass 3 (post-rebase)

Branch: `codex/concurrency-runtime-m4-pipe-handle-boundaries` rebased onto `origin/main` after PR #2378 (top-level async child kill/terminate) and PR #2381 (async owned process pipes) merged.

## Pass-2 follow-up status

- **Out-of-scope HTTP/network files**: resolved in this worktree. `git status --porcelain` shows only in-scope files staged: `crates/sifr_lowering/src/lower/task_scope_calls.rs`, the nine new `crates/sifr/tests/e2e/fail/process_*.sifr` fixtures, `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`, `verification/stdlib/concurrency_runtime_m4_process_traceability.md`, and the pass-1 and pass-2 review artifacts. The only untracked entry is this pass-3 review placeholder, which is the artifact being written. No network/HTTP files leaked into the PR worktree.
- **Async pipe handle coverage gap**: resolved. After the rebase onto current main (which lands public `AsyncPipeReader` / `AsyncPipeWriter` via PR #2381), the wave added three async pipe fixtures so the classifier is pinned across both sync and async owned-pipe surfaces:
  - `process_async_pipe_reader_task_boundary_rejected.sifr` -> `SIFR-OWN-0010`
  - `process_async_pipe_writer_channel_element_rejected.sifr` -> `SIFR-OWN-0011`
  - `process_async_pipe_writer_shared_rejected.sifr` -> `SIFR-OWN-0012`

## Scope verified in this pass

- Central lowering change in `crates/sifr_lowering/src/lower/task_scope_calls.rs` (538 lines, under the 900-line cap).
- Nine fail fixtures under `crates/sifr/tests/e2e/fail/` exercising `SIFR-OWN-0010` (3 sync + async task-boundary, 1 offload capture), `SIFR-OWN-0011` (sync + async channel element), `SIFR-OWN-0012` (sync + async shared publication), and `SIFR-TYPE-0002` (sync `Child` `spawn_cpu` return).
- Doc updates in `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` (records PR #2381 merged, this wave in progress, implementation block, targeted validation block, prior review-loop log) and `verification/stdlib/concurrency_runtime_m4_process_traceability.md` (status sentence, PipeReader/PipeWriter/AsyncPipe rows, fail-suite row, follow-up list).
- Pass-1, pass-2, and (this) pass-3 review artifacts.

## Findings

### Blockers
None.

### High
None.

### Medium
None.

### Low (non-blocking)

- **Post-rebase create-pr lane has not been rerun on this exact tree.** The pre-rebase create-pr lane in the ledger PASSed twice with `report_signature=d08ce200366c588c`, but a fresh `scripts/run_all_tests.sh --profile create-pr` against the rebased tree (which now includes PR #2378 and PR #2381 substrate) is not in the ledger yet. The user explicitly flagged this as not blocking for the code/docs shape; the focused validation set (`cargo fmt`, `cargo fmt --check`, `git diff --check`, both guardrail scripts, `cargo check -p sifr_lowering -p sifr_driver -p sifr --quiet`, direct fixture checks on all nine fixtures, `cargo test -p sifr test_e2e_fail` with 434 fail tests completed) covers the surface this wave actually changes. Recommend recording the fresh post-rebase create-pr run before merge for the same evidence shape the earlier waves use.

- **Pass-1 and pass-2 review artifacts predate the post-rebase async-pipe fixture additions.** Pass-1 cites "Five new fail fixtures" and pass-2 cites "Six new fail fixtures", whereas the rebased tree carries nine. These are honest historical records of what each pass actually reviewed and explicitly hand off to follow-up passes — pass-3 (this artifact) covers the post-rebase delta. Not a correctness issue; flagging only so reviewers reading the artifacts in order can locate the bridging step.

- **Bare-name class matching collision surface (carried over from pass-1 and pass-2).** `process_handle_type_label_by_name` matches unqualified class names (`Child`, `AsyncChild`, `PipeReader`, `PipeWriter`, `AsyncPipeReader`, `AsyncPipeWriter`). Consistent with the existing `is_share_safe_sync_wrapper`, `sync_guard_type_label_by_name`, and `class_has_non_send_marker` patterns. `Child` is more likely than the others to collide with a user-defined class. Not a regression introduced by this wave; long-term consider namespacing stdlib-owned class names in the classifier.

## Verification against the review goals

1. **Central classifier hook covers all six process handle surfaces.** Verified at `crates/sifr_lowering/src/lower/task_scope_calls.rs:423-433`. Both `non_send_reason_inner` (line 350-352) and `non_share_safe_reason_inner` (line 296-298) call `process_handle_type_label_by_name` as the first matched-class check. The function matches `public_type_name(name)` against `Child`, `AsyncChild`, `PipeReader`, `PipeWriter`, `AsyncPipeReader`, and `AsyncPipeWriter`. These reasons flow uniformly to: task-spawn argument boundary (`non_send_task_boundary_argument`), offload worker captures (`offload_worker_captures.rs`), offload/CPU return types (`task_calls.rs`), channel-element sends (`validate_channel_send_element`), nested field walks, and `Shared(...)` construction (`validate_shared_constructor`). Branch ordering is correct: in `non_share_safe_reason_inner` the process-handle check fires before `is_share_safe_sync_wrapper`, so no compat-named wrapper short-circuit can mask a process handle.

2. **Process handle classes are correctly non-send/non-share for current M4 semantics.** After the rebase onto current main, `AsyncPipeReader` and `AsyncPipeWriter` are part of the public surface (PR #2381 merged). All six handles wrap an opaque `_handle: int` that indexes a private generated handle table tied to the local runtime (`std::process::Child` for sync, `tokio::process::Child` for async, pipe FDs for the readers/writers). M4 has not yet introduced cancellation-safe observation or scoped process supervision, so moving these handles across runtime task boundaries or publishing them via `Shared` would invite the exact races those later waves address. Treating them as task-bound now is the right substrate behavior.

3. **All nine fail fixtures emit the intended diagnostic codes.** The user's focused validation set (direct `check` on each fixture) confirms:
   - `process_pipe_reader_task_boundary_rejected.sifr` -> `SIFR-OWN-0010` (`scope.spawn(drain(reader))` with `own reader: PipeReader`)
   - `process_async_child_task_boundary_rejected.sifr` -> `SIFR-OWN-0010` (`scope.spawn(observe(child))` with `own child: AsyncChild`)
   - `process_pipe_writer_spawn_blocking_capture_rejected.sifr` -> `SIFR-OWN-0010` (nested `@blocking_io` closure captures `writer: PipeWriter`)
   - `process_child_spawn_cpu_return_rejected.sifr` -> `SIFR-TYPE-0002` (`@cpu_heavy` returning `Child`; offload-return-type check intentionally uses `TYPE_MISMATCH` rather than `OWN_NON_SEND`)
   - `process_pipe_reader_channel_element_rejected.sifr` -> `SIFR-OWN-0011` (`ChannelSender[PipeReader].send(reader)`)
   - `process_pipe_reader_shared_rejected.sifr` -> `SIFR-OWN-0012` (`Shared(reader)`)
   - `process_async_pipe_reader_task_boundary_rejected.sifr` -> `SIFR-OWN-0010` (`scope.spawn(drain(reader))` with `own reader: AsyncPipeReader`)
   - `process_async_pipe_writer_channel_element_rejected.sifr` -> `SIFR-OWN-0011` (`ChannelSender[AsyncPipeWriter].send(writer)`)
   - `process_async_pipe_writer_shared_rejected.sifr` -> `SIFR-OWN-0012` (`Shared(writer)`)
   Each fixture relies on a single, predictable lowering check and no runtime behavior.

4. **No user-triggerable panic paths.** `process_handle_type_label_by_name` is a pure pattern match on the class name returning `Option<&'static str>`. No `.unwrap()`, no `.expect()`, no `assert!`. The `format!` reasons use static string labels for the public type name. `ownership_diagnostics` and `expression_diagnostics` emission paths are unchanged. No generated-runtime code is affected.

5. **Phase scope honesty.**
   - `verification/stdlib/concurrency_runtime_m4_process_traceability.md` status sentence correctly records PR #2378, PR #2381 as merged and "process handle boundary diagnostics are the current wave". The PipeReader/PipeWriter rows now explicitly state "Boundary diagnostics reject moving pipe readers/writers across task, channel, offload, and shared-state boundaries." The AsyncPipeReader/AsyncPipeWriter row adds the same statement while honestly preserving "cancellation-safe observation and scoped supervision remain later M4 work." The fail-suite row in the validation table lists all nine new fixtures. The follow-up list correctly removes only the now-closed pipe-handle sendability/shareability item and keeps cancellation-safe observation, termination escalation / non-Unix signal status / parent cancellation / supported-host matrix, scoped process supervision (`TaskGroup.spawn_process`), full subprocess text-mode closeout, and the drop-cleanup notice intact.
   - `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` records PR #2381 as merged, this wave as "in progress", and adds an implementation block, a targeted validation block, and a review-loop block referencing pass-1 and pass-2. No claim of merge or completion is made for this wave.

6. **Current-main rebase handling.**
   - Execution ledger PR list shows PR #2378 and PR #2381 as merged URLs, and this wave as "in progress" with the correct title.
   - Traceability status sentence enumerates PR #2378 and PR #2381 as merged.
   - Async pipe handles are now covered by three new fixtures, so the classifier expansion is not silently leaving the public async owned-pipe surface unpinned.
   - No conflict markers remain in any tracked file (`<<<<<<<`, `=======`, `>>>>>>>` scan over `*.rs`, `*.sifr`, `*.md` returned no hits).

7. **Validation evidence.** The targeted local validation block (refreshed in the execution ledger) records:
   - `cargo fmt` -> PASS; `cargo fmt --check` -> PASS; `git diff --check` -> PASS.
   - `python3 scripts/check_file_size_guardrails.py` -> PASS over 2206 files; `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
   - `cargo check -p sifr_lowering -p sifr_driver -p sifr --quiet` -> PASS.
   - Direct `check` for each of the nine fixtures -> PASS with the intended diagnostic code.
   - `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; 434 fail tests completed.
   - Pre-rebase `scripts/run_all_tests.sh --profile create-pr` -> PASS twice (`105 passed, 0 failed, report_signature=d08ce200366c588c`). A fresh post-rebase create-pr lane is acknowledged as still to be recorded; non-blocking for this review pass.

8. **File-size guardrail.** `crates/sifr_lowering/src/lower/task_scope_calls.rs` measured at 538 lines, well under the 900-line cap.

9. **PR hygiene.** This worktree's `git status` confirms only in-scope files are staged: nine fail fixtures, the lowering change, the two doc updates, and the pass-1/pass-2 review artifacts; the only untracked file is this pass-3 review. No unrelated network/HTTP changes are present. The user noted their original worktree still has the unrelated network/HTTP files dirty; those are correctly absent from this PR worktree.

## Recommendation

PASS. The post-rebase tree carries a well-scoped centralization of process-handle non-send/non-share classification, nine fail fixtures that pin all targeted boundaries (sync and async, task-boundary / offload capture / offload return / channel element / shared publication), no panic paths in user code, file-size and HIR guardrails green, traceability scope-honest about both delivered coverage and remaining M4 follow-ups, and no out-of-scope files in the worktree. The one remaining housekeeping item is to record a fresh post-rebase `scripts/run_all_tests.sh --profile create-pr` lane in the execution ledger before merge; the user has explicitly flagged this as non-blocking for the review of code/docs shape.
