# Ad Hoc Phase: Production Network And Web Stdlib Parity

Status: draft
Phase placement: ad hoc expansion phase after the stdlib boundary refactor and before any stable GA claim that Sifr is production-ready for networked programs.
Phase owner: stdlib/runtime implementation with compiler import, effect, and codegen support

## Objective

Close the production stdlib gaps that prevent Sifr from supporting common networked Python-shaped programs:

- networking and readiness: `socket`, `select`, `selectors`
- TLS: `ssl`
- URL and HTTP: `urllib.*`, `http.*`, `socketserver`

This phase is complete when each target surface has either:

- current-CPython-shaped source parity with Sifr-safe semantics,
- a native Sifr async/runtime implementation that backs that compatibility surface,
- or an explicit, tested waiver with rationale, revisit rule, and CPython test-family evidence.

This phase does not add backward-compatibility or legacy support. Parity means the current supported CPython stdlib API shape and behavior adapted under Sifr's canonical `sifr.*` namespace with Sifr's static, typed, ownership-safe model. Bare CPython stdlib imports, historical aliases, deprecated APIs, compatibility shims, and hidden bridge names are not implemented; they receive diagnostics or waivers.

## Split-Out Phases

The original broad planning scan also covered two important areas that are now tracked as separate ad hoc phases:

- [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md): `queue`, `subprocess`, `asyncio.subprocess`, `concurrent.futures`, `multiprocessing`, `contextlib`, `warnings`, `signal`
- [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md): `codecs`, `encodings`, `unicodedata`, `locale`, `gettext`

This phase may depend on those phases for optional text decoding, subprocess demos, or executor-backed serving, but it must not implement their module surfaces here.

This phase also depends on [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md). Its namespace contract is assumed complete before these stdlib parity milestones ship: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.

## Cross-Phase Dependency Contract

The three split phases are not an implied ship order. Each phase may implement and test its self-contained binary/runtime subset independently, but cross-phase consumer features are blocked until their provider phase is complete:

- Text/i18n is a hard prerequisite for non-UTF-8 HTTP body decoding, file/text handlers that require codec lookup, and any network demo that depends on `open(..., encoding=...)`.
- The precise unblock point for those text-dependent network features is completion of text/i18n `milestone_text_i18n_1: Codecs Registry, Encodings, And Text I/O Integration`; M0/M3 network inventory entries must record those surfaces as `blocked-on-text-i18n-m1` until that milestone is closed.
- Concurrency/runtime is a hard prerequisite for executor-backed server APIs. This phase does not implement public thread, executor, queue, process, warning, or signal modules.
- Async scheduler/task primitives are prior runtime infrastructure owned by the existing async model. This phase owns only network-specific compatibility additions such as `asyncio.open_connection` and `asyncio.start_server`.
- Binary socket, TLS, URL parsing, cookie parsing, and loopback HTTP tests can ship before the text/i18n phase as long as they do not duplicate the codec registry.

## Source Of Truth

The authoritative CPython source tree for this phase is:

- `/Users/yaseralnajjar/work/sifr/cpython`

The implementation must scan and classify these CPython files before each milestone implementation PR:

| Domain | CPython library sources | CPython test sources | Native backing sources |
| --- | --- | --- | --- |
| sockets/selectors | `Lib/socket.py`, `Lib/selectors.py`, `Doc/library/socket.rst`, `Doc/library/selectors.rst`, `Doc/library/select.rst` | `Lib/test/test_socket.py`, `Lib/test/test_select.py`, `Lib/test/test_selectors.py`, `Lib/test/test_asyncio/test_streams.py`, `Lib/test/test_asyncio/test_server.py`, `Lib/test/test_asyncio/test_sock_lowlevel.py`, `Lib/test/test_asyncio/test_selector_events.py` | `Modules/socketmodule.c`, `Modules/selectmodule.c`, `Modules/clinic/socketmodule.c.h`, `Modules/clinic/selectmodule.c.h` |
| TLS | `Lib/ssl.py`, `Doc/library/ssl.rst` | `Lib/test/test_ssl.py`, `Lib/test/test_asyncio/test_ssl.py`, `Lib/test/test_asyncio/test_sslproto.py` | `Modules/_ssl.c`, `Modules/_ssl/*`, `Modules/clinic/_ssl.c.h` |
| HTTP and URLs | `Lib/http/*.py`, `Lib/urllib/*.py`, `Lib/socketserver.py`, `Doc/library/http*.rst`, `Doc/library/urllib*.rst`, `Doc/library/socketserver.rst` | `Lib/test/test_httplib.py`, `Lib/test/test_httpservers.py`, `Lib/test/test_http_cookies.py`, `Lib/test/test_http_cookiejar.py`, `Lib/test/test_socketserver.py`, `Lib/test/test_urllib.py`, `Lib/test/test_urllib2.py`, `Lib/test/test_urllib2_localnet.py`, `Lib/test/test_urllib_response.py`, `Lib/test/test_urllibnet.py`, `Lib/test/test_urllib2net.py` | stdlib Python sources plus Rust HTTP/TLS/runtime crates selected by this phase |

Path note: CPython paths above are relative to `/Users/yaseralnajjar/work/sifr/cpython`.

## Current Sifr Baseline

Current Sifr stdlib support is intentionally curated under `lib/sifr/*.sifr`. Relevant existing surfaces:

- `sifr.asyncio` is a compatibility veneer over the canonical task model, but intentionally omits raw event loops, public selectors, subprocesses, process pools, and transport/protocol APIs.
- `sifr.io` has file handles and in-memory stream wrappers, but no socket streams.
- `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib`, `sifr.socketserver`, and public `sifr.http` modules are not present as production stdlib surfaces.
- `sifr.asyncio` already owns the core scheduler/task veneer (`run`, `create_task`, `gather`, `sleep`, timeout helpers). This phase consumes that runtime and adds only network stream compatibility entry points.

The Phase 32 async model remains binding:

- Native async I/O APIs must be real suspension points.
- Sync network APIs that can block must be classified as `@blocking_io`.
- Direct calls to blocking sync APIs from `async def` remain compiler errors unless routed through native async APIs or explicit offload.
- The compiler must not expose Tokio, event-loop objects, or raw callback-first APIs as the normal user model.

## Parity Definition

This phase targets current CPython-shaped interfaces under the canonical `sifr.*` namespace, not legacy compatibility layers or bare CPython import compatibility.

For each module in scope:

1. Support canonical Sifr stdlib imports for the CPython-shaped surface (`from sifr.socket import socket`, `from sifr.urllib.parse import urlparse`, etc.).
2. Do not add bare CPython module-name imports as aliases for `sifr.*`. Bare forms such as `from socket import socket` or `from urllib.parse import urlparse` should receive the namespace-contract diagnostic once normal user/package resolution fails.
3. Match CPython function/class names, constructor forms, constants, and common keyword arguments where compatible with Sifr's static type system.
4. Adapt CPython exception behavior into Sifr-safe `Result[T, E]`, `Option[T]`, or compile-time diagnostics.
5. Keep host-specific behavior explicitly marked `host-limited`.
6. Keep CPython implementation-detail, deprecated, and historical compatibility behavior waived rather than reimplemented blindly.

Every reviewed CPython test family must end in exactly one state:

- `adopted`: direct Sifr equivalent added.
- `adapted`: behavior preserved with Sifr `Result`/`Option`, ownership, async, or host-limited adaptation.
- `waived`: explicit unsupported/intentional-diff/host-limited rationale recorded.

Every public surface must end in exactly one state:

- `done`
- `intentional-diff`
- `unsupported`
- `host-limited`

`open` is allowed during implementation only and is forbidden at phase exit.

## Milestone Dependency Graph

Implementation PRs must follow this dependency order unless the execution ledger records an explicit split that preserves the same prerequisites:

1. `milestone_network_web_0` first. No implementation milestone starts until the inventory, CPython test matrix, import plan, and shared error mapping are checked in.
2. `milestone_network_web_1` before network-dependent HTTP/TLS/server work. Socket readiness and async streams are the substrate for TLS and HTTP.
3. `milestone_network_web_2` before HTTPS client/server support in `milestone_network_web_3`. M3 may start URL parsing and plain HTTP work after M1, but HTTPS and async HTTPS wait for M2 `AsyncTlsStream`.
4. `milestone_network_web_4` last, after every target surface and CPython test family in this phase is closed as `done`, `intentional-diff`, `unsupported`, or `host-limited`.

Parallel work is allowed only for pure parser work that does not consume unfinished runtime substrate.

## Architecture Principles

### Native Runtime First, Compatibility Second

Implement the canonical runtime primitive first, then layer CPython-shaped modules over it.

- Tokio remains the backing async runtime for this phase because the generated task runtime already depends on `tokio` and `sifr_stdlib::StdlibFeature::Tokio`.
- M0 must expand the Tokio dependency feature plan from the current task/sync/time set to the concrete features needed for `tokio::net` and `tokio::io`.
- No `async-std`, custom event-loop runtime, or public Tokio type is introduced without a separate architecture issue.
- `sifr.net` / private intrinsics own TCP/UDP/socket readiness.
- `sifr.tls` / private intrinsics own TLS handshakes and certificate verification.
- `sifr.http` / private intrinsics own request/response transport.
- CPython-shaped canonical Sifr modules (`sifr.socket`, `sifr.ssl`, `sifr.urllib.request`, `sifr.http.client`, `sifr.http.server`, `sifr.socketserver`) delegate to those primitives.

The exact internal module names may change during implementation, but the public stdlib namespace remains `sifr.*` and the boundary must exist: public modules must not duplicate target-runtime logic.

### Async Counterpart Rule

Any blocking production API added in this phase must have one of:

- a native async counterpart,
- a documented reason why async is not meaningful,
- or an explicit `@blocking_io` classification plus approved offload-only guidance.

Required native async counterparts:

- TCP connect/listen/accept/read/write/close
- TLS connect/accept/handshake/read/write/close
- HTTP client request/response body streaming
- HTTP server accept/request dispatch/shutdown

M0 must add every network/web API to the stdlib workload database. The first table must include at least:

| API family | Classification | Async-context behavior |
| --- | --- | --- |
| `socket.connect`, `accept`, `recv`, `recv_into`, `send`, `sendall`, `sendto`, `recvfrom`, DNS helpers, `create_connection`, `create_server` | sync `@blocking_io` | compile-time diagnostic suggesting native async network APIs or explicit offload |
| `select.select`, selector `select`, blocking readiness waits | sync `@blocking_io` | compile-time diagnostic suggesting async readiness/stream APIs or explicit offload |
| `ssl.SSLContext.wrap_socket`, sync TLS handshake/read/write/shutdown | sync `@blocking_io` | compile-time diagnostic suggesting async TLS APIs or explicit offload |
| `urllib.request.urlopen`, `http.client` request/response body operations, `http.server`/`socketserver` serve loops | sync `@blocking_io` | compile-time diagnostic suggesting native async HTTP/server APIs or explicit offload |
| `asyncio.open_connection`, `asyncio.start_server`, async TLS, and async HTTP APIs | async-native | legal suspension points |
| pure URL parsing, percent-encoding, cookies, robotparser parsing | pure, or `@cpu_heavy` if M0 finds size-dependent paths | legal unless the exact path is marked `@cpu_heavy` |

Implementation milestones cannot claim CPython conformance for a blocking family until its workload entries and async diagnostics are checked in.

### No Raw Event Loop As Public Model

CPython `asyncio` tests must be mined for behavior, but Sifr must not make raw event loops, event-loop policies, callback transports/protocols, or public selector internals the primary API. Compatibility functions should map to `task`, `sync`, stream, and network primitives.

### Server Handler Model Without Python Inheritance

