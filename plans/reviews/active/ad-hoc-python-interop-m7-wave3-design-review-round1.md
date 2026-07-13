I have enough context. Writing the review now.

# Review — M7 Wave 3 Cancellation Carrier Design

## What holds up

- **Carrier atomicity model** (mutex-guarded claim + request, hooks fired outside the lock, distinct outcomes for the four races) is correct and matches the "either abort before submission or be observed by registration" rule the plan calls out at plans/issues/active/ad-hoc-declaration-first-python-interop.md:540.
- **Preserving Tokio abort as the fallback hook** is the correct compatibility surface for Wave 3, because typed wrappers stay gated until Wave 4/5 activates them — no user program can produce a claimed task yet, so "fallback-only" supervisors do not (yet) violate the delivery rule at internal_docs/python_interop_protocol_architecture.md:93.
- **Task-local carrier + private claim helper** is the right shape: only async spawns install the task local, and typed wrappers will claim it inside the async body once Wave 4 lands. `tokio::spawn_blocking` does not propagate task locals — this is fine because raw blocking + spawn_blocking children can never enter a Python `await`.
- **`__SifrTaskResult::Cancelled(_)` fallthrough for `cancel_and_join`/`timeout`** already discards the receiver value (task_runtime.rs:232, task_runtime.rs:251), so keeping "return cancelled unconditionally after `receiver.await`" is source-compatible.
- **File-size acknowledgement** is necessary — task_runtime.rs is at 892/900 (task_runtime.rs:1-892). The extraction has to happen before any new emission code is added.

## Correctness / ownership / semantics gaps

### 1. Exact-task publication is buildable but the design underspecifies the pieces that make it safe

Publishing `asyncio.Task` back from the loop thread is workable, but the design does not commit to the following, and each is a distinct failure mode:

- **Where the `Py<PyAny>` for the exact Task lives.** If it sits inside `CancellationCarrier.state: Mutex<...>`, the carrier is dropped from arbitrary Sifr threads and `Py<PyAny>::drop` needs the GIL or the pending-release queue. Design must state: (a) the carrier is dropped through the existing detach-before-decref queue, and (b) `state.submissions.remove(id)` at async_runtime.rs:305 must drop the `Py<PyAny>` *after* releasing `ASYNC_STATE`, not while holding it.
- **Setup-callback failure delivery.** If `ensure_future(coro, loop=loop)` raises on the loop thread (bad coroutine, GC storm, `RuntimeError: coroutine already awaited`), Sifr must observe an error, not a hang. Design must promise: setup callback wraps its body in try/except and completes the completion oneshot with an error, and registration bookkeeping (`pending_submissions--`) unwinds. Nothing in the design mentions this path.
- **`call_soon_threadsafe` failure between reserve and register.** `reserve_submission` bumps `pending_submissions` (async_runtime.rs:278). If `call_soon_threadsafe` throws (loop closing between reserve and submit) we must `release_pending_submission()`. Current code does this only around `run_coroutine_threadsafe`. The new driver must preserve or re-derive that unwind, and it must do it *without* leaving the carrier in a claimed-with-no-task state that will latch cancel forever.
- **Shutdown-drain of claimed submissions.** `cancel_registered_submissions` (async_runtime.rs:310) calls `.cancel()` on a stored `concurrent.futures.Future`. If we replace the stored object with an `asyncio.Task`, shutdown must switch to `loop.call_soon_threadsafe(task.cancel)` and, crucially, must wait on the *completion oneshots* rather than the current `while !state.submissions.is_empty()` polling on the shutdown Condvar. Otherwise shutdown can return while a claimed Sifr `__SifrTask` is still blocked on `receiver.await` and its child future is holding the completion sender. This isn't optional — the architecture doc (line 106-108) says shutdown must terminally drain, and Wave 3 owns this substrate.

### 2. Raw blocking submission's cancellation story is left in a broken state

Design point 5 says raw blocking "uses the same exact-task driver without a carrier." Consequences:

- The Sifr sync-entry path (`run_coroutine_blocking` at async_runtime.rs:112) has no async await point. If a caller wraps a raw coroutine in a `__SifrTask` and later calls `task.cancel()`, the fallback hook aborts a Tokio task that is executing a sync Rust function that blocks the thread on the loop. Abort marks the task but cannot interrupt sync code, and the current `.call_method0("result")` will keep waiting. This is pre-existing but the design *removes the concurrent.futures.Future* that today lets shutdown cancel via `.cancel()`.
- With an `asyncio.Task`-based driver and no carrier, there is no thread-safe way for external code to say "cancel this raw call." Design must either (a) give raw submissions a lightweight carrier whose fallback hook does `call_soon_threadsafe(task.cancel)` plus completion-latch wait, or (b) explicitly acknowledge that a raw blocking call is uninterruptible except by shutdown-registry sweep, and route shutdown through the same task-cancel path.
- The synchronous waiter also needs to release the GIL before blocking. Today `concurrent.futures.Future.result()` releases the GIL internally. If we replace it with a Rust `oneshot::Receiver::blocking_recv()`, that call panics inside a Tokio runtime and does not release the GIL. Design must specify: `Python::detach` (release GIL) around a `blocking_recv` on a std channel, and it must reason about being called from inside a Tokio worker (not allowed) versus off the runtime. This is a mandatory piece, not a detail.

