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
| Subprocess spawning and termination | blocked-on-concurrency-runtime | blocked-on-concurrency-runtime | blocked-on-concurrency-runtime | concurrency/runtime | Text mode additionally waits for text/i18n M1. |
| Signals | blocked-on-concurrency-runtime | blocked-on-concurrency-runtime | host-limited | concurrency/runtime | Platform-specific signal behavior is recorded by concurrency/runtime. |
| TCP sockets and DNS | blocked-on-network-http | blocked-on-network-http | blocked-on-network-http | network/HTTP | Text decoding must consume text/i18n M1. |
| TLS roots and certificate verification | blocked-on-network-http | blocked-on-network-http | blocked-on-network-http | network/HTTP | No local text fallback for diagnostics or IDNA. |
