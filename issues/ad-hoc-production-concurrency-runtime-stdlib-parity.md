# Ad Hoc Phase: Production Concurrency, Process, And Runtime Substrate

Status: draft
Phase placement: ad hoc expansion phase after the stdlib boundary refactor and after the async workload/effect model is stable enough to enforce blocking-process diagnostics.
Phase owner: runtime/stdlib implementation with compiler effect, ownership, import, and codegen support

## Objective

Build the production-grade concurrency, scheduling, synchronization, subprocess, shutdown, diagnostics, and offload substrate required by real Sifr programs and by later web, worker, data, CLI, and interop phases.

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

CPython-shaped APIs may be added only as thin adapters over these primitives, and only when they are production-useful, maintainable, current, non-deprecated, and compatible with Sifr's static typed ownership model.

This phase does not add backward-compatibility or legacy support. Bare CPython stdlib imports, historical aliases, deprecated APIs, compatibility shims, fake generator paths, pickle-style fallbacks, hidden bridge names, and partial public toy modules are not implemented; they receive diagnostics or waivers.

## Related Phases

- Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md).
- Text and internationalization parity is tracked in [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md).
- Subprocess text mode, warning output encodings, locale-aware formatting, and demos relying on `open(..., encoding=...)` depend on text/i18n `milestone_text_i18n_1: Codecs Registry, Encodings, And Text I/O Integration`.
- This phase assumes [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md) is complete: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.

## Support Tiers

Every proposed API, test family, and CPython-derived surface must be assigned one support tier during M0:

| Tier | Meaning | Examples |
| --- | --- | --- |
| `production-substrate` | Required runtime foundation for real programs and later phases | scheduler, cancellation, channels, sync primitives, process runtime, signals |
| `production-public` | Recommended Sifr API for user code | `sifr.task`, `sifr.sync`, `sifr.process`, `sifr.runtime`, `sifr.parallel`, `sifr.resource` |
| `internal-runtime` | Implementation detail only | Tokio runtime, Tokio process, Tokio sync, crossbeam/Rayon-like internals |
| `adapter-later` | Optional wrapper over production APIs after the substrate is complete | selected `sifr.asyncio`, selected `sifr.subprocess`, selected `sifr.concurrent.futures` |
| `deferred` | Potential future production API requiring a separate design gate | process workers, selected queue adapters, task context extensions |
| `rejected` | Too dynamic, global, Python-specific, legacy, or unsafe for Sifr | `threading` parity, raw event-loop policies, callback transports, pickle transport, `signal.signal` handlers |

No phase milestone may be marked complete while any surface remains unclassified.

## Cross-Phase Dependency Contract

The split stdlib phases are not an implied ship order. This phase may implement binary/process/synchronization/runtime behavior independently, but cross-phase consumer features are blocked until their provider phase is complete:

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
- `sifr.sync` channels are the canonical queue-like primitive, but the production channel/backpressure/sync surface is not yet closed for all worker/process use cases.
- `sifr.contextlib`, `sifr.warnings`, `sifr.signal`, `sifr.queue`, `sifr.concurrent.futures`, and `sifr.multiprocessing` are not production substrate surfaces. Their CPython shapes must be classified rather than assumed.

The Phase 32 async model remains binding:

- Native async process, queue/channel, and synchronization APIs must be real suspension points.
- Sync APIs that can block must be classified as `@blocking_io`.
- CPU-heavy work must use the existing `@cpu_heavy`/offload model.
- Direct calls to blocking sync APIs from `async def` remain compiler errors unless routed through native async APIs or explicit offload.
- The compiler must not expose Tokio, event-loop objects, raw callback transports/protocols, or runtime internals as the normal user model.

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

Selected adapters can exist later only when M0 classifies them as `adapter-later`, they delegate to the production APIs above, and they do not introduce legacy behavior or dynamic fallback paths.

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

## Sendability And Shareability Contract

Before raw task spawning, blocking offload, CPU offload, thread pools, process workers, or cross-boundary closure captures ship, Sifr must define a user-visible sendability/shareability model:

- `Send[T]`-equivalent eligibility for moving values across task/thread/process boundaries
- `Sync[T]`/`Share[T]`-equivalent eligibility for shared references
- lifetime-like capture constraints for spawned work that may outlive the lexical caller
- non-send capture diagnostics
- shared mutable state diagnostics
- safe closure capture rules for task, blocking, CPU, and process-worker APIs
- typed compile-time diagnostics where unsupported values cross worker boundaries

