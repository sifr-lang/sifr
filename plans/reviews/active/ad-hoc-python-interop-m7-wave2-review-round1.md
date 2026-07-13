# M7 Wave 2 Review — Prioritized Findings

## Prioritized Actionable Findings

### 1. HIGH — Lock-order inversion between the state mutex and the GIL in `shutdown()` can deadlock a live worker (correctness)

`crates/sifr_runtime/src/python/async_runtime.rs:160-163`
```rust
let loop_object =
    Python::try_attach(|py| state.loop_object.as_ref().map(|value| value.clone_ref(py)))
        .flatten();
(loop_object, state.loop_thread.take())
```

Shutdown holds `ASYNC_STATE` (from line 149) and then blocks acquiring the GIL via `Python::try_attach` (backed by `PyGILState_Ensure`). But every worker path acquires the GIL first (via `super::attach`), then acquires `ASYNC_STATE`:

- `reserve_submission` (line 265) locks state, `clone_ref(py)` needs the GIL.
- `register_submission` (line 285) locks state, `future.clone_ref(py)` needs the GIL.
- `finish_submission` (line 306) locks state, invoked from within the GIL-holding closure in `coroutine_ops.rs:6-9`.

Concrete deadlock trace (all fields verified in the diff):

1. Worker W is in `run_coroutine_blocking` (line 114). It reserves, submits, and registers. After `register_submission` returns, `pending_submissions` is 0 and W still holds the GIL, at line 138 preparing `call_method0("result")`.
2. Concurrent `shutdown()` acquires `ASYNC_STATE` (line 149), transitions to `Stopping`, wakes from `wait_for_change` (line 157‑159) because pending is 0, and reaches line 160 holding state, wanting the GIL.
3. If the coroutine is already done (e.g. `asyncio.sleep(0.0)` or the identity coroutine used in the concurrent test), `Future.result()` returns synchronously without releasing the GIL. W stays GIL-holding through `map(Bound::unbind)` and `map_err`, then calls `finish_submission` at line 143 — which needs state.
4. Shutdown holds state and blocks on GIL; W holds GIL and blocks on state → classic deadlock.

This is not triggered by the shipped tests only because every test joins its workers before calling `shutdown()`. `PythonRuntimeGuard::drop` (crates/sifr_runtime/src/python.rs:117) calls `async_runtime::shutdown()` on program exit while other threads may still be inside the raw path (e.g. a `spawn_blocking` task that isn't drained by the tokio runtime). This is a real production hazard and blocks SATISFIED.

Suggested fix (minimal): take `state.loop_object` out of the guard before touching the GIL; do not hold `ASYNC_STATE` across `Python::try_attach`. For example:
```rust
let (maybe_loop, loop_thread) = {
    let mut state = lock_state()?;
    // (transition + wait_for_change loop unchanged)
    (state.loop_object.take(), state.loop_thread.take())
};
```
`state.loop_object` is already cleared later at line 191, so taking it here is equivalent semantically and eliminates the inverted acquisition order.

### 2. MEDIUM — Concurrent lazy `ensure_started` racing with an in-flight `Starting` spuriously fails (correctness/UX)

`crates/sifr_runtime/src/python/async_runtime.rs:104-112`
```rust
AsyncLifecycle::Starting => Err(PythonRuntimeError::AsyncRuntimeNotRunning),
```

If two threads call `run_coroutine_blocking` before the loop is running (e.g. `start_async_loop=false` and both racing on first use), thread A transitions to `Starting` inside `start()` and blocks on `ready_receiver.recv()`; thread B sees `Starting` and returns `AsyncRuntimeNotRunning` immediately even though the loop will be up microseconds later. Failure scenario: a program that opts out of `requires_async_loop` detection (say a future dynamic-only path) and issues two concurrent raw calls would see one worker error out.

Mitigation today: generated bootstrap sets `start_async_loop=true` whenever the plan detects the intrinsic, so the loop is already `Running` before any worker attaches. But the lazy-start invariant is asserted by the tests' single-caller path (`run_coroutine_blocking_uses_the_application_owned_event_loop`), not by a concurrent lazy start. Prefer `wait_for_change` until lifecycle leaves `Starting` (analogous to the shutdown drain), rather than erroring.

### 3. LOW — Test cannot exercise concurrent lazy start; identity proof runs with pre-started loop only (test coverage)

`crates/sifr_runtime/src/python/coroutine_ops.rs:60-97`
`concurrent_raw_coroutines_share_one_owned_loop_and_thread` sets `config.start_async_loop = true`, so the loop is already `Running` when the workers spin up. That is fine for proving "one loop, one thread" identity, but it does not exercise the more interesting invariant of Finding 2 (concurrent lazy start) or the shutdown/worker race behind Finding 1. Non-blocking, but consider one additional case that omits `start_async_loop` and lets `ensure_started` bring the loop up under contention.

## Verified Correct

- Bootstrap detection is conditional on either an async declaration (`PythonInteropEffect::Async`) or the raw intrinsic (`sifr.python.run_coroutine_blocking`), including aliased local names — python_interop_plan.rs:83, 96, 130, 158‑203 and the three plan tests.
- Cache identity flips on the new flag: `python.requires_async_loop=` fragment (python_interop_plan.rs:239-245); flowed through `PackagePythonRuntime::set_start_async_loop` → `render_python_runtime_prelude` (`start_async_loop: {start_async_loop}`) → the generated `main.rs` — asserted at project_codegen.rs:180-201.
- Wave 1 gate preserved: `@python.coroutine` still hard-errors with `PYRES_UNIMPLEMENTED_DECLARATION` (python_interop.rs:410, 435, 639); async wrappers are not generated; no `async_close` runtime activation.
- Raw API keeps `@blocking_io` classification (stdlib/sifr/python.sifr:778) — no offload semantics changed.
- Single named OS thread (`sifr-python-asyncio`), started only after CPython + bridge init (python.rs:250-256), readiness published via bounded `sync_channel(1)` before admission (async_runtime.rs:76-101), state machine covers Disabled/Starting/Running/Stopping/Stopped/Failed, submissions keyed by monotonic `next_submission_id: u64`.
- `reserve_submission` rejects when not `Running` (async_runtime.rs:267‑273); `saturating_add`/`saturating_sub` guard the counters.
- Init-failure path can't leak the thread: `fail_start` (async_runtime.rs:334‑343) clears `loop_object`/`loop_thread` and the `loop_setup_failure_is_joined_and_leaves_no_live_thread` test verifies both `diagnostics == default` and the second shutdown surfacing `AsyncRuntimeFailed`.
- Ready-sender lifecycle is correct: `Option::take` consumes it exactly once, and the post-readiness failure path routes into the `ASYNC_STATE` fallback (async_runtime.rs:228-262).
- No `Any`/`unwrap`/`expect` in user paths; no `asyncio.run` remains in the raw path; concurrent workers observe identical `id(loop):thread_ident`.
- File sizes all under the 900-line guardrail; async runtime cleanly isolated in its own module.

## Verdict

The wave scope, state machine, cache identity, bootstrap detection (including aliased raw calls), gate discipline, and init-failure guarantee all land as specified in the plan review round 2. However, Finding 1 is a real lock-order inversion that can deadlock the runtime guard's `drop` against an in-flight fast raw coroutine on another thread — the exact "runtime shutdown leaves no live asyncio task or loop thread" acceptance criterion cannot be guaranteed until this is fixed. This is a small mechanical fix (take `loop_object` out of the guard before touching the GIL) but it is a material correctness gap.

VERDICT: NOT SATISFIED
