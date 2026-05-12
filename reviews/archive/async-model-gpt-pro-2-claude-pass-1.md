

---

# External Review Evaluation: Sifr Async/Concurrency Model (gpt-pro-2)

## Summary of the Review

The reviewer acknowledges massive improvement since prior rounds. The `Coroutine`/`Task`/`TaskResult` split is sound, the `TaskScope` nursery ownership model is coherent, channels are much better, and the compatibility section is honest. **The model is close to implementation-ready but has 20 remaining "state-machine, type-shape, and API-signature holes."**

My task: evaluate each concern, categorize as blocker/fix/reject/defer, and propose exact edits.

---

## Categorized Decisions

### Category A: ACCEPTED BLOCKERS (must fix before implementation)

#### 1. `task.timeout(duration)` context-manager desugaring (model lines 457-458)

**Concern:** The contract says "uses the same completion-vs-deadline policy through structured scope cancellation" but never defines where `TimeoutError` goes when using `async with task.timeout(5.s)`. The review asks for a precise rule.

**Decision: ACCEPT AS BLOCKER. Fix with Option A (compiler-recognized cancellation scope returning `Err(TimeoutError)`).**

Rationale: The context-manager form is explicitly listed in milestone_async_1 as a built-in form alongside `task.scope()`, and it's the canonical target for `sifr.asyncio.timeout(duration)`. Users will hit this immediately. Leaving it hand-wavy forces codegen to invent behavior.

**Proposed model edit** (add under "Timeout Semantics" section, replacing line 457-458):

```sifr
`task.timeout(duration)` is the async context-manager form used for inline blocks.
It is a compiler-recognized scope that desugars to:
  - spawn the block body as a child task of a temporary inner scope
  - apply the same completion-vs-deadline race policy as `task.timeout(handle, duration)`
  - if the deadline wins: cancel the child, await cleanup, and return Err(TimeoutError) as scope exit
  - if inner completion wins first: return Ok(None) from the block
  - if the enclosing function is Result-returning, TimeoutError propagates through the ordinary error channel
  - if the enclosing function is Result[None, E], the block's TimeoutError is wrapped as Err(TimeoutError)
  - Cancellation during the block is handled identically to task.scope() abnormal exit
```

Also add to the model invariants:

```sifr
23. `async with task.timeout(duration)` desugars to structured scope cancellation; TimeoutError from the deadline surfaces as Err(TimeoutError) from the enclosing block, never as cancellation evidence.
```

#### 2. `ScopeFailure` type for unobserved child failures (model line 419)

**Concern:** "surfaced at scope exit as structured scope failure evidence" — but what is the actual type? The review proposes `struct ScopeFailure` with `enum ScopeFailureCause`. This is a real blocker because it affects async-with lowering, return-type checking, and diagnostics.

**Decision: ACCEPT AS BLOCKER. Implement Option B from the review (type-erased scope failure).**

Rationale: This is the reviewer's strongest practical point. Without a defined `__aexit__` return type, implementers will make incompatible choices. Option B (type-erased `ScopeFailure: Error`) is the most practical for v1 and matches Sifr's error hierarchy.

**Proposed model edit** (add after line 291, before Core API Signatures):

```sifr
### Scope Failure

When a `TaskScope` exits and has unobserved children that failed, the scope exit produces a typed scope failure:

```sifr
struct ScopeFailure:
    primary: ScopeFailureCause
    secondary: list[SecondaryError]

enum ScopeFailureCause:
    UnobservedChildFailed(error: Error, task_id: str)
    UnobservedChildCancelled(cause: CancellationError, task_id: str)
```

`TaskScope.__aexit__` returns `Result[None, ScopeFailure]`. A normal scope exit with all children successful returns `Ok(None)`. A scope with unobserved failed children returns `Err(ScopeFailure(...))`. Explicit observation means: `await handle`, `gather`, `select`, `race`, `timeout`, or `join` marks the child as observed. `TaskScope` therefore requires all fallible children to be either explicitly observed or the scope must be declared to handle `ScopeFailure`.

Note: `TaskScope` in v1 requires all spawned children to be either `Task[T, Never]` (no error channel) or explicitly observed before scope exit. If a child is `Task[T, E]` and not observed, the scope exit type must account for `ScopeFailure`.
```

