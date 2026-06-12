# Network HTTP Substrate Inventory

Status: M5 closeout candidate; M0-M4 implementation milestones are merged and terminal-state inventory is closed pending final M5 review.

CPython checkout: `/Users/yaseralnajjar/work/sifr/cpython` at `14cbd0e6afa98355bdc6749b8230fed4c9b21bd6`.

Platform contract: [platform_contract.md](../platform/platform_contract.md).

## Production Surface Boundary

| Surface | Owner milestone | Terminal state | Stability | Notes |
| --- | --- | --- | --- | --- |
| `sifr.net` TCP streams, listener, address values, DNS helpers | M1 | production-public | stable-public-api | Canonical low-level async network API. Tokio and raw descriptors stay internal. |
| `sifr.net` constrained UDP datagrams | M1 | deferred-to-phase-X | stable-public-api if later accepted | M0 found no named near-term production consumer that cannot be served by TCP/TLS/HTTP loopback fixtures. No partial public UDP API ships. |
| Internal readiness primitives | M1 | internal-only | unstable-internal-substrate | Used only behind async streams and transport drivers; no `select`/selector API. |
| `sifr.tls` configs and TLS streams | M2 | production-public | stable-public-api | Rustls-backed TLS with safe verification defaults, mTLS, SNI, ALPN, and typed errors. |
| `sifr.url` URL/query/percent primitives | M3 | production-public | stable-public-api | ASCII and already-valid Sifr text only until text/i18n unblocks codec and Unicode behavior. |
| `sifr.http` method/status/version/header/head/body/error primitives | M3/M4 | production-substrate | stable-production-substrate | Protocol substrate consumed by Phase 41 and the future HTTP client phase. |
| Internal HTTP client/server transport harness | M4 | test-only-harness | test-only-harness | Deterministic loopback validation only; not `sifr.http.client` or `sifr.http.server`. |
| Server accept/dispatch/shutdown substrate | M4/M5 | production-substrate | stable-production-substrate | Single-runtime-worker per process; Phase 41 owns framework product and multi-core throughput waits for `ad-hoc-network-http-serving-scale-follow-up`. |
| Observability hooks for DNS/connect/TLS/HTTP lifecycle | M5 | production-substrate | stable-production-substrate | `tracing` events and optional approved `metrics` schema; no global subscriber/exporter. |

## Rejected Or Unsupported CPython-Shaped Surfaces

| Surface | Terminal state | Replacement | Rationale | Revisit rule |
| --- | --- | --- | --- | --- |
| bare `socket` and `sifr.socket` | unsupported-with-diagnostic | `sifr.net` | Descriptor-shaped sockets, `makefile`, `dup`, `detach`, `fromfd`, and monkeypatchable globals conflict with typed async stream ownership. | Only a new Sifr-native extension to `sifr.net` can revisit this. |
| bare `ssl` and `sifr.ssl` | unsupported-with-diagnostic | `sifr.tls` | TLS is typed config/stream substrate, not `SSLContext`/`SSLSocket` parity. | Only a Sifr-native TLS product requirement can revisit this. |
| bare `select`, `selectors`, `sifr.select`, `sifr.selectors` | unsupported-with-diagnostic | `sifr.net` / `sifr.task` | Manual readiness and raw event-loop policy are internal. | Future low-level readiness API needs a separate architecture issue. |
| bare `urllib`, `sifr.urllib`, `sifr.urllib.parse`, `sifr.urllib.request` | unsupported-with-diagnostic | `sifr.url` | URL parsing is typed; opener/handler request APIs belong to a future HTTP client phase. | Future HTTP client must remain Sifr-native. |
| bare `http.client` and `sifr.http.client` | unsupported-with-diagnostic | `sifr.http` substrate now; client phase later | CPython-shaped client policy is not the product surface. | Future client phase owns modern pooling/retry/auth/cookie/proxy APIs. |
| bare `http.server` and `sifr.http.server` | unsupported-with-diagnostic | Phase 41 over `sifr.http` | Toy server and handler-subclass model are rejected. | Phase 41 owns framework routing, middleware, extractors, and lifecycle. |
| bare `socketserver` and `sifr.socketserver` | rejected | Phase 41 over `sifr.http` | Inheritance-heavy mixins conflict with Sifr's static model and serving-scale boundary. | No compatibility adapter; new server APIs must be Sifr-native. |
| HTTP/3 / QUIC | deferred-to-phase-X | future transport phase | Requires separate runtime, security, and QUIC strategy. | Revisit through a transport issue after HTTP/2 substrate closes. |
| WebSocket and CONNECT public APIs | deferred-to-phase-X | future product phase | Upgrade behavior requires separate backpressure/security decisions. | Internal upgrade hooks may be test-only harness only. |
| Multipart/form parsing | deferred-to-phase-41 | Phase 41 or HTTP client phase | Product-level body parsing and bomb limits are outside substrate. | Revisit with framework/client requirements. |
| Content-Encoding compression | deferred-to-phase-X | future client/framework compression | Compression/decompression bomb policy needs separate ownership. | HPACK remains in HTTP/2 substrate; body compression is separate. |