This is a phase-wide gate, not an executor-only caveat. APIs such as `ThreadPoolExecutor.initializer`, process-pool `initializer`, and process-worker payload capture remain `unsupported` or `deferred` until this contract is complete.

## Typed Errors Instead Of Exceptions

All fallible APIs must expose typed error results:

- `TaskError`, `TaskGroupError`, `Cancelled`, `TimedOut`
- `ChannelClosed`, `ChannelFull`, `ChannelEmpty`, `BackpressureError`
- `SubprocessError`, `CalledProcessError`, `TimeoutExpired`, `PipeError`
- `WorkerError`, `WorkerRuntimeError`, `OffloadError`
- `SignalError`, `ShutdownError`
- `ContextError`, `DiagnosticError`

Names may align with CPython evidence where useful, but the operational contract is Sifr `Result`/`Option` and typed sums, not exception-driven control flow. Convenience APIs such as `check_output` adapters must return typed failure evidence instead of throwing.

## Non-Goals And Permanent Boundaries

The following are not accepted as silent omissions. They must be rejected, deferred, or explicitly waived with tests:

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

1. `milestone_concurrency_runtime_0` first. No implementation milestone starts until product tiers, CPython evidence inventory, typed error map, sendability/shareability gate, workload/effect classification, and adapter/deferred/rejected decisions are checked in.
2. `milestone_concurrency_runtime_1` defines the structured async task substrate and blocks async synchronization, async subprocess, and offload APIs that need task cancellation/deadline semantics.
3. `milestone_concurrency_runtime_2` closes synchronization/backpressure on top of the task model.
4. `milestone_concurrency_runtime_3` implements blocking and CPU offload after sendability/shareability is accepted.
5. `milestone_concurrency_runtime_4` implements native process/subprocess after task, sync, and offload semantics are stable.
6. `milestone_concurrency_runtime_5` implements shutdown, signal streams, cleanup, task context, and structured diagnostics after the task/process contracts are available.
7. `milestone_concurrency_runtime_6` designs and implements typed IPC, with process-worker pools deferred unless the IPC gate is complete.
8. `milestone_concurrency_runtime_7` closes docs, demos, validation, waivers, and adapter decisions last.

## Milestones

### milestone_concurrency_runtime_0: Product Boundary And Rust Concurrency Contract

Scope:

- Add a machine-readable inventory under `verification/stdlib/concurrency_runtime_substrate_inventory.*`.
- Scan every source/test/doc file listed in `Source Of Truth`.
- Extract public functions, classes, constants, methods, common keyword forms, deprecation/legacy markers, and test-class/test-method names.
- Assign every inventory entry one support tier: `production-substrate`, `production-public`, `internal-runtime`, `adapter-later`, `deferred`, or `rejected`.
- Assign every public surface one terminal state: `done`, `intentional-diff`, `unsupported`, `host-limited`, `adapter-later`, `deferred`, or `rejected`.
- Assign every CPython test family one state: `adopted`, `adapted`, or `waived`.
- Define the public/native API boundary for `sifr.task`, `sifr.sync`, `sifr.process`, `sifr.runtime`, `sifr.parallel`, `sifr.signal`, `sifr.resource`, and `sifr.ipc`.
- Define sendability/shareability rules for task/thread/process captures.
- Define blocking/cpu-heavy effect rules.
- Define cancellation/deadline/timeout semantics.
- Define typed worker/task/process/signal/diagnostic error models.
- Define the no-public-Tokio rule and Tokio feature plan for `tokio::process`, `tokio::io`, `tokio::sync`, and signal support where internally adopted.
- Add import-resolution tests for canonical `sifr.*` module names and negative diagnostics for bare CPython stdlib import attempts.
- Assign every deprecated, historical, or legacy-only CPython entry the terminal state `unsupported`, `intentional-diff`, or `rejected`.

Definition of done:

- The backlog is derived from CPython source/tests plus Sifr runtime needs, not hand-written memory.
- Every proposed API is classified by support tier and terminal state.
- M1-M7 implementation PRs have concrete backlog entries rather than prose-only scope.
- No CPython module shape is still treated as automatic production scope.

### milestone_concurrency_runtime_1: Structured Async Runtime

Scope:

- Add or close the production `sifr.task` surface:
  - `Task[T, E]`
  - `TaskHandle[T, E]`
  - `TaskGroup[T, E]`
  - `spawn_scoped`
  - `sleep`
  - `timeout`
  - `deadline`
  - `cancel_scope`
  - `join_all`
  - `race`/`select`
