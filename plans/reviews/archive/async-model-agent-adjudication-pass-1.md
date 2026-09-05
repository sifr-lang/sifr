# Review Adjudication: agent async-model review

Date: 2026-05-09
Reviewer: agent (adjudication pass)
Source: reviews/async-model-review-agent-4.md
Canonical docs: internal_docs/async_concurrency_model.md, internal_docs/phases/32_async_ecosystem.md, internal_docs/architecture.md

---

## Executive Summary

agent's review is structurally sound on the blocking issues and overstated on several of its rejections. The model has real semantic contradictions that must be fixed before implementation, but the overall direction is right.

**Verdict: BLOCKED until edits. Four blockers, twelve valid refinements, three rejections.**

---

## Blocker 1: Canonical example is type-wrong

**Severity: Blocking**

**What agent says:** The opening example shows:
```sifr
a: str = await first
b: str = await second
```
But `await Task[T, E] -> Result[T, E | CancellationError]`. The example should not type-check as written.

**Assessment: True blocker.** The model itself states:
> `await Task[T, E] always produces Result[T, E | CancellationError]`

Yet the canonical example assigns to `str` directly. This is a self-contradiction in the model document. Implementation will invent a resolution — likely "we auto-unwrap in simple contexts" — and that ad hoc rule will become the real contract.

**Required fix — async_concurrency_model.md, lines 32-39:**
Replace the opening example with one that type-checks under the stated rules:
```sifr
async def main() -> Result[None, Error]:
    async with task.scope() as scope:
        first = scope.spawn(fetch_one("https://example.com/a"))
        second = scope.spawn(fetch_one("https://example.com/b"))

        match await first:
            Ok(a):
                pass
            Err(e):
                return Err(e)
            Cancelled(c):
                return Err(task.ChildCancelled(c))

        match await second:
            Ok(b):
                pass
            Err(e):
                return Err(e)
            Cancelled(c):
                return Err(task.ChildCancelled(c))

        print(a + b)
        return Ok(None)
```

Or document the ergonomic auto-unwrap rule explicitly and show the expanded form as an option. Do not hide the hard case.

Also fix the product decision example in 32_async_ecosystem.md lines 32-40 (same problem).

---

## Blocker 2: Milestone dependency bug — `async with` comes after `task.scope()`

**Severity: Blocking**

**What agent says:** The proposal uses `async with task.scope() as scope` in milestones 2 and 3, but milestone 7 is where `async with` is implemented. `TaskScope` is an async context manager, but the syntax to use it doesn't exist until milestone 7.

**Assessment: True blocker.** This is an unambiguous dependency graph violation.

Looking at the phase plan:
- milestone_async_2: "Implement the minimal `sifr.task.scope` runtime container needed for scoped spawn." Uses `async with task.scope()`.
- milestone_async_3: "Implement `task.scope`" and "Implement `task.TaskGroup`". Uses `async with task.scope()`.
- milestone_async_7: "Implement `async with`."

The model document (milestone_async_0) says `TaskScope` is an async context manager, but milestone 7 implements the protocol. These are contradictory.

**Required fix — 32_async_ecosystem.md:**

Split `async with` into two parts:
1. `milestone_async_1b`: Implement the built-in async context manager protocol for the specific case of `TaskScope` (a compiler-recognized special, not general user-defined). This is minimal — just enough to wire `task.scope()`.
2. `milestone_async_7`: Generalize user-defined async context-manager protocol.

Or: make `task.scope()` return something that can be used without `async with` syntax in v1 (e.g., `scope = task.scope()` followed by `scope.__aenter__()` and `scope.__aexit__()` explicitly), and use `async with` as syntax sugar added in milestone 1b or 7.

The model doc also needs this split. The current milestone_async_0 says `TaskScope` is an async context manager — that definition is correct, but the implementation plan conflicts with it.

**Concrete edit to 32_async_ecosystem.md milestone_async_1:**
Add a scope item:
```markdown
- Implement minimal `async with` syntax support for built-in `task.scope()` as a compiler-recognized construct. General user-defined async context-manager protocol is milestone_async_7. The minimal version does not introduce a public `__aenter__`/`__aexit__` protocol — it wires `task.scope()` specifically.
```

---

## Blocker 3: Channel semantics have double-closed-state smell

**Severity: Blocking**

**What agent says:** `channel.receive()` returns `Result[Option[T], ClosedError]`. `None` means closed-and-drained. `ClosedError` also means closed. Two different ways to say "closed" on receive.

