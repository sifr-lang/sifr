# Network HTTP M1 Traceability: Async Network Runtime

Status: implemented in current M1 branch; Opus implementation review PASS; PR #2495 open with local merge-gate validation PASS, pending merge.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| `sifr.net` public module and intrinsic boundary | `production-public`, stable-public-api; no Tokio or descriptor leak. Public read chunks use built-in `bytes` with helpers under `sifr.bytes`. | `lib/sifr/net.sifr`, `_sifr.net` intrinsic registry, generated runtime helpers, and `network_http_m1_tcp_loopback_split.sifr` / `network_http_m1_tcp_errors.sifr`. |
| `SO_REUSEPORT` | Deferred from public API entirely until the serving-scale follow-up closes; `reuse_addr` never implies reuse-port. | `listen_tcp(..., reuse_addr=True)` maps only to Tokio `TcpSocket::set_reuseaddr`; no public reuse-port option or constructor exists. |
| TCP connect/listen/accept/read/write/close | Async-native over current-thread Tokio with provider cancellation/deadlines. | Loopback fixture covers bind-to-port-zero listen, local/remote address inspection, client connect, listener accept, read, write-all, close, EOF-style response, and deterministic generated-build dependencies. |
| TCP owned split halves | `split()` consumes a live stream and returns affine owned halves. | Loopback fixture splits both client and server streams sequentially after listener backlog accepts the pending connection. Runtime `tcp_stream_split` returns owned read/write handles directly; public `TcpStream.close(own self)` consumes the unsplit stream so closed streams cannot later be split through the Sifr API. |
| TCP write-side half-close | `shutdown_write()` sends FIN, preserves read side, write-after-shutdown is typed. | Loopback fixture calls `TcpWriteHalf.shutdown_write()`, reads the response on the read half, and verifies late write returns a typed `NetError`. M1's stable write-after-shutdown evidence string is `TCP write side is already shut down`; later error-taxonomy work may refine this into a richer variant without changing the typed `NetError` boundary. |
| Cancellation | Network operations consume provider task cancellation; no local cancellation token exists. | `network_http_m1_tcp_cancel_accept.sifr` spawns an in-flight listener `accept()`, cancels the task handle, awaits the handle, and asserts provider `Cancelled` evidence. |
| DNS/address resolution | `tokio::net::lookup_host`; custom resolver and Happy Eyeballs deferred unless M0 is amended. | `resolve_host("localhost:80", timeout=2.0)` smoke coverage plus deterministic invalid-timeout typed error coverage in `network_http_m1_tcp_errors.sifr`. |
| UDP | `deferred-to-phase-X` until a named production consumer plus fixture-insufficiency rationale is recorded. | No public `UdpSocket`; `network_http_m1_udp_deferred.sifr` expects `SIFR-NAME-0004` for `from sifr.net import UdpSocket`. |
| Readiness primitives | internal-only. | M1 adds no public selector, readiness, raw descriptor, or event-loop API. Existing M0 unsupported import diagnostics remain the public behavior. |
| Blocking sync helpers | sync `@blocking_io` if accepted. | M1 adds no sync network helpers; all accepted TCP and DNS operations are async-native. |

## M1 Validation

| Command | Result | Notes |
| --- | --- | --- |
| `cargo build -p sifr --bin sifr` | PASS | Rebuilt CLI after runtime, stdlib, type-system, and codegen changes. |
| `cargo check -p sifr_runtime --features net` | PASS | Validates optional `sifr_runtime/net` feature and Tokio network dependency wiring. |
| `cargo check -p sifr_stdlib -p sifr_codegen` | PASS | Validates stdlib feature mapping and network intrinsic/codegen changes. |
| `cargo test -p sifr_runtime --features net --lib net -- --nocapture` | PASS | Runtime crate builds and test harness completes; behavior is covered by generated Sifr fixtures. |
| `cargo test -p sifr_stdlib --test concurrency_runtime_dependency_snapshots -- --nocapture` | PASS | Verifies existing concurrency/Tokio dependency snapshot remains unchanged without `sifr.net`. |
| `cargo test -p sifr_stdlib --test network_http_dependency_snapshots -- --nocapture` | PASS | Verifies M0 Ring 5 absence and M1 locked generated dependency set for `sifr.net`: `sifr_runtime/net`, Tokio `net`, tracing, and no unused `bytes`/`socket2`/`tokio-util` emission. |
| `cargo test -p sifr_stdlib --test text_i18n_dependency_snapshots -- --nocapture` | PASS | Verifies moved text/i18n dependency snapshot coverage remains intact after feature-registry edits. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m1_tcp_loopback_split.sifr` | PASS | Deterministic TCP loopback, split halves, half-close, address inspection, typed timeout error. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m1_tcp_errors.sifr` | PASS | Invalid timeout/backlog typed `NetError` coverage. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m1_tcp_cancel_accept.sifr` | PASS | Provider task-handle cancellation over in-flight `accept()`. |
| `SIFR_E2E_FIXTURE_MANIFEST=<tmp manifest> SIFR_E2E_DISABLE_CACHE=1 cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` | PASS | Selected fixtures `network_http_m1_tcp_cancel_accept`, `network_http_m1_tcp_errors`, and `network_http_m1_tcp_loopback_split`; 3 pass tests completed. |
| `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` | PASS | Full fail harness completed 480 fail fixtures and validated UDP remains absent; existing negative-harness CFG panic messages are caught by the harness. |
| `cargo fmt --check` | PASS | Clean after M1 edits. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | 2302 files, limit 900 lines. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | Lowering maintainability guardrails passed. |
| `scripts/run_e2e_pass.sh` | PASS | Merge-manifest e2e pass suite completed 138 pass tests, 0 failed. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. |
| `scripts/run_all_tests.sh` | PASS | Report `target/validation_lane_reports/merge.latest.json`; all merge-lane steps passed. Advisories: warm wall-time budget exceeded and high group skew. |

Broad pass-suite note: an exploratory full `cargo test -p sifr --test e2e e2e_pass -- network_http_m1 --nocapture` invocation runs the full pass corpus rather than selecting only M1 fixtures. It exposed pre-existing non-network failures in IO context-manager mutability and a bytes codec expectation; the M1-specific pass fixtures above passed through the selected manifest.

Implementation notes:

- `TcpStream::connect` sets `TCP_NODELAY` on accepted outbound streams as the M1 default for interactive low-latency TCP substrate. No public socket option surface is added in M1.
- `MAX_READ_BYTES = 1_048_576` is the explicit M1 per-call read ceiling. M4/M5 resource-configuration work may layer user-configurable limits above this default.
- Generated `tokio` workspace features still include provider-owned `process` and `signal` features used by other runtime surfaces; `sifr.net` generated dependency snapshots remain locked to the M0-approved network feature set.

## CPython Evidence

M1 mined CPython socket and asyncio stream/server behavior only for deterministic loopback shape, address inspection, EOF/half-close behavior, and rejection of raw selector/event-loop public APIs. No CPython-shaped `socket`, selector, descriptor, or event-loop surface was added.
