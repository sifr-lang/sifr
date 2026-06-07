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

CPython-shaped APIs are not implemented in this phase and are not retained as compatibility targets. `sifr.process`, `sifr.task`, `sifr.sync`, `sifr.runtime`, and `sifr.parallel` are the production surfaces; legacy Python-shaped modules are evidence only and must resolve to removal, rejection, or unsupported diagnostics.

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
| `compat-adapter` | Shared platform state intentionally unused by this phase | none; CPython-shaped concurrency/process adapters are not accepted |
| `deferred-to-phase-X` | Potential future production API requiring a separate named design gate | process workers, task context extensions |
| `rejected` | Too dynamic, global, Python-specific, legacy, or unsafe for Sifr | `threading` parity, raw event-loop policies, callback transports, pickle transport, `signal.signal` handlers |

No phase milestone may be marked complete while any surface remains unclassified.

## Cross-Phase Dependency Contract

The split phase order is explicit:

1. Text/Unicode/encoding/i18n runtime.
2. Concurrency/process/runtime substrate.
3. Network/HTTP platform substrate.

This phase starts after the text/i18n provider phase has established the shared encoding, Unicode, explicit text I/O, and locale/i18n gates. It runs before network/HTTP so network servers and clients consume the production task, cancellation, shutdown, offload, diagnostics, and process model rather than adding local runtime substitutes.

- Text/i18n `milestone_text_i18n_1` is the hard prerequisite for subprocess text mode, warning output encodings, locale-sensitive warning formatting, and demos that rely on encoded text I/O.
- Network/web owns any rejected-or-diagnostic CPython-shaped network entry decisions; this phase owns the native task/sync/process substrate those decisions consume.
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

- Existing `sifr.subprocess` sync helpers and `CompletedProcess` are legacy implementation debt. M0 must record the removal or unsupported-diagnostic plan; production code must use `sifr.process`.
- Existing `sifr.asyncio` veneer code is legacy implementation debt, not a production namespace. M0 must record which entry points are removed, rejected, or retained only as internal tests while production code uses `sifr.task`.
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

No CPython-shaped adapter is a product goal for this phase. Future production APIs must be Sifr-native and pass the no-toy gate; migration convenience alone is not sufficient to add or retain public modules.

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
    child = group.spawn_process(process.Command("worker"))  # supervised; pipe access shape settled in M0
