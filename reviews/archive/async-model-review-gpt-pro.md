Here is the ruthless version.

The proposal is **not trash**. The core instinct is strong: one canonical async model, structured concurrency first, no raw event loop surface, explicit sharing, explicit offload, typed cancellation. That is the right shape.

But the document currently has a dangerous problem: it **sounds more precise than it is**. Several parts are semantically contradictory. If implementation starts from this as-is, the compiler/runtime team will invent missing rules ad hoc, and the model will drift into exactly the mess the proposal is trying to avoid.

The biggest issue: the proposal has not fully decided what an async computation *is*.

---

# Brutal verdict

The proposal is architecturally promising but **not implementation-ready**.

The parts that are strongest:

* structured concurrency as default;
* no user-visible event loop;
* no implicit detached tasks;
* no implicit `Arc` / `Mutex` / offload magic;
* active cancellation versus observed cancellation evidence;
* explicit blocking annotations;
* compatibility only after the canonical model exists.

The parts that do **not** make sense yet:

* async function calls, coroutines, tasks, futures, and `Result` are conflated;
* the opening example is type-wrong under the proposal’s own rules;
* `CancellationError` is both “not an ordinary error” and placed inside an ordinary `Result` error union;
* `TaskScope`, `TaskGroup`, `gather`, `select`, `race`, and timeout policies overlap without a clean ownership model;
* `async with task.scope()` is required before `async with` is implemented;
* timeout semantics probably require cancellation suppression, which the proposal says v1 does not expose;
* channel and lock APIs are underspecified enough to create runtime blocking bugs;
* `spawn_blocking` cancellation conflicts with scoped lifetime safety;
* compatibility mappings like `Event -> Notify`, `Queue -> Channel`, and `Future -> Task` are too glib.

So: the idea is good. The spec is not bulletproof yet.

---

# The fatal ambiguity: what does calling an async function return?

This is the single most important unresolved issue.

The proposal says async syntax should look like this:

```sifr
async def fetch_one(url: str) -> Result[str, NetworkError]:
    response: Response = await http.get(url)
    return response.text()
```

Then:

```sifr
first = scope.spawn(fetch_one("https://example.com/a"))
a: str = await first
```

But elsewhere it says:

```text
await Task[T, E] always produces Result[T, E | CancellationError]
```

Those cannot all be true at the same time.

Under the proposal’s own rules, this:

```sifr
a: str = await first
```

should not type-check. It should be something like:

```sifr
a_result: Result[str, NetworkError | task.CancellationError] = await first
```

or:

```sifr
a: str = try await first
```

except even that is not obviously valid because `CancellationError` is not an ordinary `Error`.

This is not a cosmetic problem. It infects the entire model.

You need to define these as separate things:

```sifr
AsyncFunction[Params, T, E]
Coroutine[T, E]
Task[T, E]
TaskResult[T, E]
```

A clean version would be:

```sifr
async def fetch_one(url: str) -> Result[str, NetworkError]
```

Calling it returns an unscheduled async computation:

```sifr
fetch_one(url) -> Coroutine[str, NetworkError]
```

Awaiting it in the same task gives:

```sifr
await fetch_one(url) -> Result[str, NetworkError]
```

Spawning it creates a child task:

```sifr
scope.spawn(fetch_one(url)) -> Task[str, NetworkError]
```

Awaiting the task handle gives:

```sifr
await task_handle -> TaskResult[str, NetworkError]
```

Do **not** blur `Coroutine` and `Task`. A coroutine is an async computation. A task is a scheduled child with lifetime, cancellation, ownership, and task-boundary rules.

Right now the proposal blurs them.

That is the first thing I would force you to fix.

---

# The `Result` model is internally confused

The proposal uses this shape:

```sifr
Task[T, E]
```

and says:

```sifr
await Task[T, E] -> Result[T, E | CancellationError]
```

But async functions themselves are written as returning `Result[T, E]`.

That raises a hard question:

```sifr
async def f() -> Result[str, NetworkError]
```

Does spawning `f()` produce:

```sifr
Task[str, NetworkError]
```

or:

```sifr
Task[Result[str, NetworkError], Never]
```

The proposal appears to want the first. But that requires a special rule:

> When an async function returns `Result[T, E]`, the async runtime lifts `E` into the task’s ordinary error channel.

That may be the right rule, but it is currently implicit.

You need to define the mapping explicitly:

```sifr
async def f() -> T
# call returns Coroutine[T, Never]

async def f() -> Result[T, E]
# call returns Coroutine[T, E]

scope.spawn(Coroutine[T, E])
# returns Task[T, E]
```

Then define what happens for nested results:

```sifr
async def f() -> Result[Result[A, E1], E2]
```

Does that become:

```sifr
Coroutine[Result[A, E1], E2]
```

Probably yes. But it must be stated.

Without this, the compiler team will invent their own lifting rules.

---

# `CancellationError` does not belong inside ordinary `Result`