**Assessment: True blocker.** This is an underspecified API that will produce runtime blocking bugs.

The model says:
> `channel.receive()` returns `Result[Option[T], ClosedError]`
> `None` indicates graceful end-of-stream after close and drain

So `None` = closed + drained, and `ClosedError` = ... what exactly? The proposal doesn't say when `ClosedError` fires on receive. Is it when the channel is closed but not drained? When the sender dropped without closing? When the channel is closed and the buffer has remaining items?

Also: bounded channels apply backpressure, so `send` on a full bounded channel cannot be synchronous. The proposal says `sync.Channel[T]` but the operations are async in practice. This needs explicit API signatures.

**Required fix — async_concurrency_model.md, type system section:**

Add explicit method signatures for channels:
```sifr
# Unbounded channel
sync.channel[T]() -> (sync.ChannelSender[T], sync.ChannelReceiver[T])

# Bounded channel
sync.bounded_channel[T](capacity: int) -> (sync.ChannelSender[T], sync.ChannelReceiver[T])

# Sender API
class ChannelSender[T]:
    async def send(value: T) -> Result[None, ClosedError]
    # ClosedError: channel is closed, send failed

# Receiver API
class ChannelReceiver[T]:
    async def receive() -> Result[T, ClosedError]
    # ClosedError: channel is closed and will never produce more values
    # (no `Option` — draining happens via explicit channel close + drain loop)
```

Remove `Result[Option[T], ClosedError]` from the channel contract. A closed channel with drained buffer is signaled by `ClosedError` on receive. A receiver that succeeds gets `T`. A receiver on a closed channel gets `ClosedError`. Users who need end-of-stream detection can use `channel.receive()` in a loop and handle `ClosedError`.

Alternatively, keep `Option[T]` and document that `ClosedError` on receive means "the channel closed in an unexpected way (sender dropped without close)" vs `None` means "graceful drain". But pick one and document it.

Also add:
- `channel.close()`: closes the channel, wakes pending senders/receivers
- `Sender` is clonable (multi-producer)
- `Receiver` is not clonable (single-consumer per receiver handle)
- Dropping all senders does not automatically close the channel (must call `close()` explicitly)

---

## Blocker 4: `sync.Lock` in async code still blocks the runtime

**Severity: Blocking (safety hazard)**

**What agent says:** `sync.Lock` uses a synchronous Rust mutex. If contended, it blocks the OS thread, not just the async task. The proposal's fix (lock guards not crossing `await`) is necessary but not sufficient.

**Assessment: True blocker.** The model currently says:
> `sync.Lock[T]` uses a synchronous Rust mutex internally in v1
> lock guards must not cross `await` points in v1

This is half a fix. The guard can't cross `await`, but the mutex acquisition itself can block the runtime worker thread. If a task holds a lock and takes a long time, or if contention is high, the worker thread is blocked.

**Required fix — async_concurrency_model.md, lock policy section:**

Add explicit warning:
```markdown
### Lock Policy

`sync.Lock[T]` uses a synchronous Rust mutex internally in v1. **Warning:** Acquiring `sync.Lock` in async code may block the current runtime worker under contention. It is permitted only for short critical sections with low contention. For most async concurrency, prefer `sync.Channel` for coordination or `sync.AsyncLock` (deferred to v2).

- `lock()` is not await-aware and returns a guard restricted to a synchronous lexical scope
- lock guards must not cross `await` points in v1
- the type checker rejects a live `LockGuard` or `RwLockGuard` at an `await` point
- diagnostic: "lock guard is still live at this await point; lock guards cannot cross await points in v1"
- help: "release the lock before the await, or use a channel to communicate results instead"
- a distinct `sync.AsyncLock[T]` is deferred to v2
```

Also update the type system section:
```markdown
`sync.Lock[T]` uses a synchronous Rust mutex internally in v1. Acquiring this lock while holding an async runtime worker may block the worker thread. Use only for short, uncontended critical sections. For async-safe locking, a future `sync.AsyncLock[T]` will be added.
```

---

## Valid Refinements (should be addressed, not blockers)

### Refinement 1: Distinguish `Coroutine`, `Task`, `TaskResult` explicitly

agent says to define `Coroutine[T, E]`, `Task[T, E]`, `TaskResult[T, E]` as separate types. The current model conflates them.

**Assessment: Valid refinement.** The model does describe the distinction implicitly, but the type language is blurred. When users ask "what does calling an async function return?" the answer should not be "it depends." Even if the initial implementation only exposes `Task[T, E]`, the model should document the lifting rules explicitly.

