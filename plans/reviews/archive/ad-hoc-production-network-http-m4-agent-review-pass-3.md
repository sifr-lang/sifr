# Network HTTP M4 — agent Review Pass 3

Reviewer: agent
Scope: branch `codex/network-http-m4-client-server` after the pass-2 blocker remediation (B1–B5). Verifies that B1, B2, B3, and B5 are addressed without regression and that B4's harness-namespace gate behaves correctly under the e2e harness contract.

## Decision

**FAIL.**

B1, B2, B3, and B5 land cleanly. B4 introduces a process-wide env-var (`SIFR_E2E_ALLOW_HTTP_TRANSPORT_HARNESS`) that gates the lowering check, and the new e2e fail fixture `crates/sifr/tests/e2e/fail/network_http_sifr_http_transport_internal.sifr` is asserted ONLY against the unset state. The pass-suite directive compile path sets that env var process-wide while non-directive compile paths (`test_e2e_fail`, `test_decimal_fail_fixtures_do_not_emit_retired_pseudo_codes`) read it without coordination, which produces a real cross-test race when cargo test runs `test_e2e_pass` and `test_e2e_fail` in parallel (the default).

## Blocking findings

### B4-RACE — env-var harness gate is racy across concurrent tests

- **Where**: `crates/sifr/tests/e2e_support/fixture_compilation.rs:7-26` sets `SIFR_E2E_ALLOW_HTTP_TRANSPORT_HARNESS=1` process-wide for the duration of a directive-marked compile under `HTTP_TRANSPORT_HARNESS_COMPILE_LOCK`. `crates/sifr_lowering/src/lower/mod_impl.rs:310-323` reads `std::env::var_os("SIFR_E2E_ALLOW_HTTP_TRANSPORT_HARNESS")` unconditionally on every `sifr.http_transport` import lowering, with no coordination with that lock. `crates/sifr/tests/e2e_support/harness_model.rs:532-543` (`compile_source`, used by `test_e2e_fail` at `crates/sifr/tests/e2e_support/e2e_entrypoints.rs:311` and by `test_decimal_fail_fixtures_do_not_emit_retired_pseudo_codes` at `:354`) does not take the lock or wrap the env at all.
- **Why it bites**: cargo test runs `#[test]` fns in parallel by default. While a `test_e2e_pass` worker thread compiles one of the three M4 directive-marked fixtures (`network_http_m4_http1_loopback.sifr`, `network_http_m4_http2_loopback.sifr`, `network_http_m4_https_h2_loopback.sifr`), the env var is set to `"1"`. If `test_e2e_fail`'s sequential loop is compiling `network_http_sifr_http_transport_internal.sifr` at that moment, `mod_impl.rs:311` observes `var_os(...).is_some()`, the legacy-module branch does not fire, `STDLIB_SOURCES` lists `sifr.http_transport` (`crates/sifr_stdlib/src/sources.rs:106-108`) so externals resolve the symbols, the import succeeds, and `test_e2e_fail` panics with `"FAIL test ... should have failed but compiled successfully"`. The window is small (a handful of M4 compiles × tens-of-milliseconds each) but real; the reported validations explicitly did not run pass+fail together (the SIFR_E2E_FIXTURE_MANIFEST run was M4-fail-only), so the race is unobserved, not absent.
- **Remediation (any one of these)**:
  1. Replace the env-var with a compile-context flag carried through `sifr_driver::CompileOptions` (or a dedicated `LowerOptions { allow_http_transport_harness: bool }`) and set it from the harness-level directive parse — no process-global state. Recommended.
  2. Take the same `HTTP_TRANSPORT_HARNESS_COMPILE_LOCK` from `compile_source` in `harness_model.rs:532`, and route every test entrypoint that compiles sifr through it, so directive-set and non-directive reads serialize. Cheap but couples the lock into `sifr_driver` callers everywhere.
  3. Drop `sifr.http_transport` from `STDLIB_SOURCES` and load it conditionally only when the env var is set, so non-directive compiles cannot resolve the symbols even if they observe a leaked env var. Aligns with the M5 N1 handoff direction but is a larger move.

This is the single blocker.

## Verification of pass-2 blockers

### B1 — HTTP/2 conformance coverage (PASS)
- `crates/sifr_runtime/src/http.rs:683-725` (`http2_settings_hpack_and_goaway_loopback`) drives a real `h2::client::handshake` + `h2::server::handshake` over a Tokio duplex, exercises SETTINGS negotiation (implicit in the handshake), HPACK roundtrip via a non-static `x-sifr-hpack` header, and asserts that `graceful_shutdown` closes without producing a stream-error reason on the client driver. Reasonable conformance shape.
- `crates/sifr_runtime/src/http.rs:727-778` (`http2_rst_stream_maps_cancel_reason`) sends a response and then `send_reset(Reason::CANCEL)`, accepting either response-body-arrives-then-fails or response-future-fails (both are valid h2 outcomes depending on timing). Both branches assert `Reason::CANCEL` — correct mapping.
- Traceability row updated at `verification/stdlib/network_http_m4_http_transport_traceability.md:8`. Acceptable.