The proposal says `CancellationError` is not a subclass of `Error`. Good. That is one of the best instincts in the document.

But then it says:

```sifr
await Task[T, E] -> Result[T, E | CancellationError]
```

That is suspicious.

If `Result[T, E]` means `E: Error`, then this is invalid because `CancellationError` is explicitly **not** an `Error`.

If `Result` can hold non-error control evidence, then `Result` is no longer just the ordinary error mechanism. That weakens the proposal’s own distinction between ordinary errors and cancellation evidence.

I would not use:

```sifr
Result[T, E | CancellationError]
```

I would define a separate type:

```sifr
TaskResult[T, E]
```

with variants:

```sifr
Completed(T)
Failed(E)
Cancelled(task.CancellationError)
```

or:

```sifr
Ok(T)
Err(E)
Cancelled(task.CancellationError)
```

Then:

```sifr
await Task[T, E] -> TaskResult[T, E]
```

That makes cancellation structurally impossible to confuse with ordinary error handling.

This also fixes the problem of `except Error` versus `except task.CancellationError`. If cancellation is a `TaskResult` branch, it is not “caught” by ordinary error handlers at all. It is matched.

Example:

```sifr
match await first:
    Ok(a):
        print(a)
    Err(e):
        return Err(e)
    Cancelled(c):
        return Err(task.ChildCancelled(c))
```

Right now the proposal says cancellation is not an error, but then jams it into the error slot.

That is not bulletproof.

---

# The opening example is misleading and should be deleted or rewritten

The proposal’s canonical example is currently dangerous because it teaches the wrong mental model.

This:

```sifr
a: str = await first
b: str = await second
```

is not honest under the proposed semantics.

A type-honest version should look closer to this:

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

That is ugly, but it exposes the truth.

Then you can add syntax sugar later:

```sifr
a = try await first
```

But only after defining whether `try` can propagate cancellation evidence from a function whose return type is `Result[None, Error]`.

Right now the example hides the central contract.

A proposal should not hide its hardest semantic rule in the first example.

---

# `try` / auto-unwrap is not defined well enough

The proposal says:

```text
Auto-unwrap applies to the Result produced by await, never to the Task handle itself.
```

That sounds nice, but it dodges the hard part.

Suppose:

```sifr
async def main() -> Result[None, Error]:
    a: str = try await first
```

and:

```sifr
await first -> Result[str, NetworkError | CancellationError]
```

Then what can `try` propagate?

`NetworkError` can probably be widened to `Error`.

But `CancellationError` cannot, because the proposal says it is not an `Error`.

So either:

1. `try await first` is illegal unless cancellation is handled explicitly;
2. `CancellationError` is auto-converted into some ordinary `Error`;
3. async functions have a hidden cancellation return channel;
4. task cancellation is control flow, not a result branch.

The proposal has not chosen.

This is a core type-system decision. It belongs in milestone 0, not later.

My recommendation:

```sifr
try await task
```

should only unwrap the ordinary error branch if cancellation has already been handled or if the enclosing function has an explicit cancellation channel.

Otherwise it should be rejected with a diagnostic like:

```text
task cancellation is not handled here
help: match the TaskResult explicitly or convert cancellation into an ordinary error
```

---

# Timeout semantics are probably impossible as written

The proposal says timeout cancels the enclosed operation and returns `TimeoutError`.

That sounds normal, but in this model it is tricky.

If timeout wraps a **child task**, then this is manageable:

```sifr
handle = scope.spawn(fetch_one(url))
result = await task.timeout(handle, 5.seconds)
```

If the timeout wins, cancel the child, wait for cleanup, return `TimeoutError`.

But if timeout wraps a **same-task awaitable**, like:

```sifr
result = await task.timeout(fetch_one(url), 5.seconds)
```

then timeout needs to cancel only the enclosed operation while letting the current task continue.

That requires some form of cancellation suppression or delimited cancellation. But the proposal says:

```text
cancellation suppression, uncancel counters, and shield-like APIs are not exposed in v1
```

Internal timeout cancellation is still cancellation suppression, even if you do not expose the API publicly.

So you need to decide:

Option A:

```sifr
task.timeout(handle, duration)
```

only accepts task handles. It cancels child tasks.

Option B:

```sifr
task.timeout(awaitable, duration)
```

internally spawns the awaitable into a child task, which means it now imposes spawn-boundary `Send` / ownership rules.

Option C:

```sifr
async with task.timeout(duration):
    ...
```

creates a cancellation scope. But then recovering from timeout requires explicit scoped cancellation semantics.

The proposal currently acts like all three are easy. They are not.

This needs to be nailed down.

---

# `async with task.scope()` appears before `async with` exists

The milestone ordering has a serious dependency bug.

The proposal uses:

```sifr
async with task.scope() as scope:
```

as the core task-scope syntax early in the model.

But milestone 7 is where `async with` and async context-manager protocols are implemented.

That means milestone 2 and milestone 3 depend on syntax/runtime behavior that does not exist yet.