`socketserver` and `http.server` handler classes rely on CPython subclass dispatch. Sifr must not emulate dynamic class inheritance. M0 must choose and document one static handler abstraction before M3 implementation starts:

- preferred: generated trait-based handlers with statically typed `handle`, `setup`, and `finish` hooks for `BaseRequestHandler`/`BaseHTTPRequestHandler`-shaped source
- allowed adaptation: enum or closure callback dispatch when CPython subclassing cannot be represented safely
- unsupported forms: monkeypatching handler methods, dynamic attribute lookup, or inheritance patterns that cannot lower to a known trait/callback shape

M3 fixtures must prove `HTTPServer(..., Handler)` and `socketserver.TCPServer(..., Handler)` route requests through the selected abstraction and must mark unsupported subclass forms with diagnostics.

### Typed Errors Instead Of Exceptions

All fallible APIs must expose typed error results:

- `SocketError`, `AddressError`, `TimeoutError`, `ConnectionError`
- `SSLError`, `CertificateError`
- `HTTPError`, `URLError`, `CookieError`

Names may align with CPython where possible, but the operational contract is Sifr `Result`/`Option`, not exception-driven control flow.

`milestone_network_web_0` must add a shared error mapping document before M1 implementation:

- map CPython `OSError`/`errno` families into stable Sifr variants used by sockets, TLS, selectors, and HTTP
- define which current CPython exception class names remain importable from canonical `sifr.*` modules as typed error constructors for parity; historical aliases are not imported for backward compatibility
- add cross-module regression tests proving equivalent failures use the same Sifr error family
- define a concrete typed error hierarchy before M1 starts:
  - `SocketError` is the base network transport error and carries address, timeout, readiness, descriptor, and platform/errno variants
  - `TlsError`/`SSLError` wraps TLS handshake, verification, certificate, ALPN, and wrapped-stream failures and preserves the underlying `SocketError` when transport is the root cause
  - `HttpError`/`URLError` wraps invalid URL, unsupported encoding, connection, redirect, bad status line, incomplete read, remote disconnect, proxy, cookie, and robotparser failures and preserves nested socket/TLS error evidence
  - modern exception names such as `ssl.SSLError`, `urllib.error.URLError`, and `http.client.HTTPException` map to typed constructors, never exception-only control flow; legacy aliases such as `socket.error` are unsupported

### Panic-Free Runtime Contract

No user-triggerable runtime panics are allowed. Generated Rust for these APIs must not contain data-dependent `.unwrap()`, `.expect()`, or `panic!` on user-controlled network, TLS, URL, cookie, or HTTP data.

## Non-Goals And Permanent Boundaries

The following are not accepted as silent omissions. They must be either implemented or explicitly waived with tests:

- platform-specific constants and address families not available on the host
- CPython refcount/finalizer behavior
- dynamic monkeypatching of module globals
- raw event-loop policy mutation
- callback transport/protocol APIs as the primary Sifr model
- `socketserver.ThreadingMixIn` and `socketserver.ForkingMixIn`; both are unsupported in this phase
- `http.server.ThreadingHTTPServer`; unsupported in this phase
- process, queue, signal, warning, locale, codec, Unicode, or gettext APIs; those belong to the split-out phases

## Milestones

### milestone_network_web_0: CPython Inventory And Harness Lock

Scope:

- Add a machine-readable parity inventory under `verification/stdlib/network_web_parity_inventory.*`.
- Scan every source/test/doc file listed in `Source Of Truth`.
- Extract public functions, classes, constants, methods, common keyword forms, deprecation/legacy markers, and test-class/test-method names.
- Create module-level CPython traceability docs for sockets/selectors, TLS, and HTTP/URL domains.
- Add CPython-derived e2e fixtures:
  - `cpython_socket_subset.sifr`
  - `cpython_socketserver_subset.sifr`
  - `cpython_ssl_subset.sifr`
  - `cpython_selectors_subset.sifr`
  - `cpython_urllib_parse_subset.sifr`
  - `cpython_urllib_response_subset.sifr`
  - `cpython_http_client_subset.sifr`
  - `cpython_http_server_subset.sifr`
