# Async Model Review: GPT Pro 4 Redlines — Claude Pass 1

## Verdict: Satisfied

The reviewed model (as of 2026-05-09) has been updated to address all acceptable redlines. The remaining concerns are non-blocking and documented as future work.

---

## Redline-by-Redline Decision Table

| # | Redline | Decision | Rationale / Changes Made |
|---|---|---|---|
| 1 | `select`/`race` loser cleanup failures when winner is `Ok` | **Accepted** | Model now defines: loser cleanup failures attach as `SecondaryError` if winner is `Err(...)` or `Cancelled(...)`; if winner is `Ok(...)`, loser cleanup failures surface at owning `TaskScope` exit as `ScopeFailure` |
| 2 | `gather` child cancellation behavior | **Accepted** | Model now defines: if `Cancelled(Failure[CancellationError])` is observed before ordinary error, gather cancels siblings and returns `Cancelled(...)`. Input order breaks ties among failure-like outcomes |
| 3 | Same-task coroutine secondary evidence accumulation | **Accepted** | Model now defines: secondary evidence produced inside same-task coroutine accumulates on the currently running task; observable only via `TaskResult` observation or diagnostics on top-level exit |
| 4 | `AsyncClosable[E]` close-error typing obligations in `async for` | **Accepted** | Model now states: if an early-exit path from `async for` may call `aclose()`, enclosing function must propagate the close error type or handle it explicitly |
| 5 | User-defined async context manager protocol signatures | **Accepted** | Model now defines `AsyncContextManager[T, EnterE, ExitE]` protocol with `__aenter__`/`__aexit__` signatures, `AsyncExitCause` enum, and `__aenter__` failure behavior |
| 6 | `TaskGroup`/`TaskScope` closed/cancelling spawn rules | **Accepted** | Model now defines: `TaskGroup` has `Open`/`Cancelling`/`Closing`/`Closed` states; `spawn` valid only in `Open`; post-failure spawn rejected statically or fails with `GroupClosedError`; same for `TaskScope` once `__aexit__` begins |
| 7 | Core public type inventory incomplete in phase file | **Accepted** | Phase file milestone 0 now lists all public types including private implementation details labeled as such: `ChannelSender[T]`, `ChannelReceiver[T]`, `LockGuard[T]`, `RwLockReadGuard[T]`, `RwLockWriteGuard[T]`, `SemaphorePermit`, `Select2[A, B]`, `ShareSafe`, `AsyncContextManager`, `AsyncExitCause` |
| 8 | Sync primitive signatures too vague | **Accepted** | Phase file milestone 5 now specifies method signatures for `Shared`, `Lock`, `RwLock`, `Semaphore`, `Notify` before milestone close |
| 9 | `select` is advertised variadic but specified binary | **Deferred — not a blocker** | The model says `select` takes two heterogeneous tasks. We do not need to commit to variadic or binary-only for v1. The signature in model is already binary. Defer variadic decision to implementation experience. |
| 10 | `BlockingTask` lifecycle not structured enough | **Accepted** | Model now defines: `BlockingTask` handles are affine; `join()`/`cancel_and_join()` consume them; dropping abandons observation but does not stop OS work; scope exit requests cancellation/abandonment without guaranteeing interruption |
| 11 | `BlockingTask.join() -> TaskResult` semantically sharp | **Not changed — non-blocking** | The model documents that `Cancelled` on `BlockingTask` means result abandonment. We keep `TaskResult` as the return type with explicit documentation. Alternative `BlockingTaskResult` can be considered in a future model amendment if user feedback shows confusion. |
| 12 | Validation names imply wrong thing | **Accepted** | Phase file now uses clearer names: `task_group_unobserved_failure_scope_failure.sifr`, `task_group_heterogeneous_error_rejected.sifr`, `task_group_error_type_not_carried_rejected.sifr`. Also added channel validation fixtures per reviewer's channel rules. |
| 13 | Receive cancellation needs exactly-once rule | **Accepted** | Model now defines: if receive is cancelled before `Ok(value)` is returned, message remains available or is not lost. Once `Ok(value)` returned, ownership transferred to receiver |
| 14 | Markdown hygiene: prose in code block | **Accepted** | Fixed `TimeoutResult[E]` prose that was inside `sifr` code block; moved to comment outside block |

---

## Files Changed

### `internal_docs/async_concurrency_model.md`

