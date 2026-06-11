# Ad Hoc Phase: Production Network and HTTP Platform Substrate

Status: draft
Phase placement: third implementation phase in the split production-stdlib substrate sequence, after the text/Unicode/encoding/i18n runtime phase and the concurrency/process/runtime substrate phase, and before Phase 41 can claim protocol/runtime production readiness for networked programs. Multi-core serving throughput is explicitly owned by the serving-scale follow-up recorded in M0, not by this substrate phase.
Phase owner: runtime/networking implementation with compiler import, effect, and codegen support

## Objective

Build the production-grade network, TLS, URL, and HTTP substrate required for real Sifr networked programs, Phase 41's FastAPI-like web framework, and a later httpx-like production HTTP client.

This phase does not attempt CPython networking/web stdlib parity. CPython is an evidence source for edge cases, protocol behavior, and explicit rejection decisions; it is not the product shape.

This phase provides production-correct network and HTTP substrate semantics. It does not by itself provide a multi-core async serving topology. The v1 scaling boundary must be recorded in M0 so Phase 41 can distinguish protocol/runtime readiness from multi-core throughput readiness.

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

- [ad-hoc-production-concurrency-runtime-platform-substrate.md](./ad-hoc-production-concurrency-runtime-platform-substrate.md): Sifr-native concurrency/process/runtime substrate, including task, sync, process, offload, shutdown, diagnostics, and typed IPC foundations.
- [ad-hoc-production-text-i18n-platform-substrate.md](./ad-hoc-production-text-i18n-platform-substrate.md): Sifr-native text/Unicode/encoding/i18n runtime substrate, including explicit text I/O, encoding, Unicode data, segmentation, locale IDs, formatting, and translation bundles.

This phase consumes the completed text/i18n and concurrency/runtime provider contracts for URL text handling, body text decoding hooks, diagnostics, subprocess-backed demos, cancellation, timers, and executor-backed serving. It must not implement their module surfaces here.

Recommended implementation order:

1. [ad-hoc-production-text-i18n-platform-substrate.md](./ad-hoc-production-text-i18n-platform-substrate.md)
2. [ad-hoc-production-concurrency-runtime-platform-substrate.md](./ad-hoc-production-concurrency-runtime-platform-substrate.md)
3. This network/HTTP platform substrate phase

This phase is third because production network/server work should consume both the shared text/encoding/Unicode substrate and the production task, cancellation, shutdown, offload, diagnostics, and process model.

This phase also depends on [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md). Its namespace contract is assumed complete before these substrate milestones ship: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.

This phase also consumes the shared platform contract in [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md). Network M0 must use that contract's terminal states, stability levels, ownership/lifetime rules, cancellation/backpressure semantics, typed error nesting, observability fields, supported-host matrix, security/resource ownership table, and cross-phase golden fixture manifest.

## Product Boundary

A module belongs in Sifr core only if it is production substrate or production developer experience.

| Layer | Decision | Public? | Rationale |
| --- | --- | --- | --- |
| Runtime substrate | build now | mostly public low-level API plus private intrinsics | required for all clients, servers, and framework work |
| Protocol substrate | build now | partly public | HTTP/TLS/URL correctness is shared foundation |
| Production HTTP client API | reserve separate phase | public later | httpx-like client behavior is a product surface, not stdlib parity |
| Production server framework | Phase 41 | public later | routing, middleware, extractors, lifecycle, and ops hooks belong there |
| CPython-shaped networking modules | reject/diagnose | no | maintenance burden without strategic production value |

### Public Surfaces Built Now

| Surface | Classification | Notes |
| --- | --- | --- |
| `sifr.net` | `production-public` | primary low-level TCP network API; constrained UDP is M0-gated and ships only if M0 records a near-term production consumer |
| `sifr.tls` | `production-public` | primary TLS API, including client certificate authentication with deterministic fixtures |
| `sifr.url` | `production-public` | typed URL API |
| `sifr.http` protocol types | `production-substrate` | canonical request/response/header/status/body primitives; `sifr.http.core` is rejected as an extra stable namespace layer |
| internal loopback harness | `test-only-harness` | never a public dev-server module |
| readiness primitives | `production-substrate` | internal or low-level only; no public manual event-loop model |

### Deferred Or Rejected Public Surfaces

| Surface | Decision | Reason |
| --- | --- | --- |
| `sifr.http.server` | rejected as public API | toy/basic server shape; server product is Phase 41 |
| `sifr.socketserver` | rejected | inheritance-heavy handler model conflicts with Sifr's static model |
| `sifr.urllib.request` | `rejected` or `unsupported-with-diagnostic` | old opener/handler architecture; future client work must be Sifr-native and httpx-like |
| `sifr.http.client` | `rejected` or `unsupported-with-diagnostic` | CPython-shaped low-level client API is not the product; future public client phase owns a modern Sifr API |
| `sifr.socket` | `rejected` or `unsupported-with-diagnostic` | CPython descriptor-shaped API must not define Sifr networking |
| `sifr.ssl` | `rejected` or `unsupported-with-diagnostic` | TLS is exposed through `sifr.tls`, not `SSLContext` mimicry |
| `sifr.select` / `sifr.selectors` | internal readiness only | users should use async streams, not manual event loops |
| `sifr.urllib.parse` | `rejected` or `unsupported-with-diagnostic` | stable URL utility is `sifr.url` |
| `urllib.robotparser` | rejected | niche CPython utility, not core platform substrate |
| `http.cookiejar` | rejected as CPython-shaped core API | a future Sifr-native HTTP client may own cookie persistence if product requirements justify it |
| HTTP/3 / QUIC | `deferred-to-transport-phase` | revisit in a future transport phase after QUIC runtime strategy is designed |
| CGI-style serving | rejected | legacy serving model |
| `ThreadingMixIn` / `ForkingMixIn` | rejected | wrong abstraction and overlaps concurrency/runtime phases |
| raw event-loop policies | rejected | Phase 32 keeps raw event loops out of the user model |

## Compatibility Policy

CPython-shaped networking modules are not part of the production baseline.

This phase does not reserve a compatibility-adapter track for Python-shaped networking/web modules. CPython-shaped modules are evidence only and must resolve to one of:

- `rejected`
- `unsupported-with-diagnostic`
- `internal-only` for implementation evidence
- `test-only-harness` for deterministic local fixtures

Future product work may add Sifr-native APIs, such as a modern HTTP client, cookie persistence, WebSocket support, CONNECT tunneling, or QUIC transport, but it must not reuse CPython module shape as the default product boundary.

This phase must not add compatibility shims, fallback paths, bridge aliases, legacy aliases, deprecated behavior, migration paths, or partial public modules.

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

