# Ad Hoc Phase: Production Concurrency, Process, And Runtime Substrate

Status: draft
Phase placement: second implementation phase in the split production-stdlib substrate sequence, after the text/Unicode/encoding/i18n runtime phase and before the network/HTTP platform substrate phase.
Phase owner: runtime/stdlib implementation with compiler effect, ownership, import, and codegen support

## Objective

Build the production-grade concurrency, scheduling, synchronization, subprocess, shutdown, diagnostics, and offload substrate required by real Sifr programs and by later web, worker, data, CLI, and interop phases.

This is a consolidation and production-hardening phase over Sifr's existing structured async/task lowering, not a blank-slate concurrency design. The implementation already points toward scoped tasks, typed handles, typed observation results, abort-based cancellation evidence, send/share diagnostics, blocking offload handles, and primitive channels. This phase finishes that model and extends it to production channels, offload, process supervision, shutdown, diagnostics, and typed IPC.

This phase is not a mandate to clone CPython's `asyncio`, `queue`, `concurrent.futures`, `multiprocessing`, `contextlib`, `warnings`, or `signal` modules. CPython remains required source/test evidence, but CPython module shape is no longer the completion target.

The required output is:

- native structured async tasks and task groups
- cancellation, deadlines, timeouts, and typed task failure aggregation
- async synchronization and bounded backpressure channels
- explicit blocking-I/O and CPU-heavy offload paths
- native subprocess/process supervision with owned pipes
- structured signal/shutdown streams
- typed worker, task, subprocess, signal, and diagnostic errors
- sendability/shareability rules for values crossing task, thread, and process boundaries
- typed IPC foundation for future process workers
- deterministic cleanup scopes
- panic-free emitted Rust and no public Tokio/runtime leaks

CPython-shaped APIs are not implemented in this phase. Later issues may add thin adapters over these primitives only when they are production-useful, maintainable, current, non-deprecated, and compatible with Sifr's static typed ownership model.

This phase does not add backward-compatibility or legacy support. Bare CPython stdlib imports, historical aliases, deprecated APIs, compatibility shims, fake generator paths, pickle-style fallbacks, hidden bridge names, and partial public toy modules are not implemented; they receive diagnostics or waivers.

## Related Phases

- Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md).
- Text/Unicode/encoding/i18n runtime substrate is tracked in [ad-hoc-production-text-i18n-platform-substrate.md](./ad-hoc-production-text-i18n-platform-substrate.md).
- This phase is second in the split production-stdlib sequence: text/i18n runs first, this phase closes the production runtime/process substrate, and network/HTTP runs third on top of both provider phases.
- Subprocess text mode, warning output encodings, locale-aware formatting, and demos relying on `open(..., encoding=...)` depend on text/i18n `milestone_text_i18n_1: Encoding And Explicit Text I/O`.
- Network/HTTP depends on this phase for production task, cancellation, shutdown, blocking/offload, diagnostics, and server/process lifecycle behavior.
- This phase assumes [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md) is complete: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.
- This phase consumes the shared platform contract in [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md). Concurrency M0 must use that contract's terminal states, stability levels, ownership/lifetime rules, cancellation/backpressure semantics, typed error nesting, observability fields, supported-host matrix, security/resource ownership table, and cross-phase golden fixture manifest.

## Support Tiers

Every proposed API, test family, and CPython-derived surface must be assigned one support tier during M0. Authoritative terminal states and stability levels come from [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md); the table below is this phase's domain view.

| Tier | Meaning | Examples |
| --- | --- | --- |
| `production-substrate` | Required runtime foundation for real programs and later phases | scheduler, cancellation, channels, sync primitives, process runtime, signals |
| `production-public` | Recommended Sifr API for user code | `sifr.task`, `sifr.sync`, `sifr.process`, `sifr.runtime`, `sifr.parallel`, `sifr.signal`, `sifr.resource` |
| `internal-only` | Implementation detail only | Tokio runtime, Tokio process, Tokio sync, crossbeam/Rayon-like internals |
| `compat-adapter` | Optional wrapper over production APIs after the substrate is complete and migration value is proven | selected `sifr.asyncio`, selected `sifr.subprocess`, selected `sifr.concurrent.futures` |
| `deferred-to-phase-X` | Potential future production API requiring a separate named design gate | process workers, selected queue adapters, task context extensions |
| `rejected` | Too dynamic, global, Python-specific, legacy, or unsafe for Sifr | `threading` parity, raw event-loop policies, callback transports, pickle transport, `signal.signal` handlers |

No phase milestone may be marked complete while any surface remains unclassified.

## Cross-Phase Dependency Contract

The split phase order is explicit:

1. Text/Unicode/encoding/i18n runtime.
2. Concurrency/process/runtime substrate.
3. Network/HTTP platform substrate.

This phase starts after the text/i18n provider phase has established the shared encoding, Unicode, explicit text I/O, and locale/i18n gates. It runs before network/HTTP so network servers and clients consume the production task, cancellation, shutdown, offload, diagnostics, and process model rather than adding local runtime substitutes.

- Text/i18n `milestone_text_i18n_1` is the hard prerequisite for subprocess text mode, warning output encodings, locale-sensitive warning formatting, and demos that rely on encoded text I/O.
- Network/web owns network stream compatibility entry points such as `asyncio.open_connection`; this phase owns the native task/sync/process substrate those entries consume.
- Async scheduler/task primitives are prior runtime infrastructure, but this phase owns their production closure states when they are consumed by queue/process/offload APIs.
- This phase owns the private thread/offload substrate for blocking and CPU-heavy work, but it does not add a public `threading` module.

## Source Of Truth

The authoritative CPython source tree for evidence is:

- `/Users/yaseralnajjar/work/sifr/cpython`

The implementation must scan and classify these CPython files before each milestone implementation PR. The scan is evidence for behavior, naming, and test coverage; it is not an instruction to expose each CPython module as a production Sifr API.

| Domain | CPython library sources | CPython test sources | Native backing sources |
| --- | --- | --- | --- |
| subprocess/process | `Lib/subprocess.py`, `Lib/asyncio/subprocess.py`, `Doc/library/subprocess.rst`, `Doc/library/asyncio-subprocess.rst` | `Lib/test/test_subprocess.py`, `Lib/test/test_asyncio/test_subprocess.py` | `Modules/_posixsubprocess.c`, `Modules/clinic/_posixsubprocess.c.h` |
| queue/concurrency | `Lib/queue.py`, `Lib/asyncio/*.py`, `Lib/concurrent/futures/*.py`, `Lib/multiprocessing/*.py`, `Doc/library/queue.rst`, `Doc/library/asyncio.rst`, `Doc/library/concurrent.futures.rst`, `Doc/library/multiprocessing*.rst` | `Lib/test/test_queue.py`, `Lib/test/test_asyncio/test_queues.py`, `Lib/test/test_asyncio/test_tasks.py`, `Lib/test/test_asyncio/test_taskgroups.py`, `Lib/test/test_asyncio/test_waitfor.py`, `Lib/test/test_asyncio/test_timeouts.py`, `Lib/test/test_asyncio/test_locks.py`, `Lib/test/test_asyncio/test_runners.py`, `Lib/test/test_concurrent_futures/`, `Lib/test/_test_multiprocessing.py`, `Lib/test/test_multiprocessing_main_handling.py`, `Lib/test/test_multiprocessing_spawn/`, `Lib/test/test_multiprocessing_fork/`, `Lib/test/test_multiprocessing_forkserver/` | `Modules/_queuemodule.c`, `Modules/_multiprocessing/*`, `Modules/clinic/_queuemodule.c.h` |
| context/warnings/signal | `Lib/contextlib.py`, `Lib/warnings.py`, `Doc/library/contextlib.rst`, `Doc/library/warnings.rst`, `Doc/library/signal.rst` | `Lib/test/test_contextlib.py`, `Lib/test/test_contextlib_async.py`, `Lib/test/test_warnings/`, `Lib/test/test_signal.py`, `Lib/test/test_io/test_signals.py` | `Modules/signalmodule.c`, `Python/_warnings.c`, `Lib/_py_warnings.py` |

Path note: CPython paths above are relative to `/Users/yaseralnajjar/work/sifr/cpython`.

## Current Sifr Baseline

