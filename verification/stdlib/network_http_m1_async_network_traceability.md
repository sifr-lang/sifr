# Network HTTP M1 Traceability: Async Network Runtime

Status: backlog from M0.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| `sifr.net` public module and intrinsic boundary | `production-public`, stable-public-api; no Tokio or descriptor leak. Public read chunks use built-in `bytes` with helpers under `sifr.bytes`. | Import/type-check/e2e pass fixtures for canonical `sifr.net` imports and unsupported bare `socket`/`select` diagnostics. |
| `SO_REUSEPORT` | Deferred from public API entirely until the serving-scale follow-up closes; `reuse_addr` never implies reuse-port. | `listen_tcp` fixtures prove no public reuse-port constructor exists in M1. |
| TCP connect/listen/accept/read/write/close | Async-native over current-thread Tokio with provider cancellation/deadlines. | Deterministic loopback tests for connect, accept, EOF, reset, local/remote address, close, and resource limits. |
| TCP owned split halves | `split()` consumes a live stream and returns affine owned halves. | Concurrent read/write task fixture, sendability diagnostics, no shared mutable aliasing. |
| TCP write-side half-close | `shutdown_write()` sends FIN, preserves read side, write-after-shutdown is typed. | Half-close request-end signaling loopback and repeated-shutdown fixture. |
| DNS/address resolution | `tokio::net::lookup_host`; custom resolver and Happy Eyeballs deferred unless M0 is amended. | Resolver timeout/cancellation/address-order fixtures using loopback literals and deterministic host rows. |
| UDP | `deferred-to-phase-X` until a named production consumer plus fixture-insufficiency rationale is recorded. | No partial public UDP API unless the M0 decision is amended and checked in. |
| Readiness primitives | internal-only. | No public `sifr.select`/`sifr.selectors`; import diagnostics remain stable. |
| Blocking sync helpers | sync `@blocking_io` if accepted. | Async-context rejection fixtures and offload guidance. |

## CPython Evidence

Mine `test_socket`, `test_select`, `test_selectors`, and asyncio stream/server tests for loopback and cancellation fixtures only.
