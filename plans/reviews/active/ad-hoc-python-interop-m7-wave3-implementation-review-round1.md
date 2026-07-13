I have enough context. Writing the review now.

# Review — M7 Wave 3 Cancellation Carrier Implementation (Round 1)

## Round-2 design contract vs. current code

| Design point | Where it lives | Verdict |
|---|---|---|
| §A/§D Py refs dropped outside mutex, submission BTreeMap removes then drops | `python/async_runtime.rs:509-518` `finish_submission` locks, `remove`, `notify_all`, releases lock, then `drop(removed)`. `cancel_submission`/`cancel_registered_submissions` clone-under-lock, release, then call. ✓ | Matches |
| §A setup catches all PyErr and panics; cancels created task; resolves latch; unwinds accounting | `python/async_runtime.rs:296-334` — `catch_unwind(AssertUnwindSafe(...))` around the whole setup body; `if registered { finish } else { release }`; `if Some(task) { cancel }`; `terminal.complete(Err(_))`. ✓ | Matches |
| §A queue failure between reserve and submit unwinds pending | `python/async_runtime.rs:201-233` — every step that can fail before `call_soon_threadsafe` returns calls `release_pending_submission` on failure. ✓ | Matches |
| §D shutdown drain: queue `task.cancel` per registered id, wait on Condvar until `submissions.is_empty()`, then stop and join | `python/async_runtime.rs:346-395`. ✓ | Matches |
| §B `PythonTerminal` = mutex + Condvar + waker, `py.detach(\|\| terminal.wait())`, no runtime-blocking sync path | `python/async_terminal.rs:32-96`, `python/async_runtime.rs:163-172`. `wait()` uses Condvar; `Future::poll` stores/overwrites waker only when `!will_wake`. ✓ | Matches (round-2 §3 explicit re-poll rule was requested — implemented) |
| §C claim / publish / request are atomic under the bridge mutex; hook fires outside the mutex | `python/async_runtime.rs:77-101` — `publish` sets id then reads requested; `request` sets requested then reads id, both under the same lock; `cancel_submission` is called outside the lock. `cancellation.rs:76-111` — `request_cancel` releases lock before invoking hook. ✓ | Matches |
| §E `__SIFR_COOPERATIVE_SUPERVISORS_READY: bool = false` gates claim; supervisors debug-panic on unexpected claimed carrier | `preamble/task_cancellation_runtime.rs:9-61`. `__sifr_claim_current_task_cancellation` short-circuits when the const is false; `__SifrCancellationCarrier::abort` `debug_assert!`s and, in release, promotes an unexpected claim into `request_cancel` (best-effort). ✓ | Matches |
| §E `__SifrBlockingTask` carrier-free | `preamble/task_runtime.rs:26-40`, `255-303` — struct fields `handle/observed/_error` only; `cancel`/`cancel_and_join` still call `handle.abort()`. ✓ | Matches |
| §F `cancel_and_join` → always Cancelled; `timeout` in wave 3 → always Timeout (Claimed branch dark until wave 4) | `preamble/task_runtime.rs:224-252`. Since supervisors gate leaves claim off, `request_cancel()` in `__sifr_timeout` returns `Fallback`, so the `matches!(_, Claimed)` branch is unreachable in wave 3 but wired forward-compatibly for wave 4. ✓ | Matches |
| §G JoinSet keeps wrapper `JoinHandle` + extracts child carrier fallback via `cancellation.abort_handle()` | `preamble/join_set_runtime.rs:250`, `319`. ✓ | Matches |
| §H task_runtime.rs split by responsibility, all files well under 900 | `task_runtime.rs` 795, `task_cancellation_runtime.rs` 64, `task_supervisor_runtime.rs` 93. Under cap. | Under cap; less granular than round-2 §H sketched (no separate `task_types` / `task_impl` / `task_scope_spawn` / `task_scope_join`) but still passes guardrail |

