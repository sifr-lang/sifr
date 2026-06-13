# Async Model Review: GPT Pro 4 Redlines — Claude Pass 2

## Verdict: Satisfied (with one targeted fix applied)

The reviewed model (as of 2026-05-09) is implementation-ready. All GPT Pro 4 redlines are resolved. One additional ambiguity in the `TaskGroup` spawn failure path was found and fixed.

---

## Redline Resolution Status

| # | Redline | Status |
|---|---|---|
| 1 | `select`/`race` loser cleanup failures when winner is `Ok` | **Resolved** — loser cleanup failures surface at owning `TaskScope` exit as `ScopeFailure` |
| 2 | `gather` child cancellation behavior | **Resolved** — `Cancelled` before ordinary error: cancel siblings, return `Cancelled`. Input order breaks ties |
| 3 | Same-task coroutine secondary evidence accumulation | **Resolved** — accumulated on the running task, observable via `TaskResult` or diagnostics on top-level exit |
| 4 | `AsyncClosable[E]` close-error typing obligations in `async for` | **Resolved** — enclosing function must propagate close error type or handle explicitly |
| 5 | User-defined async context manager protocol signatures | **Resolved** — `AsyncContextManager[T, EnterE, ExitE]`, `AsyncExitCause`, `__aenter__` failure behavior |
| 6 | `TaskGroup`/`TaskScope` closed/cancelling spawn rules | **Resolved** — state machine defined; spawn in `Open` only; post-failure spawn statically rejected or returns `GroupClosed` |
| 7 | Core public type inventory incomplete in phase file | **Resolved** — all types listed in milestone 0, private implementation details labeled |
| 8 | Sync primitive signatures too vague | **Resolved** — `Shared`, `Lock`, `RwLock`, `Semaphore`, `Notify` method signatures in milestone 5 |
| 9 | `select` variadic vs binary arity | **Deferred** — not a blocker; binary `select(a, b)` is the v1 API shape, consistent across model and phase file |
| 10 | `BlockingTask` lifecycle not structured enough | **Resolved** — affine handles, `join()`/`cancel_and_join()` consume them, dropping abandons observation |
| 11 | `BlockingTask.join() -> TaskResult` semantic overlap | **Non-blocking** — documented; future `BlockingTaskResult` can be considered if user feedback shows confusion |
| 12 | Validation names imply wrong thing | **Resolved** — clearer fixture names in milestone 3 |
| 13 | Receive cancellation needs exactly-once rule | **Resolved** — message remains available or is not lost until `Ok(value)` returned to receiver |
| 14 | Markdown hygiene: prose in code block | **Resolved** — `TimeoutResult[E]` prose moved outside `sifr` code block |

---

## Additional Fix Applied in Pass 2

### `TaskGroup.spawn` Failure Path

**Issue found:** The model used `GroupClosedError` as the error type for post-failure spawn rejection, but the actual `TaskGroup.spawn` API returns `Task[T, E]` — a type that already embeds the homogeneous error `E`. If spawn is fallible, it must return `Task[T, E] | Closed` or `Result[Task[T, E], GroupClosed]`. Using `GroupClosedError` as a standalone error type is inconsistent with the homogeneous `TaskGroup[E]` design.

**Resolution:** Changed both `async_concurrency_model.md` and `phases/32_async_ecosystem.md` to use `GroupClosed` (no `Error` suffix) as the typed error identifier, with the explicit return type note `Task[T, E] | GroupClosed`. This is the most Sifr-compatible answer:

- It avoids introducing a second error type on `TaskGroup[E]` (homogeneous group already carries `E`)
- It avoids making `TaskGroup.spawn` fallible (the "purely static/flow-diagnostic" rule already applies where statically knowable)
- It allows the dynamic case (post-failure spawn after a dynamic error path) to be typed cleanly as `Task[T, E] | GroupClosed` rather than inventing a phantom `E` variant

**Changed text in `async_concurrency_model.md` (line 509):**
> A `TaskGroup` has `Open`, `Cancelling`, `Closing`, and `Closed` states. `group.spawn(...)` is valid only in `Open`. After first child failure, explicit group cancellation, timeout, or scope exit begins, new spawn attempts are rejected statically when possible and otherwise fail with a typed `GroupClosed` error. For the homogeneous `TaskGroup[E]`, spawn produces `Task[T, E] | GroupClosed`. The same principle applies to `TaskScope`: once `__aexit__` begins, spawning is invalid.

**Changed text in `phases/32_async_ecosystem.md` (line 397):** Same change.

---

## Remaining Non-Blocking Concerns

| Concern | Status | Rationale |
|---|---|---|
| `select` variadic vs binary arity | Deferred | Binary signature (`select(a, b)`) is consistent across model and phase file. Fixed-arity overloads (Select2, Select3, ...) can be added later if variadic is desired. |
| `BlockingTask.join() -> TaskResult` reuse | Non-blocking | `Cancelled` on `BlockingTask` means result abandonment, documented. Acceptable for v1. `BlockingTaskResult` enum can be a future improvement. |

---

## Implementation-Readiness Assessment

**Architecture: Approved**

**Implementation: Ready**

All must-fix items are addressed. The core design is coherent:

- `TaskResult` is distinct from `Result`
- `CancellationError` is not an `Error`
- `try await task_handle` is rejected
- `TaskScope.__aexit__ -> Result[None, ScopeFailure]` exists
- timeout uses `TimeoutResult[E]`, not hidden union types
- timeout context blocks are same-task cancellation scopes
- v1 spawn requires owned/sendable/static captures
- async generators are `AsyncGenerator[T, E]`
- channel endpoint lifetime rules are explicit
- `task.select` is binary heterogeneous; `task.race` is homogeneous-list
- `TaskGroup` spawn failure path is typed as `Task[T, E] | GroupClosed`

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
- [x] Additional fix: `TaskGroup.spawn` failure path typed as `Task[T, E] | GroupClosed`
