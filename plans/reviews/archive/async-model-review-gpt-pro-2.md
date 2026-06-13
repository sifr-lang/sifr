This is **much better**. You fixed the original fatal problem.

The old version blurred async functions, coroutine values, task handles, `Result`, and cancellation. The new version separates them cleanly: calling an async function returns a linear `Coroutine[T, E]`; `scope.spawn` consumes that coroutine and returns `Task[T, E]`; awaiting a task gives `TaskResult[T, E]`; and `try await task_handle` is rejected because cancellation is not an ordinary error branch. That is the right move.

My current verdict:

**This is now a serious contract. It is close to implementation-ready. But it is not bulletproof yet.**

The remaining problems are not philosophical anymore. They are mostly **state-machine, type-shape, and API-signature holes**. Those are dangerous because they will force implementers to invent behavior during codegen.

---

# What improved dramatically

You fixed the biggest structural defects.

The `Coroutine` / `Task` / `TaskResult` split is now strong. `CancellationError` no longer sits inside an ordinary `Result` error union; it appears through `TaskResult.Cancelled(Failure[CancellationError])`, while ordinary task failure appears through `TaskResult.Err(Failure[E])`. That is much cleaner and much harder for users to misuse.

You also fixed the task-ownership model. `TaskScope` now uses nursery ownership: the scope owns children, handles are observers, dropping a handle does not detach the task, normal scope exit waits for children, abnormal exit cancels unfinished children, and handles cannot silently escape the owning scope. That is a coherent structured-concurrency model.

Channels are much better now too. Explicit sender/receiver endpoints, async `send`, async `receive`, single receiver in v1, no duplicate `None`/`ClosedError` end state — all of that is a major improvement over the previous channel shape.

The compatibility section is also no longer lying. `Future` is now a compatibility wrapper rather than a pure alias, `Event` is not blindly equated with `Notify`, and `asyncio.wait_for` is explicitly limited to task handles in v1. That is the right level of honesty.

The phase file is also much cleaner. It correctly says the model file is the semantic source of truth, and the phase file is only the implementation plan. That prevents implementation drift.

---

# The remaining blockers

## 1. `task.timeout(duration)` context-manager semantics are still not type-honest

You fixed `task.timeout(handle, duration)`. That one is now mostly clear:

```sifr
task.timeout(handle: Task[T, E], duration: Duration) -> TaskResult[T, E | TimeoutError]
```

It accepts task handles, not arbitrary awaitables; timeout wins by cancelling the child, awaiting cleanup, and returning `TaskResult.Err(Failure[TimeoutError])`. Good.

But this is still underdefined:

```sifr
async with task.timeout(5.s):
    await slow()
```

The contract says this form “uses the same completion-vs-deadline policy through structured scope cancellation,” and the phase file says it is usable as `async with task.timeout(duration):`.

That is not enough.

An `async with` statement does not naturally return a `TaskResult`. So where does the `TimeoutError` go?

You need to define the exact desugaring.

For example, what is the type and control flow of this?

```sifr
async def f() -> Result[None, Error]:
    async with task.timeout(1.s):
        await slow()

    print("after")
    return Ok(None)
```

If the deadline fires:

Does execution continue after the block?

```sifr
print("after")
```

Does the block produce an ordinary `Err(TimeoutError)`?

Can `except Error` catch it?

Does timeout cancel the current task or an internal child task?

If it cancels the current task, how does the current task continue afterward?

If it does not cancel the current task, what exactly is being cancelled?

This is still one of the few places where cancellation suppression sneaks back in through the side door.

You need a precise rule, probably one of these:

```sifr
# Option A: timeout context converts deadline into ordinary Result failure
async with task.timeout(duration):
    body
# desugars to a compiler-recognized cancellation scope that returns Err(TimeoutError)
# from the surrounding fallible async function if the deadline wins
```

or:

```sifr
# Option B: timeout context is only valid in a `try` context
try async with task.timeout(duration):
    body
```

or:

```sifr
# Option C: kill the context-manager form in v1
handle = scope.spawn(body_as_coroutine())
result = await task.timeout(handle, duration)
```

Right now the handle form is solid. The inline block form is still hand-wavy.

---

## 2. `TaskScope` unobserved child failure has no concrete type

