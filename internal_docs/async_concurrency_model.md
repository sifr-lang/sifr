# Async and Concurrency Model Contract

Status: canonical Phase 32 model contract
Target phase: Phase 32
Last updated: 2026-05-09

## Purpose

This document defines the Sifr async and concurrency model and, more importantly, the milestone sequence for building it correctly. `internal_docs/phases/32_async_ecosystem.md` is the phase execution plan and must reference this file for the semantic contract instead of restating or weakening it.

The phase should not be an ecosystem grab bag. Its job is to make one coherent model real:

- Python-shaped syntax: `async def`, `await`, `async with`, `async for`
- Rust-shaped safety: ownership-aware task boundaries, explicit sharing, no hidden thread-safety wrappers
- Structured concurrency by default: parent scopes own child tasks
- Typed cancellation and shutdown behavior
- Explicit offload for CPU-bound or blocking work
- Blocking/IO annotations that power diagnostics instead of hidden scheduling changes
- Compatibility layers only after the canonical model exists

The phase succeeds when users can write practical concurrent Sifr programs without learning raw event-loop internals and without escaping Sifr's core guarantee: no user-triggerable runtime panics.

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
- diagnostics and validation for the model

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
- async generators and async comprehensions as phase exit requirements

Some out-of-scope items may get thin compatibility wrappers later. They should not be allowed to shape the core model.

`contextvars` is intentionally deferred. Sifr should prefer lexical scope and explicit task arguments over implicit task-local propagation. If later evidence shows a real need for task-local storage with structured inheritance, a `sifr.task.local[T]` primitive can be designed as a scoped, lexical value with structured inheritance, not as a global mutable copy-on-fork store.

`selectors` is intentionally not a public phase requirement. Runtime internals may use readiness machinery, but users should compose tasks and channels rather than file-descriptor readiness APIs. If low-level socket work later requires a public module, it should land as a curated compatibility layer with CPython-derived tests, not as a core async concept.

Async generators are distinct from async iteration. This phase owns the async iterable protocol and `async for` over channels/streams. User-defined `async def` with `yield` requires separate `AsyncGenerator` HIR and protocol work and remains a later feature.

## Architecture Targets

### Language and HIR

Add canonical HIR concepts rather than encoding async behavior through ad hoc expression strings.

Required concepts:

- async function marker
- await expression
- awaitable/future type
- task handle type
- task scope/group type
- cancellation token or cancellation result type
- async context-manager protocol
- async iterable protocol
- spawn-boundary capture model

Required HIR additions:

- async function metadata, either `HirStmt::AsyncFnDef` or `HirFunction::is_async`
- `HirExpr::Await`
- async function invocation metadata so async calls are distinguishable from sync calls before codegen
- `HirExpr::TaskSpawn` or an equivalent canonical task-spawn expression
- `HirStmt::AsyncWith`
- `HirStmt::AsyncFor`
- HIR type representation for `Task[T, E]`
- HIR type representation for `Awaitable[T]`
- capture metadata for spawn-boundary sendability, borrowing, and lifetime diagnostics

The HIR must preserve enough source information to emit Sifr diagnostics for:

- `await` outside async
- awaiting non-awaitable values
- spawning non-sendable values
- borrowed values escaping task boundaries
- invalid borrow across await points
- blocking calls in async contexts when statically known

### Type System

The type system needs first-class awaitability and task-boundary rules.

Required type additions:

- `Coroutine[T, E]`: an unscheduled async computation created by calling an async function. It is linear: awaiting it or spawning it consumes it. `E` is the ordinary error channel.
- `Task[T, E]`: a typed task handle for a scheduled child task. It is not a `Result`; it is an awaitable handle that yields `TaskResult[T, E]` when observed from outside the task.
- `Task[T]`: shorthand for `Task[T, Never]`. It still observes cancellation through `TaskResult[T, Never]`; cancellation is not part of the ordinary `E` channel.
- `TaskResult[T, E]`: the result of observing a scheduled task handle. It has three branches: `Ok(T)`, `Err(Failure[E])`, and `Cancelled(Failure[CancellationError])`.
- `Failure[E]`: a primary failure plus secondary evidence: `primary: E`, `secondary: list[SecondaryError]`.
- `CancellationError`: materialized evidence that a child task was cancelled. It is not the in-task active cancellation signal and does not inherit from `Error`.
- `TimeoutError`: ordinary timeout failure produced when a timeout scope wins its race and cancels the enclosed operation.
- `SecondaryError`: structured evidence attached to a primary cancellation or failure when cleanup or sibling tasks also fail during unwinding.
- `Awaitable[T]`: structural protocol for values that can be awaited. `Coroutine[T, E]` implements an awaitable whose result follows the async function's surface return type; `Task[T, E]` implements `Awaitable[TaskResult[T, E]]`.
- `AsyncFunction[Params, T, E]`: the callable type of `async def`. This may be implemented as a distinct type or as a capability flag on `Callable`, but the type checker must distinguish async callables from sync callables with the same parameters.
- `Never`: bottom type used by `Task[T, Never]`, exhaustive matches, and unreachable control flow. `Never` already exists in the architecture type enum and remains the no-value type.

Required rules:

- `Task[T, E]` and `Coroutine[T, E]` require `E: Error` for their ordinary error channel. `CancellationError` is a separate task-observation branch, not part of the ordinary `E` hierarchy. `Task[T, Never]` is valid because `Never` represents no possible ordinary error value.
- `await x` is valid only when `x` has an awaitable type.
- `async def f(...) -> T` has async callable type `AsyncFunction[Params, T, Never]`; calling it returns `Coroutine[T, Never]`.
- `async def f(...) -> Result[T, E]` has async callable type `AsyncFunction[Params, T, E]`; calling it returns `Coroutine[T, E]`.
- Nested results are not flattened beyond the outer async error channel: `async def f() -> Result[Result[A, E1], E2]` returns `Coroutine[Result[A, E1], E2]`.
- Awaiting a same-task coroutine consumes the coroutine and yields the async function's surface return type: `await Coroutine[T, Never] -> T`, and `await Coroutine[T, E] -> Result[T, E]`.
- `scope.spawn(Coroutine[T, E]) -> Task[T, E]`; spawning consumes the coroutine and schedules it as a child.
- `await Task[T, E]` always produces `TaskResult[T, E]` when the awaiting task is not itself actively cancelled. The `Err(Failure[E])` branch follows ordinary `Error` rules. The `Cancelled(Failure[CancellationError])` branch is task-control evidence that must be matched or converted explicitly.
- `try await` is valid for same-task coroutines that produce `Result[T, E]`. `try await task_handle` is rejected in v1 because task cancellation is not an ordinary error branch; users must `match await task_handle` and convert cancellation into an ordinary error intentionally when that is what the enclosing API wants.
- Active cancellation of the current task bypasses user `try`/`except` handlers, including `except Error`. It runs `finally`/context cleanup and is materialized as `CancellationError` only at the task or scope boundary.
- `await` is protocol-based: any type implementing `Awaitable[T]` is awaitable, not only `Task`.
- Calling an async function returns a linear `Coroutine[T, E]`; it does not run as a sync function and does not schedule itself.
- `AsyncFunction` is not a subtype of sync `Function`/`Callable`. Storing an async function in a sync callable variable, passing it where a sync callable is required, or invoking it through a sync-call path is a compile-time error.
- `scope.spawn` requires captures and return values to satisfy task-boundary requirements.
- `scope.spawn` can use stricter lifetime-scoped rules than detached spawn.
- detached spawn is not exposed in v1. A future `spawn_detached`, if added, must require explicit owned, sendable, static captures.
- mutable cross-task access requires explicit synchronization.
- values borrowed across `await` must be proven valid or rejected.
- spawned tasks require sendable task boundaries in v1. Ordinary awaited futures within the same task do not introduce a spawn boundary.

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

Borrow rules at async boundaries:

| Value form | Across `await` in same task | Across `scope.spawn` |
| --- | --- | --- |
| immutable borrow | allowed only when the borrow remains valid and no conflicting mutation exists | allowed only when the scoped lifetime proves the task cannot outlive the borrow and the referent is share-safe |
| mutable borrow | rejected when it would remain live across `await` | rejected; use explicit synchronization or ownership transfer |
| owned value | allowed | allowed when the type is sendable across task boundaries |
| `sync.Shared[T]` | allowed for immutable shared data | allowed when `T` satisfies the share/send requirements |
| unsynchronized mutable state | rejected | rejected |

### Runtime and Codegen

The runtime substrate should be generated, not user-managed.

Codegen must:

- emit Rust async functions for Sifr async functions
- emit `.await` only for typed awaitable values
- bootstrap the runtime at async entrypoints
- materialize runtime dependencies only when needed
- preserve `Result`/`Option` safety across await points
- avoid generated `.unwrap()`, `.expect()`, and `panic!` in user-triggerable paths
- lower task scopes so all children are joined, cancelled, or consumed deterministically

### Diagnostics

All async diagnostics should be Sifr-native and stable.

Diagnostic families should cover:

- invalid async syntax/use
- non-awaitable await
- task-boundary Send/Sync failure
- borrow-across-await failure
- detached-task capture failure
- cancellation misuse
- blocking call in async context
- invalid async protocol implementation

Rust compiler errors can be used as implementation evidence, but they should not leak as the primary user experience.

## Core API Signatures

The first implementation must use these shapes unless a later reviewed model amendment changes them.

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

`ClosedError` from `receive` means the channel is closed and drained. There is no separate `None` end-of-stream state in v1.

## Milestone Sequence

The milestones below are ordered by dependency. Later ecosystem work should not start until this sequence has closed.

### milestone_async_0: Model Contract and Runtime Architecture Lock

Status: proposed

Goal: lock the semantic contract before adding code. This is a short design milestone that prevents the implementation from drifting into partial `asyncio` compatibility or raw Tokio exposure.

Work items:

- Write the canonical async/concurrency contract into `internal_docs/architecture.md`.
- Decide the runtime substrate boundary:
  - public Sifr API is runtime-neutral
  - implementation may use Tokio
  - no public event-loop object in the primary model
- Define core public modules:
  - `sifr.task`
  - `sifr.sync`
  - `sifr.concurrent`
- Define initial types:
  - `Coroutine[T, E]`
  - `Task[T, E]`
  - `Task[T]` as shorthand for `Task[T, Never]`
  - `TaskResult[T, E]`
  - `Failure[E]`
  - `TaskGroup`
  - `TaskScope`
  - `CancellationError`
  - `TimeoutError`
  - `Channel[T]`
  - `Lock[T]`
- Define async type-system additions:
  - add task-handle type representation (`Type::Task` or equivalent)
  - add coroutine type representation (`Type::Coroutine` or equivalent)
  - add task-result type representation (`Type::TaskResult` or equivalent)
  - add awaitable protocol representation (`Type::Awaitable` or equivalent structural protocol)
  - add async-callable representation (`Type::AsyncFunction` or an async capability on `Callable`)
  - confirm async functions are not interchangeable with sync functions of the same signature
  - reject assigning, passing, or invoking an `AsyncFunction` through a sync callable type
  - require `Task[T, E]` to satisfy `E: Error`, with `Never` accepted as the no-error bottom type
  - confirm `Task[T, E]` implements `Awaitable[TaskResult[T, E]]`
  - confirm `Coroutine[T, E]` is linear and consumed by `await` or `scope.spawn`