Also add to model invariants:

```sifr
24. TaskScope.__aexit__ returns Result[None, ScopeFailure]. All spawned Task[T, E] children must be explicitly observed before scope exit, or the scope type must propagate ScopeFailure.
```

#### 3. `TaskGroup` API is name-only (model line 422)

**Concern:** `TaskGroup` is in the locked v1 surface but its API signatures are less precise than `gather`, `select`, and `timeout`. The review asks for signatures: spawn form, return type, sibling-failure policy, heterogeneous error handling.

**Decision: ACCEPT AS BLOCKER. Define TaskGroup API now, not in implementation.**

Rationale: `TaskGroup` is explicitly in milestone_async_3. Deferred definition means implementation will invent behavior. The sibling-cancellation-on-failure behavior is already mentioned but not typed.

**Proposed model edit** (replace model line 422 with):

```sifr
`TaskGroup` adds sibling-failure policy on top of task scopes. A plain `TaskScope` owns lifetime; a `TaskGroup` owns group error behavior and cancels all unfinished siblings when any child completes with failure.

```sifr
async with task.TaskGroup() as group:
    group.spawn(coro: Coroutine[T, E]) -> Task[T, E]
```

`TaskGroup` spawn returns the same `Task[T, E]` observer handle as `scope.spawn`. `TaskGroup` requires all spawned children to share the same error type `E` in v1 (heterogeneous error types are deferred). On group exit with unobserved child failure, the group returns `Result[None, ScopeFailure]`. The group's `ScopeFailure.primary` is `UnobservedChildFailed(first_error, task_id)` where `first_error` is the first sibling to fail after sibling cancellation completes.

Sibling cancellation order: when one child fails, the group immediately requests cancellation on all remaining siblings, then waits for their cleanup before returning the failure. Cleanup failures from cancelled siblings surface as `SecondaryError` values on the primary failure.
```

#### 4. Task handle consumption — explicit rules (model lines 316-317)

**Concern:** "Task composition APIs consume the task handles" is stated for composition APIs but not for plain `await handle`. The review asks for explicit rules: `await Task[T, E]` consumes the handle, `Task` is not cloneable in v1, dropping a handle does not cancel the child.

**Decision: ACCEPT AS BLOCKER. Add explicit consumption semantics section.**

Rationale: The current model says handles are "observers, not owners" but never says whether `await` consumes the handle. For affine types in a systems language, this must be explicit.

**Proposed model edit** (replace model lines 316-317, or add as new section under "Core API Signatures"):

```sifr
### Task Handle Consumption

`Task[T, E]` is an **affine observer handle**:
- Dropping a handle does not cancel or detach the child task.
- Awaiting, `join()`, `gather`, `select`, `race`, and `timeout` consume the handle.
- `cancel()` borrows the handle to request cancellation; the handle may then be awaited/joined to observe cleanup.
- `Task[T, E]` is not cloneable in v1. If shared-observer semantics are needed later, a separate `SharedTask[T, E]` type will be introduced.
- A consumed handle is invalid for further observation. Attempting to await a consumed handle is a compile-time error.
```

Also add to model invariants:

```sifr
25. Task handles are affine. await Task[T, E], join(), gather, select, race, and timeout consume the handle. Task handles are not cloneable in v1.
```

#### 5. `join` and `cancel` signatures missing (model lines 293-314)

**Concern:** Phase file mentions task-handle `join` (milestone 2) and task-handle cancellation (milestone 2). The model file never gives signatures.

**Decision: ACCEPT AS BLOCKER. Add signatures now.**

Rationale: These are in milestone_async_2 scope. The model must define the API before implementation starts.

**Proposed model edit** (add after "Task composition APIs consume the task handles" block):

```sifr
### Task Handle Methods

```sifr
async def Task[T, E].join(self) -> TaskResult[T, E]
def Task[T, E].cancel(self) -> None
async def Task[T, E].cancel_and_join(self) -> TaskResult[T, E]
```

Semantics:
- `join()` consumes the handle and returns the task result once the child completes (success, failure, or cancellation).
- `cancel()` requests cancellation without awaiting cleanup; returns immediately. Safe to call on an already-completed task (no-op). The handle remains awaitable/joinable after `cancel()` to observe cleanup.
- `cancel_and_join()` requests cancellation and awaits cleanup, equivalent to `handle.cancel(); await handle.join()`. Consumes the handle.
- `await Task[T, E]` is syntactic sugar for `await Task[T, E].join()` and consumes the handle.
- `cancel()` may only be called once per handle. Subsequent calls are no-ops.
- After task completion, `cancel()` returns immediately without re-triggering cancellation.
```