This line is good but incomplete:

> child failures that are not explicitly observed are surfaced at scope exit as structured scope failure evidence, never silently discarded.

Good instinct. But surfaced **how**?

Suppose:

```sifr
async def main() -> Result[None, Error]:
    async with task.scope() as scope:
        scope.spawn(fails_with_network_error())
        scope.spawn(fails_with_parse_error())

    return Ok(None)
```

At scope exit, what is the type of the failure?

Is it:

```sifr
ScopeFailure
```

as an ordinary `Error`?

Is it:

```sifr
Failure[Error]
```

with type-erased child errors?

Is it:

```sifr
Failure[NetworkError | ParseError]
```

which requires union types?

Does the compiler require all fallible child tasks to be explicitly observed?

Does plain `TaskScope` only allow unobserved `Task[T, Never]` children?

Right now the document says unobserved failures are surfaced, but it does not define the API or type that surfaces them.

This is a real blocker. It affects `async with task.scope()` lowering, function return-type checking, scope-exit behavior, and diagnostics.

My recommendation:

```sifr
struct ScopeFailure:
    primary: ScopeFailureCause
    secondary: list[SecondaryError]
```

with something like:

```sifr
enum ScopeFailureCause:
    UnobservedChildFailed(error: Error, task_id: str)
    UnobservedChildCancelled(cause: CancellationError, task_id: str)
```

Then define:

```sifr
TaskScope.__aexit__ -> Result[None, ScopeFailure]
```

or define it as compiler-recognized scope-exit failure evidence.

Without this, implementers will make incompatible choices.

---

## 3. `TaskGroup` is still mostly a name, not an API

The document says `TaskGroup` adds sibling-failure policy on top of task scopes. Good.

But I still do not know the API.

Does it look like this?

```sifr
async with task.TaskGroup() as group:
    a = group.spawn(foo())
    b = group.spawn(bar())
```

Does `group.spawn` return the same `Task[T, E]` observer handle?

Does the group cancel siblings as soon as any child fails, even if nobody awaited that child yet?

Does group exit return `TaskResult`? `ScopeFailure`? `Failure[E]`? `Failure[Error]`?

Can a `TaskGroup` contain heterogeneous child error types?

If yes, how is the group error typed?

If no, say so.

Right now `TaskGroup` is in the locked v1 surface, but its shape is less precise than `gather`, `select`, and `timeout`. That is backwards. If `TaskGroup` is first-class, it needs first-class signatures.

---

## 4. `Task` handle consumption is not fully specified

Composition APIs consume handles. That part is clear:

```sifr
task.select(a, b)
# a and b are no longer usable
```

Good.

But what about plain `await handle`?

This matters because `T` may be move-only or non-cloneable.

If this is allowed:

```sifr
r1 = await handle
r2 = await handle
```

then the runtime must store and clone/move the task result twice. That is not generally possible.

So the rule should probably be:

```sifr
await Task[T, E] consumes the task handle.
```

Then say whether this is valid:

```sifr
handle.cancel()
result = await handle
```

Probably yes.

And say whether `Task` handles are cloneable. I would make them **not cloneable in v1** unless you introduce a separate shared-observer handle.

Recommended rule:

```sifr
Task handles are affine observer handles.
Dropping a handle does not cancel or detach the child.
Awaiting, joining, gather/select/race, or timeout consumes the handle.
cancel() borrows the handle and requests cancellation; the handle may then be awaited/joined to observe cleanup.
Task handles are not clonable in v1.
```

This needs to be explicit.

---

## 5. `join` and `cancel` are mentioned but not typed

The phase file says milestone 2 implements task-handle `join` and task-handle cancellation.

But the model file does not give signatures.

You need these:

```sifr
async def Task[T, E].join(self) -> TaskResult[T, E]
def Task[T, E].cancel(self) -> None
async def Task[T, E].cancel_and_join(self) -> TaskResult[T, E]
```

Or whatever naming you want.

But the semantics need to be locked:

* Does `join()` consume the handle?
* Is `await handle` exactly sugar for `await handle.join()`?
* Does `cancel()` return immediately?
* Does `cancel()` wait for cleanup?
* Can you call `cancel()` after task completion?
* Can you call `cancel()` after handle consumption?
* Can `cancel()` produce `Cancelled(Failure[CancellationError])` if the task was already done?

