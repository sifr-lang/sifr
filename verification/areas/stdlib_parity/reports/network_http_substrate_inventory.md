# Network HTTP Substrate Inventory

Status: closed; network/HTTP implementation capabilities are merged, terminal-state inventory is closed, and final readiness review is recorded.

CPython checkout: `../cpython` at `14cbd0e6afa98355bdc6749b8230fed4c9b21bd6`.

Platform rules: [platform_rules.md](../platform/platform_rules.md).

## Production Surface Boundary

| Surface | Owner capability | Terminal state | Stability | Notes |
| --- | --- | --- | --- | --- |
| `sifr.net` TCP streams, listener, address values, DNS helpers | TCP | production-public | stable-public-api | Canonical low-level async network API. Tokio and raw descriptors stay internal. |
| `sifr.net` constrained UDP datagrams | TCP | deferred-to-future-capability | stable-public-api if later accepted | network/HTTP baseline capability found no named near-term production consumer that cannot be served by TCP/TLS/HTTP loopback fixtures. No partial public UDP API ships. |
| Internal readiness primitives | TCP | internal-only | unstable-internal-substrate | Used only behind async streams and transport drivers; no `select`/selector API. |
| `sifr.tls` configs and TLS streams | TLS | production-public | stable-public-api | Rustls-backed TLS with safe verification defaults, mTLS, SNI, ALPN, and typed errors. |
| `sifr.url` URL/query/percent primitives | URL/HTTP primitives | production-public | stable-public-api | ASCII and already-valid Sifr text only until text/i18n unblocks codec and Unicode behavior. |
| `sifr.http` method/status/version/header/head/body/error primitives | URL/header/cookie capability/HTTP-transport capability | production-substrate | stable-production-substrate | Protocol substrate consumed by capability handoff and the future HTTP client capability. |
| Internal HTTP client/server transport harness | HTTP transport | test-only-harness | test-only-harness | Deterministic loopback validation only; not `sifr.http.client` or `sifr.http.server`. |
| Server accept/dispatch/shutdown substrate | process runtime/task context and shutdown | production-substrate | stable-production-substrate | Single-runtime-worker per process; the HTTP server framework owns framework product and multi-core throughput wait for `network-http-serving-scale-capability`. |
| Observability hooks for DNS/connect/TLS/HTTP lifecycle | handoff | production-substrate | stable-production-substrate | `tracing` events and optional approved `metrics` schema; no global subscriber/exporter. |

## Rejected Or Unsupported CPython-Shaped Surfaces

| Surface | Terminal state | Replacement | Rationale | Revisit rule |
| --- | --- | --- | --- | --- |
| bare `socket` and `sifr.socket` | unsupported-with-diagnostic | `sifr.net` | Descriptor-shaped sockets, `makefile`, `dup`, `detach`, `fromfd`, and monkeypatchable globals conflict with typed async stream ownership. | Only a new Sifr-native extension to `sifr.net` can revisit this. |
| bare `ssl` and `sifr.ssl` | unsupported-with-diagnostic | `sifr.tls` | TLS is typed config/stream substrate, not `SSLContext`/`SSLSocket` parity. | Only a Sifr-native TLS product requirement can revisit this. |
| bare `select`, `selectors`, `sifr.select`, `sifr.selectors` | unsupported-with-diagnostic | `sifr.net` / `sifr.task` | Manual readiness and raw event-loop policy are internal. | Future low-level readiness API needs a separate architecture issue. |
| bare `urllib`, `sifr.urllib`, `sifr.urllib.parse`, `sifr.urllib.request` | unsupported-with-diagnostic | `sifr.url` | URL parsing is typed; opener/handler request APIs belong to a future HTTP client capability. | Future HTTP client must remain Sifr-native. |
| bare `http.client` and `sifr.http.client` | unsupported-with-diagnostic | `sifr.http` substrate now; client capability later | CPython-shaped client policy is not the product surface. | Future client capability owns modern pooling/retry/auth/cookie/proxy APIs. |
| bare `http.server` and `sifr.http.server` | unsupported-with-diagnostic | capability handoff over `sifr.http` | Toy server and handler-subclass model are rejected. | the HTTP server framework capability owns framework routing, middleware, extractors, and lifecycle. |
| bare `socketserver` and `sifr.socketserver` | rejected | capability handoff over `sifr.http` | Inheritance-heavy mixins conflict with Sifr's static model and serving-scale boundary. | No compatibility adapter; new server APIs must be Sifr-native. |
| HTTP/3 / QUIC | deferred-to-future-capability | future transport capability | Requires separate runtime, security, and QUIC strategy. | Revisit through a transport issue after HTTP/2 substrate closes. |
| WebSocket and CONNECT public APIs | deferred-to-future-capability | future product capability | Upgrade behavior requires separate backpressure/security decisions. | Internal upgrade hooks may be test-only harness only. |
| Multipart/form parsing | deferred-to-http-server-framework | capability handoff or HTTP client capability | Product-level body parsing and bomb limits are outside substrate. | Revisit with framework/client requirements. |
| Content-Encoding compression | deferred-to-future-capability | future client/framework compression capability | Compression/decompression bomb policy needs separate ownership. | HPACK remains in HTTP/2 substrate; body compression is separate. |