#### 6. Union type in `TaskResult[T, E | TimeoutError]` (model line 329)

**Concern:** `task.timeout(handle, duration) -> TaskResult[T, E | TimeoutError]` uses union syntax that may not exist in Sifr.

**Decision: ACCEPT AS FIX (minor). Replace union notation with explicit enum.**

Rationale: The architecture doc confirms Sifr has first-class union types (`int | str` generates Rust enum). But for `TimeoutResult` in error position, a named enum is clearer and avoids "notation vs. type-system feature" ambiguity.

**Proposed model edit** (replace model line 329):

```sifr
task.timeout(handle: Task[T, E], duration: Duration) -> TaskResult[T, TimeoutResult[E]]

enum TimeoutResult[E]:
    Inner(E)
    Timeout(TimeoutError)
```

This avoids smuggled union types and makes the error union explicit. The enum names `Inner` and `Timeout` are self-documenting.

---

### Category B: ACCEPTED IMPROVEMENTS (good fixes, not blockers)

#### 7. Canonical example: definite assignment and undefined `ChildCancelled` (model lines 36-53)

**Concern 7a:** `a` and `b` may not be definitely assigned outside match arms.
**Concern 7b:** `task.ChildCancelled(cancelled.primary)` is not defined in the type-system section.

**Decision: ACCEPT BOTH AS FIXES.**

Rationale: 7a is a legitimate concern — the example should use expression-match syntax to guarantee definite assignment. 7b is also legitimate — undefined API in a flagship example is a red flag. `ChildCancelled` should be defined if used, or replaced with a concrete error wrapping pattern.

**Proposed model edit** (replace lines 27-53):

```sifr
The primary model is:

```sifr
async def fetch_one(label: str) -> Result[str, FetchError]:
    await task.sleep(10.ms)
    return Ok(label)

async def main() -> Result[None, Error]:
    async with task.scope() as scope:
        first = scope.spawn(fetch_one("a"))
        second = scope.spawn(fetch_one("b"))

        a: str = match await first:
            Ok(value):
                value
            Err(failure):
                return Err(failure.primary)
            Cancelled(cancelled):
                return Err(TaskCancelled(f"task cancelled: {cancelled}"))

        b: str = match await second:
            Ok(value):
                value
            Err(failure):
                return Err(failure.primary)
            Cancelled(cancelled):
                return Err(TaskCancelled(f"task cancelled: {cancelled}"))

        print(a + b)
        return Ok(None)

class TaskCancelled(Error):
    message: str
```

`TaskCancelled` is the canonical conversion from materialized child cancellation evidence into an ordinary `Error`. It is not a `CancellationError` and participates in ordinary error handling. Users convert `Cancelled(Failure[CancellationError])` to `TaskCancelled` when they want cancellation to propagate as an ordinary failure through their error channel.
```

Also add to the model types section:

```sifr
class TaskCancelled(Error):
    message: str
```

And to the model invariants:

```sifr
26. `TaskCancelled` is the canonical wrapper for materializing child cancellation into an ordinary Error for propagation.
```

#### 8. Scoped borrowed spawn feasibility (model line 468)

**Concern:** Scoped borrowed spawn is conceptually good but requires a different runtime strategy than plain `tokio::spawn`. Phase file has `spawn_scoped_borrow_ok.sifr` as a positive target. If v1 uses `tokio::spawn` with `'static` requirements, this feature needs explicit deferral or a specific runtime architecture.

**Decision: ACCEPT AS DEFER. Mark scoped borrowed spawn as deferred in v1.**

Rationale: The phase file already has `spawn_scoped_borrow_ok.sifr` as a positive fixture — but this creates a mismatch. If the runtime uses `tokio::spawn`, spawned futures need `'static`. Scoped borrowed async tasks require either a different runtime substrate or scoped spawn that polls children inside the parent scope. This is a major implementation decision that should not be left to codegen to figure out.

**Proposed model edit** (update borrowing table and milestone_async_4 scope):

