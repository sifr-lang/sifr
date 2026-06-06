# Ad Hoc Phase: Production Network and HTTP Platform Substrate

Status: draft
Phase placement: third implementation phase in the split production-stdlib substrate sequence, after the text/Unicode/encoding/i18n runtime phase and the concurrency/process/runtime substrate phase, and before Phase 41 can claim production readiness for networked programs.
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

The cancellation, timeout, backpressure, shutdown, offload, and diagnostics items above are network-layer applications of the concurrency/runtime provider phase. This phase consumes those primitives and must not introduce its own cancellation token model, shutdown coordinator, offload pool, or diagnostics routing system.

## Split-Out Phases

The original broad planning scan also covered two important areas that are now tracked as separate ad hoc phases:

- [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md): Sifr-native concurrency/process/runtime substrate, including task, sync, process, offload, shutdown, diagnostics, and typed IPC foundations.
- [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md): Sifr-native text/Unicode/encoding/i18n runtime substrate, including explicit text I/O, encoding, Unicode data, segmentation, locale IDs, formatting, and translation bundles.

This phase consumes the completed text/i18n and concurrency/runtime provider contracts for URL text handling, body text decoding hooks, diagnostics, subprocess-backed demos, cancellation, timers, and executor-backed serving. It must not implement their module surfaces here.

Recommended implementation order:

1. [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md)
2. [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md)
3. This network/HTTP platform substrate phase

This phase is third because production network/server work should consume both the shared text/encoding/Unicode substrate and the production task, cancellation, shutdown, offload, diagnostics, and process model.

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
| `sifr.net` | `production-public` | primary low-level network API, including TCP and the constrained UDP surface defined below |
| `sifr.tls` | `production-public` | primary TLS API, including client certificate authentication with deterministic fixtures |
| `sifr.url` | `production-public` | typed URL API |
| `sifr.http` protocol types | `production-substrate` | canonical request/response/header/status/body primitives; `sifr.http.core` is rejected as an extra stable namespace layer |
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
| `http.cookiejar` | deferred | cookie persistence belongs in a future HTTP client phase, not this substrate phase |
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

The split phase order is explicit:

1. Text/Unicode/encoding/i18n runtime.
2. Concurrency/process/runtime substrate.
3. Network/HTTP platform substrate.

This phase runs third, after the text/i18n and concurrency/runtime provider phases. The dependency states below describe which completed provider milestone each network/HTTP surface consumes; they are not permission to start network/HTTP implementation out of order.

- Text/i18n is a hard prerequisite for non-UTF-8 HTTP body decoding, URL percent-encoding variants that require codec lookup, Unicode/IDNA host canonicalization, file/text handlers, text-heavy demos, and any network demo that depends on `open(..., encoding=...)`.
- The precise unblock point for encoding-dependent network features is completion of text/i18n `milestone_text_i18n_1: Encoding And Explicit Text I/O`; this phase records those features as `blocked-on-text-i18n-m1` until that milestone is closed.
- Unicode-data-dependent network features wait for text/i18n `milestone_text_i18n_2: Unicode Core` or `milestone_text_i18n_2_5: Unicode Segmentation` when they require Unicode normalization, case folding, IDNA behavior, display boundaries, or user-visible Unicode property decisions.
- Locale-sensitive network logging, formatting, or diagnostics wait for text/i18n `milestone_text_i18n_3: Locale Identifiers And Locale-Sensitive Formatting`; this phase must not introduce locale-derived defaults.
- Concurrency/runtime is a hard prerequisite for executor-backed serving APIs. This phase does not implement public thread, executor, queue, process, warning, or signal modules.
- Async scheduler/task primitives are prior runtime infrastructure owned by the existing async model. This phase consumes that runtime and adds only network-specific stream, TLS, and HTTP suspension points.
- Phase 41 consumes this phase for server framework routing, middleware, lifecycle/shutdown, typed extractors, validation, error mapping, and operational hooks.
- A separate production HTTP client phase consumes this phase for connection pooling, redirects, retry policy, auth, cookies, proxies, streaming upload/download, JSON helpers, multipart, compression, test transports, and sync/async product design.

### Text/I18n Dependency Decisions