## network/HTTP baseline capability Resolved Decisions

| Decision | Outcome | Evidence |
| --- | --- | --- |
| byte-buffer public name | The `ByteBuffer` placeholder is resolved to Sifr's existing built-in `bytes` type. Construction and inspection use first-class `bytes` operations without an import. | Reuses the existing owned immutable byte-buffer value model and avoids creating a second public byte type for TCP/TLS/HTTP. |
| TCP full-duplex | `TcpStream.split()` consumes a live stream and returns owned affine `TcpReadHalf` and `TcpWriteHalf`; recombine and borrowed split are rejected for v1. | Prevents shared mutable aliasing while allowing task-separated full-duplex protocols. |
| TCP half-close | `shutdown_write()` is accepted; it preserves the read side, returns deterministic repeated-shutdown evidence, and write-after-shutdown is a typed error. | Required for request-end signaling and protocol-correct loopback tests. |
| DNS | `tokio::net::lookup_host` is accepted; custom resolver records and Happy Eyeballs are deferred. | Respects host resolver configuration without adding resolver product policy. |
| UDP | Deferred because no named near-term production consumer was recorded that needs datagrams and cannot use TCP/TLS/HTTP loopback fixtures. | Avoids a partial public datagram API. |
| TLS close | `TlsStream.close()` and `TlsWriteHalf.close()` consume their handle, first attempt `close_notify()` if it has not already completed, flush accepted plaintext and close alert before success, then release the underlying TCP resource. Cancellation, shutdown, or lower transport failure returns `TlsError` with deterministic close/progress evidence. `TlsReadHalf.close()` consumes only the read half and stops local reads without sending TLS alerts. | Rustls buffering requires explicit flush/shutdown evidence; write-side close remains `close_notify()`, while `close()` is resource release. |
| HTTP substrate namespace | Stable protocol primitives live under `sifr.http`; `sifr.http.core` is rejected as an extra stable namespace layer. | capability handoff and HTTP client handoff should consume one canonical substrate. |
| HTTP body stream | Body chunks are built-in `bytes`; EOF is `None`; trailers are accepted as explicit `Trailers(HeaderMap)`; default yielded chunk limit is 64 KiB; `collect_with_limit(max_bytes)` is accepted and unbounded `collect()` is rejected. | Prevents unbounded buffering and duplicated body types. |
| HTTP/2 priority/extensions | Priority is ignored/deferred for public semantics in v1; accepted `h2` behavior may parse/forward internally but Sifr exposes no priority API. Unknown extension frames are ignored only after frame-size/accounting checks and otherwise map to `ProtocolError` with unsupported-extension-frame evidence; no panic or resource-limit bypass. | Keeps HTTP/2 behavior crate-backed and resource-bounded. |
| `SO_REUSEPORT` | Deferred from public API entirely until `network-http-serving-scale-capability` ships. `listen_tcp(..., reuse_addr=True)` never implies `SO_REUSEPORT`; no separate reuse-port constructor ships in task runtime. | Serving scale belongs to the future serving-scale capability, not the substrate. |
| serving scale | Network/HTTP v1 is single-runtime-worker per process. Multi-core serving throughput is owned by `network-http-serving-scale-capability`. | Prevents server framework capability from overclaiming throughput readiness. |