Under "Borrow rules at async boundaries," change the spawn column for immutable borrow:

```sifr
| immutable borrow | allowed only when the borrow remains valid and no conflicting mutation exists | **deferred in v1; v1 requires owned, sendable, static captures; scoped borrowed spawn requires a runtime substrate that polls child futures inside the parent scope** |
```

And remove `spawn_scoped_borrow_ok.sifr` from the positive validation list in milestone_async_4. Add a note:

```sifr
Note: scoped borrowed spawn is conceptually sound but requires a runtime strategy (scoped nursery polling or equivalent) beyond plain tokio::spawn. Defer to a future milestone after the v1 runtime is proven.
```

#### 9. Milestone dependency: `task.timeout(duration)` in milestone 1 vs 2 (phase lines 213, 280-284)

**Concern:** Milestone 1 lowers `async with task.scope()` as a built-in form, but milestone 2 wants `async with task.timeout(duration)` as a usable context-manager form. That's a dependency bug.

**Decision: ACCEPT AS FIX.**

Rationale: Phase consistency matters. If `task.timeout(duration)` context-manager is part of the v1 surface, it must be available when milestone_async_1 closes, or the phase file must be explicit that the context-manager form is milestone_async_2.

The cleanest fix: add `task.timeout(duration)` as a built-in async-with form in milestone_async_1 scope, alongside `task.scope()`. The context-manager form is part of the basic async-with infrastructure, not a separate API.

**Proposed phase file edit** (milestone_async_1 scope, line 213):

Change from:

```sifr
Parse and lower minimal `async with task.scope() as scope` as a built-in scoped-task construct.
```

To:

```sifr
Parse and lower minimal `async with task.scope() as scope` and `async with task.timeout(duration)` as built-in scoped-task constructs.
```

---

### Category C: ACCEPTED DEFERRALS (not blockers, but noted)

#### 10. Async generator expressions add parser/HIR complexity (model line 391)

**Decision: DEFER AS REVIEWER SUGGESTS.**

The reviewer's ruthless advice is correct: lazy async generator expressions `(expr async for item in source)` are pure ergonomics. Parser/HIR ambiguity with normal generator-expression argument rules is a real complexity. Cut them from v1.

**Proposed model edit** (update "Out Of Scope" section):

```sifr
- async generator expressions as direct function-call arguments (deferred; lazy async generator expressions are also deferred in v1)
```

And update the async comprehension section:

```sifr
lazy: AsyncGenerator[str, IOError, None] = (line async for line in stream_lines(path))
```

Remove the above line from the comprehension examples. List, set, and dict comprehensions remain v1. Lazy async generator expressions (the standalone expression form, not the comprehension-in-expression form) are deferred.

#### 11. `AsyncGenerator[T, E, R]` exposes `R` before users can observe it (model line 255)

**Decision: DEFER AS REVIEWER SUGGESTS.**

`R` is `None` for virtually all generators. The `StopAsyncIteration.value` exposure is explicitly deferred. Paying public type complexity for a feature whose value is not exposed is a bad v1 trade.

**Proposed model edit** (update model line 255):

```sifr
- `AsyncGenerator[T, E]`: user-defined async producer created by an `async def` body that contains `yield`. `T` is the yielded item type, `E` is the ordinary error channel. Non-`None` return values from async generators are rejected in v1; generator return values are internal to cleanup/finalization machinery and not exposed publicly.
```

Update the type-system section to match:

```sifr
`AsyncGenerator[T, E, R]` exists internally, but v1 public async generators use `R = None`. Non-`None` return values are rejected at compile time in v1.
```

#### 12. `aclose()` has no result type (model lines 375-378)

**Decision: ACCEPT AS MINOR FIX (not a full blocker).**

The review's point about explicit close is correct: when `aclose()` is the primary operation (not triggered by cancellation), what does it return if cleanup fails?

**Proposed model edit** (add to async generator section):

```sifr
When `aclose()` is called explicitly (not triggered by cancellation or timeout):
- if cleanup succeeds: returns `Ok(None)`
- if cleanup fails: returns `Err(GeneratorCloseError)` as the primary result

When cleanup fails during cancellation or timeout, the cleanup failure becomes `SecondaryError` evidence attached to the owning cancellation/failure result, not a separate primary result.

```sifr
async def AsyncGenerator[T, E].aclose() -> Result[None, GeneratorCloseError]
```

`GeneratorCloseError` is distinct from ordinary stream failures (`E`) and from cancellation evidence.
```