The `blocked-on-concurrency-runtime-*` labels below are Network M0 dependency-classification labels against the completed provider milestones, not a claim that the concurrency/runtime provider phase remains open. Network M0 must consume the closed provider semantics, including abort-backed task-handle cancellation, the compiler-recognized same-task `task.timeout(...)` cancellation scope, M2 backpressure primitives, M3 offload cancellation/abandonment evidence, and M5 shutdown/diagnostics behavior; any downstream cancellation/backpressure amendment remains a network consumer contract and must not introduce a parallel provider model.

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

- `mined-as-substrate-fixture`: behavior converted into a Sifr-native substrate test.
- `adapted-for-sifr-api`: behavior adapted to a Sifr-native API shape.
- `compat-adapter-deferred`: shared platform state intentionally unused by this phase; CPython-shaped networking adapters are not accepted product targets.
- `blocked-on-phase-X`: behavior depends on a split-out phase.
- `external-signal`: test is retained only as non-blocking ecosystem signal because it depends on external network state.
- `waived-with-rationale`: behavior is explicitly waived with rationale and reviewer sign-off.
- `rejected`: behavior belongs to a CPython-shaped, legacy, unsafe, or non-product surface.

Every proposed public, substrate, internal, and test-only surface must use the shared terminal states and stability levels from [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md). Network-specific provider states are allowed only as refinements of shared `blocked-on-phase-X` and `deferred-to-phase-X` states:

- `production-public`
- `production-substrate`
- `internal-only`
- `test-only-harness`
- `deferred-to-phase-X`
- `rejected`
- `unsupported-with-diagnostic`
- `blocked-on-text-i18n-m1`
- `blocked-on-concurrency-runtime-m1`
- `blocked-on-concurrency-runtime-m2`
- `blocked-on-concurrency-runtime-m3`
- `blocked-on-concurrency-runtime-m5`
- `blocked-on-concurrency-runtime-m6`
- `host-limited`

The inventory terminal state `open` is allowed during implementation only and is forbidden at phase exit.

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
- Generated Cargo must use the resolved Tokio feature set for this network feature: `macros`, `rt`, `sync`, `time`, `net`, and `io-util`.
- No `async-std`, custom event-loop runtime, or public Tokio type is introduced without a separate architecture issue.
- `sifr.net` owns TCP, constrained UDP, DNS/address resolution, stream readiness, connection lifecycle, and network-facing application of provider timeout, cancellation, backpressure, and shutdown semantics.
- `sifr.tls` owns TLS configuration, handshakes, certificate verification, SNI, ALPN, wrapped streams, and TLS errors.
- `sifr.url` owns URL parsing/building, percent encoding, query handling, and authority/host/port validation.
- `sifr.http` protocol/runtime internals own request/response transport, headers, body streaming, parser/encoder limits, connection reuse, and HTTP protocol errors.

`sifr.net` cancellation and timeout semantics are network-layer applications of the `sifr.task` cancellation and deadline model from `milestone_concurrency_runtime_1`; this phase does not implement its own cancellation primitive. `sifr.net` graceful shutdown consumes the signal/shutdown substrate from `milestone_concurrency_runtime_5`; this phase does not implement its own shutdown coordinator.

The exact internal module names may change during implementation, but the boundary must exist: public Sifr-native modules must not duplicate target-runtime logic, and internal protocol/test harnesses must not leak as stable user API.

### Serving Scale Boundary

This phase is responsible for protocol correctness, resource limits, typed errors, cancellation/backpressure semantics, and loopback-validated serving substrate. It is not responsible for choosing a new executor topology or making every server automatically use every CPU core.

The accepted v1 serving-scale contract is:

- generated network features use the current Tokio runtime topology from the concurrency/runtime provider and do not enable Tokio `rt-multi-thread`;
- one Sifr process may run a production-correct async accept/dispatch loop over this substrate;
- CPU-heavy work is offloaded through the concurrency/runtime provider's CPU/offload substrate, not by changing the network runtime topology here;
- multi-process serving, host-limited `SO_REUSEPORT`, process workers, and future multi-thread runtime topology are separate serving-scale concerns;
- Phase 41 may claim protocol/runtime production readiness after this phase, but must not claim multi-core serving throughput readiness until the serving-scale follow-up closes.

M0 must create or link the named follow-up issue for multi-core serving strategy. The follow-up must decide between, or explicitly defer, host-limited `SO_REUSEPORT` multi-process serving, process-worker supervision, a future provider-owned Tokio `rt-multi-thread` topology, or another Sifr-native serving-scale model. Until that follow-up closes, this phase's public server handoff is single-runtime-worker per process.

### Rust Ecosystem Decisions

This phase follows [Dependency Policy](../internal_docs/dependency_policy.md). It builds a Sifr network/TLS/URL/HTTP platform, not a new async runtime, socket library, DNS resolver, TLS stack, URL parser, HTTP parser, HPACK implementation, HTTP/2 state machine, web framework, HTTP client, or observability backend.

The crate families, feature boundaries, public hiding rules, and rejected adjacent crates below are locked inputs to implementation. The exact patch versions listed are candidate M0 lockfile pins; M0 may update patch versions within the accepted family only after recording MSRV, license, binary-size, platform, security, and generated-project snapshot impact. Patch updates do not require a phase amendment unless they change features, MSRV, build requirements, provider behavior, or Sifr public semantics.

M1-M5 implementation PRs must not perform crate-family discovery, swap in adjacent crates, add broad feature flags, or introduce bespoke replacements. Changing an accepted, conditional, or rejected dependency decision requires a new issue or explicit phase amendment before implementation work starts.

Dependency rings for this phase:

- Ring 2 generated-runtime core: Tokio, Tokio Util, direct `bytes`, and `tracing`, each activated only by the Sifr feature that needs it.
- Ring 3 stdlib feature-gated substrate: targeted `socket2` and conditional `metrics`.
- Ring 4 feature-specific protocol/data substrate: URL, percent-encoding, TLS, HTTP, HTTP/2, cookie-header, service-abstraction crates, and conditional Hyper/Tokio adapter helpers.
- Ring 5 dev/test/demo only: local async/protocol/property/certificate fixtures.
- Ring 6 rejected direct dependencies: listed below.

#### Locked Dependencies By Ring