## Dependency Decisions

| Capability | Crate decision | State | Public hiding rule |
| --- | --- | --- | --- |
| async sockets/timers/I/O | `tokio` 1.53.1 with `macros`, `rt`, `sync`, `time`, `net`, `io-util` | accepted | No Tokio handles or types in public APIs. Full audit: `network_http_dependency_audit.md`. |
| cancellation/I/O helpers | `tokio-util` 0.7.18 conditional | internal-only | Only behind Sifr stream/cancellation internals. |
| byte buffers | `bytes` 1.12.1 | accepted | Public APIs expose Sifr byte buffers, never `bytes::Bytes`. |
| lifecycle tracing | `tracing` 0.1.44 `std` | accepted | Events/spans only; no subscriber/exporter type leaks. |
| socket options | `socket2` 0.6.4 conditional | host-limited where needed | Only accepted option set; no raw socket constants. |
| metrics | `metrics` 0.24.6 conditional | deferred-to-future-capability until schema approval | No recorder/exporter setup. |
| URL | `url` 2.5.8 | accepted | Wrapped into `UrlError`; IDNA behavior blocked until text/i18n TLS capability sign-off. |
| percent encoding | `percent-encoding` 2.3.2 | accepted | Named encodings call text/i18n when unblocked. |
| TLS | `rustls`, `tokio-rustls`, `rustls-platform-verifier`, `rustls-pemfile` | accepted | Public config/errors hide crate types. |
| HTTP | `http`, `http-body`, `http-body-util`, `hyper`, `h2`, `tower-service` | accepted/conditional | Public HTTP types are Sifr-owned; Tower/Hyper types do not leak. |
| cookie header | Sifr-owned parser; no external cookie crate | accepted | Header-level parse/build only. |
| Ring 5 fixtures | `tokio-test`, `proptest`, `rcgen`, `tracing-subscriber` | test-only-harness | Must be absent from production dependency snapshots. |

## HTTP Type Table

| Type | Terminal state | Stability | network/HTTP baseline capability decision |
| --- | --- | --- | --- |
| `Method` | production-substrate | stable-production-substrate | Backed by `http::Method`; validates registered token methods plus extension tokens; invalid bytes return `ProtocolError`. |
| `Status` | production-substrate | stable-production-substrate | Backed by `http::StatusCode`; only 100-999 accepted, unknown numeric codes preserved without reason-phrase policy. |
| `Version` | production-substrate | stable-production-substrate | Supports HTTP/1.0, HTTP/1.1, HTTP/2; HTTP/3 deferred. |
| `HeaderName` | production-substrate | stable-production-substrate | Lowercase canonical HTTP token; ASCII only; invalid token returns `HeaderError` with invalid-name evidence. |
| `HeaderValue` | production-substrate | stable-production-substrate | Byte/ASCII-safe value; obs-fold rejected; arbitrary text decoding blocked on text/i18n async-network capability. |
| `HeaderMap` | production-substrate | stable-production-substrate | Preserves duplicate order; singleton semantic checks are owned by transport validation. |
| `RequestHead` | production-substrate | stable-production-substrate | Method, URL/authority target metadata, version, headers; no body ownership. |
| `ResponseHead` | production-substrate | stable-production-substrate | Status, version, headers; no body ownership. |
| `BodyStream` | production-substrate | stable-production-substrate | Async stream of built-in `bytes` chunks with typed EOF/cancellation/reset evidence. |
| `BodyChunk` | production-substrate | compiler-known-intrinsic | Built-in `bytes` type with first-class construction and inspection operations. |
| `Trailers` | production-substrate | stable-production-substrate | Accepted for HTTP/2 and HTTP/1.1 chunked bodies as `HeaderMap` after EOF; disabled by default for collected bodies unless explicitly requested. |
| `HttpError`, `ProtocolError`, `HeaderError`, `BodyError` | production-substrate | stable-production-substrate | Flat typed error classes with deterministic lower-layer, cancellation, timeout, and size evidence messages. |

