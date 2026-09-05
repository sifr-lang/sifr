# Review: Sifr Async and Concurrency Model Proposal

**Reviewer:** agent
**Date:** 2026-05-09
**Document:** `internal_docs/async_concurrency_model.md`
**Branch:** `codex/figure-out-concurrency`

---

## Summary

The proposal is well-scoped, architecturally coherent, and correctly rejects Python asyncio's sharp edges. The milestone dependency graph is sound. However, three **blockers** and numerous non-blocking issues were identified. The blockers are all in the **open decisions** that are deferred to `milestone_async_0` — meaning the contract is not locked before implementation begins, which violates the proposal's own intent.

---

## Blocker 1: `await Task[T, E]` semantics are ambiguous

**Location:** Open Decision #3 (line 743), repeated in Acceptance Criteria for milestone_async_1 (lines 210-211) and milestone_async_1 design note (line 313).

**Problem:** The proposal states:
> `await Task[T, E]` produces `Result[T, E]` or participates in existing `try`/`except` auto-unwrap rules.

This creates a two-path model that is ambiguous. If `await Task[T, E]` can return either `T` or `Result[T, E]` depending on context, then:
- Type narrowing downstream becomes context-dependent and harder to reason about.
- The borrow checker and HIR must track which context produced which result type.
- Sifr's "no surprise" principle is violated.

**Proposed fix — add to milestone_async_0:**

