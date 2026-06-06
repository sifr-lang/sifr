# Ad Hoc Phase: Production Concurrency And Runtime Stdlib Parity

Status: draft
Phase placement: ad hoc expansion phase after the stdlib boundary refactor and after the async workload/effect model is stable enough to enforce blocking-process diagnostics.
Phase owner: stdlib/runtime implementation with compiler effect, ownership, import, and codegen support

## Objective

Close the production stdlib gaps for queues, subprocesses, process-backed concurrency, and runtime ergonomics:

- asyncio control/synchronization compatibility needed by this phase: selected `asyncio` task, wait, timeout, and synchronization primitives
- queues and async queues: `queue`, `asyncio.Queue`
- subprocess and async subprocess: `subprocess`, `asyncio.subprocess`
- executors and process concurrency: `concurrent.futures`, `multiprocessing`
- runtime ergonomics and shutdown behavior: `contextlib`, `warnings`, `signal`

This phase is complete when each target surface has either:

- current-CPython-shaped source parity with Sifr-safe semantics,
- a native Sifr runtime implementation that backs that compatibility surface,
- or an explicit, tested waiver with rationale, revisit rule, and CPython test-family evidence.

This phase does not add backward-compatibility or legacy support. Parity means the current supported CPython stdlib API shape and behavior adapted under Sifr's canonical `sifr.*` namespace with Sifr's static, typed, ownership-safe model. Bare CPython stdlib imports, historical aliases, deprecated APIs, compatibility shims, fake generator paths, pickle-style fallbacks, and hidden bridge names are not implemented; they receive diagnostics or waivers.

## Related Phases

- Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md).
- Text and internationalization parity is tracked in [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md).
- Subprocess `text=True`, `encoding=...`, `errors=...`, locale-aware formatting, and warning text encoding depend on text/i18n `milestone_text_i18n_1: Codecs Registry, Encodings, And Text I/O Integration`. Until that milestone closes, this phase implements binary pipe semantics and records text-mode APIs as `blocked-on-text-i18n-m1`/adapted.
- This phase assumes [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md) is complete: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.

## Cross-Phase Dependency Contract

The three split phases are not an implied ship order. This phase may implement binary/process/queue/runtime behavior independently, but cross-phase consumer features are blocked until their provider phase is complete:

- Text/i18n `milestone_text_i18n_1` is the hard prerequisite for subprocess text mode, warning output encodings, locale-sensitive warning formatting, and demos that rely on `open(..., encoding=...)`.
- Network/web owns network stream compatibility entry points such as `asyncio.open_connection`; this phase owns the `asyncio` control/synchronization closure audit plus `asyncio.Queue` and `asyncio.subprocess` additions.
- Async scheduler/task primitives are prior runtime infrastructure, but this phase owns their CPython compatibility closure states for the primitives consumed by queue/process APIs.
- This phase owns the private thread-pool substrate needed by `ThreadPoolExecutor`, but it does not add a public `threading` module. If a CPython `concurrent.futures` behavior requires public `threading.Thread` semantics, that behavior is `unsupported` or `adapted` with explicit evidence.

## Source Of Truth

The authoritative CPython source tree for this phase is:

- `/Users/yaseralnajjar/work/sifr/cpython`

The implementation must scan and classify these CPython files before each milestone implementation PR:

| Domain | CPython library sources | CPython test sources | Native backing sources |
| --- | --- | --- | --- |
| subprocess/process | `Lib/subprocess.py`, `Lib/asyncio/subprocess.py`, `Doc/library/subprocess.rst`, `Doc/library/asyncio-subprocess.rst` | `Lib/test/test_subprocess.py`, `Lib/test/test_asyncio/test_subprocess.py` | `Modules/_posixsubprocess.c`, `Modules/clinic/_posixsubprocess.c.h` |
| queue/concurrency | `Lib/queue.py`, `Lib/asyncio/*.py`, `Lib/concurrent/futures/*.py`, `Lib/multiprocessing/*.py`, `Doc/library/queue.rst`, `Doc/library/asyncio.rst`, `Doc/library/concurrent.futures.rst`, `Doc/library/multiprocessing*.rst` | `Lib/test/test_queue.py`, `Lib/test/test_asyncio/test_queues.py`, `Lib/test/test_asyncio/test_tasks.py`, `Lib/test/test_asyncio/test_taskgroups.py`, `Lib/test/test_asyncio/test_waitfor.py`, `Lib/test/test_asyncio/test_timeouts.py`, `Lib/test/test_asyncio/test_locks.py`, `Lib/test/test_asyncio/test_runners.py`, `Lib/test/test_concurrent_futures/`, `Lib/test/_test_multiprocessing.py`, `Lib/test/test_multiprocessing_main_handling.py`, `Lib/test/test_multiprocessing_spawn/`, `Lib/test/test_multiprocessing_fork/`, `Lib/test/test_multiprocessing_forkserver/` | `Modules/_queuemodule.c`, `Modules/_multiprocessing/*`, `Modules/clinic/_queuemodule.c.h` |
| context/warnings/signal | `Lib/contextlib.py`, `Lib/warnings.py`, `Doc/library/contextlib.rst`, `Doc/library/warnings.rst`, `Doc/library/signal.rst` | `Lib/test/test_contextlib.py`, `Lib/test/test_contextlib_async.py`, `Lib/test/test_warnings/`, `Lib/test/test_signal.py`, `Lib/test/test_io/test_signals.py` | `Modules/signalmodule.c`, `Python/_warnings.c`, `Lib/_py_warnings.py` |