## M0 Resolved Decisions

| Decision | Outcome | Evidence |
| --- | --- | --- |
| byte-buffer public name | The `ByteBuffer` placeholder is resolved to Sifr's existing built-in `bytes` type. No import is required for the type; helper constructors/utilities live under `sifr.bytes`. | Reuses the existing owned immutable byte-buffer value model and avoids creating a second public byte type for TCP/TLS/HTTP. |
| TCP full-duplex | `TcpStream.split()` consumes a live stream and returns owned affine `TcpReadHalf` and `TcpWriteHalf`; recombine and borrowed split are rejected for v1. | Prevents shared mutable aliasing while allowing task-separated full-duplex protocols. |
| TCP half-close | `shutdown_write()` is accepted; it preserves the read side, returns deterministic repeated-shutdown evidence, and write-after-shutdown is a typed error. | Required for request-end signaling and protocol-correct loopback tests. |
| DNS | `tokio::net::lookup_host` is accepted; custom resolver records and Happy Eyeballs are deferred. | Respects host resolver configuration without adding resolver product policy. |
| UDP | Deferred because no named near-term production consumer was recorded that needs datagrams and cannot use TCP/TLS/HTTP loopback fixtures. | Avoids a partial public datagram API. |
| TLS close | `TlsStream.close()` and `TlsWriteHalf.close()` consume their handle, first attempt `close_notify()` if it has not already completed, flush accepted plaintext and close alert before success, then release the underlying TCP resource. Cancellation returns `TlsError::Cancelled { during: "close", progress }`; failure preserves `TlsError::Shutdown` or nested `TlsError::Transport(NetError)` evidence. `TlsReadHalf.close()` consumes only the read half and stops local reads without sending TLS alerts. | Rustls buffering requires explicit flush/shutdown evidence; write-side close remains `close_notify()`, while `close()` is resource release. |
| HTTP substrate namespace | Stable protocol primitives live under `sifr.http`; `sifr.http.core` is rejected as an extra stable namespace layer. | Phase 41 and HTTP client handoff should consume one canonical substrate. |
| HTTP body stream | Body chunks are built-in `bytes`; EOF is `None`; trailers are accepted as explicit `Trailers(HeaderMap)`; default yielded chunk limit is 64 KiB; `collect_with_limit(max_bytes)` is accepted and unbounded `collect()` is rejected. | Prevents unbounded buffering and duplicated body types. |
| HTTP/2 priority/extensions | Priority is ignored/deferred for public semantics in v1; accepted `h2` behavior may parse/forward internally but Sifr exposes no priority API. Unknown extension frames are ignored only after frame-size/accounting checks and otherwise map to `ProtocolError::UnsupportedExtensionFrame`; no panic or resource-limit bypass. | Keeps HTTP/2 behavior crate-backed and resource-bounded. |
| `SO_REUSEPORT` | Deferred from public API entirely until `ad-hoc-network-http-serving-scale-follow-up` closes. `listen_tcp(..., reuse_addr=True)` never implies `SO_REUSEPORT`; no separate reuse-port constructor ships in M1. | Serving scale belongs to the follow-up, not the substrate. |
| serving scale | Network/HTTP v1 is single-runtime-worker per process. Multi-core serving throughput is owned by `ad-hoc-network-http-serving-scale-follow-up`. | Prevents Phase 41 from overclaiming throughput readiness. |

## Dependency Decisions