> **`Task[T, E]` always awaits to `Result[T, E]`.** The caller must `try`/`except` the result to extract `T` or handle `E`. Inside a `try` block, `Result[T, E]` auto-unwrap follows existing rules (Sifr's `try`/`except` pattern-matches on `Result`). Outside a `try` block, `await task` returns `Result[T, E]` as the observable type. This is the same model as Go's `<-ch` (which returns `Result` that must be explicitly checked) and matches Sifr's error-handling contract: no silent unwrap, no ambient exception propagation.

**Rationale:** This makes the type system predictable. `Task[T, E]` is a future that resolves to `Result[T, E]`. Users always see `Result[T, E]`. Inside `try`/`except`, auto-unwrap works because `Result[T, E]` is a `Result` type. Outside, users write `try: x = await task` or `match await task: Ok(v) => ... Err(e) => ...`. No context-dependent magic.

**Recommended change to lines 210-211:**
Replace:
> `await Task[T, E]` produces `Result[T, E]` or participates in existing `try`/`except` auto-unwrap rules.

With:
> `await Task[T, E]` always produces `Result[T, E]`. Inside a `try` block, `Result[T, E]` follows existing auto-unwrap semantics (the compiler inserts `?` in HIR). Outside `try`, the caller observes `Result[T, E]` as the expression type.

---

## Blocker 2: Lock guard across `await` — decision deferred but impacts codegen design

**Location:** Open Decision #6 (line 746): *Should lock guards be allowed across `await` if the lock type is async-aware, or should the first version reject guard-across-await for simplicity?*

**Problem:** This is not an open question for language design — it is a question with binary implementation consequences. The `sync.Lock` codegen strategy (does the generated Rust hold a `tokio::sync::Mutex` across `.await`, or does Sifr use a scoped guard that must not cross await?) fundamentally affects:
- The `Lock[T]` runtime type: `tokio::sync::Mutex` (async-aware) vs `std::sync::Mutex` (blocking, spawned to thread pool).
- The HIR guard type: whether `with lock: ...` produces a guard that is `!Unpin` and therefore cannot cross await.
- The diagnostic model: whether guard-across-await produces a compile error or is silently accepted.

If this is deferred past `milestone_async_4` (Send/Sync boundary checking), the HIR design for lock guards cannot be finalized.

**Proposed fix — add to milestone_async_0:**

> **Lock guards MUST NOT outlive their enclosing await point in v1.** `sync.Lock[T]` generates a `tokio::sync::Mutex` (async-aware). The guard type is bound by the compiler to the synchronous scope of a single statement or block. Crossing an `await` with a live guard is a compile-time diagnostic. This is the safer choice: it matches Rust's `std::sync::MutexGuard` lifetime semantics, is easier to reason about, and avoids subtle deadlock patterns. Users who need to hold a lock across await must explicitly re-acquire after await.

**Alternative:** If async-aware locks are desired, a separate `sync.AsyncLock[T]` type (generating `tokio::sync::Mutex` with explicit async acquire semantics) is needed. This should be a v2 decision, not v1.

**Recommended change to lines 746-747:**
Replace:
> 6. Should lock guards be allowed across `await` if the lock type is async-aware, or should the first version reject guard-across-await for simplicity?

With:
> 6. **LOCKED: Lock guards must not outlive their enclosing await point in v1.** `sync.Lock[T]` generates `tokio::sync::Mutex`. The guard is scope-bound. Crossing an `await` with a live guard is a compile error. `sync.AsyncLock[T]` is deferred to v2 if there is a demonstrated need for async-aware lock acquisition.

---

## Blocker 3: Channel close semantics are underspecified for producer/consumer cancellation

**Location:** `sync.Channel[T]` work items in milestone_async_5 (lines 488-494), and Open Decision #5 (line 745).

**Problem:** The proposal defines:
> Implement channel send/receive close semantics.

But it does not specify the behavioral contract for **cancellation during channel operations**. In Python asyncio, `Queue.get()` raises `CancelledError` when the queue is closed or the task is cancelled. In Swift's structured concurrency, `Task` cancellation propagates into channel operations.

If `Channel[T]` does not define how cancellation interacts with `send`/`receive`, users writing real programs will encounter inconsistent behavior between channels and task scopes.

**Proposed fix — add to milestone_async_5 work items:**

> **Define cancellation interaction for channels:**
> - `channel.send(value)` on a closed channel returns `Result[None, ClosedError]`.
> - `channel.receive()` when the channel is closed returns `Result[Option[T], ClosedError]` (empty with explicit `None`).
> - If the task owning a channel operation is cancelled (via scope cancellation or explicit cancel), the send/receive operation is aborted and the cancellation propagates.
> - Closing a channel cancels all pending receivers and senders — the behavior is deterministic and typed.
> - The `sync.Channel[T]` constructor creates either a bounded (with backpressure) or unbounded channel; bounded channels block senders when full.

Also update Open Decision #5:
> 5. **LOCKED: `sync.Channel[T]` is a multi-producer, multi-consumer channel.** Bounded and unbounded variants are exposed through separate constructors: `sync.channel[T]()` (unbounded, `mpmc`) and `sync.bounded_channel[T](capacity)` (bounded with backpressure). Single-consumer use cases use scoped receivers.

**Recommended change — add to milestone_async_5 work items after line 494:**
> **Cancellation interaction for channel operations:**
> - `channel.close()` cancels all pending `send` and `receive` operations.
> - `channel.receive()` returns `Result[Option[T], ClosedError]`; `None` indicates graceful end-of-stream.
> - A task cancelled while blocked on `send` or `receive` propagates cancellation.
> - Bounded channels block `send` when full and return `Result[None, FullError]` on graceful timeout (if timeout is specified).

---

## Non-Blocking Issues

### N-1: IO-bound annotation is proposed but not modeled

**Location:** Review angle #6 (request) — not in the proposal.

The proposal does not include an IO-bound annotation or decorator. The request to assess it is valid: Python asyncio has no way to mark a function as inherently blocking, so diagnostics must rely on heuristics (detecting known blocking stdlib calls, as mentioned in line 541).

**Recommendation:** Add to milestone_async_0 or milestone_async_6:
> **Deferred to v2.** The idea of a `@blocking` or `@io_bound` decorator that produces diagnostics or auto-offloads is worth exploring, but it requires a stable blocking-call heuristics database and/or an opt-in annotation system that Sifr does not yet have. In v1, diagnostics for known-blocking stdlib calls in async contexts (line 541) is sufficient. A formal annotation system can be added in v2 after the stdlib surface is stable.

### N-2: `Task[T]` type aliasing creates confusion with `concurrent.Future`

**Location:** milestone_async_8 (line 628): *Add `sifr.concurrent.Future` as a compatibility type where it maps cleanly to task handles.*

**Recommendation:** Clarify in milestone_async_0 that `sifr.concurrent.Future` is a type alias for `sifr.task.Task` in the compatibility layer, not a distinct type. The mapping must be explicit to avoid confusion about whether `Future` is a separate async primitive.

**Add to milestone_async_8 work items:**
> `sifr.concurrent.Future` is a **type alias** for `sifr.task.Task`, not a separate type. Compatibility is achieved through the alias, not through reimplementation. CPython's `concurrent.futures.Future` is a different runtime model (thread-pool based); the alias is a naming concession only.

### N-3: Process pools deferred but the typed IPC dependency is not tracked

**Location:** line 99, line 552, line 629, and milestone_async_8 negative validation (line 655).

**Recommendation:** The proposal correctly defers `ProcessPoolExecutor`, but it does not track the **dependency**: typed IPC/serialization contract (from Phase 40 or a pre-Phase-32 contract). This should be noted as a hard dependency:

> **Process pools: hard dependency on Phase 40 typed IPC contract.** Process pools cannot be implemented without stable typed data serialization for cross-process value transfer. This dependency is on the Phase 40 deliverables (`sifr.typed_data` or equivalent). Process pools are not merely "deferred" — they are blocked on a future phase deliverable.

### N-4: Async generators deferred but the protocol boundary is unclear

**Location:** Out of scope (lines 172-173), repeated in milestone_async_7 (line 584).

**Recommendation:** Clarify in milestone_async_7 acceptance criteria that async iteration for built-in types (channels, streams) is in scope, but async generators (user-defined `async def` that `yield`) are a separate feature with a separate protocol:

> Async generators (`async def` with `yield`) are deferred to v2. The async iteration protocol (`async for x in channel`) is fully specified in milestone_async_7. User-defined async generators require a separate `AsyncGenerator` protocol and HIR support that is out of scope for this phase.

### N-5: `contextvars` — needs explicit rationale beyond "deferred"

**Location:** Out of scope (line 169).

**Recommendation:** Add rationale:
> `contextvars` (async task-local storage) is deferred. Rationale: Sifr's task model is structured by default, so task-local state can be modeled via lexical scope (normal variable capture). Cross-task implicit context propagation (as in Python's `copy_context()`) is a footgun that structured concurrency aims to eliminate. If a future use case requires task-local storage with structured inheritance, a `sifr.task.local[T]` primitive can be added then.

