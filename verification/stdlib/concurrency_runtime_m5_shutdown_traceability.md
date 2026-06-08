# Concurrency Runtime M5 Shutdown Traceability

Milestone: `milestone_concurrency_runtime_5`

Status: In progress; signal value-model foundation started with portable `sifr.signal.Signal`, `sigint()`, and `sigterm()` evidence. Structured signal constants, signal streams, deterministic cleanup scopes, explicit task context propagation, and structured runtime diagnostics remain M5 work.

## Production Surface Traceability

| Surface | M5 evidence | Notes |
| --- | --- | --- |
| `sifr.signal.Signal` | `signal_value_model_basic` | `Signal` is a Sifr-owned value with `name` and `number` fields. It is not CPython's `Signals` enum and does not expose raw host signal-handler mutation. |
| `sifr.signal.SignalError` | foundation symbol | Reserved for later structured shutdown stream and diagnostic surfaces; this wave does not raise it. |
| `sifr.signal.sigint()`, `sifr.signal.sigterm()` | `signal_value_model_basic` | The first accepted signal values are portable shutdown targets. Importable module-level constants remain follow-up work because current stdlib export collection does not expose object-valued constants. Unix-only values such as `SIGHUP` remain host-limited until a deterministic host row and fixture are added. |
| `sifr.signal.SIGINT`, `sifr.signal.SIGTERM` | planned M5 follow-up | True module-level signal constants require object-valued stdlib constant export support or another explicit enum-like representation before they can be exposed honestly. |
| `sifr.signal.ctrl_c`, `sifr.signal.terminate`, `sifr.signal.shutdown_stream`, `sifr.signal.strsignal` | planned M5 follow-up | These structured shutdown APIs are not exposed by the foundation wave. They must use Tokio signal APIs or a documented host-limited design and must not leak Tokio handles. |
| `sifr.signal.pause`, `signal.signal`, `getsignal`, `raise_signal`, `pthread_sigmask` | `signal_pause_unsupported`; `signal_handler_registration_unsupported`; `signal_getsignal_unsupported`; `signal_raise_signal_unsupported`; `signal_pthread_sigmask_host_limited` | Unsafe arbitrary handler registration, process-global signal wakeup, and mask mutation are not production APIs. Current evidence pins stable missing-member diagnostics on `sifr.signal` imports until a future explicitly supported surface is designed. |
| `sifr.resource.ExitStack`, `sifr.resource.AsyncExitStack`, `closing`, `aclosing`, `nullcontext` | planned M5 follow-up | Deterministic cleanup scopes must report cleanup failures under cancellation without hiding the initiating failure. |
| `sifr.task.Context`, `sifr.task.ContextKey[T]` | planned M5 follow-up | M1 reserved `ctx=None` call shapes. M5 must implement explicit opt-in propagation without Python `contextvars` implicit dynamic mutation. |
| Python global `warnings` filter model | M0/M0a negative import fixtures; planned M5 warning-global rejection fixture | Python warning filters remain rejected. Runtime warning-style events, if needed, must be structured diagnostics or tracing events. |

## Signal Host Matrix

| Signal surface | macOS arm64 | Linux x86_64 | Windows x86_64 | Notes |
| --- | --- | --- | --- | --- |
| `sigint()` value | supported | supported | supported | Portable interrupt shutdown target; fixture validates the Sifr value model. |
| `sigterm()` value | supported | supported | supported | Portable termination shutdown target; fixture validates the Sifr value model. |
| `SIGINT` / `SIGTERM` module constants | planned | planned | planned | Await object-valued stdlib constant export support or another explicit enum-like representation. |
| Unix-only constants such as `SIGHUP` | host-limited | host-limited | host-limited | Not exposed until the exact host contract and deterministic fixture are recorded. |
| `ctrl_c` stream | planned | planned | planned | Must be fixture-backed before supported. |
| `terminate` stream | planned | planned | host-limited | Non-Unix termination semantics require host-specific evidence before support. |
| Arbitrary handler registration / signal masks | unsupported-with-diagnostic | unsupported-with-diagnostic | host-limited | Not a safe production API in this phase. |

## Validation Coverage

| Lane | Representative entries |
| --- | --- |
| Create PR | `signal_value_model_basic` |
| Merge | `signal_value_model_basic` |
| Fail suite | `signal_pause_unsupported`, `signal_handler_registration_unsupported`, `signal_getsignal_unsupported`, `signal_raise_signal_unsupported`, `signal_pthread_sigmask_host_limited`; existing `bare_cpython_signal_import`; existing `bare_cpython_contextlib_import`; existing `bare_cpython_warnings_import`; existing `legacy_sifr_contextlib_removed`; existing `legacy_sifr_warnings_removed` |

## Follow-up Boundaries

- `sigint()` and `sigterm()` are value-model evidence only; they do not claim stream delivery or host signal subscription.
- Importable `SIGINT` and `SIGTERM` constants remain M5 follow-up work; this foundation does not depend on private stdlib export behavior that currently only handles importable object classes/functions and integer constants.
- Unsupported signal APIs are intentionally absent from `sifr.signal` so static imports produce stable diagnostics instead of runtime surprises.
- `pthread_sigmask` is grouped with unsupported signal APIs in the current fixture set because no safe mask-mutation surface exists; it remains host-limited for any future explicitly designed Unix-only API.
- Cleanup stacks, task context propagation, diagnostics/tracing, and warning-global rejection remain separate M5 waves.
