## Verdict: PASS — M4 is acceptable to open as the PR after `scripts/run_all_tests.sh --profile create-pr` passes locally.

### B4-RACE remediation — closed

- `SIFR_E2E_ALLOW_HTTP_TRANSPORT_HARNESS` appears nowhere in active code, docs, or fixtures. Remaining hits are in archived review markdown (`reviews/ad-hoc-production-network-http-m4-agent-review-pass-2.md`, `…pass-3.md`) which describe the prior, removed implementation.
- The new gate is a stack-local typed option, not process state:
  - `crates/sifr_lowering/src/lower/mod_context.rs:320-323` defines `LoweringOptions { allow_http_transport_harness_imports }`; default = `false` at `:167`.
  - `LowerCtx::with_options` (`:198-202`) installs it per `LowerCtx::new()`.
  - `crates/sifr_lowering/src/lower/mod_impl.rs:307-323` rejects `sifr.http_transport` unless `ctx.allow_http_transport_harness_imports`; emits `SIFR-IMPORT-0009` via the existing `unsupported_legacy_stdlib_module` path (`crates/sifr_stdlib/src/lib.rs:264-268`).
  - Public driver exposes `compile_with_metadata_allowing_http_transport_harness` (`crates/sifr_driver/src/frontend/api.rs:62-78`); all default `compile`/`build`/`check`/project/package entrypoints pass `LoweringOptions::default()` (e.g., `crates/sifr_driver/src/build/api.rs:45`, `build/entrypoint.rs:87/95/119/191`).
- E2e harness is per-fixture-directive: `crates/sifr/tests/e2e_support/fixture_compilation.rs:5-14` scans each source for `# sifr-e2e-allow-http-transport-harness` and routes only those compiles through the harness-allowing API. `compile_source` in `harness_model.rs:532-543` (used by `test_e2e_fail` / decimal retired-code test) goes through the default API and rejects `sifr.http_transport` automatically. No env var, no lock, no shared state — concurrent compiles cannot race.

### Phase-contract and fixture coverage

- Public protocol primitives live in `lib/sifr/http.sifr`; internal harness in `lib/sifr/http_transport.sifr` is registered as a legacy/internal module suggesting `sifr.http` (`crates/sifr_stdlib/src/lib.rs:220, 264-268`).
- New fail fixture `crates/sifr/tests/e2e/fail/network_http_sifr_http_transport_internal.sifr` has no directive and asserts `SIFR-IMPORT-0009` at col=1 — exercises the ordinary-user rejection path.
- Three new pass fixtures each open with the directive on line 1: `network_http_m4_http1_loopback.sifr` (HTTP/1 TCP + independent body-limit exercise + too-large body rejection), `network_http_m4_http2_loopback.sifr` (h2c with absolute URI), `network_http_m4_https_h2_loopback.sifr` (HTTPS + ALPN `h2`).

### Independent body limits + typed errors + HTTP/2 conformance

- `crates/sifr_runtime/src/http.rs:348-606` exposes `http{1,2}_{request,respond}_{tcp,tls}` taking `max_request_bytes` and `max_response_bytes` as separate parameters; outbound side `checked_body(_, max_*)`, inbound side `collect_limited(_, max_*)`. Codegen preamble shims (`crates/sifr_codegen/src/preamble/url_http_runtime.rs:492-678`) pass both through.
- Runtime tests at `crates/sifr_runtime/src/http.rs`: `http1_malformed_response_maps_to_typed_error:614`, `client_request_and_response_limits_are_independent:645`, `http2_settings_hpack_and_goaway_loopback:682`, `http2_rst_stream_maps_cancel_reason:727`.
- Intrinsic mapping at `crates/sifr_codegen/src/intrinsics/registry.rs:770-773` correctly routes `http1_*` / `http2_*` to the Hyper feature requirement (`requirements.rs:9-25` `HTTP_TRANSPORT_REQUIRED_FEATURES`).

### Traceability / serving-scale handoff / dependency snapshot

- `verification/stdlib/network_http_m4_http_transport_traceability.md:13` links `issues/ad-hoc-network-http-serving-scale-follow-up.md` with the stable identifier `ad-hoc-network-http-serving-scale-follow-up` (file present, line 4).
- `verification/stdlib/hyper_util_necessity.md` documents that Hyper-Util is enabled with only the `tokio` feature (TokioIo + TokioExecutor), no client pooling/auto-server/graceful-shutdown.
- `verification/stdlib/network_http_dependency_snapshots.json` adds the `http-transport` snapshot (id `milestone_network_http_4`) and the assertion test `network_http_m4_transport_intrinsics_emit_locked_hyper_runtime_dependencies` at `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs:241-295` pins the exact emitted dep set.
- Guardrails: file-size script `PASS` (largest new/touched: `sifr_runtime/src/http.rs` at 779, `intrinsics/registry.rs` at 885, both under 900).

### Non-blocking observations (informational)

- `crates/sifr_codegen/src/intrinsics/registry.rs` is at 885/900 lines; the next intrinsic addition will trip the guardrail. Worth a responsibility-based decomposition in M5 but not an M4 blocker.
- The `sifr.http_transport` rejection at `mod_impl.rs:310` is a hard-coded literal match (rather than a generic `is_test_only_module(...)` predicate). That's deliberate — the harness gate is intentionally one-off — and the M5 handoff in the traceability doc already calls out converting this to a structurally private/test-only source.

The empty `reviews/ad-hoc-production-network-http-m4-agent-review-pass-4.md` placeholder is the slot for this review; happy to write it into that file as the pass-4 record if you want it persisted alongside passes 1–3.
