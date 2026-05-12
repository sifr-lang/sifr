Now it’s **almost there**.

I’d call this version **implementation-ready after a short cleanup pass**, not a redesign pass. The model now has a coherent semantic core: `Coroutine[T, E]`, affine `Task[T, E]`, `TaskResult[T, E]`, `ScopeFailure`, `TimeoutResult[E]`, parameterized `AsyncClosable[E]`, same-task timeout scopes, v1 owned/static spawn boundaries, explicit channels, and a milestone plan that mostly lines up with the contract. That is a huge improvement.

But I would still block implementation on a few precise contract holes.

# Verdict

**Approve the architecture. Do not start coding until the remaining redlines below are fixed.**

The proposal is no longer conceptually confused. It is now mostly a **control-flow and API-completeness problem**.

The big wins:

* `TaskResult` is now distinct from ordinary `Result`.
* `CancellationError` is not an `Error`.
* `try await task_handle` is rejected.
* `TaskScope.__aexit__ -> Result[None, ScopeFailure]` exists.
* timeout over task handles uses `TimeoutResult[E]`, avoiding hidden union types.
* timeout context blocks are same-task cancellation scopes, not child-task hacks.
* v1 `scope.spawn` requires owned/sendable/static captures, deferring scoped borrowed spawn.
* async generators are `AsyncGenerator[T, E]`, not coroutine-returning generators.
* channel endpoint lifetime rules are finally explicit.
* phase milestones now mostly respect dependency order.

That is strong.

# Remaining redlines

## 1. `select` / `race` still have a secondary-error hole

This is the biggest unresolved semantic issue.

You say `select` and `race` cancel losers by default, consume input handles, and return the winner result. Good. But what happens if a loser’s cancellation cleanup fails after the winner succeeds?

Example:

```sifr
winner = task.race([fast_success, slow_cleanup_fails])
```

If `fast_success` returns `Ok(value)`, then `race` wants to return:

```sifr
TaskResult.Ok(value)
```

But `TaskResult.Ok(T)` has no place for `SecondaryError`.

So where does the loser cleanup failure go?

Right now, there is no clean answer. It cannot be silently dropped, because the model repeatedly says cleanup failures become structured evidence. But there is no primary failure to attach it to.

You need one explicit rule. I would use this:

```text
For select/race, loser cleanup failures attach as secondary evidence if the selected winner result is Err(...) or Cancelled(...). If the selected winner result is Ok(...), loser cleanup failures are surfaced at the owning TaskScope exit as ScopeFailure rather than being dropped.
```

Or change `TaskResult.Ok(T)` to carry secondary evidence:

```sifr
Ok(T, secondary: list[SecondaryError])
```

I would not do that unless you want to complicate the whole model. The `ScopeFailure` route is simpler.

This same rule should cover same-tick cases where one ready task wins by input order but another ready loser has already failed.

## 2. `gather` does not define child cancellation

`gather` defines success and ordinary child error behavior. It does not clearly define what happens when a child is cancelled.

You need this case:

```sifr
task.gather([a, b, c])
```

where `b` completes as:

```sifr
TaskResult.Cancelled(Failure[CancellationError])
```

Before any ordinary `Err(E)`.

Does `gather`:

* return `Cancelled(...)`?
* cancel unfinished siblings?
* treat cancellation like ordinary failure?
* preserve input-order primary selection if cancellation and ordinary failure both happen?

My recommendation:

```text
If any gathered child is observed as Cancelled before an ordinary child error is selected as primary, gather cancels unfinished siblings and returns TaskResult.Cancelled(Failure[CancellationError]). If ordinary errors and cancellations are observed during the same drain, deterministic input order chooses the primary among failure-like outcomes; the rest become SecondaryError evidence.
```

But you need to define the exact priority. Without it, implementers will guess.

## 3. Secondary evidence has no clear path for same-task coroutine errors

The model says cleanup failures during ordinary error propagation become secondary evidence “at the task/scope observation boundary.” That works when the current async computation is a spawned task, because `await Task[T, E]` returns `Failure[E]`.

But same-task coroutine await returns ordinary `Result[T, E]`:

```sifr
await Coroutine[T, E] -> Result[T, E]
```

There is no `Failure[E]` wrapper there.

So if this happens:

```sifr
async def inner() -> Result[None, MyError]:
    async with resource_that_fails_on_exit():
        return Err(MyError())
```

and someone does:

```sifr
result = await inner()
```

where does the context-exit failure go?

You need a task-local secondary-evidence rule:

```text
Secondary evidence produced inside a same-task coroutine is accumulated on the currently running task. Same-task `await Coroutine[T, E]` returns only `Result[T, E]`; accumulated secondary evidence becomes observable only when the current task is later observed through `TaskResult`, or through diagnostics/logging if the top-level task exits.
```