This matrix records how network/HTTP consumes the completed text/i18n provider phase. It must not be used to justify local encoding fallbacks, duplicate codec registries, or silent byte-to-UTF-8 coercion.

| Network/HTTP surface | Text/i18n state | Text/i18n dependency | Decision |
| --- | --- | --- | --- |
| TCP, constrained UDP, DNS socket transport, TLS handshakes, HTTP/1.1 and HTTP/2 framing | `production-substrate` | none | Protocol bytes and transport state are independent of text decoding. |
| HTTP request/response bodies | `blocked-on-text-i18n-m1` for text helpers | M1 for charset decoding | Bodies are `Bytes`/stream values at the protocol layer. `body.text(...)`, charset-aware decoding, and text body previews consume M1. |
| HTTP headers | `blocked-on-text-i18n-m1` for non-ASCII/user-text conversion | M1 only for non-ASCII/user-text conversion | Header names use HTTP token/ASCII validation. Header values remain typed protocol values or byte/ASCII-safe values; decoding arbitrary header bytes to Sifr text consumes M1. |
| `Content-Type` and `charset` parameters | parse ASCII parameter names/labels only | M1 for actual charset decoding | This phase may parse and preserve charset labels, but must not decode payloads locally. Unsupported labels remain typed errors or blocked records. |
| URL parsing/building | ASCII and already-valid Sifr text are `production-substrate`; non-UTF-8 and Unicode host behavior are provider-gated | M1 for non-UTF-8 encoding labels; M2 for Unicode/IDNA alignment | `sifr.url` parses/builds typed URLs over valid Sifr strings and bytes. Non-UTF-8 `encoding=` behavior and codec-label lookup consume M1. Unicode host canonicalization/IDNA consumes the text/i18n-approved Unicode version from M2. |
| Percent encoding/decoding | byte/ASCII/UTF-8 safe operations | M1 for named encodings and error handlers | Percent helpers that take raw bytes are allowed. Helpers that accept `encoding=` or `errors=` must call the text substrate after M1. |
| Query/form helpers | byte/ASCII/UTF-8 safe parsing only | M1 for non-UTF-8 form decoding | `application/x-www-form-urlencoded` charset variants, dynamic `encoding=`, and text error handlers are blocked on M1. Multipart/form parsing is rejected for this phase and deferred to the future HTTP client/framework phases. |
| Cookie header parsing | header-level syntax only | M1 for percent-decoded/user-text values | Cookie names/values may remain typed header strings/bytes. Percent-decoded or non-UTF-8 cookie text is blocked on M1. Cookie persistence remains outside this phase. |
| TLS certificate verification | allowed | none | Verification is owned by `rustls`/TLS substrate and does not need text/i18n. |
| TLS certificate field inspection/display | typed raw/DER or ASCII-safe fields only | M1/M2 for decoded display names and Unicode normalization | User-visible certificate subject/issuer/SAN display must not invent local string decoding. M2 gates Unicode normalization/IDNA-sensitive display behavior. |
| Error messages and diagnostics | static ASCII diagnostic templates with typed evidence | M1/M3 for decoded remote text or locale-sensitive formatting | Errors carry typed bytes/labels/status evidence. No locale-sensitive formatting or decoded body/header snippets before the provider milestones. |
| Observability hooks | structured events with typed fields | M1 for decoded previews; M3 for locale-sensitive formatting | Trace/metric labels use stable ASCII keys and typed values. Body/header previews that decode text are blocked on M1. |
| Demos and fixtures | `blocked-on-text-i18n-m1` for text-mode file I/O or non-UTF-8 demos | M1 for `open(..., encoding=...)` or non-UTF-8 demos | Demos that need text-mode file I/O or non-UTF-8 behavior consume M1 and must not use local codec workarounds. |
| Phase 41 web framework handoff | provider-state-specific | M1 for text body/extractor decoding; M2 for Unicode path/host normalization; M3 for locale-sensitive formatting | Framework extractors such as text bodies, decoded forms, JSON text helpers, and decoded path/query values must consume text/i18n provider APIs. |
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

### Concurrency/Runtime Dependency Decisions