This is not optional. Users will hit this immediately.

---

## 6. `Task[T, E | TimeoutError]` still smuggles in union types

You fixed the cancellation union problem by adding `TaskResult`.

But timeout still says:

```sifr
task.timeout(handle: Task[T, E], duration: Duration) -> TaskResult[T, E | TimeoutError]
```

That uses `E | TimeoutError`.

Does Sifr have union types?

If yes, the type system section should explicitly say so.

If no, this is another hidden dependency.

Alternative:

```sifr
enum TimeoutFailure[E]:
    Inner(E)
    Timeout(TimeoutError)

task.timeout(handle: Task[T, E], duration: Duration)
    -> TaskResult[T, TimeoutFailure[E]]
```

That is more verbose but avoids inventing union typing inside async.

If unions already exist in Sifr, fine. But the contract should say:

```sifr
E | TimeoutError is a closed sum type and participates in ordinary Error matching as ...
```

Right now it is notation, not a specified type-system feature.

---

## 7. The canonical example still has two hidden issues

The example is much better than before, but this part is risky:

```sifr
match await first:
    Ok(a):
        pass
    Err(failure):
        return Err(failure.primary)
    Cancelled(cancelled):
        return Err(task.ChildCancelled(cancelled.primary))

# later
print(a + b)
```

Unless Sifr deliberately has Python-style pattern bindings that leak out of `match` arms, `a` may not be definitely assigned outside the `match`.

Even if Sifr does allow that, it is a bad flagship example because it makes definite assignment subtle.

Use expression-match instead:

```sifr
a: str = match await first:
    Ok(value):
        value
    Err(failure):
        return Err(failure.primary)
    Cancelled(cancelled):
        return Err(task.ChildCancelled(cancelled.primary))
```

Same for `b`.

Second issue: the example uses this:

```sifr
task.ChildCancelled(cancelled.primary)
```

But `ChildCancelled` is not listed among the public types in the type-system section. If it is the blessed conversion from materialized cancellation evidence into an ordinary `Error`, define it. If it is just illustrative, remove it from the canonical example.

The example should not use undefined API.

---

## 8. Scoped borrows across spawned tasks are still a major implementation risk

The model allows immutable borrows across `scope.spawn` when the scoped lifetime proves safety and the referent is share-safe.

The phase file even has:

```text
spawn_scoped_borrow_ok.sifr
```

as a positive validation target.

This is conceptually good, but implementation-heavy.

If the backend uses ordinary `tokio::spawn`, spawned futures generally need to be owned and `'static`. Scoped borrowed async tasks require a different runtime strategy: a real scoped nursery that polls child futures inside the parent scope, or a runtime substrate that supports scoped async tasks.

So milestone 0 should not merely say “implementation may use Tokio.” It should answer:

```text
Can v1 scope.spawn support non-'static borrowed captures?
```

If yes, say what runtime/codegen architecture enables it.

If no, change the v1 rule to:

```sifr
scope.spawn requires owned captures in v1.
Scoped borrowed spawn is deferred.
```

Do not leave this as “the borrow checker will figure it out.” It will not if the runtime substrate requires `'static`.

---

## 9. `async with task.timeout(duration)` appears in milestone 2, but milestone 1 only lowers `async with task.scope()`

The phase file says milestone 1 parses and lowers minimal:

```sifr
async with task.scope() as scope
```

as a built-in scoped-task construct, while general user-defined async context managers wait until milestone 7a.

But milestone 2 already wants this:

```sifr
async with task.timeout(duration):
```

as a usable context-manager form.

That is a dependency bug.

Fix it one of three ways:

```text
Option A:
milestone_async_1 supports built-in async with forms for both task.scope() and task.timeout(duration).

Option B:
task.timeout(duration) context-manager form moves to milestone_async_7a.

Option C:
drop timeout context-manager form from v1.
```

The simplest fix is Option A.

---

## 10. Async generators are now in v1, and that makes the phase much bigger