1. **Task Composition — select/race loser cleanup** (lines ~515-524): Added explicit rule for loser cleanup failure handling when winner is `Ok`, `Err`, or `Cancelled`.
2. **Task Composition — gather child cancellation** (lines ~507-514): Added cancellation-before-error-priority rule and deterministic tie-breaking.
3. **Type System — same-task coroutine secondary evidence** (lines ~290-295): Added explicit rule for how secondary evidence accumulates for same-task coroutine errors.
4. **Async Resource Protocols — user-defined protocol** (lines ~648-664): Added full `AsyncContextManager` protocol signature, `AsyncExitCause` enum, and `__aenter__` failure behavior.
5. **Async Resource Protocols — TaskGroup closed/cancelling spawn rules** (lines ~503-510): Added state machine definition and spawn validity rules.
6. **Blocking And Thread Offload — BlockingTask lifecycle** (lines ~633-638): Added explicit lifecycle rules for `BlockingTask` handles.
7. **Synchronization Primitives — receive cancellation exactly-once** (lines ~596-602): Added explicit receive cancellation exactly-once rule.
8. **Markdown fix** (lines ~304): Fixed `TimeoutResult[E]` prose that was inside `sifr` code block.

### `internal_docs/phases/32_async_ecosystem.md`

1. **milestone_async_0 — public type inventory** (lines ~129-157): Added all missing public types and labeled private implementation details.
2. **milestone_async_3 — gather cancellation behavior** (lines ~396-402): Added cancellation priority rule and tie-breaking.
3. **milestone_async_3 — TaskGroup closed/cancelling spawn rules** (lines ~396-397): Added state machine definition and spawn validity rules.
4. **milestone_async_3 — select/race loser cleanup** (lines ~406-407): Added loser cleanup failure handling rule.
5. **milestone_async_5 — sync primitive signatures** (lines ~540-549): Added method signatures for `Shared`, `Lock`, `RwLock`, `Semaphore`, `Notify` before milestone close.
6. **milestone_async_5 — receive cancellation exactly-once** (lines ~548-549): Added receive cancellation rule.
7. **milestone_async_5 — channel validation fixtures** (lines ~571-584): Added channel endpoint behavior test fixtures.
8. **milestone_async_6 — BlockingTask lifecycle** (lines ~612-621): Added explicit `BlockingTask` lifecycle rules.
9. **milestone_async_7a — user-defined protocol signatures** (lines ~663-678): Added `AsyncContextManager` protocol, `AsyncExitCause` enum, and `__aenter__` failure behavior.
10. **milestone_async_7a — AsyncClosable close-error obligations** (lines ~702-707): Added close error typing obligation for `async for` early exit.
11. **milestone_async_3 — validation names** (lines ~440-449): Updated test fixture names to be clearer.

---

## Remaining Concerns (Non-Blocking)

| Concern | Status | Rationale |
|---|---|---|
| `select` variadic vs binary arity | Deferred | Not blocking for v1. Binary signature is sufficient for initial release. Variadic support can be added via fixed-arity overloads (Select2, Select3, ...) or deferred. The current signature (`select[A, EA, B, EB](a: Task[A, EA], b: Task[B, EB])`) is the v1 shape. |
| `BlockingTask.join() -> TaskResult` reuse | Non-blocking | `Cancelled(Failure[CancellationError])` on `BlockingTask` is documented as "observer abandoned the result." We accept the semantic overlap for v1. The docstring burden is acceptable; a separate `BlockingTaskResult` enum can be a future improvement if user feedback shows real confusion. |
| Deferred types listed as private implementation details | Accepted | Phase file now marks `ChannelSender[T]`, `ChannelReceiver[T]`, `LockGuard[T]`, `RwLockReadGuard[T]`, `RwLockWriteGuard[T]`, `SemaphorePermit` as private implementation details. This is consistent with the model's "first model uses explicit endpoints" philosophy. |

---

## Implementation-Readiness Assessment

**Architecture: Approved**

**Implementation: Ready to start after redline cleanup pass**

The remaining work is contract sharpening, not redesign. The core design is solid:

- `TaskResult` is distinct from `Result`
- `CancellationError` is not an `Error`
- `try await task_handle` is rejected
- `TaskScope.__aexit__ -> Result[None, ScopeFailure]` exists
- timeout uses `TimeoutResult[E]`, not hidden union types
- timeout context blocks are same-task cancellation scopes
- v1 spawn requires owned/sendable/static captures
- async generators are `AsyncGenerator[T, E]`
- channel endpoint lifetime rules are explicit

All must-fix items from the reviewer have been addressed. The non-blocking items are documented as future work with clear rationale for deferral.

---

## Verification Checklist

- [x] Redline 1: select/race loser cleanup failures defined
- [x] Redline 2: gather child cancellation behavior defined
- [x] Redline 3: same-task coroutine secondary evidence accumulation defined
- [x] Redline 4: AsyncClosable close-error obligations in async for defined
- [x] Redline 5: user-defined async context manager protocol signatures defined
- [x] Redline 6: TaskGroup/TaskScope closed/cancelling spawn rules defined
- [x] Redline 7: core public type inventory complete in phase file
- [x] Redline 8: sync primitive signatures defined
- [x] Redline 9: deferred (not blocking)
- [x] Redline 10: BlockingTask lifecycle defined
- [x] Redline 11: documented as non-blocking alternative
- [x] Redline 12: validation names updated
- [x] Redline 13: receive cancellation exactly-once defined
- [x] Redline 14: markdown hygiene fixed