### N-6: Select/race cancellation behavior needs explicit lock (not open decision)

**Location:** Open Decision #4 (line 744).

**Recommendation:** Lock this decision now:
> **LOCKED: `select` and `race` cancel losing/loser tasks by default.** When `task.race(t1, t2)` resolves with `t1`'s result, `t2` is cancelled. This is consistent with Swift's `withTaskGroup` race behavior and Go's `select` semantics. Loser cancellation is deterministic and the cancellation propagates through the scope. Users who need non-cancelling race (returning both results with a timeout) can use `task.gather` with explicit timeout on each task.

### N-7: Missing cancellation semantics for async context manager exit

**Location:** milestone_async_7 work items (lines 582-583): *Define cancellation cleanup behavior for async context managers.*

The work item mentions defining behavior but the contract is not in the acceptance criteria.

**Recommendation:** Add to milestone_async_7 acceptance criteria:
> - When a task is cancelled while inside an `async with` block, the async `__aexit__` method is always called (cancellation cleanup). If `__aexit__` itself is fallible and returns an error, the cancellation error takes precedence — the task is already cancelled, so the `__aexit__` error is logged or propagated as a secondary error through the scope's error aggregator, not as the primary cancellation result.

### N-8: Missing validation for cancellation determinism across task groups

**Location:** milestone_async_3 acceptance criteria (line 412): *Task-group failure cancels unfinished siblings.*

