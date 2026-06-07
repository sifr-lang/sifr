# Supported Host Matrix

Status: active baseline for split production-stdlib substrate phases.

| Host Concern | macOS arm64 | Linux x86_64 | Windows x86_64 | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| Rust `String`/`str` text invariants | supported | supported | supported | text/i18n | Normal Sifr `str` is valid Unicode scalar text on every host. |
| Path byte/text boundary | host-limited | host-limited | host-limited | text/i18n + runtime | OS path interop that needs byte-preserving text is deferred to a separate issue; this phase does not smuggle invalid Unicode into `str`. |
| Binary file I/O prerequisite | supported | supported | supported | existing `sifr.io` owner | M0 smoke passes read/write/close/drop and byte-preserving round trips; file-handle seek/tell remains unsupported where not implemented. |
| Explicit text file I/O | blocked-on-text-i18n-m1 | blocked-on-text-i18n-m1 | blocked-on-text-i18n-m1 | text/i18n | Text `open(...)` requires explicit encoding and literal/static mode. |
| Host locale discovery | host-limited | host-limited | host-limited | text/i18n | Read-only `sifr.i18n.host_locale() -> Option[LocaleId]`; never supplies default text encodings. |
| ICU4X compiled locale data | planned | planned | planned | text/i18n | M3 records exact ICU4X components and supported data set. |
| Current-thread async task scheduler | blocked-on-concurrency-runtime-m1 | blocked-on-concurrency-runtime-m1 | blocked-on-concurrency-runtime-m1 | concurrency/runtime | Tokio remains internal and current-thread; no public runtime or event-loop handles. |
| Task/thread sendability and shareability diagnostics | blocked-on-concurrency-runtime-m1 | blocked-on-concurrency-runtime-m1 | blocked-on-concurrency-runtime-m1 | concurrency/runtime | M1 starts task-boundary enforcement; M2/M3/M4/M6 extend it to channels, offload, process, and IPC. |
| Channels, locks, semaphores, and async sync primitives | blocked-on-concurrency-runtime-m2 | blocked-on-concurrency-runtime-m2 | blocked-on-concurrency-runtime-m2 | concurrency/runtime | Bounded backpressure and lock/permit await rules are host-independent unless an implementation crate proves otherwise. |
| Blocking I/O offload | blocked-on-concurrency-runtime-m3 | blocked-on-concurrency-runtime-m3 | blocked-on-concurrency-runtime-m3 | concurrency/runtime | Tokio blocking pool is internal; APIs expose typed WorkerError and no raw thread handles. |
| CPU parallelism | blocked-on-concurrency-runtime-m3 | blocked-on-concurrency-runtime-m3 | blocked-on-concurrency-runtime-m3 | concurrency/runtime | Private Rayon pools only; default sizing uses available_parallelism(). |
| Subprocess spawning and termination | blocked-on-concurrency-runtime-m4 | blocked-on-concurrency-runtime-m4 | blocked-on-concurrency-runtime-m4 | concurrency/runtime | Text mode additionally waits for text/i18n M1; scoped spawn uses ProcessHandle with owned pipe access. |
| Shell subprocess execution effect | blocked-on-concurrency-runtime-m4 | blocked-on-concurrency-runtime-m4 | blocked-on-concurrency-runtime-m4 | concurrency/runtime | Shell usage is explicit and carries @shell_exec plus blocking/async workload classification. |
| Signals and structured shutdown streams | blocked-on-concurrency-runtime-m5 | blocked-on-concurrency-runtime-m5 | host-limited | concurrency/runtime | SIGINT/SIGTERM are the initial portable targets; Unix-only signals require host-specific rows and fixtures. |
| Deterministic cleanup scopes | blocked-on-concurrency-runtime-m5 | blocked-on-concurrency-runtime-m5 | blocked-on-concurrency-runtime-m5 | concurrency/runtime | ExitStack/AsyncExitStack cleanup reports typed evidence under cancellation. |
| Typed IPC frames over process pipes | blocked-on-concurrency-runtime-m6 | blocked-on-concurrency-runtime-m6 | blocked-on-concurrency-runtime-m6 | concurrency/runtime | Payloads require IpcSerializable; schema negotiation and malformed-frame behavior are M6-owned. |
| TCP sockets and DNS | blocked-on-network-http | blocked-on-network-http | blocked-on-network-http | network/HTTP | Text decoding must consume text/i18n M1. |
| TLS roots and certificate verification | blocked-on-network-http | blocked-on-network-http | blocked-on-network-http | network/HTTP | No local text fallback for diagnostics or IDNA. |