### 3. Supervisor "fallback hook only" is technically legal *this wave* but the invariant that keeps it legal is not stated as a compile-time enforcement

The design leans on "typed wrappers are gated, therefore no `__SifrTask` will be claimed, therefore fallback-only supervisors don't violate anything." That is currently true but there is no assertion that binds the two. Recommend either:

- The private claim helper `debug_assert!`s that it is only reachable from lowered typed wrappers (e.g., a codegen-generated cfg or a runtime `#[cfg(feature = "sifr-python-typed")]` gate that matches the frontend gate on `@python.coroutine`), or
- Wave 3 emits a hard runtime guard: if the current task-local carrier is claimed at the moment a supervisor extracts a fallback hook, panic. That converts "silent contract violation" into a build-time or first-run failure if Wave 4 gets landed out of order or partially.

Without either, a wave-ordering slip re-introduces the delivery-rule bug silently.

### 4. `cancel_and_join` and `timeout` value semantics under Python suppression are not resolved

The architecture doc says at internal_docs/python_interop_protocol_architecture.md:104 that if Python suppresses cancellation and returns another value, "that terminal result wins and is mapped normally." The current preamble discards `receiver.await` (task_runtime.rs:232, task_runtime.rs:251). The design keeps that pattern.

For `cancel_and_join`, "always cancelled" is defensible Sifr semantics. For `timeout`, always returning `Timeout` after a Python suppression contradicts the doc: timeout is a Sifr-driven cancellation and Python may have returned a value; the doc says the terminal Python result wins. If Wave 3 sets the shape without deciding this, Wave 4 will inherit a preamble that intentionally throws the Python-winning value away, and code readers will have to reason about "which cancel semantics apply here." Decide now, even if the implementation lands with Wave 4.

### 5. `__SifrBlockingTask` is not addressed

The design specifically upgrades `__SifrTask` and leaves `__SifrBlockingTask` untouched. This is correct (spawn_blocking cannot enter async awaits, so no Python-await claim), but it should be stated. Otherwise future waves may add a carrier to `__SifrBlockingTask` and the semantics will diverge.

### 6. JoinSet's `abort_handle: Option<AbortHandle>` swap needs a concrete shape

join_set_runtime.rs:249, join_set_runtime.rs:319 store `Option<AbortHandle>` and in `__sifr_cancel_all` fall back to `entry.handle.abort()` when the option is None. If the field becomes an `Option<CancellationHook>`, that fallback path (`entry.handle.abort()`) no longer has a symmetric analogue — a hook is opaque and cannot "escalate" to the wrapper JoinHandle. Design must say whether:

- JoinSet keeps `AbortHandle` and *also* gets a fallback hook clone (bloat but no behavior change), or
- JoinSet stores the wrapper task's `AbortHandle` (the outer `tokio::spawn` in `__sifr_add_task`) and the fallback hook is discarded because the wrapper future dropping already collapses the child.

The current wording ("destructure the carrier and use a cloned fallback hook only") elides which handle backs `__sifr_cancel_all`.

## PyO3 / GIL deadlock analysis

- `finish_submission` (async_runtime.rs:303) drops `Py<PyAny>` inside `submissions.remove()` while holding `ASYNC_STATE`. Today the object is a concurrent.futures.Future whose drop is cheap; if we swap in an `asyncio.Task` we still need pending-release safe drop. Move the take/drop outside the lock.
- If the setup callback (loop thread) needs to call back into `CancellationCarrier::publish_exact_task(...)`, ordering is: (loop thread holds GIL) → carrier lock → possibly `call_soon_threadsafe(task.cancel)` if latched. `call_soon_threadsafe` acquires an internal asyncio lock but does not need the GIL held (it does need the caller to have the GIL initially — we have it). To avoid holding the carrier mutex across a Python call, publish should copy out the latched-cancel bit under the lock, drop the lock, then optionally call `task.cancel()` directly (we are on the loop thread). Design says "hooks run outside the lock"; extend that to the publish path.
- Sifr-side sync wait: if `run_coroutine_blocking` blocks on a Rust channel while holding the GIL, and the loop thread cannot make progress without the GIL, we deadlock. Must `Python::detach` (or the equivalent PyO3 0.22+ `py.detach`) around the wait.
- Async-side wait: `__SifrTask::cancel_and_join.await` and `__sifr_timeout` are async fns without any Python attach, so they cannot deadlock on the GIL. Fine.