```

The exact `spawn_blocking`, `spawn_cpu`, and `spawn_process` method signatures are settled in M0/M3/M4, but the ownership rule is not optional: offload and process work must participate in scoped observation and cancellation. Module-level helpers may exist only when they still require an active structured scope or return a linear handle whose observation is compiler-enforced.

`TaskGroup[E]` is the canonical owner for mixed runtime work under the fail-fast structured-concurrency policy. A distinct `task.Scope` or `runtime.Scope` type is introduced only if M0 identifies a concrete use case `TaskGroup[E]` cannot satisfy; M0 must record that finding before M1 starts. Individual handles may have different success types. Homogeneous result collections such as `join_all`, `race`, `select`, and `JoinSet[T, E]` require one result/error shape unless the user constructs an explicit sum/enum result type.

Scoped offload inserted into `TaskGroup[E]` must map user errors plus runtime/offload failures into the group's error type or an accepted wrapper such as `WorkerError[E]`; M0 records the exact error shape. Scoped process spawning must preserve owned pipe access while binding child lifetime to the parent scope; M0 decides whether it returns `Child`, `TaskHandle[Status, SubprocessError]`, or a distinct `ProcessHandle`.

`TaskHandle[T, E]` is the public affine observation handle name. `Task` may remain an internal type name only; exposing both names as public aliases is rejected unless a new Sifr-native API design proves separate semantics.

`sifr.task` is the canonical public namespace for scoped runtime work. Existing CPython-shaped surfaces such as `sifr.asyncio`, `sifr.threading`, `sifr.concurrent.futures`, `sifr.subprocess`, and `sifr.multiprocessing` are evidence or implementation debt only. They are not adapters, not fallback paths, and not the runtime spine.

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
4. It is a Sifr-native production API, not a compatibility wrapper.

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
2. CPython-shaped `sifr.*` modules such as `sifr.asyncio`, `sifr.subprocess`, and `sifr.concurrent.futures` are not accepted compatibility modules in this phase.
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

## Rust Ecosystem Decisions

This phase follows [Dependency Policy](../internal_docs/dependency_policy.md). It builds a Sifr concurrency/runtime platform, not a new async runtime, channel library, process supervisor, tracing stack, cleanup stack, error framework, IPC serializer, or CPU scheduler. The crate choices below are locked inputs to implementation. M1-M6 implementation PRs must not perform crate-family discovery, swap in adjacent crates, or add broad feature flags; changing an accepted/rejected dependency decision requires a new issue or explicit phase amendment before implementation starts.

Discovery completed for this decision table:

- workspace `Cargo.toml` direct dependencies and feature flags
- `Cargo.lock` transitive crate versions already present through Ruff/test tooling
- `cargo metadata --format-version=1 --no-deps`
- crates.io `cargo search`/`cargo info` for crates not directly pinned today

Public Sifr APIs must never expose Tokio, Futures, Rayon, Crossbeam, Rustix, tracing, metrics, serde, postcard, thiserror, or platform-helper types directly. Every accepted crate is internal substrate hidden behind Sifr ownership, typed errors, cancellation evidence, sendability/shareability checks, and no-panic generated-runtime guarantees. Generated user projects include these crates only when the corresponding Sifr runtime/stdlib feature is used.

Dependency rings for this phase:

- Ring 2 generated-runtime core: `tokio`, `tokio-util`, conditional `futures-util`, and `tracing`, each feature-gated by the Sifr runtime capability that requires it.
- Ring 3 stdlib feature-gated substrate: `crossbeam-channel`, `rayon`, targeted conditional `rustix`, `metrics`, and `thiserror`.
- Ring 4 feature-specific protocol/data substrate: `serde` plus `postcard`, gated to M6 typed IPC and any later explicitly accepted serialization feature.
- Ring 5 dev/test/demo only: `tracing-subscriber` and `serde_json` where local tests, demos, inventories, or golden artifacts require them.
- Ring 6 rejected direct dependencies: listed below.

### Locked Dependencies By Ring

| Ring | Capability | Crate decision | Version and feature plan | Milestone | Binding notes |
| --- | --- | --- | --- | --- | --- |
| Ring 2 | async runtime, timers, scoped task lowering, blocking pool, async process I/O, signals, async sync primitives | `tokio` | keep workspace `tokio = 1.52.3`; expand features only to `macros`, `rt`, `time`, `sync`, `process`, `io-util`, and `signal`; do not enable `full`, `net`, `rt-multi-thread`, or Tokio `parking_lot` in this phase | M1, M2, M4, M5 | Tokio owns async execution and async OS integration internally. The runtime entrypoint remains `current_thread`; `TaskGroup[E]` async work is cooperatively concurrent, not implicit CPU parallelism. Blocking I/O parallelism uses Tokio's blocking pool, and CPU parallelism uses Rayon. Sifr keeps the public model as `TaskHandle`, `TaskGroup[E]`, `sifr.process`, `sifr.signal`, and `sifr.sync`; no Tokio handles, runtimes, tasks, or channels leak. |
| Ring 2 | cooperative cancellation helper and Tokio I/O utilities | `tokio-util` | add `tokio-util = 0.7.18` with `default-features = false`, features `rt`, `io-util`, and `time`; in tokio-util 0.7.18 `rt` exposes `tokio_util::sync::CancellationToken`, and there is no separate `sync` feature; do not enable `full`, `net`, `codec`, `compat`, or `join-map` | M1, M4 | Used behind Sifr-owned `CancelScope`/cancellation internals and process pipe helpers if needed. `tokio_util::sync::CancellationToken` is never public. |
| Ring 2 | future combinators and stream utilities | `futures-util` | add `futures-util = 0.3.32` with `default-features = false`, features `std` and `async-await` only if M1 proves `join_all`, `race`, `select`, or stream adapters would otherwise require substantial custom `Future`/`poll` code; do not enable `channel`, `compat`, `io`, or `sink` features | M1 | Conditional generated-runtime helper only. Sifr result containers remain compiler-owned API. If M1 can implement the accepted combinators cleanly with Tokio and generated helpers, `futures-util` is not added. |
| Ring 2 | async channels and async coordination | `tokio::sync` | provided by accepted Tokio `sync` feature; no separate crate | M2 | Backs Sifr-owned `AsyncChannel`, `AsyncMutex`/`AsyncLock`, `Semaphore`, `Notify`/`Event` as accepted. Channel/lock guard semantics are defined by Sifr diagnostics, not Tokio docs. |
| Ring 3 | sync cross-thread channels and queues | `crossbeam-channel` | add `crossbeam-channel = 0.5.15` with default `std` feature only if M2 keeps sync cross-thread channels production-public | M2, M3 | Backs sync bounded/unbounded producer-consumer queues for cross-thread blocking/CPU offload handoff. Public API is Sifr-owned channel/queue types. If M2 narrows to async channels only, Crossbeam becomes `deferred-to-phase-X` without a substitute sync channel stack. |
| Ring 3 | sync locks and once/lazy state | Rust standard library | use `std::sync::{Mutex, RwLock, OnceLock, Condvar}` and `std::thread::available_parallelism`; do not add a new crate for these in this phase | M2, M3 | Sifr lock poisoning, guard movement, and await-crossing diagnostics are language/runtime rules, not direct std API exposure. |
| Ring 3 | CPU parallelism and work stealing | `rayon` | add `rayon = 1.12.0`; do not enable `web_spin_lock`; MSRV is compatible with workspace Rust 1.93 | M3 | Backs `sifr.parallel` and CPU-heavy offload. Use private `rayon::ThreadPool`s only; never configure Rayon's global pool. Avoid unobserved Rayon `spawn` patterns; user CPU closures are wrapped so panics/failures become typed `WorkerRuntimeError`/`WorkerError` evidence rather than user-triggerable process panics. |
| Ring 2 | process execution and async pipes | `tokio::process` and `std::process` | provided by accepted Tokio `process`/`io-util` features plus std | M4 | `tokio::process` backs async child supervision and pipes; `std::process` backs explicitly blocking sync process calls routed through `@blocking_io`/offload rules. |
| Ring 3 | host-limited process/signal/fd details not covered by std/Tokio | `rustix` | add `rustix = 1.1.4` only in the crate that needs host APIs and only after M4/M5 records that `std`/Tokio cannot provide the required behavior, with `default-features = false`, features `std`, `process`, `pipe`, `fs`, and `stdio`; do not enable `all-apis`, `net`, `io_uring`, `pty`, `shm`, or broad Linux-version features | M4, M5 | Used only for documented host-limited behavior such as process group/session/fd inheritance details. Every use must have a supported-host matrix row and a deterministic host-specific fixture. |
| Ring 2 | structured diagnostics and spans | `tracing` | add `tracing = 0.1.44` as a direct workspace dependency with `default-features = false`, feature `std`; do not enable the `attributes` feature, so `#[instrument]` and tracing attribute macros are unavailable in this phase | M5 | Emits structured runtime events/spans behind Sifr diagnostics types. No Python `warnings` global filter or implicit contextvars behavior. |
| Ring 3 | runtime metrics facade | `metrics` | add `metrics = 0.24.6` with default features only after M5 records concrete metric names, label/cardinality policy, emission points, redaction policy, and deterministic tests | M5 | Emits counters/histograms for accepted runtime events. Default features are accepted because they expose only the metrics facade API; no exporter, recorder, or integration features are enabled or implied. |
| Ring 3 | Rust internal error enum derivation | `thiserror` | add `thiserror = 2.0.18` with default `std` feature only in first-party Rust crates defining internal runtime/compiler errors, and only if error boilerplate becomes material | M1-M6 | Implementation aid only. Sifr language errors remain typed `Result`/sum variants; `thiserror` types are not public Sifr API and are not emitted into user-authored source. |
| Ring 4 | IPC schema serialization | `serde` plus `postcard` | keep workspace `serde = 1.0.228` with `derive`; add `postcard = 1.1.3` with `default-features = false`, feature `use-std`; do not use postcard derive macros | M6 | M6 typed IPC only. Payload eligibility, schema identity, version negotiation, compatibility policy, cancellation frames, and malformed-frame diagnostics are generated from Sifr IPC schemas; no arbitrary object transport and no general serialization baseline. |

