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

Sifr does not expose separate Python-shaped concurrency worlds for `asyncio`, `threading`, `concurrent.futures`, `subprocess`, and `multiprocessing`. Those modules are evidence sources or legacy implementation debt to remove/diagnose, not future adapters. The production model is structured runtime work.

## Product Decision

All runtime-owned work belongs to a scope or another linear owner. Child work must not silently outlive the owner that created it. A work handle must be awaited, joined, cancelled and joined, aggregated by its owning scope, or consumed by an explicit collection such as `JoinSet`.

Canonical user-facing direction:

```sifr
async with task.TaskGroup[AppError]() as group:
    users = group.spawn(fetch_users())
    config = group.spawn_blocking(read_config)
    index = group.spawn_cpu(build_index)
    child = group.spawn_process(process.Command("worker"))  # supervised; pipe access shape settled in M0

    users_result = await users
    config_result = await config
    index_result = await index
```

The exact method names are settled by the implementation phase. The semantic constraint is fixed: async, blocking, CPU, and process work participate in scoped lifetime, observation, cancellation, and typed failure evidence.

## Work Kinds

| Work kind | Public model | Execution substrate |
| --- | --- | --- |
| async coroutine work | `sifr.task.TaskHandle`, `TaskGroup` | Tokio task internals hidden behind Sifr APIs |
| blocking I/O offload | scoped `spawn_blocking` returning a `BlockingTask`-like handle | Tokio blocking pool |
| CPU-heavy offload | scoped `spawn_cpu`, `sifr.parallel`, `Pool` | private Rayon-backed pools |
| long-running child process | `sifr.process.Child` plus scoped supervision | `tokio::process`, `std::process`, host process APIs |
| future typed process worker | future worker handle over typed IPC | `sifr.process` plus `sifr.ipc` frames |

Threads and processes are execution substrates, not the public model. Users choose the kind of work and boundary they need; Sifr provides the scope, typed handle, safe communication, and deterministic cancellation/failure behavior.

`TaskGroup[E]` is the canonical owner for mixed runtime work under the fail-fast structured-concurrency policy. A distinct `task.Scope` or `runtime.Scope` type is introduced only if M0 identifies a concrete use case `TaskGroup[E]` cannot satisfy; M0 must record that finding before M1 starts. Individual child handles may have different success types. Homogeneous result collections such as `join_all`, `race`, `select`, and `JoinSet[T, E]` require one result/error shape unless the user constructs an explicit sum/enum result type.

M0 must test the `TaskGroup[E]`-only owner model against shutdown, child process supervision with pipe-pump tasks, mixed blocking/async workloads, and CPU cancellation. Non-fail-fast ownership requires an explicit `task.Scope` or `runtime.Scope`.

Scoped offload inserted into `TaskGroup[E]` maps user errors plus runtime/offload failures into the group's error type or an accepted wrapper such as `WorkerError[E]`. Scoped process spawn must preserve owned pipe access while binding child lifetime to the parent scope; M0 decides whether it returns `Child`, `TaskHandle[Status, SubprocessError]`, or a distinct `ProcessHandle`.

`TaskHandle[T, E]` is the public affine observation handle name. `Task` may remain an internal type name only; exposing both names as public aliases is rejected unless a new Sifr-native API design proves separate semantics.

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
- existing `sifr.asyncio` veneer code is legacy implementation debt, not the runtime model
- `sifr.threading`, `sifr.concurrent`, and current `sifr.subprocess` are not production substrates and must resolve to removal, internal-test-only, or unsupported diagnostics

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

`IpcSerializable` is stricter than `Sendable`. A value can be safe to move inside one process but not safe to serialize across a process boundary. File handles, pipes, lock guards, and channel endpoints are not IPC-serializable unless a later design explicitly supports them. Initial IPC payloads are primitives, strings, bytes, serializable lists/maps, and generated records/enums.

## Communication Model

Communication has separate tiers. Typed IPC is important, but it is not a substitute for channels or process pipes.

| Communication case | Production substrate |
| --- | --- |
| same-process tasks | `sifr.sync` channels with bounded backpressure, close, and cancellation semantics |
| child subprocess I/O | `process.PipeReader` and `process.PipeWriter` over stdin/stdout/stderr |
| Sifr-to-Sifr process workers | future `sifr.ipc.Connection[Req, Res, Err]` typed frames |