The wave-3 boundary invariants — carrier field on `__SifrTask`, no claim ever reaching supervisors, blocking task carrier-free, JoinSet unchanged behavior — are enforced by codegen and asserted by tests `test_task_handle_cancel_uses_cooperative_carrier_with_abort_fallback` and `test_join_set_extracts_abort_fallback_from_task_cancellation_carrier`.

## CancellationCarrier linearization

The state machine at `cancellation.rs:8-125` is correct.

- `request_cancel` under one lock: check requested → set requested → snapshot exact-or-fallback hook Arc → drop lock → invoke hook. Winners are chosen atomically; late fallback binding after a pending request replays via `bind_fallback` returning `InvokedPendingCancellation`.
- `claim` under one lock: check requested → check already claimed → install exact hook. The `Claimed vs CancelledBeforeClaim` pair is racy-safe because both readers snapshot inside the same critical section.
- The 128-round barrier test (`contended_claim_and_request_choose_exactly_one_path`, cancellation.rs:216-262) actually exercises the race and asserts that exactly one of `(Claimed,Claimed)` or `(CancelledBeforeClaim,Fallback)` occurs and exactly one hook fires. That's the strongest form of the contract.
- Hook Arc capture: `CancellationHook = Arc<dyn Fn() + Send + Sync + 'static>`; the wrapping `__SifrCancellationCarrier::new` closure captures a `tokio::task::AbortHandle` (Send + Sync + Clone). No `Py<PyAny>` ever enters the carrier, so drop paths do not need GIL.

Poisoned-lock behavior: `bind_fallback`, `claim`, `request_cancel` all return distinct `StateUnavailable` outcomes so callers can observe the fault. `publish` treats poisoning as "assume requested → cancel the newly created task" — the safe direction. `SubmissionCancellationBridge::request` silently swallows poisoning, but by that point the outer carrier has already latched requested=true; the loss is the bridge's cancel dispatch, not carrier consistency.

## Asyncio task publication / cancellation

- `submit_coroutine` (`async_runtime.rs:174-235`) claims the carrier before reserving a submission id, so a rejected claim never reserves a slot; reservations that fail after a successful claim leave the carrier "claimed with a bridge that has `submission_id = None`" — safe, subsequent `request_cancel` just latches `requested=true` and no-ops.
- Setup callback (`async_runtime.rs:277-339`): `create_task → add_done_callback → register_submission → publish → optional inline cancel`. Every failure branch (a) removes registration if it happened, (b) releases pending if it didn't, (c) cancels the created task if it exists, (d) completes the terminal with the error. `catch_unwind(AssertUnwindSafe(...))` handles a native panic and converts to `PythonRuntimeError::AsyncRuntimeFailed`.
- Done callback (`async_runtime.rs:237-274`): `catch_unwind` around `task.result()`, then unconditional `finish_submission` + `terminal.complete`. Panic path yields `AsyncRuntimeFailed`, the submission is still removed, and the terminal is still resolved. Round-2 nice-to-have #1 (done-callback error containment) is implemented — a panic maps to `Err(Runtime(AsyncRuntimeFailed))` and the entry is removed.
- Terminal-latch single-completion: `PythonTerminal::complete` (`async_terminal.rs:42-56`) takes the mutex, checks `outcome.is_some()`, returns `false` if already set. Late setup errors and late done-callback errors both no-op on the terminal.
- GIL-detached blocking wait: `run_coroutine_blocking` uses `super::detach(py, || terminal.wait())`, which delegates to `py.detach`. No `oneshot::blocking_recv` inside a Tokio runtime.
- Shutdown drain: `shutdown` transitions to Stopping (rejecting new reservations), waits for `pending_submissions == 0`, takes both `loop_object` and `loop_thread` out of state (Py drop is not held under mutex), queues per-submission cancels via `call_soon_threadsafe`, waits for `submissions.is_empty()`, then queues `loop.stop` and joins the thread.
- Loop-thread failure recovery: `run_loop_thread` either sends `Err(message)` on the readiness channel (start observes and calls `fail_start`) or sets `AsyncLifecycle::Failed` under lock, so `shutdown()` can still transition Failed → Stopping. `fail_start` clears `loop_object`/`loop_thread` on the failure path.
- Registry Py drops: `cancel_registered_submissions` and `cancel_submission` clone `(loop_object, exact_task)` under the state lock, release the lock, then call `queue_exact_task_cancel` inside `Python::try_attach` so drops happen with the GIL held. `finish_submission` moves `removed` out of the locked section and drops explicitly.