You added async generators and async comprehensions to the first model. The spec is thoughtful: async generator functions are not coroutines, `AsyncGenerator[T, E, R]` implements `AsyncIterator[T, E]`, direct `await` is rejected, close/cancellation runs cleanup, and unsupported Python controls like `send`, `throw`, and async `yield from` are deferred.

This is coherent.

But it is a lot.

Async generator lowering without unstable Rust generator features is a nontrivial compiler project. The phase file correctly calls that out, but it still makes Phase 32 much larger.

My ruthless advice: either accept that Phase 32 is now a major compiler phase, or cut async generator expressions from v1.

I would keep:

```sifr
async def ... yield ...
async for
anext()
```

But consider deferring:

```sifr
(expr async for item in source)
```

Lazy async generator expressions are pure ergonomics. They add parser/HIR/lifetime complexity without being essential to the first model.

---

## 11. `AsyncGenerator[T, E, R]` exposes `R` before users can observe it

The model says `R` is the generator return value available to internal cleanup/finalization machinery, while v1 does not expose Python-style `StopAsyncIteration.value` publicly.

Then why is `R` public?

If users cannot observe it, it should probably not be part of the v1 public type.

This is simpler:

```sifr
AsyncGenerator[T, E]
```

Then defer explicit generator return values entirely.

Or lock this rule:

```sifr
AsyncGenerator[T, E, R] exists internally, but public v1 async generators require R = None.
Non-None return values from async generators are rejected in v1.
```

Right now you are paying complexity for a feature whose value is explicitly not exposed.

That is not a good v1 trade.

---

## 12. `aclose()` has no result type

The model says:

```sifr
agen.aclose()
```

requests generator close, runs cleanup, and then completes. It also says cleanup failures become `SecondaryError` evidence attached to the owning cancellation/failure result.

But if the user explicitly calls `aclose()`, what is the owning primary result?

Example:

```sifr
agen = stream_lines(path)
await anext(agen)
result = await agen.aclose()
```

If cleanup fails, what does `aclose()` return?

You need a signature:

```sifr
async def AsyncGenerator[T, E, R].aclose() -> Result[None, GeneratorCloseError]
```

or:

```sifr
async def AsyncGenerator[T, E, R].aclose() -> TaskResult[None, CloseError]
```

Probably not `TaskResult`, because no task handle is being observed.

Maybe:

```sifr
async def aclose() -> Result[None, Failure[GeneratorCloseError]]
```

But decide.

The “secondary evidence” story works when there is already a primary cancellation/failure. It does not work when explicit close is the primary operation.

Rule needed:

```text
If cleanup fails during normal explicit close, the cleanup failure is primary.
If cleanup fails during cancellation or timeout, it is secondary.
```

---

## 13. Concurrent `anext()` is underdefined

You define what happens if `anext()` is called while cleanup is running: it waits for cleanup and then returns final state.

But what about two concurrent `anext()` calls while the generator is active?

```sifr
a = scope.spawn(anext(agen))
b = scope.spawn(anext(agen))
```

Is that illegal?

Does the second wait?

Does it return `GeneratorBusyError`?

Does it consume the next item after the first call completes?

Most generator models should be non-reentrant by default. I would define:

```sifr
AsyncGenerator is single-consumer and non-reentrant in v1.
Calling anext() while a previous anext() is still active is a typed runtime protocol error.
```

Or reject it statically where possible.

Do not silently queue concurrent `anext()` calls unless you want generators to become channels.

---

## 14. Async iterator close needs a protocol

The model says eager async comprehensions close the active iterator on cancellation or abandonment “when that iterator has async-generator cleanup semantics.”

That phrase is doing too much work.

The compiler/runtime needs a trait/protocol:

```sifr
protocol AsyncClosable:
    async def aclose(self) -> Result[None, CloseError]
```

or:

```sifr
protocol AsyncClosableIterator[T, E]:
    async def anext(self) -> Result[Option[T], E]
    async def aclose(self) -> Result[None, CloseError]
```

Then `async for` and async comprehensions can have deterministic cleanup rules.

Otherwise “has async-generator cleanup semantics” is a vibes-based runtime check.

---

## 15. Channel close/drop semantics still need sharper rules

Channel endpoints are much better now, but you still need to define endpoint lifetime behavior.