| Ring | Capability | Crate decision | Version and feature plan | Milestone | Binding notes |
| --- | --- | --- | --- | --- | --- |
| Ring 2 | async runtime, TCP/UDP, timers, async I/O, async sync, generated entrypoints | `tokio` | keep workspace `tokio = 1.52.3`; expand generated-runtime features only to `macros`, `rt`, `sync`, `time`, `net`, and `io-util`; do not enable `full`, `rt-multi-thread`, `process`, `signal`, `fs`, `parking_lot`, or `tokio_unstable` for this network feature | M1-M5 | Network APIs consume the concurrency/runtime provider for task lifetime, cancellation, deadlines, shutdown, diagnostics, process, and signal semantics. This phase adds network suspension points and socket I/O only; it does not choose a new runtime topology or expose Tokio handles. |
| Ring 2 | Tokio cancellation/I/O helpers | `tokio-util` | add `tokio-util = 0.7.18` with `default-features = false`; enable `rt`, `io-util`, and `time` only if M1/M4 needs accepted helpers; do not enable `full`, `net`, `codec`, `compat`, or `join-map` | M1, M4 | Used only behind Sifr-owned stream/cancellation internals. Tokio Util token, codec, and compatibility types are never public. If Tokio plus Sifr wrappers suffice, M0 may keep this dependency conditional and unused. |
| Ring 2 | owned byte buffers and HTTP body chunks | `bytes` | add `bytes = 1.11.1` with default features only for generated/runtime crates that implement network or HTTP bodies | M1, M4 | `bytes::Bytes` may be used internally to avoid copies and support backpressure. Public Sifr APIs expose `bytes` values as Sifr-owned byte buffers, never Rust crate types. This is the narrow production need that overrides the general no-direct-`bytes` default in the dependency policy. |
| Ring 2 | structured spans and events | `tracing` | add `tracing = 0.1.44` with `default-features = false`, feature `std`; do not enable `attributes` | M1-M5 | Emits DNS/connect/TLS/HTTP lifecycle spans and events behind Sifr observability hooks. Applications/tests choose subscribers; no subscriber, recorder, or tracing type leaks. |
| Ring 3 | socket options not exposed by Tokio/std | `socket2` | add `socket2 = 0.6.4` only where M1 proves Tokio/std cannot expose required behavior; do not enable `all` | M1 | Limited to `SO_REUSEADDR`, host-limited `SO_REUSEPORT`, `TCP_NODELAY`, `SO_KEEPALIVE`, and `IPV6_V6ONLY`. Every host-limited option needs a supported-host matrix row and fixture. Other options are not public. |
| Ring 3 | metrics facade | `metrics` | add `metrics = 0.24.6` only after M0/M5 records metric names, label/cardinality policy, emission points, redaction policy, and deterministic tests | M5 | Optional facade only. No exporter, global recorder setup, or metrics crate type appears in public APIs. If the concrete schema is not approved, metrics remain deferred while tracing events still ship. |
| Ring 4 | URL parsing/building | `url` | update/use workspace `url = 2.5.8`; default `std` only; do not enable `serde` or `expose_internals` | M3 | Backing for `sifr.url` over valid Sifr text and bytes. Unicode/IDNA decisions still consume the text/i18n provider; crate behavior is wrapped into Sifr-owned typed errors and no crate type leaks. |
| Ring 4 | percent encoding | `percent-encoding` | add `percent-encoding = 2.3.2` with default `std` only | M3 | Used for byte/ASCII-safe percent helpers and URL internals. Named encodings and error handlers still call the text/i18n provider after M1. |
| Ring 4 | TLS core | `rustls` | add `rustls = 0.23.35` with default provider `aws_lc_rs`; no custom provider, `ring`, FIPS, compression, or zlib/brotli features in this phase | M2 | Owns TLS protocol machinery internally. Sifr exposes `TlsClientConfig`, `TlsServerConfig`, typed verification outcomes, and typed errors only. |
| Ring 4 | async TLS streams | `tokio-rustls` | add `tokio-rustls = 0.26.4`; default `aws_lc_rs` provider; do not enable `ring`, `fips`, `early-data`, compression, or zlib/brotli features | M2 | Wraps accepted Rustls streams over Sifr/Tokio TCP internals. Early data and compression are out of scope. |
| Ring 4 | platform certificate verification | `rustls-platform-verifier` | add `rustls-platform-verifier = 0.7.0` with default features disabled unless platform support proves a required feature; do not enable debug/cert logging features in production builds | M2 | Production client verification uses host platform roots. Deterministic tests use explicit in-memory roots from `rcgen` fixtures; `webpki-roots` is not a fallback. |
| Ring 4 | PEM parsing | `rustls-pemfile` | add `rustls-pemfile = 2.2.0` with default `std` only | M2 | Parses user-supplied PEM cert/key material into Sifr-owned TLS config errors. No generic certificate display parser is accepted. |
| Ring 4 | HTTP request/response types | `http` | add `http = 1.4.1` with default `std` only | M3, M4 | Backs method/status/header/request/response representations internally. Sifr owns validation, typed errors, and public type names. |
| Ring 4 | HTTP streaming body trait | `http-body` | add `http-body = 1.0.1` | M4 | Internal body substrate only. Public Sifr body streams remain Sifr-owned and cancellation-aware. |
| Ring 4 | HTTP body adapters | `http-body-util` | add `http-body-util = 0.1.3` with `default-features = false`; enable `channel` only if M4 proves it is needed for bounded body tests; never enable `full` | M4 | Used for narrow body adapters/fixtures, not as public body API. |
| Ring 4 | HTTP/1.1 and HTTP/2 transport | `hyper` | add `hyper = 1.10.1` with `default-features = false`, features `http1`, `http2`, `client`, and `server`; do not enable `full`, unstable `tracing`, `ffi`, `capi`, or `nightly` | M4 | Accepted core HTTP transport stack with `h2`. Internal client/server transport exists for protocol validation and future handoff, not as `sifr.http.client` or `sifr.http.server`. Sifr emits wrapper-level `tracing` spans/events instead of enabling Hyper's unstable tracing feature. |
| Ring 4 conditional | Tokio/Hyper adapter helpers | `hyper-util` | add `hyper-util = 0.1.20` with `default-features = false`, starting with `tokio` only, if M4 proves Hyper alone would require substantial custom runtime adapter code; add `http1`, `http2`, `server`, or `service` only when the selected adapter path proves each one necessary; default to a Sifr-owned graceful-shutdown loop over provider shutdown primitives and avoid `server-graceful` unless M4 proves it composes with provider-owned shutdown; do not enable `full`, `client-legacy`, `client-pool`, `client-proxy`, `client-proxy-system`, or `server-auto` by default | M4 | Conditional, internal-only adapter helper. Prefer Hyper directly plus small Sifr-owned adapters first. No Sifr public type, lifecycle policy, framework handoff contract, or shutdown semantics may depend on Hyper-Util's module shape. |
| Ring 4 | HTTP/2 state machine and flow control | `h2` | add `h2 = 0.4.14` with default features only as required by Hyper or direct HTTP/2 fixtures | M4 | HTTP/2 SETTINGS, HPACK, flow control, RST_STREAM, PING, GOAWAY, and multiplexing are crate-backed. Sifr maps protocol errors into typed `HttpError`/`ProtocolError`. M0 verifies whether Hyper already provides the required `h2` dependency transitively; a direct `h2` dependency is used only for direct fixtures or APIs that need it, and the lockfile must contain a coherent version/feature set. |
| Ring 4 | cookie header parsing | `cookie` | add `cookie = 0.18.1` with `default-features = false`; enable no signed/private/secure/jar-related features | M3 | Header-level parse/build only. Cookie persistence, signed/private jars, key management, and percent-decoded user text are not substrate features. |
| Ring 4 | service abstraction for Phase 41 handoff | `tower-service` | add `tower-service = 0.3.3` only; do not add the full `tower` crate | M4, M5 | `Service` may be used internally to shape transport/framework handoff. Public Sifr APIs expose Sifr request/response and middleware concepts, not Tower traits. |
| Ring 5 | async runtime tests | `tokio-test` | add `tokio-test = 0.4.5` as dev/test only | M1-M5 | Deterministic async tests only; never a generated production dependency. |
| Ring 5 | parser/property tests | `proptest` | add `proptest = 1.11.0` as dev/test only | M3, M4 | URL/header/cookie/HTTP parser property tests and shrinkable regression fixtures. |
| Ring 5 | deterministic certificates | `rcgen` | add `rcgen = 0.14.8` as dev/test only; use `aws_lc_rs` or default fixture crypto consistently with the TLS test plan; do not enable `x509-parser` | M2 | Local CA, server, and client certificates for TLS/mTLS loopback tests. Production binaries do not depend on `rcgen`. |
| Ring 5 | local observability tests and demos | `tracing-subscriber` | dev/test/demo only if fixtures need a subscriber | M5 | Runtime emits events; tests/demos may subscribe. No production substrate dependency. |