Path note: CPython paths above are relative to `/Users/yaseralnajjar/work/sifr/cpython`.

## Current Sifr Baseline

- `sifr.subprocess` has sync helpers and `CompletedProcess`, but no `Popen`, pipe lifecycle, timeout, signal, or async subprocess object model.
- `sifr.asyncio` is a compatibility veneer over the canonical task model, but intentionally omits raw event loops, subprocesses, process pools, and transport/protocol APIs.
- Core `asyncio` scheduler/task helpers are existing async-model infrastructure, not new scope here. This phase owns only `asyncio.Queue`, `asyncio.subprocess`, and queue/process-specific compatibility glue.
- `sifr.sync` channels are the canonical queue-like async primitive, but `queue.Queue` accounting, `PriorityQueue`, `LifoQueue`, `SimpleQueue`, and multiprocessing queues are missing.
- `sifr.contextlib`, `sifr.warnings`, `sifr.signal`, `sifr.queue`, `sifr.concurrent.futures`, and `sifr.multiprocessing` are not present as production stdlib surfaces.

The Phase 32 async model remains binding:

- Native async process and queue APIs must be real suspension points.
- Sync queue/process APIs that can block must be classified as `@blocking_io`.
- CPU-heavy work submitted to executors must use the existing `@cpu_heavy`/offload model.
- Direct calls to blocking sync APIs from `async def` remain compiler errors unless routed through native async APIs or explicit offload.
- The compiler must not expose Tokio, event-loop objects, raw callback transports/protocols, or runtime internals as the normal user model.

## Parity Definition

This phase targets current CPython-shaped interfaces under the canonical `sifr.*` namespace, not legacy compatibility layers or bare CPython import compatibility.

For each module in scope:

1. Support canonical Sifr stdlib imports for the CPython-shaped surface (`from sifr.queue import Queue`, `from sifr.subprocess import Popen`, etc.).
2. Do not add bare CPython module-name imports as aliases for `sifr.*`. Bare forms such as `from queue import Queue` or `from subprocess import Popen` should receive the namespace-contract diagnostic once normal user/package resolution fails.
3. Match CPython function/class names, constructor forms, constants, and common keyword arguments where compatible with Sifr's static type system.
4. Adapt CPython exception behavior into Sifr-safe `Result[T, E]`, `Option[T]`, or compile-time diagnostics.
5. Keep host-specific behavior explicitly marked `host-limited`.
6. Keep CPython implementation-detail, deprecated, and historical compatibility behavior waived rather than reimplemented blindly.

Every reviewed CPython test family must end in exactly one state: `adopted`, `adapted`, or `waived`. Every public surface must end in exactly one state: `done`, `intentional-diff`, `unsupported`, or `host-limited`. `open` is forbidden at phase exit.

## Milestone Dependency Graph

1. `milestone_concurrency_runtime_0` first. No implementation milestone starts until the inventory, CPython test matrix, import plan, shared error mapping, and workload/effect classification are checked in.
2. `milestone_concurrency_runtime_1` before executor and multiprocessing queue work. Its asyncio control/synchronization closure audit is the first sub-step inside M1 and must close before `asyncio.Queue` or `asyncio.subprocess` claims CPython conformance.
3. `milestone_concurrency_runtime_2` before process pools or multiprocessing pools. Process lifecycle, pipes, timeout, and signals are the substrate for process-backed concurrency, and its async subprocess conformance waits on M1's asyncio closure audit.
4. `milestone_concurrency_runtime_3` may implement `Future`, `Executor`, and `ThreadPoolExecutor` after M1; `ProcessPoolExecutor` waits for M4 typed IPC.
5. `milestone_concurrency_runtime_4` implements typed IPC, `ProcessPoolExecutor`, and the bounded multiprocessing subset after M2 and the M4 typed IPC design are ready.
6. `milestone_concurrency_runtime_5` can run in parallel after M0 except where `signal` behavior depends on M2 subprocess lifecycle.
7. `milestone_concurrency_runtime_6` closes docs, demos, validation, and waivers last.

