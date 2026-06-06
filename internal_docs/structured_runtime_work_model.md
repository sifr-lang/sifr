# Structured Runtime Work Model

Status: draft production runtime model contract
Last updated: 2026-06-06

## Purpose

This document defines Sifr's broader concurrency/runtime model beyond the async-only contract in [async_concurrency_model.md](./async_concurrency_model.md).

The async contract explains `async def`, `await`, scoped task groups, task cancellation, async iteration, and async syntax lowering. This document explains how that model extends to all runtime-owned work:

- async tasks
- blocking I/O offload
- CPU-heavy offload
- data-parallel jobs
- long-running child processes
- future typed process workers

The core rule is:

```text
scope owns work
work returns handles
handles must be observed
failures and cancellation are typed
cross-boundary values must be safe
```

Sifr does not expose separate Python-shaped concurrency worlds for `asyncio`, `threading`, `concurrent.futures`, `subprocess`, and `multiprocessing`. Those modules are evidence sources or future adapters. The production model is structured runtime work.

## Product Decision

All runtime-owned work belongs to a scope or another linear owner. Child work must not silently outlive the owner that created it. A work handle must be awaited, joined, cancelled and joined, aggregated by its owning scope, or consumed by an explicit collection such as `JoinSet`.

Canonical user-facing direction:

```sifr
async with task.TaskGroup[AppError]() as group:
    users = group.spawn(fetch_users())
    config = group.spawn_blocking(read_config)
    index = group.spawn_cpu(build_index)
    child = group.spawn_process(process.Command("worker"))

    users_result = await users
    config_result = await config
    index_result = await index
```

The exact method names are settled by the implementation phase. The semantic constraint is fixed: async, blocking, CPU, and process work participate in scoped lifetime, observation, cancellation, and typed failure evidence.

## Work Kinds

| Work kind | Public model | Execution substrate |
| --- | --- | --- |
| async coroutine work | `sifr.task.TaskHandle`, `TaskGroup` | Tokio task internals hidden behind Sifr APIs |
| blocking I/O offload | scoped `spawn_blocking` returning a blocking-work handle | Tokio blocking pool |
| CPU-heavy offload | scoped `spawn_cpu`, `sifr.parallel`, `Pool` | private Rayon-backed pools |
| long-running child process | `sifr.process.Child` plus scoped supervision | `tokio::process`, `std::process`, host process APIs |
| future typed process worker | future worker handle over typed IPC | `sifr.process` plus `sifr.ipc` frames |

Threads and processes are execution substrates, not the public model. Users choose the kind of work and boundary they need; Sifr provides the scope, typed handle, safe communication, and deterministic cancellation/failure behavior.

`task.Scope` or `runtime.Scope` is the general owner for mixed runtime work. M0 chooses the public name and method placement. `TaskGroup[E]` is the fail-fast structured-concurrency policy for child work that shares one error type. Individual child handles may have different success types. Homogeneous result collections such as `join_all`, `race`, `select`, and `JoinSet[T, E]` require one result/error shape unless the user constructs an explicit sum/enum result type.

`TaskHandle[T, E]` is the public affine observation handle name. `Task` may remain an internal type name or compatibility alias only if M0 records a concrete reason to expose both names.

## Existing Implementation Baseline

The current compiler/runtime already points toward this model:

- task lowering recognizes scoped spawn through `task.scope()` and `TaskGroup`
- unscoped top-level `task.spawn(...)` is rejected
- task handles are affine observation resources
- `Task` and `BlockingTask` are awaitable handle types
- task results, failures, cancellation, timeout, select/race, and blocking-task results already have internal type support
- task-boundary captures and channel sends already have sendability checks
- lock guards held across `await` are diagnosed
- blocking offload exists through `task.spawn_blocking(...)`
- generated Rust currently supplies task runtime preambles when async/task features are used
- generated channel support proves the direction but still needs production backpressure semantics
- `sifr.asyncio` is a compatibility veneer, not the runtime model
- `sifr.threading`, `sifr.concurrent`, and current `sifr.subprocess` are not mature production substrates

The production phase should consolidate and harden these paths rather than create a second thread/process-centric runtime.

## Boundary Model

Structured lifetime is separate from boundary safety. Every work boundary has a value-safety rule.

| Boundary | Required contract |
| --- | --- |
| async task boundary | captured values are task-sendable and do not borrow beyond their lexical owner |
| blocking/thread boundary | inputs, outputs, errors, and captures are sendable and observed through typed handles |
| CPU-parallel boundary | items, captures, results, and errors are sendable; shared state uses explicit sync wrappers |
| process boundary | payloads are pipe-owned bytes/text or explicitly `IpcSerializable` typed frames |
| shared-state boundary | shared references use immutable data or explicit synchronization wrappers |

The implementation should formalize these concepts:

- `Sendable`
- `ShareSafe`
- `IpcSerializable`
- no lock guard across `await`
- no unprotected shared mutable state
- no borrowed capture that can outlive the lexical owner

## Communication Model

Communication has separate tiers. Typed IPC is important, but it is not a substitute for channels or process pipes.

| Communication case | Production substrate |
| --- | --- |
| same-process tasks | `sifr.sync` channels with bounded backpressure, close, and cancellation semantics |
| child subprocess I/O | `process.PipeReader` and `process.PipeWriter` over stdin/stdout/stderr |
| Sifr-to-Sifr process workers | future `sifr.ipc.Connection[Req, Res, Err]` typed frames |