- Add import-resolution tests for canonical `sifr.*` module names and negative diagnostics for bare CPython stdlib import attempts.
- Add shared error mapping for all network/web target domains.
- Add the concrete network/TLS/HTTP typed error hierarchy required by `Typed Errors Instead Of Exceptions`.
- Add workload classifications and async-context diagnostics for socket, select/selectors, TLS, HTTP client/server, and URL/cookie/parser APIs.
- Decide the `socketserver`/`http.server` handler abstraction used to adapt CPython subclass dispatch.
- Assign each inventory entry one owner milestone and one terminal state. Duplicate ownership is forbidden unless the entry is explicitly shared infrastructure.
- Assign every deprecated, historical, or legacy-only entry the terminal state `unsupported` or `intentional-diff`. M0 may implement only current, non-deprecated target CPython surfaces that remain elegant under Sifr semantics.

Validation:

- inventory generator/test proves no target module lacks a surface state
- `cargo test -p sifr_stdlib`
- `cargo test -p sifr -- stdlib`
- `scripts/run_all_tests.sh --profile create-pr`

Definition of done:

- The implementation backlog is derived from CPython source/tests, not hand-written memory.
- Every target module has a first-pass surface matrix and CPython test-family matrix.
- Canonical `sifr.*` module imports are explicitly planned and regression-tested.
- M1-M4 implementation PRs have concrete backlog entries rather than prose-only scope.

### milestone_network_web_1: Socket, Select, Selectors, And Async Network Streams

Scope:

- Add `sifr.socket`, `sifr.select`, and `sifr.selectors` CPython-shaped modules. Do not add bare CPython stdlib import aliases.
- Implement TCP IPv4/IPv6 client/server basics:
  - `socket.socket`
  - `socketpair`
  - `fromfd`, `dup`, `close`
  - `bind`, `listen`, `accept`
  - `connect`, `connect_ex`
  - `send`, `sendall`, `recv`, `recv_into` where compatible with Sifr bytes/buffer model
  - `shutdown`, `close`
  - `detach` only if ownership semantics are safe; otherwise explicit waiver
  - `settimeout`, `gettimeout`, `setblocking`
  - `getdefaulttimeout`, `setdefaulttimeout`
  - `getsockname`, `getpeername`
  - `family`, `type`
  - `create_connection`, `create_server`
  - `getaddrinfo`, `getnameinfo`, `gethostname`, `getfqdn`
  - `gethostbyname`, `gethostbyaddr`, `getservbyname`, `getservbyport`, `getprotobyname`
  - `inet_aton`, `inet_ntoa`, `inet_pton`, `inet_ntop`
  - address-family, socket-type, protocol, message, shutdown, and option constants discovered from `Modules/socketmodule.c`
- Classify low-level descriptor APIs:
  - `detach`/`fromfd`/`dup` are adopted only if ownership transfer is statically safe.
  - If a raw descriptor would allow double-close or aliasing, the API is `intentional-diff` with a CPython test adaptation.
  - Host-only constants are `host-limited` with generated inventory evidence.
- Implement UDP basics:
  - `sendto`, `recvfrom`
  - address tuple adaptation with typed address structs where needed
- Implement selector readiness APIs as compatibility over runtime readiness:
  - `EVENT_READ`, `EVENT_WRITE`
  - `SelectorKey`
  - `DefaultSelector`
  - `SelectSelector`
  - `BaseSelector.register`, `unregister`, `modify`, `select`, `close`, `get_key`, `get_map`
- Add native async network streams:
  - `asyncio.open_connection` compatibility over canonical Sifr streams
  - `asyncio.start_server` compatibility over canonical Sifr streams
  - async connect, accept loop, read/write, and close
  - async iteration over incoming bytes/lines where appropriate
- Mark sync socket APIs as `@blocking_io`.
- Ensure async code gets diagnostics suggesting native async network APIs instead of direct sync calls.

