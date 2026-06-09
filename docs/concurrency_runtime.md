# Concurrency And Runtime

Sifr's production concurrency model uses native `sifr.*` modules instead of CPython compatibility veneers. The public surface is structured, typed, cancellation-aware, and compiled to Rust without exposing Tokio, Rayon, OS process handles, or tracing/metrics types in Sifr source.

Use explicit imports from the accepted modules:

```python
from sifr.task import TaskGroup, sleep, timeout
from sifr.sync import channel
from sifr.runtime import spawn_cpu
from sifr.parallel import map
from sifr.process import Command
from sifr.signal import shutdown_stream
from sifr.resource import nullcontext
from sifr.ipc import require_serializable
```

Bare CPython concurrency imports such as `asyncio`, `queue`, `threading`, `subprocess`, `concurrent.futures`, `multiprocessing`, `signal`, `contextlib`, and `warnings` are not aliases for Sifr production APIs. Use the native `sifr.*` modules directly.

## `sifr.task`

`sifr.task` is the public async work model. Tasks are scoped and owned: a child task must be spawned under a live `TaskGroup`, observed exactly once, and prevented from escaping its owner.

Core surfaces:

- `TaskHandle[T, E]`: affine observation handle for a child task result.
- `TaskGroup[E]`: structured owner for homogeneous child error type `E`.
- `spawn_scoped(...)`: spawn under the active structured owner.
- `sleep`, `timeout`, `deadline`, and `cancel_scope`: suspension and cancellation helpers.
- `join_all`, `race`, and `select`: structured multi-task observation helpers with loser-cancellation evidence.
- `Context`, `ContextKey[T]`, `empty_context()`, and `current_context()`: explicit task context values.

```python
from sifr.task import TaskGroup, sleep

async def worker(value: int) -> int:
    await sleep(0.01)
    return value + 1

async def main() -> int:
    async with TaskGroup[Error]() as group:
        handle = group.spawn(worker(41))
        return await handle
```

Task context is explicit and immutable at the task boundary. Sifr does not provide Python `contextvars` dynamic copy-on-spawn behavior in this phase.

Unsupported CPython-shaped surfaces:

- event loops, loop policies, transports, protocols, and callbacks,
- detached unowned tasks,
- ambient mutable task-local state,
- coroutine objects used without `await`.

## `sifr.sync`

`sifr.sync` owns same-process coordination. The primary data path is channels with bounded backpressure and deterministic close/drain behavior.

Core surfaces:

- `Channel[T]`, `ChannelSender[T]`, and `ChannelReceiver[T]`.
- `channel[T]()` and `bounded_channel[T](capacity)`.
- `Lock`, `RwLock`, async lock forms, semaphores, and events where fixtures pin the current accepted behavior.
- `ClosedError` for closed-and-drained channel observation.

```python
from sifr.sync import bounded_channel

async def pipeline() -> int:
    sender, receiver = bounded_channel[int](2)
    await sender.send(1)
    await sender.send(2)
    sender.close()

    total = 0
    async for item in receiver:
        total += item
    return total
```

Sendability and shareability are compile-time ownership facts. Non-send values, process handles, task handles, and synchronization endpoints cannot cross task, channel, IPC, or worker boundaries unless an accepted wrapper proves the boundary safe.

Unsupported CPython-shaped surfaces:

- `queue.Queue` task accounting such as `task_done()` and `join()`,
- raw `threading.Thread`,
- implicit global locks or condition variables that hide predicate discipline.

## `sifr.runtime`

`sifr.runtime` exposes structured offload and runtime diagnostics without exposing runtime implementation types.

Core surfaces:

- `spawn_blocking(...)`: run a known blocking-I/O synchronous worker outside the async runtime worker.
- `spawn_cpu(...)`: run a known CPU-heavy synchronous worker under typed worker error evidence.
- `JoinSet[T, E]`: homogeneous dynamically-growable task/offload collection.
- `DiagnosticLevel`, `DiagnosticEvent`, `DiagnosticError`, `diagnostic_event(...)`, and `emit_diagnostic(...)`.

Blocking and CPU-heavy annotations are declaration-site workload facts. Sifr rejects direct calls to known blocking or CPU-heavy functions from async code unless the call goes through an accepted async API, `spawn_blocking`, `spawn_cpu`, or `sifr.parallel`.

```python
from sifr.runtime import spawn_cpu

@cpu_heavy
def checksum(data: bytes) -> int:
    total = 0
    for byte in data:
        total += int(byte)
    return total

async def main(data: bytes) -> int:
    handle = spawn_cpu(checksum, data)
    return await handle
```

Runtime diagnostics lower to fixed-schema tracing events and metrics counters when used. Sifr does not install a process-global subscriber, recorder, exporter, or Python warning filter.

## `sifr.parallel`

`sifr.parallel` is the synchronous CPU parallelism API. It uses private Rayon pools and never configures Rayon's global pool.

Core surfaces:

- `map(worker, items)`: ordered parallel map.
- `try_map(worker, items)`: ordered parallel map with typed user errors.
- `Pool` and `PoolConfig`: configured private pools.

```python
from sifr.parallel import map

@cpu_heavy
def square(value: int) -> int:
    return value * value

def main() -> list[int]:
    return map(square, [1, 2, 3, 4])
```

Worker panics are caught at the generated boundary and converted into typed worker runtime errors. Direct `sifr.parallel` calls from async code are rejected; use `sifr.runtime.spawn_cpu` or a scoped offload form instead.