**Edit to async_concurrency_model.md, type system section:**
After the `Task[T, E]` definition, add:
```markdown
### Async Type Ladder

Calling an async function returns an *unscheduled async computation*. In the initial model, this is implicitly wrapped as a `Task[T, E]` when spawned:

```sifr
async def fetch_one(url: str) -> Result[str, NetworkError]
# return type is the async function's error channel; never a Result wrapper

scope.spawn(fetch_one(url)) -> Task[str, NetworkError]
# spawn wraps the coroutine as a task handle
```

Awaiting a same-task coroutine directly (not through spawn):
```sifr
await fetch_one(url) -> Result[str, NetworkError]
# same-task await does not introduce CancellationError — there is no task boundary
```

Awaiting a spawned task handle:
```sifr
await task_handle -> Result[T, E | CancellationError]
# task handle await introduces the CancellationError branch
```

A future `Coroutine[T, E]` type (before spawn) may be introduced for explicit coroutine manipulation. In v1, the implicit wrapping is acceptable as long as the lifting rules above are honored.
```

This doesn't change behavior but makes the rules explicit.

### Refinement 2: `try await` cancellation propagation rule

agent says: `try await task` — what happens when cancellation occurs? Can it propagate?

**Assessment: Valid refinement.** The current model says:
> outside `try`, the observable expression type remains `Result[T, E | CancellationError]`
> auto-unwrap is sequenced after await: first `await` produces `Result[T, E | CancellationError]`, then `try` unwraps ordinary `E` errors

This is ambiguous about what `try await task` does when the result is `Cancelled`. The current rule says `try` only unwraps ordinary errors, so `try await task` with a `Cancelled` result would not auto-unwrap — the `Cancelled` would propagate. But this needs to be stated explicitly.

**Edit to async_concurrency_model.md:**
In the "typed failure and cancellation" section, add:
```markdown
### `try await` Cancellation Behavior

Inside a `try` block, `try await task_handle` on a task handle produces `Result[T, E | CancellationError]`. The `try` auto-unwrap applies only to ordinary `E` errors. If the result is `Cancelled`, it propagates unchanged — it is NOT caught by `except Error`.

```sifr
async def main() -> Result[None, Error]:
    try:
        result: str = try await some_task  # auto-unwrap only on Ok or Err(E)
    except Error as e:
        return Err(e)
    # Cancelled propagates — not caught by except Error
```

A task that is cancelled while being awaited in a `try` block will propagate the `CancellationError` upward to the nearest non-`try` context or explicit `match` on `TaskResult`.
```

### Refinement 3: `gather`, `select`, `race`, `timeout` exact type signatures

agent says these compositors lack exact result type signatures.

**Assessment: Valid refinement.** The model describes behavior but not exact return types. Even approximate type expressions would help implementation.

**Edit to async_concurrency_model.md, scope boundaries section:**
Add a compositors subsection:
```markdown
### Task Compositor Signatures

```sifr
# gather — fail-fast, ordered results
task.gather(handles: list[Task[T, E]]) -> TaskResult[list[T], E]
# Ok(list[T]) if all succeed
# Failed(Failure{E}) if any fails — failures during cancellation are secondary

# select — first completion, heterogeneous
task.select[A, EA, B, EB](a: Task[A, EA], b: Task[B, EB])
    -> Select2[TaskResult[A, EA], TaskResult[B, EB]]

enum Select2[A, B]:
    First(A)
    Second(B)

# race — homogeneous first completion
task.race[T, E](handles: list[Task[T, E]]) -> TaskResult[T, E]
# Ok(T) for winner, Cancelled for losers

# timeout — wraps a task handle
task.timeout(handle: Task[T, E], duration: Duration) -> TaskResult[T, E | TimeoutError]
# TimeoutError is a distinct failure type, not CancellationError
# If inner completes before deadline: Ok(T) or Failed(E)
# If deadline expires first: Cancelled(TimeoutError) or equivalent TimeoutError branch
```

The exact enum representation of `TaskResult` is an implementation choice. The key semantic distinction is that `Cancelled` is structurally distinct from `Failed(E)` and from any ordinary error type.
```

### Refinement 4: `TaskScope` normal exit — nursery ownership model

agent says the orphaned handle rule and the `__aexit__` safety backstop contradict each other. If handles must be consumed, the backstop is unnecessary. If the scope owns children, why require consumption?

