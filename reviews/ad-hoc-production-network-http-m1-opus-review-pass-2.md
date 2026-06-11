I have enough. Here's my review.

## PASS — safe to open the M1 PR

Pass-1 blockers B1–B3 are remediated; the M0/M1 contract is satisfied at the Sifr surface, intrinsic registry, codegen lowerer, generated preamble, and Rust runtime. The non-blocking items are either resolved in code or explicitly documented in the M1 traceability with acceptable follow-up disposition. One residual validation note is recorded below — it does not block opening the PR but should close before the merge step.

### Blocker verification

**B1 — Global `+ Send` validation on Awaitable/Coroutine. RESOLVED for PR open.**
`crates/sifr_type_system/src/types/type_queries.rs:413-419` and `:439-442` still emit `Send`-bounded `Pin<Box<dyn Future<…> + Send>>` for every `Coroutine`/`Awaitable`, i.e., the change remains workspace-wide rather than net-scoped. The remediation closed the *validation* gap rather than the *scope*: `scripts/run_all_tests.sh --profile create-pr` (the AGENTS.md PR-opening lane) is recorded `PASS` in `verification/stdlib/network_http_m1_async_network_traceability.md:37` and `issues/ad-hoc-production-network-http-platform-substrate-execution.md:270`. The create-pr fixture manifest (`verification/validation_lanes/create_pr_e2e_manifest.json`) explicitly exercises the previously-uncovered async surfaces — `async_with_basic`, `async_with_nested_cleanup_order`, `async_with_return_cleanup`, `async_for_*`, `async_generator_*`, `process_async_*`, `cheap_sync_helper_in_async_allowed`, plus the three new M1 net fixtures — so the cross-cutting `+ Send` bound is now empirically validated against the async corpus the create-pr lane is meant to cover. The broad exploratory `cargo test … e2e_pass -- network_http_m1` run that exposed `cpython_io_subset` / `stdlib_io_consolidated` / `open_*` / `bytes_conversion_errors` failures is classified as pre-existing by the traceability; the working tree confirms it (none of those fixtures use `async`/`await`, the diff vs `main` touches only network/type-system/feature-registry code, and the I/O subsystem they depend on is untouched), so the failures are not regressions introduced by `+ Send`.

Residual note (not blocking PR open): the original pass-1 fix list also asked for `scripts/run_all_tests.sh` (the merge gate). Only `--profile create-pr` is recorded. AGENTS.md treats `--profile create-pr` as the PR-opening lane and the bare `run_all_tests.sh` as the merge gate; the merge-gate run should be recorded in `validation_evidence` before flipping the M1 checklist to merged, but is not required to open the PR.

**B2 — `TcpStream.split()` is infallible end-to-end. RESOLVED.**
The contract from `issues/ad-hoc-production-network-http-platform-substrate.md:480` is now satisfied at every layer with no `Result` and no panic:

- Sifr surface: `lib/sifr/net.sifr:61-63` declares `def split(own self) -> tuple[TcpReadHalf, TcpWriteHalf]` and the `_closed` raise is gone — affinity is enforced by `own self` alone.
- Intrinsic type registry: `crates/sifr_stdlib/src/net.rs:163-169` types `net_tcp_stream_split` as `Type::Tuple(vec![tcp_read_half_class(), tcp_write_half_class()])`, no `net_error_result` wrapper.
- Codegen lowerer: `crates/sifr_codegen/src/intrinsics/registry/net.rs:159-167` emits a plain `__sifr_net_tcp_stream_split(handle)` call — no `boxed_async_net_helper_call`, no `Result` wrapping.
- Generated preamble: `crates/sifr_codegen/src/preamble/net_runtime.rs:100-103` returns `(TcpReadHalf, TcpWriteHalf)` directly via `sifr_runtime::net::tcp_stream_split(handle)`.
- Rust runtime: `crates/sifr_runtime/src/net.rs:370-385` is now `pub fn tcp_stream_split(handle: i64) -> (i64, i64)`. Storage is the canonical owned `tokio::net::TcpStream` in `STREAMS` (no `Arc<Mutex<…>>` indirection on the unsplit path), so split removes the stream from the table and calls `stream.into_split()` directly — no `Arc::try_unwrap`, no panic, no `Err`. If the stream handle is unknown the runtime returns fresh handles that aren't backed by any half, which makes subsequent operations return a typed `NetError` — the affine `own self` boundary on the Sifr side keeps this branch unreachable in user code.

