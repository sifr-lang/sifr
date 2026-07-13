I've reviewed the full current diff plus the design-round-1 and implementation-round-1 review artifacts. Summary of what I re-verified in the current tree:

**Cancellation lease lifetime** (`cancellation.rs:130-145`, `async_terminal.rs:55-70`): lease drop occurs while the terminal-state mutex is held, before `changed.notify_all()` and `waker.wake()`. Generation tag with `wrapping_add(1).max(1)` guarantees stale-drop no-ops. Poisoned-mutex tolerance (`unwrap_or_else(PoisonError::into_inner)`) closes the design-round-1 minor observation about lease Drop under a poisoned mutex.

**Pending/active handoff** (`async_runtime.rs:539-611`): `pending_submissions` is `BTreeMap<u64, PythonTerminal>`; `register_submission` atomically moves the terminal from pending into `submissions` under `ASYNC_STATE`. All error paths in `submit_coroutine` (queue-fail, done/setup callback build, `call_soon_threadsafe` fail) select by exact submission id; setup-callback panic containment routes through `finish_submission`/`release_pending_submission` correctly (`build_setup_callback` at 285-348).

**Drain on failure** (`async_runtime.rs:632-686`): `drain_outstanding_submissions` and `fail_live_runtime` `mem::take` both maps under the lock and complete terminals outside it; `active` is dropped inside `Python::try_attach(|_py| drop(active))` so `loop_object`/`exact_task` DECREFs run under the GIL.

**Loop-failure detection in shutdown waits** (`async_runtime.rs:366` and `614-628`): both the pending-drain wait and `wait_for_submissions_to_drain` re-check `AsyncLifecycle::Failed` on every wake, closing the design-round-1 nit.

**Ordered shutdown + first-error priority** (`async_runtime.rs:355-437`): admissions-off → callback shutdown → async cleanup → submission cancel+drain → loop stop → loop join. `retain_first_error` preserves the earliest error while `record_shutdown_phase` proves every subsequent phase still runs. The `shutdown_hooks` module is minimal and cleanly encapsulates the phase-2/3 no-op-plus-test-hook seams.

**Supervisor + JoinSet + carriers**: gather/race/select/scope-fail-fast all drive `cancellation.request_cancel()` (fallback = `handle.abort()` via `__SifrCancellationCarrier::new`); `__SifrJoinEntry { cancellation, blocking_abort }` split keeps async entries carrier-owned and blocking/CPU entries abort-owned; process observers keep `stop_on_fail_fast`. `__sifr_join_all` fail-fast still awaits every observer before returning.

**Codegen shape pins**: `test_join_set_preserves_task_cancellation_carrier_until_terminal_drain` asserts `cancellation: Some(cancellation)` + `blocking_abort: None` and forbids `cancellation.abort_handle()`. `test_task_handle_cancel_uses_cooperative_carrier_with_abort_fallback` pins `cancellation: __SifrCancellationCarrier` and the `__sifr_current_task_cancellation` seam.

**File sizes**: `task_runtime.rs` 810, `async_runtime.rs` 795, `python.rs` 872, others ≤ 570 — all under the 900-line guardrail.

**Taxonomy rename**: `test_config("ordered-shutdown-errors")` at `async_runtime_tests.rs:403` is only a `probe_digest` label string; the test function name, forced-failure hooks, expected phase-trace, and terminal-drain assertion are unchanged — production semantics unaffected.

**Minor stylistic nit (not actionable):** `fail_live_runtime` at `async_runtime.rs:665` passes `&message` (a `&&str`) to `complete_drained_submissions` while `drain_outstanding_submissions` at line 646 passes `message` directly. Compiles via auto-deref; consistency-only.

No new actionable defects. Round-1 verdicts (both design and implementation) remain valid on the current tree.

VERDICT: SATISFIED
