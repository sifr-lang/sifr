# Async and Concurrency Model Contract

Status: canonical async and concurrency model contract
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

        match await first:
            Ok(a):
                pass
            Err(failure):
                return Err(failure.primary)
            Cancelled(cancelled):
                return Err(task.ChildCancelled(cancelled.primary))

        match await second:
            Ok(b):
                pass
            Err(failure):
                return Err(failure.primary)
            Cancelled(cancelled):
                return Err(task.ChildCancelled(cancelled.primary))

        print(a + b)
        return Ok(None)
```

The user-facing vocabulary is:

- `async def`
- `await`
- `sifr.task`
- `sifr.sync`
- scoped task groups
- explicit channels and locks
- explicit blocking/thread offload
- `@blocking_io` and `@cpu_bound` annotations for diagnostic guidance

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

Sifr should have one main async story: `async def`, `await`, `sifr.task`, and `sifr.sync`.

`sifr.asyncio` can exist later as a compatibility veneer, but it must be implemented on top of the canonical model. It must not define the model.

### Structured Concurrency First

Task lifetime should be visible in source code. Child tasks should normally belong to a parent scope. Detached work must be explicit and rare.

Default APIs should prefer:

- `task.scope(...)`
- `task.TaskGroup`
- `scope.spawn(...)`: canonical task creation; all spawned tasks are children of a scope
- `task.gather(*handles)`: wait for multiple task handles, preserving input ordering
- `task.select(*handles)`: first-completion semantics; losers are cancelled by default
- `task.race(*handles)`: alias for `select`; losers are cancelled by default

Default APIs should not encourage:

- ambient global tasks
- silent fire-and-forget work
- detached task handles
- shutdown behavior that depends on runtime accident

### Async Is For Waiting

Async tasks are for I/O waiting and cooperative scheduling. CPU-bound work and blocking OS calls must use explicit offload APIs.

Required surfaces:

- `@blocking_io` for sync functions that perform blocking I/O
- `@cpu_bound` for sync functions expected to burn CPU
- `task.spawn_blocking(...)`
- `sifr.concurrent.ThreadPoolExecutor`

These annotations are diagnostic facts, not scheduling commands. Calling a `@blocking_io` function from async code should produce a Sifr diagnostic that suggests an async API when one exists, or explicit offload when it does not. Calling a `@cpu_bound` function from async code should suggest `spawn_blocking` or a thread-pool executor. The compiler must not silently rewrite either call into a task or thread.

Deferred surfaces:

- `ProcessPoolExecutor`
- multiprocessing-style APIs

Process pools require a stable typed data and IPC serialization contract. Shipping them first would force a premature transport model.

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

Cancellation is part of the contract, not an implementation detail.

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
- cleanup is awaited to completion; if cleanup loops forever or deadlocks, the parent scope can hang
- v1 does not expose forceful task abort, cancellation suppression, shielding, or uncancel counters
- CPU loops that do not await cannot be interrupted until they reach a cooperative cancellation point
- cleanup failures become structured secondary evidence instead of replacing the primary cancellation/failure cause

The shutdown result is deterministic, but not guaranteed to make progress if user cleanup never completes. Cleanup hangs are programmer bugs; the v1 runtime does not paper over them with unsafe aborts.

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
- async generators and async comprehensions in the first async model

Some out-of-scope items may get thin compatibility wrappers later. They should not be allowed to shape the core model.

`contextvars` is intentionally deferred. Sifr should prefer lexical scope and explicit task arguments over implicit task-local propagation. If later evidence shows a real need for task-local storage with structured inheritance, a `sifr.task.local[T]` primitive can be designed as a scoped, lexical value with structured inheritance, not as a global mutable copy-on-fork store.

`selectors` is intentionally not a public model requirement. Runtime internals may use readiness machinery, but users should compose tasks and channels rather than file-descriptor readiness APIs. If low-level socket work later requires a public module, it should land as a curated compatibility layer with CPython-derived tests, not as a core async concept.

Async generators are distinct from async iteration. This model owns the async iterable protocol and `async for` over channels/streams. User-defined `async def` with `yield` requires separate `AsyncGenerator` HIR and protocol work and remains a later feature.

## Type System

The type system has first-class awaitability and task-boundary rules.

Core types:

- `Coroutine[T, E]`: an unscheduled async computation created by calling an async function. It is linear: awaiting it or spawning it consumes it. `E` is the ordinary error channel.
- `Task[T, E]`: a scheduled child task handle. It is not a `Result`; it is an awaitable handle that yields `TaskResult[T, E]` when observed from outside the task.
- `Task[T]`: shorthand for `Task[T, Never]`. It still observes cancellation through `TaskResult[T, Never]`; cancellation is not part of the ordinary `E` channel.
- `TaskResult[T, E]`: the result of observing a scheduled task handle. It has three branches: `Ok(T)`, `Err(Failure[E])`, and `Cancelled(Failure[CancellationError])`.
- `Failure[E]`: a primary failure plus secondary evidence: `primary: E`, `secondary: list[SecondaryError]`.
- `CancellationError`: materialized evidence that a child task was cancelled. It is not the in-task active cancellation signal and does not inherit from `Error`.
- `TimeoutError`: ordinary timeout failure produced when a timeout scope wins its race and cancels the enclosed operation.
- `SecondaryError`: structured evidence attached to a primary cancellation or failure when cleanup or sibling tasks also fail during unwinding.
- `Awaitable[T]`: structural protocol for values that can be awaited. `Coroutine[T, E]` implements an awaitable whose result follows the async function's surface return type; `Task[T, E]` implements `Awaitable[TaskResult[T, E]]`.
- `AsyncFunction[Params, T, E]`: the callable type of `async def`. The type checker must distinguish async callables from sync callables with the same parameters.
- `Never`: bottom type used by `Task[T, Never]`, exhaustive matches, and unreachable control flow.

Async function lifting rules:

- `async def f(...) -> T` has async callable type `AsyncFunction[Params, T, Never]`; calling it returns `Coroutine[T, Never]`.
- `async def f(...) -> Result[T, E]` has async callable type `AsyncFunction[Params, T, E]`; calling it returns `Coroutine[T, E]`.
- Nested results are not flattened beyond the outer async error channel: `async def f() -> Result[Result[A, E1], E2]` returns `Coroutine[Result[A, E1], E2]`.
- Calling an async function returns a linear `Coroutine[T, E]`; it does not run as a sync function and does not schedule itself.
- `AsyncFunction` is not a subtype of sync `Function`/`Callable`. Storing an async function in a sync callable variable, passing it where a sync callable is required, or invoking it through a sync-call path is a compile-time error.

Await rules:

- `await x` is valid only when `x` has an awaitable type.
- Awaiting a same-task coroutine consumes the coroutine and yields the async function's surface return type: `await Coroutine[T, Never] -> T`, and `await Coroutine[T, E] -> Result[T, E]`.
- `scope.spawn(Coroutine[T, E]) -> Task[T, E]`; spawning consumes the coroutine and schedules it as a child.
- `await Task[T, E]` always produces `TaskResult[T, E]` when the awaiting task is not itself actively cancelled.
- The `TaskResult.Err(Failure[E])` branch follows ordinary `Error` rules.
- The `TaskResult.Cancelled(Failure[CancellationError])` branch is task-control evidence that must be matched or converted explicitly.
- `try await` is valid for same-task coroutines that produce `Result[T, E]`.
- `try await task_handle` is rejected in the first model because task cancellation is not an ordinary error branch; users must `match await task_handle` and convert cancellation into an ordinary error intentionally when that is what the enclosing API wants.

`SecondaryError` is inspectable evidence attached to `Failure[E]`. It never masks the primary cause and does not participate in ordinary `except Error` matching unless a user explicitly inspects it.

```sifr
struct Failure[E]:
    primary: E
    secondary: list[SecondaryError]

