# Network HTTP Workload Database

Status: M0 baseline; implementation entries are backlog until their owning milestone closes.

| API family | Owner milestone | Classification | Async-context behavior | Diagnostic or fixture |
| --- | --- | --- | --- | --- |
| `sifr.net.connect_tcp`, `listen_tcp`, `TcpListener.accept`, TCP read/write/close/split/shutdown | TCP | async-native | Legal suspension points. | M1 loopback and cancellation fixtures. |
| `sifr.net.resolve_address` / DNS lookup helpers | TCP | async-native | Legal suspension point with provider timeout/cancellation evidence. | M1 loopback-literal and resolver-policy fixtures. |
| Accepted sync network helpers | TCP | sync `@blocking_io` | Rejected in async contexts unless routed through provider offload. | M1 workload diagnostic fixtures. |
| Constrained `sifr.net.UdpSocket` | TCP | deferred-to-phase-X | No async or sync public API in M1 unless M0 records a named production consumer and fixture-insufficiency rationale. | Inventory decision `udp-constrained-datagram`. |
| `sifr.tls` handshake, read, write, flush, `close_notify`, close, split | TLS | async-native | Legal suspension points with typed TLS and nested network evidence. | M2 TLS loopback and mTLS fixtures. |
| `sifr.url` parse/build/percent/query helpers over ASCII or already-valid Sifr text | URL/HTTP primitives | pure | Legal in async contexts; size/limit failures are typed errors. | M3 parser/property fixtures. |
| `sifr.url` non-UTF-8 codec helpers and Unicode/IDNA host canonicalization | URL/HTTP primitives | blocked-on-text-i18n-m1 / blocked-on-text-i18n-m2 | Not implemented locally; must call text/i18n provider milestones when unblocked. | M3 blocked-state fixtures. |
| `sifr.http` header/cookie-header validation primitives | URL/HTTP primitives | pure | Legal in async contexts under configured limits. | M3 header/cookie fixtures. |
| `sifr.http` request/response transport and body streaming | HTTP transport | async-native | Legal suspension points with provider cancellation/backpressure semantics. | M4 HTTP/1.1, HTTP/2, HTTPS loopback fixtures. |
| `sifr.http` body collect-with-limit helper, if accepted by M0 | HTTP transport | async-native bounded collection | Legal only with explicit limit and typed `TooLargeError`. | M4 body limit fixtures. |
| Rejected CPython-shaped network/web imports | M0 | unsupported/deferred | Compile-time import diagnostic. | `unsupported_cpython_network_imports.sifr` and M0 e2e fail fixtures. |

## Policy

No blocking network, TLS, DNS, or HTTP helper may be added without a native async counterpart or an explicit `@blocking_io` classification plus offload-only guidance. Pure parser helpers must be bounded by explicit size limits and return typed errors rather than hiding unbounded CPU or memory work.
