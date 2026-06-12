# Network HTTP M4 Traceability: HTTP Core Transport

Status: implemented in M4 candidate branch.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| HTTP/1.1 transport | Hyper-backed, Sifr-owned runtime wrapper, no client/server product API. | `network_http_m4_http1_loopback.sifr` passes check/run and selected e2e manifest. |
| HTTP/2 transport | Hyper/H2-backed with Sifr resource/error wrappers. Default fixture body buffering is bounded to 1 MiB unless overridden. | `network_http_m4_http2_loopback.sifr` passes selected e2e using absolute h2c URI with scheme/authority. Runtime test `http2_settings_hpack_and_goaway_loopback` covers SETTINGS negotiation, HPACK header roundtrip, and GOAWAY graceful shutdown; runtime test `http2_rst_stream_maps_cancel_reason` covers RST_STREAM cancellation mapping. |
| Typed request/response model | `sifr.http` substrate types from M0/M3; internal transport harness remains tuple-shaped and test-only. | M4 fixtures construct and validate `Method`, `Status`, `Version`, `RequestHead`, `ResponseHead`, `BodyStream`, and bounded body collection around transport calls. |
| Body streaming | Chunk type is built-in `bytes`; `collect_with_limit(max_bytes)` is accepted and unbounded collect is rejected. | `network_http_m4_http1_loopback.sifr` covers chunked body construction and too-large body rejection; all M4 fixtures use explicit request/response body bounds. Runtime test `client_request_and_response_limits_are_independent` verifies request and response body limits are not conflated. |
| HTTPS transport | consumes M2 TLS with ALPN. | `network_http_m4_https_h2_loopback.sifr` passes check/run with `h2` ALPN and long-lived localhost fixture certificate. |
| Malformed protocol errors | malformed protocol input maps to typed `HttpError`/protocol evidence and must not panic. | Runtime test `http1_malformed_response_maps_to_typed_error` feeds an invalid HTTP/1 response and asserts a typed HTTP error string. |
| Server accept/dispatch/shutdown substrate | single-runtime-worker per process; Phase 41 owns framework routing and lifecycle product. | Internal one-shot loopback harness validates client/server connection ownership handoff without adding public server API. `sifr.http_transport` is not an embedded stdlib source; the driver seeds it only as test harness metadata/Rust wrappers, and ordinary user imports are rejected with `SIFR-IMPORT-0009`. E2e fixtures must opt into the test-only harness with `# sifr-e2e-allow-http-transport-harness`. Serving-scale remains owned by `issues/ad-hoc-network-http-serving-scale-follow-up.md` stable identifier `ad-hoc-network-http-serving-scale-follow-up`. |
| HTTP/1 keep-alive | Deferred from the one-shot M4 loopback harness. | M4 forces `Connection: close` for HTTP/1.1 responses to keep single-connection substrate fixtures deterministic; M5 must record whether keep-alive remains deferred or add explicit keep-alive coverage before handoff. |
| Hyper-Util | conditional internal-only. | `verification/stdlib/hyper_util_necessity.md` records Hyper-only adapter burden, selected `tokio` feature, and no public Sifr contract dependency. |

## CPython Evidence

Mine `test_httplib`, `test_httpservers`, `test_socketserver`, and localnet `urllib` tests only for protocol substrate behavior. External network tests stay `external-signal`.