- `sifr.subprocess` has sync helpers and `CompletedProcess`, but no owned process object, pipe lifecycle, timeout, signal, or async subprocess model.
- `sifr.asyncio` is a compatibility veneer over the canonical task model and intentionally omits raw event loops, subprocesses, process pools, and transport/protocol APIs.
- Core scheduler/task helpers are existing async-model infrastructure, not CPython module scope to duplicate.
- Existing lowering already treats `TaskScope`/`TaskGroup` as special structured-concurrency forms, rejects unscoped top-level `task.spawn(...)`, requires direct coroutine calls for v1 scoped spawn, checks task-boundary sendability, marks task handles observed when awaited/joined, and treats `Task`/`BlockingTask` as affine observation handles.
- Current generated async runtime support is emitted in Rust preambles when needed rather than centralized in `sifr_runtime`; M0 must decide which pieces remain generated preamble and which move to shared runtime crates without changing the public Sifr model.
- Current cancellation is mostly abort-based through Tokio task handles and timeout aborts. M0 must either ratify that as the production v1 semantic or introduce a cooperative cancellation-token layer through the shared platform cancellation contract.
- `sifr.sync` channels are the canonical queue-like primitive, but the production channel/backpressure/sync surface is not yet closed for all worker/process use cases.
- Current generated channel support proves the direction but is not production backpressure: it uses generated Rust state and yield loops for full/empty states. M2 replaces or hardens it with explicit bounded capacity, close, cancellation, and fairness behavior.
- Current `sifr.threading.Thread` and `sifr.concurrent` surfaces are placeholders, not production execution primitives. This phase must not build user-visible concurrency around them.
- Current CPython-shaped `sifr.contextlib`, `sifr.warnings`, `sifr.queue`, `sifr.concurrent.futures`, and `sifr.multiprocessing` surfaces are not production substrate surfaces. CPython-shaped `sifr.signal` entries must be classified rather than assumed; structured signal streams are the accepted production direction.

The Phase 32 async model remains binding. Source-of-truth references:

- [internal_docs/async_concurrency_model.md](../internal_docs/async_concurrency_model.md)
- [internal_docs/structured_runtime_work_model.md](../internal_docs/structured_runtime_work_model.md)
- [internal_docs/phases/32_async_ecosystem.md](../internal_docs/phases/32_async_ecosystem.md)

- Native async process, queue/channel, and synchronization APIs must be real suspension points.
- Sync APIs that can block on I/O, process, channel, lock, or external runtime state must be classified as `@blocking_io` or a narrower workload/effect class.
- CPU-heavy work must use the existing `@cpu_heavy`/offload model.
- Direct calls to blocking sync APIs from `async def` remain compiler errors unless routed through native async APIs or explicit offload.
- The compiler must not expose Tokio, event-loop objects, raw callback transports/protocols, or runtime internals as the normal user model.

`@blocking_io`, `@cpu_heavy`, direct-call async diagnostics, and offload-target validation already exist from Phase 32. This phase extends the stdlib workload database for newly added runtime APIs and adds any missing diagnostics needed by task/thread/process boundary captures.

## Product Boundary

The right production Sifr concurrency story is:

- `sifr.task`: structured async tasks, task groups, cancellation, deadlines, typed task failures
- `sifr.sync`: channels, async channels, locks, semaphores, events, backpressure
- `sifr.runtime`: blocking/cpu offload, scheduler boundaries, typed worker failures
- `sifr.parallel`: data-parallel CPU work
- `sifr.process`: subprocesses, async processes, pipes, structured termination
- `sifr.signal`: structured shutdown streams
- `sifr.resource`: deterministic cleanup stacks and resource scopes
- `sifr.ipc`: typed IPC foundation for future process workers

The wrong production target is:

- `sifr.asyncio` parity as the primary async model
- `sifr.queue` parity as the primary queue model
- `sifr.concurrent.futures` parity as the primary offload model
- `sifr.multiprocessing` parity as the CPU parallelism story
- `sifr.threading` parity
- `sifr.warnings` parity with Python global warning filters

Selected adapters can exist only in separate future adapter issues after this substrate is complete. They must delegate to the production APIs above and must not introduce legacy behavior or dynamic fallback paths.

## Structured Runtime Work Model

Sifr exposes structured runtime work, not separate Python-shaped worlds for async tasks, threads, executors, subprocesses, and multiprocessing.

A scope owns runtime work. Work returns an affine handle. The handle must be awaited, joined, cancelled and joined, consumed by a collection such as `JoinSet`, or aggregated by its parent scope. Failures and cancellation are typed evidence. Child work must not silently outlive its owning scope.

The conceptual work kinds are:

| Work kind | Public model | Execution substrate |
| --- | --- | --- |
| async coroutine work | `sifr.task.TaskHandle` / `TaskGroup` | Tokio task internals hidden behind Sifr APIs |
| blocking I/O offload | scoped `spawn_blocking` returning `BlockingTask`-like handle | Tokio blocking pool |
| CPU-heavy offload | scoped `spawn_cpu` / `sifr.parallel` / `Pool` | Rayon-backed private pools |
| long-running child process | `sifr.process.Child` plus scoped supervision task | `tokio::process` / `std::process` |
| future Sifr process worker | future worker handle over typed IPC | process substrate plus `sifr.ipc` frames |

This phase does not expose raw threads as the user model. Threads are internal execution substrate for blocking work, CPU work, and runtime internals. Processes are supervised resources/work units for external tools, isolation, crash containment, and future typed workers.

Canonical user-facing direction:

```sifr
async with task.TaskGroup[AppError]() as group:
    users = group.spawn(fetch_users())
    config = group.spawn_blocking(read_config)
    index = group.spawn_cpu(build_index)
    child = group.spawn_process(process.Command("worker"))
```

The exact `spawn_blocking`, `spawn_cpu`, and `spawn_process` method signatures are settled in M0/M3/M4, but the ownership rule is not optional: offload and process work must participate in scoped observation and cancellation. Module-level helpers may exist only when they still require an active structured scope or return a linear handle whose observation is compiler-enforced.

`task.Scope` or `runtime.Scope` is the general owner for mixed runtime work. M0 chooses the public name and method placement. `TaskGroup[E]` is the fail-fast structured-concurrency policy for child work that shares one error type; individual handles may have different success types. Homogeneous result collections such as `join_all`, `race`, `select`, and `JoinSet[T, E]` require one result/error shape unless the user constructs an explicit sum/enum result type.

Scoped offload inserted into `TaskGroup[E]` must map user errors plus runtime/offload failures into the group's error type or an accepted wrapper such as `WorkerError[E]`; M0 records the exact error shape. Scoped process spawning must preserve owned pipe access while binding child lifetime to the parent scope; M0 decides whether it returns `Child`, `TaskHandle[Status, SubprocessError]`, or a distinct `ProcessHandle`.

`TaskHandle[T, E]` is the public affine observation handle name. `Task` may remain an internal type name or compatibility alias only if M0 records a concrete reason to expose both names.

`sifr.task` is the canonical public namespace for scoped runtime work. Existing `sifr.asyncio` remains a frozen `compat-adapter` and must not be extended as the primary async API. `sifr.threading`, `sifr.concurrent.futures`, `sifr.subprocess`, and `sifr.multiprocessing` remain evidence/adapters, not the runtime spine.

## Boundary And Communication Model

Structured lifetime is separate from boundary safety. Every work boundary must declare the value-safety rule it enforces:

| Boundary | Required contract |
| --- | --- |
| async task boundary | captured values must satisfy task sendability and cannot borrow beyond their lexical owner |
| blocking/thread boundary | captured inputs, outputs, and errors must be sendable and observed through typed handles |
| CPU-parallel boundary | items, closure captures, results, and errors must be sendable; shared state must use explicit sync wrappers |
| process boundary | payloads must be explicitly pipe-owned bytes/text or `IpcSerializable` typed frames |
| shared-state boundary | shared references require immutable data or explicit synchronization wrappers |

Communication has three tiers:

| Communication case | Production substrate |
| --- | --- |
| same-process tasks | `sifr.sync` channels with bounded backpressure, close, and cancellation semantics |
| child subprocess I/O | `process.PipeReader` / `process.PipeWriter` over stdin/stdout/stderr |
| Sifr-to-Sifr process workers | future `sifr.ipc.Connection[Req, Res, Err]` typed frames over process pipes or another approved transport |