#### Rejected Direct Dependencies And Features

| Dependency or feature | Decision | Reason |
| --- | --- | --- |
| `tokio/full`, Tokio `rt-multi-thread`, Tokio `process`, Tokio `signal`, Tokio `fs`, Tokio `parking_lot`, `tokio_unstable` | rejected for this network feature | The concurrency/runtime provider owns runtime topology, process, signal, and offload behavior. Network adds async socket/TLS/HTTP suspension points only. |
| `async-std`, `smol`, custom event-loop runtimes, direct `mio` | rejected | Sifr already uses Tokio and rejects public raw event-loop models. |
| `hickory-resolver` | deferred | TCP connect/address resolution uses `tokio::net::lookup_host`. In-process DNS records/custom resolver config require a separate Sifr-native resolver issue. |
| OpenSSL, `native-tls`, `openssl`, `ring` as selected provider, custom Rustls providers | rejected | TLS is Rustls with default `aws_lc_rs` provider and platform verification unless a future platform issue records a blocker. |
| `webpki-roots` fallback | rejected | Production verification uses platform roots; deterministic tests use explicit in-memory roots. No silent fallback root store. |
| `x509-parser` | deferred | Public certificate subject/issuer/SAN display is not in this phase. Errors carry typed verification evidence and raw DER fingerprints only. |
| `reqwest`, `ureq`, `isahc`, `surf` | rejected | High-level HTTP clients belong to a future Sifr-native HTTP client phase. |
| `axum`, `warp`, `actix-web`, `rocket`, `tower-http` | rejected | Server framework behavior belongs to Phase 41; this phase provides substrate only. |
| Hyper unstable `tracing`; Hyper/Hyper-Util `full`, `client-legacy`, `client-pool`, `client-proxy`, `client-proxy-system`, `ffi`, `capi`, `nightly`, unconditional `server-auto`, and unconditional `server-graceful` | rejected | These add unstable cfg requirements, policy, compatibility, proxy, pool, FFI, auto-detection, shutdown, or unstable surfaces outside the substrate boundary. Baseline graceful shutdown is Sifr-owned over provider shutdown primitives; `server-graceful` may be considered only through the conditional Hyper-Util row after proving it composes with that model. |
| `tower`, `tower-layer`, Tower utility stacks | rejected | Only `tower-service` is accepted internally; no Tower middleware/product API leaks. |
| multipart crates such as `multer` or `multipart` | rejected for this phase | Multipart parsing belongs to Phase 41 or the production HTTP client phase. |
| WebSocket crates such as `tokio-tungstenite` and `tungstenite` | deferred | Upgrade/WebSocket product APIs need a separate phase with security and backpressure decisions. |
| HTTP/3/QUIC crates such as `quinn`, `h3`, and `h3-quinn` | deferred | HTTP/3 transport strategy is explicitly out of scope. |
| `cookie` signed/private/secure/jar features | rejected for this phase | Cookie persistence, signing, encryption, and key management belong to future product phases if justified. |
| OpenTelemetry exporter/bridge crates | deferred | This phase emits `tracing` and optional `metrics` only; exporters are an observability phase decision. |

#### Resolved Ecosystem Behavior Decisions