enum SecondaryError:
    CleanupFailed(error: Error, location: str)
    SiblingFailed(error: Error, task_id: str)
    CancellationDuringCleanup(cause: CancellationError)
```

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
```

Task composition APIs consume the task handles passed to them. Consumed handles are no longer usable by the caller.

```sifr
task.gather(handles: list[Task[T, E]]) -> TaskResult[list[T], E]

task.select[A, EA, B, EB](a: Task[A, EA], b: Task[B, EB]) -> Select2[TaskResult[A, EA], TaskResult[B, EB]]

enum Select2[A, B]:
    First(A)
    Second(B)

task.race(handles: list[Task[T, E]]) -> TaskResult[T, E]

task.timeout(handle: Task[T, E], duration: Duration) -> TaskResult[T, E | TimeoutError]
```

Timeout translates cancellation caused by the deadline into the ordinary `TimeoutError` branch. If the child was already externally cancelled before the deadline wins, the result remains `Cancelled(Failure[CancellationError])`. If the deadline and inner completion become ready in the same scheduler tick, inner completion wins.

Channel endpoints are explicit so close and backpressure semantics are unambiguous:

```sifr
sync.channel[T]() -> (sync.ChannelSender[T], sync.ChannelReceiver[T])
sync.bounded_channel[T](capacity: int) -> (sync.ChannelSender[T], sync.ChannelReceiver[T])

async def ChannelSender[T].send(value: T) -> Result[None, ClosedError]
def ChannelSender[T].close() -> None

async def ChannelReceiver[T].receive() -> Result[T, ClosedError]
```