## Minimum test set (design misses several)

Runtime unit (Rust-only):
- Request-before-claim; claim-before-request; contended claim (single winner); repeat request idempotent; fallback hook fires exactly once; hook Send/Sync bounds enforced; hook stored `Arc<dyn Fn()>` cloning is thread-safe.
- Carrier drop with a Python task ref does not require the GIL and does not panic under `Python::detach`.

Runtime unit (Python-facing, `test_guard`-serialized):
- Cancel before setup callback runs → setup observes latch, does not `ensure_future`, sends `cancelled` on completion oneshot.
- Cancel during in-flight coroutine with a `finally:` block that sets a marker; assert marker set before completion oneshot resolves.
- Coroutine that suppresses `CancelledError` and returns a value → completion oneshot resolves with that value; `cancel_and_join` still reports cancelled; `timeout` behavior matches whichever semantics the design chooses (this is the test that forces the decision).
- Setup callback raises (`ensure_future` type error) → Sifr wrapper returns a `PythonError`, `pending_submissions` returns to zero, no orphan in `submissions`.
- Shutdown while a claimed submission is in-flight → completion oneshot resolves, shutdown returns Ok, no live loop thread or unjoined tokio task.
- Two concurrent claimed submissions cancel independently.
- Raw blocking submission under `Python::detach` waits without holding the GIL, and one running raw call does not block a second raw call on the loop.

Codegen snapshots:
- `__SifrTask` no longer contains `abort_handle`; contains `carrier`.
- `cancel`, `cancel_and_join`, `__sifr_timeout` call `carrier.request_cancel()`.
- Spawn helpers create the carrier, bind fallback hook to the child's `AbortHandle`, and scope the child future under the task-local carrier.
- Supervisors (`gather`, `race`, `select`, `__sifr_add_task`, JoinSet `__sifr_cancel_all`) still call the fallback abort path — the snapshot is the wave boundary and Wave 4's PR will diff it.
- File-size guardrail: `scripts/check_hir_maintainability_guardrails.py` (or the equivalent line-count check invoked by `run_all_tests.sh`) passes.

## Concrete recommended changes to the design

1. **Add a "loop-thread setup contract" section**: setup runs under GIL, catches all exceptions, publishes exact Task via `carrier.publish_exact_task(task)` which atomically checks and consumes any latched cancel, attaches `task.add_done_callback(...)` that fires the Rust completion oneshot, and on failure delivers the exception through the completion oneshot. Nothing else on the loop thread touches the completion oneshot.
2. **Specify sync waiter mechanics**: raw `run_coroutine_blocking` uses `Python::detach` around a `std::sync::mpsc::Receiver::recv()` (or `oneshot::blocking_recv` off-runtime). Add a `debug_assert` that the current thread is not a Tokio worker (`tokio::runtime::Handle::try_current().is_err()`), or route sync waits through `spawn_blocking` on the caller side.
3. **Extend shutdown**: replace `cancel_registered_submissions` and the drain condvar with per-submission completion oneshots collected during `reserve_submission`; shutdown awaits them (with the existing mutex-based state machine only tracking counts). Store `asyncio.Task` instead of the concurrent.futures.Future.
4. **Give raw submissions a lightweight carrier** whose fallback hook is `call_soon_threadsafe(task.cancel)`, so raw calls participate in the same shutdown/cancel registry and the "no two paths" architecture rule (protocol doc line 116) still holds.
5. **Decide `timeout` value semantics under Python suppression** before Wave 3 lands, even if only in a comment on the emitted `__sifr_timeout`.
6. **Add a hard gate on the claim helper**: `debug_assert!(cfg-guard-that-matches-frontend)` so Wave 3's fallback-only supervisors cannot silently be paired with a claimed task.
7. **Concrete task_runtime.rs split**: (a) `preamble/task_types.rs` — `__SifrTask`, `__SifrBlockingTask`, `__SifrScopeChild`, result/select/timeout enums; (b) `preamble/task_impl.rs` — `__SifrTask`/`__SifrBlockingTask` inherent impls (join/cancel/cancel_and_join/timeout); (c) `preamble/task_scope_spawn.rs` — `__sifr_spawn_infallible`/`_result`, blocking spawns; (d) `preamble/task_scope_join.rs` — `__sifr_join_all`; (e) `preamble/task_supervisors.rs` — gather/race/select. New carrier emission lives in `preamble/task_cancellation.rs`. Each stays well under 900.
8. **State that `__SifrBlockingTask` intentionally has no carrier** and never will.
9. **State that `finish_submission` and any similar mutator must drop `Py<PyAny>` outside the state mutex** (a small explicit rule that survives future refactors).

VERDICT: NOT SATISFIED