You have three choices:

1. Move `async with` implementation much earlier.
2. Make `task.scope` a special compiler-recognized construct until milestone 7.
3. Do not use `async with` for task scopes in v1.

Option 1 is probably best.

The dependency graph should be changed. `async with` is not a late resource feature if task scopes depend on it. It is foundational.

---

# `TaskScope` and `TaskGroup` are not cleanly separated

The proposal says:

```text
TaskScope owns lifetime, while TaskGroup owns group error policy
```

That distinction is promising, but the rest of the proposal does not honor it cleanly.

You currently have:

* `task.scope`;
* `TaskScope`;
* `TaskGroup`;
* `scope.spawn`;
* `task.gather`;
* `task.select`;
* `task.race`;
* timeout cancellation;
* sibling failure cancellation.

These overlap.

A user will ask:

> When do I use `scope`, `TaskGroup`, or `gather`?

Right now the answer is muddy.

Make the layers brutally simple:

```sifr
TaskScope
```

Owns lifetime only. On normal exit, waits for children. On abnormal exit, cancels children.

```sifr
TaskGroup
```

Adds policy: fail-fast sibling cancellation, error aggregation, primary/secondary errors.

```sifr
gather
```

Consumes handles and returns ordered results. Decide whether it is fail-fast or collect-all.

```sifr
select/race
```

Consumes handles and returns one winner while cancelling losers.

Do not let all of these own overlapping cancellation policy unless you want users to memorize edge cases.

---

# The orphaned handle rule fights structured concurrency

The proposal says every `scope.spawn` handle must be awaited, joined, cancelled, or moved into a tracked collection.

Then it says `TaskScope.__aexit__` cancels remaining unconsumed child handles as a runtime backstop.

That is a confusing hybrid.

In structured concurrency, the scope itself normally owns child tasks. So this should be valid in a nursery-style model:

```sifr
async with task.scope() as scope:
    scope.spawn(worker_a())
    scope.spawn(worker_b())
# scope waits here
```

But the proposal seems to reject that because every handle must be consumed.

You need to choose one model.

## Model A: strict handle consumption

Every spawned task returns a must-use handle. If the handle is not consumed, compile error.

Then `__aexit__` should not silently cancel forgotten handles on normal exit. The compiler should catch it. Runtime cleanup is only a panic/abnormal safety backstop.

## Model B: nursery ownership

The scope owns every child. Handles are optional observers. Scope exit waits for children by default.

Then unawaited handles are not automatically errors.

Both models are defensible. The current hybrid is not.

My recommendation: use **nursery ownership** for `TaskScope`, and use must-consume semantics only for APIs like `select`, `race`, or detached/future-like handles.

---

# `gather` semantics are underspecified

The proposal says `gather` preserves input ordering.

Then it says v1 `gather` is fail-fast: first child error cancels unfinished children and returns that typed error.

Those are not contradictory, but they need sharper wording.

In fail-fast mode, ordered results only exist in the all-success case:

```sifr
task.gather([a, b, c]) -> Result[[A, B, C], E]
```

But if `b` fails, what happens to a successful `a`?

Is it discarded?

Attached as secondary evidence?

Unavailable?

Also, what does “first error” mean?

* first by completion time?
* first by input order among failed tasks?
* first observed by scheduler?
* first after cancellation drain?

The proposal says if multiple fail before cancellation completes, earliest handle in input order is primary. That is deterministic, but it is not exactly “first child error.”

Make it explicit:

```text
If any child fails, gather cancels unfinished siblings. After all cancellation cleanup completes, gather chooses the lowest input-index failed child as the primary error among failures observed before cleanup completion. Other failures become SecondaryError evidence.
```

Or choose completion-time order.

But do not mix both.

---

# `select` / `race` need ownership semantics

The proposal says `select` and `race` cancel losers by default.

Good.

But then the handles should be considered consumed.

This should be illegal:

```sifr
winner = await task.select(a, b)
later = await b
```

because `b` may have been cancelled by `select`.

So `select` should take ownership of handles:

```sifr
task.select(a, b)
```

After that, `a` and `b` are moved and unusable.

Also, the result shape is missing.

For homogeneous tasks:

```sifr
select(Task[T, E], Task[T, E]) -> TaskResult[T, E]
```

Fine.

For heterogeneous tasks:

```sifr
select(Task[A, EA], Task[B, EB])
```

What is the type?

You probably need:

```sifr
Select2[A, EA, B, EB]
```

with variants:

```sifr
First(TaskResult[A, EA])
Second(TaskResult[B, EB])
```

Otherwise users cannot know which task won.

---

# Channel semantics are not coherent yet

This line is a problem:

```sifr
channel.receive() returns Result[Option[T], ClosedError]
None indicates graceful end-of-stream after close and drain
```

If `None` means closed-and-drained, when does `ClosedError` happen on receive?

For send, `ClosedError` makes sense.

For receive, you probably want one of these:

```sifr
receive() -> Option[T]
```

where `None` means closed and drained.

Or:

```sifr
receive() -> Result[T, ClosedError]
```

where `ClosedError` means no more values.

But `Result[Option[T], ClosedError]` has two different ways to say “closed.”

That is a smell.

Also: bounded channels apply backpressure. Therefore `send` on a full bounded channel cannot be a normal synchronous method in async code. It must be awaitable:

```sifr
await sender.send(value)
```

or you need two APIs:

```sifr
sender.try_send(value) -> Result[None, TrySendError[T]]
await sender.send(value) -> Result[None, ClosedError]
```

The proposal currently says `sync.Channel`, but the operations are async in practice.

That needs to be explicit.

Also missing:

* does `sync.channel[T]()` return one channel object or `(Sender[T], Receiver[T])`?
* are senders clonable?
* are receivers clonable?
* who can close?
* does dropping all senders close the channel?
* what happens to a value if a blocked send is cancelled?
* are messages FIFO per sender, globally FIFO, or unspecified?

Channels are central. They cannot be hand-waved.

---

# `sync.Lock` in async code is dangerous

The proposal says `sync.Lock[T]` uses a synchronous Rust mutex internally in v1, and lock guards cannot cross `await`.

Rejecting guards across `await` is good.

But it is not enough.

This can still block an async runtime worker:

```sifr
value = shared.lock()
```

If the mutex is contended, the task blocks the OS thread, not just the async task.

So either:

1. `sync.Lock` is allowed in async code only with a warning;
2. `sync.Lock.lock()` is rejected in async code unless statically known uncontended, which is unrealistic;
3. v1 introduces `sync.AsyncLock`;
4. the docs explicitly say this is a synchronous lock and should only be used for tiny uncontended critical sections.

The proposal currently makes it sound safe as long as the guard does not cross `await`. That is not true.

At minimum, add:

```text
Acquiring sync.Lock in async code may block the current runtime worker under contention. It is permitted only for short critical sections. Channels are preferred. AsyncLock is deferred.
```

---

# `Shared[T]` needs deep immutability rules

The proposal says:

```sifr
sync.Shared[T] exposes shared ownership only; mutation requires Lock, RwLock, or message passing.
```

That is good, but incomplete.

What if `T` contains interior mutability?

```sifr
Shared[Cell[int]]
Shared[List[MutableThing]]
Shared[UnsafeHandle]
```

If `Shared[T]` is just “no mutation API on Shared itself,” users may still mutate through the contained object.

You need a trait/capability rule:

```sifr
Shared[T] requires T: ShareSafe
```

or:

```sifr
Shared[T] requires deep immutability unless T provides explicit synchronization internally.
```

Otherwise `Shared` becomes a fake safety boundary.

---

# Scoped borrows across spawned tasks may be too ambitious for v1

The proposal wants:

```sifr
spawn_scoped_borrow_ok.sifr
```

That implies a task spawned into a scope can borrow local state, as long as the scope proves the task cannot outlive the borrow.

That is a beautiful goal.

But it is also one of the hardest implementation pieces.

If v1 uses a thread-moving async runtime, spawned futures usually need to be sendable and lifetime-safe. Supporting non-`'static` scoped async tasks is not just a type-checker detail. It affects the runtime substrate and codegen architecture.

You need to decide early:

## Conservative v1

```text
scope.spawn requires owned captures.
Borrowed captures are rejected in v1.
```

This is simpler and safer.

## Ambitious v1

```text
scope.spawn supports scoped borrows.
```

Then milestone 0 must prove the runtime/codegen architecture can actually support scoped async tasks.

Right now the proposal wants ambitious semantics but treats them like a later ownership-checking detail. That is risky.

---

# `spawn_blocking` cancellation conflicts with lifetime safety

The proposal says cancelling `spawn_blocking` requests cancellation and drops/abandons the result, but v1 does not forcibly abort a running OS thread.

That is realistic.

But it creates a lifetime problem.

If blocking work may keep running after the async scope exits, then it cannot borrow anything from the scope.

So `spawn_blocking` must require:

```sifr
owned + sendable + static captures
```

even inside a scope.

Otherwise this is unsound:

```sifr
async with task.scope() as scope:
    local = SomeResource()
    h = task.spawn_blocking(fn() {
        use(local)
    })
    h.cancel()
# local is dropped while blocking thread may still use it
```

The proposal says thread-pool tasks obey Send/Sync capture rules, but it needs to say more:

```text
spawn_blocking does not support scoped borrowed captures in v1 because cancellation cannot stop already-running OS work.
```

Also, if a structured scope exits, does it wait for already-running blocking work to finish?

If yes, cancellation may hang.

If no, borrowed captures are impossible and resource cleanup becomes detached.

Pick one.

---

# `@blocking_io` policy contradicts the validation plan

The proposal says diagnostics are warning-level in v1 unless the API is known to break runtime safety.

But the negative validation fixture is named:

```text
blocking_call_in_async_rejected.sifr
```

That implies hard rejection.

Choose one.

Either:

```text
Known blocking calls in async are warnings by default.
```

Then the test should be:

```text
blocking_call_in_async_warns.sifr
```

Or:

```text
Known blocking calls in async are hard errors.
```

Then say so.

Right now the spec and test plan disagree.

---

# `Event -> Notify` is wrong

The proposal says `threading.Event` or `asyncio.Event` can map to `sync.Notify`.

That is probably wrong.

An Event is usually level-triggered:

```text
set once, future waiters pass until cleared
```

A Notify is often edge-triggered:

```text
wake current waiters; future waiters may miss it
```

These are not the same abstraction.

A compatibility `Event` likely needs:

```sifr
Lock[bool] + Notify
```

or a dedicated `sync.Event`.

Same for `Condition`. Saying `Condition = Notify + Lock` is directionally true but not enough. A condition variable has a predicate discipline, wake semantics, and lock release/reacquire behavior. A thin wrapper may not be thin.

This is one of those places where the proposal is trying too hard to avoid adding primitives. Avoiding primitives is good. Lying about semantic equivalence is bad.

---

# `asyncio.create_task` compatibility reintroduces the thing you rejected

The proposal rejects ambient detached tasks.

Good.

Then compatibility later includes:

```sifr
sifr.asyncio.create_task(fn)
```

But Python’s `asyncio.create_task` is basically ambient task creation on the current loop.

If Sifr’s version only works inside an explicit task scope, it is not really `asyncio.create_task`.

That may be acceptable, but document the divergence brutally:

```text
sifr.asyncio.create_task is only valid inside an explicit Sifr task scope or TaskGroup. It does not create ambient orphan tasks.
```

Otherwise compatibility users will expect Python behavior.

Better: omit `create_task` from the first compatibility veneer unless you have an explicit compatibility scope.

---

# `concurrent.Future = task.Task` is probably false

The proposal says:

```sifr
sifr.concurrent.Future is a type alias for sifr.task.Task
```

That seems too aggressive.

A thread-pool future and an async task handle may have different semantics:

* async task cancellation is cooperative at await points;
* blocking thread cancellation cannot abort already-running OS work;
* thread futures may be waited from sync code;
* task handles may be scoped to async lifetimes;
* thread-pool jobs may be `'static` only.

So `Future` as a pure alias risks lying.

It may be better to define:

```sifr
concurrent.Future[T, E]
```

as a compatibility wrapper with a subset of `Task`-like observation semantics, but not pretend it is identical unless you can prove the cancellation/lifetime rules match.

---

# `Queue -> Channel` is also too casual

An `asyncio.Queue` has queue-specific behavior: maxsize, FIFO, `put`, `get`, sometimes `task_done` / `join` in Python-compatible surfaces.

A channel is close, but not identical.

If you map Queue to Channel, document exactly which Queue features are supported and which are intentionally absent.

Otherwise your “compatibility veneer” becomes a source of false promises.

---

# `SecondaryError` is hand-waved

The proposal mentions `SecondaryError` several times:

* sibling tasks fail during cancellation;
* cleanup fails during unwinding;
* panic-like failures are surfaced as secondary structured errors.

Good instinct.

But what is the type?

Is it attached to `Err(E)`?

```sifr
Err(E, secondary: List[SecondaryError])
```

Is it metadata?

```sifr
TaskResult[T, E].secondary_errors()
```

Is it logged only?

Can users match on it?

Does it affect equality?

Can secondary errors themselves contain cancellation?

Until this is defined, the proposal’s cleanup/error story is not testable.

I would define:

```sifr
struct Failure[E]:
    primary: E
    secondary: List[SecondaryError]
```

Then:

```sifr
TaskResult[T, E] =
    Ok(T)
    Failed(Failure[E])
    Cancelled(Failure[CancellationError])
```

Something like that. The exact shape can differ, but it must be first-class.

---

# “Cancellation waits for cleanup” can hang forever

The proposal says cancellation waits for cleanup before scope exit.

That is usually correct.

But what if cleanup never finishes?

```sifr
async def __aexit__(...):
    await never_returns()
```

Then scope exit hangs forever.

That may be acceptable, but then do not claim deterministic shutdown in a strong sense. The result is deterministic but not guaranteed to complete.

You need a section called:

```text
Cancellation progress guarantees
```

And it should answer:

* Can cleanup be cancelled?
* Can cleanup receive a second cancellation?
* Is there a hard shutdown path?
* Are cleanup timeouts supported?
* What happens at process exit?
* What happens if a cleanup future loops forever?
* What happens if cleanup panics?

Right now the proposal says cleanup runs to completion unless forcefully aborted, but does not define forceful abort.

That is a hole.

---

# Panic handling is overpromised

The proposal says no user-triggerable runtime panics and says panic-like failures in async cleanup should become secondary structured errors.

That is an excellent goal, but it is also very strong.