`ClosedError` from `receive` means the channel is closed and drained. There is no separate `None` end-of-stream state in the first model.

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

`TaskGroup` adds sibling-failure policy on top of task scopes. A plain `TaskScope` owns lifetime; a `TaskGroup` owns group error behavior.

General tracked-collection proof is not part of the first model. Handles may be consumed by explicit composition APIs (`gather`, `select`, `race`) or by simple explicit loops such as `for h in handles: await h`; the scope still owns child lifetime regardless of handle observation.

## Task Composition

`task.gather` is fail-fast:

- successful completion returns `TaskResult.Ok(list[T])` with results in input order
- the first observed child error cancels unfinished children and returns `TaskResult.Err(Failure[E])`
- after cancellation cleanup, the earliest failed handle in input order is the primary error if multiple failures surface
- later sibling failures and cleanup failures are recorded as `SecondaryError` values on the primary `Failure[E]`
- collect-all semantics require a future separate API

`task.select` and `task.race` are first-completion APIs:

- they consume input handles
- losing tasks are cancelled by default
- loser handles cannot be awaited or joined after the selection API owns them
- if multiple tasks complete in the same scheduler tick, input order breaks ties deterministically
- users who need all results should use `gather`
- users who need non-cancelling competition must keep explicit handles and perform explicit cleanup through a future API

## Timeout Semantics

`task.timeout(handle, duration)` accepts task handles. Arbitrary awaitables are not accepted directly; users spawn them into a child task first.

Timeout behavior:

- if the inner task completes before `duration`, timeout returns the inner `TaskResult` and does not cancel it
- if `duration` expires first, timeout cancels the inner task, waits for cleanup, and returns `TaskResult.Err(Failure[TimeoutError])`
- if inner completion and timeout expiry become ready in the same scheduler tick, inner completion wins
- cancelling the outer scope while timeout is running cancels the inner task unconditionally
- cleanup failures after timeout cancellation become secondary evidence on the timeout failure

`task.timeout(duration)` is the async context-manager form used for inline blocks and compatibility with `sifr.asyncio.timeout(duration)`. It uses the same completion-vs-deadline policy through structured scope cancellation.

## Ownership And Borrowing

`scope.spawn` requires captures and return values to satisfy task-boundary requirements. Detached spawn is not exposed; a future detached spawn must require explicit owned, sendable, static captures.

Borrow rules at async boundaries:

| Value form | Across `await` in same task | Across `scope.spawn` |
| --- | --- | --- |
| immutable borrow | allowed only when the borrow remains valid and no conflicting mutation exists | allowed only when the scoped lifetime proves the task cannot outlive the borrow and the referent is share-safe |
| mutable borrow | rejected when it would remain live across `await` | rejected; use explicit synchronization or ownership transfer |
| owned value | allowed | allowed when the type is sendable across task boundaries |
| `sync.Shared[T]` | allowed for immutable shared data | allowed when `T` satisfies the share/send requirements |
| unsynchronized mutable state | rejected | rejected |

Spawned tasks require sendable task boundaries in the first model. Ordinary awaited coroutines within the same task do not introduce a spawn boundary. Local non-send task sets are deferred.

## Synchronization Primitives

The compiler does not silently turn local state into shared state. Shared memory and coordination are explicit.

`sync.Shared[T]` exposes immutable shared ownership. It requires `T` to satisfy the `ShareSafe` capability:

- `T` must be `Send + Sync`
- `T` must not contain unsynchronized interior mutability
- types with their own synchronization may satisfy `ShareSafe`
- `Shared[Cell[int]]` and `Shared[list[MutableThing]]` are rejected

`sync.Lock[T]` and `sync.RwLock[T]` provide explicit mutable sharing. They use synchronous Rust mutex primitives in the first model. Acquiring one in async code may block the current runtime worker under contention, so they are permitted only for short, low-contention critical sections. Channels are preferred for async coordination. A distinct `sync.AsyncLock[T]` is deferred.

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
- bounded channels apply async backpressure when full

`sync.Semaphore` and `sync.Notify` cover common coordination patterns. `Notify` is edge-triggered; level-triggered event behavior requires explicit state such as `sync.Shared[bool] + Notify`.

## Blocking And Thread Offload

Async tasks are for waiting and cooperative scheduling. CPU-bound work and blocking OS calls must use explicit offload.

`@blocking_io` and `@cpu_bound` are diagnostic annotations. They never imply automatic task or thread scheduling. Calling a known blocking or CPU-heavy function from async code should produce a diagnostic that suggests an async API when one exists or explicit offload when it does not.