**Assessment: Valid refinement.** The current hybrid is confusing.

**Decision to record:** `TaskScope` uses **nursery ownership**:
- The scope owns every child. Handles are optional observers.
- `scope.spawn()` returns a handle that can be awaited, but is not required to be awaited.
- `TaskScope.__aexit__` waits for all children on normal exit — no handles required.
- Orphaned handles (not awaited) are not compile-time errors in `TaskScope`.
- `TaskGroup` is stricter: handles used in fail-fast composition must be awaited or consumed.

**Edit to async_concurrency_model.md:**
Replace the orphaned handle rules with:
```markdown
### TaskScope Ownership Model

`TaskScope` uses nursery ownership:
- Every spawned child belongs to the scope.
- `scope.spawn(...)` returns a task handle as an observer.
- Handles are not required to be awaited, joined, or cancelled. The scope owns the child.
- `TaskScope.__aexit__` waits for all children on normal exit.
- `TaskScope.__aexit__` cancels and waits for cleanup on abnormal exit (exception, cancellation).

`TaskGroup` adds composition policy:
- `gather`, `select`, and `race` consume handles and require explicit ownership transfer.
- Handle consumption for composition is enforced; orphan handles for simple fire-and-wait are not.

Compile-time diagnostics for handles are only required for composition APIs (`gather`, `select`, `race`), not for general `TaskScope` usage.
```

Update 32_async_ecosystem.md milestone_async_3 orphaned handle rules accordingly.

### Refinement 5: "Tracked collection" proof is too complex for v1

agent says: "a handle moved into a collection is tracked only when the compiler can prove the collection is drained, consumed, or dropped before task-scope exit" is too complex for milestone 3.

**Assessment: Valid. Simplify v1.**

**Edit to async_concurrency_model.md and 32_async_ecosystem.md milestone_async_3:**
Replace the tracked collection rule with:
```markdown
### Handle Collection in v1

v1 allows handles to be moved into explicit collection consumption APIs only:
- `task.gather(handles)`
- `task.select(handles)`
- `task.race(handles)`
- explicit `for h in handles: await h` loops

General collection tracking (proving a collection is drained before scope exit) is deferred to v2. Handles moved to arbitrary user collections in v1 are not tracked — the scope waits for all children on exit regardless of handle consumption.
```

### Refinement 6: `SecondaryError` needs structural definition

agent says `SecondaryError` is mentioned repeatedly but its type is unspecified.

**Assessment: Valid refinement.** The model says `SecondaryError` is structured evidence attached to failures, but doesn't define its structure.

**Edit to async_concurrency_model.md, error types section:**
```markdown
`SecondaryError`: structured evidence for cleanup or sibling failures that occur during unwinding. Attached to a primary `Failure[E]`:

```sifr
struct Failure[E]:
    primary: E
    secondary: list[SecondaryError]

enum SecondaryError:
    CleanupFailed(error: Error, location: str)
    SiblingFailed(error: Error, task_id: str)
    CancellationDuringCleanup(cause: CancellationError)
```

`SecondaryError` is inspection evidence. It never masks the primary `Failure.primary`. Users can inspect it for diagnostics, logs, or debugging. A future collect-all API may expose it as a first-class return value, but in v1 it is observable metadata only.
```

### Refinement 7: Timeout should only accept task handles (Option A)

agent says: timeout wrapping arbitrary enclosed operations requires internal spawning, which imposes Send/ownership rules. The proposal acts like all forms are easy.

**Assessment: Valid. Clarify scope.**

**Edit to async_concurrency_model.md, timeout section:**
```markdown
### Timeout API Forms

`task.timeout(handle: Task[T, E], duration)` — accepts a task handle. If the timeout wins, cancel the child task, wait for cleanup, return `TimeoutError`.

`task.timeout(duration)` — context-manager form for inline blocks:
```sifr
async with task.timeout(5.seconds):
    await some_work()
```
This internally creates a child task for the enclosed block.

Arbitrary enclosed awaitables (not task handles) are not supported in v1. If you have an arbitrary awaitable, spawn it into a child task first and pass the handle to `task.timeout`.
```

### Refinement 8: `Shared[T]` needs `ShareSafe` capability rule

agent says: what if `T` contains interior mutability?

**Assessment: Valid refinement.** The current wording is insufficient.