| Capability | Crate decision | State | Public hiding rule |
| --- | --- | --- | --- |
| async sockets/timers/I/O | `tokio` 1.52.3 with `macros`, `rt`, `sync`, `time`, `net`, `io-util` | accepted | No Tokio handles or types in public APIs. Full audit: `network_http_dependency_audit.md`. |
| cancellation/I/O helpers | `tokio-util` 0.7.18 conditional | internal-only | Only behind Sifr stream/cancellation internals. |
| byte buffers | `bytes` 1.11.1 | accepted | Public APIs expose Sifr byte buffers, never `bytes::Bytes`. |
| lifecycle tracing | `tracing` 0.1.44 `std` | accepted | Events/spans only; no subscriber/exporter type leaks. |
| socket options | `socket2` 0.6.4 conditional | host-limited where needed | Only accepted option set; no raw socket constants. |
| metrics | `metrics` 0.24.6 conditional | deferred-to-phase-X until schema approval | No recorder/exporter setup. |
| URL | `url` 2.5.8 | accepted | Wrapped into `UrlError`; IDNA behavior blocked until text/i18n M2 sign-off. |
| percent encoding | `percent-encoding` 2.3.2 | accepted | Named encodings call text/i18n when unblocked. |
| TLS | `rustls`, `tokio-rustls`, `rustls-platform-verifier`, `rustls-pemfile` | accepted | Public config/errors hide crate types. |
| HTTP | `http`, `http-body`, `http-body-util`, `hyper`, `h2`, `tower-service` | accepted/conditional | Public HTTP types are Sifr-owned; Tower/Hyper types do not leak. |
| cookie header | `cookie` 0.18.1 without jar/signing features | accepted | Header-level parse/build only. |
| Ring 5 fixtures | `tokio-test`, `proptest`, `rcgen`, `tracing-subscriber` | test-only-harness | Must be absent from production dependency snapshots. |

## HTTP Type Table

| Type | Terminal state | Stability | M0 decision |
| --- | --- | --- | --- |
| `Method` | production-substrate | stable-production-substrate | Backed by `http::Method`; validates registered token methods plus extension tokens; invalid bytes return `ProtocolError`. |
| `Status` | production-substrate | stable-production-substrate | Backed by `http::StatusCode`; only 100-999 accepted, unknown numeric codes preserved without reason-phrase policy. |
| `Version` | production-substrate | stable-production-substrate | Supports HTTP/1.0, HTTP/1.1, HTTP/2; HTTP/3 deferred. |
| `HeaderName` | production-substrate | stable-production-substrate | Lowercase canonical HTTP token; ASCII only; invalid token returns `HeaderError::InvalidName`. |
| `HeaderValue` | production-substrate | stable-production-substrate | Byte/ASCII-safe value; obs-fold rejected; arbitrary text decoding blocked on text/i18n M1. |
| `HeaderMap` | production-substrate | stable-production-substrate | Preserves duplicate order; singleton semantic checks are owned by transport validation. |
| `RequestHead` | production-substrate | stable-production-substrate | Method, URL/authority target metadata, version, headers; no body ownership. |
| `ResponseHead` | production-substrate | stable-production-substrate | Status, version, headers; no body ownership. |
| `BodyStream` | production-substrate | stable-production-substrate | Async stream of built-in `bytes` chunks with typed EOF/cancellation/reset evidence. |
| `BodyChunk` | production-substrate | compiler-known-intrinsic | Built-in `bytes` type; helper API under `sifr.bytes`. |
| `Trailers` | production-substrate | stable-production-substrate | Accepted for HTTP/2 and HTTP/1.1 chunked bodies as `HeaderMap` after EOF; disabled by default for collected bodies unless explicitly requested. |
| `HttpError`, `ProtocolError`, `HeaderError`, `BodyError` | production-substrate | stable-production-substrate | Typed errors with nested `NetError`, `TlsError`, and provider cancellation/timeout evidence. |

## Header And Request-Smuggling Rules

| Rule | M0 decision |
| --- | --- |
| Header name syntax | ASCII `tchar` token only; uppercase is accepted at parse boundary and canonicalized to lowercase; non-token bytes return `HeaderError::InvalidName`. |
| obs-fold | Always rejected with `HeaderError::ObsFold`; no line unfolding compatibility. |
| Whitespace | Trim optional whitespace around field values at parse boundary only; preserve interior bytes; reject control bytes except HTAB where HTTP allows it. |
| Duplicate headers | Preserve insertion order. `Set-Cookie` always remains multi-valued. Singleton transport headers (`Content-Length`, `Host`, `Transfer-Encoding`) receive explicit validation in M4. |
| `Content-Length` disagreement | Multiple identical values accepted as one value; conflicting values return `ProtocolError::ConflictingContentLength`. Body shorter/longer than accepted length returns `BodyError::LengthMismatch`. |
| `Content-Length` plus `Transfer-Encoding: chunked` | Rejected for requests and responses with `ProtocolError::AmbiguousBodyLength`; no request-smuggling fallback. |
| Header total limit | Default 64 KiB decoded header section; configurable lower or higher with hard maximum 1 MiB. |