| Decision area | Decision |
| --- | --- |
| Runtime feature boundary | Network-generated Cargo features may add Tokio `net`/`io-util`/`sync`/`time`, but they do not add a new executor topology. Serving scale v1 is single-runtime-worker per process. Any future multi-thread runtime choice, multi-process serving model, or process-worker scaling model is owned by the M0 serving-scale follow-up, not this substrate phase. |
| DNS | TCP connect and address resolution use `tokio::net::lookup_host` to respect host resolver configuration. Deterministic tests use loopback literals and host-matrix fixtures; custom record lookup is deferred. |
| TLS roots | Production client verification uses `rustls-platform-verifier`. Deterministic tests use explicit in-memory `RootCertStore` values built from `rcgen` fixtures. `webpki-roots` is not a fallback. |
| Rustls crypto provider | Use Rustls 0.23's default `aws_lc_rs` provider. Compression, early data, FIPS, custom provider, and `ring` provider choices are out of scope unless a future platform issue records a concrete blocker. |
| Stream I/O ownership | Streams use owned-buffer reads: `read_chunk(max_bytes) -> Result[Option[bytes], NetError]`, where `None` means EOF. Writes provide `write(data) -> Result[int, NetError]` and `write_all(data) -> Result[None, NetError]`. Concurrent full-duplex use requires owned split halves. TCP write-side half-close is accepted as substrate and must be specified in M0. |
| UDP | UDP remains M0-gated. M1 implements constrained `UdpSocket` only if M0 records a named near-term production consumer, such as telemetry datagrams, QUIC preparation, explicit DNS-like internal fixtures, or another named phase dependency. If accepted, the surface is limited to `bind`, `send_to`, `recv_from`, `connect`, `send`, `recv`, `local_addr`, and `close`; broadcast, multicast, raw sockets, packet options, and platform-specific socket constants are deferred or host-limited. |
| HTTP stack | `hyper` and `h2` are the accepted HTTP/1.1 and HTTP/2 protocol stack. `hyper-util` is conditional/internal-only for Tokio/Hyper adapters if M4 proves it is needed. Policy features such as pools, redirects, retries, auth, proxies, compression, cookies-as-storage, and test transports belong to the future HTTP client phase. |
| Service substrate | Use `tower-service` only, not `tower`. The trait is internal and may be hidden behind generated adapters. Public Sifr APIs expose Sifr request/response and middleware concepts. |
| Observability | Emit `tracing` spans/events and optional `metrics` counters/histograms after schema approval. The runtime/networking phase owner owns the metric schema proposal and reviewer sign-off in M0/M5. No global subscriber/recorder setup, exporter bridge, or OpenTelemetry dependency is accepted here. |
| mTLS | M2 includes client certificate authentication configuration and deterministic `rcgen` client/server certificate fixtures. The API exposes configuration, verification outcomes, and typed errors, not raw Rustls types. |
| Multipart/form | Multipart parsing is rejected for this phase and deferred to Phase 41 or the production HTTP client phase. No multipart crate is accepted. |
| Upgrade hooks | HTTP upgrade hooks are `test-only-harness` only for transport validation. Public WebSocket, CONNECT tunneling, and upgrade APIs are deferred to product phases with concrete use cases. |
| External CPython network tests | External-network CPython tests are never required for local validation. Localnet cases are converted to loopback where useful; true external tests remain `external-signal`. |

M0 must verify this table as the dependency decision record. Verification means confirming current versions, workspace feature availability, public API hiding boundary, and rejected-crate no-use checks; it does not reopen crate-family discovery. Each accepted or conditional decision must include:

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
- `async TcpStream.shutdown_write() -> Result[None, NetError]`
- `TcpStream.split() -> (TcpReadHalf, TcpWriteHalf)`
- `async TcpReadHalf.read_chunk(max_bytes) -> Result[Option[bytes], NetError]`
- `async TcpReadHalf.close() -> Result[None, NetError]`
- `async TcpWriteHalf.write(data) -> Result[int, NetError]`
- `async TcpWriteHalf.write_all(data) -> Result[None, NetError]`
- `async TcpWriteHalf.shutdown_write() -> Result[None, NetError]`
- `async TcpWriteHalf.close() -> Result[None, NetError]`
- `async TcpStream.close() -> Result[None, NetError]`
- `UdpSocket` constrained datagram support as defined in the resolved ecosystem decisions.

The API must expose local/remote address inspection, graceful shutdown, resource-limit controls, and deterministic cancellation semantics. It must not expose descriptor aliasing, monkeypatchable globals, or public raw event-loop policies.

The accepted stream ownership model is owned-buffer I/O:

- `read_chunk(max_bytes) -> Result[Option[bytes], NetError]`; `None` means EOF.
- `write(data) -> Result[int, NetError]`; returns the number of bytes accepted by the underlying stream.
- `write_all(data) -> Result[None, NetError]`; retries partial writes until completion or typed failure.

`max_bytes` must be positive and within the configured resource limit. Zero-length and too-large reads return typed errors. Cancellation of `write_all` is cancellation-safe for memory safety but may have written a prefix to the peer; this must be documented and surfaced through typed cancellation evidence rather than hidden retries.

M0 must define the full-duplex TCP ownership contract before M1 starts:

- an unsplit `TcpStream` is affine and supports sequential read/write operations;
- concurrent read/write across tasks requires `split()` into owned `TcpReadHalf` and `TcpWriteHalf` values;
- `split()` consumes a live `TcpStream` and is infallible; closed or moved streams cannot be split because the affine handle is no longer available;
- split halves are affine resources with independent read/write APIs over one underlying socket state; peer close, reset, local close, and shutdown outcomes must surface as typed `NetError`/EOF evidence from the underlying socket rather than through a local channel, cancellation token, or diagnostics substitute;
- split halves may cross task boundaries only when the compiler's sendability rules accept them;
- borrowed split views are rejected unless a future phase proves a lifetime-safe design;
- recombining split halves is rejected for v1 unless M0 records a concrete production need and a panic-free ownership design.

M0 must also define TCP half-close semantics before M1 starts:

- `shutdown_write()` sends a write-side FIN and keeps the read side usable until EOF or typed failure;
- unsplit `TcpStream.shutdown_write()` does not consume the `TcpStream`; subsequent reads remain usable until EOF or typed failure;
- `close()` closes/releases both directions and consumes the stream or split half according to the final affine-handle rules;
- repeated `shutdown_write()` behavior is either idempotent success or a typed already-shutdown outcome, but must be deterministic;
- `shutdown_write()` on a split write half propagates EOF behavior to the peer while preserving local read-half ownership;
- after successful `shutdown_write()`, subsequent `write` or `write_all` on the unsplit stream or split write half returns a stable typed write-after-shutdown error; silent no-op and panic are rejected;
- cancellation during shutdown preserves typed partial-progress evidence.

M0 must define the public Sifr byte-buffer type used by `TcpStream.read_chunk`, `TlsStream.read_chunk`, HTTP body chunks, and header/body diagnostics. Rust `bytes::Bytes` is internal only. The public Sifr byte buffer must define ownership, immutability, slicing, cloning, equality, conversion, redaction, and display/debug rules before M1 starts.

M0 must also define DNS/address-resolution semantics:

- timeout result type
- whether cancellation aborts lookup or only stops waiting
- whether partial address results can be returned
- address ordering stability
- IPv4/IPv6 behavior
- whether multi-address `connect_tcp(host)` tries one address or multiple addresses
- whether Happy Eyeballs is deferred or accepted

Default M1 policy is simple host resolution with typed timeout/cancellation and a documented ordered address list. Happy Eyeballs remains deferred unless M0 records it as substrate rather than HTTP-client transport tuning.

### TLS API Shape

The accepted public shape is:

- `TlsClientConfig` and `TlsServerConfig`
- safe default certificate verification
- production root strategy through `rustls-platform-verifier`; deterministic tests through explicit `rcgen` roots
- SNI and ALPN support
- client certificate authentication with deterministic `rcgen` client/server fixtures
- async TLS client and server streams over `TcpStream`
- typed certificate and TLS errors preserving underlying network evidence

No CPython-shaped `SSLContext` or `SSLSocket` is exposed in this phase. References to those shapes are rejected or routed to unsupported diagnostics with `sifr.tls` replacement guidance.

M2 must define TLS stream semantics for:

- `write`
- `write_all`
- `flush`
- `close`
- TLS `close_notify`
- cancellation during handshake
- cancellation during flush
- cancellation during TLS shutdown
- partial-progress evidence when plaintext was accepted but encrypted bytes were not fully flushed

TLS write semantics must account for Tokio Rustls buffering: plaintext accepted by a TLS stream is not guaranteed to have reached the underlying TCP stream until flush/shutdown completes.

### HTTP Substrate Shape

This phase builds HTTP transport substrate, not the final public web framework or HTTP client product.

Required substrate:

- crate-backed HTTP/1.1 parser/encoder integrated through Hyper and Sifr-owned wrappers
- crate-backed HTTP/2 transport, HPACK header compression, stream state machine, SETTINGS negotiation, flow control, PING, RST_STREAM, and GOAWAY handling integrated through Hyper/H2 and Sifr-owned typed error/resource wrappers
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
- upgrade hooks are `test-only-harness` only; public WebSocket, CONNECT tunneling, and upgrade APIs are deferred

Server framework behavior such as routing, middleware, extractors, validation, generated docs, and deployment ergonomics belongs to Phase 41. Client behavior such as pooling, redirects, retries, auth, proxies, cookie persistence, JSON helpers, multipart, compression, and test transports belongs to the separate production HTTP client phase.

M0 must define the stable public `sifr.http` substrate type table:

| Type | M0 decision |
| --- | --- |
| `Method` | production-substrate or stable-public utility |
| `Status` | production-substrate or stable-public utility |
| `Version` | production-substrate |
| `HeaderName` | production-substrate with ASCII/token validation |
| `HeaderValue` | production-substrate with byte/ASCII-safe behavior and text decoding blocked on text/i18n |
| `HeaderMap` | production-substrate with duplicate/ordering policy |
| `RequestHead` | production-substrate metadata only |
| `ResponseHead` | production-substrate metadata only |
| `BodyStream` | production-substrate streaming body abstraction |
| `BodyChunk` | public Sifr byte-buffer type selected by M0 |
| `Trailers` | accepted or rejected explicitly before M4 |
| `HttpError`, `ProtocolError`, `HeaderError`, `BodyError` | production-substrate typed errors with lower-layer evidence nesting |

Internal-only HTTP components:

- Hyper connection drivers
- Hyper/H2 adapters
- conditional Hyper-Util adapters
- loopback harness
- server accept/dispatch harness, which may be promoted from internal-only to production-substrate only through M0 No-Toy-Module Gate approval and Phase 41 reviewer sign-off
- client transport helpers used only for validation and future handoff

M0 must define the public body stream contract. M4 implements that M0 contract and must not redefine it:

- chunk type
- EOF behavior
- trailer support or rejection
- max chunk size
- max collected body size
- collect-with-limit helper, if any
- cancellation while reading request body
- cancellation while writing response body
- HTTP/2 stream reset mapping
- partial-progress evidence

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

CPython `asyncio` tests may be classified as `mined-as-substrate-fixture` for scheduling and transport edge cases, but Sifr must not make raw event loops, event-loop policies, callback transports/protocols, or public selector internals the primary API. Public APIs map to task, stream, TLS, and HTTP primitives.

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

### Security And Resource Model

Network/HTTP M0 must record concrete security and resource decisions for the surfaces this phase owns. These decisions are implementation inputs, not later discovery tasks.

| Concern | Phase decision |
| --- | --- |
| TLS verification defaults | Verification is on by default for clients. Disabling verification, custom trust anchors, certificate pinning, and mTLS identity mapping require explicit typed config and must be visible in diagnostics/observability without leaking secrets. Silent downgrade on verification failure is rejected. |
| TLS generated-build requirements | M2 records `aws-lc-rs` build tooling needs, CMake/toolchain requirements, cross-compilation behavior, binary-size impact, supported-host matrix rows, and generated dependency snapshots proving non-TLS programs do not build crypto providers. |
| Root store strategy | Production client verification uses platform roots through `rustls-platform-verifier`. Tests use explicit `rcgen` roots. No `webpki-roots`, local file, environment, or best-effort fallback root store is accepted in this phase. M2 records platform-verifier behavior per supported host; hosts that need extra setup, non-platform roots, or fallback behavior are `host-limited` until the generated-project story is proven. |
| Request smuggling and header normalization | Header parsing must define canonical validation for names, obs-fold rejection, duplicate header policy, whitespace normalization, `Content-Length` disagreement handling, and `Content-Length` plus chunked conflict handling before M4 starts. |
| HTTP/2 abuse | M4 must define SETTINGS limits, max concurrent streams, flow-control window defaults, max frame/body buffering, PING handling, RST_STREAM cancellation mapping, GOAWAY graceful shutdown mapping, and malformed-frame typed errors. |
| Body and header size limits | Every parser/body reader has explicit configured limits. Unbounded buffering is rejected unless an API name explicitly collects and M0 records a size cap and typed `TooLargeError`. |
| Timeouts and cancellation | Connect, accept, read, write, TLS handshake, TLS shutdown, HTTP request write, response read, and HTTP/2 stream cancellation map to the provider timeout/cancellation model and preserve partial-progress evidence. |
| URL and authority security | Userinfo redaction, host/port validation, path normalization semantics, percent-decoding boundaries, and IDNA/Unicode blocking states are recorded before `sifr.url` becomes public. Before text/i18n M2, `sifr.url` must prevent accidental Unicode/IDNA behavior by rejecting non-ASCII host input before calling `url` or accepting only ASCII and already-punycode hosts. The `url` crate's IDNA behavior may become the approved backend only after explicit text/i18n provider owner sign-off that it matches the accepted Unicode/IDNA version and canonicalization rules. |
| Cookie security | This phase parses cookie headers only. Persistence, signing, private/encrypted cookies, SameSite policy, key management, and browser-like cookie rules are rejected here or deferred to product phases. |
| Content-Encoding compression and decompression bombs | Content-Encoding compression/decompression such as gzip, brotli, or zstd is not implemented in this substrate phase. Future client/framework compression must own bomb limits and hooks; this phase only exposes body streaming limits needed by those phases. HTTP/2 HPACK is part of the accepted HTTP/2 protocol substrate and is handled through the accepted HTTP/2 stack with size and abuse limits. |
| Logging and tracing redaction | URLs with credentials, query values classified as sensitive, headers, cookies, bodies, certificate fields, peer addresses where configured, and TLS material must have redaction rules before observability hooks ship. |
| External network dependency | Validation uses loopback and deterministic fixtures only. External CPython/network tests remain `external-signal` and cannot gate local validation. |