At minimum, narrow the claim:

```text
Generated async/runtime code must not use unwrap, expect, or panic in user-triggerable paths. Runtime panics from user code or foreign libraries are caught at task boundaries where technically possible and surfaced as structured failure evidence.
```

Do not promise that every panic-like failure can always be tamed unless the implementation architecture proves it.

Also, if a panic occurs while another panic is unwinding, or while a lock is poisoned, the behavior must be defined.

The current wording sounds safer than it has proven.

---

# The type system needs a real `Union` decision

The proposal repeatedly writes:

```sifr
E | CancellationError
```

Does Sifr actually have union types?

If yes, milestone 0 must say that async depends on union typing.

If no, this syntax is misleading and should be replaced by an enum-like result type.

This is another argument for `TaskResult[T, E]`.

A new union type system is a huge dependency. Do not smuggle it into async by notation.

---

# Awaitable protocol is underspecified

The proposal says await is protocol-based:

```text
any type implementing Awaitable[T] is awaitable
```

But what is the protocol?

Questions:

* Is an awaitable one-shot or reusable?
* Can it be awaited twice?
* Is `Task` awaitable multiple times?
* Is a coroutine awaitable only once?
* Does awaiting consume the awaitable?
* Can awaitables be stored?
* Can awaitables borrow local values?
* What method implements the protocol?
* Is the protocol public or compiler-only?
* Can user types implement custom awaitables in v1?

Python has painful coroutine-reuse behavior. Rust futures are usually polled once by ownership. Sifr needs its own rule.

I would strongly prefer:

```text
Coroutine values are linear: awaiting or spawning consumes them.
Task handles are affine: observing may consume them unless explicitly cloneable/shared.
```

But whatever you choose, define it.

---

# Lock guards across `await` are only one borrow-across-await case

The proposal correctly rejects lock guards across await.

But the broader borrow-across-await rule is underspecified.

Example:

```sifr
x = list[0]       # borrow element?
await something()
use(x)
```

or:

```sifr
ref = object.field
await something()
ref.method()
```

The proposal says borrowed values across await must be proven valid or rejected. That is true but vague.

You need to decide whether v1 is conservative:

```text
No mutable borrow may be live across await. Immutable borrow may cross await only if source is immutable and not moved/mutated.
```

Good.

Then define what counts as live:

* lexical scope?
* last use?
* compiler liveness?
* destructors/finalizers?
* closures?

The proposal gestures at this but does not make it implementable.

---

# `sync.Channel` being in `sync` but async-aware is confusing

`sync.Lock` is synchronous.

`sync.Channel` seems to have async send/receive behavior.

`sync.Notify` may be async wait.

`sync.Semaphore` may have async acquire.

The module name `sync` is okay if it means “synchronization,” not “synchronous.” But then the API docs must distinguish:

```sifr
lock.lock()              # synchronous, may block
await channel.receive()  # async wait
await semaphore.acquire()
await notify.notified()
```

Right now the proposal groups them together without making blocking behavior obvious.

That will confuse users.

---

# `ThreadPoolExecutor` risks adding a second concurrency model

The proposal says compatibility veneers should not define a second model.

But `sifr.concurrent.ThreadPoolExecutor` and `sifr.threading.Thread` absolutely can become a second model if not constrained.

You need policies for:

* can threads outlive async scopes?
* are thread handles must-join?
* is detach allowed?
* can threads borrow local state?
* how are panics/errors returned?
* can async tasks wait on thread handles without blocking?
* can sync code wait on async task handles?
* does thread cancellation exist?
* how do thread handles interact with `TaskScope`?

Without this, the async model is clean but the thread model becomes the escape hatch.

---

# `TaskScope.__aexit__` behavior needs normal versus abnormal exit distinction

The proposal says:

```text
TaskScope.__aexit__ waits for all children to finish, or cancels unfinished children on abnormal exit
```

Good.

But elsewhere it says unconsumed children are cancelled at scope exit as a safety backstop.

Those are different policies.

Define:

## Normal scope exit

Possible policies:

```text
wait for all children
```

or:

```text
error if handles unconsumed
```

or:

```text
cancel unconsumed children
```

## Abnormal scope exit

Probably:

```text
cancel unfinished children, wait for cleanup
```

## Child failure

If plain `TaskScope`:

```text
child failure is stored until observed
```

or:

```text
scope exit fails
```

If `TaskGroup`:

```text
first failure cancels siblings
```

Right now plain scope and task group are too blended.

---

# The proposal underestimates how hard “tracked collection” is

This rule is ambitious:

```text
a handle moved into a collection is tracked only when the compiler can prove the collection is drained, consumed, or dropped before the task scope exits
```

That can explode into complex static analysis.

Start smaller.

Allow only blessed consumption APIs in v1:

```sifr
await handle
handle.cancel()
handle.join()
task.gather(handles)
task.select(handles)
task.race(handles)

for h in handles:
    await h
```

Reject clever cases.