**Edit to async_concurrency_model.md:**
In the sync primitives section:
```markdown
`sync.Shared[T]` exposes shared ownership. It requires `T` to be share-safe:
- `T` must be `Send + Sync`
- `T` must not contain interior mutability that is not itself share-safe
- `Shared[Cell[int]]` is rejected
- `Shared[List[MutableThing]]` is rejected
- `Shared[ImmutType]` is allowed

If a type contains internal synchronization (e.g., a type that wraps its own `Mutex`), it must still be `Send + Sync` for `Shared[T]` to be valid. The `ShareSafe` requirement is satisfied by `Send + Sync` auto-derivation in v1.
```

### Refinement 9: `spawn_blocking` lifetime/cancellation policy needs explicit rules

agent says: if blocking work continues after scope exit, it cannot borrow from the scope.

**Assessment: Valid refinement.**

**Edit to async_concurrency_model.md, blocking section:**
```markdown
### `spawn_blocking` Lifetime Rules

`spawn_blocking` requires owned + sendable + static captures:
- Local borrows cannot be passed to `spawn_blocking` because cancellation cannot stop already-running OS work.
- If a blocking task is cancelled while running, the handle result is dropped/abandoned.
- Already-running blocking work completes regardless of scope exit.
- `spawn_blocking` does not support scoped borrowed captures in v1.
- If your blocking work needs access to local state, pass owned copies (`.clone()` for cloneable types, or construct the data before the call).
- v1 does not forcibly abort a running OS thread.
```

### Refinement 10: `select` tie-breaking should use input order

agent says: handle creation order is less intuitive than input order.

**Assessment: Valid. Use input order.**

**Edit to async_concurrency_model.md:**
```markdown
Selection tie-breaking: if multiple tasks complete in the same scheduler tick, input order breaks ties. `task.select(a, b)` prefers `a` in a tie, not whichever was spawned first. This is easier to teach and test.
```

Update 32_async_ecosystem.md accordingly.

### Refinement 11: Cancel-hangs-cleanup needs explicit section

agent says: "cancellation waits for cleanup" can hang forever if cleanup loops.

**Assessment: Valid refinement. Add a cancellation progress guarantees section.**

**Edit to async_concurrency_model.md:**
```markdown
### Cancellation Progress Guarantees

Cancellation is cooperative:
- Cancellation requests cooperation from the target task at await points.
- Cleanup (`async with` exits, finally blocks) runs to completion after cancellation.
- If cleanup never completes (infinite loop, deadlock), scope exit hangs. This is accepted behavior — a stuck cleanup is a programmer bug, not a runtime abort.
- v1 does not expose a forceful abort path for cleanup hangs.
- Panic during cleanup: caught at the runtime/codegen boundary, surfaced as `SecondaryError.CancellationDuringCleanup`, does not become a double-panic abort.

Cancellation is not guaranteed to be immediate. A task that does not await (e.g., a tight CPU loop) cannot be cancelled until it reaches a cooperative cancellation point.
```

### Refinement 12: Compatibility mapping needs divergence documentation

agent says the mapping table (`Event -> Notify`, `Queue -> Channel`, etc.) overpromises.

**Assessment: Valid refinement. Expand the compatibility table.**

**Edit to 32_async_ecosystem.md, milestone_async_8 compatibility mapping:**
Add a third column for intentional divergence:

| Compatibility API | Sifr equivalent | Divergence |
|---|---|---|
| `asyncio.Event` | `sifr.sync.Notify` | Event is level-triggered (set, all waiters pass until clear); Notify is edge-triggered (wake current waiters only). Use `sync.Shared[bool] + Notify` for level-triggered behavior. |
| `threading.Event` | `sifr.sync.Notify` | Same level-triggered vs edge-triggered divergence as asyncio.Event. |
| `threading.Condition` | `sifr.sync.Notify + sifr.sync.Lock` | Condition variable has predicate discipline; Notify + Lock is a close approximation but the wait semantics differ. |
| `asyncio.Queue` | `sifr.sync.Channel` | Queue has `task_done`/`join` semantics for producer-consumer completion tracking; Channel does not. Use a completion channel for equivalent behavior. |
| `concurrent.futures.Future` | `sifr.task.Task` (alias) | Thread-pool futures are waited from sync code and may block the waiting thread; Task handles are scoped to async lifetimes and awaited from async code. Not identical. The alias is a naming convenience, not a semantic guarantee of identical behavior. |
| `asyncio.create_task` | `scope.spawn(fn)` | Sifr's version only works inside an explicit task scope. It does not create ambient orphan tasks. Error if used outside a scope. |

