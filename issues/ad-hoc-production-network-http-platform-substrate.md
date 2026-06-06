# Ad Hoc Phase: Production Network and HTTP Platform Substrate

Status: draft
Phase placement: ad hoc expansion phase after the stdlib boundary refactor and before Phase 41 can claim production readiness for networked programs.
Phase owner: runtime/networking implementation with compiler import, effect, and codegen support

## Objective

Build the production-grade network, TLS, URL, and HTTP substrate required for real Sifr networked programs, Phase 41's FastAPI-like web framework, and a later httpx-like production HTTP client.

This phase does not attempt CPython networking/web stdlib parity. CPython is an evidence source for edge cases, protocol behavior, and explicit rejection decisions; it is not the product shape.

The required output is:

- Sifr-native async TCP networking
- UDP support only where near-term production workloads justify it
- DNS/address resolution
- TLS client/server streams with safe verification defaults
- HTTP/1.1 and HTTP/2 client/server transport substrate
- typed URL, header, and small cookie-header parsing primitives
- streaming request/response bodies
- cancellation, timeouts, backpressure, graceful shutdown, and resource limits
- typed network/TLS/HTTP errors
- compiler diagnostics for blocking I/O in async contexts
- production observability hooks for connection, TLS, and HTTP lifecycle events
- internal loopback test infrastructure to validate the substrate without external network dependency

CPython-shaped modules such as `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib.request`, `sifr.http.client`, `sifr.http.server`, and `sifr.socketserver` are not product goals for this phase.

## Split-Out Phases

The original broad planning scan also covered two important areas that are now tracked as separate ad hoc phases:

- [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md): `queue`, `subprocess`, `asyncio.subprocess`, `concurrent.futures`, `multiprocessing`, `contextlib`, `warnings`, `signal`
- [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md): `codecs`, `encodings`, `unicodedata`, `locale`, `gettext`

This phase may depend on those phases for optional text decoding, subprocess demos, or executor-backed serving, but it must not implement their module surfaces here.

This phase also depends on [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md). Its namespace contract is assumed complete before these substrate milestones ship: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.

## Product Boundary

A module belongs in Sifr core only if it is production substrate or production developer experience.

| Layer | Decision | Public? | Rationale |
| --- | --- | --- | --- |
| Runtime substrate | build now | mostly public low-level API plus private intrinsics | required for all clients, servers, and framework work |
| Protocol substrate | build now | partly public | HTTP/TLS/URL correctness is shared foundation |
| Production HTTP client API | reserve separate phase | public later | httpx-like client behavior is a product surface, not stdlib parity |
| Production server framework | Phase 41 | public later | routing, middleware, extractors, lifecycle, and ops hooks belong there |
| CPython-shaped networking modules | reject/defer | no | maintenance burden without strategic production value |

### Public Surfaces Built Now

| Surface | Classification | Notes |
| --- | --- | --- |
| `sifr.net` | `production-public` | primary low-level network API |
| `sifr.tls` | `production-public` | primary TLS API, including optional client certificate authentication when M0 confirms deterministic fixtures and backend support |
| `sifr.url` | `production-public` | typed URL API |
| `sifr.http` protocol types | `production-substrate` | request/response/header/status/body primitives; final path may be `sifr.http.core` if review prefers a narrower public boundary |
| internal loopback harness | `internal-test` | never a public dev-server module |
| readiness primitives | `production-substrate` | internal or low-level only; no public manual event-loop model |

### Deferred Or Rejected Public Surfaces

| Surface | Decision | Reason |
| --- | --- | --- |
| `sifr.http.server` | rejected as public API | toy/basic server shape; server product is Phase 41 |
| `sifr.socketserver` | rejected | inheritance-heavy handler model conflicts with Sifr's static model |
| `sifr.urllib.request` | deferred/rejected | old opener/handler architecture; production client deserves a modern API |
| `sifr.http.client` | deferred | low-level client transport may exist internally; public API should be httpx-like |
| `sifr.socket` | deferred | CPython descriptor-shaped API must not define Sifr networking |
| `sifr.ssl` | deferred | TLS is exposed through `sifr.tls`, not `SSLContext` mimicry |
| `sifr.select` / `sifr.selectors` | internal readiness only | users should use async streams, not manual event loops |
| `sifr.urllib.parse` | deferred adapter | stable URL utility is `sifr.url` |
| `urllib.robotparser` | deferred/rejected | niche utility, not core platform substrate |
| `http.cookiejar` | deferred | cookie persistence belongs in the HTTP client phase if needed |
| HTTP/3 / QUIC | deferred | revisit in a future transport phase after QUIC runtime strategy is designed |
| CGI-style serving | rejected | legacy serving model |
| `ThreadingMixIn` / `ForkingMixIn` | rejected | wrong abstraction and overlaps concurrency/runtime phases |
| raw event-loop policies | rejected | Phase 32 keeps raw event loops out of the user model |

## Compatibility Policy

CPython-shaped networking modules are not part of the production baseline.

They may be considered later only when all of the following are true:

1. The Sifr-native production API already exists.
2. There is evidence of real migration demand.
3. The adapter delegates to production primitives.
4. The adapter does not expose legacy, dynamic, blocking, descriptor-aliasing, or unsafe semantics as recommended API.
5. The adapter does not increase maintenance burden disproportionate to usage.

Until then, these surfaces are deferred or unsupported. This phase must not add compatibility shims, fallback paths, bridge aliases, legacy aliases, deprecated behavior, or partial public modules.

Bare CPython imports such as `from socket import socket`, `import ssl`, and `from urllib.parse import urlparse` remain unsupported by the namespace contract unless a normal user/package module resolves first. The production imports for this phase are Sifr-native forms such as `from sifr.net import connect_tcp`, `from sifr.tls import TlsClientConfig`, and `from sifr.url import Url`.