Or you need to change same-task coroutine await to return `Failure[E]`, which I do **not** recommend.

The hidden accumulator rule is fine. It just needs to be explicit.

## 4. `AsyncClosable[E]` creates a close-error typing obligation

You fixed `AsyncClosable[E]`. Good.

But now `async for` has two possible error channels:

```sifr
AsyncIterator[T, IterE]
AsyncClosable[CloseE]
```

The desugaring handles `IterE`, but early exit cleanup can fail with `CloseE`.

Example:

```sifr
async for item in stream:
    if done(item):
        return Ok(value)
```

If `stream.aclose()` can fail, the enclosing function must be able to carry `CloseE`, or the close error must be handled locally.

The current text says close failure is primary on normal `break`/`return`, which is right, but it should also say:

```text
If an early-exit path from `async for` may call `aclose()`, the enclosing function must be able to propagate the iterator's close error type, or the close error must be handled explicitly.
```

Otherwise users and implementers will not know how to type-check `break`, `return`, and `Err` propagation in closable async loops.

## 5. User-defined async context manager protocol signatures are still missing

The cleanup rules are now good, and the fallible exit table is useful. But the actual protocol is not specified.

You need something like:

```sifr
protocol AsyncContextManager[T, EnterE, ExitE]:
    async def __aenter__(self) -> Result[T, EnterE]
    async def __aexit__(self, cause: AsyncExitCause) -> Result[None, ExitE]
```

Then define:

```sifr
enum AsyncExitCause:
    Normal
    Return
    OrdinaryError(Error)
    Timeout(TimeoutError)
    Cancellation(CancellationError)
    RuntimeFault(...)
```

The exact names can differ. But without method signatures, milestone 7a still has to invent the user-defined protocol.

Also define whether `__aexit__` runs if `__aenter__` fails. Usually it should not, because the resource was not acquired.

## 6. `TaskGroup` still needs closed/failed-state spawn rules

You now define sibling cancellation and internal observation. Good.

But what happens here?

```sifr
async with task.TaskGroup[MyError]() as group:
    a = group.spawn(fails())
    await a
    group.spawn(new_work())  # allowed?
```

If the group has already entered failure/cancelling state, I think this must be invalid.

Add a state rule:

```text
A TaskGroup has Open, Cancelling, Closing, and Closed states. `group.spawn(...)` is valid only in Open. After first child failure, explicit group cancellation, timeout, or scope exit begins, new spawn attempts are rejected statically when possible and otherwise fail with a typed ScopeClosedError/GroupClosedError diagnostic path.
```

Same principle for `TaskScope`: once `__aexit__` begins, spawning is invalid.

## 7. Core public type inventory is incomplete in the phase file

The model uses these public-ish types:

```sifr
ChannelSender[T]
ChannelReceiver[T]
ClosedError
Shared[T]
ShareSafe
RwLock[T]
LockGuard[T]
RwLockGuard[T]
Semaphore
Notify
Select2
ThreadPoolExecutor
```

But the milestone 0 public type list only includes some of them, like `Channel[T]` and `Lock[T]`. The phase file says all public modules/types for v1 must be named and scoped, so the list should be exhaustive.

Add every public type. If some are private implementation details, say so.

## 8. Sync primitive signatures are still too vague

Channels now have signatures. Tasks now have signatures. Generators now have signatures.

Locks, RwLocks, Semaphores, and Notify still do not.

Before milestone 5, add:

```sifr
sync.Shared[T](value: T) -> Shared[T]

sync.Lock[T](value: T) -> Lock[T]
def Lock[T].lock(self) -> LockGuard[T]
def Lock[T].try_lock(self) -> Result[LockGuard[T], WouldBlockError]

sync.RwLock[T](value: T) -> RwLock[T]
def RwLock[T].read(self) -> RwLockReadGuard[T]
def RwLock[T].write(self) -> RwLockWriteGuard[T]

sync.Semaphore(permits: int) -> Semaphore
async def Semaphore.acquire(self) -> Result[SemaphorePermit, ClosedError]
def Semaphore.try_acquire(self) -> Result[SemaphorePermit, WouldBlockError]

sync.Notify() -> Notify
async def Notify.notified(self) -> None
def Notify.notify_one(self) -> None
def Notify.notify_all(self) -> None
```

The exact API may differ. But the contract should not say “implement Semaphore and Notify” without method shapes.

## 9. `select` is advertised variadic but specified binary

The design text says:

```sifr
task.select(*handles)
```

The signature only defines:

```sifr
task.select[A, EA, B, EB](a: Task[A, EA], b: Task[B, EB]) -> Select2[...]
```