#### 13. Concurrent `anext()` is underdefined (model lines 381-382)

**Decision: ACCEPT AS MINOR FIX.**

The model says concurrent `anext()` while cleanup is running waits for cleanup and returns final state. But two concurrent `anext()` calls while the generator is active is not defined.

**Proposed model edit** (add to async generator section):

```sifr
`AsyncGenerator` is single-consumer and non-reentrant in v1. Calling `anext()` while a previous `anext()` is still pending (generator suspended at `yield` awaiting the next call) is a protocol error. The runtime returns `Result[None, GeneratorBusyError]` if the generator is already being iterated. Static analysis may reject concurrent `anext()` at compile time where the borrow state is provable.
```

Add `GeneratorBusyError` to the error type table if async generators are in v1.

#### 14. Async iterator close needs a protocol (model lines 546-547)

**Decision: ACCEPT AS MINOR FIX.**

The phrase "has async-generator cleanup semantics" is doing too much work. A named protocol is clearer for implementation.

**Proposed model edit** (add to async iteration section):

```sifr
Async iterators that own resources implement the `AsyncClosable` protocol:

```sifr
protocol AsyncClosable:
    async def aclose(self) -> Result[None, GeneratorCloseError]
```

`async for` and async comprehensions use this protocol to ensure deterministic cleanup when iteration is cancelled or abandoned. `AsyncGenerator` implements `AsyncClosable`. Channel receivers also implement `AsyncClosable` (calling `aclose()` on a receiver closes the channel cleanly).
```

---

### Category D: REJECTED / OVERREACHING (not accepted)

#### 15. Channel close/drop/FIFO rules (model lines 496-506)

**Concern:** The review asks for exact rules on: whether dropping all senders closes the channel, FIFO global vs per-sender, what happens to a value passed to `send(value)` if cancelled before enqueue.

**Decision: REJECT AS OVERREACHING.**

Rationale: The reviewer's proposed rules are sensible, but they represent **implementation details** that belong in the runtime design doc, not the semantic model contract. The model already specifies:
- `ClosedError` from `receive` means closed and drained (no second closed state)
- `sender.close()` wakes pending senders and receivers
- Cancellation while blocked on send/receive propagates without duplicating or losing a message

The "no message duplication or loss" rule already covers the core safety guarantee. Exact drop semantics and FIFO implementation details are runtime concerns. The model should define *behavior*, not *implementation*.

**What I'll add:** A single clarifying rule about cancellation during send:

```sifr
On cancellation during `sender.send(value)`, the value is either not enqueued (dropped) or enqueued exactly once. The value is never duplicated or lost.
```

That's the semantic guarantee. The exact drop-point is implementation.

#### 16. Direct `receive()` vs `async for` disagreement (model line 348)

**Concern:** Direct `receive()` returns `Result[T, ClosedError]` but `anext()` returns `Result[Option[T], E]` where close maps to `Ok(None)`. The review asks to state this translation explicitly.

**Decision: PARTIALLY ACCEPT AS MINOR FIX.**

Rationale: The model already says "Normal exhaustion is `Ok(None)`" for async iteration. The gap is that the translation from `ClosedError` to `Ok(None)` through `ChannelReceiver -> AsyncIterator` is not explicit. This is worth one sentence of clarification, not a full adapter definition.

**Proposed model edit** (add to channel section):

```sifr
`ChannelReceiver` implements `AsyncIterator[T, Never]` by mapping a closed-and-drained `ClosedError` receive to `Ok(None)`. Direct `receive()` exposes `ClosedError` so manual receive loops can distinguish terminal close from item delivery without `Option`. Both APIs are consistent: one surface is for explicit result-handling, the other is for the iteration protocol.
```

#### 17. `SecondaryError` is too narrow (model lines 287-291)

**Concern:** Missing likely variants: `SiblingCancelled`, `TaskPanickedOrRuntimeFault`, `BlockingTaskAbandoned`, `TimeoutCleanupFailed`.

**Decision: REJECT AS OVERREACHING — premature specification.**

