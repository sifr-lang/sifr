Confirmed the async_runtime touches only `asyncio` module surfaces (event loops and concurrent futures) — never `PyCapsule` or DLPack. There is no direct or indirect path by which async lifecycle contamination can increment `TEST_DELETER_CALLS`.

## M7 Wave 2 Review — Round 2 Findings

### Round 1 findings — re-audit

**1. HIGH lock-order inversion in `shutdown()` — FIXED.** `async_runtime.rs:146-159` now takes `state.loop_object` and `state.loop_thread` out while holding `ASYNC_STATE`, releases the guard at the block boundary, and only then invokes `Python::try_attach` at line 169. `cancel_registered_submissions` at line 308-316 acquires the GIL first and then `ASYNC_STATE`, matching the worker paths (`reserve_submission`, `register_submission`, `finish_submission` all run under `attach(|py| ...)` from `coroutine_ops.rs:6`). No path exists where one actor holds `ASYNC_STATE` and blocks on the GIL while another holds the GIL and blocks on `ASYNC_STATE`.

**2. MEDIUM concurrent lazy `ensure_started` — FIXED.** `start()` at `async_runtime.rs:61-77` now loops on lifecycle: `Running → Ok`, `Starting → wait_for_change`, `Stopping → Err`, `Disabled|Stopped|Failed → transition to Starting and break to spawn`. `ensure_started` delegates to `start()`. Second concurrent lazy caller now sits on the condvar until the initiator completes and observes `Running`. Spurious wakeups are handled by the outer loop. On start failure, `fail_start` sets `Failed` and notifies; the waiter wakes, re-enters the loop, and retries — which is a reasonable UX for transient failures.

**3. LOW test coverage — ADDRESSED.** `concurrent_raw_coroutines_share_one_owned_loop_and_thread` (`coroutine_ops.rs:60-106`) now leaves `start_async_loop` unset (defaults to `false` per `python_test_support.rs:77`), spawns two workers that block on a 3-party `Barrier`, and races their `ensure_started` calls. `shutdown_cancels_and_joins_an_in_flight_raw_coroutine` (`coroutine_ops.rs:108-151`) waits for a raw caller's submission to register, calls `shutdown()` concurrently, and asserts the caller sees an error and diagnostics return to default.

### Residual concerns (non-blocking)

**A. Shutdown error paths after handles are taken leak the loop thread.** `async_runtime.rs:168-183`: if `Python::try_attach` returns `None`, or `getattr("stop")` / `call_method1("call_soon_threadsafe", ...)` fails, `shutdown` returns early. The `loop_thread` `JoinHandle` in scope is then dropped (detaching the thread), the `loop_object` we already took is dropped, and `state.lifecycle` stays at `Stopping` forever. A subsequent `shutdown()` sees `Stopping`, finds `loop_object`/`loop_thread` already `None`, and short-circuits to `Stopped` — leaving the detached OS thread alive if it hadn't already exited. Only reachable if the interpreter is gone or the loop is already dead at shutdown time; implausible in normal shipped paths and not exercised by any test. Not a blocker.

**B. Post-readiness self-failure leaves stale handles in state.** `run_loop_thread` at `async_runtime.rs:253-257` sets `state.lifecycle = Failed` but does not clear `state.loop_object` / `state.loop_thread`. A follow-up `shutdown()` then attempts `call_soon_threadsafe` on a dead loop, which fails and triggers concern A. Only occurs if `run_forever` returns unexpectedly (e.g. via SIGINT bubbling through), so implausible. Init-failure (pre-readiness) is correctly handled by `fail_start`. Not a blocker.

**C. Fixed 1s poll in `shutdown_cancels_and_joins_an_in_flight_raw_coroutine`.** `coroutine_ops.rs:124-140` polls at 1 ms × 1 000 iterations for `active_submissions == 1`. On a heavily loaded CI worker the raw caller might not reach `register_submission` within 1 s, in which case the test panics without `worker.join()` and leaves the worker blocked in `future.result()` for 60 s. TEST_LOCK's poisoning path recovers on the next test, but the detached worker still holds a submission entry until the shutdown of a subsequent test cancels it. Test-only flakiness, not a production defect. Not a blocker.

**D. DLPack deleter-counter ordering failure — unrelated to this wave.** Grep confirms `TEST_DELETER_CALLS`, `test_deleter`, and `test_capsule_destructor` are referenced only by `dlpack_ops.rs`. `async_runtime.rs` never creates a `PyCapsule`, holds only asyncio `EventLoop` and `concurrent.futures.Future` references, and `reset_for_tests` merely replaces `AsyncRuntimeState` (dropping asyncio objects into pyo3's internal release pool). There is no path by which async lifecycle drops or GCs a DLPack capsule. The ordering failure is a pre-existing cross-test contamination in the DLPack suite itself (likely in how pyo3's internal release pool drains vs. `TEST_DELETER_CALLS.store(0)` timing), not introduced by this wave.

### Verified correct

- No lock-order inversion anywhere: workers acquire GIL → `ASYNC_STATE`; shutdown holds `ASYNC_STATE` only when GIL is not held, and vice versa.
- Reserve/register/finish accounting is symmetric: `reserve_submission` increments pending, `register_submission` decrements pending and inserts to `submissions`, `finish_submission` removes from `submissions`, `release_pending_submission` covers the reserve-succeeded-but-schedule-failed path (`async_runtime.rs:126-129`) and reserve-succeeded-but-register-failed path (line 130-134).
- `shutdown()`'s two condvar waits are correct: first drains `pending_submissions` (all reserves have moved to registered or released), second drains `submissions` (all registered futures have been cancelled and finish_submission has run). Between them, `cancel_registered_submissions` snapshots futures under the lock and cancels them without holding it.
- Cache key fragment `python.requires_async_loop=` is wired through `python_interop_plan.rs:239-245` and asserted by two plan tests plus the driver test at `project_codegen.rs:180-201`.
- Bootstrap detection covers both `PythonInteropEffect::Async` declarations and aliased `sifr.python.run_coroutine_blocking` intrinsics (`python_interop_plan.rs:158-203`), verified by `raw_coroutine_call_requires_the_owned_async_loop`, `aliased_raw_coroutine_call_requires_the_owned_async_loop`, and `sync_python_declaration_does_not_require_the_owned_async_loop`.
- Init-failure path is verified: `loop_setup_failure_is_joined_and_leaves_no_live_thread` proves diagnostics normalize and a second shutdown surfaces `AsyncRuntimeFailed`.
- Wave 1 gate discipline preserved: no `@python.coroutine` or `cleanup=async_close` activation; raw API still `@blocking_io`.
- All touched files under the 900-line guardrail.

## VERDICT: SATISFIED