Do not promise general collection proof unless you want milestone 3 to become a research project.

---

# Selection tie-breaking by handle creation order is awkward

The proposal says if multiple tasks complete in the same scheduler tick, handle creation order breaks ties.

Handle creation order is less intuitive than input order.

If I write:

```sifr
task.select(b, a)
```

I probably expect `b` to have priority in a tie, not whichever was spawned first.

Use input order for tie-breaking unless you have a strong reason not to.

It is easier to teach and easier to test.

---

# `Task[T]` shorthand is confusing

The proposal says:

```sifr
Task[T]: shorthand for Task[T, Never] plus cancellation
```

But every task already has cancellation observation.

So this phrase is odd.

Better:

```sifr
Task[T] = Task[T, Never]
```

and:

```sifr
await Task[T] -> TaskResult[T, Never]
```

where the only non-success branch is cancellation.

Do not say “plus cancellation” as if `Task[T, E]` does not also have cancellation.

---

# `TimeoutError` as ordinary error needs a type rule

If:

```sifr
task.timeout(handle: Task[T, E], duration)
```

then the result is probably:

```sifr
TaskResult[T, E | TimeoutError]
```

or:

```sifr
Result[TaskResult[T, E], TimeoutError]
```

or:

```sifr
TaskResult[T, E] | TimeoutError
```

Each has different semantics.

If timeout cancels the child and returns an ordinary error, does child cancellation appear as `CancellationError` or `TimeoutError`?

Probably timeout should translate child cancellation caused by the timeout into `TimeoutError`.

But if the child was externally cancelled at the same time, maybe it should be `Cancelled`.

These edge cases need rules:

* inner succeeds before deadline;
* inner fails before deadline;
* inner is externally cancelled before deadline;
* deadline fires first;
* deadline and success tie;
* deadline and failure tie;
* outer task is cancelled;
* cleanup fails after timeout cancellation.

The proposal covers some of this, but not enough.

---

# “No raw event loop” is good, but you still need scheduling contracts

The proposal correctly hides raw event loops.

But users still need some scheduling expectations:

* Is task scheduling fair?
* Is starvation possible?
* Is there `task.yield_now()`?
* Are CPU loops diagnosed?
* How often is cancellation observed?
* Are channel operations cancellation points?
* Is lock acquisition a cancellation point?
* Is `spawn_blocking` completion delivery ordered?

You do not need to expose an event loop to specify these.

Right now “cooperative scheduling” is implied but not made operational.

---

# Async resource protocols are too late

Milestone 7 implements async context managers and async iteration.

But task scopes need async context managers much earlier.

Channels may want async iteration before milestone 7.

So either:

* split protocol syntax from general user-defined protocol support;
* move `async with` earlier;
* avoid `async with task.scope()` until milestone 7.

Best fix:

```text
milestone_async_1:
  parse/lower async with

milestone_async_2:
  implement built-in async context manager support for task.scope

milestone_async_7:
  generalize user-defined async context-manager protocol
```

That makes the dependency honest.

---

# The compatibility phase is too optimistic

Compatibility veneers should be the last step. Good.

But the current compatibility list is still too broad unless you document divergences.

Risky mappings:

```text
asyncio.create_task -> scope.spawn
asyncio.Queue -> Channel
asyncio.Event -> Notify
threading.Event -> Notify
threading.Condition -> Notify + Lock
concurrent.Future -> Task
```

These are not clean aliases. They are approximations.

You need a table with three columns:

```text
Compatible behavior
Intentional divergence
Unsupported behavior
```

Otherwise users will assume Python compatibility and be angry.

---

# The proposal says “practical concurrent programs,” but omits networking reality

The scope excludes web frameworks and database clients. Fine.

But the canonical example uses:

```sifr
await http.get(url)
```

Is `http` part of the phase or not?

If not, the example is fake.

Use examples based on primitives the phase actually owns:

```sifr
await task.sleep(...)
channel.send(...)
scope.spawn(...)
```

Or explicitly say:

```text
http.get is illustrative and not part of Phase 32.
```

A proposal’s flagship example should not depend on an out-of-scope library.

---

# The “no ecosystem grab bag” discipline is good, but some exclusions need consequences

Deferring these is good:

* process pools;
* subprocess;
* signals;
* contextvars;
* raw selectors;
* full asyncio parity.

But each deferral has consequences.

No process pool means CPU-bound hard cancellation is impossible.

No subprocess/signals means graceful shutdown from OS signals is not a phase exit criterion.

No contextvars means request-local tracing/logging requires explicit arguments or future `task.local`.

No async generators means stream transformations are clunky.

That is okay, but the proposal should state the tradeoffs honestly, not just list exclusions.

---

# What I would force into milestone 0

Before implementation, milestone 0 should answer these exact questions.

## 1. What are the core async types?

Define these precisely:

```sifr
Coroutine[T, E]
Task[T, E]
TaskResult[T, E]
AsyncFunction[Params, T, E]
```