## No-Toy-Module Gate

A public module must not be added unless it satisfies at least one of:

1. It is necessary production substrate.
2. It is the recommended production developer API.
3. It is a stable, broadly useful utility with low long-term maintenance cost.
4. It is required for Phase 41 or the production HTTP client phase.

The following are not sufficient reasons:

- CPython has the module.
- It helps a small compatibility demo.
- It is useful for local experiments only.
- It is easy to implement partially.
- It can be marked as basic and fixed later.

Partial public modules are rejected unless they are explicitly unstable/internal and inaccessible as stable user API.

## Maintenance Burden Test

For every public network/web API, answer:

- Would a production team reasonably use this directly?
- Will this still be a recommended API after Phase 41 and the production HTTP client phase?
- Does it compose with typed errors, async, ownership, and cancellation?
- Does it avoid global mutable process state?
- Does it avoid dynamic monkeypatching, subclass tricks, descriptor aliasing, and raw event-loop policies?
- Can it be documented as safe by default?
- Can it be tested deterministically without external network dependency?
- Is it worth supporting for years?

If the answer is no, the API is internal, deferred, or rejected.

## Cross-Phase Dependency Contract

The split phases are not an implied ship order. Each phase may implement and test its self-contained binary/runtime subset independently, but cross-phase consumer features are blocked until their provider phase is complete:

- Text/i18n is a hard prerequisite for non-UTF-8 HTTP body decoding, URL percent-encoding variants that require codec lookup, Unicode/IDNA host canonicalization if M0 cannot align it with the text Unicode version, file/text handlers, text-heavy demos, and any network demo that depends on `open(..., encoding=...)`.
- The precise unblock point for encoding-dependent network features is completion of text/i18n `milestone_text_i18n_1: Encoding And Explicit Text I/O`; this phase records those features as `blocked-on-text-i18n-m1` until that milestone is closed.
- Unicode-data-dependent network features wait for text/i18n `milestone_text_i18n_2: Unicode Core` or `milestone_text_i18n_2_5: Unicode Segmentation` when they require Unicode normalization, case folding, IDNA behavior, display boundaries, or user-visible Unicode property decisions.
- Locale-sensitive network logging, formatting, or diagnostics wait for text/i18n `milestone_text_i18n_3: Locale Identifiers And Locale-Sensitive Formatting`; this phase must not introduce locale-derived defaults.
- Concurrency/runtime is a hard prerequisite for executor-backed serving APIs. This phase does not implement public thread, executor, queue, process, warning, or signal modules.
- Async scheduler/task primitives are prior runtime infrastructure owned by the existing async model. This phase consumes that runtime and adds only network-specific stream, TLS, and HTTP suspension points.
- Phase 41 consumes this phase for server framework routing, middleware, lifecycle/shutdown, typed extractors, validation, error mapping, and operational hooks.
- A separate production HTTP client phase consumes this phase for connection pooling, redirects, retry policy, auth, cookies, proxies, streaming upload/download, JSON helpers, multipart, compression, test transports, and sync/async product design.

### Text/I18n Dependency Decisions

This phase is allowed to ship binary and protocol-correct network substrate before the text/i18n phase. It must not implement local encoding fallbacks, duplicate codec registries, or silently coerce bytes into UTF-8 text to avoid the dependency.

| Network/HTTP surface | Status before text/i18n M1 | Text/i18n dependency | Decision |
| --- | --- | --- | --- |
| TCP, UDP if accepted, DNS socket transport, TLS handshakes, HTTP/1.1 and HTTP/2 framing | `production-substrate` | none | Protocol bytes and transport state are independent of text decoding. |
| HTTP request/response bodies | binary streaming only | M1 for charset decoding | Bodies are `Bytes`/stream values in this phase. `body.text(...)`, charset-aware decoding, and text body previews are blocked on M1. |
| HTTP headers | validated protocol representation | M1 only for non-ASCII/user-text conversion | Header names use HTTP token/ASCII validation. Header values remain typed protocol values or byte/ASCII-safe values; decoding arbitrary header bytes to Sifr text is blocked on M1. |
| `Content-Type` and `charset` parameters | parse ASCII parameter names/labels only | M1 for actual charset decoding | This phase may parse and preserve charset labels, but must not decode payloads locally. Unsupported labels remain typed errors or blocked records. |
| URL parsing/building | ASCII, already-valid Sifr text, UTF-8 percent behavior where no codec lookup is required | M1 for non-UTF-8 encoding labels; M2 for Unicode/IDNA alignment if needed | `sifr.url` may parse/build typed URLs over valid Sifr strings and bytes. Non-UTF-8 `encoding=` behavior and codec-label lookup are blocked on M1. Unicode host canonicalization/IDNA must either use a text/i18n-approved Unicode version or remain `blocked-on-text-i18n-m2`. |
| Percent encoding/decoding | byte/ASCII/UTF-8 safe operations | M1 for named encodings and error handlers | Percent helpers that take raw bytes are allowed. Helpers that accept `encoding=` or `errors=` must call the text substrate after M1. |
| Query/form helpers | byte/ASCII/UTF-8 safe parsing only | M1 for non-UTF-8 form decoding | `application/x-www-form-urlencoded` charset variants, dynamic `encoding=`, and text error handlers are blocked on M1. Multipart/form parsing is reserved for the future HTTP client/framework phases unless M0 accepts substrate-only parsing. |
| Cookie header parsing | header-level syntax only | M1 for percent-decoded/user-text values | Cookie names/values may remain typed header strings/bytes. Percent-decoded or non-UTF-8 cookie text is blocked on M1. Cookie persistence remains outside this phase. |
| TLS certificate verification | allowed | none | Verification is owned by `rustls`/TLS substrate and does not need text/i18n. |
| TLS certificate field inspection/display | typed raw/DER or ASCII-safe fields only | M1/M2 for decoded display names and Unicode normalization | User-visible certificate subject/issuer/SAN display must not invent local string decoding. M2 gates Unicode normalization/IDNA-sensitive display behavior. |
| Error messages and diagnostics | static ASCII diagnostic templates with typed evidence | M1/M3 for decoded remote text or locale-sensitive formatting | Errors carry typed bytes/labels/status evidence. No locale-sensitive formatting or decoded body/header snippets before the provider milestones. |
| Observability hooks | structured events with typed fields | M1 for decoded previews; M3 for locale-sensitive formatting | Trace/metric labels use stable ASCII keys and typed values. Body/header previews that decode text are blocked on M1. |
| Demos and fixtures | binary loopback and ASCII/UTF-8-only demos | M1 for `open(..., encoding=...)` or non-UTF-8 demos | Demos must not require text-mode file I/O or local codec workarounds before M1. |
| Phase 41 web framework handoff | binary request/response pipeline, routing substrate, and typed protocol metadata | M1 for text body/extractor decoding; M2 for Unicode path/host normalization if accepted; M3 for locale-sensitive formatting | Framework extractors such as text bodies, decoded forms, JSON text helpers, and decoded path/query values must consume text/i18n provider APIs. Binary routing and protocol metadata can proceed on this substrate. |
| Production HTTP client handoff | pooling, timeouts, retries, TLS, redirects over typed URLs, binary streaming | M1 for text response helpers, charset decoding, form helpers, and decoded cookie/header values; M2 for Unicode/IDNA alignment; M3 for locale-sensitive diagnostics | Client features must not add local charset detection/decoding or duplicate URL Unicode behavior. Text helpers wait for the text/i18n provider. |