Rationale: This is the reviewer's weakest concern. The model already says `SecondaryError` is "inspectable evidence attached to `Failure[E]`." Adding hypothetical variants before we have implementation experience is over-engineering. The three current variants (`CleanupFailed`, `SiblingFailed`, `CancellationDuringCleanup`) cover the core v1 cases. Additional variants can be added when the actual surfaces are implemented.

The only useful addition is acknowledging that `SiblingCancelled` is a plausible future variant:

```sifr
Note: additional `SecondaryError` variants such as `SiblingCancelled` may be added as the model gains implementation experience. The enum is not frozen; new variants follow the same model amendment process as any other type change.
```

That's sufficient.

#### 18. `sifr.threading` in milestone 6 vs "compatibility after canonical" (phase line 542)

**Concern:** `sifr.threading` is placed in milestone 6 (before async resources) but the stated principle is that compatibility comes after the canonical model.

**Decision: REJECT AS OVERREACHING.**

Rationale: `sifr.threading` in this context is **not a Python compatibility veneer** — it's Sifr's own canonical threading API surface (`Thread`, `Lock`, `Event`, `Condition`) built on top of the native Sifr runtime. It's the canonical `sifr.concurrent` sibling to `sifr.sync`. The compatibility-after-canonical principle applies to `sifr.asyncio` and similar Python interop layers. `sifr.threading` is Sifr-native thread coordination, not a compatibility wrapper.

The phase file already distinguishes them:
- milestone 6: `sifr.threading` (native API) — thin compatibility where it can stay canonical
- milestone 8: `sifr.asyncio` (Python compat) — after canonical model is proven

The naming is a bit confusing. A minor clarification in the phase file resolves this:

```sifr
Note: `sifr.threading` is the canonical Sifr thread coordination API (not a Python compatibility layer). Its primitives (`Thread`, `Lock`, `Event`, `Condition`) are Sifr-native and can land in milestone 6 alongside the async blocking-offload model. `sifr.asyncio` (Python compatibility) lands in milestone 8.
```

#### 19. `spawn_blocking` return type (model lines 522-530)

**Concern:** The return type is not specified. Is it `Task[T, E]`, `BlockingTask[T, E]`, or `concurrent.Future[T, E]`?

**Decision: ACCEPT AS MINOR FIX (not a full blocker).**

Rationale: The model already says "cancelling `task.spawn_blocking` or thread-pool work requests cancellation and drops/abandons the handle result" and "already-running blocking work may continue to completion." This already distinguishes blocking cancellation semantics from cooperative task cancellation. A named type clarifies this.

**Proposed model edit** (add to blocking offload section):

```sifr
```sifr
task.spawn_blocking(fn: Fn() -> Result[T, E]) -> BlockingTask[T, E]
```

`BlockingTask[T, E]` is a task handle for blocking work. It differs from `Task[T, E]` in cancellation semantics:
- `cancel()` requests cancellation but does not forcibly abort the OS thread.
- `BlockingTask` is not awaitable through plain `await` (use `join()` explicitly).
- `join()` returns `TaskResult[T, E]` where `TaskResult.Cancelled` means "result abandoned by cancellation," not "OS work stopped."
- `TaskResult.Cancelled` on a `BlockingTask` means the result was discarded; the blocking work may have completed or been interrupted.

```sifr
enum BlockingTaskResult[T, E]:
    Ok(T)
    Err(Failure[E])
    Abandoned  # result discarded after cancellation; OS work may have completed
```
```

Or simply note that `BlockingTask` uses `TaskResult` with the understanding that `Cancelled` has a different meaning for blocking work.

#### 20. Old proposal file should be deleted or renamed (model line 1)

**Decision: ACCEPT AS A CLEANUP TASK.**

**Decision: REJECT AS OUT OF SCOPE for model editing.**

Rationale: This is a file hygiene concern, not a model concern. The question is: does an old proposal file still exist in the repo? Let me check.

The Glob search earlier found no files matching `*proposal*async*`. The older review files are all external review artifacts. There's no "superseded proposal" sitting in `internal_docs/`. The reviewer's concern is about the old design that has already been replaced. Since there's no stale file to rename, this concern is already resolved by prior cleanup.

If the user wants, I can note this as resolved and there's nothing to do.

---

## Summary of Edits