Do not use `Task` for all of them.

## 2. How does `Result[T, E]` in an async function signature map to `Coroutine[T, E]`?

Define the lifting rule.

## 3. What does `await` consume?

Answer separately for:

```sifr
Coroutine
Task
custom Awaitable
```

## 4. Is cancellation a `Result` error branch or a distinct task-result branch?

I strongly recommend distinct branch.

## 5. Can `try await task` propagate cancellation?

If yes, into what return type?

If no, require explicit match.

## 6. Does `scope.spawn` allow borrowed captures in v1?

Pick owned-only or fully scoped borrow support.

## 7. What does `TaskScope` do on normal exit?

Wait, cancel, or require all handles consumed?

Pick one.

## 8. What is the exact result type of `gather`, `select`, `race`, and `timeout`?

Write them as type signatures.

## 9. Are channel send/receive async methods?

Write the method signatures.

## 10. Can `sync.Lock.lock()` be called in async code?

If yes, document worker blocking risk.

## 11. What are `spawn_blocking` lifetime rules?

Require owned/static captures unless you wait for running blocking work to finish.

## 12. What is `SecondaryError` structurally?

Make it inspectable or remove it from v1.

---

# The most important rewrite

I would rewrite the model around this:

```sifr
async def fetch_one(url: str) -> Result[str, NetworkError]:
    response = try await http.get(url)
    return Ok(response.text())
```

Calling it:

```sifr
coro: Coroutine[str, NetworkError] = fetch_one(url)
```

Spawning it:

```sifr
handle: Task[str, NetworkError] = scope.spawn(coro)
```

Awaiting same-task coroutine:

```sifr
result: Result[str, NetworkError] = await fetch_one(url)
```

Awaiting spawned task:

```sifr
result: TaskResult[str, NetworkError] = await handle
```

Task result:

```sifr
enum TaskResult[T, E]:
    Ok(T)
    Err(E)
    Cancelled(task.CancellationError)
```

Then `gather`:

```sifr
task.gather(handles: List[Task[T, E]]) -> TaskResult[List[T], E]
```

Or, for heterogeneous gather, require tuple overloads.

Then `select`:

```sifr
task.select(a: Task[A, EA], b: Task[B, EB])
    -> Select2[TaskResult[A, EA], TaskResult[B, EB]]
```

Then timeout:

```sifr
task.timeout(handle: Task[T, E], duration: Duration)
    -> TaskResult[T, E | task.TimeoutError]
```

Or avoid unions:

```sifr
TaskResult[T, TimeoutFailure[E]]
```

The exact names do not matter. The separation matters.

---

# What is actually “trash”?

These are the parts I would call trash in their current form, not because the ideas are bad, but because the spec lies by omission.

## Trash as written: `await Task[T, E] -> Result[T, E | CancellationError]`

This undermines the cancellation distinction. Use `TaskResult`.

## Trash as written: the canonical example

It is type-wrong and hides cancellation/error handling.

## Trash as written: timeout over arbitrary enclosed operations

This is much harder than the proposal admits. Restrict timeout to child task handles first.

## Trash as written: `Event -> Notify`

Wrong semantics unless `Notify` is level-triggered, which would make it not just notify.

## Trash as written: `concurrent.Future` as a type alias for `Task`

Thread-pool futures and async task handles are not obviously identical.

## Trash as written: channel receive returning `Result[Option[T], ClosedError]`

Double closed-state smell.

## Trash as written: implementing task scopes before async context managers

Milestone dependency bug.

## Trash as written: “tracked collection” proof

Too broad for v1 unless sharply constrained.

---

# What is solid and worth keeping

Do not throw away the proposal. The good parts are very good.

Keep:

* no public event loop;
* scoped spawn as canonical task creation;
* detached spawn deferred;
* no implicit sharing wrappers;
* explicit `Shared`, `Lock`, `Channel`, `Semaphore`, `Notify`;
* blocking annotations as diagnostics, not magic;
* `spawn_blocking` explicit;
* process pools deferred until typed IPC/serialization exists;
* cancellation not catchable by broad `except Error`;
* compatibility after canonical model;
* no raw selectors/contextvars/transports in v1.

The proposal’s philosophy is right.

The current weakness is that it needs a **formal semantic core**, not more milestone prose.

---

# My ruthless recommendation

Do not implement this yet.

Spend one short milestone producing a stricter contract with type signatures and state machines.

The document should include:

1. exact async type model;
2. exact await/spawn consumption rules;
3. exact task result/cancellation representation;
4. exact scope-exit state machine;
5. exact timeout state machine;
6. exact channel method signatures;
7. exact lock behavior in async contexts;
8. exact `spawn_blocking` cancellation/lifetime policy;
9. corrected milestone dependency graph;
10. corrected examples that type-check under the rules.

Once those are fixed, this becomes a strong proposal.

Right now it is a strong architectural direction wrapped around several unresolved semantic contradictions. Those contradictions are exactly where async runtimes rot.