M0 must add these decisions to the inventory and assign every text-dependent surface one of:

- `production-substrate`
- `blocked-on-text-i18n-m1`
- `blocked-on-text-i18n-m2`
- `blocked-on-text-i18n-m2_5`
- `blocked-on-text-i18n-m3`
- `deferred-to-http-client-phase`
- `deferred-to-phase-41`
- `rejected`

When a text-dependent feature is unblocked, it must call `sifr.encoding`, `sifr.unicode`, `sifr.io`, or `sifr.i18n` from the text/i18n phase. It must not introduce a second registry, handler table, locale default, Unicode table, or fallback decoder.

## Evidence Sources

The authoritative CPython source tree for evidence scans is:

- `/Users/yaseralnajjar/work/sifr/cpython`

The implementation must scan these CPython files during M0 and before any milestone that reuses their behavior. The scan is used to find protocol edge cases, platform caveats, test fixture ideas, deprecated/legacy traps, and explicit rejection evidence. It is not a parity backlog.

| Domain | CPython library/docs sources | CPython test sources | Native backing evidence |
| --- | --- | --- | --- |
| sockets/readiness | `Lib/socket.py`, `Lib/selectors.py`, `Doc/library/socket.rst`, `Doc/library/selectors.rst`, `Doc/library/select.rst` | `Lib/test/test_socket.py`, `Lib/test/test_select.py`, `Lib/test/test_selectors.py`, `Lib/test/test_asyncio/test_streams.py`, `Lib/test/test_asyncio/test_server.py`, `Lib/test/test_asyncio/test_sock_lowlevel.py`, `Lib/test/test_asyncio/test_selector_events.py` | `Modules/socketmodule.c`, `Modules/selectmodule.c`, `Modules/clinic/socketmodule.c.h`, `Modules/clinic/selectmodule.c.h` |
| TLS | `Lib/ssl.py`, `Doc/library/ssl.rst` | `Lib/test/test_ssl.py`, `Lib/test/test_asyncio/test_ssl.py`, `Lib/test/test_asyncio/test_sslproto.py` | `Modules/_ssl.c`, `Modules/_ssl/*`, `Modules/clinic/_ssl.c.h` |
| HTTP and URLs | `Lib/http/*.py`, `Lib/urllib/*.py`, `Lib/socketserver.py`, `Doc/library/http*.rst`, `Doc/library/urllib*.rst`, `Doc/library/socketserver.rst`, HTTP/2 and HPACK protocol specs | `Lib/test/test_httplib.py`, `Lib/test/test_httpservers.py`, `Lib/test/test_http_cookies.py`, `Lib/test/test_http_cookiejar.py`, `Lib/test/test_socketserver.py`, `Lib/test/test_urllib.py`, `Lib/test/test_urllib2.py`, `Lib/test/test_urllib2_localnet.py`, `Lib/test/test_urllib_response.py`, `Lib/test/test_urllibnet.py`, `Lib/test/test_urllib2net.py` | Rust HTTP/TLS/runtime crates selected by this phase |

Each reviewed CPython test family must end in exactly one state:

- `mined`: behavior converted into a Sifr-native substrate test.
- `blocked`: behavior depends on a split-out phase.
- `rejected`: behavior belongs to a CPython-shaped, legacy, unsafe, or non-product surface.
- `external-signal`: test is retained only as non-blocking ecosystem signal because it depends on external network state.

Every proposed public surface must end in exactly one state:

- `production-public`
- `production-substrate`
- `internal-test`
- `deferred`
- `rejected`
- `blocked-on-text-i18n-m1`
- `blocked-on-concurrency-runtime`
- `host-limited`

`open` is allowed during implementation only and is forbidden at phase exit.

## Current Sifr Baseline