### Rejected Or Non-Production Dependencies

| Crate/family | Decision | Rationale |
| --- | --- | --- |
| `flume`, `async-channel`, `futures-channel` | rejected | Sifr uses Tokio MPSC for async channels and Crossbeam Channel for sync cross-thread channels. Additional channel stacks would fragment close/backpressure/cancellation semantics. |
| `parking_lot` | rejected for direct production use in this phase | The phase uses `std::sync` and `tokio::sync` so lock poisoning, guard movement, and await diagnostics remain Sifr-owned. Existing transitive `parking_lot` from Ruff/Salsa is unrelated and not a Sifr runtime dependency. |
| `once_cell` | rejected for new runtime work in this phase | New runtime code uses `std::sync::OnceLock`. Existing workspace `once_cell` uses are unrelated legacy/tooling dependencies and do not authorize new phase use. |
| `scopeguard` | rejected | Cleanup behavior must be explicit Sifr scope/Drop code with typed cleanup evidence. `scopeguard`'s panic/unwind framing is not the language contract. |
| `tracing-subscriber` | dev/test/demo only, not production substrate | Libraries emit `tracing` events; applications or tests may install subscribers. The runtime must not choose a global subscriber or logging policy. |
| `serde_json` | rejected for IPC payload frames | JSON may remain for diagnostics/tests where already used, but typed IPC payloads use postcard binary frames with schema identity. |
| `bincode` | rejected for this phase | Postcard is the selected compact binary Serde codec for typed IPC, and multiple production IPC codecs would complicate schema/version compatibility. |
| pickle-like serializers and arbitrary object transport | permanently rejected | No arbitrary object transport, no Python multiprocessing-style pickle fallback, and no schema-less process-pool payloads. |
| `signal-hook`, `nix` | rejected | Signals and host details use Tokio signal APIs plus targeted Rustix use. No unsafe user signal handlers or broad Unix wrapper dependency. |
| `mio`, `bytes`, `dashmap` | rejected as direct phase dependencies | These may appear transitively through Tokio/Ruff, but Sifr does not code directly against them in this phase. Tokio owns readiness/buffer internals. |
| `anyhow`, `eyre` | rejected for runtime/language-facing errors | Runtime/generated-project errors use typed enums and Sifr diagnostics, not dynamic error bags. Ring 1 compiler/tooling-only use may remain if it does not replace structured diagnostics or leak into generated user projects. Existing unrelated workspace use does not authorize runtime phase use. |

From-scratch async runtimes, future combinators, channel implementations, lock/once utilities, work-stealing schedulers, process supervisors, signal handling frameworks, tracing systems, error frameworks, IPC serializers, or thread pools are rejected in this phase. If the accepted crate stack cannot satisfy a required surface under these constraints, the affected surface becomes `deferred-to-phase-X` with evidence instead of receiving a bespoke implementation.

## Sendability And Shareability Contract

Before scoped task spawning, blocking offload, CPU offload, thread pools, process workers, or cross-boundary closure captures ship, Sifr must define a user-visible sendability/shareability model:

- `Send[T]`-equivalent eligibility for moving values across task/thread/process boundaries
- `Sync[T]`/`Share[T]`-equivalent eligibility for shared references
- lifetime-like capture constraints for spawned work that may outlive the lexical caller
- non-send capture diagnostics
- shared mutable state diagnostics
- safe closure capture rules for task, blocking, CPU, and process-worker APIs
- typed compile-time diagnostics where unsupported values cross worker boundaries

This is a phase-wide gate, not an executor-only caveat. M0 defines the model, M1 owns the initial HIR/type-checker and codegen enforcement for task spawning and task handles, M2 extends enforcement to channel and synchronization value types, M3 extends verification to blocking/CPU offload and `sifr.parallel`, M4 extends verification to process/subprocess callbacks and captures, and M6 extends verification to typed IPC payloads. CPython-shaped executor and process-pool initializer APIs are `deferred-to-phase-X`; unsafe cross-boundary process-worker payload capture is rejected until typed IPC proves an accepted production process-worker API.

## Cancellation And Failure Contract

Cancellation and failure semantics apply to every structured work kind, not only async tasks:

- cancellation is idempotent and produces typed evidence
- timeout preserves the wrapped operation's normal typed outcome and adds typed timeout evidence as a distinct variant
- `TaskGroup` exit reports unhandled child failures plus cancellation/cleanup evidence; a child result explicitly awaited and statically handled under the M0 proof is observed and does not by itself fail group exit
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

Names may align with CPython evidence only when they are the best Sifr-native names. The operational contract is Sifr `Result`/`Option` and typed sums, not exception-driven control flow. Legacy convenience APIs such as `check_output` are not adapter targets; accepted Sifr-native process helpers return typed failure evidence.

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
2. `milestone_concurrency_runtime_0a` removes, hides, or diagnoses legacy CPython-shaped surfaces according to the M0 disposition. No production implementation milestone may start while these surfaces remain publicly usable.
3. `milestone_concurrency_runtime_1` defines the structured async task substrate and implements the first sendability/shareability compiler enforcement pass for task spawning. It blocks async synchronization, async subprocess, and offload APIs that need task cancellation/deadline semantics.
4. `milestone_concurrency_runtime_2` closes synchronization/backpressure on top of the task model.
5. `milestone_concurrency_runtime_3` implements blocking and CPU offload after sendability/shareability is accepted.
6. `milestone_concurrency_runtime_4` implements native process/subprocess after task, sync, and offload semantics are stable.
7. `milestone_concurrency_runtime_5` implements shutdown, signal streams, cleanup, task context, and structured diagnostics after the task/process contracts are available.
8. `milestone_concurrency_runtime_6` designs and implements typed IPC. Process-worker pools remain `deferred-to-phase-X` by design in this phase.
9. `milestone_concurrency_runtime_7` closes docs, demos, validation, waivers, and rejected-surface decisions last.

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
- Record the structured runtime work model, with `TaskGroup[E]` as the canonical owner for mixed runtime work unless M0 records a concrete use case requiring a distinct `task.Scope` or `runtime.Scope`.
- Test the `TaskGroup[E]`-only mixed-owner model against server shutdown, child process supervision with stdout/stderr pump tasks, blocking offload plus async task workloads, and CPU offload plus cancellation. If any case needs non-fail-fast ownership, introduce `task.Scope` or `runtime.Scope` in M0.
- Record whether scoped process spawn returns `Child`, `TaskHandle[Status, SubprocessError]`, or a distinct `ProcessHandle`; the accepted API must preserve owned pipe access while binding child lifetime to the parent scope.
- Audit current generated-code/internal usages of `Task` and `BlockingTask`; record whether those names remain internal only or whether either becomes a public alias to `TaskHandle`.
- Record sendability/shareability rules for task/thread/process captures.
- Record the HIR/type-checker/codegen ownership plan for sendability/shareability enforcement and the exact diagnostics/fixtures required in M1, M2, M3, M4, and M6.
- Use error-homogeneous task groups and homogeneous result collections: `TaskGroup[E]` may own child handles with different success types but one error type; `join_all`, `race`, `select`, and `JoinSet[T, E]` require one result/error shape unless users construct an explicit sum/enum type.
- Use structured tasks only: stable public tasks are structured by default, handle drop before failure observation is diagnosed, and detached tasks are rejected in this phase.
- Record `TaskGroup` versus `JoinSet`: `TaskGroup` is scoped structured concurrency with automatic cancellation on unhandled child failure; `JoinSet` is a dynamically-growable offload/task-result collection for collecting completed homogeneous work items without pretending to be a structured parent scope.
- Record `TaskGroup` observed/unobserved failure semantics, including whether an observed failure still triggers fail-fast sibling cancellation or only unhandled failures do.
- Define static handled-failure proof for `TaskHandle` observation, distinguishing awaited-and-ignored, awaited-and-assigned-but-uninspected, exhaustively matched, propagated with `?`, converted into another error, and explicit intentional discard if accepted.
- Record `race` and `select` result containers as part of the M0 public API boundary artifact, not an implementation detail.
- Record `select` call API syntax: whether it accepts named kwargs, requires a special compiler keyword/macro form, or uses another static-branch mechanism; record the branch-tag type and how the compiler enforces static branch identity at compile time.
- Record the exact error-type binding for `TaskGroup[E].spawn_blocking` and `TaskGroup[E].spawn_cpu`: whether the callable returns `Result[T, E]` with runtime offload errors representable in `E`, whether the group is typed as `TaskGroup[WorkerError[E]]`, or whether an explicit mapper closure is required. M0 must also confirm alignment or record an explicit rationale difference against `JoinSet.join_all().await -> list[Result[T, WorkerError[E]]]` in M3.
- Record blocking/cpu-heavy effect rules.
- Record cancellation/deadline/timeout semantics.
- Record typed worker/task/process/signal/diagnostic error models.
- Use fixed default `sifr.parallel` pool sizing equal to `available_parallelism()` with optional `sifr.parallel.PoolConfig { workers: PositiveInt }`; no implicit global pool mutation API is exposed.
- Verify the locked Rust Ecosystem Decisions table, including exact crates, versions, features, rejected crates, no-public-Rust-type boundaries, and the rule that implementation PRs do no crate-family discovery.
- Add or update workspace dependency entries only from the accepted production dependency table, with the exact feature plans recorded there.
- Record the runtime threading model from the Rust Ecosystem Decisions table: Tokio remains `current_thread` for async task cooperation, `spawn_blocking` handles blocking I/O parallelism, and Rayon handles CPU parallelism; M1 does not introduce Tokio `rt-multi-thread`.
- Record concurrency/runtime rows in the shared supported host matrix at `verification/platform/supported_host_matrix.md`, and require all host-limited subprocess/signal entries to reference that matrix. This supersedes any per-phase host matrix path.
- Add or update concurrency-owned entries in `verification/platform/golden/manifest.json` and ensure `scripts/run_platform_golden.sh` knows which entries are blocked by unfinished milestones.
- Map subprocess, IPC, cancellation, offload, queue bounds, signal, and shell-exec security/resource concerns to the shared platform contract.
- Record the stdlib workload database artifact for newly added runtime APIs, including owner, schema, blocking/CPU/suspension classification, and validation command.
- Implement explicit typed task/request context in M5 as `sifr.task.Context` and `sifr.task.ContextKey[T]` with explicit propagation only; no Python-style implicit `contextvars` behavior.
- Record the designated compiler/runtime reviewer and the typed IPC design approval process in the execution ledger.
- Record existing `sifr.asyncio` veneer entry points as legacy implementation debt: M1 does not build on them or extend them, and M0 records removal, internal-test-only, or unsupported-diagnostic disposition.
- Classify M5 convenience helpers (`redirect_stdout`, `redirect_stderr`, `chdir`, `suppress`, `contextmanager`, `asynccontextmanager`) during the M0 inventory.
- Copy the resolved decision register into the execution ledger before M0 closes.
- Add import-resolution tests for canonical `sifr.*` module names and negative diagnostics for bare CPython stdlib import attempts.
- Assign every deprecated, historical, or legacy-only CPython entry the terminal state `unsupported-with-diagnostic` or `rejected`.