This matrix records how network/HTTP consumes the completed concurrency/runtime provider phase. It must not be used to justify local cancellation, timeout, shutdown, offload, diagnostics, process, executor, queue, task-context, or IPC substitutes.

| Network/HTTP surface | Concurrency/runtime state | Concurrency/runtime dependency | Decision |
| --- | --- | --- | --- |
| TCP/TLS/HTTP cancellation and timeout handling | `blocked-on-concurrency-runtime-m1` | `sifr.task` cancellation/deadline model | Network operations apply the provider task cancellation/deadline model. This phase must not introduce a parallel cancellation token or timeout coordinator. |
| Stream backpressure and task-aware suspension | `blocked-on-concurrency-runtime-m1` or `blocked-on-concurrency-runtime-m2` as classified in M0 | structured task and synchronization/backpressure semantics | Async streams must compose with provider task cancellation and accepted backpressure primitives. Local queue/channel substitutes are forbidden. |
| Sync network helpers that can block, including sync DNS/connect helpers | `blocked-on-concurrency-runtime-m3` | `sifr.runtime.spawn_blocking` and workload/effect diagnostics | Blocking helpers must use the provider offload substrate and stdlib workload database. This phase must not introduce a local thread pool or blocking executor. |
| Graceful connection/server shutdown | `blocked-on-concurrency-runtime-m5` | structured shutdown and signal stream model | Servers must consume the provider shutdown/signal substrate. This phase must not introduce a local shutdown coordinator. |
| Network observability and diagnostic routing | `blocked-on-concurrency-runtime-m5` | structured runtime diagnostics and task/request context model | Network lifecycle events must compose with the provider diagnostics/context model. This phase must not introduce a separate diagnostic routing system. |
| Executor-backed serving, worker handoff, or process-backed network helpers | `blocked-on-concurrency-runtime-m3` or `blocked-on-concurrency-runtime-m6` as classified in M0 | offload and typed IPC/process-worker gates | Executor/process-backed features remain blocked or deferred until the provider offload and IPC gates are complete. No local worker pool is allowed. |

M0 must add these decisions to the inventory and assign every runtime-dependent surface one of:

- `production-substrate`
- `blocked-on-concurrency-runtime-m1`
- `blocked-on-concurrency-runtime-m2`
- `blocked-on-concurrency-runtime-m3`
- `blocked-on-concurrency-runtime-m5`
- `blocked-on-concurrency-runtime-m6`
- `deferred-to-http-client-phase`
- `deferred-to-phase-41`
- `rejected`

When a runtime-dependent feature is unblocked, it must call the task, sync, runtime, process, signal, diagnostics, or IPC substrate from the concurrency/runtime phase. It must not introduce a second cancellation model, deadline coordinator, shutdown manager, offload pool, local executor, queue/channel primitive, task context, diagnostics bus, or IPC mechanism.

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
- `blocked-on-concurrency-runtime-m1`
- `blocked-on-concurrency-runtime-m2`
- `blocked-on-concurrency-runtime-m3`
- `blocked-on-concurrency-runtime-m5`
- `blocked-on-concurrency-runtime-m6`
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
4. `milestone_network_http_3` before M4 HTTP integration where URL/header/cookie parsing is consumed. M3 owns canonical URL, header, and cookie-header primitives; M4 consumes them and must not define parallel representations.
5. `milestone_network_http_4` before M5 handoff.
6. `milestone_network_http_5` last, after every proposed surface and CPython evidence family in this phase is closed.

Parallel work is allowed only for pure parser work that does not consume unfinished runtime substrate.

## Architecture Principles

### Native Runtime First

Implement the canonical runtime primitive first. Do not layer CPython-shaped public modules over it in this phase. Use production Rust ecosystem crates for protocol and transport machinery; do not hand-roll networking, TLS, DNS, URL, HTTP, HPACK, or observability infrastructure in this phase. If the selected Rust ecosystem stack cannot satisfy a required surface, that surface is deferred with evidence instead of receiving a bespoke implementation.