## Architecture Principles

### Native Runtime First, Compatibility Second

- Tokio remains the backing async runtime because the generated task runtime already depends on `tokio` and `sifr_stdlib::StdlibFeature::Tokio`.
- M0 must expand the Tokio feature plan for `tokio::process`, `tokio::io`, `tokio::sync`, and signal support where adopted.
- `sifr.process` / private intrinsics own child-process lifecycle, pipes, cancellation, and signal delivery.
- `sifr.sync` / private intrinsics own queue, channel, and async backpressure primitives.
- CPython-shaped canonical Sifr modules delegate to those primitives and must not duplicate target-runtime logic.

### Typed Errors Instead Of Exceptions

All fallible APIs must expose typed error results:

- `SubprocessError`, `CalledProcessError`, `TimeoutExpired`
- `QueueEmpty`, `QueueFull`, `QueueShutDown`
- `CancelledError`, `ExecutorError`, `BrokenExecutor`, `TimeoutError`
- `SignalError`
- `ContextError`, `WarningError`

Names may align with CPython where possible, but the operational contract is Sifr `Result`/`Option`, not exception-driven control flow. `check_*` convenience APIs such as `subprocess.check_output` must return typed failure evidence instead of throwing.

### Panic-Free Runtime Contract

Generated Rust for these APIs must not contain data-dependent `.unwrap()`, `.expect()`, or `panic!` on user-controlled process, pipe, queue, warning, signal, or executor data.

### No Dynamic Serialization Fallback

No arbitrary pickle-equivalent process-pool transport may be introduced. Process-backed APIs require typed IPC with explicit supported payload types. Unsupported payloads are compile-time errors where possible and typed runtime errors otherwise.

## Non-Goals And Permanent Boundaries

The following are not accepted as silent omissions. They must be either implemented or explicitly waived with tests:

- raw event-loop policy mutation
- callback transport/protocol APIs as the primary Sifr model
- `contextvars` module parity, including single-task `ContextVar` reads/writes and implicit per-task context propagation; unsupported in this phase unless a separate contextvars phase lands first
- public `threading` module parity; only the private thread-pool substrate needed by `ThreadPoolExecutor` is in scope
- `threading.local`; unsupported in this phase with diagnostics because public thread-local storage belongs to future public `threading`/context isolation work
- arbitrary object pickling for process pools
- process pools without the stable typed IPC serialization contract designed and accepted inside `milestone_concurrency_runtime_4`; no separate external prerequisite is implied
- `signal.signal` custom handler registration; unsupported in this phase
- `contextmanager` and `asynccontextmanager`; formally waived in this phase because CPython-compatible generator `send`/`throw`/`close` and async-generator cleanup semantics are outside this phase's scope. The revisit rule is a future generator/async-generator semantics phase; until that exists, these decorators remain `unsupported` with diagnostics and CPython evidence rather than blocked implementation work.
- `multiprocessing.Value`, `multiprocessing.Array`, and `multiprocessing.shared_memory`; unsupported until typed shared-memory ownership and unlink/drop rules are proven
- fork/forkserver semantics without host-specific ownership evidence
- mutation of interpreter-global warning/filter state from unstructured concurrent contexts

## Milestones

### milestone_concurrency_runtime_0: CPython Inventory, Error Mapping, And Effect Classification

Scope:

- Add a machine-readable parity inventory under `verification/stdlib/concurrency_runtime_parity_inventory.*`.
- Scan every source/test/doc file listed in `Source Of Truth`.
- Extract public functions, classes, constants, methods, common keyword forms, deprecation/legacy markers, and test-class/test-method names.
- Add CPython-derived e2e fixtures:
  - `cpython_asyncio_core_subset.sifr`
  - `cpython_asyncio_sync_subset.sifr`
  - `cpython_queue_subset.sifr`
  - `cpython_asyncio_queue_subset.sifr`
  - `cpython_subprocess_full_subset.sifr`
  - `cpython_asyncio_subprocess_subset.sifr`
  - `cpython_concurrent_futures_subset.sifr`
  - `cpython_multiprocessing_subset.sifr`
  - `cpython_contextlib_subset.sifr`
  - `cpython_warnings_subset.sifr`
  - `cpython_signal_subset.sifr`
