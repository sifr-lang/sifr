This is now **very close**.

You fixed the big stuff: `Coroutine`/`Task`/`TaskResult` are separated, `CancellationError` is no longer shoved into ordinary `Result`, task handles are affine, `ScopeFailure` exists, `TaskGroup[E]` is homogeneous, timeout uses `TimeoutResult[E]`, scoped borrowed spawn is deferred, async generators dropped public `R`, lazy async generator expressions are deferred, and `BlockingTask` is distinct from cooperative `Task`. Those were the right moves.

My verdict now:

**I would approve this as the Phase 32 direction. I would not start implementation until a small set of redlines are resolved.**

The remaining problems are much narrower. They are not “this model is confused” problems anymore. They are “this statement still forces implementers to guess” problems.

---

# What is now genuinely strong

The model has a real semantic spine now:

```sifr
async def f(...) -> Result[T, E]
f(...) -> Coroutine[T, E]
scope.spawn(f(...)) -> Task[T, E]
await Task[T, E] -> TaskResult[T, E]
```

That is clean. It avoids Python’s coroutine/task confusion and avoids Rust/Tokio leaking into the public surface. The explicit `TaskResult.Ok / Err / Cancelled` split is the right shape.

The `ScopeFailure` fix is also important. You now have a way to surface unobserved child failure at scope exit instead of silently discarding it. That makes nursery ownership safe without returning to the old “every handle must be manually consumed” model.

The Phase 32 plan is also much tighter. The phase file now treats the model contract as the source of truth, separates milestones cleanly, and moves compatibility to the end. That was the right structural decision.

The decision to defer scoped borrowed spawn is especially good. Earlier, that was a hidden implementation landmine. Now v1 requires owned, sendable, static captures, and the model admits that scoped borrowed spawn needs a different runtime strategy. That is honest.

---

# The biggest remaining problem: timeout context blocks

This is the one place where the model still has a serious semantic trap.

You define:

```sifr
async with task.timeout(duration):
    ...
```

as a compiler-recognized timeout scope lowered into a temporary child task owned by the current structured scope. The deadline cancels that child and exits through ordinary `TimeoutError`.

That sounds clean, but it collides with your v1 spawn rule:

> v1 `scope.spawn` requires owned, sendable, static captures. Scoped borrowed spawn is deferred.

Now consider ordinary user code:

```sifr
async def f() -> Result[int, Error]:
    value = 10

    async with task.timeout(1.s):
        value = value + try await slow_number()

    return Ok(value)
```

If the timeout block is lowered into a child task, then `value` crosses a task boundary. Under the v1 rule, that requires owned/sendable/static capture. But inline timeout blocks are expected to behave lexically, like normal code. Users will expect to read and mutate surrounding locals.

So you have to choose one of these:

## Option A: timeout context blocks are same-task cancellation scopes

Then they can access locals naturally, but the runtime/compiler needs internal delimited cancellation. You can still keep public cancellation suppression/shield APIs deferred, but internally you need a cancellation-scope mechanism.

## Option B: timeout context blocks are child-task blocks

Then say explicitly:

```sifr
async with task.timeout(duration):
    ...
```

has spawn-boundary capture rules in v1. It cannot borrow or mutate surrounding locals unless moved through owned/sendable/static state.

That is surprising and probably too restrictive.

## Option C: defer timeout context blocks

Keep only:

```sifr
handle = scope.spawn(work())
result = await task.timeout(handle, 1.s)
```

This is the most honest v1 implementation if you do not want internal cancellation scopes yet.

My recommendation: **do not ship the timeout context-manager form until you can make it same-task lexical code**. The handle form is already solid. The context form is still dangerous.

This is the most important remaining redline.

---

# Second redline: fallible `async with` needs exact propagation rules

You now say:

```sifr
TaskScope.__aexit__ -> Result[None, ScopeFailure]
```

and timeout context exit can produce ordinary `TimeoutError`.

But what does `async with` do with a fallible `__aexit__` result?

Example:

```sifr
async def main() -> Result[None, Error]:
    async with task.scope() as scope:
        scope.spawn(fails())
        return Ok(None)
```

