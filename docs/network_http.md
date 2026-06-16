# Network, TLS, URL, And HTTP Substrate

Sifr networking is exposed through Sifr-native modules under `sifr.*`:

- `sifr.net` is the low-level async TCP and DNS substrate.
- `sifr.tls` adds Rustls-backed TLS configs and streams over `sifr.net`.
- `sifr.url` owns typed URL, query, and percent primitives.
- `sifr.http` owns HTTP protocol primitives: methods, statuses, versions, headers, request/response heads, body streams, and typed HTTP errors.

These modules are production substrate for Phase 41 web-framework work and the later production HTTP client phase. They are not CPython compatibility modules.

## Public Boundary

Use Sifr-native imports:

```python
from sifr.net import connect_tcp, listen_tcp
from sifr.tls import client_config_from_ca_pem
from sifr.url import parse_url
from sifr.http import request_head, headers_from_pairs
```

CPython-shaped networking and web modules are rejected or diagnosed:

```python
import socket
import ssl
from urllib.parse import urlparse
from http.server import HTTPServer
```

The replacement boundary is explicit:

| CPython-shaped surface | Sifr boundary |
| --- | --- |
| `socket`, `sifr.socket` | `sifr.net` TCP streams, listeners, and DNS helpers |
| `ssl`, `sifr.ssl` | `sifr.tls` configs and streams |
| `urllib.parse`, `sifr.urllib.*` | `sifr.url` |
| `http.client`, `http.server`, `socketserver` | `sifr.http` substrate now; product APIs in later phases |

## Provider Dependencies

Network operations consume the concurrency/runtime provider for task cancellation, deadlines, backpressure, blocking-work diagnostics, shutdown, and runtime diagnostics. The network substrate does not define its own cancellation token, shutdown coordinator, executor, queue, process worker, or diagnostics bus.

The current server transport handoff is production-correct for one runtime worker per Sifr process. Multi-core serving throughput, `SO_REUSEPORT`, process-worker supervision, and any future multi-thread runtime topology are deferred to `plans/issues/archive/ad-hoc-network-http-serving-scale-follow-up.md`; Phase 41 must not claim those scale properties until that follow-up closes.

Text-heavy behavior remains provider-gated:

| Surface | State |
| --- | --- |
| TCP/TLS/HTTP protocol bytes | production substrate |
| HTTP text body helpers | blocked on text/i18n encoding work |
| non-ASCII header/cookie user text | blocked on text/i18n encoding work |
| non-UTF-8 URL encodings | blocked on text/i18n encoding work |
| Unicode/IDNA host canonicalization | blocked on text/i18n Unicode alignment |
| locale-sensitive network diagnostics | blocked on text/i18n locale formatting |

## HTTP Transport Boundary

The current phase validates HTTP/1.1 and HTTP/2 transport through an internal e2e harness. `sifr.http_transport` is test-only and ordinary user imports fail with `SIFR-IMPORT-0009`. Public server routing, middleware, request extractors, JSON/form helpers, cookies, redirects, retries, auth, proxies, compression, WebSockets, CONNECT, and HTTP/3 are intentionally outside this substrate.

Representative runnable examples:

- `demos/network_tcp_echo/main.sifr`
- `demos/network_tls_loopback/main.sifr`
- `demos/network_http_substrate/main.sifr`

Deterministic HTTP transport loopback coverage lives in e2e fixtures because it uses the private harness: `network_http_http1_loopback`, `network_http_http2_loopback`, and `network_http_https_h2_loopback`.