Typed IPC sits above an accepted process or transport substrate. It must define framing, versioning, request IDs, result/error frames, cancellation frames, stream close, malformed-frame errors, and backpressure. It is a prerequisite for future process workers and process pools, not a reason to ship process pools before the process substrate is ready.

Every typed IPC schema has stable schema identity/hash and an explicit compatibility policy. Exact schema hash proceeds, compatible version ranges proceed by negotiated version, and unknown or incompatible schema returns `Reject` or `UnsupportedSchema`.

IPC compatibility is generated from Sifr IPC schema definitions, not inferred dynamically from arbitrary runtime values.

## Synchronization Model

Synchronization APIs must say whether they are sync shared-state primitives or async coordination primitives.

| Category | Examples | Contract |
| --- | --- | --- |
| sync shared-state primitives | `Mutex[T]`, `RwLock[T]`, `Once` where accepted | guards cannot cross `await`; blocking lock acquisition is diagnosed in async contexts unless offloaded |
| async coordination primitives | `AsyncMutex[T]`, `AsyncRwLock[T]`, `Semaphore`, `Event`, `Notify`, `AsyncChannel[T]` | operations are real suspension points; guard await restrictions are documented and diagnosed |
| optional first-pass primitives | `Barrier`, public `Once` | public only if M0 finds near-term production need; otherwise `internal-only` or `deferred-to-phase-X` |

Default Sifr rule: sync lock guards cannot cross any `await`. Async lock guards may cross `await` only if the API explicitly marks the guard await-safe. M0 records whether each accepted async guard is await-safe, await-forbidden, or lint-only.

Semaphore permits are guard-like resources. M0 records whether permits may cross `await`, are await-forbidden, or are lint-only.

## Cancellation And Failure

Cancellation applies consistently across work kinds:

- cancellation is idempotent and produces typed evidence
- timeouts preserve the wrapped operation's normal typed outcome and add typed timeout evidence as a distinct variant
- `TaskGroup` exit reports unhandled child failures plus cancellation/cleanup evidence; a child result explicitly awaited and statically handled under the M0 proof is observed and does not by itself fail group exit
- `race` and `select` cancel losers and return loser-cancellation evidence
- blocking work has limited cancellation evidence, including whether work had already started or completed
- CPU-heavy work declares whether cancellation is cooperative, boundary-only, or wait-for-completion
- child process cancellation escalates through request shutdown, terminate, then kill where the host supports it
- future IPC worker cancellation sends a typed cancel frame before process escalation when the protocol is still live
- cleanup scopes run under cancellation and report cleanup failures without hiding the initiating failure

Current task cancellation is mostly abort-based. The production phase must explicitly decide which abort-based behavior remains v1 semantics and whether a cooperative cancellation layer is introduced internally. Tokio or tokio-util token types must not leak publicly; a Sifr-owned cancellation scope handle, named `CancelScope` or another M0-recorded name, is a settled stable API. M0 records its concrete public type name and Rust implementation boundary.

M0 defines static handled-failure proof for `TaskHandle` observation. It distinguishes awaited-and-ignored, awaited-and-assigned-but-uninspected, exhaustively matched, propagated with `?`, converted into another error, and explicit intentional discard if accepted.

Minimum `CancelOutcome` states are `Cancelled`, `AlreadyCompleted`, `AlreadyFailed`, `AlreadyStarted`, `CouldNotCancel`, `CancelFailed`, and `TimedOutDuringCancel`.

`race` and `select` return containers, not ad hoc tuples. `race` records winner index, typed outcome, and loser cancellation evidence. `select` records winner branch tag, typed outcome, and loser cancellation evidence. Loser evidence is `list[CancelOutcome]` unless M0 records a stricter equivalent container. Concrete names and generic parameters are public API boundary decisions for M0.

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

Existing `sifr.subprocess` is legacy implementation debt. Production behavior must not depend on it, it must not be extended by this substrate phase, and M0 records whether it is removed, kept internal-test-only, or routed to unsupported diagnostics. `sifr.process` is the only accepted public process API.

Process supervision distinguishes expected exit from unexpected exit. Normal exit, nonzero exit, signal termination, timeout, and parent cancellation map to success, typed error, or cancellation evidence.

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

Rejected or evidence-only CPython-shaped namespaces:

- `sifr.asyncio`
- `sifr.threading`
- `sifr.concurrent.futures`
- `sifr.subprocess`
- `sifr.multiprocessing`

These names are not compatibility adapters and are not fallback paths. Existing implementations are removed, kept internal-test-only, or routed to unsupported diagnostics by the phase inventory.