## Header And Request-Smuggling Rules

| Rule | network/HTTP baseline capability decision |
| --- | --- |
| Header name syntax | ASCII `tchar` token only; uppercase is accepted at parse boundary and canonicalized to lowercase; non-token bytes return `HeaderError` with invalid-name evidence. |
| obs-fold | Always rejected with `HeaderError` carrying obs-fold evidence; no line unfolding compatibility. |
| Whitespace | Trim optional whitespace around field values at parse boundary only; preserve interior bytes; reject control bytes except HTAB where HTTP allows it. |
| Duplicate headers | Preserve insertion order. `Set-Cookie` always remains multi-valued. Singleton transport headers (`Content-Length`, `Host`, `Transfer-Encoding`) receive explicit validation in HTTP-transport capability. |
| `Content-Length` disagreement | Multiple identical values accepted as one value; conflicting values return `ProtocolError` with conflicting-content-length evidence. Body shorter/longer than accepted length returns `BodyError` with length-mismatch evidence. |
| `Content-Length` plus `Transfer-Encoding: chunked` | Rejected for requests and responses with `ProtocolError` carrying ambiguous-body-length evidence; no request-smuggling fallback. |
| Header total limit | Default 64 KiB decoded header section; configurable lower or higher with hard maximum 1 MiB. |

## HTTP Body Stream Rules

| Field | network/HTTP baseline capability decision |
| --- | --- |
| Chunk type | Built-in `bytes`. |
| EOF behavior | `None` from `read_chunk` means clean EOF. Trailers, when accepted, are retrieved through an explicit `trailers()` result after EOF. |
| Trailers | Accepted as `Trailers(HeaderMap)` for protocol substrate; unsupported trailer usage returns `BodyError` with trailers-unsupported evidence when a caller disabled trailers. |
| Max chunk size | Default 64 KiB per yielded chunk; hard maximum 1 MiB unless later HTTP/runtime work records a larger use case. |
| Max collected body size | Default 16 MiB for `collect_with_limit`; caller must pass an explicit lower/higher limit and cannot exceed hard maximum 128 MiB without a later HTTP/runtime amendment. |
| Collect helper | `collect_with_limit(max_bytes)` accepted; unbounded `collect()` rejected. |
| Cancellation while reading | Returns `BodyError` with read-cancellation and `bytes_observed` evidence and propagates provider cancellation evidence. |
| Cancellation while writing | Returns `BodyError` with write-cancellation and `bytes_accepted` evidence; peer may have received a prefix. |
| HTTP/2 reset mapping | `RST_STREAM` maps to `ProtocolError`/`HttpError` with stream-reset code and byte-observation evidence. |
| Partial progress | Every cancellation, reset, timeout, and write failure carries byte-count evidence when the layer can know it. |

## HTTP/2 Limits And Protocol Mapping

