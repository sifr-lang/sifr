# Async and Concurrency Model Rules

Status: canonical async and concurrency model rules
Last updated: 2026-05-09

## Purpose

This document defines the final Sifr async and concurrency model: the public vocabulary, type semantics, cancellation behavior, task ownership rules, synchronization primitives, and compatibility boundaries.

The model is one coherent async story:

- Python-shaped syntax: `async def`, `await`, `async with`, `async for`
- Rust-shaped safety: ownership-aware task boundaries, explicit sharing, no hidden thread-safety wrappers
- Structured concurrency by default: parent scopes own child tasks
- Typed cancellation and shutdown behavior
- Explicit offload for CPU-bound or blocking work
- Blocking/IO annotations that power diagnostics instead of hidden scheduling changes
- Compatibility layers only after the canonical model exists

The model succeeds when users can write practical concurrent Sifr programs without learning raw event-loop internals and without escaping Sifr's core guarantee: no user-triggerable runtime panics.

## Product Decision

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
                return Err(TaskCancelled(f"task cancelled: {cancelled.primary}"))

        b: str = match await second:
            Ok(value):
                value
            Err(failure):
                return Err(failure.primary)
            Cancelled(cancelled):
                return Err(TaskCancelled(f"task cancelled: {cancelled.primary}"))

        print(a + b)
        return Ok(None)

class TaskCancelled(Error):
    message: str