CPython tests to mine:

- `Lib/test/test_socket.py`
- `Lib/test/test_select.py`
- `Lib/test/test_selectors.py`
- `Lib/test/test_asyncio/test_streams.py`
- `Lib/test/test_asyncio/test_server.py`
- `Lib/test/test_asyncio/test_sock_lowlevel.py`
- `Lib/test/test_asyncio/test_selector_events.py`

Rust/runtime candidates:

- `std::net` for sync compatibility
- `tokio::net` for async network operations
- `socket2` if low-level socket option coverage requires it

Definition of done:

- TCP/UDP loopback tests pass deterministically without external network dependency.
- Timeout and nonblocking behavior is typed, deterministic, and panic-free.
- Async network APIs are real `AsyncIo` suspension points.
- Selector compatibility is implemented or explicitly waived per platform.

### milestone_network_web_2: TLS And SSL

Scope:

- Add `ssl` CPython-shaped module and any native TLS helper module needed.
- Implement:
  - protocol/version constants and `TLSVersion`
  - `Purpose`
  - `SSLContext`
  - `create_default_context`
  - `get_default_verify_paths`
  - `cert_time_to_seconds`
  - `DER_cert_to_PEM_cert`
  - `PEM_cert_to_DER_cert`
  - `get_server_certificate`
  - `SSLContext.load_verify_locations`
  - `SSLContext.load_default_certs`
  - `SSLContext.wrap_socket`
  - `SSLContext.set_alpn_protocols` where the selected Rust TLS backend supports it
  - `SSLSocket` read/write/recv/send/shutdown/unwrap/selected_alpn_protocol/cipher/version
- Add async TLS:
  - canonical `AsyncTlsStream`-style runtime primitive over the M1 async stream type
  - `asyncio.open_connection(..., ssl=...)` compatibility over that primitive
  - `asyncio.start_server(..., ssl=...)` compatibility over that primitive
  - async client handshake
  - async server handshake
  - async read/write/close
  - certificate verification failures as typed `SSLError`/`CertificateError`
- Do not expose CPython's event-loop-driven `SSLWantReadError`/`SSLWantWriteError` retry model as the public Sifr async API. Mine those tests for readiness and retry behavior, then adapt them to real suspension points.
- `SSLContext.wrap_socket` is sync-only in this phase. Async TLS uses the canonical async stream constructor/API added by M2, not an overloaded `wrap_socket`.
- Mark sync TLS socket operations as `@blocking_io`.
- Preserve safe ownership around wrapped sockets:
  - `SSLContext.wrap_socket(sock, ...)` consumes/moves the plain socket handle and returns `Result[SSLSocket, SSLError]`
  - after a successful wrap, the original plain socket variable is invalid and cannot be used, closed, detached, or aliased independently
  - failed wrapping returns `Err(TlsWrapError { socket_state, error })` or equivalent; `socket_state` is either `Recovered(Socket)` when the underlying descriptor remains usable or `Closed` when the backend consumed/closed it during failure
  - the original plain socket variable is invalid on both success and failure; on failure, the only way to recover a usable plain socket is through `TlsWrapError.socket_state`
  - transport-root failures preserve nested `SocketError` inside the TLS error evidence
  - `SSLSocket.unwrap()` consumes the TLS handle and returns the underlying plain socket only after a successful TLS shutdown; failure returns typed error evidence without leaving two mutable handles alive

CPython tests to mine:

- `Lib/test/test_ssl.py`
- `Lib/test/test_asyncio/test_ssl.py`
- `Lib/test/test_asyncio/test_sslproto.py`

Rust/runtime candidates:

- `rustls`
- `tokio-rustls`
- `webpki-roots` or platform-native roots, selected explicitly
- `x509-parser` only if certificate inspection requires it

Definition of done:

- Local self-signed and CA-backed handshake fixtures are deterministic.
- Host-network certificate tests are either disabled by default or recorded as non-blocking ecosystem signal.
- TLS verification errors are typed and never panic.

