

Now I have the complete picture. Let me write the review.

---

**Verdict: Blockers found**

The model is architecturally sound and avoids most of Python's cancellation pitfalls by design. However, three specific gaps create real user-facing hazards that must be closed before the phase is considered ready.

---

### B1: `CancellationError` is silently caught by `except Error`

**File:** `internal_docs/async_concurrency_model.md`, "Lock task result semantics" (milestone_async_0, lines 364–368) and "Error Semantics" in `architecture.md` (Contract #3, lines 365–484)

**Problem:** `CancellationError` is typed as an `Error` subclass. `except Error as e` is Sifr's documented catch-all. This means a user who writes a broad error handler to cover I/O failures, timeouts, and validation errors will **silently absorb cancellation**:

```sifr
async def process():
    try:
        result = await fetch_data()  # can be cancelled
    except Error as e:
        print(f"failed: {e}")
        return Err(e)
    return Ok(result)
```

Here, cancellation produces `CancellationError`, which matches `except Error as e` — the function returns successfully instead of propagating cancellation upward. In Python, the equivalent `except Exception` absorbs `CancelledError` the same way, and this is a well-known footgun.

The model states "`CancellationError` and `TimeoutError` live in `sifr.task`" and "`Task[T, E]` uses `CancellationError` when a task is cancelled before completing" but never addresses whether these are **re-raised by scope exit** or **catchable by user handlers**.

**Fix:** Add to milestone_async_0 "Lock task result semantics" a clear statement:

> Cancellation is **scope-exit semantics**, not a catchable user error. When a task is cancelled and the scope exits abnormally, `CancellationError` propagates through the scope as a structured cancellation signal and is **not** routed to `except Error as e` handlers. The error type for cancellation is `CancellationError`, but its propagation path differs from ordinary error paths: it bypasses user-level `try`/`except` blocks inside child tasks and surfaces as a structured cancellation at the scope boundary.

This requires either:
- (a) `CancellationError` has a special flag that makes `except Error` skip it (special-cased in the auto-unwrap logic), or
- (b) scopes do not route cancellation through `try`/`except` at all — cancellation unwinds the task frame directly and produces the scope-level result directly.

Option (b) is cleaner and matches Rust's `tokio::select` + cancellation propagation model. Option (a) is more complex but preserves the current `try`/`except` uniformity.

Additionally, the architecture error type table (`architecture.md`, lines 499–522) should add `CancellationError` and `TimeoutError` to the error type reference with clear propagation semantics.

---

### B2: `task.timeout` inner-task race is undefined

**File:** `internal_docs/async_concurrency_model.md`, "Lock task result semantics" (lines 376–378) and milestone_async_2 work items (lines 488–489)

**Problem:** The spec says "`task.timeout` uses `TimeoutError` when an operation exceeds its deadline" and "timeout cancels the enclosed operation." It does not specify what happens if the inner operation **completes successfully before the timeout fires**.

In Python's `asyncio.wait_for`:
- If the inner task completes before timeout, `wait_for` returns the result and **the inner task completes normally** (it is NOT cancelled).
- If the timeout fires, `wait_for` cancels the inner task and raises `TimeoutError`.

But the spec does not state this. A naive implementation might cancel the inner task on scope exit regardless of whether it finished in time, causing the inner result to be lost. Or it might cancel on timeout but not handle the race where the inner completes at the same tick as timeout.

**Fix:** Add to milestone_async_2 work items or to the cancellation policy section:

> **`task.timeout(task, duration)` behavior:**
> - If the inner task completes before `duration`, `timeout` returns `Ok(result)` and the inner task is left to complete normally.
> - If `duration` expires first, the inner task is cancelled and `timeout` returns `Err(TimeoutError)`.
> - If the inner task completes in the same scheduler tick as the timeout, completion wins (deterministic tie-breaking: inner completion is checked before timer expiry in the same poll).
> - Cancelling the outer scope while `timeout` is running cancels the inner task unconditionally.

This maps directly to Rust's `tokio::time::timeout` behavior, which is the correct semantics for Sifr to adopt.

---

### B3: Cancelled-but-unawaited task behavior is unaddressed

**File:** `internal_docs/async_concurrency_model.md`, milestone_async_3 acceptance criteria (lines 553–561) and milestone_async_2 negative validation (lines 516–521)

**Problem:** The spec includes `task_handle_unused_must_join_or_cancel` as a negative validation test but never explains **what happens to a spawned task whose handle is discarded without join or cancel**. In Python, this is the "fire-and-forget leak" problem: tasks keep running until they complete or the process exits. In structured concurrency, the expectation is that either:

- (a) the compiler rejects unused task handles (emit a diagnostic), or
- (b) the scope automatically joins or cancels any orphaned handles at scope exit.

The spec says "There is no free-floating detached spawn in v1" and "add diagnostics for leaked task handles" but does not define the runtime semantics when a user spawns a task and discards the handle without awaiting it.

**Fix:** Add to milestone_async_2 or milestone_async_3:

> **Orphaned task handles:** A `scope.spawn` that produces a task handle **must** be either:
> - awaited via `await handle`,
> - explicitly cancelled via `handle.cancel()`, or
> - joined via `handle.join()`.
>
> If the task handle goes out of scope without one of these, the compiler emits diagnostic `SIFR-ASYNC-XXXX: unused task handle`. There is no implicit fire-and-forget. This is a compile-time guarantee, not a runtime fallback.
>
> At scope `__aexit__`, if any task handles remain unconsumed, the scope cancels those tasks and awaits their cleanup before completing. No task handle may escape a scope silently.

This makes orphaning impossible at compile time for the common case and guarantees a safe runtime path for edge cases (e.g., handle stored in a collection, branching on `None`).

---

### Non-blocking refinements

**R1: `spawn_blocking` cancellation stops work is unverified** (`internal_docs/async_concurrency_model.md`, milestone_async_6, lines 714–716)

The spec says "Define return/error/cancellation behavior for blocking tasks" but does not state whether `spawn_blocking` cancellation **actually aborts the thread** or merely abandons the handle. Tokio's `spawn_blocking` does not interrupt running CPU-bound work — cancellation only stops the future from polling the handle. This is a documented Tokio limitation. Sifr should match this explicitly:

> **`task.spawn_blocking` cancellation semantics:** Cancellation of a blocking task does not forcibly abort the thread. The task is requested to stop, and the handle is dropped. The thread continues to completion but its result is discarded. Users who need interruptible CPU work must use process-level isolation (deferred to Phase 40 typed IPC).

**R2: `gather` secondary errors are vague** (`async_concurrency_model.md`, lines 544–547)

"Later errors are secondary structured errors" uses undefined "structured errors." What type do secondary errors have? How are they propagated? The spec should define:

> `gather` returns `Result[list[T], Error]` where `Error` is the primary (first to surface) error. Secondary errors are dropped unless a future `collect_all` variant is added.

**R3: `uncancel` / cancellation counter is deferred but should be tracked** (`async_concurrency_model.md`, out of scope, lines 191–192)

`contextvars` and cancellation suppression are deferred, which is correct. But the spec should note a design direction:

> Future `sifr.task.cancellation_scope` may provide `suppress()` / `uncancel()` for scoped cancellation suppression, following the same lexical-structured pattern as `task.local[T]`.

**R4: Async cleanup failure during cancellation needs a typed path** (`async_concurrency_model.md`, lines 772–778)

The spec says "errors from async exit during cancellation are secondary structured errors" and "panic-like failures in async exit do not become process-terminating double-panic paths." It does not define **what the secondary error type is** or **how it surfaces** (does it get logged, stored in a field, discarded?). Clarify:

> If async `__aexit__` raises during cancellation unwinding, the original `CancellationError` is the primary result, and the `__aexit__` error is wrapped in `SecondaryError` (a new error type) and stored in the scope's error log. The scope still completes. The primary result is never masked.

**R5: No design for graceful shutdown tokens** (`async_concurrency_model.md`, out of scope, lines 188–189)

Subprocess and signal APIs are deferred, which is fine. But the spec should define the **minimal shutdown token surface** for v1:

> Graceful shutdown in v1 uses `scope.spawn` with cancellation. There is no explicit shutdown token in v1. Future `sifr.signal.shutdown_channel()` provides a cancellation-triggered channel for graceful shutdown orchestration.

---

### Recommendation: iterate again

The model is 80% correct and the architectural choices (structured concurrency, typed errors, `Result` over exceptions, no fire-and-forget, explicit synchronization) are all sound. The three blockers are specific, fixable gaps in the cancellation propagation contract and timeout race semantics. They must be addressed in the model before implementation begins, because the compiler and runtime codegen must implement the exact propagation semantics defined here.

After B1, B2, and B3 are resolved in the document, the model should be in good shape for implementation.