- Add import-resolution tests for canonical `sifr.*` module names and negative diagnostics for bare CPython stdlib import attempts.
- Add shared error mapping for all concurrency/runtime target domains.
- Add workload classifications for every blocking sync process, queue, executor, and signal API.
- Add a named asyncio closure-audit checklist and pass/fail gate for the task/wait/timeout/synchronization primitives consumed by M1 and M2.
- Assign each inventory entry one owner milestone and one terminal state.
- Assign every deprecated, historical, or legacy-only entry the terminal state `unsupported` or `intentional-diff`. M0 may implement only current, non-deprecated target CPython surfaces that remain elegant under Sifr semantics.

Definition of done:

- The backlog is derived from CPython source/tests, not hand-written memory.
- Every target module has a first-pass surface matrix and CPython test-family matrix.
- M1-M6 implementation PRs have concrete backlog entries rather than prose-only scope.

### milestone_concurrency_runtime_1: Asyncio Core, Queue, And Async Queue Parity

Scope:

- Close the `asyncio` control/synchronization subset consumed by this phase:
  - `run`, `create_task`, `gather`, `wait`, `wait_for`, `sleep`, `timeout`
  - `TaskGroup`
  - `Event`, `Lock`, `Semaphore`, `Condition`
  - raw event-loop policy and callback transport APIs remain out of scope
  - `TaskGroup` maps to Sifr's fixed structured-concurrency executor and does not require exposing event-loop policies or callback transports
  - `gather` borrows or clones task/future observation handles and must not consume caller-owned cancel handles
  - `gather(return_exceptions=True)` returns per-element typed results, such as `List[Result[T, FutureError[E]]]`, not `List[T]`
  - `gather(return_exceptions=False)` uses Sifr's structured failure policy and returns a typed aggregate/cancellation result rather than raising
  - `TaskGroup` child failures are aggregated as `TaskGroupError[E]` or equivalent containing all observed `FutureError[E]` values; single-future `FutureError[E]` is not reused for multi-child failure without aggregation
  - `contextvars.ContextVar` is unsupported in this phase, including single-task reads/writes and cross-task propagation; M1 must add fixtures proving all `contextvars` usage is diagnosed or explicitly waived, not silently ignored
- The asyncio closure audit must finish before `asyncio.Queue` or `asyncio.subprocess` fixtures can be marked conformant. The audit passes only when M1 records all of these as binary pass/fail checklist items:
  - each listed `asyncio` primitive has a terminal state (`done`, `adapted`, `unsupported`, or `host-limited`) in the inventory
  - typed result/cancellation/timeout behavior is specified for `gather`, `wait`, `wait_for`, `timeout`, and `TaskGroup`
  - handle ownership is specified for task/future observation, cancellation, and task-group child lifetimes
  - synchronization primitives have deterministic wakeup/cancellation/drop behavior or explicit waivers
  - raw event-loop policy, callback transport/protocol, and `contextvars` usages produce diagnostics or waivers rather than silent partial support
  - CPython test families listed below are classified as `adopted`, `adapted`, or `waived` with at least one regression fixture per adopted/adapted behavior
  - the execution ledger records `asyncio_closure_audit: pass`; any failed item blocks M1 conformance claims and M2 async-subprocess conformance
- Add `queue`:
  - `Queue`
  - `PriorityQueue`
  - `LifoQueue`
  - `SimpleQueue`
  - `Empty`
  - `Full`
  - `ShutDown`
  - `put`, `get`, `put_nowait`, `get_nowait`, `task_done`, `join`, `shutdown`, `qsize`, `empty`, `full`
- Add or extend `asyncio.Queue` compatibility on top of `sifr.sync`:
  - `put`, `get`, `put_nowait`, `get_nowait`
  - bounded capacity and backpressure
  - `join`/`task_done` if accounting can be made deterministic
- Mark sync queue operations that can block as `@blocking_io` when used from async contexts; prefer async queue/channel APIs.

CPython tests to mine:

- `Lib/test/test_queue.py`
- `Lib/test/test_asyncio/test_tasks.py`
- `Lib/test/test_asyncio/test_taskgroups.py`
- `Lib/test/test_asyncio/test_waitfor.py`
- `Lib/test/test_asyncio/test_timeouts.py`
- `Lib/test/test_asyncio/test_locks.py`
- `Lib/test/test_asyncio/test_runners.py`
- `Lib/test/test_asyncio/test_queues.py`