**B3 — Active cancellation coverage. RESOLVED.**
`crates/sifr/tests/e2e/pass/network_http_m1_tcp_cancel_accept.sifr` binds a listener on `127.0.0.1:0` (no client ever connects, so `accept()` is genuinely blocking), spawns `wait_for_accept(listener)` into `task.scope(...)`, calls `handle.cancel()`, awaits the handle, and asserts `"Cancelled" in str(result)`. The provider-task cancellation path is the `__SifrTask` impl at `crates/sifr_codegen/src/preamble/task_runtime.rs:232` — `abort_handle.abort()` followed by `__SifrTaskResult::cancelled()` — i.e., it consumes the existing provider cancellation primitive rather than introducing a network-local token, matching the M0 requirement. Tokio's `TcpListener::accept` is cancellation-safe, so the in-flight accept drops cleanly without panic. The fixture is recorded in the traceability row for Cancellation (`verification/stdlib/network_http_m1_async_network_traceability.md:12`) and in the validation grid (`:31`).

### Non-blocking items — all resolved or documented

- **N1 (write-after-shutdown stable typed error).** `crates/sifr_runtime/src/net.rs:286-291` still returns the single `TCP write side is already shut down` string. The M1 traceability (`network_http_m1_async_network_traceability.md:11`) explicitly designates this string as the stable M1 evidence and reserves richer error-taxonomy refinement to a later phase. Documented — acceptable.
- **N2 (Tokio process/signal features in the generated set).** `crates/sifr_stdlib/src/features.rs:579-586` still emits `process` + `signal` in the Tokio feature list, and the snapshot test (`crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs:81-88`) bakes that into the assertion. The M1 traceability (`:46`) records the rationale: those features are owned by the concurrency/runtime provider used by other generated surfaces and the snapshot's network-specific feature set (`net`, `io-util`, `macros`, `rt`, `sync`, `time`) remains intact. Documented — acceptable for M1.
- **N3 (Bytes/Socket2/TokioUtil unused features).** Fully removed from `crates/sifr_stdlib/src/features.rs` — `StdlibFeature` no longer declares those variants, and `features_for_stdlib_module("sifr.net")` now pulls only `SifrRuntime`, `Tokio`, `Tracing` (`features.rs:453-457`). The new snapshot test asserts the locked-down set without `bytes`/`socket2`/`tokio-util` emission.
- **N4 (`TCP_NODELAY` policy).** `crates/sifr_runtime/src/net.rs:175-176` still hardcodes `set_nodelay(true)` on outbound connect. The M1 traceability (`:44`) records this as the M1 default for interactive low-latency substrate and notes no public socket-option surface lands in M1. Documented — acceptable.
- **N5 (substrate inventory flip).** `verification/stdlib/network_http_substrate_inventory.json` now records `sifr.net` as `production-public` / `stable-public-api` with M1 ownership. Resolved.
- **N6 (hardcoded `MAX_READ_BYTES`).** `crates/sifr_runtime/src/net.rs:17` still hardcodes the 1 MiB ceiling. The M1 traceability (`:45`) flags this as the explicit M1 default with configuration deferred to M4/M5. Documented — acceptable.

### What is good

- The owned-stream model is honored at the runtime layer: `STREAMS` now stores bare `tokio::net::TcpStream` (not `Arc<Mutex<…>>`), so `into_split()` is a direct ownership transfer with no panic potential. Split halves stay `Arc<tokio::sync::Mutex<…>>` for legitimate concurrent-half use.
- `__sifr_net_error`-based mapping into typed `NetError` is consistent across all network helpers in `preamble/net_runtime.rs`; the runtime returns `Result<_, String>` and never panics on user data. Validation paths (timeout finiteness, read-size limits, backlog bounds) all return typed errors.
- Generated Cargo features for `sifr.net` (`tests/network_http_dependency_snapshots.rs`) lock down to the M1-approved set and explicitly assert no `bytes`/`socket2`/`tokio-util` emission, plus `sifr_runtime` with the `net` feature, Tokio with `net`, and Tracing.
- UDP remains properly deferred via the `SIFR-NAME-0004` fail fixture at `crates/sifr/tests/e2e/fail/network_http_m1_udp_deferred.sifr`.
- `shutdown_write()` on the unsplit `TcpStream` preserves the read side (`crates/sifr_runtime/src/net.rs:346-359`); the bitset is idempotent so repeated calls remain deterministic.
- The cancellation fixture consumes the existing provider task scope and `handle.cancel()` rather than introducing a parallel network-local cancellation primitive, matching the M0 contract.
- M1 traceability and execution-ledger updates cover every blocker remediation row and validation command actually run.

### Before merging (not blocking PR open)

- Record `scripts/run_all_tests.sh` (the full merge gate) in M1 validation evidence before flipping the milestone checklist; create-pr is the right lane for opening the PR but the merge gate is the recorded merge contract per AGENTS.md.
- If the broad-pass note's pre-existing failures (`cpython_io_subset`, `stdlib_io_consolidated`, `open_*`, `bytes_conversion_errors`) are unfamiliar to the merge reviewer, link the main-branch baseline failure history so the merge-gate run can be interpreted consistently.

These are recording requirements, not contract violations — opening the M1 PR is safe.