`task.spawn_blocking` and `sifr.concurrent.ThreadPoolExecutor` provide explicit offload:

- blocking work returns typed results
- cancelling `task.spawn_blocking` or thread-pool work requests cancellation and drops/abandons the handle result
- the first model does not forcibly abort a running OS thread
- already-running blocking work may continue to completion, but its result is discarded after cancellation
- `spawn_blocking` requires owned, sendable, static captures
- scoped borrowed captures are rejected for `spawn_blocking` because already-running OS work may outlive the async scope after cancellation
- hard interruption requires future process isolation and typed IPC

## Async Resource Protocols

`async with` is part of the user-facing async model.

`task.scope()` and `task.timeout(duration)` use async context-manager behavior. General user-defined async context managers follow the same cleanup contract:

- async enter/exit protocol methods are awaited
- cleanup order is LIFO
- cancellation inside `async with` unwinds active async context managers
- async exit receives the cancellation cause
- async exit cleanup runs to completion unless the runtime is forcefully aborted by an unrecoverable system failure
- errors from async exit during cancellation become `SecondaryError` evidence attached to the owning task/scope result
- panic-like failures from async exit are caught at task/runtime boundaries where technically possible and surfaced as structured failure evidence

`async for` works for async iterable values such as channel-backed streams. Async generators and async comprehensions are separate future features.

## Compatibility Veneers

Compatibility layers wrap the canonical model; they do not define a second async model.

| Compatibility API | Canonical Sifr equivalent | Intentional divergence |
| --- | --- | --- |
| `sifr.asyncio.run(fn)` | compatibility shim over direct async entrypoint bootstrap | not needed for new Sifr code; no public event loop is exposed |
| `sifr.asyncio.create_task(fn)` | `scope.spawn(fn)` inside an explicit task scope | invalid outside a scope; does not create ambient orphan tasks |
| `sifr.asyncio.gather(*tasks)` | `sifr.task.gather(*tasks)` | fail-fast by default; collect-all behavior is deferred |
| `sifr.asyncio.TaskGroup` | `sifr.task.TaskGroup` | follows Sifr `TaskResult`/`Failure` semantics |
| `sifr.asyncio.sleep(delay)` | `sifr.task.sleep(delay)` | no event-loop parameter |
| `sifr.asyncio.wait_for(task, timeout)` | `sifr.task.timeout(task, timeout)` | accepts task handles, not arbitrary awaitables, in the first model |
| `sifr.asyncio.timeout(duration)` | `sifr.task.timeout(duration)` context-manager form | implemented through structured scope cancellation |
| `sifr.asyncio.Queue` | `sifr.sync.Channel` / `sifr.sync.bounded_channel` | no `task_done`/`join` queue accounting in the first model |
| `asyncio.Event` / `threading.Event` | `sifr.sync.Notify` or `sync.Shared[bool] + Notify` | `Notify` is edge-triggered; level-triggered Event behavior needs explicit state |
| `threading.Condition` | `sifr.sync.Notify` plus `sifr.sync.Lock` | predicate discipline is explicit; not a transparent alias |
| `sifr.concurrent.Future` | compatibility wrapper over task/blocking handles | not a pure alias; blocking work has different cancellation/lifetime behavior |

Unsupported compatibility surfaces are intentionally absent or diagnosed: raw event loops, loop policies, transports/protocols, public selectors, `contextvars`, multiprocessing, process pools, subprocess, signals, and raw callback-first APIs.

## Diagnostics Contract

Async diagnostics are Sifr-native and stable. Rust compiler errors may be used as implementation evidence, but they must not be the primary user experience for covered cases.

Diagnostic families cover:

- invalid async syntax/use
- `await` outside async
- awaiting non-awaitable values
- async calls from sync callable paths
- `try await` on task handles
- task-boundary Send/Sync failure
- borrow-across-await failure
- borrowed values escaping task boundaries
- detached-task capture failure if detached tasks are added later
- cancellation misuse
- blocking call in async context
- lock guard live at an `await` point
- invalid async protocol implementation
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
11. Compatibility veneers must not introduce a second runtime model.
12. Public selectors, `contextvars`, multiprocessing, process pools, raw event loops, and transport/protocol APIs are deferred.
13. `ProcessPoolExecutor` is blocked on the future typed IPC/serialization contract.
14. `@blocking_io` and `@cpu_bound` are diagnostic annotations, not implicit scheduling directives.
15. Subprocess and signal APIs require a later model amendment.
16. Cancellation suppression, shielding, cancellation counters, and graceful shutdown tokens are deferred; graceful shutdown uses structured scope cancellation and explicit channels.
