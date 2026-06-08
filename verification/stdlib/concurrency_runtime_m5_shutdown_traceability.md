# Concurrency Runtime M5 Shutdown Traceability

Milestone: `milestone_concurrency_runtime_5`

Status: In progress; signal value-model foundation covers portable `sifr.signal.Signal`, `sigint()`, `sigterm()`, `SIGINT`, `SIGTERM`, and `strsignal(signal)` evidence, deterministic cleanup helper coverage includes no-value and value-carrying generic `sifr.resource.nullcontext(...)`, and task context value types are importable through `sifr.task.Context` / `ContextKey[T]`. Structured signal streams, cleanup stacks, explicit task context propagation, and structured runtime diagnostics remain M5 work.

## Production Surface Traceability

| Surface | M5 evidence | Notes |
| --- | --- | --- |
| `sifr.signal.Signal` | `signal_value_model_basic` | `Signal` is a Sifr-owned value with `name` and `number` fields. It is not CPython's `Signals` enum and does not expose raw host signal-handler mutation. |
| `sifr.signal.SignalError` | foundation symbol | Reserved for later structured shutdown stream and diagnostic surfaces; this wave does not raise it. |
| `sifr.signal.sigint()`, `sifr.signal.sigterm()` | `signal_value_model_basic` | The first accepted signal factory helpers are portable shutdown targets. Unix-only values such as `SIGHUP` remain host-limited until a deterministic host row and fixture are added. |
| `sifr.signal.strsignal(signal)` | `signal_strsignal_basic` | Pure Sifr value helper returning the owned signal name. It does not consult process-global host signal state and does not claim stream delivery. |
| `sifr.signal.SIGINT`, `sifr.signal.SIGTERM` | `signal_constants_basic` | Portable module-level constants are importable Sifr-owned `Signal` values. They follow normal ownership rules and do not claim signal-stream delivery or process-global handler state. |
| `sifr.signal.ctrl_c`, `sifr.signal.terminate`, `sifr.signal.shutdown_stream` | planned M5 follow-up | These structured shutdown APIs are not exposed by the value-helper wave. They must use Tokio signal APIs or a documented host-limited design and must not leak Tokio handles. |
| `sifr.signal.pause`, `signal.signal`, `getsignal`, `raise_signal`, `pthread_sigmask` | `signal_pause_unsupported`; `signal_handler_registration_unsupported`; `signal_getsignal_unsupported`; `signal_raise_signal_unsupported`; `signal_pthread_sigmask_host_limited` | Unsafe arbitrary handler registration, process-global signal wakeup, and mask mutation are not production APIs. Current evidence pins stable missing-member diagnostics on `sifr.signal` imports until a future explicitly supported surface is designed. |
| `sifr.resource.NullContext[T]`, `sifr.resource.nullcontext(...)` | `resource_nullcontext_basic` | `nullcontext()` remains a no-op helper whose entered value is `None`; `nullcontext(value)` now preserves the carried value type through the synchronous `with` protocol. Generated context-manager guards preserve generic type arguments for this value model. |
| `sifr.resource.ExitStack`, `sifr.resource.AsyncExitStack`, `closing`, `aclosing` | planned M5 follow-up | Deterministic cleanup stacks must report cleanup failures under cancellation without hiding the initiating failure. `closing` and `aclosing` require an explicit owned-close protocol before support. |
| Python `contextlib` convenience helpers: `redirect_stdout`, `redirect_stderr`, `chdir`, `suppress`, `contextmanager`, `asynccontextmanager` | `resource_redirect_stdout_unsupported`; `resource_redirect_stderr_unsupported`; `resource_chdir_unsupported`; `resource_suppress_unsupported`; `resource_contextmanager_unsupported`; `resource_asynccontextmanager_unsupported` | These CPython-shaped helpers are not production APIs in this phase. Generator decorator helpers require a future generator-semantics design, and process-global stdout/stderr/cwd mutation is rejected for production concurrent code. |
| `sifr.task.Context`, `sifr.task.ContextKey[T]`, `sifr.task.empty_context()` | `task_context_value_model_basic` | Importable Sifr-owned value-model foundation for future explicit propagation. `ContextKey[T]` carries a typed default marker so the key's value type is preserved without dynamic Python `contextvars` behavior. |
| `task.TaskGroup(ctx=ctx)`, `task.spawn_scoped(..., ctx=ctx)` propagation | `task_context_propagation_rejected`; existing M1 lowering unit tests | Non-`None` context propagation is still rejected until propagation semantics are implemented. This prevents a fake context path while keeping the reserved call shape stable. |
| Python global `warnings` filter model | `warnings_filter_global_rejected`; M0/M0a negative import fixtures | Python warning filters remain rejected. Runtime warning-style events, if needed, must be structured diagnostics or tracing events. This closes the global-filter parity surface without introducing a `warnings` adapter. |

