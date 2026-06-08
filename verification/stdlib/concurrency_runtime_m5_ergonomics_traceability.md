# Concurrency Runtime M5 Ergonomics Traceability

Milestone: `milestone_concurrency_runtime_5`

Status: In progress; signal/shutdown foundation wave adds the first production `sifr.signal` surface and keeps cleanup scopes, task/request context, and structured diagnostics pending.

## Production Surface Traceability

| Surface | M5 evidence | Notes |
| --- | --- | --- |
| `sifr.signal.Signal` | `signal_constants_strsignal`; `lowers_signal_intrinsics_via_registry` | Native signal value object with numeric signal number, stable name, and explicit support flag. M5 first wave exposes constructor functions `sigint()` and `sigterm()` rather than module constants because embedded stdlib constant re-export remains a separate module-export limitation. `sigterm().supported` is backed by a target cfg probe so non-Unix hosts do not overclaim SIGTERM support. |
| `sifr.signal.strsignal` | `signal_constants_strsignal` | Maps supported M5 signal values to stable user-facing descriptions and returns typed `SignalError` for unsupported signal numbers. This is a narrow Sifr-native mapping, not CPython global signal state. |
| `sifr.signal.ctrl_c` | `lowers_signal_intrinsics_via_registry` | Returns an awaitable `Result[Signal, SignalError]` backed by `tokio::signal::ctrl_c()`. Tokio remains internal and generated projects enable the `signal` feature only through the stdlib dependency feature path. Runtime delivery requires an external Ctrl-C event, so this wave pins lowering rather than blocking e2e execution on a real OS signal. |
| `sifr.signal.terminate` | `lowers_signal_intrinsics_via_registry` | Returns an awaitable `Result[Signal, SignalError]`. On Unix it waits for `SIGTERM` through `tokio::signal::unix::SignalKind::terminate`; on non-Unix hosts it returns a typed unsupported `SignalError`. |
| `sifr.signal.shutdown_stream` / `ShutdownStream.next` | `lowers_signal_intrinsics_via_registry` | `shutdown_stream().next()` waits for Ctrl-C or Unix SIGTERM and resolves to a typed `Signal`. On non-Unix hosts the stream waits for Ctrl-C only, keeping SIGTERM host-limited rather than overclaiming parity. |
| CPython handler/control functions | `signal_handler_registration_unsupported`; `signal_pause_unsupported`; `signal_getsignal_unsupported`; `signal_raise_signal_unsupported`; `signal_pthread_sigmask_unsupported` | Arbitrary `signal.signal` handlers, `pause`, `getsignal`, `raise_signal`, and `pthread_sigmask` are absent from `sifr.signal` and receive missing-member diagnostics. Production shutdown uses structured signal streams instead of Python-shaped global handler state. |

## Host Matrix

| Signal surface | macOS arm64 | Linux x86_64 | Windows x86_64 | Evidence |
| --- | --- | --- | --- | --- |
| `ctrl_c()` | supported-by-tokio | supported-by-tokio | supported-by-tokio | Tokio `ctrl_c()` lowering is host-independent in generated Rust; deterministic e2e delivery requires an external signal harness and is deferred. |
| `terminate()` | supported | supported | host-limited | Unix lowering uses Tokio's Unix signal stream. `sigterm().supported` lowers to `cfg!(unix)`, and non-Unix `terminate()` returns typed `SignalError` until a Windows shutdown signal design and fixture are accepted. |
| `shutdown_stream().next()` | supported | supported | host-limited | Unix waits on Ctrl-C or SIGTERM. Non-Unix waits on Ctrl-C only and does not claim SIGTERM support. |

## CPython Family Mapping

| CPython family | Sifr disposition | Representative M5 fixtures |
| --- | --- | --- |
| `signal.signal` arbitrary handler registration | `unsupported-with-diagnostic` | `signal_handler_registration_unsupported` |
| `signal.pause`, `signal.getsignal`, `signal.raise_signal`, `signal.pthread_sigmask` | `unsupported-with-diagnostic` / `host-limited` | `signal_pause_unsupported`, `signal_getsignal_unsupported`, `signal_raise_signal_unsupported`, `signal_pthread_sigmask_unsupported` |
| `strsignal`, `SIGINT`, `SIGTERM` | `adapted-for-sifr-api` | `signal_constants_strsignal` |

## Follow-up Boundaries

- Add deterministic external-signal harness coverage before claiming runtime delivery e2e beyond codegen lowering.
- Add cleanup-scope traceability for `sifr.resource`.
- Add explicit task/request context propagation traceability for `sifr.task.Context` and `ContextKey[T]`.
- Add structured diagnostics and warning-global rejection traceability.