Current rules say sender is clonable, receiver is single-consumer, `sender.close()` wakes pending senders and receivers, and `receive()` returns `ClosedError` when closed and drained.

Missing rules:

```sifr
# Does dropping all senders close the channel?
# Does dropping the receiver close the channel to senders?
# Does sender.close() from one clone close the whole channel?
# Can a different cloned sender still send after one sender calls close()?
# Is there receiver.close()?
# Are messages FIFO globally, FIFO per sender, or unspecified?
# What exactly happens to the value passed to send(value) if the send is cancelled before enqueue?
```

My recommendation:

```text
Dropping the last sender closes the channel after buffered messages drain.
Dropping the receiver closes the channel immediately to senders.
Calling close() on any sender closes the whole channel to future sends.
Buffered messages remain receivable.
send(value) is exactly-once: on cancellation, the value is either not enqueued and dropped, or enqueued exactly once; it is never duplicated.
FIFO is guaranteed per channel according to enqueue order.
```

If you do not want global FIFO, say so.

---

## 16. Direct `receive()` and `async for` disagree semantically unless you define the adapter

Direct receive:

```sifr
await receiver.receive() -> Result[T, ClosedError]
```

Async iteration:

```sifr
anext() -> Result[Option[T], E]
```

Normal exhaustion is `Ok(None)`.

So for channel-backed async iteration, channel close must be translated:

```sifr
receive() == Err(ClosedError)
anext() == Ok(None)
```

That is fine, but it must be stated.

Otherwise users will ask why channel close is an error in one API and exhaustion in another.

Add:

```text
ChannelReceiver implements AsyncIterator[T, Never] by mapping closed-and-drained receive to Ok(None).
Direct receive exposes ClosedError so manual receive loops can distinguish terminal close from item delivery without Option.
```

Or reconsider direct `receive()` returning `Result[Option[T], E]`.

---

## 17. `SecondaryError` is better, but still too narrow

You now define:

```sifr
enum SecondaryError:
    CleanupFailed(error: Error, location: str)
    SiblingFailed(error: Error, task_id: str)
    CancellationDuringCleanup(cause: CancellationError)
```

Good.

But this is not enough.

Missing likely cases:

```sifr
SiblingCancelled(cause: CancellationError, task_id: str)
TaskPanickedOrRuntimeFault(...)
BlockingTaskAbandoned(...)
TimeoutCleanupFailed(...)
```

Also, `SiblingFailed(error: Error, task_id)` type-erases sibling errors. That is probably acceptable, but then say secondary evidence is type-erased.

You should also define whether `SecondaryError` is stable user-matchable API or diagnostic/log evidence only.

Right now it says inspectable evidence. That implies public stable shape. Be ready to support it.

---

## 18. `sifr.threading` in milestone 6 may violate “compatibility after canonical model”

The model says compatibility layers come after the canonical model exists. The phase file follows that for `sifr.asyncio` in milestone 8.

But milestone 6 adds:

```text
sifr.threading
```

as a compatibility veneer before async resources, async iteration, async generators, comprehensions, and phase closure.

That is inconsistent with the stated principle.

If `sifr.threading` is genuinely compatibility, move it to milestone 8.

If you need canonical threads earlier, make the canonical API `sifr.concurrent.Thread` or `sifr.concurrent.ThreadPoolExecutor` and keep `sifr.threading` as a later wrapper.

Do not sneak compatibility into milestone 6 while saying compatibility waits until the canonical model is proven.

---

## 19. `ThreadPoolExecutor` and blocking futures need signatures

You fixed `spawn_blocking` lifetime policy: owned, sendable, static captures; cancellation does not abort already-running OS work; result may be abandoned. Good.

But the return type is not specified.

Is this:

```sifr
task.spawn_blocking(fn) -> Task[T, E]
```

or:

```sifr
task.spawn_blocking(fn) -> BlockingTask[T, E]
```

or:

```sifr
task.spawn_blocking(fn) -> concurrent.Future[T, E]
```

It should probably not be ordinary `Task[T, E]`, because cancellation/lifetime semantics differ from cooperative async tasks.

You already say `sifr.concurrent.Future` is a compatibility wrapper over task/blocking handles, not a pure alias.

So define a canonical blocking handle:

```sifr
BlockingTask[T, E]
```