Public Sifr APIs must not expose Tokio, Futures, Rayon, Crossbeam, Rustix, tracing, metrics, serde, postcard, thiserror, or other Rust implementation crate types.

General dependency-ring rules live in [dependency_policy.md](./dependency_policy.md). The accepted Rust implementation crate set for the concurrency/runtime substrate is locked in [ad-hoc-production-concurrency-runtime-platform-substrate.md](../issues/ad-hoc-production-concurrency-runtime-platform-substrate.md#rust-ecosystem-decisions). This model document does not reopen dependency choices; implementation uses that phase table and changes it only through an explicit issue/phase amendment before implementation work starts.

## M7 Production Closure Audit

M7 closes the internal architecture gate by treating the production runtime model as one integrated substrate. Earlier sections preserve the design history; this table is the terminal audit surface for implementation and future maintenance.

| Boundary | Terminal M7 contract | Evidence |
| --- | --- | --- |
| Task ownership | Child async work is owned by `sifr.task` scopes/groups; handles are affine and must be observed, cancelled, joined, or consumed by an accepted collection. Detached ambient tasks remain rejected. | M1 traceability plus `docs/concurrency_runtime.md` `sifr.task` section. |
| Process ownership | `sifr.process` owns sync/async subprocess execution, explicit shell effects, owned pipes, timeout/cancel/kill/terminate, and status/output evidence. Legacy `sifr.subprocess` and CPython `Popen` parity are not production APIs. | M4 process traceability, supported-host matrix, public docs. |
| Channels and synchronization | Same-process communication uses `sifr.sync` typed channels, close/drain, bounded backpressure, cancellation, locks, semaphores, notifications, and explicit `Shared[T]`. Guards and permits are scoped resources and cannot cross invalid await/work boundaries. | M2 sync traceability, architecture concurrency safety section. |
| Blocking and CPU offload | Blocking I/O and CPU-heavy work require explicit annotations/offload. `sifr.runtime`, `sifr.parallel`, `task.spawn_blocking`, CPU spawn, and `JoinSet` evidence form the accepted offload model; direct async-context calls are diagnostics. | M3 offload traceability and generated-code panic-boundary evidence. |
| Sendability and shareability | Task/thread/offload boundaries require owned sendable values. Shared immutable values require `ShareSafe`; shared mutable state requires explicit sync wrappers. The compiler never inserts hidden thread-safe wrappers. | `internal_docs/architecture.md` concurrency safety contract and M1-M3 diagnostics. |
| Task and request context | Task context is explicit through `sifr.task.Context`, `ContextKey[T]`, `empty_context`, `current_context`, and accepted propagation APIs. Implicit CPython `contextvars` copying and global task-local mutation are rejected. | M5 task-context traceability and public docs. |
| Diagnostics and signal global state | Runtime diagnostics are structured events with redaction rules. Signal APIs expose values and structured shutdown streams; global handler mutation, warning filters as runtime control, and host process-global signal mutation are rejected. | M5 shutdown/diagnostics traceability, supported-host matrix, public docs. |
| Typed IPC policy | `sifr.ipc` is the typed schema/frame substrate over accepted process transports. It includes schema identity, protocol negotiation, frame codec, request tracking, payload eligibility diagnostics, Unix fixture-worker composition proof, and host-limited Windows fixture follow-up. Public worker pools and public `ipc.Connection` remain deferred. | M6 typed IPC design, host matrix, public docs. |
| Rejected CPython-shaped surface index | `sifr.asyncio`, `sifr.threading`, `sifr.concurrent.futures`, `sifr.subprocess`, `sifr.multiprocessing`, CPython queues, raw event-loop policy, global signal handlers, cleanup stacks, process pools, and multiprocessing shared memory are rejected, unsupported, evidence-only, or deferred according to the inventory. They are not fallback paths. | `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md` and substrate inventory. |

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
- CPython-shaped modules that are evidence-only and would be immediately superseded by production APIs

## Relationship To The Async Model

[async_concurrency_model.md](./async_concurrency_model.md) remains the canonical contract for async syntax, task scopes, async cancellation, async context managers, and async iteration. Any existing `sifr.asyncio` veneer boundary is implementation debt to resolve during the phase inventory, not a production API promise.

This document extends that contract to all runtime-owned work. If the two documents appear to conflict, the implementation phase must either:

1. resolve the conflict in both docs before implementation starts, or
2. record an explicit phase decision and update the affected contract.