Definition of done:

- The backlog is derived from CPython source/tests plus Sifr runtime needs, not hand-written memory.
- Every proposed API is classified by support tier and terminal state.
- M0a-M7 implementation PRs have concrete backlog entries rather than prose-only scope; each backlog entry has at least one named fixture, acceptance criterion, or design artifact.
- No CPython module shape is still treated as automatic production scope.
- The resolved decision register is copied into the execution ledger, including task typing, detached tasks, task context, host matrix, workload database, rejected CPython-shaped surface disposition, and Rust ecosystem choices.
- The shared platform contract artifacts are present or updated: `verification/platform/platform_contract.md`, `verification/platform/platform_contract.json`, `verification/platform/supported_host_matrix.md`, `verification/platform/golden/manifest.json`, and `scripts/run_platform_golden.sh`.
- The Rust Ecosystem Decisions table remains the checked-in dependency decision record. M0 closes only after the execution ledger confirms that every implementation backlog item uses that table, that no accepted crate exposes public Sifr types, and that rejected crates have diagnostics or no-use checks where applicable.
- The execution ledger records the current-thread Tokio runtime invariant and the explicit offload/parallelism split before M1 starts.
- Pool-sizing policy for `sifr.parallel` is recorded in the execution ledger before M0 closes; M3 must not start until this entry exists.
- Reviewer identity is recorded in the execution ledger.
- Post-M0 external review is complete, has a `PASS` result, and is recorded in the planning reviews section of the execution ledger before M1 starts, or the five-working-day fallback review procedure is recorded with attempted review, open questions, conservative self-review, and no unresolved blocking questions.
- Import-resolution tests for canonical `sifr.*` names pass, and negative-diagnostic tests for bare CPython stdlib import forms pass.
- Sendability/shareability diagnostics have named representative fixtures assigned to the enforcing milestone.

### milestone_concurrency_runtime_0a: Legacy CPython-Shaped Surface Removal Gate

Entry gate: M0 has assigned every CPython-shaped runtime/concurrency/process surface a terminal state and recorded removal, internal-test-only, or unsupported-diagnostic disposition for existing implementations.

Scope:

- Remove public importability for legacy CPython-shaped runtime/process modules that M0 marks `rejected`.
- Move any M0-approved evidence-only helpers behind internal test namespaces; they must not be reachable as public `sifr.*` modules.
- Add unsupported-diagnostic fixtures for legacy public names that remain visible only to explain the Sifr-native replacement.
- Add negative import/use fixtures for `sifr.subprocess`, `sifr.asyncio` new APIs, `sifr.queue`, `sifr.concurrent.futures`, and `sifr.multiprocessing`.
- Update [async_concurrency_model.md](../internal_docs/async_concurrency_model.md) if it still describes `sifr.asyncio` as a supported veneer.
- Prove production APIs do not depend on legacy surfaces before M1 starts.

Definition of done:

- `sifr.process` is the only public process API.
- No CPython-shaped concurrency/process module remains publicly usable as an adapter, fallback, or alias.
- All retained evidence-only code is `internal-only` and used only by tests or inventory tooling.
- Unsupported diagnostics point to Sifr-native APIs such as `sifr.task`, `sifr.sync`, `sifr.runtime`, `sifr.parallel`, and `sifr.process`.
- M1 cannot start until this milestone is complete.

### milestone_concurrency_runtime_1: Structured Async Runtime

Entry gate: the post-M0 external review recorded in the execution ledger must have a `PASS` result, or the M0 fallback review procedure must be recorded after five working days with conservative self-review and no unresolved blocking questions. `milestone_concurrency_runtime_0a` must also be complete.

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
- A child failure explicitly awaited and statically handled under the M0 proof is observed and does not by itself fail `TaskGroup` exit. M0 records whether observed failures still trigger fail-fast sibling cancellation or only unhandled failures do.
- M1 consolidates the existing `TaskScope`/`TaskGroup` lowering contract instead of adding a parallel task model: top-level detached spawn remains rejected, direct coroutine-call requirements are recorded or deliberately relaxed with tests, handle observation remains affine, and borrowed/non-send captures remain diagnostics.
- `join_all`, `race`, and `select` require result-type unification or an explicit user sum/enum type; heterogeneous collections without an explicit sum are diagnostics.
- `race` accepts a homogeneous collection of awaitables and returns the first completed typed outcome plus its collection index; it cancels every still-pending loser and returns typed cancellation evidence for losers.
- `select` is the named-branch form for a statically known set of awaitable branches. It returns the winning branch tag plus the branch's typed outcome, requires branch result/error unification or an explicit user sum/enum type, cancels every still-pending loser, and returns typed cancellation evidence for losers.
- M0 records concrete result containers for `race` and `select`, including winner identity, typed outcome, and loser cancellation evidence.
- `timeout` wraps one awaited operation and returns the operation's normal typed outcome plus typed timeout evidence. Inner errors are preserved; timeout is a distinct error/evidence variant. `deadline` uses an absolute monotonic deadline. `cancel_scope` groups multiple child operations under a Sifr-owned cancellation scope; it is a settled stable API per the Resolved Decisions table. M0 records the concrete public type name and Sifr ownership boundary.
- `spawn_scoped` is a module-level `sifr.task` entry point distinct from `TaskGroup.spawn`; M0 records the calling convention and how it proves an active structured owner.
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
- Keep existing `sifr.asyncio` out of the production dependency graph; no new `sifr.asyncio` API is implemented in this phase, and M0 records removal, internal-test-only, or unsupported-diagnostic disposition for existing veneer entry points.
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
- M0's recorded sibling-cancellation policy for observed failures is implemented and has a named representative fixture.
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
  - default rule: sync lock guards cannot cross any `await`; async lock guards may cross `await` only if the API explicitly marks the guard await-safe. M0 records whether each accepted async guard is await-safe, await-forbidden, or lint-only.
  - semaphore permits are guard-like resources; M0 records whether permits may cross `await`, are await-forbidden, or are lint-only.
  - `Semaphore`, `Event`, `Notify`, accepted `Barrier`, and accepted `Once` must not smuggle non-send captured state through waiters or callbacks.
- Mark sync operations that can block on I/O, process, channel, lock, or external runtime state as `@blocking_io` or a narrower workload/effect class when used from async contexts.
- Mark `sifr.queue`, `sifr.asyncio.Queue`, `PriorityQueue`, `LifoQueue`, `SimpleQueue`, and `task_done`/`join` accounting as `rejected` or `unsupported-with-diagnostic` unless a future Sifr-native queue design proves production value over `sifr.sync` channels.

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
- CPython queue/async-queue test families are classified with shared evidence states, including `rejected` or `unsupported-with-diagnostic` where they only exercise Python-shaped queue APIs.

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
  - `parallel.map(items: list[T], fn: Callable[[T], U]) -> Result[list[U], WorkerRuntimeError]` for owned homogeneous lists
  - `parallel.try_map(items: list[T], fn: Callable[[T], Result[U, E]]) -> Result[list[U], WorkerError]`
  - `PoolConfig { workers: PositiveInt }`
  - `Pool(config: PoolConfig)` backed by a private `rayon::ThreadPool`
  - `Pool.map(items: list[T], fn: Callable[[T], U]) -> Result[list[U], WorkerRuntimeError]`
  - `Pool.try_map(items: list[T], fn: Callable[[T], Result[U, E]]) -> Result[list[U], WorkerError]`
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
- Map task, worker, foreign/runtime boundary, and panic-like runtime failures into typed evidence. In M3's first implementation wave, `WorkerError` is the public non-generic worker wrapper used by `parallel.try_map` and `Pool.try_map`; `JoinSet`/scoped offload keep the stricter `WorkerError[E]` target if the type system can carry the user error parameter without erasing it.
- Use CPython `concurrent.futures` as evidence for future/cancellation/deadline edge cases, not as the production API.
- Mark `sifr.concurrent.futures`, `Future.result(timeout=...)`, `Executor.map`, `as_completed`, and `ThreadPoolExecutor` as `rejected` or `unsupported-with-diagnostic` unless a future Sifr-native API proves production value over `sifr.runtime`, `sifr.parallel`, and `JoinSet`.

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

