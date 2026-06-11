# Network HTTP M4 Traceability: HTTP Core Transport

Status: backlog from M0.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| HTTP/1.1 transport | Hyper-backed, Sifr-owned wrappers, no client/server product API. | Loopback request/response, chunked transfer, keep-alive, and malformed protocol fixtures. |
| HTTP/2 transport | Hyper/H2-backed with Sifr resource/error wrappers. Default max concurrent streams is 100; body buffering is 1 MiB per stream; PING flood, RST_STREAM, GOAWAY, HPACK, malformed-frame, priority, and extension-frame mappings follow the inventory tables. | SETTINGS, flow control, PING, RST_STREAM, GOAWAY, HPACK, malformed frame, priority/extension decision fixtures. |
| Typed request/response model | `sifr.http` substrate types from M0/M3; no duplicate header/body representations. | Compile/type/e2e fixtures for `Method`, `Status`, `Version`, heads, errors, and limits. |
| Body streaming | Chunk type is built-in `bytes`; EOF is `None`; trailers are accepted as explicit `Trailers(HeaderMap)`; default chunk limit is 64 KiB; `collect_with_limit(max_bytes)` is accepted and unbounded collect is rejected; cancellation/reset carries byte progress. | Streaming upload/download, collect-with-limit, HTTP/2 reset, and no-unbounded-buffer fixtures. |
| HTTPS transport | consumes M2 TLS with ALPN. | HTTP/1.1 and HTTP/2 over TLS loopback fixtures. |
| Server accept/dispatch/shutdown substrate | single-runtime-worker per process; Phase 41 owns framework routing and lifecycle product. | Internal loopback harness and handoff docs. |
| Hyper-Util | conditional internal-only. | If enabled, `hyper_util_necessity.md` records Hyper-only attempt, avoided adapter code, selected features, and no public contract dependency. |

## CPython Evidence

Mine `test_httplib`, `test_httpservers`, `test_socketserver`, and localnet `urllib` tests only for protocol substrate behavior. External network tests stay `external-signal`.