**Recommendation:** Add a specific validation fixture category for cancellation determinism:
- `cancellation_scope_timeout.sifr` — verify timeout cancels only the timed operation.
- `cancellation_group_sibling.sifr` — verify first error cancels all siblings.
- `cancellation_nested_scopes.sifr` — verify cancellation propagates through nested scopes correctly.
- `cancellation_cleanup_runs.sifr` — verify `async with` cleanup runs under cancellation.

### N-9: Milestone sequencing — milestone_async_4 (Send/Sync) should inform milestone_async_5 (sync primitives) design

**Location:** Dependency graph (lines 661-684).

**Recommendation:** The dependency graph is correct as-is (`m4 → m5`). The issue is that **milestone_async_5 must include Send/Sync validation for channel send/receive operations** specifically — not just the primitive types. Add to milestone_async_5 negative validation:
- `channel_non_send_element_rejected.sifr`
- `lock_across_task_boundary_rejected.sifr`

### N-10: Missing `sifr.sync.Barrier` and `sifr.sync.Condvar`

**Location:** Allowed explicit surfaces (lines 108-115).

**Recommendation:** Add to allowed surfaces or explicitly note as deferred:
> **Deferred to v1.1:** `sync.Barrier` and `sync.Condvar` are common coordination primitives. They should be considered for v1.1 after the core primitives (Lock, RwLock, Channel, Semaphore, Notify) are stable. `Barrier` is useful for "wait for N tasks to complete before proceeding" patterns. `Condvar` is lower-level and can be modeled via `Notify` + `Lock` in most cases.

### N-11: `sifr.asyncio` curated subset needs explicit mapping table

**Location:** milestone_async_8 (lines 620-628).

**Recommendation:** Add a mapping table:

| `sifr.asyncio` API | Canonical Sifr equivalent |
|---|---|
| `asyncio.run(fn)` | Direct `async def main()` — no wrapper needed |
| `asyncio.create_task(fn)` | `sifr.task.spawn(fn)` |
| `asyncio.gather(*tasks)` | `sifr.task.gather(*tasks)` |
| `asyncio.TaskGroup` | `sifr.task.TaskGroup` (same type, different module) |
| `asyncio.sleep(delay)` | `sifr.task.sleep(delay)` |
| `asyncio.wait_for(task, timeout)` | `sifr.task.timeout(task, timeout)` |
| `asyncio.Queue` | `sifr.sync.Channel` (bounded channel with `maxsize`) |

This table should be in the proposal as an appendix to milestone_async_8.

### N-12: Python async coverage gap — `threading` compatibility

**Location:** Compatibility scope (lines 168-169, 620-628).

**Recommendation:** Add `sifr.threading` as a compatibility layer alongside `sifr.asyncio`:
> **Add `sifr.threading` compatibility layer to milestone_async_6 or milestone_async_8.** Python's `threading` module is distinct from async concurrency. Sifr should provide `sifr.threading.Lock`, `sifr.threading.Event`, `sifr.threading.Condition`, and `sifr.threading.Thread` as thin wrappers over the same primitives that `sifr.sync` uses (via `std::thread`, `std::sync`). This is important for CPython compatibility and for programs that mix thread-based and async-based concurrency.

### N-13: Missing spawn policy for detached tasks

**Location:** Open Decision #1 (line 741), allowed surfaces (lines 108-115).

**Recommendation:** Lock now:
> **LOCKED: `task.spawn` means scoped spawn only in v1.** Detached spawn is not exposed in v1. All spawned tasks must be scoped via `task.scope()` or `TaskGroup`. This enforces structured concurrency by default. `spawn_detached` can be added in v2 if there is a demonstrated need (long-lived background workers, daemon tasks). Scoped spawn simplifies ownership tracking: the scope owns the task handle, and the compiler can enforce that all handles are joined or cancelled before scope exit.