### milestone_concurrency_runtime_4: Process Runtime

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
- M4 distinguishes expected child exit from unexpected child exit; normal exit, nonzero exit, signal termination, timeout, and parent cancellation must map to success, typed error, or cancellation evidence.
- Implement binary pipe mode and own the subprocess text-mode disposition.
- M4 owns subprocess text-mode integration by consuming text/i18n `milestone_text_i18n_1`; binary-only closure is not allowed.
- Mark existing `sifr.subprocess` and `sifr.asyncio.subprocess` as legacy implementation debt to remove or route to unsupported diagnostics. Production behavior must not depend on them, they must not be extended, and `sifr.process` is the only accepted public process surface in this phase.
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
- No rejected Python-shaped surface can bypass owned process/pipe lifecycle.

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
- IPC compatibility is generated from Sifr IPC schema definitions, not inferred dynamically from arbitrary runtime values.
- `IpcSerializable` is stricter than `Sendable`; file handles, pipes, lock guards, and channel endpoints are not process payloads unless explicitly accepted by a later IPC design.
- Classify `ProcessPoolExecutor`, `multiprocessing.Process`, `multiprocessing.Queue`, `multiprocessing.Pipe`, and `multiprocessing.Pool` as `rejected` or `unsupported-with-diagnostic` for public CPython-shaped APIs. Defer only a future Sifr-native process-worker API built on `sifr.process` plus `sifr.ipc`.
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
- Python-shaped process pools remain `rejected` or `unsupported-with-diagnostic` by design in this phase.
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
  - rejected CPython-shaped surface index
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
- All CPython-shaped surfaces are `rejected`, `unsupported-with-diagnostic`, `internal-only`, or `host-limited` unless a new Sifr-native production API is justified by M0 and backed by production APIs.
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
- mined-as-substrate-fixture/adapted-for-sifr-api/blocked-on-phase-X/external-signal/waived-with-rationale/rejected CPython evidence families
- rejected/unsupported-with-diagnostic/internal-only CPython-shaped surface index
- final unsupported-with-diagnostic/host-limited waiver index

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No backward-compatibility shims, legacy aliases, deprecated behavior, fallback paths, or CPython-luggage APIs may survive phase exit.
- Compatibility adapters are not accepted in this phase; any future public surface must be a Sifr-native production API with its own design gate.
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
| Scoped process spawn return type | M0 is the binding gate; no pre-M0 default. M0 must choose among `Child`, `TaskHandle[Status, SubprocessError]`, and a distinct `ProcessHandle` and record the choice with pipe-ownership semantics before M4 starts. This row must be updated with the M0 outcome before M4's first implementation PR. |
| TaskGroup offload error binding | M0 is the binding gate for `TaskGroup[E].spawn_blocking` and `TaskGroup[E].spawn_cpu`: M0 must choose `Result[T, E]` with runtime offload errors representable in `E`, `TaskGroup[WorkerError[E]]`, or an explicit mapper closure, then update this row before M3's first implementation PR. This decision must also be reconciled with `JoinSet.join_all().await -> list[Result[T, WorkerError[E]]]` in M3. If M0 chooses `Result[T, E]` with `E` absorbing runtime errors, M0 must either update M3's `JoinSet` return type to `list[Result[T, E]]` or record an explicit rationale for why `JoinSet` and `TaskGroup` use different error-wrapping patterns for the same offload operations. |
| CPython-shaped surfaces | No CPython-shaped adapters are implemented or retained as product goals in this phase. Existing `sifr.asyncio`, `sifr.subprocess`, and `sifr.asyncio.subprocess` are legacy implementation debt to remove, keep internal-test-only, or route to unsupported diagnostics. `sifr.queue`, `sifr.concurrent.futures`, and `sifr.multiprocessing` are `rejected` or `unsupported-with-diagnostic` unless a future Sifr-native API design proves production value over the native APIs. |
| Signal APIs | Safe deterministic APIs are structured signal/shutdown streams for supported host signals (`SIGINT`, `SIGTERM`, and Unix `SIGHUP` where available). `signal.signal`, arbitrary Python handlers, `pthread_sigmask`, and `signal.pause` are `unsupported-with-diagnostic` or `host-limited` evidence entries, not production APIs. |
| Diagnostics and warnings | Runtime diagnostics use structured `tracing` spans/events plus `metrics` counters/histograms behind Sifr diagnostics types. No Python global `warnings` filter adapter ships in this phase. |
| Task/request context | M5 implements explicit `sifr.task.Context` and `sifr.task.ContextKey[T]` with explicit propagation through task spawn and request handoff. There is no implicit dynamic task-local mutation or `contextvars` parity. |
| Typed IPC serialization | M6 uses generated typed IPC schemas over `serde` plus `postcard` after sendability/shareability approval. `IpcSerializable` is stricter than `Sendable`, every schema has stable identity/hash plus compatibility policy, and arbitrary pickle-like object transport is permanently rejected. |
| Subprocess text mode | Because text/i18n runs first, M4 implements subprocess `text=True`, `encoding=...`, and `errors=...` by consuming `sifr.encoding` and explicit text I/O from text/i18n M1. Binary-only subprocess closure is not allowed. |
| Rust ecosystem | Implementation uses [Dependency Policy](../internal_docs/dependency_policy.md) plus the locked Rust Ecosystem Decisions table in this phase doc. Accepted phase crates are classified by dependency ring: Ring 2 `tokio 1.52.3`, `tokio-util 0.7.18`, conditional `futures-util 0.3.32`, Tokio `sync`, Tokio process/std process, and `tracing 0.1.44`; Ring 3 feature-gated `crossbeam-channel 0.5.15`, `rayon 1.12.0`, targeted conditional `rustix 1.1.4`, `metrics 0.24.6`, and conditional `thiserror 2.0.18`; Ring 4 M6 typed-IPC-only `serde 1.0.228` and `postcard 1.1.3`. Tokio remains `current_thread`; blocking I/O parallelism uses Tokio's blocking pool and CPU parallelism uses Rayon. `flume`, `async-channel`, `futures-channel`, direct `parking_lot`, new `once_cell`, `scopeguard`, production `tracing-subscriber`, IPC `serde_json`, `bincode`, `signal-hook`, `nix`, direct `mio`/`bytes`/`dashmap`, runtime `anyhow`/`eyre`, and bespoke replacements are rejected for this phase. |
| `JoinSet` drop | `JoinSet[T, E]` is a linear scoped resource. It must be consumed by `join_all()` or `cancel_all().await` before scope exit. Dropping a live/non-empty `JoinSet` is a compile-time diagnostic; `cancel_all()` returns `CancelOutcome` evidence for every pending item, including already-started/already-completed/could-not-cancel cases. |
| Rayon pool architecture | Top-level `sifr.parallel.map`/`try_map` use a lazily initialized private default `rayon::ThreadPool` built with `rayon::ThreadPoolBuilder` and `available_parallelism()`. Configured parallelism uses explicit `sifr.parallel.Pool(config: PoolConfig)` objects backed by private Rayon pools. The global Rayon pool is never configured, and there is no mutable global pool configuration API. |
| `JoinSet` submission API | `JoinSet[T, E]` accepts homogeneous work through `spawn_blocking(fn) -> JoinItemId`, `spawn_cpu(fn) -> JoinItemId`, and `add(handle: TaskHandle[T, E]) -> JoinItemId`. `add` consumes the handle so results have one observer. `join_all().await -> list[Result[T, WorkerError[E]]]` and `cancel_all().await -> list[CancelOutcome]` consume the set. |
| `JoinSet` result ordering and `JoinItemId` role | `join_all().await` returns results in submission order. `cancel_all().await` returns cancellation evidence in submission order. `JoinItemId` is an opaque user-side correlation token with no further `JoinSet` API; callers use it to index their own submission records, not to query the set. |
| `Pool` instance API | Configured `Pool` objects expose `pool.map(items, fn) -> Result[list[U], WorkerRuntimeError]` and `pool.try_map(items, fn) -> Result[list[U], WorkerError]` with the same result ordering and sendability rules as top-level `parallel.map`/`try_map`. `Pool` has no global shutdown call; active calls borrow the pool, and dropping an idle pool releases the private Rayon pool. |
| Task context API shape | M1 reserves `ctx: Option[sifr.task.Context] = None` parameters on `spawn_scoped` and `TaskGroup` constructors. M1 type-checks and stores the parameter; M5 implements explicit propagation semantics without changing M1 API shape. |
| Existing `sifr.asyncio` veneer | Existing `sifr.asyncio` veneer code is legacy implementation debt, not a compatibility commitment. M1 does not build on it or extend it; M0 records removal, internal-test-only, or unsupported-diagnostic disposition for existing veneer entry points. New task/process/queue APIs are implemented through `sifr.task`, `sifr.sync`, and `sifr.process`; bare `asyncio` imports still receive namespace-contract diagnostics. |
| `race` and `select` loser evidence | `race` returns a container with winner index, typed outcome, and loser cancellation evidence. `select` returns a container with winner branch tag, typed outcome, and loser cancellation evidence. Loser evidence is `list[CancelOutcome]` unless M0 records a stricter equivalent container. Concrete Sifr return-type signatures are recorded in the M0 public API boundary artifact before M1 starts. |
| Shell subprocess effect | Shell subprocess usage is a distinct `@shell_exec` security effect. Sync shell APIs are also `@blocking_io` and are rejected in async contexts unless offloaded. Native async shell process APIs may run in async contexts only through explicit `shell=True` or `Command.shell(...)` and still carry `@shell_exec`. |