Rust/runtime candidates:

- `crossbeam-channel`
- `tokio::sync`

Definition of done:

- Thread queues and async queues have deterministic blocking/backpressure/cancellation behavior.
- Asyncio core/synchronization primitives consumed by this phase are closed as `done`, `adapted`, or `unsupported` with CPython test evidence.
- `gather(return_exceptions=...)` and `TaskGroup` have fixtures for handle ownership, per-element typed results, fail-fast/structured cancellation, and aggregated child failures.
- Queue accounting preserves CPython-compatible `task_done`/`join` semantics or records exact waivers.
- Async queue operations are real suspension points.

### milestone_concurrency_runtime_2: Subprocess, Popen, Async Subprocess, And Signals

Scope:

- Expand `subprocess` from helper-only to CPython-shaped surface:
  - `run`, `call`, `check_call`, `check_output`
  - `CompletedProcess`, `CalledProcessError`, `TimeoutExpired`
  - `Popen`
  - `PIPE`, `STDOUT`, `DEVNULL`
  - `Popen.poll`, `wait`, `communicate`, `send_signal`, `terminate`, `kill`, `close`
  - stdin/stdout/stderr pipes as owned stream resources
  - timeout handling
  - binary pipe mode in this phase; text/encoding/error mode waits for text/i18n `milestone_text_i18n_1`
  - `getoutput` and `getstatusoutput` are unsupported in this phase as legacy shell-invocation helpers; users should use typed `run(..., shell=...)` forms only where shell execution is explicitly allowed and effect-classified
- Add `asyncio.subprocess` compatibility:
  - `create_subprocess_exec`
  - `create_subprocess_shell`
  - async `Process.wait`
  - async `Process.communicate`
  - async pipe read/write/close
- Add `signal` module basics required by subprocess and production shutdown:
  - signal constants for supported host
  - `Signals` enum-like surface where compatible
  - `raise_signal`
  - `strsignal`
  - `getsignal` returns supported default/ignored state only; custom Python handler state remains unsupported with `signal.signal`
  - `pause` is `unsupported` in this phase because safe custom handler registration and structured async signal wakeup are not adopted; static uses produce diagnostics, the waiver records CPython evidence, and the revisit rule is a future safe signal-handler or structured signal-stream phase
  - `pthread_sigmask` only if host-limited support is deliberately added
  - `signal.signal` custom handler registration is `unsupported` in this phase
  - async signal notification may be added as a canonical `sifr.process`/`sifr.signal` stream if it integrates with structured cancellation and avoids arbitrary signal-handler closures
- Integrate structured cancellation:
  - cancelling async process wait requests child termination only when API says so
  - `communicate` drains pipes deterministically
  - timeout kills/terminates according to documented Sifr policy and returns typed evidence
- Mark sync process APIs as `@blocking_io`.

CPython tests to mine:

- `Lib/test/test_subprocess.py`
- `Lib/test/test_asyncio/test_subprocess.py`
- `Lib/test/test_signal.py`
- `Lib/test/test_io/test_signals.py`

Rust/runtime candidates:

- `std::process`
- `tokio::process`
- `nix` or platform-specific crates only for signal handling that cannot be covered by std/Tokio

Definition of done:

- Sync and async subprocess loopback tests pass on the supported host matrix.
- Pipe ownership prevents double-close and use-after-close.
- Timeout/cancellation semantics are documented and tested.
- Signal support is clearly split into supported host behavior and waived platform behavior.
- Custom handler registration is recorded as `unsupported` with CPython evidence and a revisit rule.
- `getsignal`, `pause`, and `pthread_sigmask` each have terminal inventory states with CPython test evidence.

### milestone_concurrency_runtime_3: Concurrent Futures And Thread Executors

Scope:

- Expand `concurrent.futures`:
  - `Future`
  - `Executor`
  - `ThreadPoolExecutor`
  - `Future.cancel`, cancellation observation, and cancelled terminal state
  - `wait`
  - `as_completed`
  - `CancelledError`, `TimeoutError`, `BrokenExecutor`, executor-specific errors
- Integrate thread execution with existing Sifr task/offload diagnostics:
  - private thread-pool execution is owned by this milestone
  - public `threading.Thread`, locks, conditions, and thread-local APIs are not implemented here
  - CPU-heavy sync work must be routed through approved offload.
  - Blocking sync functions submitted from async contexts require explicit offload semantics.
  - cancellation semantics must not leave unobserved failed work without typed evidence.