Typed IPC is required for future process workers, but it does not replace channels or process pipes. It sits above the process substrate and must define framing, versioning, cancellation messages, payload eligibility, malformed-message behavior, and backpressure.

`IpcSerializable` is stricter than `Sendable`: file handles, pipes, lock guards, and channel endpoints may be sendable inside one process but are not IPC-serializable unless a later design explicitly supports them. IPC-serializable values start with primitives, strings, bytes, serializable lists/maps, and generated records/enums. Every IPC schema must have stable schema identity/hash and a compatibility policy: exact schema hash proceeds, compatible version range proceeds by negotiated version, and unknown or incompatible schema returns `Reject`/`UnsupportedSchema`.

## No-Toy-Concurrency Gate

A public concurrency/runtime API may be added only if it satisfies at least one of:

1. It is required production runtime substrate.
2. It is the recommended Sifr API for real user code.
3. It is required by a near-term production phase such as web, HTTP client, workers, data processing, CLI, or interop.
4. It is a thin adapter over a production API with proven migration value.

The following are not sufficient reasons:

- CPython has this module.
- A CPython test exists for it.
- It is easy to partially implement.
- It is useful for demos only.
- It can be marked "basic" and fixed later.

Partial public concurrency modules are rejected unless explicitly unstable/internal and hidden from stable user imports.

## Namespace Contract

This phase uses Sifr's canonical `sifr.*` namespace.

1. Production public APIs use names such as `sifr.task`, `sifr.sync`, `sifr.process`, `sifr.runtime`, `sifr.parallel`, `sifr.signal`, and `sifr.resource`.
2. Optional adapters, if later accepted, also live under `sifr.*`, such as `sifr.asyncio`, `sifr.subprocess`, or `sifr.concurrent.futures`.
3. Bare CPython module-name imports are not aliases. Bare forms such as `from queue import Queue`, `from subprocess import Popen`, or `from concurrent.futures import ThreadPoolExecutor` receive the namespace-contract diagnostic once normal user/package resolution fails.
4. Every embedded stdlib module added by this phase must have canonical `sifr.*` import-resolution tests and negative diagnostics for unsupported bare CPython import forms.

## Rust Lowering Contract

Sifr concurrency APIs lower to Rust/Tokio/Rayon-like primitives without exposing those primitives publicly.

Required invariants:

- no public Tokio types
- no raw event-loop handles
- no user-triggerable `.unwrap()`, `.expect()`, or `panic!`
- values crossing task, thread, or process boundaries must satisfy Sifr sendability rules
- shared state must use explicit synchronization
- async tasks must not run blocking work directly
- blocking I/O and CPU-heavy work must use explicit offload
- task failures, cancellation, worker runtime failures, malformed IPC, and foreign/runtime boundary failures must become typed evidence
- generated Rust for user-controlled process, pipe, queue/channel, signal, cleanup, diagnostic, and executor data remains panic-free

## Rust Ecosystem First

This phase builds a Sifr concurrency/runtime platform, not a new async runtime, channel library, process supervisor, tracing stack, cleanup stack, error framework, IPC serializer, or CPU scheduler. The implementation must prefer wrapping, constraining, and testing mature Rust ecosystem crates wherever they satisfy Sifr's static typing, ownership, cancellation, sendability/shareability, panic-free, host-matrix, license, binary-size, and maintenance requirements.

M0 must produce a dependency decision record for every crate family below. Each decision must include accepted crate and feature flags, Sifr abstraction that hides crate types from public APIs, cancellation/drop semantics, sendability/shareability mapping, panic/unsafe audit for user-controlled data paths, typed error mapping into Sifr variants, license/MSRV/binary-size/platform impact, deterministic local test strategy, and supply-chain/maintenance signal.

| Area | Preferred crate families | Role |
| --- | --- | --- |
| Async task runtime, timers, process I/O, and signals | `tokio` plus `tokio-util` `CancellationToken`/stream helpers | structured task execution, timers/deadlines, async subprocess pipes, signal streams, and cancellation building blocks behind Sifr APIs. |
| Future combinators and stream utilities | `futures` / `futures-util` where they reduce bespoke combinator code | internal join/race/select, stream adaptation, and poll-safe utility behavior behind Sifr APIs. |
| Async synchronization and notifications | `tokio::sync` behind Sifr-owned wrappers | async mutexes, semaphores, notify/event behavior, and bounded async backpressure while preserving Sifr ownership and typed diagnostics. |
| Sync channels and cross-thread queues | `crossbeam-channel` for sync channels; `tokio::sync::mpsc` for async channels; `flume` is not used in this phase | producer/consumer pipelines, bounded/unbounded channels, close/drop behavior, and task/thread handoff behind `sifr.sync`. |
| Low-level locking and once/cell utilities | `std::sync`, `parking_lot`, `once_cell`, and `scopeguard` | mutex/rwlock/once/drop cleanup internals while preserving Sifr's explicit ownership and no-panic contract. |
| CPU parallelism and work stealing | `rayon` | `sifr.parallel` CPU-heavy offload and typed result collection without exposing Rayon types. |
| Blocking offload | Tokio blocking pool for blocking I/O; `rayon` for CPU-heavy parallel work | `sifr.runtime.spawn_blocking`, workload diagnostics, shutdown behavior, and capacity policy without a bespoke thread pool. |
| Process/subprocess execution | `tokio::process`, `std::process`, and `rustix` for Unix host-limited process details | owned child processes, pipes, timeout/termination behavior, and async/sync process supervision. |
| Signals and shutdown | `tokio::signal` for async signal streams; `rustix` for Unix host-limited process/signal details | structured signal streams and shutdown events without unsafe user signal handlers; `signal-hook` and `nix` are not used in this phase. |
| Diagnostics, tracing, and task context | `tracing`, `tracing-subscriber`, and `metrics`; explicit Sifr context objects for propagation | structured runtime diagnostics, deprecation metadata, and task/request context hooks without Python-style global warnings or implicit contextvars behavior. |
| Typed error implementation | `thiserror` only as an internal Rust implementation aid | concise typed Rust error definitions while preserving Sifr `Result`/typed-sum semantics at the language boundary. |
| IPC serialization and process-worker payloads | `serde` plus `postcard`; `bincode` is not used in this phase | typed IPC payload encoding after the sendability/shareability and IPC gates are complete. |

From-scratch async runtimes, future combinators, channel implementations, lock/once utilities, work-stealing schedulers, process supervisors, signal handling frameworks, tracing systems, error frameworks, IPC serializers, or thread pools are rejected in this phase. If a listed Rust ecosystem option cannot satisfy Sifr's requirements, the affected surface becomes `deferred-to-phase-X` with evidence instead of receiving a bespoke implementation. Public Sifr APIs must never expose Tokio, Futures, Rayon, Crossbeam, Flume, Parking Lot, tracing, serde, thiserror, or platform-helper types directly.

## Sendability And Shareability Contract

Before scoped task spawning, blocking offload, CPU offload, thread pools, process workers, or cross-boundary closure captures ship, Sifr must define a user-visible sendability/shareability model:

- `Send[T]`-equivalent eligibility for moving values across task/thread/process boundaries
- `Sync[T]`/`Share[T]`-equivalent eligibility for shared references
- lifetime-like capture constraints for spawned work that may outlive the lexical caller
- non-send capture diagnostics
- shared mutable state diagnostics
- safe closure capture rules for task, blocking, CPU, and process-worker APIs
- typed compile-time diagnostics where unsupported values cross worker boundaries

This is a phase-wide gate, not an executor-only caveat. M0 defines the model, M1 owns the initial HIR/type-checker and codegen enforcement for task spawning and task handles, M2 extends enforcement to channel and synchronization value types, M3 extends verification to blocking/CPU offload and `sifr.parallel`, M4 extends verification to process/subprocess callbacks and captures, and M6 extends verification to typed IPC payloads. CPython-shaped executor and process-pool initializer APIs are `deferred-to-adapter-phase`; unsafe cross-boundary process-worker payload capture is rejected until typed IPC proves an accepted production process-worker API.

## Cancellation And Failure Contract

Cancellation and failure semantics apply to every structured work kind, not only async tasks:

- cancellation is idempotent and produces typed evidence
- timeout preserves the wrapped operation's normal typed outcome and adds typed timeout evidence as a distinct variant
- `TaskGroup` exit reports unhandled child failures plus cancellation/cleanup evidence; a child result explicitly awaited and handled by user code is observed and does not by itself fail group exit
- `race`/`select` cancel losers and return typed loser-cancellation evidence
- blocking offload has limited cancellation; cancellation evidence must say whether the work had already started or completed
- CPU-heavy work must define whether cancellation is cooperative, boundary-only, or wait-for-completion, with typed evidence
- child process cancellation escalates through the M4 policy: request shutdown where available, then terminate, then kill
- future IPC worker cancellation sends a typed cancel frame such as `Cancel(request_id)` before process escalation where the protocol is still live
- cleanup scopes run under cancellation and report cleanup failures without hiding the initiating failure

M0 must record which current abort-based implementation behavior is retained for M1 and whether a Sifr-owned cooperative cancellation scope is added. Tokio or tokio-util cancellation token types must not leak publicly.

## Typed Errors Instead Of Exceptions

All fallible APIs must expose typed error results:

- `TaskError`, `TaskGroupError`, `Cancelled`, `TimedOut`
- `CancelOutcome` for work that may already have started, completed, failed during cancellation, or resisted cancellation
- Minimum `CancelOutcome` states are `Cancelled`, `AlreadyCompleted`, `AlreadyFailed`, `AlreadyStarted`, `CouldNotCancel`, `CancelFailed`, and `TimedOutDuringCancel`
- `ChannelClosed`, `ChannelFull`, `ChannelEmpty`, `BackpressureError`
- `SubprocessError`, `CalledProcessError`, `TimeoutExpired`, `PipeError`
- `WorkerError`, `WorkerRuntimeError`, `OffloadError`
- `SignalError`, `ShutdownError`
- `ContextError` (M5), `DiagnosticError` (M5)

Names may align with CPython evidence where useful, but the operational contract is Sifr `Result`/`Option` and typed sums, not exception-driven control flow. Convenience APIs such as `check_output` adapters must return typed failure evidence instead of throwing.

## Non-Goals And Permanent Boundaries

The following are not accepted as silent omissions. They must be classified as `rejected`, `deferred-to-phase-X`, or `waived-with-rationale` with tests:

- raw event-loop policy mutation
- callback transport/protocol APIs as the primary Sifr model
- `contextvars` module parity, implicit Python context propagation, or dynamic task-local mutation
- public `threading` module parity
- `threading.local`
- Python `queue.Queue`/`PriorityQueue`/`LifoQueue`/`SimpleQueue` as the core queue model
- Python `concurrent.futures` as the core task/offload model
- arbitrary object pickling for process workers
- `multiprocessing` as the CPU parallelism model
- process pools without a stable typed IPC serialization contract
- `multiprocessing.Value`, `multiprocessing.Array`, and `multiprocessing.shared_memory`
- fork/forkserver semantics without host-specific ownership evidence
- `signal.signal` custom handler registration
- `signal.pause` in this phase
- mutation of interpreter-global warning/filter state from unstructured concurrent contexts
- `contextmanager` and `asynccontextmanager` until a future generator/async-generator semantics phase owns `send`/`throw`/`close` and async-generator cleanup
- `subprocess.getoutput` and `subprocess.getstatusoutput` as legacy shell helpers

## Milestone Dependency Graph

1. `milestone_concurrency_runtime_0` first. No implementation milestone starts until product tiers, CPython evidence inventory, typed error map, sendability/shareability gate, workload/effect classification, and shared terminal-state decisions are checked in.
2. `milestone_concurrency_runtime_1` defines the structured async task substrate and implements the first sendability/shareability compiler enforcement pass for task spawning. It blocks async synchronization, async subprocess, and offload APIs that need task cancellation/deadline semantics.
3. `milestone_concurrency_runtime_2` closes synchronization/backpressure on top of the task model.
4. `milestone_concurrency_runtime_3` implements blocking and CPU offload after sendability/shareability is accepted.
5. `milestone_concurrency_runtime_4` implements native process/subprocess after task, sync, and offload semantics are stable.
6. `milestone_concurrency_runtime_5` implements shutdown, signal streams, cleanup, task context, and structured diagnostics after the task/process contracts are available.
7. `milestone_concurrency_runtime_6` designs and implements typed IPC. Process-worker pools remain `deferred-to-phase-X` by design in this phase.
8. `milestone_concurrency_runtime_7` closes docs, demos, validation, waivers, and adapter decisions last.

## Milestones

### milestone_concurrency_runtime_0: Product Boundary And Rust Concurrency Contract

Scope:

- Add a machine-readable inventory under `verification/stdlib/concurrency_runtime_substrate_inventory.*`.
- Scan every source/test/doc file listed in `Source Of Truth`.
- Extract public functions, classes, constants, methods, common keyword forms, deprecation/legacy markers, and test-class/test-method names.
- Assign every inventory entry one support tier and one shared platform terminal state from [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md).
- Assign every public or semi-public surface one shared platform stability level.
- Assign every CPython test family one shared evidence state.
- Record the public/native API boundary for `sifr.task`, `sifr.sync`, `sifr.process`, `sifr.runtime`, `sifr.parallel`, `sifr.signal`, `sifr.resource`, and `sifr.ipc`.
- Record the current implementation audit: internal task/coroutine/timeout/select/blocking types, `TaskScope`/`TaskGroup` lowering, generated task runtime preamble, generated channel runtime replacement, workload/offload diagnostics, current subprocess intrinsics, and placeholder compatibility surfaces.
- Record the structured runtime work model, including whether scoped offload/process methods live on `TaskGroup`, `TaskScope`, `sifr.runtime`, or a combination that still enforces active-scope ownership.
- Record whether scoped process spawn returns `Child`, `TaskHandle[Status, SubprocessError]`, or a distinct `ProcessHandle`; the accepted API must preserve owned pipe access while binding child lifetime to the parent scope.
- Record sendability/shareability rules for task/thread/process captures.
- Record the HIR/type-checker/codegen ownership plan for sendability/shareability enforcement and the exact diagnostics/fixtures required in M1, M2, M3, M4, and M6.
- Use error-homogeneous task groups and homogeneous result collections: `TaskGroup[E]` may own child handles with different success types but one error type; `join_all`, `race`, `select`, and `JoinSet[T, E]` require one result/error shape unless users construct an explicit sum/enum type.
- Use structured tasks only: stable public tasks are structured by default, handle drop before failure observation is diagnosed, and detached tasks are rejected in this phase.
- Record `TaskGroup` versus `JoinSet`: `TaskGroup` is scoped structured concurrency with automatic cancellation on unhandled child failure; `JoinSet` is a dynamically-growable offload/task-result collection for collecting completed homogeneous work items without pretending to be a structured parent scope.
- Record `TaskGroup` observed/unobserved failure semantics, including whether an observed failure still triggers fail-fast sibling cancellation or only unhandled failures do.
- Record `race` and `select` result containers as part of the M0 public API boundary artifact, not an implementation detail.
- Record blocking/cpu-heavy effect rules.
- Record cancellation/deadline/timeout semantics.
- Record typed worker/task/process/signal/diagnostic error models.
- Use fixed default `sifr.parallel` pool sizing equal to `available_parallelism()` with optional `sifr.parallel.PoolConfig { workers: PositiveInt }`; no implicit global pool mutation API is exposed.
- Record the no-public-Tokio rule and Tokio feature plan for `tokio::process`, `tokio::io`, `tokio::sync`, `tokio-util`, and signal support.
- Create checked-in dependency decision records for every Rust Ecosystem First crate family, including accepted crate/features or rejection rationale.
- Record concurrency/runtime rows in the shared supported host matrix at `verification/platform/supported_host_matrix.md`, and require all host-limited subprocess/signal entries to reference that matrix. This supersedes any per-phase host matrix path.
- Add or update concurrency-owned entries in `verification/platform/golden/manifest.json` and ensure `scripts/run_platform_golden.sh` knows which entries are blocked by unfinished milestones.
- Map subprocess, IPC, cancellation, offload, queue bounds, signal, and shell-exec security/resource concerns to the shared platform contract.
- Record the stdlib workload database artifact for newly added runtime APIs, including owner, schema, blocking/CPU/suspension classification, and validation command.
- Implement explicit typed task/request context in M5 as `sifr.task.Context` and `sifr.task.ContextKey[T]` with explicit propagation only; no Python-style implicit `contextvars` behavior.
- Record the designated compiler/runtime reviewer and the typed IPC design approval process in the execution ledger.
- Record existing `sifr.asyncio` compatibility veneer as frozen `compat-adapter`: M1 does not build on it, extend it, or remove already-supported veneer entry points.
- Classify M5 convenience helpers (`redirect_stdout`, `redirect_stderr`, `chdir`, `suppress`, `contextmanager`, `asynccontextmanager`) during the M0 inventory.
- Copy the resolved decision register into the execution ledger before M0 closes.
- Add import-resolution tests for canonical `sifr.*` module names and negative diagnostics for bare CPython stdlib import attempts.
- Assign every deprecated, historical, or legacy-only CPython entry the terminal state `unsupported-with-diagnostic` or `rejected`.

