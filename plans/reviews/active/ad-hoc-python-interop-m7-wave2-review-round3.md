## Final Pre-PR Review

### Verified — round-1 findings remain fixed

- **R1-1 (Lock-order inversion):** `shutdown()` at `async_runtime.rs:146-159` takes `state.loop_object` and `state.loop_thread` out under `ASYNC_STATE`, releases the guard at the block boundary, then acquires the GIL at line 169. `cancel_registered_submissions` (line 310-329) is GIL→state, matching worker paths. No inverted acquisition remains.
- **R1-2 (Concurrent lazy start):** `start()` at `async_runtime.rs:61-77` loops on lifecycle; `Starting` sits on `wait_for_change` instead of erroring.
- **R1-3 (Test coverage):** `concurrent_raw_coroutines_share_one_owned_loop_and_thread` (`coroutine_ops.rs:60-106`) races two lazy `ensure_started()` callers with `start_async_loop` unset. `shutdown_cancels_and_joins_an_in_flight_raw_coroutine` covers the shutdown-during-in-flight scenario.

### Verified — round-2 hardening claims (C, plus additional shutdown work)

- **Separate stop/join results:** lines 168-193. `stop_result` (`Python::try_attach` + `stop` + `call_soon_threadsafe`) and `join_result` (thread `join`) are computed independently.
- **Join executed when handle exists in the normal path:** line 179 unconditionally calls `join()` if `loop_thread` is `Some`, regardless of `stop_result`.
- **Lifecycle/handles normalized to Stopped after join:** lines 184-186 set `lifecycle = Stopped` and `loop_object = None` after both stop and join return.
- **Loop-thread failure priority preserved:** `state.failure.take()` at line 187 is checked at line 190 and returned before `stop_result.and(join_result)`. Stop error is prioritized over join error via `and()`.
- **In-flight test hardening:** `coroutine_ops.rs:124-138` polls up to 5000×1ms = 5s for registration; on timeout it explicitly calls `shutdown()` and `worker.join()` before panicking, so no dangling worker thread survives a timeout.
- **Forced-setup-failure test:** `async_runtime.rs:373-395` — `fail_start` clears handles, diagnostics normalize to default, second `shutdown()` surfaces `AsyncRuntimeFailed`.

### Verified — no new hazards introduced

- No lock-order inversion in the new shutdown flow: state → drop → GIL (cancel) → state → drop → GIL (stop) → no-lock (join) → state (normalize).
- Concurrent shutdown() calls are effectively serialized by the "who took the handles" race; the second caller sees `None` handles and no-ops.
- `finish_submission` acquires only ASYNC_STATE (no GIL needed), so worker-holding-GIL + shutdown-in-condvar-wait interleaves without deadlock.
- Codegen path (`python_interop_plan.rs` → `PackagePythonRuntime::set_start_async_loop` → `main.rs`) and cache key fragment (`python.requires_async_loop=`) are consistent with the tests at `python_interop_plan_tests.rs:99-158` and `project_codegen.rs:179-201`.
- File-size guardrail: `async_runtime.rs` is 396 lines, well under 900.

### Prioritized findings

**None material.**

Low-priority residual (non-blocking, unchanged from round-2 Concern A):

- **Residual (LOW, unchanged):** `async_runtime.rs:161-166` still has three `?` gates between "handles taken" and the join — `cancel_registered_submissions()?`, second `lock_state()?`, and `wait_for_change(state)?`. Early-return on any of these drops `loop_object` and `loop_thread` locally without invoking `stop()` or `join()`, and leaves lifecycle stuck at `Stopping`. Only reachable when `Python::try_attach` returns `None` (CPython finalized) or the mutex is poisoned — both implausible in shipped paths (`PythonRuntimeGuard::drop` runs before CPython finalize; the state mutex is only held briefly and non-panicking). This wave incrementally improved robustness — join is now unconditional in the common path — but the claim "always joins when a handle exists" is not literally true for these degenerate edges. Consider addressing in a future hardening pass by collecting `cancel`/wait errors into the final `stop_result.and(join_result)` composition instead of `?`-propagating early. Round 2 already ruled this non-blocking; nothing about the wave 2 hardening regresses this posture.

VERDICT: SATISFIED
