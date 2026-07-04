I have enough context for the review. Let me write up the findings.

## VERDICT: PASS (with non-blocking suggestions)

The M10 wave 3 URL stdlib interop migration is correctly executed. URL parse/build/percent/path/query helpers move cleanly from compiler-intrinsic ownership into private `@rust(sifr_stdlib.url.*)` declarations, the public `sifr.url` API surface is preserved, ParseError → UrlError translation is centralized in the public wrapper, and the bridge contracts (flat `list[str]` payloads, `int | None`/`str | None` conversions through `SifrIntBridge`/`Option<String>`) hold together with the new direct-interop codegen rules. The Rust leaf at `crates/sifr_stdlib/src/url.rs:1` keeps all the safety guardrails the old intrinsic preamble had — URL/query byte caps, scheme/host/percent validation, IPv4/IPv6 handling, and `set_fragment(None)` to block fragment injection — and the changes contain no new `unwrap`/`expect`/`panic` paths on user-triggerable input.

## Findings (by severity)

### Blocking
- **None.**

### Non-blocking (low severity / hygiene)

1. **Dead URL runtime preamble still ships in `crates/sifr_codegen`.** `crates/sifr_codegen/src/preamble/url_http_runtime.rs:5-313` still defines `URL_RUNTIME` (`__sifr_url_*` helpers), and `crates/sifr_codegen/src/lib_modules_and_codegen.rs:448-454` still gates it on `__sifr_url_*` references / `url_*` intrinsics. After this migration nothing in the generated stdlib preamble or the intrinsic registry can match those triggers, so the URL portion is effectively unreachable. Recommend a follow-up to delete `URL_RUNTIME`/`build_url_runtime_items` plus the `needs_url_runtime` branch (the HTTP runtime sharing this file is still live). Not blocking because dead code is harmless and `cargo clippy --workspace -- -D warnings` reportedly passes.

2. **`registry/url_http.rs` filename is now misleading.** `crates/sifr_codegen/src/intrinsics/registry/url_http.rs:1` still says “URL and HTTP primitive intrinsic lowerers,” but it is HTTP-only now. Rename to `http.rs` (or add `mod http;` alias) when the HTTP surface migrates. Non-blocking cosmetic.

3. **Robustness margin in `stdlib/sifr/url.sifr` bridge unflattening.** `_pairs_from_flat` (`stdlib/sifr/url.sifr:153-161`) advances `i = i + 2` without first checking `len(flat) % 2 == 0`, and `_optional_port` (`stdlib/sifr/url.sifr:115-122`) silently returns `None` on any non-int string. Both rely on `crates/sifr_stdlib/src/url.rs:267-289` always producing the exact 12-element layout and `Vec<String>` query pairs in `chunks_exact(2)`. The Rust side currently honors that contract, so this never fires in practice. Defensive `UrlError("URL bridge payload has invalid pair count")` and a length-odd check are nice-to-have, parallel to the existing `len(parts) != 12` guard in `_url_from_parts`.

4. **`is_optional_str` / `is_optional_int` only match exact 2-member unions.** `crates/sifr_codegen/src/rust_interop_direct.rs:57-81` quietly falls through to `else { value }` for any other optional (e.g., `bytes | None`, `float | None`, `list[T] | None`). Today only URL needs `str | None`/`int | None`, so this is fine, but the silent fallthrough will produce a hard-to-diagnose Rust type error the first time someone declares a new optional. Worth a note in `internal_docs/sifr_sysroot_and_stdlib_architecture.md:392-...` so the next migration wave knows to extend this enum-style match.

5. **Architecture doc claims `_sifr.url` no longer carries `url`/`percent-encoding` direct deps** (`internal_docs/sifr_sysroot_and_stdlib_architecture.md` and `stdlib_native_surface_ownership.toml`). The `features_tests.rs` matrix only asserts this for the public `sifr.url`. For belt-and-suspenders, add `"_sifr.url"` to the `features_tests` matrix at `crates/sifr_stdlib_model/src/features_tests.rs:237` so a future regression that re-attaches the direct `url` dep to the private leaf alone is caught. Non-blocking.

## Required Fixes vs Non-Blocking
- **Required:** none.
- **Recommended (non-blocking, can land in a follow-up):** items 1–5 above, in any order.

## Additional Validation Suggested Before PR

The focused tests listed are good signal but do not equal the project's pre-PR gate. Before opening the PR:

1. Run the workflow-mandated authoritative gate: `scripts/run_all_tests.sh --profile create-pr` (the AGENTS.md/CLAUDE.md gate — it transitively runs clippy, fmt, file-size, HIR guardrail, unit, codegen, and e2e suites including `test_e2e_pass`).
2. Run `cargo clippy --workspace --all-targets --locked -- -D warnings` once with the `url` feature exercised (the new branches in `rust_interop_direct.rs` and `sifr_stdlib::url` have not been linted in the validation summary).
3. Run the full `verification/runner/e2e/run_e2e_pass.sh` (not just the single fixture). The migration changes generated Rust for any module transitively pulling `_sifr.url`, and the lone fixture cited (`network_http_url_query_percent.sifr`) does not exercise modules like `sifr.http_transport`/`sifr.http` whose generated code interacts with the URL preamble emission gate at `lib_modules_and_codegen.rs:448-454`.
4. Once (1)–(3) pass, run `cargo test -p sifr_codegen --locked` and `cargo test -p sifr_driver --locked` without the `CARGO_BUILD_JOBS=1` constraint to surface any flakiness around the new direct-interop tests under normal concurrency.

If those are green, the change is ready to ship.