Pick one.

Options:

```text
V1 select is binary only; users compose nested select for more tasks.
```

or:

```text
V1 select supports fixed arity overloads Select2 through SelectN.
```

or:

```text
V1 select is homogeneous-list only, like race, and heterogeneous select is deferred.
```

Right now the API promise and type signature disagree.

## 10. `BlockingTask` lifecycle is not structured enough

You made `BlockingTask[T, E]` distinct from `Task[T, E]`. Correct.

But now define its lifecycle as rigorously as `Task`.

Questions still open:

```text
Is BlockingTask affine?
Does awaiting/joining consume it?
Is it owned by the current TaskScope?
What happens if a BlockingTask handle is dropped?
What happens at scope exit if blocking work is still running?
Does scope exit wait, abandon, or merely request cancellation?
Can BlockingTask escape a scope?
```

Because blocking work may continue after cancellation, you cannot pretend it is ordinary structured child work. That is fine because captures are owned/static. But the rule must be explicit.

I would define:

```text
BlockingTask handles are affine. join/cancel_and_join consume them. Dropping a BlockingTask handle abandons observation but does not stop already-running OS work. Blocking work requires owned/sendable/static captures precisely because it may outlive the async scope after abandonment. Scope exit requests cancellation/abandonment for unresolved blocking work created inside the scope but does not guarantee OS-thread interruption.
```

Then test it.

## 11. `BlockingTask.join() -> TaskResult[T, E]` is still semantically sharp

The current doc says `Cancelled` on a `BlockingTask` means “observer abandoned the result,” not “the OS work stopped.” That is documented. Good.

I still dislike reusing `TaskResult.Cancelled` here. It will confuse users.

This is not a blocker if you add loud diagnostics/docs, but the cleaner shape is:

```sifr
enum BlockingTaskResult[T, E]:
    Ok(T)
    Err(Failure[E])
    Abandoned(Failure[CancellationError])
```

If you keep `TaskResult`, every mention of `BlockingTask.cancel()` should repeat:

```text
Cancelled does not mean work stopped.
```

## 12. Validation names still imply the wrong thing in a few places

This one matters because tests encode semantics.

`task_group_unhandled_error_rejected.sifr` sounds like a compile-time rejection, but the model says unobserved child failure surfaces as `ScopeFailure` at scope exit. That may be a runtime/task result behavior, not necessarily a static rejection.

Use clearer names:

```text
task_group_unobserved_failure_scope_failure.sifr
task_group_heterogeneous_error_rejected.sifr
task_group_error_type_not_carried_rejected.sifr
```

Likewise, add tests for the newly fixed channel endpoint rules:

```text
channel_drop_last_sender_closes_after_drain.sifr
channel_drop_receiver_closes_senders.sifr
channel_sender_close_clone_closes_all.sifr
channel_fifo_order.sifr
channel_cancel_receive_no_loss.sifr
```

## 13. Receive cancellation needs the same exactly-once rule as send

You explicitly define send cancellation:

```text
value is either not enqueued and dropped, or enqueued exactly once
```

Good.

For receive, add the mirror rule:

```text
If a receive is cancelled before `Ok(value)` is returned to user code, the message remains available to another receive or is otherwise not lost. Once `Ok(value)` has been returned, ownership has transferred to the receiver task.
```

Without this, “no duplicating or losing a message” is a promise without an operational rule.

## 14. Markdown hygiene: one code block likely swallows prose

In the type-system section, the prose line about `TimeoutResult[E] implements Error when E: Error` appears inside the `sifr` code block. That is small, but fix it. Specs that mix prose into code blocks create copy/paste ambiguity.

# My implementation-readiness call

I would mark this as:

```text
Architecture: approved.
Implementation: blocked only on final redline cleanup.
```

This is no longer a “rewrite the model” situation. The remaining work is contract sharpening.

The must-fix set before coding:

1. Define `select`/`race` loser cleanup failures, especially winner-`Ok` cases.
2. Define `gather` child-cancellation behavior.
3. Define how secondary evidence accumulates for same-task coroutine errors.
4. Add exact user-defined async context-manager protocol signatures.
5. Add `TaskGroup` closed/cancelling spawn rules.
6. Complete public type inventory in milestone 0.
7. Add method signatures for `Shared`, `Lock`, `RwLock`, `Semaphore`, and `Notify`.
8. Define `BlockingTask` lifecycle and scope behavior.
9. Clarify `AsyncClosable[E]` close-error obligations in `async for`.
10. Add receive-cancellation exactly-once semantics.

Once those are patched, I would stop debating the architecture and start implementing milestone 0/1. The core design is now solid.