- Tokio remains the backing async runtime because the generated task runtime already depends on `tokio` and `sifr_stdlib::StdlibFeature::Tokio`.
- Generated Cargo must use the resolved Tokio feature set: `macros`, `rt-multi-thread`, `sync`, `time`, `net`, and `io-util`.
- No `async-std`, custom event-loop runtime, or public Tokio type is introduced without a separate architecture issue.
- `sifr.net` owns TCP, constrained UDP, DNS/address resolution, stream readiness, connection lifecycle, and network-facing application of provider timeout, cancellation, backpressure, and shutdown semantics.
- `sifr.tls` owns TLS configuration, handshakes, certificate verification, SNI, ALPN, wrapped streams, and TLS errors.
- `sifr.url` owns URL parsing/building, percent encoding, query handling, and authority/host/port validation.
- `sifr.http` protocol/runtime internals own request/response transport, headers, body streaming, parser/encoder limits, connection reuse, and HTTP protocol errors.

`sifr.net` cancellation and timeout semantics are network-layer applications of the `sifr.task` cancellation and deadline model from `milestone_concurrency_runtime_1`; this phase does not implement its own cancellation primitive. `sifr.net` graceful shutdown consumes the signal/shutdown substrate from `milestone_concurrency_runtime_5`; this phase does not implement its own shutdown coordinator.

The exact internal module names may change during implementation, but the boundary must exist: public Sifr-native modules must not duplicate target-runtime logic, and internal protocol/test harnesses must not leak as stable user API.

### Rust Ecosystem First

This phase builds a Sifr platform, not a new Rust networking stack. The implementation should wrap and constrain mature Rust crates, then add Sifr-specific type surfaces, ownership semantics, diagnostics, and panic-free error mapping.

Default ecosystem stack:

| Area | Preferred crates | Role |
| --- | --- | --- |
| Async runtime and I/O | `tokio`, `tokio-util`, `bytes` | task runtime, TCP/UDP, timers, cancellation, async read/write traits, buffer primitives |
| Socket options | `socket2` | low-level options only when Tokio/std do not expose required production behavior |
| DNS | `tokio::net::lookup_host`, with `hickory-resolver` reserved for explicit resolver APIs | system resolver integration for connect/listen address resolution; in-process resolver only for typed DNS APIs accepted later |
| TLS | `rustls`, `tokio-rustls`, `rustls-platform-verifier`, `rustls-pemfile` | TLS config, async streams, certificate verification, platform roots, PEM parsing |
| Certificate inspection | deferred; `x509-parser` only in a future certificate-inspection phase | no public subject/issuer/SAN display parser in this phase; certificate errors carry typed verification evidence and raw DER fingerprints only |
| Test certificates | `rcgen` | deterministic local CA/self-signed fixtures only |
| HTTP types and bodies | `http`, `http-body`, `http-body-util`, `bytes` | method/status/header/request/response/body abstractions |
| HTTP/1 and HTTP/2 transport | `hyper`, `hyper-util`, `h2` | production HTTP transport, HTTP/2 state machine, flow control, multiplexing, protocol errors |
| URL and percent encoding | `url`, `percent-encoding` | WHATWG/RFC URL parsing, building, and escaping |
| Cookies | `cookie` | header-level cookie parsing; jar/persistence features remain out of scope for this substrate phase |
| Middleware/service substrate | `tower-service` | internal service abstraction for Phase 41 handoff; no public `tower`, `Layer`, or tower utility types |
| Observability | `tracing`, `metrics`; OpenTelemetry bridge deferred | spans, structured events, counters, histograms, exporter-neutral hooks without exporter dependencies |
| Tests and conformance | `tokio-test`, `proptest`, `h2spec`/HTTP/2 conformance fixtures where available | deterministic async tests, parser/property tests, protocol conformance |

Resolved ecosystem decisions:

| Decision area | Decision |
| --- | --- |
| Tokio features | Generated Cargo uses explicit features only: `macros`, `rt-multi-thread`, `sync`, `time`, `net`, and `io-util`. `tokio/full` is rejected for production builds and may not be used to hide missing feature decisions. |
| Rustls crypto provider | Use rustls's default `aws-lc-rs` provider for production TLS. `ring`, custom providers, and OpenSSL/native-tls are out of scope unless a future platform issue records a concrete blocker. |
| TLS roots | Production client verification uses `rustls-platform-verifier`. Deterministic tests use explicit in-memory `RootCertStore` values built from `rcgen` fixtures. `webpki-roots` is not a fallback in this phase. |
| DNS | TCP connect and address resolution use `tokio::net::lookup_host` to respect host resolver configuration. `hickory-resolver` is deferred to an explicit `sifr.net.resolve_*` API if future requirements need record lookups, custom resolver config, or deterministic in-process DNS tests. |
| Stream I/O ownership | Streams use owned-buffer reads: `read_chunk(max_bytes) -> Result[Option[bytes], NetError]`, where `None` means EOF. Writes provide `write(data) -> Result[int, NetError]` and `write_all(data) -> Result[None, NetError]`. |
| UDP | M1 includes a constrained `UdpSocket` with `bind`, `send_to`, `recv_from`, `connect`, `send`, `recv`, `local_addr`, and `close`. Broadcast, multicast, raw sockets, packet options, and platform-specific socket constants are deferred or host-limited. |
| Socket options | `socket2` is accepted for `SO_REUSEADDR`, host-limited `SO_REUSEPORT`, `TCP_NODELAY`, `SO_KEEPALIVE`, and `IPV6_V6ONLY` when Tokio/std do not expose deterministic behavior. Other options are not public. |
| HTTP stack | `hyper`, `hyper-util`, and `h2` are the only accepted HTTP/1.1 and HTTP/2 transport stack. `reqwest`, `axum`, `warp`, `actix-web`, and `tower-http` are not substrate dependencies. |
| Service substrate | Use the `tower-service` crate only, not the full `tower` crate. The `Service` trait is internal. No `tower::Layer`, tower utility modules, or extra Tower features are pulled. Public Sifr APIs expose Sifr request/response and middleware concepts, not Tower traits. |
| OpenTelemetry | OTel exporter/bridge crates are deferred to an observability/exporter phase. This phase emits `tracing` spans/events and `metrics` counters/histograms only. |
| mTLS | M2 includes client certificate authentication configuration and deterministic `rcgen` client/server certificate fixtures. The API exposes configuration, verification outcomes, and typed errors, not raw rustls types. |
| Multipart/form | Multipart parsing is rejected for this phase and deferred to Phase 41 or the production HTTP client phase. No `multipart` crate is accepted here. |
| Upgrade hooks | HTTP upgrade hooks are `internal-test` only for transport validation. Public WebSocket, CONNECT tunneling, and upgrade APIs are deferred to product phases with concrete use cases. |
| External CPython network tests | External-network CPython tests are never required for local validation. Localnet cases are converted to loopback where useful; true external tests remain `external-signal`. |

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

From-scratch protocol parsing, TLS verification, DNS resolution, URL parsing, HPACK, HTTP/2 state machines, or metrics/tracing backends are rejected in this phase. If the selected Rust ecosystem stack cannot satisfy a required surface, that surface is deferred with evidence instead of receiving a bespoke implementation.

### Sifr-Native Network API Shape

The accepted public shape is Sifr-native and typed:

- `async connect_tcp(address, *, timeout=None, local_addr=None) -> Result[TcpStream, NetError]`
- `async listen_tcp(address, *, backlog=None, reuse_addr=false) -> Result[TcpListener, NetError]`
- `async TcpListener.accept() -> Result[(TcpStream, SocketAddr), NetError]`
- `async TcpStream.read_chunk(max_bytes) -> Result[Option[bytes], NetError]`
- `async TcpStream.write(data) -> Result[int, NetError]`
- `async TcpStream.write_all(data) -> Result[None, NetError]`
- `async TcpStream.close() -> Result[None, NetError]`
- `UdpSocket` constrained datagram support as defined in the resolved ecosystem decisions.

The API must expose local/remote address inspection, graceful shutdown, resource-limit controls, and deterministic cancellation semantics. It must not expose descriptor aliasing, monkeypatchable globals, or public raw event-loop policies.

The accepted stream ownership model is owned-buffer I/O:

- `read_chunk(max_bytes) -> Result[Option[bytes], NetError]`; `None` means EOF.
- `write(data) -> Result[int, NetError]`; returns the number of bytes accepted by the underlying stream.
- `write_all(data) -> Result[None, NetError]`; retries partial writes until completion or typed failure.