When the block returns `Ok(None)`, `__aexit__` still runs. If an unobserved child failed, `__aexit__` returns `Err(ScopeFailure)`.

What wins?

* Does `ScopeFailure` override the `return Ok(None)`?
* Does it become secondary evidence?
* Does `async with` auto-propagate it like `try`?
* Is `async with` only legal when the enclosing function can carry the exit error?
* Is there a `try async with` form?

You need a formal rule for all combinations:

```text
body succeeds, exit succeeds
body succeeds, exit fails
body returns Err(E), exit succeeds
body returns Err(E), exit fails
body returns early, exit fails
body is actively cancelled, exit fails
body panics/runtime-faults, exit fails
```

Right now the model says cleanup errors become secondary during cancellation, but it does not fully define fallible context-manager exit during normal or ordinary-error paths.

This is not cosmetic. `TaskScope`, `TaskGroup`, `task.timeout`, async generators, async iterators, and user-defined async context managers all depend on it.

Add a section:

```text
Fallible async context-manager control flow
```

with exact lowering.

---

# Third redline: `async for` needs exact error propagation rules

You define:

```sifr
AsyncIterator[T, E].anext() -> Result[Option[T], E]
```

Good. Normal exhaustion is `Ok(None)`, stream failure is `Err(E)`.

But this needs one more type rule:

```sifr
async for item in stream:
    ...
```

When `anext()` returns `Err(E)`, what happens?

Possible rules:

```sifr
# Rule A
async for auto-propagates Err(E) like `try`
```

or:

```sifr
# Rule B
async for is only valid inside a fallible context that can carry E
```

or:

```sifr
# Rule C
user must write a special form to handle iterator failure
```

The model currently says `async for` follows surrounding result/try rules, but that is not a complete lowering rule.

I would define it like this:

```sifr
async for item in source:
    body
```

desugars roughly to:

```sifr
loop:
    next = try await anext(source)
    match next:
        Some(item):
            body
        None:
            break
```

That means `async for` is fallible when the iterator is fallible. The enclosing function must be able to propagate `E`, or the compiler rejects it.

Then document how users explicitly handle iterator errors if they do not want propagation.

---

# Fourth redline: `TaskGroup` still has an observation edge case

The new `TaskGroup[E]` is much better: homogeneous child error type, sibling cancellation on first failure, group owns error policy.

But this case is still unclear:

```sifr
async with task.TaskGroup[MyError]() as group:
    a = group.spawn(fails())
    b = group.spawn(slow())

    match await a:
        Err(failure):
            handle(failure.primary)
        Ok(_):
            pass
        Cancelled(c):
            pass
```

If `a` fails, the group cancels `b`.

Now what happens at group exit?

* Is `b`’s group-triggered cancellation considered “unobserved child cancellation” and surfaced as `ScopeFailure`?
* Or does the group policy mark sibling cancellations as internally observed?
* If `a` was already explicitly observed by the user, does group exit return `Ok(None)` after cancelling `b`?
* If `b` cleanup fails, where does that secondary evidence attach?

I think the right rule is:

```text
A TaskGroup internally observes policy-triggered sibling cancellations. They do not produce ScopeFailure merely because the user did not await every cancelled sibling. Cleanup failures from those siblings attach as SecondaryError to the group failure if a group failure exists; otherwise they surface as ScopeFailure.
```

But the model needs to say it.

Otherwise `TaskGroup` can become annoying: users handle the failed task correctly, but group exit still fails because it cancelled siblings by policy.

---

# Fifth redline: timeout wording is still type-wrong in one sentence

The API is now:

```sifr
task.timeout(handle: Task[T, E], duration: Duration)
    -> TaskResult[T, TimeoutResult[E]]
```

Good.

But the prose says:

> if the inner task completes before `duration`, timeout returns the inner `TaskResult`

That is not literally true when the inner task fails.

If the child returns:

```sifr
TaskResult.Err(Failure[E])
```

timeout must return:

```sifr
TaskResult.Err(Failure[TimeoutResult.Inner(E)])
```