- Define task container protocols:
  - `task.scope()` returns a `TaskScope`
  - `TaskScope` is an async context manager implementing async `__aenter__` and `__aexit__`
  - `async with task.scope() as scope` binds `scope` to the `TaskScope` instance
  - `TaskScope.__aexit__` waits for all children to finish, or cancels unfinished children on abnormal exit, and waits for cleanup before the scope exits
  - `TaskScope` cannot be used outside its `async with` lifetime
  - `TaskGroup` is the higher-level sibling-failure composition API built on top of task scopes; `TaskScope` owns lifetime, while `TaskGroup` owns group error policy
- Define HIR additions:
  - async function marker
  - await expression
  - async call representation
  - task spawn representation
  - async context-manager statement
  - async iteration statement
  - spawn capture metadata
- Resolve scope conflict with `internal_docs/phases/32_async_ecosystem.md`:
  - Phase 32 planning must follow this 9-milestone async/concurrency model
  - the older 4-milestone phase doc must be rewritten or explicitly marked superseded before implementation begins
  - subprocess and signal handling are not Phase 32 exit criteria for this v1 model
  - any future subprocess/signal work requires a separate model amendment and cannot be inferred from the older phase document
- Lock task result semantics:
  - `await Task[T, E]` always yields `TaskResult[T, E]` when observed by a non-cancelled task
  - `await Task[T]` yields `TaskResult[T, Never]`
  - `TaskResult[T, E]` has `Ok(T)`, `Err(Failure[E])`, and `Cancelled(Failure[CancellationError])` branches
  - existing `try`/`except` auto-unwrap works on ordinary `E` errors inside `try` blocks
  - `CancellationError` is not a subclass of `Error` and is therefore never matched by broad `except Error`; it must be handled explicitly or converted into an ordinary error
  - outside `try`, the observable expression type remains `TaskResult[T, E]`
  - `try await task_handle` is rejected in v1 because `try` cannot auto-unwrap the `Cancelled` branch; use `match await task_handle`
  - active cancellation of the current task is not routed through the ordinary `Result` auto-unwrap path and cannot be swallowed by `except Error`
  - `CancellationError` is produced only when cancellation is observed from outside the cancelled task, such as by awaiting or joining a cancelled child handle
- Define detached task policy:
  - v1 exposes scoped spawn only
  - `scope.spawn(...)` is the canonical task creation API
  - free-floating detached spawn is deferred
  - any future `spawn_detached` must require explicit owned/static/sendable captures
- Define cancellation policy:
  - `CancellationError` and `TimeoutError` live in `sifr.task`
  - `Task[T, E]` materializes `CancellationError` when an owner observes a task cancelled before completing
  - `task.timeout` uses `TimeoutError` when an operation exceeds its deadline
  - timeout cancels the enclosed operation
  - task-group failure cancels unfinished siblings
  - cancelling a task is observable and typed
  - cancellation waits for cleanup before scope exit
  - broad `except Error` handlers in a cancelled task do not intercept the active cancellation signal
  - materialized `CancellationError` evidence also does not match broad `except Error`
  - cancellation suppression, `uncancel` counters, and shield-like APIs are not exposed in v1
- Define timeout API forms:
  - `task.timeout(task, duration)` wraps a task handle
  - `task.timeout(duration)` returns an async context manager usable as `async with task.timeout(duration):`
  - both forms share the same completion-vs-deadline race policy
  - the context-manager form is the canonical implementation target for `sifr.asyncio.timeout(duration)`
  - arbitrary awaitables are not accepted by `task.timeout` in v1; spawn the awaitable into a child task first
- Define selection policy:
  - `select` / `race` cancel losing tasks by default
  - `select` / `race` consume their input handles; losers cannot be awaited later
  - if multiple tasks complete in the same scheduler tick, input order breaks ties deterministically
  - users must not rely on tie-breaking order for correctness; use `gather` plus explicit priority logic when order matters
  - users who need all results should use `gather`
  - users who need non-cancelling competition must keep handles and perform explicit cleanup
- Define channel policy:
  - `sync.channel[T]()` returns `(ChannelSender[T], ChannelReceiver[T])`
  - `sync.bounded_channel[T](capacity)` returns `(ChannelSender[T], ChannelReceiver[T])`
  - senders are clonable; receivers are single-consumer handles in v1
  - `await sender.send(value) -> Result[None, ClosedError]`
  - `await receiver.receive() -> Result[T, ClosedError]`; `ClosedError` means closed and drained
  - `sender.close()` wakes pending senders and receivers deterministically
  - bounded channels apply async backpressure when full
- Define lock policy:
  - `sync.Lock[T]` uses a synchronous Rust mutex internally in v1
  - acquiring `sync.Lock` in async code may block the current runtime worker under contention and is permitted only for short, low-contention critical sections
  - `lock()` is not await-aware and returns a guard restricted to a synchronous lexical scope
  - lock guards must not cross `await` points in v1
  - the type checker rejects a live `LockGuard` or `RwLockGuard` at an `await` point
  - diagnostic family: async safety
  - message: "lock guard is still live at this await point; lock guards cannot cross await points in v1"
  - help: "release the lock before the await, or use a channel to communicate results instead"
  - a distinct `sync.AsyncLock[T]`, if needed, is deferred
- Define blocking annotation policy:
  - `@blocking_io` and `@cpu_bound` are compile-time diagnostic annotations
  - they never imply automatic task/thread scheduling
  - stdlib blocking APIs should be annotated as the database of known blocking calls
  - diagnostics are warning-level in v1 unless the called API is known to break runtime safety
  - a formal effect/capability system can be evaluated after the v1 annotation model has evidence
- Define runtime-neutral API gate:
  - no public Sifr API may expose Tokio or runtime-specific types
  - runtime internals stay behind `sifr._runtime` or an equivalent private boundary
- Define runtime selection policy:
  - Tokio or the chosen Tokio-compatible substrate is the only v1 runtime
  - users do not configure or select runtimes in the primary model
  - multiple runtime instances in one generated binary are unsupported in v1
  - custom runtime injection and runtime tuning are deferred
- Define validation fixture names and diagnostics codes before implementation begins.

Acceptance criteria:

- The model contract is documented.
- The phase explicitly rejects raw event-loop APIs as primary API.
- Typed serialization, web, process pools, and full `asyncio` parity are documented as out of scope.
- Every later milestone has positive and negative validation targets.
- The previously open decisions are locked or explicitly moved to v2 with a dependency.

Validation:

- Documentation review only.
- No compiler behavior change required.

### milestone_async_1: Async Syntax, Awaitability, and HIR Substrate

Status: proposed

Goal: teach the compiler to understand async syntax as typed Sifr semantics without introducing task scheduling yet.

Work items:

- Parse and lower `async def`.
- Parse and lower `await`.
- Parse and lower minimal `async with task.scope() as scope` as a built-in scoped-task construct. General user-defined async context-manager protocol remains `milestone_async_7`.
- Add HIR nodes for async functions and await expressions.
- Add awaitable/future type representation.
- Add await type-checking: `await x` is valid only when `x: Awaitable[T]`, and the result type of the await expression is `T`.
- Add structural awaitable protocol checking. Nominal conformance is not required when a type structurally implements the await protocol.
- Reject async function calls from sync functions. A sync function cannot create a `Task` by calling an async function directly; it must enter through an async entrypoint or a future explicit runtime bridge.
- Reject `await` outside async functions.
- Reject awaiting non-awaitable values.
- Preserve existing `try`/`except` auto-unwrap behavior for `Result` values produced by await expressions, even if runtime execution arrives in the next milestone.
- Reject `try await task_handle` in v1; task-handle cancellation must be matched or explicitly converted.
- Add source-span plumbing for async diagnostics.
- Add initial codegen shape for async functions that do not spawn tasks.

Acceptance criteria:

- `async def` is represented explicitly in HIR.
- `await` is represented explicitly in HIR.
- Type checking distinguishes awaitable and non-awaitable values.
- Awaiting `Task[T, E]` has one stable type: `TaskResult[T, E]`.
- `async with task.scope()` works as a built-in task-scope form before general async context-manager protocols land.
- Invalid async syntax/use fails before Rust compilation.
- The implementation does not introduce raw string fallback paths.

Positive validation:

- `async_basic.sifr`
- `await_chain.sifr`
- `async_result_auto_unwrap.sifr`

Negative validation:

- `await_outside_async.sifr`
- `await_non_awaitable.sifr`
- `async_return_type_mismatch.sifr`
- `async_call_without_await_from_sync_rejected.sifr`
- `cancelled_task_except_error_does_not_swallow.sifr`

Demo:

- `demos/m32_async_syntax_demo.sifr`

### milestone_async_2: Runtime Bootstrap and Core Task API

Status: proposed

Goal: make ordinary async programs run without user-managed runtime setup.

Work items:

- Auto-detect async entrypoints.
- Generate runtime bootstrap for `async def main()`.
- Support `async def main() -> Result[None, E]` where `E: Error` as the canonical async program entrypoint.
- Wire runtime dependencies only when async is used.
- Implement `sifr.task.sleep`.
- Implement `sifr.task.timeout`.
- Define `task.timeout(task, duration)` race behavior:
  - if the inner task completes before `duration`, timeout returns the inner result and does not cancel it
  - if `duration` expires first, timeout cancels the inner task, waits for cleanup, and returns `TaskResult.Err(Failure[TimeoutError])`
  - if inner completion and timeout expiry become ready in the same scheduler tick, inner completion wins
  - cancelling the outer scope while timeout is running cancels the inner task unconditionally
  - arbitrary awaitables are not accepted; users must spawn them into a child task first
- Define `task.timeout(duration)` context-manager form:
  - usable as `async with task.timeout(duration):`
  - applies the same completion-vs-deadline race policy to the enclosed block
  - cancellation or timeout of the enclosed block awaits cleanup before scope exit
  - this is the canonical implementation target for `sifr.asyncio.timeout(duration)`
- Implement the minimal `sifr.task.scope` runtime container needed for scoped spawn.
- Implement `scope.spawn` returning a typed task handle.
- Implement task-handle `join`.
- Implement task-handle cancellation API.
- Translate obvious runtime/task-boundary failures into Sifr diagnostics.

Acceptance criteria:

- Async programs run through `sifr run`.
- Sync programs do not gain async runtime dependencies.
- `scope.spawn` returns an observer handle; dropping the handle does not detach the child from the scope.
- There is no free-floating detached spawn in v1.
- `task.sleep` and `task.timeout` work.
- `task.timeout` has deterministic completion-vs-deadline tie-breaking.
- Cancelling a task produces typed, deterministic behavior.
- Runtime bootstrap does not require user-visible event-loop configuration.
- Public `sifr.task`, `sifr.sync`, and `sifr.concurrent` APIs do not expose runtime-specific implementation types.

Positive validation:

- `async_runtime_bootstrap.sifr`
- `scope_spawn_join.sifr`
- `task_sleep.sifr`
- `task_timeout_success.sifr`
- `task_timeout_completion_wins_tie.sifr`
- `task_timeout_context_manager.sifr`
- `task_cancel_basic.sifr`
- `runtime_leak_rejected.sifr`

Negative validation:

- `detached_spawn_not_available.sifr`
- `task_timeout_error_type.sifr`

Demo:

- `demos/m32_task_core_demo.sifr`

### milestone_async_3: Structured Concurrency and Cancellation Semantics

Status: proposed

Goal: make scoped concurrency the default composition model.

Work items:

- Implement `task.scope`.
- Implement `task.TaskGroup`.
- Implement `scope.spawn`.
- Define task-scope ownership rules:
  - `TaskScope` uses nursery ownership: every spawned child belongs to the scope
  - handles returned by `scope.spawn` are observers, not owners; dropping a handle does not detach or cancel the child
  - on normal exit, `TaskScope.__aexit__` waits for all children
  - on abnormal exit, `TaskScope.__aexit__` cancels unfinished children and waits for cleanup
  - child failures that are not explicitly observed are surfaced at scope exit as structured scope failure evidence, never silently discarded
  - no task handle may escape its owning task scope silently
  - general tracked-collection proof is deferred; v1 supports explicit consumption through `gather`, `select`, `race`, and simple `for h in handles: await h` loops