- Define detached-task policy: forbidden by default or explicit with failure observation.
- Ensure task failures cannot be silently dropped.
- Define handle ownership for task/future observation and cancellation.
- Define typed result/cancellation/timeout behavior for join/race/wait/timeout/task groups.
- Add CPython-derived evidence from `asyncio.TaskGroup`, `gather`, `wait`, `wait_for`, `timeout`, and `sleep`, but map production behavior to `sifr.task`.
- Keep `sifr.asyncio` as `adapter-later` unless M0 proves a narrow adapter is production-useful and fully backed by `sifr.task`.
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
- `TaskGroup` aggregates child failures as `TaskGroupError[E]` or equivalent containing all observed child errors.
- CPython async task test families are adopted, adapted, or waived against the Sifr-native task model.
- No raw Tokio/event-loop types leak.

### milestone_concurrency_runtime_2: Synchronization, Channels, And Backpressure

Scope:

- Add or close the production `sifr.sync` surface:
  - `Channel[T]`
  - `BoundedChannel[T]`
  - `UnboundedChannel[T]`
  - `AsyncChannel[T]`
  - `Mutex[T]`
  - `RwLock[T]`
  - `Semaphore`
  - `Event`/`Notify`
  - `Barrier`
  - `Once`
- Define bounded capacity, backpressure, close/drop, sender/receiver ownership, and cancellation behavior.
- Mark sync operations that can block as `@blocking_io` when used from async contexts.
- Keep `sifr.queue` and `sifr.asyncio.Queue` as adapters-later unless M0 proves a specific thin adapter is useful and not shaping the primitive.
- Defer `PriorityQueue`, `LifoQueue`, `SimpleQueue`, and `task_done`/`join` accounting unless they are adapters over stable channels with deterministic semantics.

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
- Blocking-in-async diagnostics cover sync waits.
- No raw Tokio sync type leaks.
- CPython queue/async-queue test families are adopted, adapted, waived, or classified as adapter-later with evidence.

### milestone_concurrency_runtime_3: Blocking And CPU Offload

Scope:

- Add or close the production `sifr.runtime` and `sifr.parallel` offload surface:
  - `spawn_blocking`
  - `spawn_cpu`
  - `JoinSet[T, E]` or equivalent
  - `parallel.map`
  - `parallel.try_map`
  - offload pool sizing and shutdown policy
- Separate async tasks, blocking I/O offload, CPU-heavy parallel work, and long-running supervised processes.
- Enforce blocking-in-async diagnostics.
- Enforce CPU-heavy diagnostics and explicit offload.
- Map task, worker, foreign/runtime boundary, and panic-like runtime failures into typed evidence.
- Use CPython `concurrent.futures` as evidence for future/cancellation/deadline edge cases, not as the production API.
- Keep `sifr.concurrent.futures`, `Future.result(timeout=...)`, `Executor.map`, `as_completed`, and `ThreadPoolExecutor` as adapters-later unless M0 proves a wrapper has production migration value.

CPython tests to mine:

- `Lib/test/test_concurrent_futures/`

Rust/runtime candidates:

- `rayon`-like data parallelism
- `tokio::task::spawn_blocking`

Definition of done:

- Blocking calls inside async receive compiler diagnostics.
- Offloaded work returns typed results.
- Worker runtime failures become typed evidence.
- CPU-heavy parallel APIs are distinct from blocking I/O APIs.
- Homogeneous result/cancellation/deadline behavior has CPython-derived adapted fixtures.
- Any CPython behavior requiring public `threading` objects is rejected, deferred, or waived.

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
  - explicit `shell` effect classification
- Preserve binary pipe mode in this phase; text/encoding/error mode waits for text/i18n `milestone_text_i18n_1`.
- Keep `sifr.subprocess` and `sifr.asyncio.subprocess` as adapters-later unless M0 proves a narrow wrapper is worth shipping.
- If a subprocess adapter is accepted, `subprocess.getoutput` and `subprocess.getstatusoutput` remain unsupported as legacy shell-invocation helpers.

CPython tests to mine:

- `Lib/test/test_subprocess.py`
- `Lib/test/test_asyncio/test_subprocess.py`

Rust/runtime candidates:

- `std::process`
- `tokio::process`

Definition of done:

- Sync and async subprocess loopback tests pass on the supported host matrix.
- Pipe ownership prevents double-close and use-after-close.
- Timeout/cancellation semantics are documented and tested.
- Shell usage is explicit and effect-classified.
- No adapter can bypass owned process/pipe lifecycle.

### milestone_concurrency_runtime_5: Shutdown, Signals, Cleanup, Context, And Diagnostics

Scope:

- Add or close structured signal/shutdown support:
  - `sifr.signal.Signal`
  - `ctrl_c`
  - `terminate`
  - `shutdown_stream`
  - supported signal constants/enum-like values
  - `strsignal` where host-supported
- Reject arbitrary `signal.signal(handler)` callback registration.
- Record `pause`, `getsignal`, `raise_signal`, and `pthread_sigmask` as `done`, `unsupported`, `host-limited`, `deferred`, or `rejected` with CPython evidence.
- Add or close deterministic cleanup support:
  - `sifr.resource.ExitStack`
  - `sifr.resource.AsyncExitStack`
  - `closing`
  - `aclosing`
  - `nullcontext`
- Defer or reject Python convenience helpers such as `redirect_stdout`, `redirect_stderr`, `chdir`, `suppress`, `contextmanager`, and `asynccontextmanager` unless M0 proves they are production APIs rather than compatibility luggage.
- Add a Sifr-native task/request context design if needed by tracing, deadlines, cancellation metadata, and future web observability:
  - `sifr.task.Context`
  - `sifr.task.ContextKey[T]`
  - explicit opt-in propagation across task groups
  - no implicit dynamic Python `contextvars` behavior
- Prefer compiler diagnostics, structured runtime diagnostics, logging/tracing events, and library deprecation metadata over Python `warnings` global filter parity.
- If runtime warnings are retained, make them explicit, structured, non-global by default, scope-local where possible, thread-safe, and not exception-like.

CPython tests to mine:

- `Lib/test/test_signal.py`
- `Lib/test/test_io/test_signals.py`
- `Lib/test/test_contextlib.py`
- `Lib/test/test_contextlib_async.py`
- `Lib/test/test_warnings/`

Definition of done:

- Production server/worker shutdown scenarios are covered.
- Cleanup is deterministic under cancellation.
- Structured diagnostics do not become hidden exceptions.
- Warning/filter global-state parity is rejected or narrowed to an explicit structured runtime diagnostics design.
- Generator-decorator helpers are recorded as `unsupported` with a revisit rule; no partial fake generator path is allowed.

### milestone_concurrency_runtime_6: Typed IPC And Future Process Workers

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
- Keep `ProcessPoolExecutor`, `multiprocessing.Process`, `multiprocessing.Queue`, `multiprocessing.Pipe`, and `multiprocessing.Pool` deferred unless this typed IPC gate is complete and M0 proves a production need beyond Python's GIL workaround.
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

- typed local serialization such as `bincode` or `postcard` only after typed IPC design approval
- platform process primitives from M4

Definition of done:

- IPC is safe, typed, versioned, and panic-free.
- Unsupported payloads are compile-time diagnostics where possible.
- Process pools remain deferred unless this gate is complete.
- Any accepted process-worker API exists for isolation/supervision/interop, not as Sifr's default CPU parallelism story.

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
  - any accepted adapter-later wrapper
- Update internal architecture docs for:
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
  - every CPython test family has `adopted`, `adapted`, or `waived` evidence
  - every waiver has a revisit rule and regression fixture
  - every host-limited surface records the supported host matrix
- Run an external review loop on the final inventory and close any blocking finding before phase completion.
- External review owner is the runtime/stdlib phase owner plus the designated compiler/runtime reviewer recorded in the execution ledger. If review output is unavailable for five working days after the review artifact is posted, the phase owner may proceed only by recording the attempted review, open questions, and a conservative self-review in the ledger.

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
- All compatibility surfaces are `adapter-later`, `deferred`, or `rejected` unless justified by M0 and backed by production APIs.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in added runtime surfaces.
- Async and sync APIs follow the Phase 32 workload and cancellation model.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.json`
- `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test files scanned
- public/native API tier decisions
- adopted/adapted/waived CPython test families
- adapter-later/deferred/rejected compatibility index
- final unsupported/intentional-diff/host-limited waiver index

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

## Open Planning Questions To Resolve In `milestone_concurrency_runtime_0`

1. What exact sendability/shareability rules govern task, blocking, CPU, and process-worker captures?
2. Which Sifr task APIs are stable production-public versus internal runtime details?
3. Which CPython-shaped adapters, if any, have enough migration value to justify implementation?
4. Which signal APIs are safe and deterministic across the supported host matrix?
5. What is the structured runtime diagnostics model, and does any `warnings` adapter survive?
6. What task/request context model is needed before web observability work?
7. What is the typed IPC serialization contract for future process workers?
8. Which subprocess text-mode APIs remain blocked until text/i18n `milestone_text_i18n_1` closes?

These questions must be answered in the phase execution ledger before implementing the affected milestone.