| Concern | network/HTTP baseline capability decision |
| --- | --- |
| SETTINGS max concurrent streams | Default 100 inbound streams per connection; peer-advertised lower value is honored; higher value capped unless configured. |
| Initial flow-control window | Use `h2` default initial window unless configured; Sifr body buffering caps still apply. |
| Max frame size | Accept peer frame size up to RFC maximum 16,777,215 bytes but enforce body/header buffering caps before allocation. |
| Max buffered body per stream | 1 MiB buffered at a time by default; larger bodies must stream through backpressure. |
| PING handling | Reply to valid PING through `h2`; more than 8 unanswered or rate-limit-exceeding PINGs map to `ProtocolError` with ping-flood evidence. |
| RST_STREAM | Maps to `ProtocolError` with stream-reset code and byte-observation evidence and cancels only the affected body stream. |
| GOAWAY | Drains streams with IDs at or below the last accepted ID, rejects new streams, and returns `HttpError` with connection-closing evidence for new dispatch. |
| Malformed frames | Map to `ProtocolError` with malformed-frame kind evidence; no panic and no fallback parser. |
| HPACK | Uses `h2` HPACK with header-list size capped by the header total limit. |

## Size Limits

| Input | Default limit | Hard limit | Error |
| --- | --- | --- | --- |
| URL string | 8 KiB | 64 KiB | `UrlError` with size-limit evidence |
| Query string | 8 KiB | 64 KiB | `UrlError` with size-limit evidence |
| Header name | 128 bytes | 1024 bytes | `HeaderError` with size-limit evidence |
| Header value | 8 KiB | 64 KiB | `HeaderError` with size-limit evidence |
| Header section total | 64 KiB | 1 MiB | `HeaderError` with size-limit evidence |
| Body chunk yielded | 64 KiB | 1 MiB | `BodyError` with size-limit evidence |
| Collected body | 16 MiB | 128 MiB | `BodyError` with size-limit evidence |
| TLS record/application write buffer | implementation default | bounded by stream write input length and body chunk hard limit | `TlsError` or `BodyError` with size-limit evidence |

## URL Authority And Redaction Rules

| Concern | network/HTTP baseline capability decision |
| --- | --- |
| Userinfo | Parsed and preserved only as typed URL component; never emitted in display/log output except as `***@`; password material always redacted. |
| Host validation | ASCII domain labels, IPv4, IPv6 literals, and already-punycode `xn--` labels accepted. Empty host rejected for schemes requiring authority. Non-ASCII host blocked until text/i18n TLS capability sign-off. |
| Port validation | Decimal 0-65535 only; empty or non-decimal port returns `UrlError` with invalid-port evidence; default-port normalization is display-only and does not mutate stored URL. |
| Path normalization | Dot-segment normalization helper is explicit; parsing does not silently normalize path semantics. Percent-encoded slash/backslash remains encoded unless caller explicitly decodes bytes. |
| Percent-decoding | Percent helpers return bytes by default. Text conversion uses UTF-8 only where explicitly named; other encodings blocked on text/i18n async-network capability. Invalid percent triplets return `UrlError` with invalid-percent evidence. |
| Sensitive query values | Keys matching `token`, `secret`, `password`, `key`, `signature`, `auth`, or user-configured sensitive keys are redacted in observability output. |
| Header redaction | `Authorization`, `Proxy-Authorization`, `Cookie`, `Set-Cookie`, `X-Api-Key`, and configured sensitive headers are redacted by default. |
| Body redaction | Bodies are never logged by default. Size-limited previews require explicit opt-in and text previews remain blocked on text/i18n async-network capability. |
| Certificate redaction | Raw DER, private keys, and full subject/SAN display are not logged; fingerprints and typed verification reason codes are allowed. Unicode certificate display waits for text/i18n TLS capability. |
| Peer address redaction | Peer addresses are logged by default for loopback/server diagnostics but may be redacted by config; redaction must preserve host-family and port-presence evidence. |
| TLS material | Session keys, secrets, tickets, and key material are never exposed in logs/events. |

## Text/I18n Dependency States