### B2 — Malformed HTTP typed-error coverage (PASS)
- `crates/sifr_runtime/src/http.rs:614-643` (`http1_malformed_response_maps_to_typed_error`) writes `HTTP/1.1 200 OK\r\nContent-Length: nope\r\n\r\nbad` and asserts the client returns `Err` containing `"HTTP/1.1 request failed"` or `"failed to read HTTP body frame"`. Hyper rejects the malformed Content-Length and the test allows either of the two valid error-string shapes — correct.
- Traceability row added at `:12`. Acceptable.

### B3 — Independent request/response body limits (PASS)
- API surface: `crates/sifr_runtime/src/http.rs:348-606` — every public entrypoint takes `max_request_bytes` and `max_response_bytes` as distinct parameters; `checked_body(body, max_request_bytes)` validates outgoing body separately from `collect_limited(body, max_response_bytes)` which validates the incoming peer body.
- Intrinsic registry: `crates/sifr_stdlib/src/http.rs:163-205` — both transport intrinsic families (client roundtrip, server respond) take both limits as ordered params with the correct arity.
- Lowerers: `crates/sifr_codegen/src/intrinsics/registry/url_http.rs:136-169` — `lower_http_client_roundtrip` expects `args.len() == 7` (handle, method, path, headers, body, max_request_bytes, max_response_bytes) and `lower_http_server_respond` expects 6 (handle, status, headers, body, max_request_bytes, max_response_bytes). Matches the intrinsic shapes.
- Generated preamble: `crates/sifr_codegen/src/preamble/url_http_runtime.rs:492-678` — all eight `__sifr_http{1,2}_{client_roundtrip,server_respond}_{tcp,tls}` shims forward `max_request_bytes, max_response_bytes` to the runtime independently.
- Sifr wrappers: `lib/sifr/http_transport.sifr:23-104` — all eight `transport_*` wrappers expose both parameters.
- Regression coverage:
  - Unit-level: `crates/sifr_runtime/src/http.rs:645-680` (`client_request_and_response_limits_are_independent`) — server runs with `max_request_bytes=1024` and successfully receives `b"request-body"`; client runs with `max_response_bytes=4` against an 18-byte response body and errors with `"HTTP body exceeds configured limit"`. Asserts the server still observed the full request body. Directly proves the bug.
  - Fixture-level: `crates/sifr/tests/e2e/pass/network_http_m4_http1_loopback.sifr:76-97` — independent-limit case calls the client with `max_request_bytes=1024, max_response_bytes=4` and the server with `1024, 1024`; asserts the client errors with `"exceeds"` in the message.

Acceptable.

### B4 — Harness public-namespace gate (PARTIAL; see B4-RACE blocker)
- Lowering check is present at `crates/sifr_lowering/src/lower/mod_impl.rs:310-323`. Classification is wired in `crates/sifr_stdlib/src/lib.rs:220` (in `unsupported_legacy_stdlib_module`'s canonical-name match) and `:264-268` (in `legacy_stdlib_module_info`'s diagnostic info). Fail fixture at `crates/sifr/tests/e2e/fail/network_http_sifr_http_transport_internal.sifr:1-6` asserts `SIFR-IMPORT-0009` at col=1. The directive-marked harness at `crates/sifr/tests/e2e_support/fixture_compilation.rs:7-26` is correctly scoped to lines that contain `# sifr-e2e-allow-http-transport-harness`.
- However: the env-var mechanism is process-global, and the fail fixture's expectation can flake when run concurrently with pass-suite directive compiles. See B4-RACE above. The lowering check is functionally correct in isolation; the contract just cannot be reliably tested under cargo test's default parallelism with the current mechanism.

### B5 — Serving-scale follow-up link (PASS)
- `verification/stdlib/network_http_m4_http_transport_traceability.md:13` — the server-substrate row now records: "Serving-scale remains owned by `issues/ad-hoc-network-http-serving-scale-follow-up.md` stable identifier `ad-hoc-network-http-serving-scale-follow-up`." Matches the contract requirement in `issues/ad-hoc-production-network-http-platform-substrate.md:338`. Acceptable.