## `sifr.process`

`sifr.process` is the production process substrate. It owns child process creation, supervision, status observation, owned pipes, timeout behavior, cancellation, and shell execution effects.

Core surfaces:

- `Command`: typed command builder.
- `Child`: owned child process and pipe access.
- `ProcessHandle`: scoped child supervision handle.
- Sync and async run/output/wait/spawn forms.
- Owned pipe readers and writers.

```python
from sifr.process import Command

def main() -> str:
    result = Command("printf").arg("hello").output_text(encoding="utf-8")
    return result.stdout
```

Text process output requires explicit encoding. Shell execution is an explicit effect; Sifr does not silently route command strings through a shell. Timeouts and cancellation map to typed process evidence, and generated code must clean up process groups where the accepted host contract requires it.

Unsupported CPython-shaped surfaces:

- bare `subprocess` imports,
- unstructured process handles crossing task/offload/channel boundaries,
- public process pools in this phase.

## `sifr.signal`

`sifr.signal` is structured shutdown signaling. Portable value helpers are host-independent; actual signal delivery is host-limited where the host contract requires it.

Core surfaces:

- `Signal`, `SignalError`.
- `sigint()`, `sigterm()`, `SIGINT`, `SIGTERM`.
- `strsignal(signal)`.
- `ctrl_c()`, `terminate()`, and `shutdown_stream().next()` for structured shutdown waits.

```python
from sifr.signal import shutdown_stream

async def main() -> str:
    signal = await shutdown_stream().next()
    return signal.name
```

Unix Ctrl-C and SIGTERM delivery are covered by deterministic fixture evidence. Non-Unix delivery remains host-limited until a deterministic host runner can deliver real console-control events.

Unsupported CPython-shaped surfaces:

- arbitrary `signal.signal(...)` handlers,
- process-global warning/signal mutation,
- raw signal masks unless a future host-specific API is explicitly designed.

## `sifr.resource`

`sifr.resource` owns small deterministic resource helpers. Language-level `try/finally` cleanup already runs under task cancellation.

Core supported surface:

- `nullcontext()`
- `nullcontext(value)`

```python
from sifr.resource import nullcontext

def main() -> int:
    with nullcontext(3) as value:
        return value
```

Unsupported in this phase:

- `ExitStack`
- `AsyncExitStack`
- `closing`
- `aclosing`
- `contextmanager`
- `asynccontextmanager`
- `redirect_stdout`, `redirect_stderr`, `chdir`, and `suppress`

Cleanup stacks and owned closing helpers need future typed cleanup-error aggregation and owned-close protocols before they can be production APIs.

## `sifr.ipc`

`sifr.ipc` is the typed IPC substrate for future Sifr-native process workers. It is not a public process pool and not a pickle-compatible multiprocessing adapter.

Core surfaces and evidence:

- `SchemaId`, `ProtocolVersion`, `FrameKind`, and `BackpressurePolicy` value model.
- Length-prefixed Postcard frame helpers inside `sifr_stdlib`.
- Request tracking, bounded in-flight backpressure, cancellation, shutdown, and malformed-frame evidence.
- Bootstrap and connection-state negotiation over an accepted process-pipe transport.
- `require_serializable(value)`: compiler-erased marker for representative compile-time payload eligibility diagnostics.

```python
from sifr.ipc import require_serializable

class Message:
    value: int

def main(message: Message) -> None:
    require_serializable(message)
```

Accepted payload families include concrete scalar/container/record shapes proven by the compiler. Process handles, task handles, synchronization endpoints, callables, raw host resources, and arbitrary object graphs are rejected as IPC payloads.

Public worker pools and generated worker integration are `deferred-to-phase-X`. Windows process-pipe fixture evidence remains host-limited until a deterministic Windows fixture is accepted.

Unsupported CPython-shaped surfaces:

- `multiprocessing.Process`
- `multiprocessing.Queue`
- `multiprocessing.Pipe`
- `multiprocessing.Pool`
- `concurrent.futures.ProcessPoolExecutor`
- `fork`, `forkserver`, and `shared_memory` under a CPython-shaped namespace

## Intentional Divergence Index

| CPython-shaped surface | Sifr production replacement | Phase state |
| --- | --- | --- |
| `asyncio` | `sifr.task` and `sifr.sync` | unsupported-with-diagnostic |
| `queue` | `sifr.sync` channels | unsupported-with-diagnostic |
| `threading` | `sifr.sync`, `sifr.runtime`, `sifr.parallel` | unsupported-with-diagnostic |
| `subprocess` | `sifr.process` | unsupported-with-diagnostic |
| `concurrent.futures` | `sifr.runtime` and `sifr.parallel` | unsupported-with-diagnostic |
| `multiprocessing` | `sifr.process` plus `sifr.ipc` substrate | rejected / deferred worker API |
| Python `signal` globals | `sifr.signal` structured shutdown | unsupported-with-diagnostic or host-limited |
| Python `contextlib` helpers | `sifr.resource` plus language cleanup | unsupported-with-diagnostic except `nullcontext` |
| Python `warnings` globals | `sifr.runtime` diagnostics/tracing | rejected |

Future compatibility adapters must be reviewed separately. They must wrap the native Sifr substrate without process-global mutation, hidden unstructured ownership, or panic-prone generated runtime paths.
