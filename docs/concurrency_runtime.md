# Concurrency Runtime

Sifr's concurrency runtime is a native structured-concurrency substrate, not a CPython event-loop or multiprocessing clone. The accepted public modules are:

- `sifr.task`
- `sifr.sync`
- `sifr.runtime`
- `sifr.parallel`
- `sifr.process`
- `sifr.signal`
- `sifr.resource`
- `sifr.ipc`

The common rule across these modules is that concurrency boundaries are statically typed and ownership checked. Values that cross task, thread, process, or IPC boundaries must be owned and must satisfy the boundary's sendability, shareability, or `IpcSerializable` requirements. Unsupported CPython-shaped APIs are rejected with diagnostics instead of being silently emulated.

## `sifr.task`

`sifr.task` is the structured task and task-context surface.

Public value APIs:

- `Context`
- `ContextKey[T]`
- `empty_context() -> Context`
- `current_context() -> Context`

Compiler-recognized task APIs are lowered as structured runtime operations:

- `task.scope(...)`
- `TaskGroup`
- `TaskHandle[T, E]`
- scoped spawn
- timeout and deadline helpers
- task cancellation helpers
- `join`, `race`, and `select`

Task handles are linear ownership values. Awaiting or joining a handle consumes the handle; using it again is a compile-time ownership diagnostic. Scoped spawn requires an active owner, and values captured by a spawned task must be sendable across the task boundary. Lock guards, semaphore permits, borrowed values that could escape the lexical scope, process handles, and other non-send resources are rejected at the boundary.

`Context` and `ContextKey[T]` are Sifr-owned task context values. Explicit context propagation is supported through the accepted task APIs; implicit CPython `contextvars` behavior is not exposed as global mutable task state.

Intentional divergences:

- No public CPython `asyncio` event-loop object is exposed.
- Event-loop policy, loop replacement, callback scheduling, and global task registries are unsupported.
- Detached fire-and-forget tasks are not accepted; work belongs to a scope or an explicit owner.

## `sifr.sync`

`sifr.sync` provides same-process synchronization and communication primitives.

Public APIs:

- `Shared[T]`
- `channel[T]() -> tuple[ChannelSender[T], ChannelReceiver[T]]`
- `bounded_channel[T](capacity: int) -> tuple[ChannelSender[T], ChannelReceiver[T]]`
- `ChannelSender[T].send(...)`
- `ChannelSender[T].close()`
- `ChannelReceiver[T].receive()`
- `Lock[T]`, `LockGuard[T]`
- `RwLock[T]`, `RwLockReadGuard[T]`, `RwLockWriteGuard[T]`
- `Semaphore`, `SemaphorePermit`
- `Notify`
- `WouldBlockError`
- `ClosedError`

Channels are typed. Sending transfers ownership of the value to the channel. Receiving transfers ownership back to the receiver. Closing a sender makes the close state explicit; closed receives and sends return typed `ClosedError` evidence. Bounded channels are the accepted backpressure surface.

Locks, read/write locks, semaphores, and notifications are same-process primitives. Guards and permits are scoped resources and may not cross task, thread, process, or IPC boundaries. Holding such a guard across an await or returning it from an invalid scope is rejected by ownership diagnostics.

Intentional divergences:

- CPython `queue.Queue`, `asyncio.Queue`, `threading.Lock`, and multiprocessing queue/pipe names are not aliases for `sifr.sync`.
- Same-process channels are not process IPC. Use `sifr.process` for raw subprocess I/O and `sifr.ipc` for typed future worker protocols.

## `sifr.runtime`

`sifr.runtime` exposes structured diagnostic events for runtime surfaces.

Public APIs:

- `DiagnosticLevel`
- `DiagnosticEvent`
- `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`
- `diagnostic_event(...)`
- `emit_diagnostic(event)`
- `DiagnosticError`

Runtime diagnostic events carry a level, target, name, and message. They are intended for structured runtime observability around task, sync, process, signal, offload, and IPC surfaces. Diagnostic emission returns `Result[None, DiagnosticError]`; it is not an exception-style logging side channel.

Intentional divergences:

- Runtime diagnostics are not CPython `warnings` or `logging` global handler mutation.
- Payload bytes, process command lines, environment values, and decoded IPC payloads must not be used as metric labels or diagnostic messages unless an explicit redaction rule covers that field.

## `sifr.parallel`

`sifr.parallel` is the CPU parallel map surface backed by native worker pools.

Public APIs:

- `map(items, func)`
- `try_map(items, func)`
- `PoolConfig`
- `Pool`
- `Pool.map(...)`
- `Pool.try_map(...)`
- `WorkerRuntimeError`
- `WorkerError`

`map` preserves output order for successful items. `try_map` returns a typed worker error when an item fails. Captured values and return values must satisfy worker-boundary ownership and sendability requirements. Worker panics are converted to typed worker errors instead of surfacing as user-triggerable runtime panics.

Blocking or CPU-heavy work should be explicit. Directly calling blocking helpers from async code is rejected by diagnostics; use accepted blocking/offload surfaces so the runtime can isolate the work.

Intentional divergences:

- `concurrent.futures.ThreadPoolExecutor` and `ProcessPoolExecutor` are not public aliases.
- Public process worker pools are deferred to a future phase over `sifr.ipc`; M7 does not ship a public process-worker API.

## `sifr.process`

`sifr.process` is the native subprocess surface.

Public APIs include:

- `run(...)`
- `run_timeout(...)`
- `output(...)`
- `output_text(...)`
- `output_timeout(...)`
- `spawn(...)`
- `run_shell(...)`
- `output_shell(...)`
- `output_shell_text(...)`
- `output_shell_timeout(...)`
- async variants such as `async_run`, `async_spawn`, `async_wait`, `async_output`, `async_run_shell`, and `async_output_shell`
- `Command`
- `Child`
- `AsyncChild`
- `ProcessHandle`
- `PipeReader`
- `PipeWriter`
- `AsyncPipeReader`
- `AsyncPipeWriter`
- `Status`
- `Output`
- `Stdio`, `PIPE`, `INHERIT`, `NULL`
- `ProcessError`

Process handles and pipes are owned resources. Waiting on a child consumes the wait capability; duplicate wait or use-after-close is rejected or returns typed `ProcessError` evidence. Pipe readers and writers must be closed or consumed through the accepted APIs. Timeout, cancellation, `kill`, and `terminate` return structured status or error evidence.

`output_text` and shell text helpers use explicit text behavior recorded by the platform contract. Raw byte output remains available through `Output.stdout` and `Output.stderr`.

Intentional divergences:

- `subprocess.Popen` is not a direct public alias.
- Shell helpers are explicit effectful APIs; they are not the default command execution path.
- Process groups, descendant supervision, and host-specific signal semantics are supported only where recorded by the host matrix.

## `sifr.signal`

`sifr.signal` provides structured shutdown and signal values.

Public APIs:

- `Signal`
- `SIGINT`, `SIGTERM`
- `sigint()`
- `sigterm()`
- `strsignal(signal)`
- `ctrl_c()`
- `terminate()`
- `shutdown_stream()`
- `ShutdownStream.next()`
- `SignalError`

Portable signal values and `strsignal` are host-independent. `ctrl_c` and `shutdown_stream` expose structured awaitable signal evidence. Unix SIGTERM delivery is host-limited and recorded in the supported-host matrix; non-Unix SIGTERM behavior returns typed unsupported evidence where the host cannot provide equivalent semantics.

Intentional divergences:

- Global signal handler mutation is rejected.
- CPython `signal.signal`, `set_wakeup_fd`, and arbitrary handler registration are not public Sifr runtime APIs.
- Signal delivery behavior must be represented as typed values or errors, not process-global mutation.

## `sifr.resource`

`sifr.resource` contains deterministic cleanup helpers.

Public APIs:

- `NullContext[T]`
- `nullcontext[T](value=None)`

`nullcontext` is an owned value context manager. Language-level cleanup under cancellation is part of the structured runtime contract: cleanup runs before timeout or cancellation evidence is observed.

Intentional divergences:

- `ExitStack`, `AsyncExitStack`, `closing`, and `aclosing` are unsupported until cleanup-error aggregation and owned close protocols are accepted.
- Cleanup helpers do not provide a hidden dynamic stack of arbitrary callbacks.

## `sifr.ipc`

`sifr.ipc` is the typed IPC substrate for future Sifr-native process workers.

Public value APIs:

- `SchemaId`
- `schema_id(name, version, hash)`
- `ProtocolVersion`
- `protocol_version(minimum, maximum)`
- `FrameKind`
- frame-family constants such as `HELLO`, `READY`, `RUN`, `COMPLETED`, `FAILED`, `CANCEL`, `SHUTDOWN`, and `TERMINATING`
- `BackpressurePolicy`
- `default_backpressure()`
- `schemas_match(left, right)`
- `require_serializable(value)`
- `IpcError`

The runtime substrate uses deterministic schema identity, protocol-version negotiation, length-prefixed Postcard frames, bounded request tracking, explicit close/cancel frames, and typed malformed-frame evidence. The compiler can extract accepted payload schema shapes and reject unsupported process-local resources, sync endpoints, task handles, functions, borrowed values, and other non-serializable values.

`require_serializable(...)` is a compiler-erased marker used to force representative payload eligibility diagnostics at compile time. It is not a runtime encoder and does not make unsupported values serializable.

Intentional divergences:

- CPython `multiprocessing.Queue`, `Pipe`, `Pool`, `Process`, `fork`, `forkserver`, and `shared_memory` names under `sifr.ipc` are rejected or unsupported with diagnostics.
- M7 does not ship a public process-worker pool or public `ipc.Connection` API.
- Windows process-pipe typed IPC fixture support remains host-limited until a deterministic Windows fixture is accepted.

## Validation And Host Support

The supported-host matrix records whether a surface is host-independent, Unix/macOS/Linux supported, Windows supported, or host-limited. Public documentation should be read with that matrix for platform-specific process and signal behavior.

Representative validation lives in:

- `verification/stdlib/concurrency_runtime_m1_traceability.md`
- `verification/stdlib/concurrency_runtime_m2_sync_traceability.md`
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md`
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md`
- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md`
- `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md`
- `verification/platform/supported_host_matrix.md`