```

The user-facing vocabulary is:

- `async def`
- `await`
- `sifr.task`
- `sifr.sync`
- scoped task groups
- explicit channels and locks
- explicit blocking/thread offload
- `@blocking_io` and `@cpu_heavy` annotations for synchronous workload classification

The primary model does not include:

- user-visible event-loop objects
- event-loop policies
- callback-first APIs
- implicit detached tasks
- implicit `Arc`, `Mutex`, or thread-safe wrapper insertion
- implicit offload of blocking or CPU-heavy calls
- raw `Future` manipulation as the normal user path

## Design Principles

### One Canonical Model

Sifr has one public async story: `async def`, `await`, `sifr.task`, and `sifr.sync`.

`sifr.asyncio` is not a public compatibility veneer. It was removed by the production concurrency/runtime substrate gate; code must import the native `sifr.task` and `sifr.sync` APIs directly.

### Structured Concurrency First

Task lifetime should be visible in source code. Child tasks should normally belong to a parent scope. Detached work must be explicit and rare.

Default APIs should prefer:

- `task.scope(...)`
- `task.TaskGroup`
- `scope.spawn(...)`: canonical task creation; all spawned tasks are children of a scope
- `task.gather(*handles)`: wait for multiple task handles, preserving input ordering
- `task.select(first=a, second=b)`: binary named-branch first-completion semantics; losers are cancelled by default
- `task.race(handles)`: homogeneous first-completion over a list that returns winner evidence; losers are cancelled by default

Default APIs should not encourage:

- ambient global tasks
- silent fire-and-forget work
- detached task handles
- shutdown behavior that depends on runtime accident

### Async Is For Waiting

Async tasks are for I/O waiting and cooperative scheduling. CPU-intensive work and synchronous I/O calls must use explicit offload APIs when they would otherwise occupy a cooperative runtime worker.

Required surfaces:

- `@blocking_io` for sync functions that perform synchronous I/O such as file, network, database, pipe, or blocking timer waits
- `@cpu_heavy` for sync functions expected to be CPU-intensive, such as cryptography, compression, hashing, parsing, numerical compute, or computation-heavy processing
- `task.spawn_blocking(...)`
- accepted `sifr.parallel` APIs for CPU parallelism

These annotations are declaration-site workload facts for synchronous functions, not scheduling commands and not async effects. Calling a known `@blocking_io` function from async code should produce a Sifr diagnostic that suggests an async API when one exists, or explicit offload when it does not. Calling a known `@cpu_heavy` function from async code should suggest `task.spawn_cpu` or accepted `sifr.parallel` APIs. The compiler must not silently rewrite either call into a task or thread.

### Replay-Safe Callbacks

`@retry_safe` classifies a function that the runtime can replay. It is not a
scheduling command or a general effect system.

The compiler validates each Sifr function that uses `@retry_safe`. The function
can call only these operations:

- compiler-proven pure operations
- other validated `@retry_safe` functions
- operations on the explicit replay capability parameter

The function cannot use external I/O, process state, detached tasks, random
state, clocks, or mutable global state. Each captured value must be owned and
implement `Clone`.

An external declaration needs separate certification before it can use
`@retry_safe`. An unchecked annotation is a compile error.

Database transaction replay uses the transaction as its replay capability. Each
attempt receives a fresh transaction and cloned captures.

Deferred surfaces:

- `ProcessPoolExecutor`
- multiprocessing-style APIs

Process pools require a stable typed data and IPC serialization rules. Shipping them first would force a premature transport model.

### No Implicit Shared Mutable Memory

The compiler must not silently turn local state into shared state.

Allowed explicit surfaces:

- `sync.Shared[T]`
- `sync.Lock[T]`
- `sync.RwLock[T]`
- `sync.Channel[T]`
- `sync.Semaphore`
- `sync.Notify`

Deferred coordination primitives:

- `sync.Barrier`
- `sync.Condvar`

`Barrier` and `Condvar` are useful, but they are not required to prove the first model. `Notify`, `Semaphore`, channels, and locks cover the common coordination paths with a smaller teaching surface.

Rejected implicit behavior:

- silently upgrading `Rc` to `Arc`
- silently wrapping mutable values in `Mutex`
- silently cloning captured mutable state for task safety
- silently moving a blocking call onto another executor
- implicitly detaching borrowed values from their owner

### Typed Failure and Cancellation

Cancellation is part of the rules, not an implementation detail.

Timeouts, task cancellation, sibling failure, shutdown tokens, and resource cleanup must have deterministic behavior and Sifr-native diagnostics. Cancellation must not become an ambient exception leak or an ordinary `Result` error branch.

Cancellation has two forms:

- **active cancellation signal**: the runtime control signal delivered to a running task at cooperative cancellation points. It unwinds the task, runs cleanup, and is not caught by ordinary `except Error` handlers inside the cancelled task.
- **materialized cancellation evidence**: the `sifr.task.CancellationError` value observed by a non-cancelled owner when inspecting or awaiting a cancelled child task handle.

This split avoids Python's `CancelledError` footgun. Broad user handlers such as `except Error as e` can handle ordinary task failures, but they must not accidentally turn the current task's own cancellation into a successful recovery path. A task may intentionally observe a child task's materialized cancellation result, but active cancellation of the current task remains scope-exit semantics.

`CancellationError` is not a subclass of `Error`. It belongs to the task cancellation control family and is therefore never matched by broad `except Error`. It is materialized only as the `Cancelled(...)` branch of `TaskResult[T, E]` or inside `SecondaryError` evidence. This avoids Python-style cancellation swallowing without adding a hidden subclass-matching exception to the ordinary error hierarchy.

### Cancellation Progress Guarantees

Cancellation is cooperative:

- cancellation is requested at task boundaries and observed at await/cancellation points
- active cancellation runs `finally` blocks and async context cleanup before the task is considered complete
- cleanup awaits use the bounded cleanup budget defined below
- Sifr does not expose forceful task abort, cancellation suppression, shielding, or uncancel counters
- CPU loops that do not await cannot be interrupted until they reach a cooperative cancellation point
- cleanup failures become structured secondary evidence instead of replacing the primary cancellation/failure cause

The shutdown result is deterministic. The cleanup budget prevents an unlimited
wait during cancellation.

### Bounded Cleanup After Cancellation

The runtime gives cleanup code a bounded budget during cancellation unwinding.
This rule applies to `finally`, `__aexit__`, and `AsyncClosable.aclose`.

The runtime does not deliver the active cancellation again during this budget.
A second cancellation request does not restart or extend the budget.

Each resource owner defines a cleanup timeout. The runtime uses its default
timeout when the resource does not define one.

If cleanup finishes in time, the original cancellation remains primary. A
cleanup error becomes `SecondaryError.CleanupFailed`.

If the budget expires, the runtime drops the cleanup future. The resource owner
must invalidate and discard the underlying resource without another awaited step.

The resource owner must not return an unclean resource to a pool. Budget expiry
adds `SecondaryError.CleanupFailed` to the original cancellation.

The secondary error includes the resource type, cleanup operation, and timeout.
This compiler rule does not add a public shielding API.

### Runtime Is An Implementation Detail

The implementation may use Tokio or a Tokio-compatible runtime substrate, but ordinary Sifr users should not configure a runtime directly.

The compiler should:

- detect async usage
- wire the required runtime dependency
- generate the correct entrypoint bootstrap
- reject invalid async usage before Rust compilation where possible
- translate remaining runtime/type-bound failures into Sifr diagnostics

## Scope Boundaries

### In Scope

- async syntax lowering
- awaitable type model
- runtime bootstrapping
- task handles
- scoped task groups
- cancellation and timeout semantics
- gather/select/race composition
- async context managers
- async iteration
- async generators
- async comprehensions
- task-boundary ownership and Send/Sync checking
- explicit synchronization primitives
- explicit blocking/thread offload
- diagnostics for the model

### Out Of Scope

- web framework
- typed serialization and pydantic-like validation
- database clients
- full `asyncio` parity
- raw event-loop APIs
- transports/protocols callback APIs
- public selectors module unless later socket work requires it
- `contextvars`
- multiprocessing
- process pools
- async generator `send()` and `throw()`
- async `yield from` / generator delegation
- async generator expressions
- nested async comprehensions and awaited comprehension filters

Some out-of-scope items may get thin compatibility wrappers later. They should not be allowed to shape the core model.

`contextvars` is intentionally deferred. Sifr should prefer lexical scope and explicit task arguments over implicit task-local propagation. If later evidence shows a real need for task-local storage with structured inheritance, a `sifr.task.local[T]` primitive can be designed as a scoped, lexical value with structured inheritance, not as a global mutable copy-on-fork store.

`selectors` is intentionally not a public model requirement. Runtime internals may use readiness machinery, but users should compose tasks and channels rather than file-descriptor readiness APIs. If low-level socket work later requires a public module, it should land as a curated compatibility layer with CPython-derived tests, not as a core async concept.

Async generators are the user-defined producer side of async iteration. The first model owns the async iterable protocol, `async for` over channels/streams, user-defined `async def` bodies with `yield`, and first-class async comprehensions over async iterables. Advanced Python generator controls such as async `send()`, async `throw()`, and async `yield from` are deferred.

## Type System

The type system has first-class awaitability and task-boundary rules.

Core types:

- `Coroutine[T, E]`: an unscheduled async computation created by calling an async function. It is linear: awaiting it or spawning it consumes it. `E` is the ordinary error channel.
- `Task[T, E]`: a scheduled child task handle. It is not a `Result`; it is an awaitable handle that yields `TaskResult[T, E]` when observed from outside the task.
- `Task[T]`: shorthand for `Task[T, Never]`. It still observes cancellation through `TaskResult[T, Never]`; cancellation is not part of the ordinary `E` channel.
- `TaskResult[T, E]`: the result of observing a scheduled task handle. It has three branches: `Ok(T)`, `Err(Failure[E])`, and `Cancelled(Failure[CancellationError])`.
- `Failure[E]`: a primary failure plus secondary evidence: `primary: E`, `secondary: list[SecondaryError]`.
- `CancellationError`: materialized evidence that a child task was cancelled. It is not the in-task active cancellation signal and does not inherit from `Error`.
- `TaskCancelled`: ordinary `Error` wrapper for callers that intentionally convert materialized child cancellation into their own error channel.
- `TimeoutError`: ordinary timeout failure produced when a timeout scope wins its race and cancels the enclosed operation.
- `TimeoutResult[E]`: explicit ordinary error enum for timeout wrappers. It has `Inner(E)` and `Timeout(TimeoutError)` branches.
- `ScopeFailure`: ordinary `Error` produced by scope exit when unobserved child task failure or cancellation must be surfaced.
- `SecondaryError`: structured evidence attached to a primary cancellation or failure when cleanup or sibling tasks also fail during unwinding.
- `ClosedError`: ordinary error returned when an explicit channel or synchronization endpoint is closed.
- `WouldBlockError`: ordinary error returned by non-blocking synchronization probes such as `try_lock` and `try_acquire`.
- `Awaitable[T]`: structural protocol for values that can be awaited. `Coroutine[T, E]` implements an awaitable whose result follows the async function's surface return type; `Task[T, E]` implements `Awaitable[TaskResult[T, E]]`.
- `AsyncFunction[Params, T, E]`: the callable type of `async def`. The type checker must distinguish async callables from sync callables with the same parameters.
- `AsyncIterator[T, E]`: structural protocol for async iteration. `anext()` returns `Result[Option[T], E]`: `Ok(Some(value))` for the next item, `Ok(None)` for normal exhaustion, and `Err(E)` for stream failure.
- `AsyncClosable[E]`: structural protocol for async iterators that own cleanup work. `aclose()` returns `Result[None, E]`. General enough for streams, files, sockets, and database cursors; not tied to `GeneratorCloseError`.
- `AsyncGenerator[T, E]`: user-defined async producer created by an `async def` body that contains `yield`. `T` is the yielded item type and `E` is the ordinary error channel. Non-`None` async generator return values are rejected in v1; generator return values remain internal cleanup/finalization machinery.
- `GeneratorCloseError`: ordinary error returned when explicit async generator close fails.
- `GeneratorBusyError`: ordinary protocol error for reentrant async generator advancement.
- `BlockingTask[T, E]`: explicit handle for blocking offload. It is not a cooperative `Task[T, E]` because cancellation cannot forcibly stop already-running OS work.
- `Never`: bottom type used by `Task[T, Never]`, exhaustive matches, and unreachable control flow.

Async function lifting rules:

- `async def f(...) -> T` has async callable type `AsyncFunction[Params, T, Never]`; calling it returns `Coroutine[T, Never]`.
- `async def f(...) -> Result[T, E]` has async callable type `AsyncFunction[Params, T, E]`; calling it returns `Coroutine[T, E]`.
- `async def f(...) -> AsyncGenerator[T, E]` is the type of an async generator function. Calling it returns an unscheduled async generator object, not `Coroutine[AsyncGenerator[T, E], E]`.
- Nested results are not flattened beyond the outer async error channel: `async def f() -> Result[Result[A, E1], E2]` returns `Coroutine[Result[A, E1], E2]`.
- Calling an async function returns a linear `Coroutine[T, E]`; it does not run as a sync function and does not schedule itself.
- Calling an async generator function returns an `AsyncGenerator[T, E]`; it does not run until iterated, advanced with `anext()`, or consumed by an async comprehension.
- `AsyncFunction` is not a subtype of sync `Function`/`Callable`. Storing an async function in a sync callable variable, passing it where a sync callable is required, or invoking it through a sync-call path is a compile-time error.

Await rules:

- `await x` is valid only when `x` has an awaitable type.
- Awaiting a same-task coroutine consumes the coroutine and yields the async function's surface return type: `await Coroutine[T, Never] -> T`, and `await Coroutine[T, E] -> Result[T, E]`.
- Awaiting an `AsyncGenerator[T, E]` is invalid. Users consume it with `async for`, `anext()`, async comprehensions, or explicit close.
- `scope.spawn(Coroutine[T, E]) -> Task[T, E]`; spawning consumes the coroutine and schedules it as a child.
- `await Task[T, E]` always produces `TaskResult[T, E]` when the awaiting task is not itself actively cancelled.
- The `TaskResult.Err(Failure[E])` branch follows ordinary `Error` rules.
- The `TaskResult.Cancelled(Failure[CancellationError])` branch is task-control evidence that must be matched or converted explicitly.
- `try await` is valid for same-task coroutines that produce `Result[T, E]`.
- `try await task_handle` is rejected in the first model because task cancellation is not an ordinary error branch; users must `match await task_handle` and convert cancellation into an ordinary error intentionally when that is what the enclosing API wants.

`SecondaryError` is inspectable evidence attached to `Failure[E]`. It never masks the primary cause and does not participate in ordinary `except Error` matching unless a user explicitly inspects it.

**Same-task coroutine secondary evidence:** Secondary evidence produced inside a same-task coroutine is accumulated on the currently running task. Same-task `await Coroutine[T, E]` returns only `Result[T, E]`; accumulated secondary evidence becomes observable only when the current task is later observed through `TaskResult`, or through diagnostics/logging if the top-level task exits.

```sifr
struct Failure[E]:
    primary: E
    secondary: list[SecondaryError]

