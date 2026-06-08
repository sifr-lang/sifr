RESULT: PASS

# Review: Concurrency Runtime M4 Process Handle Boundary Diagnostics - Pass 1

Branch: `codex/concurrency-runtime-m4-pipe-handle-boundaries`

## Scope verified

- Central lowering change in `crates/sifr_lowering/src/lower/task_scope_calls.rs`.
- Five new fail fixtures under `crates/sifr/tests/e2e/fail/`.
- Doc updates in `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` and `verification/stdlib/concurrency_runtime_m4_process_traceability.md`.

## Findings

### Blockers
None.

### High (must fix before PR, but does not invalidate the wave)

- **Working tree contains out-of-scope HTTP/network phase changes** that must not be committed with this M4 process-handle PR:
  - `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
  - `issues/ad-hoc-production-network-http-platform-substrate.md`
  - Untracked: `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-1.md`
  - Untracked: `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-2.md`

  These belong to the network/HTTP platform substrate phase, not the concurrency runtime M4 wave. Stage and commit only the five in-scope files (task_scope_calls.rs change, the five fail fixtures, the two listed docs).

### Medium

- **`reviews/ad-hoc-production-concurrency-runtime-m4-process-handle-boundaries-review-pass-1.md` is an empty 0-byte placeholder** in the working tree. Populate it (this artifact) or remove before commit so the PR does not introduce an empty file.

- **No fail fixture exercises the new share-safety classification** (`non_share_safe_reason` path via `validate_shared_constructor`). The execution-doc entry claims the centralization covers share-safety, and the code in `task_scope_calls.rs:296` proves it, but no fixture pins it. A small `Shared(reader)` / `Shared(child)` rejection fixture (expect `SIFR-OWN-0012`) would close the loop on the central-classifier claim and prevent silent regressions if `is_share_safe_sync_wrapper` or `process_handle_type_label_by_name` ordering is later refactored.

### Low (non-blocking)

- **`cargo test -p sifr -- test_e2e_pass` was not in the validation list.** Manual inspection of all current `crates/sifr/tests/e2e/pass/process_*.sifr` fixtures shows none combine process handles with `scope.spawn`, `task.spawn_blocking`, `task.spawn_cpu`, `Channel.send`, or `Shared(...)`, so the new classifier should not regress them. Run the pass suite once locally before merging to confirm.

- **Bare-name class matching collision surface.** `process_handle_type_label_by_name` matches on the unqualified class name (`Child`, `AsyncChild`, `PipeReader`, `PipeWriter`). This is consistent with the existing pattern used by `is_share_safe_sync_wrapper`, `sync_guard_type_label_by_name`, and `class_has_non_send_marker`, but `Child` is materially more likely to collide with a user-defined class than `Shared`/`LockGuard`/`SemaphorePermit`. Not a regression introduced by this wave; flagging because the named-class pattern is being broadened to more common identifiers. Long-term consider namespacing stdlib-owned class names in the classifier.

- **`is_share_safe_sync_wrapper` runs after the process-handle check now (good)**, but a future reader may wonder whether a `__compat_sifr_sync_PipeReader`-style compat alias could short-circuit through `public_type_name`. Today there is no such compat alias for process handles, so this is theoretical.

## Verification of review goals

1. **Central non_send/non_share classifier placement.** Verified. Both `non_send_reason_inner` (line 345-400) and `non_share_safe_reason_inner` (line 279-343) gain a single early-return through `process_handle_type_label_by_name`. External callers — `blocking_executor_calls.rs`, `task_scope_offload_calls.rs`, `task_join_set_calls.rs`, `task_calls.rs`, `parallel_calls.rs`, `offload_worker_captures.rs`, and the in-file `validate_channel_send_element` and `validate_shared_constructor` — all consume `non_send_reason` / `non_share_safe_reason` and therefore inherit the classification uniformly. Nested fields, tuples, unions, intersections, aliases, newtypes, and parameterized type constructors (Result/Task/Coroutine/etc.) propagate via the existing recursive walks. No overreach: only the four named handle classes are added, and the existing sync-guard / share-safe / NonSend-marker handling is preserved.

2. **Process handle classes correctly non-send/non-share for current M4 semantics.** Verified against `lib/sifr/process.sifr:81-190`. `PipeReader`, `PipeWriter`, `Child`, `AsyncChild` each wrap an opaque `_handle: int` that indexes a private generated handle table (`std::process::Child` for sync, `tokio::process::Child` for async, pipe FDs for the reader/writer). M4 has not yet introduced cancellation-safe observation, public async owned pipes, or scoped process supervision, so moving these handles across runtime task boundaries or publishing them via `Shared` would invite the exact races those future waves are meant to address. Treating them as task-bound now is the right substrate behavior.

3. **Fail fixtures are meaningful and use intended diagnostic codes.**
   - `process_pipe_reader_task_boundary_rejected.sifr` → `scope.spawn(drain(reader))` where `drain` takes `own reader: PipeReader`. Hits `non_send_task_boundary_argument` in `task_scope_calls.rs:106` → `SIFR-OWN-0010`. Correct.
   - `process_async_child_task_boundary_rejected.sifr` → same shape with `AsyncChild`. Correct.
   - `process_pipe_writer_spawn_blocking_capture_rejected.sifr` → nested `use_writer` captures `writer: PipeWriter`; `validate_offload_worker_captures` in `offload_worker_captures.rs:24-34` flags the capture with `OWN_NON_SEND_TASK_CAPTURE` before falling through to the "no captures yet" type-mismatch path. Diagnostic `SIFR-OWN-0010` fires first. Correct.
   - `process_child_spawn_cpu_return_rejected.sifr` → `@cpu_heavy def build_child() -> Child` consumed by `task.spawn_cpu`. `task_calls.rs:174` runs `non_send_reason(&ok_ty)` and emits `TYPE_MISMATCH` (`SIFR-TYPE-0002`). Correct — the spawn_cpu return-type check intentionally uses `type_mismatch` rather than the ownership code, matching the existing convention for offload return-type rejection.
   - `process_pipe_reader_channel_element_rejected.sifr` → `ChannelSender[PipeReader].send(reader)` hits `validate_channel_send_element` → `SIFR-OWN-0011`. Correct.

   None of the fixtures depend on runtime behavior. They each rely on a single, predictable lowering check.

4. **Docs/traceability honesty.**
   - `verification/stdlib/concurrency_runtime_m4_process_traceability.md` removed only the line "Pipe handle sendability/shareability checks beyond the one-shot sync `Child.wait()` state" — which is exactly what this wave delivers. The remaining follow-ups are intact: public async owned pipes, top-level async kill/terminate helper shape, cancellation-safe process observation, termination escalation, non-Unix signal status evidence, parent cancellation evidence, scoped process supervision (`TaskGroup.spawn_process`), and full subprocess text mode closeout. No false claim of public async owned pipes, cancellation-safe observation, or scoped process supervision being complete.
   - The Status sentence updated to "process handle boundary diagnostics are the current wave" — accurate.
   - `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` adds a clearly labeled "in progress" implementation block and a targeted validation block. No overreach.

5. **Regressions / missing tests / size.**
   - `task_scope_calls.rs` is now 537 lines, well under the 900-line cap.
   - No HIR-maintainability-guardrail concerns (user already ran the guardrail script).
   - Pass-suite regression risk surveyed via grep over `crates/sifr/tests/e2e/pass/process_*.sifr`: no pass fixture currently combines process handles with `scope.spawn` / `task.spawn_blocking` / `task.spawn_cpu` / `ChannelSender` / `Shared(...)`, so the new classifications should not break any existing pass fixture. Recommend running `cargo test -p sifr -- test_e2e_pass` once before merging to confirm.

## Recommendation

PASS. Strip the four out-of-scope HTTP/network files from the commit set, populate (or delete) the empty review placeholder, and the wave is ready to PR. Consider adding a `Shared(<process-handle>)` fail fixture and running the e2e pass suite as non-blocking follow-ups.