---

## Rejections (agent is wrong or premature)

### Rejection 1: `E | CancellationError` requires a union type system

agent says: does Sifr actually have union types? If not, this notation is misleading.

**Assessment: Wrong.** Sifr already has union types — `int | str` is a valid Sifr type. The `Result[T, E | CancellationError]` notation is just sugar for `Result[T, Union[E, CancellationError]]`, which is valid Sifr syntax. The model does not need to introduce new union semantics; it uses the existing ones.

**No change needed.** The notation is valid.

### Rejection 2: `Event -> Notify` is wrong

agent says: Event is level-triggered, Notify is edge-triggered. They're not the same.

**Assessment: Partially right on the semantics, but the proposed fix is already the plan.** The compatibility mapping is intended as an approximation, and the document does not claim equivalence. The refinement above (compatibility divergence table) addresses this. Calling it "wrong" is overstated — the mapping is labeled as a compatibility veneer, not identical behavior. The fix is to document the divergence clearly, not to reject the mapping.

**No fundamental change needed.** The refinement in section Refinement 12 is sufficient.

### Rejection 3: Scoped borrows across spawned tasks is too ambitious for v1

agent says: v1 should use owned-only captures for `spawn`.

**Assessment: Overcautious.** The current model already handles this through the Send/Sync boundary checking in milestone_async_4. If a borrow is not Send, it is rejected at spawn. The "scoped" part (proving the borrow outlives the task lifetime) is what the model already captures in the borrow table:
> immutable borrow across scope.spawn: allowed only when the scoped lifetime proves the task cannot outlive the borrow and the referent is share-safe

This is already conservative — it requires both lifetime proof AND share-safety. The `spawn_scoped_borrow_ok.sifr` validation fixture tests exactly this rule. The model is not over-promising; it's correctly stating the requirement.

**No change needed.** The model already has the conservative stance.

---

## Summary of Required Edits

### async_concurrency_model.md

1. **Blocker 1**: Rewrite canonical example to type-check (lines 32-39)
2. **Blocker 4**: Add `sync.Lock` worker-blocking warning (lock policy section)
3. **Refinement 1**: Add async type ladder documentation (type system section)
4. **Refinement 2**: Document `try await` cancellation behavior (typed failure section)
5. **Refinement 3**: Add compositor type signatures (scope boundaries section)
6. **Refinement 4**: Record nursery ownership model for TaskScope (scope boundaries section)
7. **Refinement 5**: Simplify tracked collection rule (milestone_async_3 section)
8. **Refinement 6**: Define `SecondaryError` structure (error types section)
9. **Refinement 7**: Clarify timeout accepts task handles only (timeout section)
10. **Refinement 8**: Add `ShareSafe` rule for `Shared[T]` (sync primitives section)
11. **Refinement 9**: Document `spawn_blocking` lifetime rules (blocking section)
12. **Refinement 10**: Use input order for select tie-breaking (selection policy section)
13. **Refinement 11**: Add cancellation progress guarantees section (cancellation section)

### 32_async_ecosystem.md

1. **Blocker 2**: Fix milestone dependency — add minimal `async with` for `task.scope()` in milestone 1 (or split milestone 1)
2. **Blocker 1**: Fix product decision example (same type-error as model doc)
3. **Refinement 4**: Update orphaned handle rules to nursery ownership model
4. **Refinement 5**: Simplify tracked collection rule
5. **Refinement 10**: Use input order for select tie-breaking
6. **Refinement 12**: Expand compatibility mapping table with divergence column

### Blockers requiring coordination between documents

- Blockers 1 and 2 appear in both documents and must be fixed in both.
- The dependency graph fix (Blocker 2) affects milestone sequencing — the dependency graph in both documents must be updated.

---

## Final Verdict

**BLOCKED until edits.**

The agent review identifies four real blockers:
1. The canonical example is type-wrong (self-contradiction in the model)
2. Milestone dependency bug (async with used before it's implemented)
3. Channel receive API has double-closed-state smell
4. sync.Lock blocks runtime workers (warning missing)

And twelve valid refinements that improve precision without breaking the design.

Three of agent's items are rejected: the union type notation is already valid Sifr syntax; the Event -> Notify mapping is documented as an approximation with the divergence already addressable via refinement 12; and scoped borrows are already handled conservatively through Send/Sync checking.

The overall design is sound. The fixes are targeted and do not require redesign. Once the four blockers and twelve refinements are applied, the model is implementation-ready.