## Python cancellation suppression and finally ordering

- `cancellation_test_module` (`async_runtime_tests.rs:353-361`) provides `cancellable` (uses `finally:` to set a marker after `CancelledError`) and `suppresses` (swallows `CancelledError` and returns 73).
- `exact_task_cancellation_runs_finally_before_terminal_completion` observes `marker.len() == 1` after `terminal.wait()` returns, proving `finally` runs before the done callback resolves the terminal.
- `cancellation_suppression_result_wins_after_terminal_wait` observes the integer 73 out of `terminal.wait()`, proving Python's return value wins over the request when the coroutine suppresses.
- `independent_exact_tasks_cancel_without_cross_talk` proves per-carrier isolation.
- `shutdown_terminally_drains_claimed_task_and_finally` proves shutdown honors the same ordering (marker set, terminal resolved as Python-cancelled).

## Codegen seams

- `__SifrTask<T, E>` now has `cancellation: __SifrCancellationCarrier` (no `abort_handle` field); `__SifrBlockingTask<T, E>` has no carrier — matches the accepted design and the snapshot assertions.
- `__SifrCancellationCarrier::new` binds the fallback on a fresh carrier; because `bind_fallback` only returns `AlreadyBound`/`InvokedPendingCancellation` for pre-used carriers and every carrier in the emitted code is fresh, the ignored `Result` is safe.
- Spawn seams (`preamble/task_runtime.rs:395-501`, `502-626`, and `preamble/task_scope_offload_runtime.rs:168-171`) create the inner `CancellationCarrier`, `.clone()` the Arc for the child task-local scope, spawn the child, then wrap parent-side with `__SifrCancellationCarrier::new(cancellation_inner, child.abort_handle())`. All three seams follow the same shape.
- `cancel`, `cancel_and_join`, `__sifr_timeout` all funnel through `cancellation.request_cancel()`.
- Supervisors (`gather`, `race`, `select`, `__sifr_add_task`, `JoinSet::__sifr_cancel_all`) use `cancellation.abort()` / `cancellation.abort_handle()`. Because `__SIFR_COOPERATIVE_SUPERVISORS_READY = false`, no path emits a claim in wave 3 — hard-gated at the const.
- Emission conditions in `entrypoints.rs` and `lib_modules_and_codegen.rs`: `if uses_task_scope || uses_join_set { cancellation + scope + supervisor }`. `uses_task_scope_offload` implies `uses_task_scope` transitively (offload methods only appear inside `__SifrTaskScope`), so the cancellation preamble is always emitted before the offload seam references it.
- File-size guardrail: `task_runtime.rs` 795 lines (well under 900), other new modules small.
- No user-triggerable panic: the `debug_assert!` in `__SifrCancellationCarrier::abort` is a wave-ordering trip-wire, only reachable when a supervisor gets a claimed carrier — impossible in wave 3 because the const gate prevents the emit-side claim. `catch_unwind` on setup and done callbacks contains any native panic and converts to `AsyncRuntimeFailed`. `.unwrap()`/`.expect()` in cancellation/async_terminal/async_runtime.rs are on programmer invariants only.

## Test sufficiency