### N-14: Missing runtime-neutral API boundary validation

**Location:** Runtime Is An Implementation Detail section (lines 131-141).

**Recommendation:** Add a specific validation check:
> **Validation: runtime-neutrality gate.** After milestone_async_2, verify that no Sifr public API exposes `tokio` or any runtime-specific types. The only runtime dependency should be a private `sifr._runtime` module. All public APIs must be in `sifr.task`, `sifr.sync`, or `sifr.concurrent`. A negative validation fixture `runtime_leak_rejected.sifr` should assert that runtime-internal types are not importable from public namespaces.

---

## Modern Language Lessons Comparison

| Feature | Proposal | Modern Language Comparison |
|---|---|---|
| Structured concurrency default | `task.scope()` as default — **good** | Matches Swift (task groups), Kotlin (coroutine scope), Go (goroutines in scopes) |
| Cancellation as typed result | `Cancelled` result variant — **good** | Matches Swift (try-throws), Kotlin (CoroutineCancellationException is leak); Python asyncio's `CancelledError` is an ambient exception leak |
| No implicit Arc/Mutex | Explicit `sync.Lock[T]` — **good** | Matches Rust's philosophy; Swift, Kotlin, and Go all have implicit sharing in different forms |
| Async for CPU-bound | `spawn_blocking` explicit offload — **good** | Matches Rust/Tokio's `spawn_blocking`; Go uses goroutines for everything (simpler but not zero-cost) |
| Lock guards across await | Deferred to open decision — **blocker** (fixed above) | Swift requires `@MainActor` isolation; Rust's `MutexGuard` does not cross await |
| Task as Result | Ambiguous — **blocker** (fixed above) | Go channels return `(T, bool)`; Swift async returns typed values; all avoid context-dependent unwrap |
| Channel multi-consumer | Deferred to open decision — **fixed above** | Tokio uses `mpmc` channels; Go uses `chan T` (multi-producer, single-consumer by default); multi-producer, multi-consumer is the most flexible |
| Select/race cancellation | Open decision — **fixed above** | Go's `select` can have default case; Swift races cancel losers; Kotlin's `select` is non-cancellable by default |
| Async context managers | `async with` in milestone_async_7 — **adequate** | Swift's `AsyncSequence` and Kotlin's `Flow` are more powerful but also more complex |
| Lock guard escape | Locked to scope-bound in this review — **good** | Prevents the Rust `RefCell` borrow-checker escape anti-pattern |

---

## Sifr Principles Fit Assessment

| Sifr Principle | Model Compliance | Notes |
|---|---|---|
| Borrow-by-default | **Full** | Spawn captures checked for borrows; borrow-across-await rejected |
| Result/Option safety | **Partial — blocked** | Blocker #1: ambiguous auto-unwrap for `Task[T, E]` |
| No hidden shared mutable memory | **Full** | No silent Arc/Mutex insertion; explicit primitives only |
| No user-triggerable panic | **Adequate** | Runtime task failures are `Result`, not panic; cancellation is typed |
| No data-dependent `.unwrap()` | **Adequate** | Codegen must not emit `.unwrap()` on task results; covered in phase exit gate |

---

## Final Verdict

**Satisfied with non-blocking suggestions.**

The three blockers (blockers 1-3 above) are all resolvable with precise text changes to `internal_docs/async_concurrency_model.md`. They do not require redesign; they require contract locking. The open decisions that are currently open should be locked in `milestone_async_0` with the text changes recommended in this review.

All non-blocking issues are enrichments: clearer rationale, additional validation fixtures, explicit mapping tables, and scope clarifications. None of them block implementation.

**Required actions before implementation begins:**

1. Lock Blocker #1: `await Task[T, E]` always returns `Result[T, E]`, with explicit `try`/`except` interaction.
2. Lock Blocker #2: Lock guards do not cross await points in v1.
3. Lock Blocker #3: Add cancellation interaction contract for channels in milestone_async_5.

Once these three are locked, the proposal is ready for implementation.
