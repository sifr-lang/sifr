Execution error full diff, verified the current unknown-import rejection path, checked all remaining `http_transport` / `http1_*` / `http2_*` references, and confirmed the runtime fixture setup. Findings below.

## Findings

### Non-blocking observations (minor cleanup, no functional impact)

**N1 — Dead codegen dispatch prefix branch for `http1_*/http2_*`.**
`crates/sifr_codegen/src/intrinsics/registry.rs:409` still matches `name if name.starts_with("http1_") || name.starts_with("http2_")` and forwards to `url_http::lower_http_intrinsic(...)`. That lowerer no longer has any match arms for these prefixes (all deleted at `url_http.rs:79-101` of the pre-change diff), so the branch returns `None`, propagates up through `lower_intrinsic_rendered`, and the caller falls back to a normal Rust call that would fail to compile. It's genuinely dead — retained intrinsic signatures, requirements arms, and preamble wrappers are all gone, so nothing can produce these names. Kept only because `internal_docs/stdlib_retained_compiler_intrinsics.toml:152` still lists `["http_", "http1_", "http2_"]`, and the allowlist guard compares observed prefix regex hits against that list. Removing both simultaneously would fully retire the codegen dispatch fallback per the stated M0d scope; leaving them costs nothing functionally.

**N2 — Retained surface id name is stale.**
`internal_docs/stdlib_retained_compiler_intrinsics.toml:138` id `_sifr.http::transport_and_header_helpers` and reason "HTTP transport, protocol, and handle behavior remains behind runtime/resource certification" now describe only header/cookie/method/status/version validators; transport helpers are gone. Doc-only rename opportunity.

**N3 — Test name mildly misleading.**
`crates/sifr/tests/e2e_support/network_http_dependency_rules_tests.rs:63` `test_infer_dependencies_recognizes_http_transport_runtime_references` now exercises raw hyper/h2/http-body/http-body-util/hyper-util/tower-service inference. The crate references are still valid (runtime crate still uses them for the fixture path), but the "http_transport" in the test name reads as if `sifr.http_transport` is exercised.

## Verification of the removal

Traced end-to-end and confirmed the transport source path is genuinely severed:

- **No stdlib source imports `sifr.http_transport`** or any transport symbol (grep-clean outside archived plans and the intentional negative fail fixture).
- **Bootstrap seeder deleted**: `HTTP_TRANSPORT_HARNESS_ALIASES`, `seed_http_transport_harness_aliases`, `HTTP_TRANSPORT_HARNESS_RUST` all gone from `crates/sifr_driver/src/stdlib/bootstrap.rs`.
- **Lowering escape hatch deleted**: `LoweringOptions.allow_http_transport_harness_imports` is gone. The `sifr.http_transport` special case is also gone. The current fixture reaches the ordinary unknown-import diagnostic (`SIFR-IMPORT-0002`) through `imports::report_unknown_stdlib_module`.
- **Driver frontend API surface deleted**: `compile_with_metadata_allowing_http_transport_harness` removed from `frontend/api.rs`, `frontend/mod.rs`, `lib.rs`; also from `build/mod.rs` re-exports.
- **E2E directive scan deleted**: `HTTP_TRANSPORT_HARNESS_DIRECTIVE` and the scan in `fixture_compilation.rs` are gone; no `.sifr` file contains the directive.
- **Retained transport signatures deleted**: `crates/sifr_retained_intrinsics/src/http.rs` no longer registers `http[12]_client_roundtrip_{tcp,tls}` or `http[12]_server_respond_{tcp,tls}` in `_sifr.http`.
- **Preamble wrappers deleted**: `__sifr_http1_*` / `__sifr_http2_*` async wrappers gone from `preamble/url_http_runtime.rs`; the runtime-detection guard in `lib_modules_and_codegen.rs:457` correctly narrowed to `__sifr_http_` and `http_` prefix only.
- **Feature planning deleted**: `sifr.http_transport` removed from `features_for_stdlib_module`, `features_for_module`, and `needs_sifr_runtime_http`; the module-based runtime-detection now depends only on required features. Cargo TOML generation and dependency snapshot test also purged. Manifest test `network_http_transport_intrinsics_emit_locked_hyper_runtime_dependencies` deleted.
- **Runtime fixture parity**: `verification/areas/runtime_platform/fixtures/http_transport_loopback.rs` wired as `[[test]] name = "http_transport_loopback_fixture"` in `crates/sifr_runtime/Cargo.toml:72-75` with `required-features = ["http"]` (which transitively enables tls and net). Uses `sifr_runtime::{http, net, tls}` directly, exercises HTTP/1.1 TCP, HTTP/2 h2c (absolute URI), HTTPS/H2 with `h2` ALPN — parity with the deleted three synthetic Sifr fixtures.
- **Cargo metadata classification updated**: new test target added at `cargo_metadata_classification.json:114`.
- **Docs and traceability**: `docs/network_http.md`, `internal_docs/network_http_architecture.md`, `network_http_handoff_traceability.md`, and `network_http_http_transport_traceability.md` now describe the runtime-fixture ownership and explicitly state `sifr.http_transport` is not embedded, driver-seeded, or dependency-planned.
- **Validation gate ran**: `scripts/run_all_tests.sh --profile create-pr` passed in 257.83s, advisories=none — the pass-1 blocker H1 is resolved.

No hidden fallback remaining that could silently reactivate transport code from a `_sifr.http` source: the retained signatures, the preamble wrappers, the codegen match arms, and the transport dependency planning would all have to be re-added together. The residual dispatch prefix branch (N1) doesn't compile any transport code on its own.

**READY**