### milestone_network_web_3: URL Parsing, HTTP Client, HTTP Server, Cookies, And Robots

Scope:

- Add `urllib.parse`, `urllib.request`, `urllib.response`, `urllib.error`, `urllib.robotparser`.
- Add `http`, `http.client`, `http.server`, `http.cookies`, `http.cookiejar`.
- Add `socketserver` as the HTTP server substrate:
  - `BaseServer`
  - `TCPServer`
  - `UDPServer` only if UDP service ownership can be tested deterministically
  - `ThreadingMixIn` is `unsupported` in this phase; equivalent concurrent serving uses Sifr task/spawn APIs or explicit executor APIs after the concurrency/runtime phase
  - `ForkingMixIn` is `unsupported` in this phase because fork semantics are owned by the concurrency/runtime phase and are host-limited even there
  - `BaseRequestHandler`, `StreamRequestHandler`, `DatagramRequestHandler`
- Implement URL parsing and quoting:
  - `urlparse`, `urlsplit`, `urlunparse`, `urlunsplit`, `urljoin`, `urldefrag`
  - `quote`, `quote_plus`, `quote_from_bytes`
  - `unquote`, `unquote_plus`, `unquote_to_bytes`
  - `urlencode`
  - `parse_qs`, `parse_qsl`
  - `ParseResult`, `SplitResult`, byte variants where compatible
  - percent-encoding and URL quoting use byte/ASCII and UTF-8-compatible behavior in this phase only
  - non-ASCII/non-UTF-8 forms such as `quote(value, encoding="latin-1")` are `blocked-on-text-i18n-m1` until text/i18n `milestone_text_i18n_1` ships the codec registry, core encodings, and typed codec errors; they must not be reimplemented locally
  - before `milestone_text_i18n_1`, statically visible non-UTF-8 `encoding=` arguments produce a compile-time unsupported-codec diagnostic; dynamic non-UTF-8 encoding values return a typed `UnsupportedEncodingError`/`URLError` result, never silent UTF-8 coercion or panic
- Implement HTTP client:
  - `HTTPConnection`, `HTTPSConnection`
  - `HTTPResponse`
  - `request`, `getresponse`, `close`, `set_tunnel`
  - headers and body reading
  - typed errors for invalid URL, bad status line, incomplete read, remote disconnect, timeout
- Implement higher-level URL request basics:
  - `Request`
  - `urlopen`
  - `build_opener` minimal supported handler matrix
  - redirects
  - proxy environment support only if deterministic and documented
  - file/data handlers only if Sifr path/bytes semantics are compatible
- Implement HTTP server basics:
  - `HTTPServer`
  - `ThreadingHTTPServer` is `unsupported` in this phase
  - `BaseHTTPRequestHandler`
  - `SimpleHTTPRequestHandler`
  - graceful shutdown
- Implement cookies:
  - `Morsel`
  - `SimpleCookie`
  - `Cookie`
  - `CookieJar`
  - `DefaultCookiePolicy`
  - file-backed cookie jars only if safe file I/O support is sufficient
- Implement `RobotFileParser` and pure parser helpers.
- Add native async HTTP APIs with concrete entry points:
  - `sifr.http.async_request` or final equivalent canonical API for one-shot requests
  - async request builder with streaming request body support
  - async response object with streaming body reads
  - async server accept/dispatch/shutdown over M1 async streams and M2 async TLS for HTTPS
  - `urllib.request.urlopen` remains sync `@blocking_io`; any async compatibility wrapper must have a distinct async name and real suspension behavior
  - no raw event-loop, callback transport, or selector object is exposed as the user-facing HTTP API

CPython tests to mine:

- `Lib/test/test_httplib.py`
- `Lib/test/test_httpservers.py`
- `Lib/test/test_http_cookies.py`
- `Lib/test/test_http_cookiejar.py`
- `Lib/test/test_socketserver.py`
- `Lib/test/test_urllib.py`
- `Lib/test/test_urllib2.py`
- `Lib/test/test_urllib2_localnet.py`
- `Lib/test/test_urllib_response.py`
- `Lib/test/test_urllibnet.py` and `Lib/test/test_urllib2net.py` as external-network, non-blocking signal unless converted to loopback

