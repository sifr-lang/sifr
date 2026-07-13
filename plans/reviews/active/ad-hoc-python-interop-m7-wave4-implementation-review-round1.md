I have all the information needed. Let me finalize the review.

## Review — M7 Wave 4 (cancellation-aware supervisors + ordered shutdown)

### Correctness / races / ordering — clean

- **Claim lease invariant** (`cancellation.rs:130-145`): the lease drops out of `PythonTerminal::complete` **after** the terminal-state mutex is released and **before** `changed.notify_all()`/`waker.wake()` (`async_terminal.rs:55-70`). Because standard futures and condvar waiters can only re-observe state after their wake fires, a woken caller cannot start the next `await` and re-claim before the incumbent lease is gone. Generation-tagged drop makes stale releases no-ops. The `next_generation = ...wrapping_add(1).max(1)` avoids generation 0 without introducing a collision, since `exact` is exclusive at any moment.

- **Pending/active handoff** (`async_runtime.rs:539-589`): `pending_submissions` is `BTreeMap<u64, PythonTerminal>` keyed by exact submission id; the atomic move in `register_submission` is symmetric with the earlier `reserve_submission` insert, both under `ASYNC_STATE`. Every error path in `submit_coroutine` (`async_runtime.rs:208-241`) and in the setup callback (`async_runtime.rs:333-343`) selects the right unwind by id, and neither leaves a stranded terminal — `release_pending_submission` and `finish_submission` both notify `ASYNC_STATE_CHANGED`.

- **Terminal drain on failure** (`async_runtime.rs:632-686`): `drain_outstanding_submissions` and `fail_live_runtime` `mem::take` both maps under the lock, then `terminal.complete(Err(runtime failure))` outside the lock so terminal leases release cleanly. Active `RegisteredSubmission`s are dropped inside `Python::try_attach(|_py| drop(active))`, giving the loop_object/exact_task Py handles GIL for their DECREF.

- **Loop-failure detection inside shutdown waits** (`async_runtime.rs:366` and `614-628`): the pending-drain wait and `wait_for_submissions_to_drain` both re-check `AsyncLifecycle::Failed` on every wake — this closes the Wave-3 design-review nit that the runtime could deadlock if the loop panicked mid-shutdown.

- **Shutdown phase order + first-error priority** (`async_runtime.rs:355-437`): admissions-off → callback shutdown → async cleanup → cancel+drain → loop stop → loop join → epilogue. `retain_first_error` correctly retains the earliest error while `record_shutdown_phase` proves every subsequent phase still ran (validated by `shutdown_errors_do_not_skip_cancel_drain_stop_or_join`).

- **Supervisor arbitration** (`task_supervisor_runtime.rs`, `task_runtime.rs:642-708`): gather/race/select and scope fail-fast all switched from `.abort()` to `cancellation.request_cancel()`, driving the exact hook (Python) or the fallback abort (ordinary Sifr tasks). Ordinary-task semantics are unchanged because `__SifrCancellationCarrier::new` bind_fallback = `child.abort_handle()`. Process observers stay on `stop_on_fail_fast` (`task_scope_offload_runtime.rs:237`), so blocking/CPU paths are untouched.

- **JoinSet split** (`join_set_runtime.rs:149-170, 240-303`): `__SifrJoinEntry { cancellation, blocking_abort }` — `add(__SifrTask)` stores the full carrier and never extracts an abort handle; blocking/CPU spawns store the abort handle and set `cancellation: None`. `__sifr_cancel_all` selects cleanly and idempotently, and observer wrappers convert cancellation into `TaskResult::Cancelled` at the receiver boundary.

### PyO3 / GIL

Every Py<PyAny> drop that could happen outside GIL either lands inside `Python::try_attach(|_py| drop(active))` or is a value with no Py handle (pending terminals). PyO3's deferred DECREF makes the fallback (try_attach None) safe.

### Tests genuinely prove the contract

The runtime tests cover claim before/after cancel, sequential-await re-claim after complete (`completed_submission_releases_carrier_for_sequential_python_await`), independent-carrier isolation (`independent_exact_tasks_cancel_without_cross_talk`), terminal-lease-before-wake (`terminal_completion_releases_claim_before_waking_waiter`), live-loop failure drain (`live_loop_failure_terminally_drains_pending_reservations`), shutdown drain of claimed exact tasks with `finally` fired, first-error-with-all-phases-still-run, pending-by-id unwind, callback-panic containment, and 128-round contended claim/request. Codegen tests pin the supervisor/join-set/task shapes (`request_cancel`, `cancellation: Some(...)`, `blocking_abort: None`, no `abort_handle()` on async JoinSet entries, `__sifr_current_task_cancellation` seam present but not user-reachable).

### File sizes

`task_runtime.rs` 810 / `async_runtime.rs` 795 / `python.rs` 872 — all under 900.

### Minor observations (non-blocking)

1. `join_set_runtime.rs:96-100`: with async JoinSet entries cancelled cooperatively, the observer no longer produces `JoinError::cancelled`, so `CancelOutcome::AlreadyStarted` narrows to blocking entries only. Design review flagged this as documentation-level; the `CancelOutcome` enum has no docstring today, so nothing needs to change unless one is added.
2. `cancellation.rs:97`: `state.next_generation.wrapping_add(1).max(1)` — correct (0 is reserved so a lease drop after wrap cannot accidentally clear a fresh exact slot with generation 0), but a one-liner explaining "skip 0 across wrap" would help future readers.
3. Wave 3 round-1 nit that `run_coroutine_blocking` should `debug_assert!(Handle::try_current().is_err())` remains open; the design review explicitly does not require Wave 4 to close it.

No changes required by the review.

VERDICT: SATISFIED
