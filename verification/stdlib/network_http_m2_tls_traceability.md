# Network HTTP M2 Traceability: TLS Runtime

Status: merged in PR #2496 at `742ea9f33dcac821d5abb644156d97dd2d7876cc`.

M2 implements the Sifr-native TLS substrate over the M1 TCP runtime. It keeps Rustls and Tokio-Rustls private, exposes `sifr.tls` as the public module, and leaves CPython-shaped `SSLContext` / `SSLSocket` APIs rejected by the M0 namespace and unsupported-import fixtures.

| Work item | M0 decision | Implementation evidence |
| --- | --- | --- |
| `sifr.tls` public module and config types | `production-public`, stable public Sifr API; Rustls types hidden. | `lib/sifr/tls.sifr`; private `_sifr.tls` intrinsics in `crates/sifr_stdlib/src/tls.rs`; runtime module gate in `crates/sifr_runtime/src/lib.rs`; codegen lowerers in `crates/sifr_codegen/src/intrinsics/registry/tls.rs` and `crates/sifr_codegen/src/preamble/tls_runtime.rs`. |
| Safe client verification defaults | Platform verification is the default production client strategy; no fallback root store. | `client_config_platform(...)` uses `rustls-platform-verifier`; dependency snapshot asserts `rustls-platform-verifier = { version = "0.7.0", default-features = false }`; no `webpki-roots` is emitted. |
| Deterministic test roots | Explicit in-memory roots for tests; `rcgen` is dev/test only. | `client_config_with_roots(...)` and mTLS config helpers parse PEM through `rustls-pemfile`; runtime tests generate deterministic CA/server/client material with dev-only `rcgen`; generated dependency snapshot asserts no `rcgen`, `webpki-roots`, or `x509-parser` in production deps. |
| TLS client/server streams | Async-native over M1 TCP and `tokio-rustls`; TCP handles are consumed into TLS handles. | `crates/sifr_runtime/src/tls.rs`; `crates/sifr_runtime/src/net.rs` exposes internal `consume_stream_for_tls(...)`; public fixture `crates/sifr/tests/e2e/pass/network_http_m2_tls_loopback_split.sifr` performs real loopback client/server handshakes. |
| SNI and ALPN | accepted substrate. | Public `connect_tls(..., server_name, ...)` passes SNI; config constructors accept ALPN byte protocol lists; runtime and e2e loopbacks assert selected ALPN. |
| mTLS | accepted M2 substrate with success and rejection evidence. | Runtime test `tls_loopback_split_close_notify_and_alpn` covers client-auth success; runtime test `mtls_rejects_missing_client_certificate` covers missing-client-cert rejection. |
| TLS full-duplex split | Owned affine read/write halves; no borrowed split views or recombine. | `TlsStream.split()` in `lib/sifr/tls.sifr`; runtime split handles in `crates/sifr_runtime/src/tls.rs`; e2e fixture `network_http_m2_tls_loopback_split.sifr` splits the client stream, exchanges bytes, and preserves read-half ownership after write-side `close_notify()`. |
| `flush`, `close_notify`, `close` | `close_notify()` is the TLS write-side close operation; write-after-close-notify must be typed and deterministic. | Runtime close-notify state in `crates/sifr_runtime/src/tls.rs`; e2e fixture validates `flush`, `close_notify`, protocol version recording, and write-after-close-notify as `TlsError`; runtime tests cover EOF and close-notify handling. |
| Typed TLS and certificate errors | Public Sifr errors wrap TLS/certificate/config failures; lower-layer network evidence is preserved in TLS error text. | `TlsError` and `CertificateError` in `lib/sifr/tls.sifr`; codegen maps runtime strings into typed `TlsError`; public config-error fixture maps malformed PEM to `CertificateError`; runtime invalid-root test covers certificate verification failure. |
| Build and host evidence | TLS deps are feature-gated; non-TLS generated programs do not build crypto providers; host platform verification behavior is matrixed. | `crates/sifr_stdlib/src/features.rs`; `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs`; `verification/stdlib/network_http_dependency_audit.md`; `verification/areas/runtime_platform/supported_host_matrix.md`. |