- ThreadPoolExecutor compatibility dispositions:
  - `max_workers`, `submit`, `map`, and `shutdown(wait=..., cancel_futures=...)` are in scope.
  - `thread_name_prefix` is accepted only as metadata and is an `intentional-diff` if no public thread naming is exposed.
  - `initializer`/`initargs` are `unsupported` in this phase until Sifr has a formal user-visible sendability contract for cross-thread callable captures.
  - public worker-thread inspection through `threading` objects is `unsupported`.
- Worker error propagation contract:
  - future result observation returns `Result[T, FutureError[E]]` or an equivalent typed sum with at least `Cancelled`, `TimedOut`, `Worker(E)`, and `WorkerRuntime` variants
  - submitted callables returning plain `T` complete futures with `Ok(T)`
  - submitted callables returning `Result[T, E]` complete futures with `Err(FutureError::Worker(E))` or equivalent
  - worker callables must type-check as either `Callable[..., T]` or `Callable[..., Result[T, E]]`; any user-visible fallible path outside those return shapes is a compile-time error
  - runtime failures from foreign/runtime boundaries are represented as typed `FutureError::WorkerRuntime`
  - `Future.cancel` and cancelled futures are represented as `FutureError::Cancelled` or an equivalent typed cancelled terminal state
  - `Future.cancel()` itself returns a typed cancel outcome with `Cancelled`, `AlreadyRunning`, and `AlreadyDone` variants, preserving CPython's false-return cases without hiding state in a boolean
  - `Future.result(timeout=...)` timeout is represented as `FutureError::TimedOut` or an equivalent typed timeout terminal state
  - `shutdown(wait=True, cancel_futures=...)` blocks until already-running futures complete and stores their typed results before returning; `shutdown(wait=False, cancel_futures=...)` rejects new submissions but keeps running task result channels alive so future handles can observe completion after shutdown returns
  - `shutdown(cancel_futures=True)` routes only pending, not-yet-started futures through the same typed cancellation outcome path as `Future.cancel`, producing the `Cancelled` terminal state for each affected future; already-running futures continue to completion and their typed results remain observable through their `Future` handles until those handles are dropped
  - `Executor.map` is adapted to return an ordered iterator of typed item results, `Iterator[Result[T, ExecutorError[E]]]` or equivalent, for homogeneous callable result type `T`; it does not raise during iteration
  - `Executor.map(..., timeout=...)` computes one absolute monotonic deadline at `map()` call time and reports `ExecutorError::Timeout` on the first item that cannot be delivered within the remaining budget; the timeout never resets per item
  - `wait` and `as_completed` accept homogeneous future collections only (`Iterable[Future[T, E]]` or equivalent); heterogeneous future collections require an explicit user-defined sum type and otherwise produce a compile-time diagnostic
  - `as_completed` borrows or clones observation handles for futures; it must not consume the caller's future handles. Its iterator yields typed completed-future results such as `Result[Future[T, E], FutureError[E]]` or equivalent. Futures yielded before timeout are valid and owned by the caller, timeout is emitted as a terminal typed `Err(FutureError::TimedOut)`, the iterator terminates after that timeout error, and pending futures remain live/cancellable through the caller's original handles.
  - `as_completed(..., timeout=...)` computes one absolute monotonic deadline at `as_completed()` call time and reports timeout as a typed terminal `Err(FutureError::TimedOut)` or equivalent when the total deadline expires before all futures complete; it must not silently stop with pending futures or reset the timeout per yielded future
  - `wait` borrows or clones observation handles for futures; it must not consume the caller's future handles. Returned `done` and `not_done` sets are valid independent views over the same future objects, and `not_done` futures remain live/cancellable through the caller's original handles.
  - `wait(..., return_when=FIRST_EXCEPTION)` maps "exception" to the first future whose observed worker result is typed `Err(_)`, including ordinary user `Result::Err` worker outcomes and executor/runtime worker failures; it still returns a full `(done, not_done)` partition containing all futures completed before and including the failing future, plus all remaining futures
  - `not_done` futures returned by `wait(..., return_when=FIRST_EXCEPTION)` remain scheduled on the same executor unless the caller cancels them or shuts down the executor. Subsequent `shutdown(cancel_futures=True)` cancels only not-yet-started futures among them; already-running futures continue to typed completion.
  - if `wait(..., return_when=FIRST_EXCEPTION)` observes no typed worker failure, it behaves like `ALL_COMPLETED` and returns `(all_done, empty_not_done)`
  - unsupported `return_when` values produce diagnostics
  - user-controlled worker failures must never cross the thread boundary as a panic or be swallowed silently