Definition of done:

- The backlog is derived from CPython source/tests plus Sifr runtime needs, not hand-written memory.
- Every proposed API is classified by support tier and terminal state.
- M1-M7 implementation PRs have concrete backlog entries rather than prose-only scope; each backlog entry has at least one named fixture, acceptance criterion, or design artifact.
- No CPython module shape is still treated as automatic production scope.
- The resolved decision register is copied into the execution ledger, including task typing, detached tasks, task context, host matrix, workload database, adapter migration value, and Rust ecosystem choices.
- The shared platform contract artifacts are present or updated: `verification/platform/platform_contract.md`, `verification/platform/platform_contract.json`, `verification/platform/supported_host_matrix.md`, `verification/platform/golden/manifest.json`, and `scripts/run_platform_golden.sh`.
- Every accepted or rejected Rust ecosystem crate family has a checked-in dependency decision record before M0 closes. Each record includes accepted crate, feature flags, Sifr wrapper boundary, cancellation/drop semantics, sendability/shareability mapping, typed error mapping, and panic/unsafe audit for user-controlled paths.
- Pool-sizing policy for `sifr.parallel` is recorded in the execution ledger before M0 closes; M3 must not start until this entry exists.
- Reviewer identity is recorded in the execution ledger.
- Post-M0 external review is complete, has a `PASS` result, and is recorded in the planning reviews section of the execution ledger before M1 starts, or the five-working-day fallback review procedure is recorded with attempted review, open questions, conservative self-review, and no unresolved blocking questions.
- Import-resolution tests for canonical `sifr.*` names pass, and negative-diagnostic tests for bare CPython stdlib import forms pass.
- Sendability/shareability diagnostics have named representative fixtures assigned to the enforcing milestone.

### milestone_concurrency_runtime_1: Structured Async Runtime

Entry gate: the post-M0 external review recorded in the execution ledger must have a `PASS` result, or the M0 fallback review procedure must be recorded after five working days with conservative self-review and no unresolved blocking questions.

Scope:

- Add or close the production `sifr.task` surface:
  - `TaskHandle[T, E]`
  - `TaskGroup[E]`
  - `TaskGroup(ctx: Option[sifr.task.Context] = None)` constructor signature shape reserved for M5 context propagation
  - homogeneous task-result collections unless users explicitly define a sum/enum result type
  - `spawn_scoped(fn, *, ctx: Option[sifr.task.Context] = None)` signature shape reserved for M5 context propagation
  - `sleep`
  - `timeout`
  - `deadline`
  - `cancel_scope`
  - `join_all`
  - `race`/`select`
- `TaskGroup[E]` provides scoped fail-fast structured concurrency with automatic cancellation under the M0-recorded failure policy. It is error-homogeneous and result-heterogeneous through individual `TaskHandle[T, E]` values.
- A child failure explicitly awaited and handled by user code is observed and does not by itself fail `TaskGroup` exit. M0 records whether observed failures still trigger fail-fast sibling cancellation or only unhandled failures do.
- M1 consolidates the existing `TaskScope`/`TaskGroup` lowering contract instead of adding a parallel task model: top-level detached spawn remains rejected, direct coroutine-call requirements are recorded or deliberately relaxed with tests, handle observation remains affine, and borrowed/non-send captures remain diagnostics.
- `join_all`, `race`, and `select` require result-type unification or an explicit user sum/enum type; heterogeneous collections without an explicit sum are diagnostics.
- `race` accepts a homogeneous collection of awaitables and returns the first completed typed outcome plus its collection index; it cancels every still-pending loser and returns typed cancellation evidence for losers.
- `select` is the named-branch form for a statically known set of awaitable branches. It returns the winning branch tag plus the branch's typed outcome, requires branch result/error unification or an explicit user sum/enum type, cancels every still-pending loser, and returns typed cancellation evidence for losers.
- M0 records concrete result containers for `race` and `select`, including winner identity, typed outcome, and loser cancellation evidence.
- `timeout` wraps one awaited operation and returns the operation's normal typed outcome plus typed timeout evidence. Inner errors are preserved; timeout is a distinct error/evidence variant. `deadline` uses an absolute monotonic deadline; `cancel_scope` groups multiple child operations under a Sifr-owned cancellation scope if M0 accepts that public surface.
- Stable public tasks are structured by default. Handle drop before completion/failure observation is a diagnostic, and detached tasks remain `deferred-to-phase-X` unless a future API requires an explicit failure sink.
- Implement the first sendability/shareability compiler enforcement pass for task spawning:
  - HIR capture analysis for task closures and handles
  - diagnostics for non-send captures crossing task boundaries
  - diagnostics for shared mutable state without explicit synchronization
  - panic-free codegen for rejected task-boundary cases
- Ensure task failures cannot be silently dropped.
- Implement documented handle ownership for task/future observation and cancellation.
- Implement typed result/cancellation/timeout behavior for join/race/wait/timeout/task groups.
- Add CPython-derived evidence from `asyncio.TaskGroup`, `gather`, `wait`, `wait_for`, `timeout`, and `sleep`, but map production behavior to `sifr.task`.
- Keep existing `sifr.asyncio` as frozen `compat-adapter`; no new `sifr.asyncio` adapter is implemented in this phase.
- Reject raw event-loop policy, callback transport/protocol, and `contextvars` usages with diagnostics or waivers.

CPython tests to mine:

- `Lib/test/test_asyncio/test_tasks.py`
- `Lib/test/test_asyncio/test_taskgroups.py`
- `Lib/test/test_asyncio/test_waitfor.py`
- `Lib/test/test_asyncio/test_timeouts.py`
- `Lib/test/test_asyncio/test_runners.py`

Definition of done:

- Task lifetime and cancellation behavior is deterministic.
- Task failures and cancellation produce typed evidence.
- `spawn_scoped` and `TaskGroup` expose the reserved `ctx` parameter without changing runtime propagation before M5.
- Sendability/shareability diagnostics for task-boundary captures pass representative fixtures.
- `TaskGroup` exit reports `TaskGroupError[E]` for unhandled child failures plus sibling cancellation and cleanup evidence; explicitly handled child failures are not re-reported as group-exit failures.
- CPython async task test families are classified as `mined-as-substrate-fixture`, `adapted-for-sifr-api`, or `waived-with-rationale` against the Sifr-native task model.
- No raw Tokio/event-loop types leak.

### milestone_concurrency_runtime_2: Synchronization, Channels, And Backpressure

Scope:

- Add or close the production `sifr.sync` surface:
  - `Channel[T]`
  - `BoundedChannel[T]`
  - `UnboundedChannel[T]`
  - `AsyncChannel[T]`
  - sync `Mutex[T]` / `RwLock[T]`
  - async `AsyncMutex[T]` / `AsyncRwLock[T]` if M0 accepts async lock surfaces
  - `Semaphore`
  - `Event`/`Notify`
  - `Barrier` only if M0 finds near-term production need; otherwise `internal-only` or `deferred-to-phase-X`
  - `Once` only if M0 finds user-facing need; otherwise `internal-only`
- `Channel[T]` is the abstract sender/receiver channel contract. `BoundedChannel[T]`, `UnboundedChannel[T]`, and `AsyncChannel[T]` are the concrete production channel families.
- Implement documented bounded capacity, backpressure, close/drop, sender/receiver ownership, and cancellation behavior.
- Extend sendability/shareability enforcement to channel and sync primitive value types:
  - `Channel[T]`, `BoundedChannel[T]`, `UnboundedChannel[T]`, and `AsyncChannel[T]` reject non-send `T` where values can cross task/thread boundaries.
  - sync `Mutex[T]`/`RwLock[T]` guards cannot cross `await`.
  - accepted async lock guards have documented await restrictions and diagnostics.
  - default rule: lock guards, including async lock guards, do not cross unrelated `await` points unless the API explicitly marks the guard await-safe; M0 records whether each accepted async guard is await-safe, await-forbidden, or lint-only.
  - `Semaphore`, `Event`, `Notify`, accepted `Barrier`, and accepted `Once` must not smuggle non-send captured state through waiters or callbacks.
