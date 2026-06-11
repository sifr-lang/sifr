# Network HTTP M2 Traceability: TLS Runtime

Status: backlog from M0.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| `sifr.tls` public module and config types | `production-public`, stable-public-api; Rustls types hidden. | Canonical import/type-check/e2e fixtures and unsupported `ssl` diagnostics. |
| Safe client verification defaults | Verification on by default through `rustls-platform-verifier`; no fallback root store. | Invalid certificate typed errors and no silent downgrade tests. |
| Deterministic test roots | `rcgen` is dev/test only; production snapshots exclude it. | CA-backed loopback and generated dependency snapshot evidence. |
| TLS client/server streams | Async-native over M1 TCP, `tokio-rustls`, and Rustls `aws_lc_rs`. | HTTPS-ready loopback fixtures with nested `NetError` evidence. |
| SNI and ALPN | accepted substrate. | ALPN HTTP/1.1 and HTTP/2 selection fixtures. |
| mTLS | accepted substrate. | Client-cert success and rejection fixtures with typed `CertificateError`. |
| TLS full-duplex split | Owned affine read/write halves; no borrowed split views or recombine. | Concurrent read/write fixtures and sendability diagnostics. |
| `flush`, `close_notify`, `close` | `close_notify()` is write-side TLS close. `TlsStream.close()` and `TlsWriteHalf.close()` consume their handle, attempt `close_notify()` first, flush accepted plaintext/alert before success, then release TCP; cancellation and failure preserve progress plus nested `NetError` evidence. | TLS 1.2/1.3 version-recorded fixtures for close behavior. |
| Build and host evidence | M2 records `aws-lc-rs` tooling, binary size, cross-compilation, platform verifier behavior. | Supported-host matrix updates and generated dependency snapshots. |

## CPython Evidence

Mine `test_ssl`, `test_asyncio/test_ssl.py`, and `test_asyncio/test_sslproto.py` for behavior classes only. OpenSSL object model and retry APIs stay rejected.