Rust/runtime candidates:

- `url`
- `percent-encoding`
- `http`
- `hyper`
- `hyper-util`
- `reqwest` only if its API/dependency size is accepted by review
- `tower`/`axum` only if server milestone needs them; avoid pulling a web framework into stdlib unless justified

Definition of done:

- URL parsing has CPython-derived edge-case fixtures.
- URL parsing fixtures include an explicit split between byte/ASCII/UTF-8 behavior owned here and non-UTF-8 codec behavior blocked on the text/i18n phase.
- HTTP client/server loopback tests require no external network.
- Async HTTP operations are real suspension points.
- Cookies and robotparser are pure or panic-free parser code with CPython-derived tests.

### milestone_network_web_4: Integration, Documentation, And Production Gate

Scope:

- Update public docs for every new module and major intentional divergence:
  - `socket`, `select`, `selectors`, `socketserver`
  - `ssl`
  - `urllib.parse`, `urllib.request`, `urllib.response`, `urllib.error`, `urllib.robotparser`
  - `http`, `http.client`, `http.server`, `http.cookies`, `http.cookiejar`
- Update internal architecture docs for:
  - runtime networking/TLS/HTTP boundaries
  - stdlib feature/dependency manifest
  - async counterpart policy
  - host-limited platform behavior
- Add demos:
  - TCP echo server/client
  - TLS client/server loopback
  - HTTP client/server loopback
- Add generated Cargo dependency snapshots for all new feature combinations.
- Add panic-scan and emitted-code quality checks for network/TLS/HTTP paths.
- Update validation lane manifests with representative fixtures.
- Close the inventory:
  - every public surface has a terminal state
  - every CPython test family has `adopted`, `adapted`, or `waived` evidence
  - every waiver has a revisit rule and regression fixture
  - every host-limited surface records the supported host matrix
- Run an external review loop on the final inventory and close any blocking finding before phase completion.
- External review owner is the stdlib phase owner plus the designated compiler/runtime reviewer recorded in the execution ledger. If review output is unavailable for five working days after the review artifact is posted, the phase owner may proceed only by recording the attempted review, open questions, and a conservative self-review in the ledger.

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

- Every module surface and CPython test family in the phase inventory is closed as `done`, `intentional-diff`, `unsupported`, or `host-limited`.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in the added stdlib/runtime surfaces.
- Async and sync APIs follow the Phase 32 workload and cancellation model.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-stdlib-platform-parity-execution.md`
- `verification/stdlib/network_web_parity_inventory.md`
- `verification/stdlib/network_web_parity_inventory.json`
- `verification/stdlib/network_web_parity_cpython_test_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test files scanned
- adopted/adapted/waived CPython test families
- final unsupported/intentional-diff/host-limited waiver index

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No backward-compatibility shims, legacy aliases, deprecated behavior, or fallback paths may survive phase exit. Deliberate current-CPython adapters are allowed only when recorded in the inventory with Sifr-safe semantics and tests.
- No direct Tokio/runtime types may leak into public Sifr APIs.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added blocking sync function must be classified in the stdlib workload database.
- Every added async function must have a real suspension summary.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Every module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.

## Open Planning Questions To Resolve In `milestone_network_web_0`

1. Which `sifr.*` module paths exactly host the CPython-shaped API surfaces, and which private `_sifr.*`/runtime modules remain internal implementation details?
2. Which Rust TLS root strategy is acceptable for deterministic local tests and production binaries?
3. Which HTTP client/server dependency stack meets binary-size, safety, and maintenance goals?
4. Which host-specific socket/select constants are shipped, waived, or host-limited?
5. Which external-network CPython tests are converted to loopback fixtures versus retained as non-blocking ecosystem signal?

These questions must be answered in the phase execution ledger before implementing the affected milestone.