## Verification of other called-out changes
- Cookie-header helpers: `crates/sifr_stdlib/src/features.rs:145-147` keeps `COOKIE_DEPS = &[]`; no external `cookie` crate is emitted. Phase contract row in `issues/ad-hoc-production-network-http-platform-substrate.md:378` still calls for Sifr-owned parse/build. Consistent.
- `verification/stdlib/hyper_util_necessity.md` — accurately records why a local `hyper::rt::Read`/`Write` + executor adapter was not written (would duplicate `hyper_util::rt::TokioIo`/`TokioExecutor`) and confirms only the `tokio` Hyper-Util feature is enabled.
- `crates/sifr_codegen/src/lib_modules_and_codegen.rs:449-458` — `needs_http_runtime` now triggers on `__sifr_http_`/`__sifr_http1_`/`__sifr_http2_` preamble matches and intrinsic-name prefixes `http_`/`http1_`/`http2_`. Combined with `crates/sifr_codegen/src/intrinsics/registry.rs:770-773` routing `http1_`/`http2_` to `lower_http_intrinsic`, the H2-only fixture path no longer relies on the accidental `__sifr_http_future` local-name match.

## Non-blocking follow-ups
- **N1 (carried)** — `sifr.http_transport` public-namespace tension: even after B4-RACE is resolved structurally, the M5 task to convert this from "env-var-gated public source" to a private alias or test-only source remains, per the traceability handoff at `verification/stdlib/network_http_m4_http_transport_traceability.md:13`.
- **N7 (carried, informational)** — `version_label` `HTTP/unknown` arm at `crates/sifr_runtime/src/http.rs:128` is still unreachable from valid `http::Version` values. Defensive default; cleanup later.
- **Cookie-feature variant cleanup (carried)** — `StdlibFeature::Cookie` with `COOKIE_DEPS = &[]` is cosmetic but unused; remove or document.
- **MAX_HTTP_TIMEOUT_SECONDS (carried)** — `crates/sifr_runtime/src/http.rs:16` remains a private constant; if M5 surfaces a stable Sifr-facing timeout, promote into the typed-error contract.
- **Test isolation, defensively** — even if B4-RACE is fixed by routing the harness through a compile-context flag, consider running `test_e2e_fail` ahead of `test_e2e_pass` (or pinning `--test-threads` lower) for the pre-PR profile so other future cross-test process state is less likely to cause similar flakes.

## Validation gaps before PR/merge

The reported runs cover the focused M4 surface, the new conformance tests, and the new fail fixture in isolation. The following pre-PR runs are still required per `AGENTS.md` and were not reported:

- `scripts/run_all_tests.sh --profile create-pr` — the authoritative pre-PR gate. Without it, the B4-RACE flake is unobserved rather than absent.
- `cargo clippy --workspace -- -D warnings` — the new tests heavily use `.unwrap()`/`.expect("...")` which are fine under `cfg(test)` but pedantic-tier lints in `crates/sifr_runtime/src/http.rs` (e.g., the new `match (request, &response_result)`, the `Box::pin` shim signatures, the new `format!` paths) should clear `-D warnings` first.
- `cargo test -p sifr --test e2e test_e2e_pass` without `SIFR_E2E_FIXTURE_MANIFEST` — the full corpus needs to confirm that the explicit `http1_`/`http2_` runtime preamble detection and the new `MAX_HTTP_TIMEOUT_SECONDS` cap do not change emission for M0–M3 fixtures, and that the new `legacy_modules` entry does not break any fixture that may currently import `sifr.http_transport` legitimately.
- `cargo test -p sifr --test e2e test_e2e_pass test_e2e_fail` together (i.e., the same `cargo test` invocation, default parallelism) — this is the run that exercises the B4-RACE window. If B4-RACE is fixed mechanistically, run this several times locally to demonstrate stability.

## Answers to the asked questions
1. **Verdict**: **FAIL.**
2. **Blocking findings**: one — B4-RACE (env-var harness gate is racy across concurrent `test_e2e_pass` ↔ `test_e2e_fail` execution). Exact remediation: replace the process-wide env var with a compile-context flag threaded through `sifr_driver::CompileOptions`/`LowerOptions`, set by the harness when the `# sifr-e2e-allow-http-transport-harness` directive is present. Alternative remediations listed under the blocker.
3. **Non-blocking follow-ups**: N1 namespace tension carried to M5; N7 `HTTP/unknown` dead arm; `StdlibFeature::Cookie` empty-deps cleanup; `MAX_HTTP_TIMEOUT_SECONDS` promotion when M5 surfaces a public timeout; defensive test isolation.
4. **Acceptable to open M4 PR after `scripts/run_all_tests.sh --profile create-pr` passes?** No, not yet — B4-RACE must be fixed first because the script's default-parallelism run is exactly the surface that triggers the race. Once B4 is repaired structurally and the full pre-PR gate (plus `cargo clippy --workspace -- -D warnings` and an un-manifested `test_e2e_pass`) passes locally, the branch is acceptable to open.