class TaskCancelled(Error):
    message: str

enum TimeoutResult[E]:
    Inner(E)
    Timeout(TimeoutError)
```

`TimeoutResult[E]` implements `Error` when `E: Error`. This makes it usable in ordinary error handlers.

```sifr
struct ScopeFailure:
    primary: ScopeFailureCause
    secondary: list[SecondaryError]

enum ScopeFailureCause:
    UnobservedChildFailed(error: Error, task_id: str)
    UnobservedChildCancelled(cause: CancellationError, task_id: str)

enum SecondaryError:
    CleanupFailed(error: Error, location: str)
    SiblingFailed(error: Error, task_id: str)
    CancellationDuringCleanup(cause: CancellationError)
```

`ScopeFailure` intentionally type-erases unobserved child errors to ordinary `Error` evidence. Explicitly awaited child handles preserve their typed `E` through `TaskResult[T, E]`; scope-exit failure is for cases where the child result was not otherwise consumed. Additional `SecondaryError` variants can be added by reviewed model amendment when implementation experience proves a new evidence class is needed.

## Core API Signatures

The model uses these shapes unless a future reviewed model amendment changes them.

```sifr
# Async function call and spawn
async def f() -> Result[T, E]
f() -> Coroutine[T, E]
scope.spawn(f()) -> Task[T, E]

# Same-task await
await Coroutine[T, Never] -> T
await Coroutine[T, E] -> Result[T, E]

# Task-handle observation
await Task[T, E] -> TaskResult[T, E]

enum TaskResult[T, E]:
    Ok(T)
    Err(Failure[E])
    Cancelled(Failure[CancellationError])

async def Task[T, E].join(self) -> TaskResult[T, E]
def Task[T, E].cancel(self) -> None
async def Task[T, E].cancel_and_join(self) -> TaskResult[T, E]
```

`Task[T, E]` is an affine observer handle:

- dropping a handle does not cancel or detach the child task
- awaiting a handle is syntactic sugar for `await handle.join()`
- `await handle`, `join()`, `cancel_and_join()`, `gather`, `select`, `race`, and `timeout` consume the handle
- `cancel()` borrows the handle to request cancellation; the handle may then be awaited or joined to observe cleanup
- `cancel()` returns immediately, is a no-op after task completion, and repeated calls are no-ops
- `Task[T, E]` is not clonable in v1; a future shared-observer surface would be a separate type
- a consumed handle is invalid for further observation

Task composition APIs consume the task handles passed to them. Consumed handles are no longer usable by the caller.

```sifr
task.spawn_scoped[T, E](coro: Coroutine[T, E], *, ctx: Option[task.Context] = None) -> Task[T, E]

task.gather(handles: list[Task[T, E]]) -> TaskResult[list[T], E]

task.select[A, EA, B, EB](*, branch_a: Task[A, EA], branch_b: Task[B, EB]) -> Select2[TaskResult[A, EA], TaskResult[B, EB]]

enum Select2[A, B]:
    First(A)
    Second(B)

task.race(handles: list[Task[T, E]]) -> TaskResult[T, E]

task.timeout(handle: Task[T, E], duration: Duration) -> TaskResult[T, TimeoutResult[E]]
```

`branch_a` and `branch_b` are signature placeholders. Source code supplies concrete unique keyword labels, such as `task.select(first=fast, second=slow)`, and the current binary result container maps the first supplied branch to `Select2.First` and the second supplied branch to `Select2.Second`.

Timeout translates cancellation caused by the deadline into `TaskResult.Err(Failure[TimeoutResult.Timeout(TimeoutError)])`. If the child fails before the deadline, the result is `TaskResult.Err(Failure[TimeoutResult.Inner(E)])`. If the child was already externally cancelled before the deadline wins, the result remains `Cancelled(Failure[CancellationError])`. If the deadline and inner completion become ready in the same scheduler tick, inner completion wins.

Channel endpoints are explicit so close and backpressure semantics are unambiguous:

```sifr
sync.channel[T]() -> (sync.ChannelSender[T], sync.ChannelReceiver[T])
sync.bounded_channel[T](capacity: int) -> (sync.ChannelSender[T], sync.ChannelReceiver[T])

async def ChannelSender[T].send(value: T) -> Result[None, ClosedError]
def ChannelSender[T].close() -> None