because the result type is `TaskResult[T, TimeoutResult[E]]`.

So the wording should be:

```text
If the inner task succeeds before the deadline, timeout returns Ok(T).
If the inner task fails before the deadline, timeout maps the ordinary failure to TimeoutResult.Inner(E).
If the inner task is cancelled before the deadline, timeout preserves Cancelled(Failure[CancellationError]).
If the deadline wins, timeout returns Err(Failure[TimeoutResult.Timeout(TimeoutError)]).
```

Small wording bug, but type bugs in specs become implementation bugs.

---

# Sixth redline: channel endpoint lifetime still needs rules

Channels are much better now: explicit sender/receiver endpoints, async send/receive, single receiver, no double closed state.

But you still need endpoint lifetime rules:

```text
Does dropping the last sender close the channel?
Does dropping the receiver close the channel to senders?
Does sender.close() on one cloned sender close the whole channel or only that sender?
Can another cloned sender send after one sender calls close()?
Are buffered messages still delivered after close?
Is FIFO global by enqueue order, per sender, or unspecified?
```

I would lock this:

```text
Dropping the last sender closes the channel after buffered messages drain.
Dropping the receiver closes the channel immediately to senders.
Calling close() on any sender closes the whole channel to future sends.
Buffered messages remain receivable.
Messages are received in channel enqueue order.
```

Also add a negative test for sending after receiver drop and for buffered delivery after close.

Without this, channel behavior will be copied from whatever Rust primitive you happen to use, which violates the runtime-neutrality goal.

---

# Seventh redline: `AsyncClosable` should not hardcode `GeneratorCloseError`

You currently define:

```sifr
protocol AsyncClosable:
    async def aclose(self) -> Result[None, GeneratorCloseError]
```

That is too generator-specific.

Async closable iterators are not always generators. A stream, file-line iterator, socket-frame iterator, or database cursor should not have to pretend its close failure is a `GeneratorCloseError`.

Use one of these instead:

```sifr
protocol AsyncClosable[E]:
    async def aclose(self) -> Result[None, E]
```

or:

```sifr
protocol AsyncClosable:
    async def aclose(self) -> Result[None, CloseError]
```

Then:

```sifr
AsyncGenerator[T, E] implements AsyncClosable[GeneratorCloseError]
```

Right now the protocol name is general but its error type is generator-specific. That is a bad abstraction leak.

---

# Eighth redline: `async for` early exit cleanup must be explicit

You define cleanup for async generators and async comprehensions, but ordinary `async for` early exit needs an explicit rule too.

Example:

```sifr
async for line in stream_lines(path):
    if line == "stop":
        break
```

Does this call `aclose()` on the iterator if it implements `AsyncClosable`?

It should.

Same for:

```sifr
return Ok(value)
```

inside an `async for`.

I would add:

```text
If an async for loop exits before iterator exhaustion because of break, return, ordinary error propagation, timeout, or active cancellation, and the iterator implements AsyncClosable, the compiler/runtime awaits aclose() before leaving the loop. On normal break/return, close failure is primary ordinary error. During cancellation/timeout, close failure is secondary evidence.
```

That makes async generators resource-safe.

---

# Ninth redline: milestone 2 implements spawn before ownership checking

Milestone 2 implements `scope.spawn`, task handles, join, cancellation, and timeout. Milestone 4 later implements Send/Sync and borrow task-boundary checking.

That is fine only if milestone 2 is deliberately conservative.

Add:

```text
Before milestone_async_4, scope.spawn accepts only trivially owned/static captures or fixture-limited no-capture coroutines. Nontrivial captures are rejected with a temporary diagnostic until full task-boundary checking lands.
```

Otherwise milestone 2 may accidentally allow code that milestone 4 later rejects, or worse, rely on raw Rust errors.

Do not let a partially implemented milestone create a semantics regression.

---

# Tenth redline: `BlockingTask` still reuses `TaskResult.Cancelled` in a confusing way

You made `BlockingTask[T, E]` distinct from `Task[T, E]`. Good. But then:

```sifr
BlockingTask.join() -> TaskResult[T, E]
```