## HTTP Body Stream Contract

| Field | M0 decision |
| --- | --- |
| Chunk type | Built-in `bytes`. |
| EOF behavior | `None` from `read_chunk` means clean EOF. Trailers, when accepted, are retrieved through an explicit `trailers()` result after EOF. |
| Trailers | Accepted as `Trailers(HeaderMap)` for protocol substrate; unsupported trailer usage returns `BodyError::TrailersUnsupported` when a caller disabled trailers. |
| Max chunk size | Default 64 KiB per yielded chunk; hard maximum 1 MiB unless a future phase records a larger use case. |
| Max collected body size | Default 16 MiB for `collect_with_limit`; caller must pass an explicit lower/higher limit and cannot exceed hard maximum 128 MiB without a future phase amendment. |
| Collect helper | `collect_with_limit(max_bytes)` accepted; unbounded `collect()` rejected. |
| Cancellation while reading | Returns `BodyError::Cancelled { direction: "read", bytes_observed }` and propagates provider cancellation evidence. |
| Cancellation while writing | Returns `BodyError::Cancelled { direction: "write", bytes_accepted }`; peer may have received a prefix. |
| HTTP/2 reset mapping | `RST_STREAM` maps to `ProtocolError::StreamReset { code, bytes_observed }` nested in `HttpError`. |
| Partial progress | Every cancellation, reset, timeout, and write failure carries byte-count evidence when the layer can know it. |

## HTTP/2 Limits And Protocol Mapping

| Concern | M0 decision |
| --- | --- |
| SETTINGS max concurrent streams | Default 100 inbound streams per connection; peer-advertised lower value is honored; higher value capped unless configured. |
| Initial flow-control window | Use `h2` default initial window unless configured; Sifr body buffering caps still apply. |
| Max frame size | Accept peer frame size up to RFC maximum 16,777,215 bytes but enforce body/header buffering caps before allocation. |
| Max buffered body per stream | 1 MiB buffered at a time by default; larger bodies must stream through backpressure. |
| PING handling | Reply to valid PING through `h2`; more than 8 unanswered or rate-limit-exceeding PINGs map to `ProtocolError::PingFlood`. |
| RST_STREAM | Maps to `ProtocolError::StreamReset { code, bytes_observed }` and cancels only the affected body stream. |
| GOAWAY | Drains streams with IDs at or below the last accepted ID, rejects new streams, and returns `HttpError::ConnectionClosing` for new dispatch. |
| Malformed frames | Map to `ProtocolError::MalformedFrame { kind }`; no panic and no fallback parser. |
| HPACK | Uses `h2` HPACK with header-list size capped by the header total limit. |

## Size Limits

| Input | Default limit | Hard limit | Error |
| --- | --- | --- | --- |
| URL string | 8 KiB | 64 KiB | `UrlError::TooLarge` |
| Query string | 8 KiB | 64 KiB | `UrlError::TooLarge` |
| Header name | 128 bytes | 1024 bytes | `HeaderError::TooLarge` |
| Header value | 8 KiB | 64 KiB | `HeaderError::TooLarge` |
| Header section total | 64 KiB | 1 MiB | `HeaderError::TooLarge` |
| Body chunk yielded | 64 KiB | 1 MiB | `BodyError::TooLarge` |
| Collected body | 16 MiB | 128 MiB | `TooLargeError` nested in `BodyError` |
| TLS record/application write buffer | implementation default | bounded by stream write input length and body chunk hard limit | `TlsError::TooLarge` or `BodyError::TooLarge` |

## URL Authority And Redaction Rules