async def ChannelReceiver[T].receive() -> Result[T, ClosedError]
```

`ClosedError` from `receive` means the channel is closed and drained. There is no separate `None` end-of-stream state in the first model.

`ChannelReceiver[T]` implements `AsyncIterator[T, Never]` by mapping a closed-and-drained `ClosedError` from `receive()` to `Ok(None)`. Direct `receive()` exposes `ClosedError` so manual receive loops can distinguish terminal close from item delivery without an `Option`; `async for` uses the async iteration protocol. Cancellation during `sender.send(value)` is exactly-once: the value is either not enqueued and dropped, or enqueued exactly once. It is never duplicated.

Async iteration uses `Option` for exhaustion and `Result` for stream failure:

```sifr
protocol AsyncIterator[T, E]:
    async def anext(self) -> Result[Option[T], E]

protocol AsyncClosable[E]:
    async def aclose(self) -> Result[None, E]

def anext[T, E](iterator: AsyncIterator[T, E]) -> Awaitable[Result[Option[T], E]]
```

Normal exhaustion is `Ok(None)`, not an exception-like sentinel. A stream failure is `Err(E)` and participates in ordinary Sifr error handling. `async for` desugars through this protocol and must handle the `Err(E)` channel according to the surrounding function's result/try rules. Async iterators that own cleanup work implement `AsyncClosable`; `AsyncGenerator` implements `AsyncClosable`.

Async generator functions produce async iterator objects directly:

```sifr
async def stream_lines(path: str) -> AsyncGenerator[str, IOError]:
    file = try await open_async(path)
    async with file:
        async for line in file.lines():
            yield line
```

An `async def` body that contains `yield` is an async generator function. Calling it returns `AsyncGenerator[T, E]`; it does not create a coroutine and is not awaitable. The compiler rejects `await stream_lines(path)` and suggests `async for`, `anext()`, or an async comprehension.

`AsyncGenerator[T, E]` implements `AsyncIterator[T, E]` and `AsyncClosable[GeneratorCloseError]`. `T` is inferred from all yielded values and must converge to one yield type. `E` is inferred from fallible async operations inside the generator body and from the declared return/error surface. Public v1 async generators do not expose generator return values; non-`None` return values from async generators are rejected at compile time.

Async generator cancellation and close use the same typed cancellation model as tasks:

- `agen.aclose()` requests generator close, injects generator-close control, runs `finally` blocks and async context cleanup, then completes.
- cancellation while an async generator is suspended or running unwinds the generator before the consuming task completes cancellation.
- generator-close control is not an ordinary `Error` and is not caught by broad `except Error`.
- cleanup failures become `SecondaryError` evidence attached to the owning cancellation/failure result.
- yielding after close has begun is a compile-time or runtime protocol error surfaced as a typed diagnostic/error, not a panic.

```sifr
async def AsyncGenerator[T, E].aclose(self) -> Result[None, GeneratorCloseError]
```

When `aclose()` is called explicitly, successful cleanup returns `Ok(None)`. If cleanup fails during explicit close, `aclose()` returns `Err(GeneratorCloseError)` as the primary result. When cleanup fails during cancellation or timeout, the cleanup failure becomes `SecondaryError` evidence attached to the owning cancellation/failure result instead of replacing the primary cancellation or timeout.

The first `anext()` call after `aclose()` has begun returns `Ok(None)`. If the generator was already exhausted before close began, `aclose()` runs cleanup and the final `anext()` still returns `Ok(None)`. Calling `anext()` while a `finally` block or async context-manager cleanup is executing waits until cleanup completes, then returns the final state. `Ok(None)` therefore covers both normal end and close end; callers that need to distinguish them must track generator state explicitly or use a higher-level abstraction. The first model does not add a `GeneratorClosedError` variant.

`AsyncGenerator` is single-consumer and non-reentrant in v1. Calling `anext()` while a previous `anext()` is still pending is a protocol error reported as `GeneratorBusyError` where static analysis cannot reject it earlier. Sifr must not silently queue concurrent `anext()` calls; generators are not channels.

`AsyncGenerator[T, E]` is sendable when all captured values and generated state-machine fields are sendable. Mutable borrows, unsynchronized interior mutability, and captured mutable references make the generator non-sendable. Passing a non-sendable async generator across a `scope.spawn` boundary is rejected at the spawn site with the same task-boundary diagnostics as any other non-sendable value.

The first model supports these async comprehension forms:

```sifr
lines: list[str] = [line async for line in stream_lines(path)]
unique: set[str] = {line async for line in stream_lines(path)}
lookup: dict[str, int] = {item.key: item.value async for item in stream_items(path)}
```

List, set, and dict async comprehensions eagerly consume the async iterable in the current task and propagate `Err(E)` through ordinary Sifr error handling. Lazy async generator expressions are deferred in v1 because they add parser, HIR, and lifetime complexity without being required to prove the first async model.

The first async comprehension model supports a single `async for` clause with ordinary synchronous `if` filters. Nested async comprehensions, `await` inside comprehension filters, async generator expressions, and passing async generator expressions directly as function-call arguments are deferred until the HIR and lifetime rules for those surfaces are proven.

When an async comprehension is abandoned or cancelled, it closes the active async iterator it is consuming when that iterator implements `AsyncClosable`. Eager comprehensions cancel at the same cancellation point as a manual `async for` loop over the same source.

## Runtime Model

The runtime substrate is generated, not user-managed.

The compiler detects async usage, wires the required runtime dependency, generates the correct entrypoint bootstrap, and rejects invalid async usage before Rust compilation where possible. Public Sifr APIs do not expose Tokio or runtime-specific implementation types. Runtime internals stay behind private implementation boundaries.

The implementation may use Tokio or a compatible runtime substrate, but ordinary Sifr users do not configure a runtime directly. Multiple runtime instances in one generated binary, custom runtime injection, and runtime tuning are outside the primary model.

## Task Scope And Lifecycle

`scope.spawn` is the canonical task creation API. Free-floating detached spawn is not exposed.

`TaskScope` uses nursery ownership:

- every spawned child belongs to the scope
- handles returned by `scope.spawn` are observers, not owners
- dropping a handle does not detach or cancel the child
- on normal exit, `TaskScope.__aexit__` waits for all children
- on abnormal exit, `TaskScope.__aexit__` cancels unfinished children and waits for cleanup
- child failures that are not explicitly observed are surfaced at scope exit as structured scope failure evidence, never silently discarded
- no task handle may escape its owning task scope silently

`TaskScope.__aexit__` returns `Result[None, ScopeFailure]`. A normal scope exit with all children successful returns `Ok(None)`. Explicit observation means `await handle`, `handle.join()`, `gather`, `select`, `race`, or `timeout` consumes the handle and marks the child result as observed. If a fallible or cancelled child is still unobserved at scope exit, scope exit returns `Err(ScopeFailure(...))`. This keeps nursery ownership from becoming silent failure loss.

`TaskGroup` adds sibling-failure policy on top of task scopes. A plain `TaskScope` owns lifetime; a `TaskGroup` owns group error behavior and cancels unfinished siblings when any child completes with failure.

**Sibling cancellation observation rule:** A `TaskGroup` internally observes policy-triggered sibling cancellations. They do not produce `ScopeFailure` merely because the user did not await every cancelled sibling. If an explicitly observed child failed and triggered sibling cancellation, and the failing child result was already consumed by the user, the group's cancellation of remaining siblings is an internally observed policy action. Cleanup failures from those siblings attach as `SecondaryError` to the group failure if a group failure exists; otherwise they surface as `ScopeFailure`.

```sifr
async with task.TaskGroup[MyError]() as group:
    a = group.spawn(fails())   # explicitly observed by user below
    b = group.spawn(slow())

    match await a:            # user explicitly observes a's failure
        Err(failure):
            handle(failure.primary)
        Ok(_):
            pass
        Cancelled(c):
            pass
    # group already cancelled b due to a's failure
    # b's cancellation is internally observed by the group
    # group exit returns Ok(None) if a was the only failure and b cleaned up normally
    # if b's cleanup fails, it attaches as SecondaryError to any existing group failure