## Validation Evidence

Focused validation completed for the M2 candidate:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo check -p sifr_runtime --features tls` | PASS | Verifies optional runtime TLS feature compilation with Rustls, Tokio-Rustls, PEM parsing, and platform verifier dependencies. |
| `cargo check -p sifr_stdlib -p sifr_codegen` | PASS | Verifies stdlib source embedding, intrinsic signatures, generated dependency features, and TLS lowerer/preamble compilation. |
| `cargo build -p sifr --bin sifr` | PASS | Rebuilt the CLI after public `sifr.tls` wrapper edits so generated fixture runs used the current embedded stdlib. |
| `cargo test -p sifr_runtime --features tls --lib tls -- --nocapture` | PASS | Covers TLS loopback split/ALPN/close-notify, mTLS missing-client rejection, and invalid-root certificate rejection. |
| `cargo test -p sifr_stdlib --test network_http_dependency_snapshots -- --nocapture` | PASS | Covers M0/M1/M2 generated dependency snapshots, TLS feature gating, and Ring 5 absence from production deps. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m2_tls_loopback_split.sifr` | PASS | Public TLS loopback over TCP with ALPN, protocol version evidence, split halves, close-notify, and write-after-close-notify typed error. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m2_tls_config_errors.sifr` | PASS | Public malformed-PEM config path maps to `CertificateError`. |
| `SIFR_E2E_FIXTURE_MANIFEST=/tmp/sifr_m2_tls_fixtures_$$.json cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` | PASS | Selected `network_http_m2_tls_config_errors` and `network_http_m2_tls_loopback_split`; 2 pass tests completed. |
| `cargo fmt --check` | PASS | Formatting clean after M2 edits. |
| `CARGO_TARGET_DIR=target/codex-clippy cargo clippy --workspace -- -D warnings` | PASS | Workspace clippy baseline passed; isolated target directory avoided stale default-target Cargo locks. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | 2309 files checked; touched hand-maintained files remain below the 900-line guardrail. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | Lowering maintainability guardrails remain clean after TLS intrinsic/codegen additions. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Clean PTY run passed after clearing stale interrupted validation jobs; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. |
| `scripts/run_all_tests.sh` | PASS | Full merge-gate validation passed for PR #2496 head `d4e2feb1feef13c7fd037d14301531915ed75b2a`; report `target/validation_lane_reports/merge.latest.json`; advisory only: high e2e group skew. |
| Claude Opus final branch-tip review pass 4 | PASS | `reviews/ad-hoc-production-network-http-m2-opus-review-pass-4.md` found no blockers and accepted PR #2496 for merge. |

Exploratory full-pass note: an accidental full `cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` run completed the M2 TLS fixture groups successfully but failed in unrelated pre-existing IO and `bytes_conversion_errors` pass fixtures. The targeted M2 manifest above is the authoritative M2 e2e signal for this candidate.

Full merge-gate validation initially exposed stale validation-contract assumptions around e2e Tokio feature expectations and helper binary lookup under `CARGO_TARGET_DIR`. The final branch-tip follow-up corrected those validation contracts and the merge gate passed afterward.

## CPython Evidence

M2 mines behavior classes from `Lib/test/test_ssl.py`, `Lib/test/test_asyncio/test_ssl.py`, and `Lib/test/test_asyncio/test_sslproto.py`: handshake success/failure, verification failure, client certificate authentication, ALPN, orderly TLS shutdown, and async stream integration. OpenSSL object-model APIs, `SSLContext`, `SSLSocket`, readiness retry exceptions, and descriptor-shaped compatibility behavior remain rejected as public Sifr surfaces.
