## Round 2 review

Diffs, doc updates, emit output, and guard scripts all check out. Verified against a fresh emit of both HTTP and URL fixtures.

**No blocking findings. Satisfied for PR.**

What I verified:

- **No dead references.** `grep -rn "build_url_runtime_items\|build_http_runtime_items\|needs_url_runtime\|needs_http_runtime"` returns zero live-code hits. `__sifr_url_*` / `__sifr_http_*` / `__sifr_header_error` also zero. Only vestigial mentions are in archived review docs (`plans/reviews/archive/**`).
- **Emit output confirms the claim.** `sifr emit network_http_url_query_percent.sifr` now shows only `sifr_stdlib::url::url_parse_parts(value)` / `url_build_parts(...)` / `url_percent_*` / `url_normalize_path` / `url_query_*_flat` calls — no `__sifr_url_*` helpers. `sifr emit network_http_header_cookie.sifr` shows only `sifr_stdlib::http::http_validate_*` / `http_header_map_from_pairs` / `http_parse_cookie_header` / `http_build_cookie_header` — no `__sifr_http_*` helpers. So the removed detection in `lib_modules_and_codegen.rs` is genuinely unreachable.
- **Manifest is clean.** The retained manifest currently has only `_sifr.http::header_helpers` in state `closing` with an accurate `removal_criteria`; no `_sifr.url::url_helpers` row exists, no `registry_files = ["url_http.rs"]`, no `preamble_files = ["url_http_runtime.rs"]`. Guard scripts (`check_stdlib_native_intrinsic_allowlist.py`, `check_stdlib_migration_closure.py`, `check_hir_maintainability_guardrails.py`) all PASS locally.
- **Retained intrinsics fallback removed correctly.** `_sifr.http` is out of `get_intrinsic_module`, so the closing-row constraint ("closing rows must remove fallback signature modules before deletion") is satisfied — the guard would fail otherwise.
- **Docs align.** Architecture matrix row for HTTP/URL now points at `_sifr.url`→`sifr_stdlib::url` and `_sifr.http`→`sifr_stdlib::http`; traceability report replaces the old `preamble/url_http_runtime.rs` / `crates/sifr_retained_intrinsics/src/http.rs` evidence with the new sysroot-owned locations. No doc still names the deleted preamble.
- **Test harness is coherent.** `fixture_cargo_toml.rs` dropping `HTTP_DEP` for `sifr.http`/`_sifr.http` matches the emit reality (generated code references only `sifr_stdlib::http::*`, never bare `http::`). `harness_model.rs` sysroot detection for `sifr_stdlib::http::` correctly promotes `_sifr.http` and enables the `http` sifr_stdlib feature. Rules-tests assertions on the harness output are internally consistent.

Two non-blocking observations (mention for hygiene, do not block PR):

1. `crates/sifr/tests/e2e_support/network_http_dependency_rules_tests.rs:46` still names its function `test_infer_dependencies_recognizes_url_http_runtime_references` even though `url_http_runtime` no longer exists as a module. The body correctly asserts raw-crate inference for `url::`, `percent_encoding::`, and `http::`, so it is still meaningful — the name is just a vestige. Optional rename to `..._raw_crate_references` in a follow-up.
2. Production `retained_direct_dependencies` (via `HTTP_DEPS` in `sifr_stdlib_manifest/src/features.rs:167`) still emits a top-level `http = "1.4.1"` for `sifr.http`/`_sifr.http` modules, while generated user Rust no longer references the `http` crate directly. This is pre-existing, not introduced this round — but it means production Cargo.tomls carry a top-level `http` dep that is now only used transitively through `sifr_stdlib`. Worth tightening in a later pass, not a blocker for this migration.