Current Sifr stdlib support is intentionally curated under `lib/sifr/*.sifr`. Relevant existing surfaces:

- `sifr.asyncio` is a veneer over the canonical task model, but intentionally omits raw event loops, public selectors, subprocesses, process pools, and transport/protocol APIs.
- `sifr.io` has file handles and in-memory stream wrappers, but no socket streams.
- `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib`, `sifr.socketserver`, and public web-server modules are not present as production surfaces and should remain absent in this phase.
- `sifr.net`, `sifr.tls`, `sifr.url`, and `sifr.http` protocol/runtime boundaries need to be created or confirmed during M0.

The Phase 32 async model remains binding:

- Native async I/O APIs must be real suspension points.
- Sync network APIs that can block must be classified as `@blocking_io`.
- Direct calls to blocking sync APIs from `async def` remain compiler errors unless routed through native async APIs or explicit offload.
- The compiler must not expose Tokio, event-loop objects, callback transports/protocols, or raw selector internals as the normal user model.

## Milestone Dependency Graph

Implementation PRs must follow this dependency order unless the execution ledger records an explicit split that preserves the same prerequisites:

1. `milestone_network_http_0` first. No implementation milestone starts until public/internal/deferred/rejected surface classification, error taxonomy, runtime dependency plan, workload classifications, and Phase 41 handoff contract are checked in.
2. `milestone_network_http_1` before TLS and HTTP transport. Async streams are the substrate for TLS and HTTP.
3. `milestone_network_http_2` before HTTPS transport. Plain HTTP parser work may start after M1, but HTTPS waits for M2 `AsyncTlsStream`.
4. `milestone_network_http_3` before M4 HTTP integration where URL/header/cookie parsing is consumed.
5. `milestone_network_http_4` before M5 handoff.
6. `milestone_network_http_5` last, after every proposed surface and CPython evidence family in this phase is closed.

Parallel work is allowed only for pure parser work that does not consume unfinished runtime substrate.

## Architecture Principles

### Native Runtime First

Implement the canonical runtime primitive first. Do not layer CPython-shaped public modules over it in this phase. Use production Rust ecosystem crates for protocol and transport machinery by default; do not hand-roll networking, TLS, DNS, URL, HTTP, HPACK, or observability infrastructure unless M0 records a concrete rejection finding for every suitable ecosystem option.

- Tokio remains the backing async runtime because the generated task runtime already depends on `tokio` and `sifr_stdlib::StdlibFeature::Tokio`.
- M0 must expand the Tokio dependency feature plan from the current task/sync/time set to the concrete features needed for `tokio::net` and `tokio::io`.
- No `async-std`, custom event-loop runtime, or public Tokio type is introduced without a separate architecture issue.
- `sifr.net` owns TCP, UDP if accepted, DNS/address resolution, stream readiness, timeouts, cancellation, backpressure, shutdown, and connection lifecycle.
- `sifr.tls` owns TLS configuration, handshakes, certificate verification, SNI, ALPN, wrapped streams, and TLS errors.
- `sifr.url` owns URL parsing/building, percent encoding, query handling, and authority/host/port validation.
- `sifr.http` protocol/runtime internals own request/response transport, headers, body streaming, parser/encoder limits, connection reuse, and HTTP protocol errors.

The exact internal module names may change during implementation, but the boundary must exist: public Sifr-native modules must not duplicate target-runtime logic, and internal protocol/test harnesses must not leak as stable user API.

### Rust Ecosystem First

This phase builds a Sifr platform, not a new Rust networking stack. The implementation should wrap and constrain mature Rust crates, then add Sifr-specific type surfaces, ownership semantics, diagnostics, and panic-free error mapping.

Default ecosystem stack:

| Area | Preferred crates | Role |
| --- | --- | --- |
| Async runtime and I/O | `tokio`, `tokio-util`, `bytes` | task runtime, TCP/UDP, timers, cancellation, async read/write traits, buffer primitives |
| Socket options | `socket2` | low-level options only when Tokio/std do not expose required production behavior |
| DNS | `tokio::net::lookup_host` and/or `hickory-resolver` | system resolver integration and async DNS when M0 accepts its semantics |
| TLS | `rustls`, `tokio-rustls`, `rustls-platform-verifier`, `webpki-roots`, `rustls-pemfile` | TLS config, async streams, certificate verification, roots, PEM parsing |
| Certificate inspection | `x509-parser` (conditional) | raw certificate field access only if M0/M2 accepts a production need; must pass malformed-DER, oversized-field, and hostile-chain panic/unsafe audit before merge |
| Test certificates | `rcgen` | deterministic local CA/self-signed fixtures only |
| HTTP types and bodies | `http`, `http-body`, `http-body-util`, `bytes` | method/status/header/request/response/body abstractions |
| HTTP/1 and HTTP/2 transport | `hyper`, `hyper-util`, `h2` | production HTTP transport, HTTP/2 state machine, flow control, multiplexing, protocol errors |
| URL and percent encoding | `url`, `percent-encoding` | WHATWG/RFC URL parsing, building, and escaping |
| Cookies | `cookie` | header-level cookie parsing; jar/persistence features remain out of scope unless the HTTP client phase accepts them |
| Middleware/service substrate | `tower` | optional internal service abstraction when Phase 41 handoff needs composable request/response plumbing |
| Observability | `tracing`, `metrics`, optional OpenTelemetry bridge crates | spans, structured events, counters, histograms, exporter-neutral hooks |
| Tests and conformance | `tokio-test`, `proptest`, `h2spec`/HTTP/2 conformance fixtures where available | deterministic async tests, parser/property tests, protocol conformance |

M0 must produce a dependency decision record for every crate family above. Each decision must include:

- accepted crate and feature flags, or explicit rejection rationale
- the exact Sifr abstraction that hides the crate from public APIs
- panic/unsafe audit notes for user-controlled data paths
- typed error mapping into Sifr error variants
- license, MSRV, binary-size, platform, and build-feature impact
- deterministic local test strategy
- conformance evidence for protocol crates
- supply-chain ownership/maintenance signal

No public API may expose `tokio`, `hyper`, `h2`, `rustls`, `url`, `tower`, `tracing`, or other crate-specific types directly. The only stable user-facing contract is Sifr's typed API.

From-scratch implementation is allowed only for:

- thin Sifr wrappers/adapters around accepted crates
- compiler diagnostics and workload classifications
- Sifr ownership/cancellation/resource-limit enforcement
- small deterministic fixtures where no production behavior is exposed

From-scratch protocol parsing, TLS verification, DNS resolution, URL parsing, HPACK, HTTP/2 state machines, or metrics/tracing backends are rejected unless M0 records that no suitable Rust ecosystem crate satisfies the safety, maintenance, and production requirements.

### Sifr-Native Network API Shape

M0 must finalize exact names, but the target shape is Sifr-native and typed:

- `async connect_tcp(address, *, timeout=None, local_addr=None) -> Result[TcpStream, NetError]`
- `async listen_tcp(address, *, backlog=None, reuse_addr=false) -> Result[TcpListener, NetError]`
- `async TcpListener.accept() -> Result[(TcpStream, SocketAddr), NetError]`
- `async TcpStream.read(buffer) -> Result[usize, NetError]`
- `async TcpStream.write(bytes) -> Result[usize, NetError]`
- `async TcpStream.close() -> Result[None, NetError]`
- `UdpSocket` support only if M0 accepts concrete production workloads and deterministic tests.

The API must expose local/remote address inspection, graceful shutdown, resource-limit controls, and deterministic cancellation semantics. It must not expose descriptor aliasing, monkeypatchable globals, or public raw event-loop policies.

The `read(buffer)` shape above is illustrative, not accepted design. M0 must choose the stream I/O ownership model before M1 starts:

- mutable-borrow buffer reads,
- owned-buffer reads such as `read(max_bytes) -> Result[Bytes, NetError]`,
- async iterator / chunk stream reads,
- or a deliberately combined model with explicit ownership and lifetime rules.

The decision must cover generated Rust lifetimes, cancellation safety, partial read/write behavior, backpressure, TLS wrapping, HTTP body streaming, and panic-free handling of zero-length and too-large buffers.

### TLS API Shape

M0 must finalize exact names, but the target shape is:

- `TlsClientConfig` and `TlsServerConfig`
- safe default certificate verification
- root strategy selected explicitly for production binaries and deterministic local tests
- SNI and ALPN support
- optional client certificate authentication when M0 accepts deterministic fixtures and backend support
- async TLS client and server streams over `TcpStream`
- typed certificate and TLS errors preserving underlying network evidence

No CPython-shaped `SSLContext` or `SSLSocket` is exposed in this phase. If a future adapter is accepted, it must consume/move underlying stream handles and delegate to `sifr.tls`.

### HTTP Substrate Shape

This phase builds HTTP transport substrate, not the final public web framework or HTTP client product.

Required substrate:

- HTTP/1.1 parser/encoder
- HTTP/2 frame codec, HPACK header compression, stream state machine, SETTINGS negotiation, flow control, PING, RST_STREAM, and GOAWAY handling
- typed request/response model
- method, status, version, header, and body types
- streaming request and response bodies
- content-length validation
- chunked transfer handling
- keep-alive and connection lifecycle
- multiplexed HTTP/2 request/response body streams with backpressure and cancellation
- ALPN-driven protocol selection for TLS connections
- request/response size limits
- malformed protocol typed errors
- internal loopback client/server transport harness
- upgrade hooks may be reserved but not exposed as a partial public API unless production use is defined

Server framework behavior such as routing, middleware, extractors, validation, generated docs, and deployment ergonomics belongs to Phase 41. Client behavior such as pooling, redirects, retries, auth, proxies, cookie persistence, JSON helpers, multipart, compression, and test transports belongs to the separate production HTTP client phase.

### Async Counterpart Rule

Any blocking production API added in this phase must have one of:

- a native async counterpart,
- a documented reason why async is not meaningful,
- or an explicit `@blocking_io` classification plus approved offload-only guidance.

Required native async counterparts:

- TCP connect/listen/accept/read/write/close
- DNS/address resolution where the operation can block
- TLS client/server handshake/read/write/close
- HTTP request/response body streaming
- HTTP server accept/dispatch/shutdown substrate

M0 must add every network/web API to the stdlib workload database. The first table must include at least:

| API family | Classification | Async-context behavior |
| --- | --- | --- |
| `sifr.net` async TCP, DNS, and readiness operations | async-native | legal suspension points |
| `sifr.tls` async handshake/read/write/shutdown | async-native | legal suspension points |
| `sifr.http` async transport and streaming body operations | async-native | legal suspension points |
| accepted sync network/TLS/HTTP helpers | sync `@blocking_io` | compile-time diagnostic suggesting native async APIs or explicit offload |
| pure URL, header, and cookie-header parsing | pure, or `@cpu_heavy` if M0 finds size-dependent paths | legal unless the exact path is marked `@cpu_heavy` |
| rejected CPython-shaped blocking APIs | unsupported/deferred | namespace or unsupported-surface diagnostic |

Implementation milestones cannot claim completion for a blocking family until its workload entries and async diagnostics are checked in.

### No Raw Event Loop As Public Model

CPython `asyncio` tests may be mined for scheduling and transport edge cases, but Sifr must not make raw event loops, event-loop policies, callback transports/protocols, or public selector internals the primary API. Public APIs map to task, stream, TLS, and HTTP primitives.