CPython tests to mine:

- `Lib/test/test_concurrent_futures/`

Rust/runtime candidates:

- `rayon` or existing Tokio blocking pool for threads
- `tokio::task::spawn_blocking`

Definition of done:

- Future completion, cancellation, timeout, and callback behavior has CPython-derived fixtures.
- Thread pool shutdown is deterministic and panic-free.
- Executor APIs do not leak raw Tokio or Rust thread handles.
- Any CPython `ThreadPoolExecutor` behavior requiring public `threading` module objects is explicitly adapted or waived.
- `Future.result`/equivalent observation returns typed worker failure evidence and never panics on user-controlled worker errors.
- Future cancellation has a distinct typed terminal state and CPython-derived cancellation fixtures.
- `Future.result(timeout=...)`, `shutdown(cancel_futures=True)`, `Executor.map`, `wait`, and `as_completed` have CPython-derived adapted fixtures covering timeout, cancellation, ordered result iteration, and homogeneous future collection constraints.
- `Future.cancel()` return outcome, `Executor.map(timeout=...)`, `as_completed(timeout=...)`, and `wait(return_when=FIRST_EXCEPTION)` have typed fixtures.

### milestone_concurrency_runtime_4: Typed IPC, Process Pools, And Multiprocessing

Scope:

- Add typed IPC foundation before process pools:
  - M4 owns the typed IPC design, implementation, and acceptance gate; no process-pool or multiprocessing pool item depends on an unnamed external phase
  - supported payload types must be explicit
  - no arbitrary pickle/object transport
  - model class serialization must wait for the relevant model-serialization phase if needed
  - unsupported payloads are compile-time errors where possible
  - no process-backed executor API may ship before this contract has compile-time and runtime tests
  - the design must specify payload ownership, serialization format, versioning, child-process bootstrap, result/error framing, cancellation/termination messages, and panic-free malformed-message handling before `ProcessPoolExecutor` work starts
- Add `ProcessPoolExecutor` only after typed IPC supports:
  - task submission
  - ordered and unordered result streams
  - worker lifecycle
  - cancellation/termination
  - initializer arguments
  - typed error propagation
- Add bounded `multiprocessing` subset:
  - `Process`
  - `Queue`, `SimpleQueue`, `JoinableQueue`
  - `Pipe`
  - `Pool` only after typed IPC supports the same payload and lifecycle contract as `ProcessPoolExecutor`; otherwise `Pool` is `unsupported`
  - spawn start method first
  - fork/forkserver are host-limited and require explicit ownership-safety evidence before adoption
  - `Value`/`Array` are unsupported until typed shared-memory ownership is designed
  - `shared_memory` is unsupported until explicit ownership/unlink/drop rules are proven
  - no shared mutable references may cross a process boundary
  - start-method APIs must record `done`, `intentional-diff`, `unsupported`, or `host-limited` per host

CPython tests to mine:

- `Lib/test/test_concurrent_futures/`
- `Lib/test/_test_multiprocessing.py`
- `Lib/test/test_multiprocessing_main_handling.py`
- `Lib/test/test_multiprocessing_spawn/`
- `Lib/test/test_multiprocessing_fork/`
- `Lib/test/test_multiprocessing_forkserver/`

Rust/runtime candidates:

- typed local serialization such as `bincode` or `postcard` only after typed IPC design approval
- platform process primitives from M2

Definition of done:

- Process pools do not ship until typed IPC is implemented and tested.
- `multiprocessing.Pool` and `ProcessPoolExecutor` share the same typed IPC gate and cannot diverge in payload semantics.
- Multiprocessing APIs are either implemented with typed IPC or explicitly waived; no pickle-equivalent dynamic fallback is allowed.
- Shared-memory and fork-like behavior are blocked unless the execution ledger records the ownership proof and CPython test adaptations.

### milestone_concurrency_runtime_5: Contextlib, Warnings, And Runtime Ergonomics

Scope:

- Add `contextlib`:
  - `closing`, `aclosing`
  - `nullcontext`
  - `suppress`
  - `redirect_stdout`, `redirect_stderr`
  - `chdir`
  - `ExitStack`, `AsyncExitStack`
  - `AbstractContextManager`, `AbstractAsyncContextManager`
  - `contextmanager`, `asynccontextmanager` are `unsupported` in this phase because CPython-compatible generator `send`/`throw`/`close` and async-generator cleanup semantics are outside this phase's scope
  - class-based context-manager APIs are implemented independently, not as fallback implementations for the decorator APIs