and `Cancelled` means “observer abandoned the result,” not necessarily “work stopped.”

That is technically documented, but still semantically sharp.

I would prefer:

```sifr
BlockingTaskResult[T, E]:
    Ok(T)
    Err(Failure[E])
    Abandoned(Failure[CancellationError])
```

If you keep `TaskResult`, then the docs and diagnostics must scream:

```text
Cancelled on BlockingTask does not guarantee OS work stopped.
```

Otherwise users will infer the wrong thing.

---

# Good cuts you made

These were the right ruthless cuts:

* Deferring scoped borrowed spawn. This avoids a very hard runtime/codegen problem in v1.
* Deferring process pools until typed IPC/serialization exists. Correct.
* Deferring cancellation suppression/shield/uncancel APIs. Correct for public v1.
* Deferring lazy async generator expressions. Correct. They were pure complexity at this stage.
* Making `Future` a compatibility wrapper, not a pure alias. Correct.
* Making `Event` not equal to `Notify`. Correct.

This shows the proposal is converging instead of accumulating features.

---

# Minor issues worth cleaning up

These are not blockers, but they will reduce confusion.

## `task.race` should not be called an alias for `select`

`select` returns a winner variant:

```sifr
Select2[TaskResult[A, EA], TaskResult[B, EB]]
```

`race` returns:

```sifr
TaskResult[T, E]
```

Those are not aliases. Say:

```text
race is the homogeneous convenience form of select that discards winner identity.
```

Or return winner index from `race`.

## `TimeoutResult[E]` should explicitly implement `Error`

You require task ordinary error channels to satisfy `E: Error`. Then timeout produces `TimeoutResult[E]`. Say:

```text
TimeoutResult[E] implements Error when E: Error.
```

## `TaskGroup[E]` should define whether `Never` children are allowed

Can this be accepted?

```sifr
async with task.TaskGroup[MyError]() as group:
    group.spawn(no_error_task())  # Task[T, Never]
    group.spawn(my_error_task())  # Task[U, MyError]
```

I would allow `Never` to coerce into `E`, but say it.

## Add method signatures for `Lock`, `RwLock`, `Semaphore`, and `Notify`

You have channel signatures, task signatures, and generator signatures. The sync primitives still need the same treatment before milestone 5 starts.

## Rename or justify `sifr.threading`

The phase file says `sifr.threading` is Sifr-native thread coordination, not Python compatibility.

That is defensible, but the name will read as Python compatibility. Consider making canonical thread APIs live under:

```sifr
sifr.concurrent
```

and reserve:

```sifr
sifr.threading
```

for compatibility veneer behavior.

---

# What I would change before implementation

I would add one more mini-section to the model before closing `milestone_async_0`:

```text
Control-flow desugarings
```

It should define:

```sifr
async with
async for
task.timeout(duration)
TaskScope.__aexit__
TaskGroup.__aexit__
async for early exit cleanup
```

with exact behavior for:

```text
success
ordinary Err(E)
return
break
active cancellation
timeout
cleanup failure
panic-like runtime fault
```

This is the last major missing formalism.

Right now the type vocabulary is strong. The remaining risk is control-flow lowering.

---

# Final verdict

This version is **no longer hand-wavy**. It is a serious model.

I would mark it:

```text
Approved directionally.
Needs redline pass before implementation.
```

The redlines I would require before code starts:

1. Fix `task.timeout(duration)` context semantics, especially local capture and spawn-boundary implications.
2. Define fallible `async with` propagation exactly.
3. Define `async for` error propagation exactly.
4. Define `TaskGroup` observation rules for policy-cancelled siblings.
5. Correct timeout prose so ordinary child failure maps to `TimeoutResult.Inner(E)`.
6. Add channel drop/close/FIFO rules.
7. Generalize `AsyncClosable` so it is not tied to `GeneratorCloseError`.
8. Define `async for` early-exit cleanup.
9. Make milestone 2 conservative until milestone 4 ownership checks exist.
10. Decide whether `BlockingTask.join()` should return `TaskResult` or a distinct `BlockingTaskResult`.

After those, I would call this implementation-ready.