### Typed Errors Instead Of Exceptions

All fallible APIs must expose typed error results:

- `NetError`
- `DnsError`
- `ConnectError`
- `TimeoutError`
- `TlsError`
- `CertificateError`
- `HttpError`
- `ProtocolError`
- `HeaderError`
- `BodyError`
- `TooLargeError`
- `CancelledError`

`milestone_network_http_0` must add a shared error mapping document before M1 implementation:

- map CPython `OSError`/`errno`, TLS, URL, and HTTP error evidence into stable Sifr variants
- define a concrete typed error hierarchy before M1 starts
- add cross-module regression tests proving equivalent failures use the same Sifr error family
- preserve nested evidence when higher layers fail because of lower layers, for example `HttpError::Tls(TlsError::Transport(NetError))`
- reject exception-only control flow and legacy aliases

### Panic-Free Runtime Contract

No user-triggerable runtime panics are allowed. Generated Rust for these APIs must not contain data-dependent `.unwrap()`, `.expect()`, or `panic!` on user-controlled network, TLS, URL, cookie, header, or HTTP data.

### Production Observability Hooks

The substrate must expose enough structured hooks for Phase 41 and the HTTP client phase:

- request IDs where HTTP transport creates request/response contexts
- structured access-log events for internal transport harnesses and Phase 41 consumers
- trace spans for DNS, connect, TLS handshake, request write, response read, and server dispatch
- timeout and cancellation classification
- connection lifecycle events
- TLS handshake diagnostics without leaking secrets
- HTTP status/error metrics hooks
- graceful shutdown visibility

The hooks must be deterministic, typed, and optional. They must not require global mutable state.

## Non-Goals And Permanent Boundaries

The following are not accepted as silent omissions. They must be explicitly classified in M0 and either rejected, deferred, blocked, host-limited, or internal-only:

- public `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib.*`, `sifr.http.client`, `sifr.http.server`, or `sifr.socketserver`
- CPython refcount/finalizer behavior
- dynamic monkeypatching of module globals
- raw event-loop policy mutation
- HTTP/3 / QUIC as this phase's implemented protocol version; it is deferred with a revisit rule
- callback transport/protocol APIs as the primary Sifr model
- descriptor aliasing APIs such as `detach`, `fromfd`, and `dup` as public Sifr behavior
- `socketserver.ThreadingMixIn` and `socketserver.ForkingMixIn`
- `http.server.ThreadingHTTPServer`
- `CGIHTTPRequestHandler`-style behavior
- public compatibility web servers or local toy HTTP fixtures
- process, queue, signal, warning, locale, codec, Unicode, or gettext APIs; those belong to the split-out phases

## Milestones

### milestone_network_http_0: Product Boundary And Architecture

Scope:

- Define public, internal, deferred, rejected, blocked, and host-limited API surfaces.
- Remove CPython stdlib parity as a completion goal.
- Define typed network/TLS/URL/HTTP error model.
- Define async/blocking workload classifications and diagnostics.
- Define runtime dependency features and approved Rust crates.
- Define the Rust ecosystem dependency stack and feature flags for network, DNS, TLS, URL, HTTP/1, HTTP/2, cookies, observability, and tests.
- Record explicit rejection rationale before implementing any protocol/domain component from scratch.
- Define HTTP client/server substrate boundaries.
- Define protocol version scope, including HTTP/1.1, HTTP/2, and explicit HTTP/3 deferral entries.
- Define buffer ownership and API pattern for stream I/O before M1 backlog entries are finalized.
- Define the complete text/i18n dependency inventory for URL, headers, bodies, cookies, certificate display, observability, diagnostics, demos, Phase 41 handoff, and HTTP client handoff.
- Define Phase 41 handoff contract.
- Define the separate production HTTP client phase handoff contract.
- Scan every CPython source/test/doc file listed in `Evidence Sources`.
- Create evidence docs for sockets/readiness, TLS, URLs, and HTTP showing which behavior was mined, rejected, blocked, or retained as external signal.

Validation:

- classification artifact proves no proposed public/internal/deferred/rejected surface lacks a state
- evidence scan proves every listed CPython test family was reviewed
- `cargo test -p sifr_stdlib`
- `cargo test -p sifr -- stdlib`
- `scripts/run_all_tests.sh --profile create-pr`

Definition of done:

- Every proposed surface is classified as `production-public`, `production-substrate`, `internal-test`, `deferred`, `rejected`, `blocked-on-text-i18n-m1`, `blocked-on-concurrency-runtime`, or `host-limited`.
- Every text-dependent surface is classified as `production-substrate`, `blocked-on-text-i18n-m1`, `blocked-on-text-i18n-m2`, `blocked-on-text-i18n-m2_5`, `blocked-on-text-i18n-m3`, `deferred-to-http-client-phase`, `deferred-to-phase-41`, or `rejected`.
- No module is accepted merely because CPython has it.
- Dependency decision records are present and checked in for every crate family in the Rust Ecosystem First table, covering accepted crate and feature flags, Sifr abstraction that hides the crate from public APIs, panic/unsafe audit for user-controlled data paths, typed error mapping into Sifr variants, license/MSRV/binary-size/platform impact, deterministic local test strategy, conformance evidence for protocol crates, and supply-chain/maintenance signal.
- Stream I/O ownership, lifetime, cancellation, and partial read/write semantics are decided before M1 starts.
- M1-M5 implementation PRs have concrete backlog entries rather than prose-only scope.

### milestone_network_http_1: Async Network Runtime

Scope:

- Implement `sifr.net` as the primary low-level networking API.
- Add TCP client/server streams:
  - async connect
  - async listen
  - async accept
  - async read/write
  - close and graceful shutdown
  - local/remote address inspection
- Add DNS/address resolution with typed errors and deterministic timeout behavior.
- Add cancellation, backpressure, and resource limits.
- Add UDP only if M0 records near-term production use and deterministic loopback tests.
- Add internal readiness primitives without exposing public selector/event-loop APIs.
- Mark accepted sync helpers as `@blocking_io`; reject direct calls from async contexts.

CPython evidence to mine:

- `Lib/test/test_socket.py`
- `Lib/test/test_select.py`
- `Lib/test/test_selectors.py`
- `Lib/test/test_asyncio/test_streams.py`
- `Lib/test/test_asyncio/test_server.py`
- `Lib/test/test_asyncio/test_sock_lowlevel.py`
- `Lib/test/test_asyncio/test_selector_events.py`

Rust/runtime stack:

- `tokio::net`
- `tokio::io`
- `tokio-util` and `bytes` when M0 accepts their buffer/framing role
- `socket2` only if low-level socket option coverage is accepted by M0
- `hickory-resolver` only if M0 accepts in-process async DNS semantics over or alongside system resolver behavior

Definition of done:

- TCP loopback tests pass deterministically without external network dependency.
- UDP loopback tests pass if UDP is accepted.
- Timeout and cancellation behavior is deterministic, typed, and panic-free.
- Blocking sync paths are rejected from async contexts.
- No public API leaks Tokio, raw descriptors, selectors, or event-loop internals.

### milestone_network_http_2: TLS Runtime

Scope:

- Implement `sifr.tls` as the primary TLS API.
- Add `TlsClientConfig` and `TlsServerConfig`.
- Add safe default certificate verification.
- Add deterministic root strategy for local tests and production binaries.
- Add SNI and ALPN.
- Add optional client certificate authentication or record a concrete deferral with deterministic revisit criteria if M0 rejects it.
- Add async TLS client streams.
- Add async TLS server streams.
- Add typed TLS and certificate errors.
- Preserve nested network evidence inside TLS errors.
- Reject CPython-shaped `SSLContext`, `SSLSocket`, and readiness retry errors as public surfaces.

CPython evidence to mine:

- `Lib/test/test_ssl.py`
- `Lib/test/test_asyncio/test_ssl.py`
- `Lib/test/test_asyncio/test_sslproto.py`

Rust/runtime stack:

- `rustls`
- `tokio-rustls`
- `rustls-platform-verifier` and/or `webpki-roots`, selected explicitly
- `rustls-pemfile`
- `rcgen` for deterministic local certificate fixtures
- `x509-parser` only if certificate inspection requires it

Definition of done:

- Local self-signed and CA-backed handshake fixtures are deterministic.
- HTTPS-ready TLS loopback tests pass.
- Invalid certificate tests produce typed errors.
- Safe verification is default.
- TLS verification failures never panic and never silently downgrade verification.

### milestone_network_http_3: URL, Header, And Cookie Primitives

Scope:

- Implement `sifr.url` as a typed URL API:
  - `Url`
  - `UrlQuery`
  - parse/build APIs
  - percent encode/decode
  - path normalization helpers
  - authority/host/port parsing
  - query parsing/building
- Implement HTTP header representation and validation primitives.
- Implement small cookie header parsing required by real HTTP request/response handling.
- Keep cookie persistence and jar policy out of this phase.
- Record non-UTF-8 codec-dependent behavior as `blocked-on-text-i18n-m1`; do not duplicate codec registry behavior locally.
- Record Unicode/IDNA host behavior as `production-substrate` only if M0 proves the selected URL crate's Unicode version and behavior align with text/i18n M2; otherwise keep Unicode host canonicalization `blocked-on-text-i18n-m2` and allow only ASCII/already-punycode host behavior before M2.
- Record query/form/cookie text decoding behavior with the Text/I18n Dependency Decisions matrix.

CPython evidence to mine:

- `Lib/test/test_urllib.py`
- `Lib/test/test_urllib2.py`
- `Lib/test/test_urllib_response.py`
- `Lib/test/test_http_cookies.py`
- `Lib/test/test_http_cookiejar.py` for rejection/defer evidence around persistence

Rust/runtime stack:

- `url`
- `percent-encoding`
- `http` header/status/method types if accepted by M0
- `cookie` for header-level cookie parsing only

Definition of done:

- URL parsing has CPython-derived and RFC-derived edge-case fixtures.
- Invalid input returns typed errors.
- Parser behavior needed by the HTTP substrate, Phase 41, and the HTTP client phase is covered.
- Non-UTF-8 codec behavior is blocked on text/i18n rather than reimplemented.
- Unicode/IDNA behavior is either aligned with text/i18n M2 or blocked on text/i18n M2 with regression fixtures for ASCII and already-punycode host behavior.
- Cookie persistence is not exposed as a partial core API.

### milestone_network_http_4: HTTP Core Transport

Scope:

- Implement HTTP/1.1 parser/encoder.
- Implement typed request/response model.
- Implement method, status, version, headers, and body types.
- Implement body streaming without unbounded buffering.
- Implement content-length and chunked transfer handling.
- Implement keep-alive and connection lifecycle.
- Implement request/response limits.
- Implement malformed protocol typed errors.
- Implement internal loopback client/server transport harness.
- Implement async server accept/dispatch/shutdown substrate over M1 async streams and M2 async TLS for HTTPS.
- Keep Phase 41 routing/middleware/extractors out of this phase.
- Keep production HTTP client features out of this phase except for the internal transport needed to validate the protocol.
- Keep text body decoding, decoded header/body previews, locale-sensitive diagnostics, JSON helpers, multipart/form helpers, and charset-aware client/framework helpers blocked on their provider phases according to the Text/I18n Dependency Decisions matrix.