| Concern | M0 decision |
| --- | --- |
| Userinfo | Parsed and preserved only as typed URL component; never emitted in display/log output except as `***@`; password material always redacted. |
| Host validation | ASCII domain labels, IPv4, IPv6 literals, and already-punycode `xn--` labels accepted. Empty host rejected for schemes requiring authority. Non-ASCII host blocked until text/i18n M2 sign-off. |
| Port validation | Decimal 0-65535 only; empty or non-decimal port returns `UrlError::InvalidPort`; default-port normalization is display-only and does not mutate stored URL. |
| Path normalization | Dot-segment normalization helper is explicit; parsing does not silently normalize path semantics. Percent-encoded slash/backslash remains encoded unless caller explicitly decodes bytes. |
| Percent-decoding | Percent helpers return bytes by default. Text conversion uses UTF-8 only where explicitly named; other encodings blocked on text/i18n M1. Invalid percent triplets return `UrlError::InvalidPercentEncoding`. |
| Sensitive query values | Keys matching `token`, `secret`, `password`, `key`, `signature`, `auth`, or user-configured sensitive keys are redacted in observability output. |
| Header redaction | `Authorization`, `Proxy-Authorization`, `Cookie`, `Set-Cookie`, `X-Api-Key`, and configured sensitive headers are redacted by default. |
| Body redaction | Bodies are never logged by default. Size-limited previews require explicit opt-in and text previews remain blocked on text/i18n M1. |
| Certificate redaction | Raw DER, private keys, and full subject/SAN display are not logged; fingerprints and typed verification reason codes are allowed. Unicode certificate display waits for text/i18n M2. |
| Peer address redaction | Peer addresses are logged by default for loopback/server diagnostics but may be redacted by config; redaction must preserve host-family and port-presence evidence. |
| TLS material | Session keys, secrets, tickets, and key material are never exposed in logs/events. |

## Text/I18n Dependency States

| Surface | State | Provider dependency |
| --- | --- | --- |
| TCP/TLS/HTTP protocol bytes | production-substrate | none |
| HTTP text body helpers | blocked-on-text-i18n-m1 | `sifr.encoding` / explicit text I/O |
| Header and cookie non-ASCII/user-text conversion | blocked-on-text-i18n-m1 | text decoding substrate |
| URL non-UTF-8 encodings | blocked-on-text-i18n-m1 | codec lookup and error handlers |
| Unicode/IDNA host canonicalization | blocked-on-text-i18n-m2 | approved Unicode/IDNA version and normalization |
| Text-heavy demos using `open(..., encoding=...)` | blocked-on-text-i18n-m1 | explicit text I/O |
| Locale-sensitive diagnostics/formatting | blocked-on-text-i18n-m3 | locale identifiers and formatters |

## Concurrency/Runtime Dependency States

| Surface | State | Provider dependency |
| --- | --- | --- |
| TCP/TLS/HTTP cancellation and deadlines | blocked-on-concurrency-runtime-m1 | task cancellation/deadline model |
| Stream backpressure and suspension | blocked-on-concurrency-runtime-m2 | bounded sync/backpressure semantics |
| Blocking sync helper offload | blocked-on-concurrency-runtime-m3 | `sifr.runtime.spawn_blocking` and workload diagnostics |
| Process-backed serving scale | blocked-on-concurrency-runtime-m4 / blocked-on-concurrency-runtime-m6 | process runtime and typed IPC gates |
| Graceful shutdown | blocked-on-concurrency-runtime-m5 | signal/shutdown substrate |
| Observability/context propagation | blocked-on-concurrency-runtime-m5 | runtime diagnostics and task/request context |

## Error Taxonomy

| Error | Owner | Required nesting |
| --- | --- | --- |
| `NetError` | M1 | wraps OS/socket timeout/cancel/connect/reset/closed evidence. |
| `DnsError` | M1 | nests in `NetError::Dns`. |
| `TimeoutError`, `CancelledError` | provider + M1-M4 | provider evidence preserved at network/TLS/HTTP layer. |
| `TlsError`, `CertificateError` | M2 | `TlsError::Transport(NetError)` and certificate verification evidence. |
| `UrlError` | M3 | validation, parse, blocked-provider, and size-limit evidence. |
| `HeaderError`, `BodyError`, `ProtocolError`, `HttpError` | M3/M4 | `HttpError::Tls(TlsError)`, `HttpError::Transport(NetError)`, and body/protocol limit evidence. |
| `TooLargeError` | M3/M4 | parser/body size-cap evidence. |

## Implementation Backlog

| Milestone | Traceability artifact | Acceptance |
| --- | --- | --- |
| M1 | `network_http_m1_async_network_traceability.md` | async TCP/DNS substrate, split/half-close, workload diagnostics, no UDP unless M0 is amended. |
| M2 | `network_http_m2_tls_traceability.md` | TLS configs/streams, safe verification, mTLS, close semantics, host/build records. |
| M3 | `network_http_m3_url_header_cookie_traceability.md` | URL, percent, header, and cookie-header primitives with text/i18n blocking states. |
| M4 | `network_http_m4_http_transport_traceability.md` | HTTP/1.1 and HTTP/2 loopback transport, body streaming, HTTPS/ALPN, resource limits. |
| M5 | `network_http_m5_handoff_traceability.md` | docs, demos, dependency snapshots, panic scans, final inventory closure, final review. |
