RESULT: PASS

Re-check of the current git diff for milestone M4 scoped process supervision. The diff set is unchanged from pass-1 (same 20 modified files plus the new `process_scoped_spawn_handle.sifr` fixture); no new correctness concerns surfaced on re-inspection. Restating the outcome from pass-1: implementation is acceptable, with only non-blocking residual risks to track for future passes.

Outcome: PASS. No blockers in the current diff.

Key correctness traces (re-verified):

- **Scope-exit observer deferral** (`crates/sifr_codegen/src/preamble/task_scope_offload_runtime.rs:218-248`). Observer awaits `__start_receiver` before touching `__SIFR_PROCESS_ASYNC_CHILDREN`, so user code in the scope body can call `handle.stdin()/stdout()/stderr()` without racing it. Start signals are sent inside `__sifr_join_all` for every child before awaiting, in both fail-fast and sequential paths (`task_runtime.rs:642-655`, `task_runtime.rs:702-705`).

- **Explicit-wait / observer interaction** (`process_async_child_runtime.rs:556-575`). `process_handle_wait_body` sets the `observed` AtomicBool before invoking the cancellation-safe `__sifr_process_async_wait`, then drops the `observed_children` entry. On wait cancellation, `__SifrAsyncChildWaitGuard` (`process_async_child_runtime.rs:521-528`) restores the child to the table and the observer reaps via the `Some(mut __child)` branch; on success, the `None if observed.load() => Ok` branch prevents double-reaping and spurious unobserved-failure.

- **Fail-fast process kill** (`task_runtime.rs:686-694`, `task_scope_offload_runtime.rs:249-251`). `stop_on_fail_fast` closures are moved out of the child into a vec before `join_set.spawn`, outliving the spawned join future. The observer's `tokio::select!` drains `__stop_receiver` and calls `start_kill()` then `wait()` to reap; outcome `Cancelled` is suppressed under `policy_cancelling`, preventing kill reclassification as scope failure. Fixture step 8 (`crates/sifr/tests/e2e/pass/process_scoped_spawn_handle.sifr:75-82`) covers this.

- **Send-bound drop quirk is benign**. The `_ = &mut __stop_receiver` arm fires on sender drop, not just `send`. Sequential path: `child.stop_on_fail_fast` is not taken and stays alive across `child.handle.await`, so the sender outlives the observer. Fail-fast path: closures live in the local `stop_on_fail_fast` vec until either popped (sends) or end of `__sifr_join_all` after `join_next` has drained — after all observers finished. No spurious kill window.

- **Preamble emission split** (`lib_modules_and_codegen.rs:418-625`, `stdlib_filter/implementation.rs:351-432`). Adding `needs_spawn_function` and `needs_handle_wait` lets `ProcessHandle.wait()` users emit `__sifr_process_async_wait` + child/pipe helpers without dragging in public `__sifr_process_async_spawn`. `module_uses_task_scope_process` forces `needs_spawn`/`needs_wait` on for scoped processes, required because the inlined raw-string body in `scoped_process_body` is not visited by the syn-based collector.

- **Boundary diagnostics** (`task_scope_calls.rs:431`, plus `task_group_spawn_owner` skipping `__sifr_scope_spawn_process`). `ProcessHandle` is classified as a non-send process handle, matching `PipeWriter`/`AsyncPipe*` treatment; cross-task/channel/offload movement is rejected.

- **Fixture coverage** (`process_scoped_spawn_handle.sifr`). Eight expected `True` values map cleanly to pipes, explicit `wait` success/code, second-wait closed-handle rejection, unobserved-success scope aggregation, and fail-fast kill marker absence. Manifests updated in both create-pr and merge lanes.

- **Doc consistency**. M4 traceability adds the scoped supervision row and removes the "scoped supervision remains later M4 work" qualifier from the AsyncChild/AsyncPipe rows; supported-host matrix adds the host-limited Windows note consistent with peers; issue ledger captures implementation + validation.

Non-blocking residual risks (worth tracking but not blockers):

- Sequential (non-fail-fast) scope path starts observers serially via `start.send()` inside the pop loop, so multiple scoped processes are reaped one-at-a-time even if all have already exited. Resource-use suboptimality, not a correctness defect.
- If `__sifr_join_all` is never reached (parent task aborted before scope cleanup), the `__SIFR_PROCESS_ASYNC_CHILD_OBSERVED` entry leaks for that handle. Consistent with existing scope-drop behavior and bounded by process lifetime; worth a future pass if/when scope `Drop` cleanup is wired up.
- `process_handle_wait_body` cleans up the observed-children map after the await; if the wait future itself is cancelled, the map entry remains until the scope-exit observer drains it. Functionally fine since the observer always removes on exit, just a minor delayed-cleanup.