M0 must add these rows to the shared security/resource ownership artifacts and create backlog entries for any missing diagnostic, limit, fixture, or typed error needed by M1-M5.

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
- Apply the shared platform contract from [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md), including terminal states, stability levels, host matrix rows, security/resource ownership, and cross-phase golden fixtures.
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
- Define TCP full-duplex ownership semantics, including owned split halves, task-boundary sendability, shared close/error propagation, and recombine rejection or acceptance.
- Define TCP half-close semantics, including `shutdown_write`, repeated shutdown behavior, split-half behavior, cancellation, and partial-progress evidence.
- Define the public Sifr byte-buffer type used by TCP, TLS, HTTP bodies, and diagnostics.
- Define DNS/address-resolution semantics, including timeout, cancellation, address ordering, IPv4/IPv6 behavior, multi-address connect policy, and Happy Eyeballs acceptance or deferral.
- Define TLS stream write/flush/shutdown semantics, including `close_notify`, cancellation, and partial-progress evidence.
- Define the stable `sifr.http` substrate type table and internal-only HTTP components.
- Define the HTTP body stream contract, including chunks, EOF, trailers, collect limits, cancellation, HTTP/2 reset mapping, and partial-progress evidence.
- Define URL/IDNA guard behavior before text/i18n M2 so the `url` crate cannot accidentally publish Unicode-host semantics.
- Define the complete text/i18n dependency inventory for URL, headers, bodies, cookies, certificate display, observability, diagnostics, demos, Phase 41 handoff, and HTTP client handoff.
- Define the complete concurrency/runtime dependency inventory for cancellation, deadlines, backpressure, blocking/offload, shutdown, diagnostics, task context, worker/process handoff, Phase 41 handoff, and HTTP client handoff.
- Define Phase 41 handoff contract.
- Define the v1 serving-scale boundary for Phase 41, including the named follow-up issue for multi-core serving strategy and the explicit single-runtime-worker-per-process limitation until that follow-up closes.
- Record whether the serving-scale follow-up will extend `listen_tcp` with host-limited `SO_REUSEPORT`, add a separate host-limited listener constructor, or defer `SO_REUSEPORT` from public API entirely.
- Define the separate production HTTP client phase handoff contract.
- Define the network-owned security/resource model for TLS verification, TLS generated-build requirements, root stores, request smuggling, header normalization, HTTP/2 abuse, body/header limits, URL authority validation, cookie-header security, redaction, and external-network test policy.
- Scan every CPython source/test/doc file listed in `Evidence Sources`.
- Create evidence docs for sockets/readiness, TLS, URLs, and HTTP showing each behavior's shared CPython/evidence state.

Validation:

- classification artifact proves no proposed public/internal/deferred/rejected surface lacks a state
- evidence scan proves every listed CPython test family was reviewed
- `cargo test -p sifr_stdlib`
- `cargo test -p sifr -- stdlib`
- `scripts/run_all_tests.sh --profile create-pr`

Definition of done:

- Every proposed surface is classified with a shared platform terminal state and stability level from [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md).
- Every text-dependent surface is classified as `production-substrate`, `blocked-on-text-i18n-m1`, `blocked-on-text-i18n-m2`, `blocked-on-text-i18n-m2_5`, `blocked-on-text-i18n-m3`, `deferred-to-http-client-phase`, `deferred-to-phase-41`, or `rejected`.
- Every runtime-dependent surface is classified as `production-substrate`, `blocked-on-concurrency-runtime-m1`, `blocked-on-concurrency-runtime-m2`, `blocked-on-concurrency-runtime-m3`, `blocked-on-concurrency-runtime-m5`, `blocked-on-concurrency-runtime-m6`, `deferred-to-http-client-phase`, `deferred-to-phase-41`, or `rejected`.
- No module is accepted merely because CPython has it.
- Dependency decision records are present and checked in for every crate family in the Rust Ecosystem Decisions table, covering accepted crate and feature flags, Sifr abstraction that hides the crate from public APIs, panic/unsafe audit for user-controlled data paths, typed error mapping into Sifr variants, license/MSRV/binary-size/platform impact, deterministic local test strategy, conformance evidence for protocol crates, and supply-chain/maintenance signal.
- Security/resource rows are checked into the shared platform artifacts with concrete limits, redaction rules, typed errors, and deterministic fixtures for every network-owned concern.
- Public byte-buffer, DNS, TLS stream, `sifr.http` type, HTTP body stream, and URL/IDNA guard contracts are checked in with concrete backlog entries.
- Stream I/O ownership, lifetime, full-duplex split, half-close, cancellation, and partial read/write semantics are decided before M1 starts.
- Phase 41 serving-scale handoff is checked in: this phase provides single-runtime-worker-per-process production-correct serving substrate, and M0 has created or linked the multi-core serving follow-up issue with a stable identifier recorded in this phase doc.
- M1-M5 implementation PRs have concrete backlog entries rather than prose-only scope.

### milestone_network_http_1: Async Network Runtime

Scope:

- Implement `sifr.net` as the primary low-level networking API.
- Add TCP client/server streams:
  - async connect
  - async listen
  - async accept
  - async read/write
  - owned full-duplex split into read/write halves
  - write-side half-close
  - close and graceful shutdown
  - local/remote address inspection
- Add DNS/address resolution with typed errors, deterministic timeout behavior, and the M0 address ordering / IPv4 / IPv6 / multi-address connect policy.
- Add cancellation, backpressure, and resource limits.
- Add constrained UDP datagram support only if M0 records a named near-term production consumer:
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
- TCP full-duplex split loopback tests pass with concurrent read/write tasks, typed cancellation evidence, and no shared mutable stream aliasing.
- TCP half-close loopback tests pass for request-end signaling while the read side remains usable.
- If M0 accepts constrained UDP, UDP loopback tests pass for the accepted datagram surface; otherwise UDP remains `deferred-to-phase-X` or `rejected` with rationale and no partial public API.
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
- Add TLS `flush`, `close`, and `close_notify` behavior according to the M0 TLS stream contract.
- Add typed TLS and certificate errors.
- Preserve nested network evidence inside TLS errors.
- Record generated-build requirements for `aws-lc-rs`, including CMake/toolchain needs, cross-compilation behavior, binary-size impact, supported-host rows, and generated dependency snapshots proving non-TLS programs do not build crypto providers.
- Record `rustls-platform-verifier` behavior per supported host and mark any host with unproven setup/fallback behavior as `host-limited`.
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
- TLS write, flush, shutdown, and cancellation tests cover `close_notify` and partial-progress evidence.
- Generated build snapshots prove TLS dependencies are feature-gated and absent from non-TLS generated programs.
- Platform-verifier host behavior is recorded in the shared supported-host matrix.

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
- Publish M3 primitives into two namespaces: URL and query APIs under `sifr.url`, and header/cookie-header protocol primitives under `sifr.http`.
- Own the canonical header primitives consumed by M4 HTTP transport; M4 must not define duplicate header-name, header-value, or cookie-header representations.
- Implement small cookie header parsing required by real HTTP request/response handling.
- Keep cookie persistence and jar policy out of this phase.
- Record non-UTF-8 codec-dependent behavior as `blocked-on-text-i18n-m1`; do not duplicate codec registry behavior locally.
- Keep Unicode/IDNA host canonicalization `blocked-on-text-i18n-m2`; M3 accepts ASCII and already-punycode host behavior only until the text/i18n provider defines Unicode alignment.
- Enforce the M0 URL/IDNA guard so non-ASCII host behavior cannot ship as an accidental side effect of the `url` crate before text/i18n M2.
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
- Non-ASCII host behavior before text/i18n M2 is rejected or blocked with typed diagnostics; the `url` crate's IDNA behavior does not leak accidentally.
- Cookie persistence is not exposed as a partial core API.