CPython evidence to mine:

- `Lib/test/test_httplib.py`
- `Lib/test/test_httpservers.py`
- `Lib/test/test_socketserver.py`
- `Lib/test/test_urllib2_localnet.py`
- `Lib/test/test_urllibnet.py` and `Lib/test/test_urllib2net.py` as external-network, non-blocking signal unless converted to loopback
- HTTP/2 and HPACK protocol conformance cases selected during M0

Rust/runtime stack:

- `http`
- `http-body`
- `http-body-util`
- `bytes`
- `hyper`
- `hyper-util`
- `h2`
- `tower` only if M0 accepts it as internal handoff substrate for Phase 41
- avoid pulling a web framework into the substrate

Definition of done:

- HTTP/1.1 and HTTP/2 loopback client/server transport tests pass without external network.
- HTTPS transport works through M2 TLS, including ALPN selection for HTTP/2.
- Malformed HTTP tests produce typed protocol errors.
- Body streaming and HTTP/2 multiplexing work without unbounded buffering.
- HTTP transport stores and forwards binary bodies and typed protocol metadata without local text decoding fallbacks.
- No `http.server`, `socketserver`, or handler-subclass public API is added.

### milestone_network_http_5: Integration, Documentation, And Production Handoff

Scope:

- Update public docs for:
  - `sifr.net`
  - `sifr.tls`
  - `sifr.url`
  - public HTTP protocol/substrate types that M0 accepts
  - rejected/deferred CPython-shaped surfaces and why they are not recommended APIs
  - text/i18n dependency decisions and blocked surface states
- Update internal architecture docs for:
  - runtime networking/TLS/HTTP boundaries
  - stdlib feature/dependency manifest
  - async counterpart policy
  - host-limited platform behavior
  - observability hooks
  - Phase 41 handoff contract
  - production HTTP client phase handoff contract
- Add demos:
  - TCP echo server/client
  - TLS client/server loopback
  - HTTP transport loopback
- Add generated Cargo dependency snapshots for all new feature combinations.
- Add panic-scan and emitted-code quality checks for network/TLS/URL/HTTP paths.
- Update validation lane manifests with representative fixtures.
- Close the inventory:
  - every proposed surface has a terminal state
  - every CPython evidence family has `mined`, `blocked`, `rejected`, or `external-signal` evidence
  - every rejection/defer decision has rationale and revisit rule
  - every host-limited surface records the supported host matrix
- Run an external review loop on the final inventory and close any blocking finding before phase completion.
- External review owner is the runtime/networking phase owner plus the designated compiler/runtime reviewer recorded in the execution ledger. If review output is unavailable for five working days after the review artifact is posted, the phase owner may proceed only by recording the attempted review, open questions, and a conservative self-review in the ledger.

Validation:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- file-size guardrail
- `cargo test -p sifr_stdlib`
- `cargo test -p sifr -- stdlib`
- `scripts/run_e2e_pass.sh`
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`

Definition of done:

- Every proposed surface and CPython evidence family in the phase inventory is closed.
- Phase 41 can build routing, middleware, lifecycle, request/response pipeline, typed extractors, validation, and production hooks on the substrate.
- The production HTTP client phase can build pooling, timeouts, redirects, retries, auth, cookies, proxies, and streaming on the substrate.
- Text/i18n consumers have no duplicated codec registry, Unicode table, locale default, or local fallback decoder in the network substrate.
- Rejected toy/compatibility modules have explicit rationale.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in the added stdlib/runtime surfaces.
- Async and sync APIs follow the Phase 32 workload and cancellation model.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
- `verification/stdlib/network_http_substrate_inventory.md`
- `verification/stdlib/network_http_substrate_inventory.json`
- `verification/stdlib/network_http_cpython_evidence_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test/doc files scanned
- mined/blocked/rejected/external-signal CPython test families
- final deferred/rejected/host-limited/internal-only decision index
- text/i18n dependency state for every URL, header, body, cookie, certificate-display, observability, diagnostics, demo, Phase 41, and HTTP client handoff surface

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No CPython stdlib parity objective, backward-compatibility shim, legacy alias, deprecated behavior, bridge alias, migration path, or fallback path may survive phase exit.
- No direct Tokio/runtime types may leak into public Sifr APIs.
- No local encoding registry, local Unicode data table, locale-derived default encoding, fallback decoder, or duplicate text error-handler table may be introduced in this phase.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added blocking sync function must be classified in the stdlib workload database.
- Every added async function must have a real suspension summary.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Any external crate accepted during M1-M4 implementation that was not in the M0 Rust Ecosystem First table must complete the same dependency decision record in the PR that first introduces the dependency.
- Every public module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.
- Every public network/web API must pass the No-Toy-Module Gate and Maintenance Burden Test.

## Open Planning Questions To Resolve In `milestone_network_http_0`

1. Which exact `sifr.net`, `sifr.tls`, `sifr.url`, and `sifr.http` paths are public, and which private runtime modules remain internal implementation details?
2. Which Rust TLS root strategy is acceptable for deterministic local tests and production binaries?
3. Which HTTP transport dependency stack meets binary-size, safety, and maintenance goals without importing a web framework into substrate code?
4. Which host-specific socket/readiness constants or behaviors are needed internally, waived, or host-limited?
5. Which external-network CPython tests are converted to loopback fixtures versus retained as non-blocking ecosystem signal?
6. Does UDP enter M1 based on concrete near-term production workloads, or remain deferred?
7. Which public HTTP protocol types, if any, remain stable substrate API before Phase 41 and the HTTP client phase?