Typed IPC sits above an accepted process or transport substrate. It must define framing, versioning, request IDs, result/error frames, cancellation frames, stream close, malformed-frame errors, and backpressure. It is a prerequisite for future process workers and process pools, not a reason to ship process pools before the process substrate is ready.

## Synchronization Model

Synchronization APIs must say whether they are sync shared-state primitives or async coordination primitives.

| Category | Examples | Contract |
| --- | --- | --- |
| sync shared-state primitives | `Mutex[T]`, `RwLock[T]`, `Once` where accepted | guards cannot cross `await`; blocking lock acquisition is diagnosed in async contexts unless offloaded |
| async coordination primitives | `AsyncMutex[T]`, `AsyncRwLock[T]`, `Semaphore`, `Event`, `Notify`, `AsyncChannel[T]` | operations are real suspension points; guard await restrictions are documented and diagnosed |
| optional first-pass primitives | `Barrier`, public `Once` | public only if M0 finds near-term production need; otherwise `internal-only` or `deferred-to-phase-X` |

## Cancellation And Failure

Cancellation applies consistently across work kinds:

- cancellation is idempotent and produces typed evidence
- timeouts preserve the wrapped operation's normal typed outcome and add typed timeout evidence as a distinct variant
- `TaskGroup` child failure cancels siblings and aggregates observed failures
- `race` and `select` cancel losers and return loser-cancellation evidence
- blocking work has limited cancellation evidence, including whether work had already started or completed
- CPU-heavy work declares whether cancellation is cooperative, boundary-only, or wait-for-completion
- child process cancellation escalates through request shutdown, terminate, then kill where the host supports it
- future IPC worker cancellation sends a typed cancel frame before process escalation when the protocol is still live
- cleanup scopes run under cancellation and report cleanup failures without hiding the initiating failure

Current task cancellation is mostly abort-based. The production phase must explicitly decide which abort-based behavior remains v1 semantics and whether a cooperative cancellation layer is introduced internally. Tokio or tokio-util token types must not leak publicly; a Sifr-owned `CancelScope` or cancellation handle may be exposed if the language model needs it.

## Process And Worker Policy

`sifr.process` is the production process substrate. It owns:

- command construction
- environment and working directory
- sync and async spawn/wait/communicate
- owned stdin/stdout/stderr pipes
- binary and explicit text-mode pipe behavior
- timeout and cancellation escalation
- shell execution as an explicit `@shell_exec` security effect

Sync shell APIs are also `@blocking_io` and are rejected in async contexts unless explicitly offloaded. Native async process APIs may use shell execution in async contexts only through explicit `shell=True` or `Command.shell(...)`, and still carry the `@shell_exec` security effect.

`sifr.subprocess` may later become an adapter over `sifr.process`, but it is not the substrate.

Typed process workers are future work after:

1. process supervision exists;
2. owned process pipes are production-grade;
3. typed IPC framing is approved and implemented;
4. payload eligibility and serialization diagnostics are stable.

`ProcessPoolExecutor`, `multiprocessing.Pool`, arbitrary pickle-like transport, and shared-memory APIs are not production runtime foundations.

Minimum typed IPC frame families:

| Family | Frames |
| --- | --- |
| bootstrap | `Hello(protocol_version, schema_hash, worker_kind)`, `Ready(worker_id, capabilities)`, `Reject(reason)` |
| work | `Run(request_id, payload, deadline, context)`, `Started(request_id)`, `Completed(request_id, value)`, `Failed(request_id, error)` |
| control | `Cancel(request_id)`, `Shutdown(mode)`, `Terminating(reason)` |
| health | `Heartbeat(worker_id)`, `WorkerStatus(...)` |
| protocol errors | `MalformedFrame`, `UnsupportedVersion`, `UnsupportedSchema`, `UnsupportedPayload` |

## Public Namespace Policy

Production namespaces:

- `sifr.task`
- `sifr.sync`
- `sifr.runtime`
- `sifr.parallel`
- `sifr.process`
- `sifr.signal`
- `sifr.resource`
- `sifr.ipc`

Compatibility or evidence-only namespaces:

- `sifr.asyncio`
- `sifr.threading`
- `sifr.concurrent.futures`
- `sifr.subprocess`
- `sifr.multiprocessing`

Compatibility adapters must delegate to production APIs and must not introduce legacy global state, raw thread/process handles, detached work, pickle-like transport, event-loop objects, or unstructured failure handling.

## Non-Goals

This model does not include:

- public raw thread creation as the recommended model
- public raw process pools as the recommended CPU model
- public event-loop objects or event-loop policy mutation
- implicit detached tasks
- implicit offload of blocking or CPU-heavy calls
- implicit shared mutable memory
- arbitrary pickle-like process transport
- multiprocessing shared-memory surfaces without explicit ownership/drop/unlink rules
- compatibility modules that become immediately obsolete after production APIs ship

## Relationship To The Async Model

[async_concurrency_model.md](./async_concurrency_model.md) remains the canonical contract for async syntax, task scopes, async cancellation, async context managers, async iteration, and `sifr.asyncio` veneer boundaries.

This document extends that contract to all runtime-owned work. If the two documents appear to conflict, the implementation phase must either:

1. resolve the conflict in both docs before implementation starts, or
2. record an explicit phase decision and update the affected contract.