- Mark sync operations that can block on I/O, process, channel, lock, or external runtime state as `@blocking_io` or a narrower workload/effect class when used from async contexts.
- Keep `sifr.queue` and `sifr.asyncio.Queue` as `deferred-to-adapter-phase`; no queue adapter is implemented in this phase.
- Defer `PriorityQueue`, `LifoQueue`, `SimpleQueue`, and `task_done`/`join` accounting to a future adapter issue.

CPython tests to mine:

- `Lib/test/test_queue.py`
- `Lib/test/test_asyncio/test_queues.py`
- `Lib/test/test_asyncio/test_locks.py`

Rust/runtime candidates:

- `crossbeam-channel`
- `tokio::sync`

Definition of done:

- Producer/consumer pipelines are test-covered.
- Cancellation and backpressure have deterministic behavior.
- Channel and sync-primitive sendability/shareability diagnostics pass representative fixtures.
- Blocking-in-async diagnostics cover sync waits.
- No raw Tokio sync type leaks.
- CPython queue/async-queue test families are classified with shared evidence states, including `compat-adapter-deferred` where adapter-only.

### milestone_concurrency_runtime_3: Blocking And CPU Offload

Entry gate: M2 sendability/shareability enforcement must be complete, and `sifr.parallel` pool-sizing policy must be recorded in the execution ledger before M3 starts.

Scope:

- Add or close the production `sifr.runtime` and `sifr.parallel` offload surface:
  - `spawn_blocking`
  - `spawn_cpu`
  - scoped offload methods on the accepted scope/group API where M0 places them
  - `JoinSet[T, E]` dynamically-growable collection for homogeneous completed work items
  - `JoinSet.spawn_blocking(fn: Callable[[], Result[T, E]]) -> JoinItemId`
  - `JoinSet.spawn_cpu(fn: Callable[[], Result[T, E]]) -> JoinItemId`
  - `JoinSet.add(handle: TaskHandle[T, E]) -> JoinItemId`, consuming the handle
  - `JoinSet.join_all().await -> list[Result[T, WorkerError[E]]]`, consuming the set and returning results in submission order
  - `JoinSet.cancel_all().await -> list[CancelOutcome]`, consuming the set and returning cancellation evidence in submission order
  - `parallel.map(items: list[T], fn: Callable[[T], U]) -> list[U]` for owned homogeneous lists
  - `parallel.try_map(items: list[T], fn: Callable[[T], Result[U, E]]) -> Result[list[U], E]`
  - `PoolConfig { workers: PositiveInt }`
  - `Pool(config: PoolConfig)` backed by a private `rayon::ThreadPool`
  - `Pool.map(items: list[T], fn: Callable[[T], U]) -> list[U]`
  - `Pool.try_map(items: list[T], fn: Callable[[T], Result[U, E]]) -> Result[list[U], E]`
  - offload pool sizing and shutdown policy
- `JoinSet` is not a structured parent scope. It collects homogeneous work submitted through `sifr.runtime`/`sifr.parallel`, preserves typed cancellation/deadline evidence, and does not duplicate `TaskGroup`'s child-failure cancellation semantics.
- A live/non-empty `JoinSet` must be consumed by `join_all()` or `cancel_all().await` before scope exit. Dropping it without explicit observation is a compile-time diagnostic.
- `JoinItemId` is only an opaque user-side correlation token; no `JoinSet` API accepts it as input.
- `Pool` has no mutable global shutdown or reconfiguration API. Top-level calls use the private default pool. Configured `Pool` instances are scoped values; active `map`/`try_map` calls borrow the pool, and dropping an idle `Pool` releases its private Rayon pool.
- Separate async tasks, blocking I/O offload, CPU-heavy parallel work, and long-running supervised processes.
- Blocking and CPU offload are structured work. Module-level offload helpers must either require an active scope or return linear handles whose observation cannot be silently dropped.
- Enforce blocking-in-async diagnostics.
- Enforce CPU-heavy diagnostics and explicit offload.
- `parallel.map`, `parallel.try_map`, `Pool.map`, and `Pool.try_map` are synchronous CPU-heavy blocking calls; in async contexts they must be wrapped in `spawn_cpu`, and direct calls trigger the CPU-heavy diagnostic.
- Require sendable owned items and sendable closure captures for `parallel.map`/`try_map`; non-send items or captures are compile-time diagnostics.
- Use a lazily initialized private default `rayon::ThreadPool` built via `rayon::ThreadPoolBuilder` and `available_parallelism()`. Configured parallelism uses explicit `Pool(config)` objects backed by private Rayon pools. Do not configure Rayon's global pool.
- Map task, worker, foreign/runtime boundary, and panic-like runtime failures into typed evidence.
- Use CPython `concurrent.futures` as evidence for future/cancellation/deadline edge cases, not as the production API.
- Keep `sifr.concurrent.futures`, `Future.result(timeout=...)`, `Executor.map`, `as_completed`, and `ThreadPoolExecutor` as `deferred-to-adapter-phase`; no futures/executor adapter is implemented in this phase.

CPython tests to mine:

- `Lib/test/test_concurrent_futures/`

Rust/runtime candidates:

- `rayon`-like data parallelism
- `tokio::task::spawn_blocking`

Definition of done:

- Blocking calls inside async receive compiler diagnostics.
- Offloaded work returns typed results.
- Offload and `sifr.parallel` sendability/shareability diagnostics pass representative fixtures.
- Worker runtime failures become typed evidence.
- CPU-heavy parallel APIs are distinct from blocking I/O APIs.
- `sifr.parallel` input ownership, output ordering, pool sizing, and closure-capture rules are documented and tested.
- `JoinSet` drop diagnostics and explicit `join_all`/`cancel_all` observation paths are tested.
- Homogeneous result/cancellation/deadline behavior has CPython-derived adapted fixtures.
- Any CPython behavior requiring public `threading` objects is classified as `rejected`, `deferred-to-phase-X`, or `waived-with-rationale`.

### milestone_concurrency_runtime_4: Process And Subprocess Runtime

Scope:

- Add or close the production `sifr.process` surface:
  - `Command`
  - `Child`
  - `Output`
  - `Status`
  - `Stdio`
  - `PipeReader`
  - `PipeWriter`
  - sync `spawn`, `run`, `output`, and `wait`
  - async spawn/wait/communicate
  - stdin/stdout/stderr pipes as owned stream resources
  - timeout handling
  - `terminate`, `kill`, and structured cancellation
  - env/cwd
  - explicit `@shell_exec` effect classification for shell subprocess usage, in addition to `@blocking_io`
  - scoped process supervision entry point accepted by M0, such as `scope.spawn_process(Command(...))` or an equivalent API that binds child lifetime to a parent scope
- M0 must decide whether scoped process spawn returns `Child`, `TaskHandle[Status, SubprocessError]`, or a distinct `ProcessHandle`; the accepted shape must preserve owned stdin/stdout/stderr pipe access.
- Implement binary pipe mode and own the subprocess text-mode disposition.
- M4 owns subprocess text-mode integration by consuming text/i18n `milestone_text_i18n_1`; binary-only closure is not allowed.
- Keep existing `sifr.subprocess` and `sifr.asyncio.subprocess` frozen or marked compatibility-only; production behavior must not depend on them and they must not be extended in this phase.
- `subprocess.getoutput` and `subprocess.getstatusoutput` remain unsupported as legacy shell-invocation helpers.

CPython tests to mine:

- `Lib/test/test_subprocess.py`
- `Lib/test/test_asyncio/test_subprocess.py`

Rust/runtime candidates:

- `std::process`
- `tokio::process`

Definition of done:

- Sync and async subprocess loopback tests pass on the supported host matrix.
- Subprocess text-mode APIs are implemented with text/i18n evidence.
- Pipe ownership prevents double-close and use-after-close.
- Timeout/cancellation semantics are documented and tested.
- Shell usage is explicit and effect-classified.
- Shell subprocess APIs are registered with the `@shell_exec` security effect and tested for async/offload diagnostics.
- Sync shell subprocess APIs are also `@blocking_io` and are rejected in async contexts unless offloaded. Native async shell process APIs may be used in async contexts only through explicit `shell=True` or `Command.shell(...)` and still carry `@shell_exec`.
- Process/subprocess sendability/shareability diagnostics for callbacks, env/cwd data, pipe ownership, and cross-boundary captures pass representative fixtures.
- No adapter can bypass owned process/pipe lifecycle.

### milestone_concurrency_runtime_5: Shutdown, Signals, Cleanup, Context, And Diagnostics

Entry gate: M4 process lifecycle, pipe ownership, subprocess cancellation, and shell-effect contracts are complete, and the M0 supported-host/signal matrix has no unclassified entries.

Scope:

- Add or close structured signal/shutdown support:
  - `sifr.signal.Signal`
  - `ctrl_c`
  - `terminate`
  - `shutdown_stream`
  - supported signal constants/enum-like values
  - `strsignal` where host-supported
- Reject arbitrary `signal.signal(handler)` callback registration.
- Record `pause`, `getsignal`, `raise_signal`, and `pthread_sigmask` as `unsupported-with-diagnostic` or `host-limited` evidence entries; they are not production APIs.
- Record a signal-to-host matrix in the inventory before adopting signal constants or `strsignal`.
- Add or close deterministic cleanup support:
  - `sifr.resource.ExitStack`
  - `sifr.resource.AsyncExitStack`
  - `closing`
  - `aclosing`
  - `nullcontext`
- Reject Python convenience helpers `redirect_stdout`, `redirect_stderr`, `chdir`, `suppress`, `contextmanager`, and `asynccontextmanager` in this phase.
- Add a Sifr-native task/request context design required by tracing, deadlines, cancellation metadata, and future web observability:
  - `sifr.task.Context`
  - `sifr.task.ContextKey[T]`
  - explicit opt-in propagation across task groups
  - no implicit dynamic Python `contextvars` behavior
- Prefer compiler diagnostics, structured runtime diagnostics, logging/tracing events, and library deprecation metadata over Python `warnings` global filter parity.
- Runtime warning-style events, where needed, are structured diagnostics/tracing events only; no Python `warnings` adapter or global filter state is retained.

CPython tests to mine:

- `Lib/test/test_signal.py`
- `Lib/test/test_io/test_signals.py`
- `Lib/test/test_contextlib.py`
- `Lib/test/test_contextlib_async.py`
- `Lib/test/test_warnings/`

Definition of done:

- Production server/worker shutdown scenarios are covered.
- Cleanup is deterministic under cancellation.
- Task/request context is implemented with explicit propagation rules.
- Structured diagnostics do not become hidden exceptions.
- Warning/filter global-state parity is rejected or narrowed to an explicit structured runtime diagnostics design.
- Generator-decorator helpers are recorded as `unsupported-with-diagnostic` with a revisit rule; no partial fake generator path is allowed.

### milestone_concurrency_runtime_6: Typed IPC And Future Process Workers

Entry gate: typed IPC design approval must be recorded in the execution ledger before M6 selects a serialization crate or starts process-worker implementation.

Scope:

- Add typed IPC foundation before any process worker pool:
  - payload eligibility
  - explicit supported payload types
  - no arbitrary pickle/object transport
  - serialization format
  - versioning
  - child-process bootstrap
  - result/error framing
  - cancellation/termination messages
  - panic-free malformed-message handling
  - compile-time diagnostics for unsupported payloads where possible
- Define typed IPC as a communication substrate layered above M4 process pipes or another explicitly accepted transport. It does not replace same-process channels or raw process pipes.
- Define future `ipc.Connection[Req, Res, Err]` semantics at the protocol level, including request IDs, result/error frames, cancellation frames, stream close, backpressure, versioning, and malformed-frame errors. Do not ship a public process worker pool in this phase.
- Minimum frame families are bootstrap (`Hello`, `Ready`, `Reject`), work (`Run`, `Started`, `Completed`, `Failed`), control (`Cancel`, `Shutdown`, `Terminating`), health (`Heartbeat`, `WorkerStatus`), and protocol errors (`MalformedFrame`, `UnsupportedVersion`, `UnsupportedSchema`, `UnsupportedPayload`).
- Every typed IPC schema has stable schema identity/hash and an evolution policy: exact hash proceeds, compatible version ranges negotiate a version, and unknown/incompatible schema returns `Reject` or `UnsupportedSchema`.
- `IpcSerializable` is stricter than `Sendable`; file handles, pipes, lock guards, and channel endpoints are not process payloads unless explicitly accepted by a later IPC design.
- Keep `ProcessPoolExecutor`, `multiprocessing.Process`, `multiprocessing.Queue`, `multiprocessing.Pipe`, and `multiprocessing.Pool` `deferred-to-adapter-phase`; M6 ships typed IPC foundations only, not Python-shaped process pools.
- Reject fork/forkserver unless host-limited ownership evidence is recorded.
- Reject shared-memory APIs until explicit ownership/unlink/drop rules are proven.

CPython tests to mine:

- `Lib/test/test_concurrent_futures/`
- `Lib/test/_test_multiprocessing.py`
- `Lib/test/test_multiprocessing_main_handling.py`
- `Lib/test/test_multiprocessing_spawn/`
- `Lib/test/test_multiprocessing_fork/`
- `Lib/test/test_multiprocessing_forkserver/`

Rust/runtime candidates:

- typed local serialization with `serde` plus `postcard` after typed IPC design approval
- platform process primitives from M4

Typed IPC design approval means a named design artifact is reviewed by the phase owner and designated compiler/runtime reviewer, then recorded in the execution ledger before any serialization crate is selected.

Definition of done:

- IPC is safe, typed, versioned, and panic-free.
- IPC is explicitly layered over an accepted process/transport substrate and has deterministic close, backpressure, cancellation, and malformed-frame behavior.
- Unsupported payloads are compile-time diagnostics where possible.
- Sendability/shareability and IPC payload eligibility diagnostics pass representative fixtures.
- All M6 CPython process-pool/multiprocessing test families are classified with shared evidence states.
- Process pools remain `deferred-to-adapter-phase` by design in this phase.
- Any later accepted process-worker API exists for isolation/supervision/interop, not as Sifr's default CPU parallelism story.

### milestone_concurrency_runtime_7: Integration, Documentation, And Production Gate

Scope:

- Update public docs for every production API and major intentional divergence:
  - `sifr.task`
  - `sifr.sync`
  - `sifr.runtime`
  - `sifr.parallel`
  - `sifr.process`
  - `sifr.signal`
  - `sifr.resource`
  - `sifr.ipc`
- Update internal architecture docs for:
  - structured runtime work model
  - task/process/channel/offload/runtime boundaries
  - typed IPC and unsupported payload policy
  - blocking/offload policy
  - sendability/shareability
  - task/request context
  - diagnostics and signal global-state policy
  - rejected compatibility index
- Add demos:
  - structured task group
  - producer/consumer channel pipeline
  - blocking offload
  - CPU parallel map
  - async subprocess pipeline
  - structured shutdown
  - cleanup under cancellation
- Add generated Cargo dependency snapshots for all new feature combinations.
- Add panic-scan and emitted-code quality checks for task/channel/process/runtime paths.
- Update validation lane manifests with representative fixtures.
- Close the inventory:
  - every public surface has a terminal state
  - every CPython test family has a shared evidence state
  - every waiver has a revisit rule and regression fixture
  - every host-limited surface records the supported host matrix
- Run an external review loop on the final inventory and close any blocking finding before phase completion.
- External review owner is the runtime/stdlib phase owner plus the designated compiler/runtime reviewer recorded in the execution ledger. Phase completion requires a `PASS` result or the five-working-day fallback review procedure recorded with attempted review, open questions, conservative self-review, and no unresolved blocking questions.