- Implement deterministic scope exit:
  - all child tasks complete,
  - or unfinished children are cancelled,
  - and cleanup is awaited before exit.
- Implement sibling cancellation on first failure for task groups.
- Implement `task.gather` with deterministic result ordering.
- Define `task.gather` error behavior:
  - by default, the first observed child error cancels unfinished children and returns `TaskResult.Err(Failure[E])`
  - after cancellation cleanup, the earliest failed handle in input order is the primary error and later errors are secondary structured errors
  - cleanup errors from cancelled children surface as `SecondaryError` values attached to the primary `Failure[E]`
  - a future collect-all API may return all `Result` values, but v1 `gather` is fail-fast and cancellation-safe
- Implement `task.select` and `task.race` for first-completion behavior.
- Cancel losing tasks by default for `select` and `race`.
- `select` and `race` consume their input handles. A loser handle cannot be awaited or joined after the selection API owns it.
- Define how cancellation composes with `TaskResult`.
- Add diagnostics for leaked task handles and invalid scope escape.

Acceptance criteria:

- Task scopes own child task lifetimes.
- A task spawned inside a scope cannot escape with borrowed state that outlives the scope.
- Dropping a task handle does not detach the task; scope exit still waits for or cancels the child according to normal/abnormal exit rules.
- Task-group failure cancels unfinished siblings.
- Cancellation is observable through the Sifr type model.
- `gather` preserves input ordering for successes and has documented fail-fast cancellation behavior for errors.
- `select`/`race` deterministically cancel losing tasks by default.
- Nested cancellation scopes propagate cancellation in a documented order.

Positive validation:

- `task_scope_basic.sifr`
- `task_group_basic.sifr`
- `task_group_error_cancels_siblings.sifr`
- `task_gather_ordered.sifr`
- `task_gather_cleanup_error_secondary.sifr`
- `task_handle_collection_consumed.sifr`
- `task_scope_unobserved_child_waits.sifr`
- `task_select_first_completion.sifr`
- `task_race_cancels_losers.sifr`
- `cancellation_scope_timeout.sifr`
- `cancellation_group_sibling.sifr`
- `cancellation_nested_scopes.sifr`
- `cancellation_cleanup_runs.sifr`

Negative validation:

- `task_escape_scope_rejected.sifr`
- `cancelled_task_use_rejected.sifr`
- `task_handle_escape_scope_rejected.sifr`
- `task_group_unhandled_error_rejected.sifr`

Demo:

- `demos/m32_structured_concurrency_demo.sifr`

### milestone_async_4: Async Ownership and Send/Sync Boundary Checking

Status: proposed

Goal: make concurrency safety a Sifr compiler feature, not a raw Rust error after codegen.

Work items:

- Track task captures in HIR.
- Check `Send` eligibility for spawned task captures.
- Check `Sync` eligibility where shared references cross boundaries.
- Reject borrowed values that escape a task boundary.
- Reject invalid borrows held across await points.
- Validate scoped spawn requirements and keep detached spawn unavailable in v1.
- Implement field-path diagnostics for non-sendable captures.
- Ensure no compiler path silently inserts sharing wrappers.
- Add regression tests for user-defined classes, lists, dicts, closures, nested functions, and captured `self`.

Acceptance criteria:

- Spawn-boundary errors are reported as Sifr diagnostics.
- Diagnostics identify the captured value and non-sendable field where possible.
- Scoped spawn allows only lifetimes that the scope can prove safe.
- Detached spawn remains unavailable; any attempt to use it receives an intentional diagnostic.
- Borrow-across-await rejection is deterministic and documented.
- No `rustc` Send/Sync diagnostic is the primary user-facing error for covered cases.

Positive validation:

- `spawn_owned_send_value.sifr`
- `spawn_scoped_borrow_ok.sifr`
- `spawn_capture_immutable_shared_ok.sifr`
- `await_without_live_borrow.sifr`

Negative validation:

- `spawn_non_send_field_rejected.sifr`
- `spawn_borrowed_value_escapes_rejected.sifr`
- `borrow_across_await_rejected.sifr`
- `spawn_mutable_alias_rejected.sifr`
- `spawn_self_with_non_send_field_rejected.sifr`

Demo:

- `demos/m32_async_safety_demo.sifr`

### milestone_async_5: Synchronization Primitives and Channels

Status: proposed

Goal: provide the explicit primitives users need when concurrency really requires sharing or coordination.

Work items:

- Implement `sync.Shared[T]` for immutable shared ownership, mapping to atomic shared ownership (`Arc<T>`-style semantics) with no mutation API.
- Require `sync.Shared[T]` to satisfy the v1 `ShareSafe` capability: `T` must be `Send + Sync` and must not contain unsynchronized interior mutability. Types with their own synchronization may satisfy `ShareSafe`; `Shared[Cell[int]]` and `Shared[list[MutableThing]]` are rejected.
- Implement `sync.Lock[T]`.
- Implement `sync.RwLock[T]`.
- Implement `sync.Channel[T]`.
- Implement unbounded multi-producer, single-receiver channels via `sync.channel[T]()`.
- Implement bounded multi-producer, single-receiver channels via `sync.bounded_channel[T](capacity)`.
- Implement channel endpoint and close semantics:
  - `sync.channel[T]()` returns `(ChannelSender[T], ChannelReceiver[T])`
  - `sync.bounded_channel[T](capacity)` returns `(ChannelSender[T], ChannelReceiver[T])`
  - `ChannelSender[T]` is clonable; `ChannelReceiver[T]` is single-consumer in v1
  - `await sender.send(value)` on a closed channel returns `Result[None, ClosedError]`
  - `await receiver.receive()` returns `Result[T, ClosedError]`
  - `ClosedError` from `receive` means closed and drained; there is no second `None` closed state
  - `sender.close()` wakes pending senders and receivers deterministically
  - a task cancelled while blocked on `send` or `receive` propagates task cancellation without duplicating or losing a message
  - bounded channels apply async backpressure when full