```

`TaskGroup.spawn` returns the same affine observer handle as `scope.spawn`. V1 task groups require all spawned children to share one ordinary error type `E`; heterogeneous task groups are deferred. `Never` coerces into `E` for children that cannot fail. When one child fails, the group requests cancellation of all unfinished siblings and waits for their cleanup before group exit completes. If the failing child was not explicitly observed, group exit returns `Err(ScopeFailure(...))` with the first failed child as primary evidence and cleanup or later sibling failures as secondary evidence.

**`TaskGroup` and `TaskScope` closed/cancelling spawn rules:** A `TaskGroup` has `Open`, `Cancelling`, `Closing`, and `Closed` states. `group.spawn(...)` is valid only in `Open` and returns `Task[T, E]`, not a fallible union. V1 treats group openness as a flow-checked capability: after child failure or cancellation is observed, explicit group cancellation or timeout occurs, or scope exit begins, later `group.spawn(...)` on that control path is rejected unless the compiler can prove the group is still `Open`. The same principle applies to `TaskScope`: once `__aexit__` begins, spawning is invalid. A future fallible spawn API would be a separate surface rather than changing `TaskGroup.spawn`.

General tracked-collection proof is not part of the first model. Handles may be consumed by explicit composition APIs (`gather`, `select`, `race`) or by simple explicit loops such as `for h in handles: await h`; the scope still owns child lifetime regardless of handle observation.

## Task Composition

`task.gather` is fail-fast:

- successful completion returns `TaskResult.Ok(list[T])` with results in input order
- the first observed child error cancels unfinished children and returns `TaskResult.Err(Failure[E])`
- after cancellation cleanup, the earliest failed handle in input order is the primary error if multiple failures surface
- later sibling failures and cleanup failures are recorded as `SecondaryError` values on the primary `Failure[E]`
- collect-all semantics require a future separate API
- if a gathered child is observed as `Cancelled(Failure[CancellationError])` before an ordinary child error is selected as primary, gather cancels unfinished siblings and returns `TaskResult.Cancelled(Failure[CancellationError])`. If cancellation and ordinary errors are both observed during the same drain, deterministic input order chooses the primary among failure-like outcomes; the rest become `SecondaryError` evidence.

`task.select` and `task.race` are first-completion APIs:

- they consume input handles
- `task.select` uses named keyword branches so branch identity is visible at the call site; the current binary runtime container maps declaration order to `Select2.First` and `Select2.Second`
- `task.race` operates on a homogeneous collection and owns winner/cancellation evidence for the collection
- losing tasks are cancelled by default
- loser handles cannot be awaited or joined after the selection API owns them
- if multiple tasks complete in the same scheduler tick, input order breaks ties deterministically
- users who need all results should use `gather`
- users who need non-cancelling competition must keep explicit handles and perform explicit cleanup through a future API

**Loser cleanup failure handling:** If the selected winner result is `Err(...)` or `Cancelled(...)`, any loser cleanup failures attach as `SecondaryError` evidence to that result. If the selected winner result is `Ok(...)`, loser cleanup failures surface at the owning `TaskScope` exit as `ScopeFailure` rather than being dropped. Same-tick cases where one ready task wins by input order but another ready loser has already failed follow the same rule.

## Timeout Semantics

`task.timeout(handle, duration)` accepts task handles. Arbitrary awaitables are not accepted directly; users spawn them into a child task first.

Timeout behavior:

- if the inner task succeeds before `duration`, timeout returns `TaskResult.Ok(T)`
- if the inner task fails before `duration`, timeout maps the ordinary failure to `TaskResult.Err(Failure[TimeoutResult.Inner(E)])`
- if the inner task is cancelled before `duration`, timeout preserves `Cancelled(Failure[CancellationError])`
- if `duration` expires first, timeout cancels the inner task, waits for cleanup, and returns `TaskResult.Err(Failure[TimeoutResult.Timeout(TimeoutError)])`
- if inner completion and timeout expiry become ready in the same scheduler tick, inner completion wins
- cancelling the outer scope while timeout is running cancels the inner task unconditionally
- cleanup failures after timeout cancellation become secondary evidence on the timeout failure

`task.timeout(duration)` is the async context-manager form used for inline blocks. It is a compiler-recognized cancellation scope, not an ordinary user-defined context manager. The compiler lowers the block into a same-task cancellation scope using internal delimited cancellation: deadline expiry sets an internal cancellation flag, cooperative await points observe it, and cleanup runs normally before scope exit completes.

Timeout context blocks do not introduce a spawn boundary and can access surrounding locals naturally. `await` and `try await` inside the block follow the normal same-task rules. If code inside the block spawns child tasks, those children follow the ordinary `scope.spawn` task-boundary rules. Nested timeout context blocks are allowed and compose as inner-first cancellation.

If the block finishes before the deadline, scope exit returns `Ok(None)`. If the deadline wins, the internal cancellation flag causes cooperative unwinding, cleanup runs, and scope exit returns `Err(TimeoutError)` through the ordinary error channel. The deadline is not materialized as `Cancelled(Failure[CancellationError])`; timeout is an ordinary fallible operation that must be handled by `try`/`except` or by an enclosing `Result` type that can carry `TimeoutError` or `Error`. Outer cancellation remains active cancellation and cancels the inner scope unconditionally.

## Ownership And Borrowing

`scope.spawn` requires captures and return values to satisfy task-boundary requirements. Detached spawn is not exposed; a future detached spawn must require explicit owned, sendable, static captures.

Borrow rules at async boundaries:

| Value form | Across `await` in same task | Across `scope.spawn` |
| --- | --- | --- |
| immutable borrow | allowed only when the borrow remains valid and no conflicting mutation exists | deferred in v1; v1 `scope.spawn` requires owned, sendable, static captures |
| mutable borrow | rejected when it would remain live across `await` | rejected; use explicit synchronization or ownership transfer |
| owned value | allowed | allowed when the type is sendable across task boundaries |
| `sync.Shared[T]` | allowed for immutable shared data | allowed when `T` satisfies the share/send requirements |
| unsynchronized mutable state | rejected | rejected |

Spawned tasks require owned, sendable, static task boundaries in the first model. Ordinary awaited coroutines within the same task do not introduce a spawn boundary. Local non-send task sets and scoped borrowed spawn are deferred. Scoped borrowed spawn is conceptually valid, but it requires a runtime strategy that polls child futures inside the parent scope rather than plain `'static` runtime spawn.