### milestone_network_http_4: HTTP Core Transport

Scope:

- Integrate crate-backed HTTP/1.1 parser/encoder through Hyper and Sifr-owned wrappers.
- Integrate crate-backed HTTP/2 transport, HPACK, flow-control, SETTINGS, RST_STREAM, PING, and GOAWAY behavior through Hyper/H2 and Sifr-owned typed error/resource wrappers.
- Implement typed request/response model.
- Implement method, status, version, and body types while consuming the M3 URL/header/cookie primitives.
- Implement body streaming without unbounded buffering according to the M0 body stream contract.
- Implement content-length and chunked transfer handling.
- Implement keep-alive and connection lifecycle.
- Implement request/response/header/body limits.
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
- conditional `hyper-util` only if M4 proves Hyper alone requires substantial custom runtime adapter code
- `h2`
- `tower-service` crate internally for Phase 41 handoff
- avoid pulling a web framework into the substrate

Definition of done:

- HTTP/1.1 and HTTP/2 loopback client/server transport tests pass without external network.
- HTTPS transport works through M2 TLS, including ALPN selection for HTTP/2.
- Malformed HTTP tests produce typed protocol errors.
- Body streaming and HTTP/2 multiplexing work without unbounded buffering.
- The M0 `sifr.http` substrate type table and body stream contract are implemented or explicitly deferred/rejected with rationale.
- HTTP/2 protocol-level behaviors selected in the M0 conformance inventory, including SETTINGS negotiation, RST_STREAM stream cancellation, GOAWAY graceful shutdown, and HPACK correctness edge cases, have loopback test coverage.
- HTTP transport stores and forwards binary bodies and typed protocol metadata without local text decoding fallbacks.
- Server transport handoff documentation states that this phase is single-runtime-worker per process until the M0 serving-scale follow-up closes.
- No `http.server`, `socketserver`, or handler-subclass public API is added.

### milestone_network_http_5: Integration, Documentation, And Production Handoff

Scope:

- Update public docs for:
  - `sifr.net`
  - `sifr.tls`
  - `sifr.url`
  - public HTTP protocol/substrate types under `sifr.http`
  - rejected or unsupported CPython-shaped surfaces and why they are not recommended APIs
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
  - every proposed surface has a shared terminal state and stability level
  - every text/i18n-dependent and runtime-dependent surface has a provider milestone state
  - every CPython evidence family has a shared evidence state
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
- Phase 41 can build routing, middleware, lifecycle, request/response pipeline, typed extractors, validation, and production hooks on the substrate, with multi-core serving throughput explicitly deferred to the M0 serving-scale follow-up.
- Phase 41 handoff documentation states that multipart/form parsing, WebSocket/upgrade products, Content-Encoding compression, and HTTP/3/QUIC are outside this substrate and require separate accepted product/transport phases before Phase 41 may claim those capabilities.
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
- `verification/platform/platform_contract.md`
- `verification/platform/platform_contract.json`
- `verification/platform/supported_host_matrix.md`
- `verification/platform/golden/manifest.json`
- `scripts/run_platform_golden.sh`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test/doc files scanned
- mined-as-substrate-fixture/adapted-for-sifr-api/blocked-on-phase-X/external-signal/waived-with-rationale/rejected CPython evidence families; `compat-adapter-deferred` is recorded only as a shared vocabulary state intentionally unused by this phase
- final deferred-to-phase-X/rejected/host-limited/internal-only/unsupported-with-diagnostic decision index
- text/i18n dependency state for every URL, header, body, cookie, certificate-display, observability, diagnostics, demo, Phase 41, and HTTP client handoff surface
- concurrency/runtime dependency state for every cancellation, timeout, backpressure, blocking/offload, shutdown, diagnostics, task-context, executor/worker, process, Phase 41, and HTTP client handoff surface
- cross-phase golden fixture entries and skip/pass status for network-owned contracts
- security/resource ownership rows for network-owned concerns

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
- Any external crate or broad feature flag not listed in the M0 Rust Ecosystem Decisions table requires a new issue or explicit phase amendment before implementation; implementation PRs must not perform crate-family discovery.
- Every public module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.
- Every public network/web API must pass the No-Toy-Module Gate and Maintenance Burden Test.

## Resolved Planning Decisions For M0

M0 validates these decisions and records dependency audit evidence; it does not reopen them without a concrete blocking finding.

1. Public paths are `sifr.net`, `sifr.tls`, `sifr.url`, and `sifr.http`; `_sifr.*` and Rust modules remain implementation details.
2. Production TLS verification uses `rustls-platform-verifier`; deterministic tests use explicit `rcgen` roots.
3. HTTP transport uses `hyper` and `h2`; conditional `hyper-util` is internal-only and allowed only if M4 proves Hyper alone would require substantial custom runtime adapter code. No web framework or high-level client crate enters this substrate.
4. Host-specific socket options are limited to the socket2 list above and recorded as portable or host-limited.
5. External-network CPython tests are `external-signal`; useful localnet behavior is converted to loopback.
6. UDP is M0-gated. M1 implements a constrained datagram API only if M0 records a named near-term production consumer; advanced datagram features are deferred or host-limited.
7. Stable HTTP substrate types live under `sifr.http`; public client/server products remain future phases.
8. Serving scale v1 is single-runtime-worker per process. Multi-core serving throughput is not hidden inside this substrate phase; M0 must create or link the follow-up issue that owns multi-process serving, host-limited `SO_REUSEPORT`, process workers, or future provider-owned multi-thread runtime topology.
9. TCP full-duplex is supported through owned split halves, not shared mutable stream aliasing. TCP write-side half-close is part of the M1 substrate.