| Surface | State | Provider dependency |
| --- | --- | --- |
| TCP/TLS/HTTP protocol bytes | production-substrate | none |
| HTTP text body helpers | blocked-on-text-i18n-async-network capability | `sifr.encoding` / explicit text I/O |
| Header and cookie non-ASCII/user-text conversion | blocked-on-text-i18n-async-network capability | text decoding substrate |
| URL non-UTF-8 encodings | blocked-on-text-i18n-async-network capability | codec lookup and error handlers |
| Unicode/IDNA host canonicalization | blocked-on-text-i18n-TLS capability | approved Unicode/IDNA version and normalization |
| Text-heavy demos using `open(..., encoding=...)` | blocked-on-text-i18n-async-network capability | explicit text I/O |
| Locale-sensitive diagnostics/formatting | blocked-on-text-i18n-URL/header/cookie capability | locale identifiers and formatters |

## Concurrency/Runtime Dependency States

| Surface | State | Provider dependency |
| --- | --- | --- |
| TCP/TLS/HTTP cancellation and deadlines | blocked-on-concurrency-runtime-async-network capability | task cancellation/deadline model |
| Stream backpressure and suspension | blocked-on-concurrency-runtime-TLS capability | bounded sync/backpressure semantics |
| Blocking sync helper offload | blocked-on-concurrency-runtime-URL/header/cookie capability | `sifr.runtime.spawn_blocking` and workload diagnostics |
| Process-backed serving scale | blocked-on-concurrency-runtime-HTTP-transport capability / blocked-on-concurrency-runtime-typed IPC capability | process runtime and typed IPC gates |
| Graceful shutdown | blocked-on-concurrency-runtime-network/HTTP readiness | signal/shutdown substrate |
| Observability/context propagation | blocked-on-concurrency-runtime-network/HTTP readiness | runtime diagnostics and task/request context |

## Error Taxonomy

Post-readiness Fable High amendment, 2026-06-12: the closed implementation exposes flat Sifr error classes with deterministic evidence messages. The unimplemented variant/nested names from earlier planning (`DnsError`, `ConnectError`, `TimeoutError`, `CancelledError`, `TooLargeError`, `NetError::Dns`, `HttpError::Tls`, and similar variant paths) are not shipped API.

| Error class | Owner | Evidence carried |
| --- | --- | --- |
| `NetError` | TCP | DNS, connect, timeout, reset, closed-handle, listener, stream, and socket-option evidence as stable messages. |
| `TlsError` | TLS | TLS config, handshake, read/write/flush, close-notify, transport, timeout, and closed-handle evidence as stable messages. |
| `CertificateError` | TLS | certificate/root/private-key parse and verification setup evidence as stable messages. |
| `UrlError` | URL/HTTP primitives | URL, authority, port, query, percent, provider-blocked, and size-limit evidence as stable messages. |
| `HeaderError` | URL/HTTP primitives | header name/value, header-list, and cookie-header validation evidence as stable messages. |
| `ProtocolError` | HTTP transport | HTTP method/status/version and protocol validation evidence as stable messages. |
| `BodyError` | HTTP transport | body chunk/collection size and body stream evidence as stable messages. |
| `HttpError` | HTTP transport | HTTP transport, TLS, protocol, body, timeout, and server/client lifecycle evidence as stable messages. |

## Capability Queue

| Capability | Traceability artifact | Acceptance |
| --- | --- | --- |
| TCP | `network_http_async_network_traceability.md` | async TCP/DNS substrate, split/half-close, workload diagnostics, no UDP unless the substrate baseline is amended. |
| TLS | `network_http_tls_traceability.md` | TLS configs/streams, safe verification, mTLS, close semantics, host/build records. |
| URL/HTTP primitives | `network_http_url_header_cookie_traceability.md` | URL, percent, header, and cookie-header primitives with text/i18n blocking states. |
| HTTP transport | `network_http_http_transport_traceability.md` | HTTP/1.1 and HTTP/2 loopback transport, body streaming, HTTPS/ALPN, resource limits. |
| handoff | `network_http_handoff_traceability.md` | docs, demos, dependency snapshots, panic scans, final inventory readiness, final review. |