Sendability is derived structurally for ordinary values crossing `scope.spawn`. Built-in scalar and owned collection values are sendable when their component types are sendable. User classes are sendable when all stored fields are sendable and the class does not inherit the zero-runtime `NonSend` marker. `NonSend` is a source-level type fact for local executor state and other intentionally thread-local values; it is not emitted as a runtime parent field.

## Synchronization Primitives

The compiler does not silently turn local state into shared state. Shared memory and coordination are explicit.

`sync.Shared[T]` exposes immutable shared ownership. It requires `T` to satisfy the `ShareSafe` capability:

- `T` must be `Send + Sync`
- `T` must not contain unsynchronized interior mutability
- types with their own synchronization may satisfy `ShareSafe`
- `Shared[Cell[int]]` and `Shared[list[MutableThing]]` are rejected

`sync.Lock[T]` and `sync.RwLock[T]` provide explicit mutable sharing. They use synchronous Rust mutex primitives in the first model. Acquiring one in async code may block the current runtime worker under contention, so they are permitted only for short, low-contention critical sections. Channels are preferred for async coordination. Distinct `sync.AsyncMutex[T]` and `sync.AsyncRwLock[T]` surfaces are deferred until a later semantic record defines await-safe guard semantics.

Lock guard rules:

- `lock()` is not await-aware and returns a guard restricted to a synchronous lexical scope
- lock guards must not cross `await` points
- a live `LockGuard` or `RwLockGuard` at an `await` point is a compile-time error
- lock guards cannot cross task boundaries

Channels are the canonical queue-like concurrency primitive:

- `sync.channel[T]()` returns `(ChannelSender[T], ChannelReceiver[T])`
- `sync.bounded_channel[T](capacity)` returns `(ChannelSender[T], ChannelReceiver[T])`
- `ChannelSender[T]` is clonable; `ChannelReceiver[T]` is single-consumer in the first model
- `await sender.send(value)` on a closed channel returns `Result[None, ClosedError]`
- `await receiver.receive()` returns `Result[T, ClosedError]`
- `ClosedError` from `receive` means closed and drained; there is no second `None` closed state
- `sender.close()` wakes pending senders and receivers deterministically
- cancellation while blocked on send or receive propagates task cancellation without duplicating or losing a message
- cancellation while blocked on send is exactly-once: the value is either not enqueued and dropped, or enqueued exactly once
- cancellation while blocked on receive is exactly-once: if a receive is cancelled before `Ok(value)` is returned to user code, the message remains available to another receive or is otherwise not lost. Once `Ok(value)` has been returned, ownership has transferred to the receiver task.
- bounded channels apply async backpressure when full
- generated channel runtime uses explicit wakeups for sender capacity, receiver availability, close, and endpoint-drop events; sender and receiver wait loops register `Notify` interest before state checks and do not use polling/yield loops for backpressure

**Channel endpoint lifetime rules:**
- Dropping the last sender closes the channel after buffered messages drain.
- Dropping the receiver closes the channel immediately to senders.
- Calling `close()` on any sender closes the whole channel to future sends.
- Buffered messages remain receivable after close.
- Messages are received in channel enqueue order (FIFO).

`sync.Semaphore` and `sync.Notify` cover common coordination patterns. `SemaphorePermit` is a guard-like resource: a live permit at an `await` point is a compile-time error, and permits cannot be returned from a function. `Notify` is edge-triggered; level-triggered `Event` behavior requires explicit state such as `sync.Shared[bool] + Notify` in the first model. Public `Barrier` is deferred and public `Once` remains internal-only unless a later design record records a production need.

## Blocking And Thread Offload

Async tasks are for waiting and cooperative scheduling. CPU-bound work and blocking OS calls must use explicit offload.

`@blocking_io` and `@cpu_heavy` are declaration-site workload classification annotations for synchronous functions. They never imply automatic task or thread scheduling.

- `@blocking_io`: the sync function performs I/O that can block an OS thread, such as file read/write, network I/O, database calls, pipe operations, or blocking timer waits. Calling a known `@blocking_io` function from an `async def` body is an error in the sealed async model: use an async API if available, or wrap the call with `spawn_blocking`.
- `@cpu_heavy`: the sync function is CPU-intensive, such as cryptography, compression, hashing, parsing, numerical compute, or computation-heavy processing. Calling a known `@cpu_heavy` function from an `async def` body is an error in the sealed async model: use `task.spawn_cpu` or the accepted `sifr.parallel` APIs to avoid starving the runtime.

The stdlib maintains a built-in annotation database for stdlib functions. User code can annotate sync declarations with `@blocking_io` or `@cpu_heavy`. Unannotated user functions are assumed to be cheap compute and do not warn by default; this avoids making every short helper look like a scheduler problem. External/FFI calls are treated conservatively as potentially blocking in async contexts unless a future FFI rules classifies them more precisely.

These annotations guide diagnostics and offload validation. The compiler must not silently rewrite either call.

## Async Effect Discipline

Async functions must be async for a real reason. The compiler tracks an internal suspension summary for async bodies. The exact internal enum is not public API, but the semantic categories are:

- `NoSuspend`: no operation in the body can suspend.
- `AsyncIo`: awaiting a native async I/O operation or an async API with transitive I/O wait.
- `TimerWait`: awaiting sleep, timeout, or timer-backed scheduling.
- `ChannelWait`: awaiting channel send/receive or async iteration over a channel-backed stream.
- `TaskWait`: awaiting task handles, blocking task handles, task composition APIs, task scope/group cleanup, or same-task coroutines with a non-empty suspension summary.
- `AsyncResourceWait`: awaiting async context-manager enter/exit, async iterator advancement, or async cleanup.
- `GeneratorSuspend`: suspension at an async generator `yield`, or an async generator await with a non-empty suspension summary.

`async def` with a `NoSuspend` summary is rejected. The user should write `def` instead. Async protocol conformance may require an async-shaped method with no current suspension, but that must use an explicit reviewed escape hatch with a reason; the compiler must not silently accept fake async functions.

`await` remains valid only for awaitable values. Awaiting a non-awaitable value, including the result of a sync function call, is a hard error. In addition, awaiting a same-task coroutine whose transitive suspension summary is `NoSuspend` is rejected because the callee is async in shape only.

`@blocking_io` and `@cpu_heavy` do not create async effects and do not make sync functions awaitable. They are sync workload facts. Applying either annotation to `async def` is an error; async APIs receive suspension summaries such as `AsyncIo`, not sync workload annotations. Calling a known `@blocking_io` or `@cpu_heavy` function directly from an `async def` body is an error in the sealed async model. Users must choose a native async API, `task.spawn_blocking`, `task.spawn_cpu`, or accepted `sifr.parallel` APIs.

`task.spawn_blocking(fn)` requires classified sync work. The target function must be annotated `@blocking_io` or `@cpu_heavy`, known by the stdlib annotation database as blocking or CPU-heavy, or known by an external/FFI rules as blocking or CPU-heavy. Unannotated cheap sync helpers are rejected as offload targets; the diagnostic should say to call them directly, or annotate the declaration if it is genuinely blocking or expensive.

