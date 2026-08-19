## Review findings — M0d HTTP harness → verification-owned fixture

### Correctness / scope

**H1 — Missing authoritative pre-PR validation gate.** AGENTS.md requires `scripts/run_all_tests.sh --profile create-pr` (or the full merge-gate variant) as the authoritative gate before PRs — CI mirrors it exactly. The listed validation runs only focused unit tests for three crates and skips workspace clippy/format guardrails on other crates that transitively depend on the changed types (`LoweringOptions`, driver frontend API, sifr_stdlib_manifest, sifr codegen). Run `scripts/run_all_tests.sh --profile create-pr` before opening the PR; it's cheap signal versus the CI cycle. Not a code defect, just a missing checkbox.

### Residual risk / follow-up items (non-blocking)

**M1 — Retained-but-orphaned `_sifr.http` transport intrinsics and codegen wrappers.** With `HTTP_TRANSPORT_HARNESS_RUST` and `seed_http_transport_harness_aliases` deleted from `bootstrap.rs`, and no stdlib `.sifr` source importing any `httpN_client_roundtrip_*` / `httpN_server_respond_*` from `_sifr.http`, the following are now dead code but still present:
- `crates/sifr_retained_intrinsics/src/http.rs:164-189` — 8 transport intrinsic entries in `_sifr.http`.
- `crates/sifr_codegen/src/intrinsics/registry/url_http.rs:132-155` — 8 dispatch arms.
- `crates/sifr_codegen/src/intrinsics/registry/requirements.rs:45-52` — 8 feature-requirement arms.
- `crates/sifr_codegen/src/preamble/url_http_runtime.rs:492-678` — 8 `__sifr_httpN_*` Rust wrapper functions (~185 lines).

M0d's goal calls for "no hidden fallback". These paths are not user-reachable (they require `from _sifr.http import …` inside a stdlib `.sifr` source, which no source does), but they are latent — a stdlib change could silently reactivate them without any test detecting it. Not required by M0d's stated scope, but flag as candidate for M0e cleanup or mark as retained-by-design.

**M2 — Sifr codegen ↔ runtime combined-flow parity gap.** The retired `.sifr` fixtures were the only tests exercising the codegen path `sifr.http_transport.transport_httpN_*(...)` → `__sifr_httpN_*_*` wrapper → `sifr_runtime::http::*`. With them deleted, no test covers those codegen wrappers or the intrinsic registry dispatch. The traceability doc claims equivalent parity via `demos/network_http_substrate/main.sifr` + `http_transport_loopback_fixture`, but:
- `demos/network_http_substrate/main.sifr` invokes no transport function (nor TCP/TLS I/O) — only substrate type construction.
- `verification/areas/runtime_platform/fixtures/http_transport_loopback.rs` uses raw `Vec<(String,String)>` headers and `sifr_runtime::http::*` directly, bypassing Sifr codegen entirely — no `HeaderMap`/`RequestHead`/`ResponseHead`/`BodyStream`.

Runtime-level tests (`http1_malformed_response_maps_to_typed_error`, `client_request_and_response_limits_are_independent`, `http2_settings_hpack_and_goaway_loopback`, `http2_rst_stream_maps_cancel_reason`) do cover the Rust runtime thoroughly, so no runtime-behavior regression. But the codegen glue itself is now uncovered. Defensible only if paired with M1's follow-up removal.

**M3 — Traceability wording implies stronger demo coverage than exists.** In `verification/areas/stdlib_parity/reports/network_http_http_transport_traceability.md` rows "Typed request/response model" and "Body streaming", the phrasing "public `sifr.http` source behavior remains covered by `demos/network_http_substrate/main.sifr`" is accurate for substrate type construction but not "around transport calls" — the demo makes no transport call. Consider clarifying that Sifr-typed heads/bodies are exercised in isolation from transport, and transport parity is verified at the runtime layer only.

### Minor cleanup

**L1 — `compile_single_file_entrypoint_with_metadata_and_options` is now single-caller.** `crates/sifr_driver/src/build/entrypoint.rs:153` is only invoked once from `compile_single_file_entrypoint_with_metadata` (line 150) with `LoweringOptions::default()`. The wrapper existed to flip the removed `allow_http_transport_harness_imports` field. `LoweringOptions` still carries `python_trust_policy`, but that field flows through other paths (`project/frontend.rs`, `build/python_runtime.rs`). Consider inlining the helper or dropping the `_and_options` variant — small dead-parameter cleanup, not required.

### Verdict

**Not ready for PR** solely because the create-pr validation gate hasn't been run (H1). Code-wise the removal is clean and internally consistent: the synthetic `sifr.http_transport` bootstrap seeder, the `HTTP_TRANSPORT_HARNESS_RUST` blob, the lowering escape hatch, the frontend override API, the e2e directive scan, the Cargo/dependency planning entries, and the three `.sifr` fixtures + create-PR manifest entries are all gone; the current ordinary unknown-import rejection routes through `report_unknown_stdlib_module`; and the new Rust fixture exercises HTTP/1 TCP, HTTP/2 h2c, HTTPS/H2+ALPN loopback with equivalent request/response assertions. Docs and traceability updated consistently. After running the create-pr profile, this can ship; carry the listed follow-ups with the architecture doc updates and source-origin privacy work.
