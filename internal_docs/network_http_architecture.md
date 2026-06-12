# Network And HTTP Architecture

The production network/HTTP substrate is split by ownership boundary:

- `sifr.net` provides async TCP listeners, streams, owned split halves, write-side half-close, DNS lookup, typed `NetError`, and no public raw descriptor or selector API.
- `sifr.tls` wraps the TCP substrate with Rustls-backed client/server configs, ALPN, SNI, mTLS fixture coverage, `close_notify`, owned split halves, and typed `TlsError`.
- `sifr.url` owns URL, query, percent, authority, redaction, and IDNA guard behavior. Unicode/IDNA behavior remains blocked until the text/i18n provider accepts the Unicode version and normalization policy.
- `sifr.http` owns protocol values and body streams. M4 transport is implemented in the runtime through Hyper/H2 and validated by the e2e-only `sifr.http_transport` harness.

## Runtime Boundary

The runtime backing is internal:

| Layer | Accepted backing | Public hiding rule |
| --- | --- | --- |
| Async I/O | Tokio current-thread runtime and I/O traits | no Tokio handle or task type in public APIs |
| TCP/DNS | `tokio::net` | no raw socket descriptors or resolver product API |
| TLS | `rustls`, `tokio-rustls`, platform verifier | no Rustls config/error types leak |
| URL | `url`, `percent-encoding` | wrapped in Sifr URL values and `UrlError` |
| HTTP | `http`, `http-body`, `http-body-util`, `hyper`, `h2`, `hyper-util`, `tower-service` | no Hyper/H2/Tower types leak |
| Observability | `tracing` events/spans | no subscriber/exporter setup |

Generated Cargo dependencies are feature-gated in `sifr_stdlib` and snapshot-tested in `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs`.

## Provider Consumption

Network, TLS, URL, and HTTP do not create duplicate provider models:

- Cancellation and deadlines consume the task provider semantics.
- Stream backpressure consumes the synchronization/backpressure provider semantics.
- Blocking helper policy consumes the workload/offload provider semantics.
- Shutdown consumes the signal/shutdown provider semantics.
- Diagnostics and request context consume the runtime diagnostics provider semantics.
- Process-worker serving scale remains deferred to `issues/ad-hoc-network-http-serving-scale-follow-up.md`.
- The server transport handoff is single-runtime-worker per Sifr process until that serving-scale follow-up closes; multi-core throughput, `SO_REUSEPORT`, process-worker supervision, and future multi-thread runtime topology are not hidden inside this substrate.

## Handoff

Phase 41 may build routing, middleware, lifecycle, request/response pipelines, typed extractors, validation, and production hooks on the protocol/runtime substrate. It must not claim multi-core serving throughput until the serving-scale follow-up closes.

The future production HTTP client phase may build pooling, redirects, retries, auth, cookies, proxies, streaming upload/download, and policy helpers on the substrate. It owns those product policies; this substrate owns protocol correctness, typed errors, resource limits, and deterministic loopback validation.