`spawn_blocking` on `@blocking_io` work is valid and should not warn by default. A later informational diagnostic may suggest a specific native async replacement only when the compiler knows one.

`task.spawn_blocking` provides explicit blocking-work offload:

```sifr
task.spawn_blocking(fn: Fn() -> Result[T, E]) -> BlockingTask[T, E]
```

- blocking work returns typed results
- cancelling `task.spawn_blocking` work requests cancellation and drops/abandons the handle result
- the first model does not forcibly abort a running OS thread
- already-running blocking work may continue to completion, but its result is discarded after cancellation
- `spawn_blocking` requires owned, sendable, static captures
- scoped borrowed captures are rejected for `spawn_blocking` because already-running OS work may outlive the async scope after cancellation
- hard interruption requires future process isolation and typed IPC

`BlockingTask[T, E]` is distinct from cooperative `Task[T, E]` because cancellation means result abandonment, not guaranteed work stoppage. `BlockingTask.cancel()` requests cancellation and marks the result as abandoned if the work cannot stop cooperatively. `BlockingTask.join()` returns `TaskResult[T, E]`; a `Cancelled(Failure[CancellationError])` branch means the observer abandoned the result after cancellation, even if the OS work later completed.

**`BlockingTask` lifecycle:** `BlockingTask` handles are affine. `join()` and `cancel_and_join()` consume them. Dropping a `BlockingTask` handle abandons observation but does not stop already-running OS work. Blocking work requires owned/sendable/static captures precisely because it may outlive the async scope after abandonment. Scope exit requests cancellation/abandonment for unresolved blocking work created inside the scope but does not guarantee OS-thread interruption.

`sifr.threading` and `sifr.concurrent` are not public compatibility veneers. Coordination uses native `sifr.sync` primitives, blocking work uses `task.spawn_blocking`, and CPU parallelism uses accepted `sifr.parallel` APIs.

## Async Resource Protocols

`async with` is part of the user-facing async model.

`task.scope()` and `task.timeout(duration)` use async context-manager behavior. General user-defined async context managers follow the same cleanup rules:

- async enter/exit protocol methods are awaited
- cleanup order is LIFO
- cancellation inside `async with` unwinds active async context managers
- async exit receives the cancellation cause
- async exit cleanup runs to completion unless the runtime is forcefully aborted by an unrecoverable system failure
- errors from async exit during cancellation become `SecondaryError` evidence attached to the owning task/scope result
- panic-like failures from async exit are caught at task/runtime boundaries where technically possible and surfaced as structured failure evidence

**User-defined async context manager protocol:**

```sifr
protocol AsyncContextManager[T, EnterE, ExitE]:
    async def __aenter__(self) -> Result[T, EnterE]
    async def __aexit__(self, cause: AsyncExitCause) -> Result[None, ExitE]
```

`__aenter__` and `__aexit__` are the async context-manager methods. If `__aenter__` fails, `__aexit__` is not called, because the resource was not acquired.

Implementation status: the initial async context-manager and iterator compiler capability supports the normal-exit path for user-defined async context managers that structurally provide async `__aenter__` and `__aexit__` methods. Abnormal body exit, cancellation-specific causes, secondary cleanup evidence, and `async for` cleanup remain deferred cleanup cases in the same implementation track.

```sifr
enum AsyncExitCause:
    Normal
    Return
    OrdinaryError(Error)
    Timeout(TimeoutError)
    Cancellation(CancellationError)
    RuntimeFault(...)
```

`AsyncExitCause` is passed to `__aexit__` so the async context manager can distinguish normal exit from error/timeout/cancellation paths. `RuntimeFault` covers unrecoverable runtime failures; cleanup runs best-effort and the fault remains primary.

`async for` works for async iterable values such as channel-backed streams and user-defined async generators. Async comprehensions are surface syntax over the same protocol; they do not introduce a second iteration model.

### Control-Flow Desugaring

#### Fallible `async with` Exit Propagation

Fallible async context managers expose an exit error type `ExitE`. For `TaskScope` and `TaskGroup`, `ExitE` is `ScopeFailure`; for `task.timeout(duration)`, it is `TimeoutError`; user-defined async context managers choose their own ordinary `Error` type. The interaction with body result, return value, cancellation, and exit error follows this exact rule:

| Body outcome | Exit outcome | Final result |
|---|---|---|
| Body completes normally, exit succeeds | `Ok(None)` | body value or normal fallthrough |
| Body completes normally, exit fails | `Err(ExitE)` | exit failure is primary |
| Body performs explicit `return`, exit succeeds | `Ok(None)` | return proceeds |
| Body performs explicit `return`, exit fails | `Err(ExitE)` | exit failure is primary; the return is not performed |
| Body propagates ordinary `Err(E)`, exit succeeds | `Ok(None)` | body `Err(E)` propagates |
| Body propagates ordinary `Err(E)`, exit fails | `Err(ExitE)` | body `Err(E)` remains primary; exit failure is secondary evidence at the task/scope observation boundary |
| Body is actively cancelled or times out, exit succeeds | `Ok(None)` | cancellation or timeout remains primary |
| Body is actively cancelled or times out, exit fails | `Err(ExitE)` | cancellation or timeout remains primary; exit failure is secondary evidence |
| Body hits unrecoverable runtime fault, exit fails | best-effort cleanup | runtime fault remains primary at the runtime boundary |

**Key rules:**
- A fallible async context manager in a non-fallible function is rejected unless the exit failure is handled locally.
- When the body is actively cancelled or timed out, that control cause takes precedence over exit failure.
- During ordinary error propagation, the body error remains primary; exit failure becomes secondary evidence instead of replacing the body error.
- Unrecoverable runtime faults remain outside ordinary `Error` handling, but generated/runtime boundaries still run best-effort cleanup before surfacing the fault.

#### `async for` Desugaring

```sifr
async for item in source:
    body
```

desugars to:

```sifr
loop:
    next = try await anext(source)
    match next:
        Some(item):
            body
        None:
            break
```

`async for` is fallible when the iterator is fallible:
- `anext()` returning `Ok(Some(item))` yields the item and continues the loop.
- `anext()` returning `Ok(None)` breaks normally (normal exhaustion).
- `anext()` returning `Err(E)` causes automatic propagation through ordinary Sifr error handling. The enclosing function must be able to carry `E`, or the compiler rejects the `async for`.

If an early-exit path from `async for` may call `aclose()`, the enclosing function must be able to propagate the iterator's close error type, or the close error must be handled explicitly. This applies to both the `IterE` error from `anext()` and the `CloseE` error from `aclose()` on early exit.

If the loop exits before iterator exhaustion via `break`, `return`, ordinary error propagation, timeout, or active cancellation, and the iterator implements `AsyncClosable`, the compiler awaits `aclose()` before leaving the loop:
- On normal `break` or `return`, `aclose()` failure is a primary ordinary error.
- During cancellation or timeout, `aclose()` failure is secondary evidence attached to the owning cancellation/failure result.

Users who want to handle iterator errors without propagation must use the explicit form:

```sifr
loop:
    next = await anext(source)
    match next:
        Ok(Some(item)):
            body
        Ok(None):
            break
        Err(e):
            match e:
                # explicit error handling here
            break
```

## Removed Compatibility Veneers

CPython-shaped compatibility layers are removed or diagnosed. The native replacement table is a migration aid, not an importable adapter rules.

| Removed legacy module | Native Sifr direction |
| --- | --- |
| `sifr.asyncio` | `sifr.task` and `sifr.sync` |
| `sifr.concurrent` / `sifr.concurrent.futures` | `sifr.runtime` and `sifr.parallel` |
| `sifr.threading` | `sifr.sync`, `sifr.runtime`, and scoped offload |
| `sifr.queue` | `sifr.sync` |
| `sifr.subprocess` | `sifr.process` |
| `sifr.multiprocessing` | future `sifr.ipc` design gates |

Unsupported compatibility surfaces are intentionally absent or diagnosed: raw event loops, loop policies, transports/protocols, public selectors, `contextvars`, process pools, signals shaped like CPython handlers, and raw callback-first APIs.

## Diagnostics Rules

Async diagnostics are Sifr-native and stable. Rust compiler errors may be used as implementation evidence, but they must not be the primary user experience for covered cases.

Diagnostic families cover:

- invalid async syntax/use
- `await` outside async
- awaiting non-awaitable values
- async functions with no real suspension effect
- awaiting same-task coroutines with no transitive suspension effect
- async calls from sync callable paths
- `try await` on task handles
- task-boundary Send/Sync failure
- borrow-across-await failure
- borrowed values escaping task boundaries
- consumed task handle reuse
- unobserved child failure at scope exit
- detached-task capture failure if detached tasks are added later
- cancellation misuse
- timeout scope failure not handled by surrounding error type
- `@blocking_io`, `@cpu_heavy`, or potentially blocking FFI call in async context
- unclassified functions passed to `spawn_blocking`

New diagnostic codes for the async effect seal (`SIFR-ASYNC-*`):

- `SIFR-ASYNC-0001`: `async def` body has no real suspension effect (transitive `NoSuspend`).
- `SIFR-ASYNC-0002`: awaiting a same-task coroutine whose transitive suspension summary is `NoSuspend`.
- `SIFR-ASYNC-0003`: direct `@blocking_io` call from async context.
- `SIFR-ASYNC-0004`: direct `@cpu_heavy` call from async context.
- `SIFR-ASYNC-0005`: `spawn_blocking` target is unannotated and not classified by stdlib/FFI rules.
- `SIFR-ASYNC-0006`: `@blocking_io` or `@cpu_heavy` applied to `async def`.
- lock guard live at an `await` point
- invalid async protocol implementation
- invalid async generator use, including `await` on an async generator
- inconsistent async generator yield types
- live mutable borrow across an async generator `yield`
- reentrant async generator `anext()`
- async generator explicit close failure
- unsupported async generator controls such as `send()`, `throw()`, or async `yield from`
- unsupported async comprehension shapes
- runtime-specific type leakage into public APIs

## Model Invariants

These decisions are part of the first async/concurrency model:

1. `scope.spawn` is the canonical task creation API. Free-floating detached spawn is not exposed.
2. Active task cancellation is scope-exit semantics and is not catchable by ordinary `except Error`.
3. `CancellationError` is not an `Error` subclass and is never matched by broad `except Error`.
4. `await Task[T, E]` always produces `TaskResult[T, E]` when observed by a non-cancelled task.
5. `try await task_handle` is rejected; task cancellation requires explicit matching and intentional conversion into an ordinary error when needed.
6. `TaskScope` owns children; task handles are observers, not owners.
7. `task.select` and `task.race` consume input handles and cancel losing tasks by default.
8. Channels use explicit sender/receiver endpoints. Send and receive are async operations; receive returns `Result[T, ClosedError]` with no second closed state.
9. Lock guards must not cross `await` points.
10. Spawned tasks require sendable task boundaries. Local, non-send task sets are deferred.
11. CPython-shaped compatibility veneers must not be publicly importable.
12. Public selectors, `contextvars`, multiprocessing, process pools, raw event loops, and transport/protocol APIs are deferred.
13. `ProcessPoolExecutor` is blocked on the future typed IPC/serialization rules.
14. `@blocking_io` and `@cpu_heavy` are declaration-site workload classification annotations for sync functions. They power diagnostics and offload validation but never trigger implicit scheduling. The stdlib ships with a pre-annotated database of known stdlib functions.
15. Async functions must have a real suspension effect. `async def` with no suspension is rejected unless an explicit reviewed protocol-conformance escape hatch applies.
16. Awaiting a same-task coroutine with no transitive suspension effect is rejected.
17. Direct `@blocking_io` or `@cpu_heavy` sync calls from async code are errors in the sealed model; cheap unannotated sync helper calls remain allowed.
18. `task.spawn_blocking` requires classified `@blocking_io`, `@cpu_heavy`, stdlib-known, or external-rules-known blocking/CPU-heavy work.
19. Subprocess and signal APIs require a later model amendment.
20. Public cancellation suppression, shielding, and counters are deferred. Runtime cleanup uses the required bounded cleanup budget.
21. `async def` with `yield` creates `AsyncGenerator[T, E]`, not `Coroutine[AsyncGenerator[T, E], E]`.
22. `AsyncGenerator[T, E]` is an `AsyncIterator[T, E]` and is not awaitable.
23. Async iteration exhaustion is `Ok(None)` through `Result[Option[T], E]`; stream failure remains `Err(E)`.
24. Async generator close and cancellation run `finally` blocks and async context cleanup before termination.
25. Async comprehensions are protocol sugar over `async for`; they must not introduce hidden task creation or detached work.
26. Async generator `send()`, `throw()`, async `yield from`, async generator expressions, nested async comprehensions, and awaited comprehension filters are deferred.
27. `TaskScope.__aexit__` returns `Result[None, ScopeFailure]`; unobserved child failure or cancellation must be surfaced, never dropped.
28. Task handles are affine. `await Task[T, E]`, `join()`, `cancel_and_join()`, `gather`, `select`, `race`, and `timeout` consume the handle. Task handles are not clonable in v1.
29. `TaskGroup[E]` requires homogeneous child error type `E` in v1 and cancels unfinished siblings on first child failure.
30. `task.timeout(handle, duration)` returns `TaskResult[T, TimeoutResult[E]]`; timeout is an ordinary timeout failure, not child cancellation evidence.
31. `async with task.timeout(duration)` is a compiler-recognized timeout scope whose deadline failure exits with ordinary `TimeoutError`.
32. V1 `scope.spawn` requires owned, sendable, static captures. Scoped borrowed spawn and local non-send task sets are deferred.
33. Public v1 `AsyncGenerator` is `AsyncGenerator[T, E]`; non-`None` async generator return values and lazy async generator expressions are deferred.
34. `AsyncGenerator` is single-consumer and non-reentrant in v1.
35. `TaskCancelled` is the canonical ordinary `Error` wrapper when a caller intentionally converts materialized child cancellation into its own error channel.