- Implement `sync.Semaphore`.
- Implement `sync.Notify`.
- Define sync primitive behavior in async and blocking contexts.
- Implement static lock-guard liveness analysis at await points.
- Reject lock guards that remain live across an `await` point using the v1 `LockGuard`/`RwLockGuard` liveness rule defined in `milestone_async_0`.
- Warn in docs and diagnostics that acquiring `sync.Lock` in async code may block the runtime worker under contention; v1 permits it only for short, low-contention critical sections.
- Add diagnostics for lock misuse where statically knowable.

Acceptance criteria:

- Shared immutable state works across tasks through `sync.Shared[T]`.
- `sync.Shared[T]` exposes shared ownership only for `ShareSafe` types; mutation requires `Lock`, `RwLock`, or message passing.
- Mutable shared state requires `Lock` or `RwLock`.
- Channels are the canonical queue-like concurrency primitive and use clonable senders plus a single receiver handle in v1.
- Channel close and receiver exhaustion behavior is typed and deterministic.
- Channel cancellation behavior is typed and deterministic.
- Lock guard liveness at await points is rejected at compile time.
- Lock guards cannot cross `await` points in v1.
- Semaphore and notify primitives support common coordination patterns.
- The compiler rejects unsynchronized shared mutable access.

Positive validation:

- `shared_basic.sifr`
- `lock_basic.sifr`
- `rwlock_readers.sifr`
- `channel_basic.sifr`
- `channel_backpressure.sifr`
- `channel_close.sifr`
- `channel_cancel_pending_receive.sifr`
- `semaphore_basic.sifr`
- `notify_basic.sifr`

Negative validation:

- `shared_mut_without_lock_rejected.sifr`
- `channel_send_wrong_type_rejected.sifr`
- `channel_non_send_element_rejected.sifr`
- `lock_guard_escape_rejected.sifr`
- `lock_guard_across_await_rejected.sifr`
- `lock_across_task_boundary_rejected.sifr`

Demo:

- `demos/m32_sync_primitives_demo.sifr`

### milestone_async_6: Blocking and Thread-Based Offload

Status: proposed

Goal: keep the async scheduler for waiting and provide explicit APIs for CPU-bound or blocking work.

Work items:

- Implement `@blocking_io` as a diagnostic annotation for sync functions that perform blocking I/O.
- Implement `@cpu_bound` as a diagnostic annotation for sync functions that are CPU-heavy.
- Annotate known blocking stdlib functions so async-context diagnostics can be precise.
- Implement `task.spawn_blocking`.
- Implement `sifr.concurrent.ThreadPoolExecutor`.
- Add `sifr.threading` as a compatibility veneer over the same thread and sync substrate where it can stay thin:
  - `Thread`
  - `Lock`
  - `Event`
  - `Condition`
- Define return/error/cancellation behavior for blocking tasks.
  - cancelling `task.spawn_blocking` or thread-pool work requests cancellation and drops/abandons the handle result
  - v1 does not forcibly abort a running OS thread
  - already-running blocking work may continue to completion, but its result is discarded after cancellation
  - `spawn_blocking` requires owned, sendable, `'static` captures in v1
  - scoped borrowed captures are rejected for `spawn_blocking` because already-running OS work may outlive the async scope after cancellation
  - users who need hard interruption must use future process isolation/typed IPC, which is deferred
- Add diagnostics for known blocking stdlib calls used directly in async contexts.
- Ensure blocking work cannot accidentally occupy cooperative async workers where Sifr can control the path.
- Document when users should choose async tasks, channels, locks, or blocking offload.

Compatibility mapping:

| Compatibility API | Canonical Sifr equivalent | Intentional divergence |
| --- | --- | --- |
| `sifr.threading.Thread` | `sifr.concurrent.Thread` or thread-pool-backed standalone thread API | no detached unjoined thread by default; handles must remain observable |
| `sifr.threading.Lock` | `sifr.sync.Lock` compatibility wrapper | synchronous lock; can block if used from async code |
| `sifr.threading.Event` | `sifr.sync.Notify` or `sync.Shared[bool] + Notify` | Python Event is level-triggered; `Notify` is edge-triggered |
| `sifr.threading.Condition` | `sifr.sync.Notify` plus `sifr.sync.Lock` | predicate discipline is explicit; not a transparent alias |

Acceptance criteria:

- CPU-bound functions can be offloaded explicitly.
- Blocking work returns typed results.
- Thread-pool tasks obey Send/Sync capture rules.
- Direct known-blocking calls and annotated `@blocking_io` calls in async functions produce diagnostics where statically knowable.
- Direct `@cpu_bound` calls in async functions suggest `spawn_blocking` or `ThreadPoolExecutor`.
- `sifr.threading` compatibility APIs are thin wrappers and do not introduce a second synchronization model.
- Process pools remain explicitly blocked on the future typed IPC/serialization contract.

Positive validation:

- `spawn_blocking_basic.sifr`
- `spawn_blocking_result.sifr`
- `thread_pool_executor_basic.sifr`
- `thread_pool_executor_many_tasks.sifr`
- `blocking_io_annotation_diagnostic.sifr`
- `cpu_bound_annotation_diagnostic.sifr`
- `threading_lock_subset.sifr`
- `threading_event_subset.sifr`

Negative validation:

- `blocking_call_in_async_rejected.sifr`
- `thread_pool_non_send_capture_rejected.sifr`
- `process_pool_deferred_diagnostic.sifr`

Demo:

- `demos/m32_blocking_offload_demo.sifr`