`max_bytes` must be positive and within the configured resource limit. Zero-length and too-large reads return typed errors. Cancellation of `write_all` is cancellation-safe for memory safety but may have written a prefix to the peer; this must be documented and surfaced through typed cancellation evidence rather than hidden retries.

### TLS API Shape

The accepted public shape is:

- `TlsClientConfig` and `TlsServerConfig`
- safe default certificate verification
- production root strategy through `rustls-platform-verifier`; deterministic tests through explicit `rcgen` roots
- SNI and ALPN support
- client certificate authentication with deterministic `rcgen` client/server fixtures
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
- upgrade hooks are internal-test only; public WebSocket, CONNECT tunneling, and upgrade APIs are deferred

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
| pure URL, header, and cookie-header parsing under configured size limits | pure | legal in async contexts; over-limit inputs return typed errors instead of becoming hidden CPU-heavy work |
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
- Use the resolved ecosystem decisions in this phase as the starting point; M0 may only change one by recording a blocking implementation finding and a replacement decision with the same audit fields.
- Record explicit rejection rationale before implementing any protocol/domain component from scratch.
- Define HTTP client/server substrate boundaries.
- Define protocol version scope, including HTTP/1.1, HTTP/2, and explicit HTTP/3 deferral entries.
- Define buffer ownership and API pattern for stream I/O before M1 backlog entries are finalized.
- Define the complete text/i18n dependency inventory for URL, headers, bodies, cookies, certificate display, observability, diagnostics, demos, Phase 41 handoff, and HTTP client handoff.
- Define the complete concurrency/runtime dependency inventory for cancellation, deadlines, backpressure, blocking/offload, shutdown, diagnostics, task context, worker/process handoff, Phase 41 handoff, and HTTP client handoff.
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

- Every proposed surface is classified as `production-public`, `production-substrate`, `internal-test`, `deferred`, `rejected`, `blocked-on-text-i18n-m1`, `blocked-on-concurrency-runtime-m1`, `blocked-on-concurrency-runtime-m2`, `blocked-on-concurrency-runtime-m3`, `blocked-on-concurrency-runtime-m5`, `blocked-on-concurrency-runtime-m6`, or `host-limited`.
- Every text-dependent surface is classified as `production-substrate`, `blocked-on-text-i18n-m1`, `blocked-on-text-i18n-m2`, `blocked-on-text-i18n-m2_5`, `blocked-on-text-i18n-m3`, `deferred-to-http-client-phase`, `deferred-to-phase-41`, or `rejected`.
- Every runtime-dependent surface is classified as `production-substrate`, `blocked-on-concurrency-runtime-m1`, `blocked-on-concurrency-runtime-m2`, `blocked-on-concurrency-runtime-m3`, `blocked-on-concurrency-runtime-m5`, `blocked-on-concurrency-runtime-m6`, `deferred-to-http-client-phase`, `deferred-to-phase-41`, or `rejected`.
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
- Add constrained UDP datagram support:
  - bind/connect
  - send/send_to
  - recv/recv_from
  - close
  - local address inspection
  - deterministic loopback tests
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
- `tokio-util` and `bytes`
- `socket2` for the accepted socket options listed in the resolved ecosystem decisions
- `tokio::net::lookup_host`; `hickory-resolver` remains deferred for explicit DNS record APIs

Definition of done:

- TCP loopback tests pass deterministically without external network dependency.
- UDP loopback tests pass for the constrained datagram surface.
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
- Add client certificate authentication with deterministic fixtures.
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
- `rustls-platform-verifier` for production client verification
- `rustls-pemfile`
- `rcgen` for deterministic local certificate fixtures
- no `x509-parser` in this phase

Definition of done:

- Local self-signed and CA-backed handshake fixtures are deterministic.
- HTTPS-ready TLS loopback tests pass.
- Client certificate authentication loopback tests pass with `rcgen`-generated client and server certificate fixtures; mTLS handshake rejection and typed `CertificateError` are covered.
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
- Own the canonical header primitives consumed by M4 HTTP transport; M4 must not define duplicate header-name, header-value, or cookie-header representations.
- Implement small cookie header parsing required by real HTTP request/response handling.
- Keep cookie persistence and jar policy out of this phase.
- Record non-UTF-8 codec-dependent behavior as `blocked-on-text-i18n-m1`; do not duplicate codec registry behavior locally.
- Keep Unicode/IDNA host canonicalization `blocked-on-text-i18n-m2`; M3 accepts ASCII and already-punycode host behavior only until the text/i18n provider defines Unicode alignment.
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
- `http` header/status/method types
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
- Implement method, status, version, and body types while consuming the M3 URL/header/cookie primitives.
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
- `tower-service` crate internally for Phase 41 handoff
- avoid pulling a web framework into the substrate

Definition of done:

- HTTP/1.1 and HTTP/2 loopback client/server transport tests pass without external network.
- HTTPS transport works through M2 TLS, including ALPN selection for HTTP/2.
- Malformed HTTP tests produce typed protocol errors.
- Body streaming and HTTP/2 multiplexing work without unbounded buffering.
- HTTP/2 protocol-level behaviors selected in the M0 conformance inventory, including SETTINGS negotiation, RST_STREAM stream cancellation, GOAWAY graceful shutdown, and HPACK correctness edge cases, have loopback test coverage.
- HTTP transport stores and forwards binary bodies and typed protocol metadata without local text decoding fallbacks.
- No `http.server`, `socketserver`, or handler-subclass public API is added.

### milestone_network_http_5: Integration, Documentation, And Production Handoff

Scope:

- Update public docs for:
  - `sifr.net`
  - `sifr.tls`
  - `sifr.url`
  - public HTTP protocol/substrate types under `sifr.http`
  - rejected/deferred CPython-shaped surfaces and why they are not recommended APIs
  - text/i18n dependency decisions and blocked surface states
  - concurrency/runtime dependency decisions and blocked surface states
- Update internal architecture docs for:
  - runtime networking/TLS/HTTP boundaries
  - provider consumption for task cancellation, deadlines, shutdown, offload, diagnostics, and process/worker handoff
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
  - every text/i18n-dependent and runtime-dependent surface has a provider milestone state
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
- concurrency/runtime dependency state for every cancellation, timeout, backpressure, blocking/offload, shutdown, diagnostics, task-context, executor/worker, process, Phase 41, and HTTP client handoff surface

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No CPython stdlib parity objective, backward-compatibility shim, legacy alias, deprecated behavior, bridge alias, migration path, or fallback path may survive phase exit.
- No direct Tokio/runtime types may leak into public Sifr APIs.
- No local encoding registry, local Unicode data table, locale-derived default encoding, fallback decoder, or duplicate text error-handler table may be introduced in this phase.
- No local cancellation token model, timeout/deadline coordinator, shutdown manager, offload pool, executor, queue/channel primitive, task context, process/worker pool, IPC mechanism, or diagnostic routing system may be introduced in this phase; these must consume the concurrency/runtime provider substrate.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added blocking sync function must be classified in the stdlib workload database.
- Every added async function must have a real suspension summary.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Any external crate accepted during M1-M4 implementation that was not in the M0 Rust Ecosystem First table must complete the same dependency decision record in the PR that first introduces the dependency.
- Every public module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.
- Every public network/web API must pass the No-Toy-Module Gate and Maintenance Burden Test.

## Resolved Planning Decisions For M0

M0 validates these decisions and records dependency audit evidence; it does not reopen them without a concrete blocking finding.

1. Public paths are `sifr.net`, `sifr.tls`, `sifr.url`, and `sifr.http`; `_sifr.*` and Rust modules remain implementation details.
2. Production TLS verification uses `rustls-platform-verifier`; deterministic tests use explicit `rcgen` roots.
3. HTTP transport uses `hyper`, `hyper-util`, and `h2`; no web framework or high-level client crate enters this substrate.
4. Host-specific socket options are limited to the socket2 list above and recorded as portable or host-limited.
5. External-network CPython tests are `external-signal`; useful localnet behavior is converted to loopback.
6. UDP is accepted in M1 as a constrained datagram API; advanced datagram features are deferred or host-limited.
7. Stable HTTP substrate types live under `sifr.http`; public client/server products remain future phases.