Validation:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- file-size guardrail
- `cargo test -p sifr_stdlib`
- `cargo test -p sifr -- stdlib`
- `scripts/run_e2e_pass.sh`
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`

Definition of done:

- No public toy modules are required for phase completion.
- All compatibility surfaces are `compat-adapter`, `deferred-to-phase-X`, or `rejected` unless justified by M0 and backed by production APIs.
- All artifacts listed in `Required Tracking Artifacts` are complete with no unclassified entries.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in added runtime surfaces.
- Async and sync APIs follow the Phase 32 workload and cancellation model.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.json`
- `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md`
- `verification/stdlib/concurrency_runtime_workload_database.md`
- one traceability document per milestone domain under `verification/stdlib/`
- `verification/platform/platform_contract.md`
- `verification/platform/platform_contract.json`
- `verification/platform/supported_host_matrix.md`
- `verification/platform/golden/manifest.json`
- `scripts/run_platform_golden.sh`

Each milestone's traceability document must be created in that milestone's first implementation PR and closed before that milestone is marked complete; M7 audits all traceability documents for completeness.

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test files scanned
- public/native API tier decisions
- shared platform terminal states and stability levels
- shared platform golden fixture entries and skip/pass status for concurrency-owned contracts
- shared platform security/resource ownership rows for subprocess, IPC, cancellation, offload, shell execution, and queue bounds
- mined-as-substrate-fixture/adapted-for-sifr-api/compat-adapter-deferred/blocked-on-phase-X/external-signal/waived-with-rationale/rejected CPython evidence families
- compat-adapter/deferred-to-phase-X/rejected compatibility index
- final unsupported-with-diagnostic/host-limited waiver index

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No backward-compatibility shims, legacy aliases, deprecated behavior, fallback paths, or CPython-luggage APIs may survive phase exit.
- Deliberate adapters are allowed only when recorded in the inventory with Sifr-safe semantics, tests, and delegation to production APIs.
- No direct Tokio/runtime types may leak into public Sifr APIs.
- No arbitrary pickle-equivalent process-pool transport may be introduced.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added blocking sync function must be classified in the stdlib workload database.
- Every added async function must have a real suspension summary.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Every module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.

## Resolved Decisions

M0 records evidence for these decisions; it does not reopen them without a new issue.

| Decision area | Decision |
| --- | --- |
| Structured runtime work | Sifr exposes scoped runtime work rather than separate Python-shaped async/threading/multiprocessing worlds. Async tasks, blocking offload, CPU offload, long-running child processes, and future typed workers are owned by scopes, return affine handles, and must be observed, cancelled-and-joined, or aggregated. Raw threads and raw process pools are not the public model. |
| Sendability/shareability | A value is task/thread/process-sendable only when its full type graph is owned, has no non-send handles, has no borrowed capture that can outlive its lexical scope, and all contained types are marked sendable by Sifr. Shared references across tasks/threads require immutable data or explicit synchronization wrappers. Process-worker payloads additionally require typed IPC serialization. |
| Stable task APIs | Stable production APIs are `TaskHandle`, `TaskGroup[E]`, `spawn_scoped`, `sleep`, `timeout`, `deadline`, `cancel_scope`, `join_all`, `race`, and `select`. `TaskHandle[T, E]` is the public affine observation handle. `TaskGroup[E]` is error-homogeneous and result-heterogeneous through individual handles. `race`/`select` result containers are M0 public API boundary artifacts. Raw runtimes, event loops, callback transports, detached tasks, and Tokio handles are not public. |
| CPython-shaped adapters | No new CPython-shaped adapters are implemented in this phase. Existing `sifr.asyncio`, `sifr.subprocess`, and `sifr.asyncio.subprocess` may remain frozen compatibility-only surfaces; `sifr.queue`, `sifr.concurrent.futures`, and `sifr.multiprocessing` remain `deferred-to-adapter-phase` until a later migration issue proves value over the native APIs. |
| Signal APIs | Safe deterministic APIs are structured signal/shutdown streams for supported host signals (`SIGINT`, `SIGTERM`, and Unix `SIGHUP` where available). `signal.signal`, arbitrary Python handlers, `pthread_sigmask`, and `signal.pause` are `unsupported-with-diagnostic` or `host-limited` evidence entries, not production APIs. |
| Diagnostics and warnings | Runtime diagnostics use structured `tracing` spans/events plus `metrics` counters/histograms behind Sifr diagnostics types. No Python global `warnings` filter adapter ships in this phase. |
| Task/request context | M5 implements explicit `sifr.task.Context` and `sifr.task.ContextKey[T]` with explicit propagation through task spawn and request handoff. There is no implicit dynamic task-local mutation or `contextvars` parity. |
| Typed IPC serialization | M6 uses generated typed IPC schemas over `serde` plus `postcard` after sendability/shareability approval. `IpcSerializable` is stricter than `Sendable`, every schema has stable identity/hash plus compatibility policy, and arbitrary pickle-like object transport is permanently rejected. |
| Subprocess text mode | Because text/i18n runs first, M4 implements subprocess `text=True`, `encoding=...`, and `errors=...` by consuming `sifr.encoding` and explicit text I/O from text/i18n M1. Binary-only subprocess closure is not allowed. |
| Rust ecosystem | Use Tokio/Tokio Util, Futures Util, Crossbeam Channel, Tokio MPSC, std sync/Parking Lot/Once Cell/Scopeguard, Rayon, Tokio process/std process/Rustix, Tokio signal/Rustix, tracing/tracing-subscriber/metrics, thiserror, and Serde/Postcard as internal implementation tools. Do not use Flume, Signal Hook, Nix, or Bincode in this phase. Bespoke replacements for these crate families are not allowed; defer the affected surface instead. |
| `JoinSet` drop | `JoinSet[T, E]` is a linear scoped resource. It must be consumed by `join_all()` or `cancel_all().await` before scope exit. Dropping a live/non-empty `JoinSet` is a compile-time diagnostic; `cancel_all()` returns `CancelOutcome` evidence for every pending item, including already-started/already-completed/could-not-cancel cases. |
| Rayon pool architecture | Top-level `sifr.parallel.map`/`try_map` use a lazily initialized private default `rayon::ThreadPool` built with `rayon::ThreadPoolBuilder` and `available_parallelism()`. Configured parallelism uses explicit `sifr.parallel.Pool(config: PoolConfig)` objects backed by private Rayon pools. The global Rayon pool is never configured, and there is no mutable global pool configuration API. |
| `JoinSet` submission API | `JoinSet[T, E]` accepts homogeneous work through `spawn_blocking(fn) -> JoinItemId`, `spawn_cpu(fn) -> JoinItemId`, and `add(handle: TaskHandle[T, E]) -> JoinItemId`. `add` consumes the handle so results have one observer. `join_all().await` and `cancel_all().await -> list[CancelOutcome]` consume the set. |
| `JoinSet` result ordering and `JoinItemId` role | `join_all().await` returns results in submission order. `cancel_all().await` returns cancellation evidence in submission order. `JoinItemId` is an opaque user-side correlation token with no further `JoinSet` API; callers use it to index their own submission records, not to query the set. |
| `Pool` instance API | Configured `Pool` objects expose `pool.map(items, fn)` and `pool.try_map(items, fn)` with the same result ordering and sendability rules as top-level `parallel.map`/`try_map`. `Pool` has no global shutdown call; active calls borrow the pool, and dropping an idle pool releases the private Rayon pool. |
| Task context API shape | M1 reserves `ctx: Option[sifr.task.Context] = None` parameters on `spawn_scoped` and `TaskGroup` constructors. M1 type-checks and stores the parameter; M5 implements explicit propagation semantics without changing M1 API shape. |
| Existing `sifr.asyncio` veneer | Existing `sifr.asyncio` veneer code is frozen as `compat-adapter`. M1 does not build on it, does not extend it, and does not remove already-supported veneer entry points. New task/process/queue APIs are implemented through `sifr.task`, `sifr.sync`, and `sifr.process`; bare `asyncio` imports still receive namespace-contract diagnostics. |
| `race` and `select` loser evidence | `race` returns a container with winner index, typed outcome, and loser cancellation evidence. `select` returns a container with winner branch tag, typed outcome, and loser cancellation evidence. Concrete Sifr return-type signatures are recorded in the M0 public API boundary artifact before M1 starts. |
| Shell subprocess effect | Shell subprocess usage is a distinct `@shell_exec` security effect. Sync shell APIs are also `@blocking_io` and are rejected in async contexts unless offloaded. Native async shell process APIs may run in async contexts only through explicit `shell=True` or `Command.shell(...)` and still carry `@shell_exec`. |