| # | Concern | Decision | Action |
|---|---|---|---|
| 1 | timeout context-manager desugaring | **BLOCKER** | Add explicit desugaring rule (Option A) |
| 2 | ScopeFailure type | **BLOCKER** | Add `ScopeFailure` struct and `ScopeFailureCause` enum, define `__aexit__` return type |
| 3 | TaskGroup API | **BLOCKER** | Define spawn API, error policy, homogeneous error requirement |
| 4 | Task handle consumption | **BLOCKER** | Add affine handle section with consumption rules |
| 5 | join/cancel signatures | **BLOCKER** | Add `join()`, `cancel()`, `cancel_and_join()` signatures and semantics |
| 6 | union type in timeout | **FIX** | Replace `E \| TimeoutError` with `TimeoutResult[E]` enum |
| 7a | definite assignment in example | **FIX** | Use expression-match with type annotation |
| 7b | undefined `ChildCancelled` | **FIX** | Define `TaskCancelled` wrapper type |
| 8 | scoped borrowed spawn | **DEFER** | Mark as deferred in v1; remove from milestone_async_4 positive fixtures |
| 9 | milestone dependency | **FIX** | Add `task.timeout(duration)` to milestone_async_1 built-in forms |
| 10 | async generator expressions | **DEFER** | Remove from v1; deferred to future milestone |
| 11 | AsyncGenerator R exposure | **DEFER** | Reject non-None R at compile time in v1 |
| 12 | aclose() result type | **FIX** | Add `GeneratorCloseError` type and explicit close semantics |
| 13 | concurrent anext() | **FIX** | Define single-consumer non-reentrant policy |
| 14 | AsyncClosable protocol | **FIX** | Define protocol for async iterator cleanup |
| 15 | channel close/drop/FIFO rules | **REJECT** (overreaching) | Add single cancellation guarantee rule only |
| 16 | receive vs async for | **PARTIAL FIX** | Add one sentence about ChannelReceiver implementing AsyncIterator |
| 17 | SecondaryError variants | **REJECT** (overreaching) | Add forward-compatibility note only |
| 18 | sifr.threading placement | **REJECT** (overreaching) | Add clarifying note that sifr.threading is Sifr-native, not compat |
| 19 | spawn_blocking return type | **FIX** | Define `BlockingTask[T, E]` and clarify cancellation semantics |
| 20 | stale proposal file | **RESOLVED** | No stale file exists in internal_docs/ |

**Blocked count: 5** (concerns 1-5)
**Fix count: 12** (concerns 6, 7a, 7b, 9, 12, 13, 14, 16 partial, 19 + minor edits to 11, 12)
**Defer count: 3** (concerns 8, 10, 11)
**Reject count: 4** (concerns 15, 17, 18, 20)

---

## Proposed Model Edits (consolidated)

Here are the exact edits to `internal_docs/async_concurrency_model.md`:

**Edit 1 — Scope Failure (after line 291):**

```sifr
### Scope Failure

When a `TaskScope` exits and has unobserved children that failed, the scope exit produces a typed scope failure:

```sifr
struct ScopeFailure:
    primary: ScopeFailureCause
    secondary: list[SecondaryError]

enum ScopeFailureCause:
    UnobservedChildFailed(error: Error, task_id: str)
    UnobservedChildCancelled(cause: CancellationError, task_id: str)
```

`TaskScope.__aexit__` returns `Result[None, ScopeFailure]`. Explicit observation means `await handle`, `gather`, `select`, `race`, `timeout`, or `join` marks the child as observed.
```

**Edit 2 — Task Handle Consumption (after line 317):**

```sifr
### Task Handle Consumption

`Task[T, E]` is an **affine observer handle**:
- Dropping a handle does not cancel or detach the child task.
- Awaiting, `join()`, `gather`, `select`, `race`, and `timeout` consume the handle.
- `cancel()` borrows the handle to request cancellation; the handle may then be awaited/joined to observe cleanup.
- `Task[T, E]` is not cloneable in v1.
```

**Edit 3 — join/cancel signatures (after Task Handle Consumption):**

```sifr
```sifr
async def Task[T, E].join(self) -> TaskResult[T, E]
def Task[T, E].cancel(self) -> None
async def Task[T, E].cancel_and_join(self) -> TaskResult[T, E]
```

`await Task[T, E]` is syntactic sugar for `await Task[T, E].join()` and consumes the handle.
```