or explicitly state:

```sifr
spawn_blocking returns Task[T, E], but TaskResult.Cancelled means "result abandoned", not "OS work stopped."
```

That second option is dangerous because it overloads task cancellation semantics. I would use a distinct handle.

---

## 20. The old proposal file should be deleted or marked superseded

The new files are much better. But the old uploaded proposal still contains the old wrong ideas: `await Task[T, E] -> Result[T, E | CancellationError]`, MPMC channels, `Future = Task` alias, and must-consume orphaned handles.

If that old file still lives in the repo, it is now dangerous.

Do not leave it as “proposal candidate.” Rename it to something like:

```text
superseded_async_concurrency_model_2026_05_09.txt
```

or delete it.

The new phase file says the model file is authoritative. Good. Now make sure there is no stale document that a future implementer can accidentally follow.

---

# The biggest design decision still unresolved

The hardest unresolved design question is this:

**What happens to unobserved task failures at scope exit, type-wise?**

You have three possible answers.

## Option A: require observation of fallible tasks

```sifr
scope.spawn(task_that_returns_Result())  # must be awaited/gathered/handled
```

Only `Task[T, Never]` may be ignored.

This is easiest to type.

## Option B: type-erased scope failure

Unobserved child failures become:

```sifr
ScopeFailure: Error
```

Then `async with task.scope()` can fail with `ScopeFailure`.

This is practical and probably best for v1.

## Option C: scope error type evolves from spawned children

```sifr
TaskScope[E]
```

where `E` accumulates child error types.

This is elegant but complicated, especially with heterogeneous children and no explicit union-type contract.

I would pick Option B.

Add this to the model:

```sifr
struct ScopeFailure:
    primary: ScopeFailureCause
    secondary: list[SecondaryError]

enum ScopeFailureCause:
    UnobservedChildFailed(error: Error, task_id: str)
    UnobservedChildCancelled(cause: CancellationError, task_id: str)
```

Then:

```sifr
TaskScope.__aexit__ -> Result[None, ScopeFailure]
```

And define that explicit `await handle`, `gather`, `select`, `race`, or `timeout` marks a child result as observed.

That makes nursery ownership work without silently dropping failures.

---

# What I would change before implementation

I would make these edits before closing `milestone_async_0`:

1. Define `Task` handle consumption: direct `await`, `join`, `timeout`, `gather`, `select`, and `race` consume handles.
2. Add signatures for `Task.join`, `Task.cancel`, and maybe `Task.cancel_and_join`.
3. Define `ScopeFailure` and the exact type behavior of unobserved child failures at scope exit.
4. Give `TaskGroup` real API signatures and error-policy semantics.
5. Define `task.timeout(duration)` context-manager desugaring or defer it.
6. Fix the milestone dependency: milestone 1 must lower built-in `async with task.timeout(...)`, or timeout context manager must move later.
7. Decide whether `E | TimeoutError` depends on real union types. If not, replace it with an enum.
8. Define `ChildCancelled` or remove it from the canonical example.
9. Rewrite the canonical example using expression-match so `a` and `b` are definitely assigned.
10. Decide whether scoped borrowed spawn is truly v1. If yes, lock the runtime architecture that supports it.
11. Add channel endpoint drop/close/FIFO/cancelled-send rules.
12. Add `aclose()` signature and concurrent-`anext()` policy.
13. Either remove public `R` from `AsyncGenerator[T, E, R]` or reject non-`None` generator returns in v1.
14. Move `sifr.threading` compatibility to milestone 8 or rename the canonical thread API to `sifr.concurrent`.

---

# My final verdict

The update is a **huge upgrade**. The model now has a real semantic core instead of a Python/Rust/Tokio blend held together by prose.

I would now approve the direction and allow `milestone_async_0` to proceed.

I would **not** approve broad implementation yet until the remaining type holes are closed, especially:

```text
ScopeFailure
TaskGroup signatures
Task handle consumption
timeout context-manager desugaring
join/cancel signatures
scoped borrowed spawn feasibility
async generator close/reentrancy
```

Those are the places where async runtimes rot.

The proposal is no longer “promising but vague.” It is now **strong but still missing several contract edges**. Fix those, and it becomes genuinely bulletproof.