Rust-only carrier unit tests: 5 (unclaimed-fallback, late-fallback, claimed-request, cancel-before-claim, 128-round contended). Terminal-latch: 2 (blocking wait, future re-poll waker replacement). Runtime tests: 9 (`loop_setup_failure`, `cancellation_before_claim`, `exact_task_cancellation_runs_finally_before_terminal_completion`, `cancellation_suppression_result_wins_after_terminal_wait`, `independent_exact_tasks_cancel_without_cross_talk`, `shutdown_terminally_drains_claimed_task_and_finally`, `invalid_awaitable_setup_resolves_without_leaking_submission_counts`, `submission_queue_failure_releases_pending_reservation`, `terminal_callback_panic_is_contained_and_removes_registration`). Codegen: 15 tests, including two new ones that pin the carrier field, the const gate, the debug-assert message string, the `__SIFR_TASK_CANCELLATION.scope(child_cancellation)` seam, the `cancellation.request_cancel()` cancel body, `handle.__sifr_timeout` claimed-branch code, and the JoinSet `cancellation.abort_handle()` extraction.

Every branch the round-2 design surfaced is exercised: setup panic, queue failure, cancel-before-claim, in-flight cancel with finally, suppression wins, two independent claimed submissions, claimed shutdown drain, terminal panic containment, invalid-awaitable, snapshot proof of the wave boundary.

## Non-blocking observations

1. **`__SifrCancellationCarrier::abort` double-locks in debug builds.** `preamble/task_cancellation_runtime.rs:35-44` calls `self.inner.is_claimed()` inside `debug_assert!`, then again in the `if`. Both acquire the state mutex. In release the assert compiles away and only one lock is taken; in debug two are. A local `let claimed = self.inner.is_claimed().unwrap_or(false); debug_assert!(!claimed, …); if claimed { … } else { … }` collapses this without changing semantics. Not a correctness issue.

2. **`CancellationCarrier::fallback_hook` is currently unused.** `sifr_runtime/src/cancellation.rs:113-118` exposes a getter that no caller invokes yet. Presumably reserved for wave 4/5 supervisor upgrades. If it is not needed then, remove it; otherwise leave a `// TODO(wave4)` marker so it does not read as dead code.

3. **No `debug_assert!(Handle::try_current().is_err())` on `run_coroutine_blocking`.** Round-2 §2 (nice-to-have) asked to lock the invariant that raw sync waiters only run outside a Tokio worker (else `terminal.wait()` deadlocks that worker). The accepted design left it to the `blocking_io`/`explicit_offload` classification. Adding the assert would harden the invariant without behavior change; not required by round-2's acceptance criteria.

4. **Design §H suggested a finer split of task_runtime.rs.** The implementation extracted only `task_cancellation_runtime` and `task_supervisor_runtime`, leaving 795 lines in `task_runtime.rs`. Under the 900-line cap and under the guardrail, but less granular than the design's five-file sketch. Deferrable to a follow-up.

5. **Every `RegisteredSubmission` stores its own `loop_object` clone.** They're all the same loop; a single `Py<PyAny>` at state level would save an Arc per submission. Memory only, not correctness.

6. **`__sifr_cancel_all` fallback branch (`entry.handle.abort()`) leaks async child work in principle.** When the child was carrier-tracked and its `AbortHandle` is stored in `abort_handle`, wave 3 code always uses that branch. The `else` path is only reachable for `__sifr_add_blocking_task` with a `None` handle, which is safe. Noting the invariant so wave 4's supervisor upgrade preserves it explicitly.

## Scope check for wave 4

The seam for wave 4 is exactly the const flip: replace `__SIFR_COOPERATIVE_SUPERVISORS_READY = false` with `true`, wire typed-wrapper lowering to call `__sifr_claim_current_task_cancellation`, and upgrade the four supervisors (`__sifr_task_gather`/`race`/`select`/`__sifr_cancel_all`) to use the exact carrier request+terminal-latch wait path. The current code already has the correct `Claimed` branch in `__sifr_timeout` waiting for that flip. No wave-3 change makes wave 4 harder.

VERDICT: SATISFIED