## Signal Host Matrix

| Signal surface | macOS arm64 | Linux x86_64 | Windows x86_64 | Notes |
| --- | --- | --- | --- | --- |
| `sigint()` value | supported | supported | supported | Portable interrupt shutdown target; fixture validates the Sifr value model. |
| `sigterm()` value | supported | supported | supported | Portable termination shutdown target; fixture validates the Sifr value model. |
| `strsignal(signal)` | supported | supported | supported | Host-independent value helper returning the Sifr-owned `Signal.name`; no signal delivery behavior. |
| `SIGINT` / `SIGTERM` module constants | supported | supported | supported | Host-independent Sifr-owned values validated by `signal_constants_basic`; no signal delivery behavior. |
| Unix-only constants such as `SIGHUP` | host-limited | host-limited | host-limited | Not exposed until the exact host contract and deterministic fixture are recorded. |
| `ctrl_c` stream | planned | planned | planned | Must be fixture-backed before supported. |
| `terminate` stream | planned | planned | host-limited | Non-Unix termination semantics require host-specific evidence before support. |
| Arbitrary handler registration / signal masks | unsupported-with-diagnostic | unsupported-with-diagnostic | host-limited | Not a safe production API in this phase. |
| `nullcontext()` / `nullcontext(value)` | supported | supported | supported | Host-independent Sifr `with` protocol helper; no platform cleanup behavior. The value-carrying form preserves the entered value type in generated generic context-manager guards. |
| `Context` / `ContextKey[T]` value model | supported | supported | supported | Host-independent Sifr-owned value types for future explicit propagation; no task-local dynamic state. |
| Explicit task context propagation | planned | planned | planned | Non-`None` `ctx` values remain rejected until propagation semantics and handoff rules are implemented. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `signal_value_model_basic`; `signal_strsignal_basic`; `signal_constants_basic`; `task_context_value_model_basic`; `resource_nullcontext_basic` |
| Merge | `signal_value_model_basic`; `signal_strsignal_basic`; `signal_constants_basic`; `task_context_value_model_basic`; `resource_nullcontext_basic` |
| Fail suite | `signal_pause_unsupported`, `signal_handler_registration_unsupported`, `signal_getsignal_unsupported`, `signal_raise_signal_unsupported`, `signal_pthread_sigmask_host_limited`; `task_context_propagation_rejected`; `warnings_filter_global_rejected`; `resource_redirect_stdout_unsupported`, `resource_redirect_stderr_unsupported`, `resource_chdir_unsupported`, `resource_suppress_unsupported`, `resource_contextmanager_unsupported`, `resource_asynccontextmanager_unsupported`; existing `bare_cpython_signal_import`; existing `bare_cpython_contextlib_import`; existing `bare_cpython_warnings_import`; existing `legacy_sifr_contextlib_removed`; existing `legacy_sifr_warnings_removed` |

## Follow-up Boundaries

- `sigint()`, `sigterm()`, `SIGINT`, `SIGTERM`, and `strsignal(signal)` are value-model evidence only; they do not claim stream delivery or host signal subscription.
- Unsupported signal APIs are intentionally absent from `sifr.signal` so static imports produce stable diagnostics instead of runtime surprises.
- `pthread_sigmask` is grouped with unsupported signal APIs in the current fixture set because no safe mask-mutation surface exists; it remains host-limited for any future explicitly designed Unix-only API.
- `nullcontext()` is supported as a no-op cleanup helper and `nullcontext(value)` preserves the entered value type; cleanup stacks, owned closing helpers, task context propagation, and structured diagnostics/tracing remain separate M5 waves.
- `Context`, `ContextKey[T]`, and `empty_context()` are value-model evidence only; non-`None` task propagation remains rejected until explicit propagation rules are implemented.
