# Network HTTP Workload Database

Status: network/HTTP baseline capability; implementation entries are pending until their owning capability closes.

| API family | Owner capability | Classification | Async-context behavior | Diagnostic or fixture |
| --- | --- | --- | --- | --- |
| `sifr.net.connect_tcp`, `listen_tcp`, `TcpListener.accept`, TCP read/write/close/split/shutdown | TCP | async-native | Legal suspension points. | async-network capability loopback and cancellation fixtures. |
| `sifr.net.resolve_address` / DNS lookup helpers | TCP | async-native | Legal suspension point with provider timeout/cancellation evidence. | async-network capability loopback-literal and resolver-policy fixtures. |
| Accepted sync network helpers | TCP | sync `@blocking_io` | Rejected in async contexts unless routed through provider offload. | async-network capability workload diagnostic fixtures. |
| Constrained `sifr.net.UdpSocket` | TCP | deferred-to-future-capability | No async or sync public API in async-network capability unless network/HTTP baseline capability records a named production consumer and fixture-insufficiency rationale. | Inventory decision `udp-constrained-datagram`. |
| `sifr.tls` handshake, read, write, flush, `close_notify`, close, split | TLS | async-native | Legal suspension points with typed TLS and nested network evidence. | TLS capability TLS loopback and mTLS fixtures. |
| `sifr.url` parse/build/percent/query helpers over ASCII or already-valid Sifr text | URL/HTTP primitives | pure | Legal in async contexts; size/limit failures are typed errors. | URL/header/cookie capability parser/property fixtures. |
| `sifr.url` non-UTF-8 codec helpers and Unicode/IDNA host canonicalization | URL/HTTP primitives | blocked-on-text-i18n-async-network capability / blocked-on-text-i18n-TLS capability | Not implemented locally; must call text/i18n provider capabilitys when unblocked. | URL/header/cookie capability blocked-state fixtures. |
| `sifr.http` header/cookie-header validation primitives | URL/HTTP primitives | pure | Legal in async contexts under configured limits. | URL/header/cookie capability header/cookie fixtures. |
| `sifr.http` request/response transport and body streaming | HTTP transport | async-native | Legal suspension points with provider cancellation/backpressure semantics. | HTTP-transport capability HTTP/1.1, HTTP/2, HTTPS loopback fixtures. |
| `sifr.http` body collect-with-limit helper, if accepted by network/HTTP baseline capability | HTTP transport | async-native bounded collection | Legal only with explicit limit and typed `TooLargeError`. | HTTP-transport capability body limit fixtures. |
| Rejected CPython-shaped network/web imports | network/HTTP baseline capability | unsupported/deferred | Compile-time import diagnostic. | `unsupported_cpython_network_imports.sifr` and network/HTTP baseline capability e2e fail fixtures. |

## Policy

No blocking network, TLS, DNS, or HTTP helper may be added without a native async counterpart or an explicit `@blocking_io` classification plus offload-only guidance. Pure parser helpers must be bounded by explicit size limits and return typed errors rather than hiding unbounded CPU or memory work.