**Edit 4 — Timeout enum (replace line 329):**

```sifr
task.timeout(handle: Task[T, E], duration: Duration) -> TaskResult[T, TimeoutResult[E]]

enum TimeoutResult[E]:
    Inner(E)
    Timeout(TimeoutError)
```

**Edit 5 — Canonical example (replace lines 27-53):** Use expression-match with type annotations and define `TaskCancelled` (see full replacement above).

**Edit 6 — Scope borrow rule (update line 468):**

```sifr
| immutable borrow | ... | **deferred in v1; v1 requires owned, sendable, static captures** |
```

**Edit 7 — AsyncGenerator R (update type-system line 255):**

```sifr
`AsyncGenerator[T, E]`: user-defined async producer created by an `async def` body that contains `yield`. Non-`None` return values are rejected at compile time in v1.
```

**Edit 8 — aclose() result type (add after line 378):**

```sifr
When `aclose()` is called explicitly (not triggered by cancellation/timeout):
- if cleanup succeeds: returns `Ok(None)`
- if cleanup fails: returns `Err(GeneratorCloseError)` as the primary result

When cleanup fails during cancellation or timeout, the cleanup failure becomes `SecondaryError` evidence attached to the owning result.

```sifr
async def AsyncGenerator[T, E].aclose() -> Result[None, GeneratorCloseError]
```
```

**Edit 9 — AsyncGenerator reentrancy (add to generator section):**

```sifr
`AsyncGenerator` is single-consumer and non-reentrant in v1. Calling `anext()` while a previous `anext()` is still pending is a protocol error.
```

**Edit 10 — AsyncClosable protocol (add to async iteration section):**

```sifr
```sifr
protocol AsyncClosable:
    async def aclose(self) -> Result[None, GeneratorCloseError]
```
```

**Edit 11 — ChannelReceiver AsyncIterator (add to channel section):**

```sifr
`ChannelReceiver` implements `AsyncIterator[T, Never]` by mapping closed-and-drained `ClosedError` receive to `Ok(None)`. Direct `receive()` exposes `ClosedError` for manual loops; `async for` uses the iterator protocol.
```

**Edit 12 — BlockingTask (add to blocking offload section):**

```sifr
```sifr
task.spawn_blocking(fn: Fn() -> Result[T, E]) -> BlockingTask[T, E]
```

`BlockingTask[T, E]` differs from `Task[T, E]` in cancellation semantics: `cancel()` requests cancellation without forcibly aborting the OS thread, and `TaskResult.Cancelled` means "result abandoned by cancellation."
```

---

## Architecture Doc Edits (phases/32_async_ecosystem.md)

**Edit A — milestone_async_1 scope (line 213):** Add `task.timeout(duration)` to built-in forms.

**Edit B — milestone_async_4 positive fixtures:** Remove `spawn_scoped_borrow_ok.sifr`. Add note about scoped borrowed spawn deferral.

**Edit C — milestone_async_0 public types:** Add `ScopeFailure`, `ScopeFailureCause`, `TimeoutResult`, `BlockingTask`, `TaskCancelled`, `GeneratorCloseError`, `AsyncClosable` to the initial public types list.

---

## Final Verdict

The review is **substantially correct** on its blockers. The 5 accepted blockers (timeout desugaring, ScopeFailure, TaskGroup API, handle consumption, join/cancel signatures) are genuine contract holes that would force implementers to invent behavior. Fix them.

The review is **overreaching** on concerns 15, 17, and 18:
- Channel drop/FIFO implementation details are runtime concerns, not semantic contract (15)
- Adding hypothetical SecondaryError variants before implementation is over-engineering (17)
- `sifr.threading` as Sifr-native thread coordination is correctly placed in milestone 6; the "compatibility after canonical" principle applies to Python interop, not native APIs (18)

The review is **correct on deferrals**: async generator expressions (10), public `AsyncGenerator` R (11), and scoped borrowed spawn (8) should all be deferred. Async generator expressions especially are pure ergonomics that add parser/HIR complexity without being essential to the first model.

**The model is ready for implementation once the 5 blockers are addressed.** The reviewer's final verdict is correct: "fix those, and it becomes genuinely bulletproof."