### milestone_async_7: Async Resource Protocols and Stream Iteration

Status: proposed

Goal: complete the language-level async control-flow model without dragging in broad ecosystem APIs.

Work items:

- Implement `async with`.
- Define and enforce the async context-manager protocol.
- Implement async iterable protocol.
- Implement `async for`.
- Define cancellation cleanup behavior for async context managers:
  - cleanup order is LIFO, matching acquisition order in reverse
  - cancelling a task inside `async with` unwinds all active async context managers
  - async exit receives the cancellation cause
  - async exit runs to completion unless the enclosing runtime is forcefully aborted
  - errors from async exit during cancellation are wrapped as `SecondaryError` evidence attached to the owning scope result
  - panic-like failures from async exit must be caught at the runtime/codegen boundary and surfaced as secondary structured errors, not process aborts
  - parent cancellation triggers child cancellation, but each task unwinds its own cleanup independently
- Define channel-backed async iteration.
- Keep async generators and async comprehensions deferred; they require separate `AsyncGenerator` HIR and protocol work.

Acceptance criteria:

- `async with` calls async enter/exit protocol methods correctly.
- Async resource cleanup runs under cancellation.
- When a task is cancelled inside an `async with` block, async exit/cleanup is called before scope exit completes.
- If cleanup itself fails during cancellation, the cancellation remains the primary result and cleanup failure is surfaced as secondary structured error evidence through the owning scope.
- `SecondaryError` never masks the primary cancellation/failure result; it is inspection evidence for diagnostics, logs, and future collect-all APIs.
- Async exit cleanup order is LIFO.
- Panic-like failures in async exit do not become process-terminating double-panic paths.
- Nested cancellation is deterministic: parent cancellation triggers child cancellation, and every task unwinds its own cleanup independently.
- `async for` works for channel/stream-like values.
- Async comprehensions are explicitly deferred as syntax sugar over stable `async for`.
- Non-async iterables are rejected in `async for`.
- Async protocol diagnostics are Sifr-native.

Positive validation:

- `async_with_basic.sifr`
- `async_with_cancel_cleanup.sifr`
- `async_with_nested_cleanup_order.sifr`
- `async_for_channel.sifr`
- `async_for_stream_result.sifr`

Negative validation:

- `async_with_missing_protocol_rejected.sifr`
- `async_for_non_async_iterable_rejected.sifr`
- `async_resource_cleanup_error_typed.sifr`
- `async_with_cleanup_panic_secondary.sifr`

Demo:

- `demos/m32_async_resource_demo.sifr`

### milestone_async_8: Compatibility Veneers and Phase Closure

Status: proposed

Goal: expose limited compatibility surfaces only after the canonical model is proven.

Work items:

- Add `sifr.asyncio` as a veneer over `sifr.task` and `sifr.sync`.
- Support only the curated subset:
  - `run`
  - `create_task`
  - `gather`
  - `TaskGroup`
  - `sleep`
  - `wait_for`
  - `timeout`
  - `Queue`
- Add `sifr.concurrent.Future` as a compatibility wrapper over canonical task/blocking-work observation semantics, not a second runtime primitive.
- Keep raw event loops, loop policies, transports/protocols, public selectors, context variables, multiprocessing, and process pools deferred.
- Treat `ProcessPoolExecutor` as blocked on the future typed IPC/serialization contract, not merely postponed.
- Track `ProcessPoolExecutor` as a hard dependency on Phase 40 typed IPC/serialization before any process-pool implementation begins.
- Add CPython-derived compatibility tests for the supported subset.
- Add CPython-derived negative/waiver tests for unsupported raw loop, selector, transport/protocol, contextvars, multiprocessing, and process-pool APIs.
- Document intentional divergences.
- Run full phase closure validation.

Compatibility mapping:

| Compatibility API | Canonical Sifr equivalent | Intentional divergence |
| --- | --- | --- |
| `sifr.asyncio.run(fn)` | compatibility shim over direct async entrypoint bootstrap | not needed for new Sifr code; no public event loop is exposed |
| `sifr.asyncio.create_task(fn)` | `scope.spawn(fn)` inside an explicit task scope | invalid outside a scope; does not create ambient orphan tasks |
| `sifr.asyncio.gather(*tasks)` | `sifr.task.gather(*tasks)` | fail-fast by default; collect-all behavior is deferred |
| `sifr.asyncio.TaskGroup` | `sifr.task.TaskGroup` | follows Sifr `TaskResult`/`Failure` semantics |
| `sifr.asyncio.sleep(delay)` | `sifr.task.sleep(delay)` | no event-loop parameter |
| `sifr.asyncio.wait_for(task, timeout)` | `sifr.task.timeout(task, timeout)` | accepts task handles, not arbitrary awaitables, in v1 |
| `sifr.asyncio.timeout(duration)` | `sifr.task.timeout(duration)` context-manager form | implemented through structured scope cancellation |
| `sifr.asyncio.Queue` | `sifr.sync.Channel` / `sifr.sync.bounded_channel` | no `task_done`/`join` queue accounting in v1 |
| `asyncio.Event` / `threading.Event` | `sifr.sync.Notify` or `sync.Shared[bool] + Notify` | `Notify` is edge-triggered; level-triggered Event behavior needs explicit state |
| `threading.Condition` | `sifr.sync.Notify` plus `sifr.sync.Lock` | predicate discipline is explicit; not a transparent alias |
| `sifr.concurrent.Future` | compatibility wrapper over task/blocking handles | not a pure alias; blocking work has different cancellation/lifetime behavior |

Curated subset rationale:

- `sifr.asyncio` covers only common compatibility APIs that map cleanly to the canonical model.
- `asyncio.Event` and `threading.Event` do not transparently alias to `Notify`; level-triggered semantics require explicit shared state plus notification.
- `asyncio.Condition` maps to `sifr.sync.Notify` plus `sifr.sync.Lock` where needed.
- `asyncio.Barrier` is deferred with `sifr.sync.Barrier`.
- `asyncio.wait` maps to `task.gather` for all-results composition or `task.select` / `task.race` for first-completion behavior.
- `asyncio.as_completed` should be modeled with channel-backed producers and `async for` rather than a distinct core primitive.