- Add `warnings`:
  - `warn`
  - `warn_explicit`
  - `filterwarnings`
  - `simplefilter`
  - `resetwarnings`
  - `catch_warnings`
  - `showwarning`, `formatwarning`
  - warning categories represented as typed classes
- Make warning state concurrency-safe:
  - process-global filters require an explicit `Mutex`/`RwLock`-backed runtime design or equivalent
  - per-module warning registries require a typed module-state representation, not dynamic monkeypatching
  - async tasks must not race warning filter mutation without explicit lock/state rules
  - compile-time diagnostics remain separate from runtime warnings
- Ensure context managers integrate with Sifr's sync and async context protocols.

CPython tests to mine:

- `Lib/test/test_contextlib.py`
- `Lib/test/test_contextlib_async.py`
- `Lib/test/test_warnings/`

Definition of done:

- Sync and async context manager helpers work with Sifr cleanup semantics.
- `ExitStack` and `AsyncExitStack` preserve LIFO cleanup and typed secondary evidence rules.
- Warnings are deterministic, thread-safe, and do not become hidden exceptions.
- Generator-decorator helpers are recorded as `unsupported` with a revisit rule; no partial fake generator path is allowed.

### milestone_concurrency_runtime_6: Integration, Documentation, And Production Gate

Scope:

- Update public docs for every new module and major intentional divergence:
  - `queue`
  - `subprocess`, `asyncio.subprocess`
  - `concurrent.futures`, `multiprocessing`
  - `contextlib`, `warnings`, `signal`
- Update internal architecture docs for:
  - process/queue/executor/runtime boundaries
  - typed IPC and unsupported payload policy
  - async counterpart and blocking/offload policy
  - warning and signal global-state policy
  - host-limited multiprocessing behavior
- Add demos:
  - async subprocess pipeline
  - producer/consumer queues
  - thread pool executor
  - process pool where supported
  - warning capture and context cleanup
- Add generated Cargo dependency snapshots for all new feature combinations.
- Add panic-scan and emitted-code quality checks for queue/process/runtime paths.
- Update validation lane manifests with representative fixtures.
- Close the inventory:
  - every public surface has a terminal state
  - every CPython test family has `adopted`, `adapted`, or `waived` evidence
  - every waiver has a revisit rule and regression fixture
  - every host-limited surface records the supported host matrix
- Run an external review loop on the final inventory and close any blocking finding before phase completion.
- External review owner is the stdlib phase owner plus the designated compiler/runtime reviewer recorded in the execution ledger. If review output is unavailable for five working days after the review artifact is posted, the phase owner may proceed only by recording the attempted review, open questions, and a conservative self-review in the ledger.

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

- Every module surface and CPython test family in the phase inventory is closed as `done`, `intentional-diff`, `unsupported`, or `host-limited`.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in the added stdlib/runtime surfaces.
- Async and sync APIs follow the Phase 32 workload and cancellation model.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md`
- `verification/stdlib/concurrency_runtime_parity_inventory.md`
- `verification/stdlib/concurrency_runtime_parity_inventory.json`
- `verification/stdlib/concurrency_runtime_parity_cpython_test_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test files scanned
- adopted/adapted/waived CPython test families
- final unsupported/intentional-diff/host-limited waiver index

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No backward-compatibility shims, legacy aliases, deprecated behavior, or fallback paths may survive phase exit. Deliberate current-CPython adapters are allowed only when recorded in the inventory with Sifr-safe semantics and tests.
- No direct Tokio/runtime types may leak into public Sifr APIs.
- No arbitrary pickle-equivalent process-pool transport may be introduced.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added blocking sync function must be classified in the stdlib workload database.
- Every added async function must have a real suspension summary.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Every module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.

## Open Planning Questions To Resolve In `milestone_concurrency_runtime_0`

1. What is the typed IPC serialization contract for `ProcessPoolExecutor` and `multiprocessing`?
2. Which signal APIs are safe and deterministic across the supported host matrix?
3. Which multiprocessing start methods are supported, host-limited, or unsupported?
4. Which executor dependency strategy meets binary-size, safety, cancellation, and maintenance goals?
5. How do warning filters and per-module warning registries interact with threads and async tasks?
6. Which subprocess text-mode APIs remain blocked until text/i18n `milestone_text_i18n_1` closes?

These questions must be answered in the phase execution ledger before implementing the affected milestone.