Acceptance criteria:

- Compatibility APIs are thin wrappers over canonical model types.
- No compatibility API introduces a second runtime model.
- `sifr.concurrent.Future` is a compatibility wrapper over canonical observation semantics, not a separate future runtime.
- Unsupported `asyncio` APIs fail with intentional diagnostics or remain absent from documented public surface.
- Intentional divergences are documented.
- The phase exit gate passes.

Positive validation:

- `asyncio_run_subset.sifr`
- `asyncio_create_task_subset.sifr`
- `asyncio_task_group_subset.sifr`
- `asyncio_wait_for_subset.sifr`
- `asyncio_queue_via_channel.sifr`
- `concurrent_future_subset.sifr`

Negative validation:

- `asyncio_loop_policy_not_supported.sifr`
- `asyncio_transport_protocol_not_supported.sifr`
- `selectors_public_api_deferred.sifr`
- `contextvars_deferred.sifr`
- `process_pool_not_available.sifr`

Demo:

- `demos/m32_async_concurrency_model_demo.sifr`

## Dependency Graph

```mermaid
flowchart TD
    m0["m32.0 Model Contract"]
    m1["m32.1 Async Syntax + HIR"]
    m2["m32.2 Runtime + Core Task API"]
    m3["m32.3 Structured Concurrency"]
    m4["m32.4 Ownership + Send/Sync"]
    m5["m32.5 Sync Primitives + Channels"]
    m6["m32.6 Blocking + Threads"]
    m7["m32.7 Async Resources + Streams"]
    m8["m32.8 Compatibility + Closure"]

    m0 --> m1
    m1 --> m2
    m2 --> m3
    m3 --> m4
    m4 --> m5
    m4 --> m6
    m5 --> m7
    m6 --> m7
    m7 --> m8
```

## Phase Exit Gate

The phase is complete only when all of these are true:

- `async def` and `await` are first-class typed Sifr constructs.
- Async entrypoints run without user-visible runtime setup.
- `sifr.task` supports scoped spawn, join, cancel, sleep, timeout, gather, select/race, `TaskGroup` structured groups, and `TaskScope` containers via `async with task.scope()`.
- Structured concurrency is the default successful path.
- Detached task behavior is either absent or explicit and restricted.
- Cancellation semantics are deterministic and typed.
- Task-boundary Send/Sync and borrow rules are enforced by Sifr diagnostics.
- Explicit synchronization primitives exist for shared state.
- Channels are the canonical producer/consumer primitive.
- CPU-bound and blocking work has explicit offload APIs.
- `async with` and `async for` work for protocol-conforming values.
- Compatibility veneers do not define a second async model.
- No new user-triggerable generated panic paths exist.
- Full local validation passes.

## Required Validation Lanes

Every implementation PR should run at least:

```bash
scripts/run_all_tests.sh --profile quick
```

Milestone closure should run:

```bash
scripts/run_all_tests.sh
```

The phase should add these validation families:

- async syntax pass/fail fixtures
- awaitability type-check fixtures
- runtime bootstrap fixtures
- task lifecycle fixtures
- cancellation determinism fixtures
- structured concurrency fixtures
- task-boundary ownership fixtures
- Send/Sync diagnostics fixtures
- synchronization primitive fixtures
- channel close/backpressure fixtures
- channel cancellation fixtures
- blocking offload fixtures
- blocking/CPU annotation diagnostics fixtures
- async context-manager fixtures
- async iteration fixtures
- compatibility veneer fixtures
- runtime-neutrality checks proving Tokio/runtime-specific types do not leak into public Sifr APIs
- async cleanup panic-boundary fixtures
- generated-code panic sweep for async/runtime paths

## Locked V1 Decisions

These decisions are locked for the first async/concurrency model. `milestone_async_0` should copy them into the architecture contract before implementation begins.

1. `scope.spawn` is the canonical v1 task creation API. Free-floating detached spawn is not exposed.
2. Active task cancellation is scope-exit semantics and is not catchable by ordinary `except Error`; `CancellationError` is the materialized boundary evidence observed from outside the cancelled task.
3. `CancellationError` is not an `Error` subclass and is never matched by broad `except Error`.
4. `await Task[T, E]` always produces `TaskResult[T, E]` when observed by a non-cancelled task. `try await task_handle` is rejected in v1; cancellation requires explicit matching and intentional conversion into an ordinary error when needed.
5. `task.select` and `task.race` cancel losing tasks by default.
6. Channels use explicit sender/receiver endpoints. Send and receive are async operations; receive returns `Result[T, ClosedError]` with no second closed state.
7. Lock guards must not cross `await` points in v1.
8. Spawned tasks require sendable task boundaries in v1. Local, non-Send task sets are deferred.
9. `sifr.asyncio` ships only as a compatibility veneer after the canonical model is complete.
10. Public selectors, contextvars, multiprocessing, process pools, raw event loops, and transport/protocol APIs are deferred.
11. `ProcessPoolExecutor` is blocked on the future typed IPC/serialization contract.
12. `@blocking_io` and `@cpu_bound` are diagnostics annotations, not implicit scheduling directives.
13. Subprocess and signal APIs are out of scope for Phase 32 v1 and require a later model amendment.
14. Cancellation suppression, shielding, cancellation counters, and graceful shutdown tokens are deferred; v1 graceful shutdown uses structured scope cancellation and explicit channels.

## Recommendation

Use this contract as the semantic source for Phase 32 implementation. The phase execution plan lives in `internal_docs/phases/32_async_ecosystem.md` and should stay synchronized with this model.

The phase should close the model first. Web, typed data, subprocess expansion, database clients, and broad CPython async parity should build on top later.
