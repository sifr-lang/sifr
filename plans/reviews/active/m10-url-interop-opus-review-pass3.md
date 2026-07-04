I've completed a thorough review of the M10 wave 3 URL migration. Here's my assessment.

## Verdict: **PASS** — no blockers found

The migration is coherent, well-tested, and follows the established regex-migration precedent. Correctness holds across the compiler, stdlib leaf, sifr glue, feature model, and harness layers.

### What I verified

**Bridge round-trip integrity (the highest-risk area) is correct.** The 12-field string-list payload from `crates/sifr_stdlib/src/url.rs:271-294` (`url_parts_from_parsed`) exactly matches the field ordering consumed by `stdlib/sifr/url.sifr:128-149` (`_url_from_parts`): scheme(0), username(1), password+marker(2,3), host(4), port(5), path(6), query+marker(7,8), fragment+marker(9,10), serialized(11). The `bool_marker`/`_optional_part` "1"/"0" convention is consistent, and empty-but-present values (e.g. `user:@host`) preserve the `Some("")` vs `None` distinction correctly.

**Error bridging follows the base64 precedent.** `@rust` decls in `stdlib/_sifr/url.sifr` return `Result[_, ParseError]`, and `rust_interop_direct.rs:129-151` (`bridge_error_expr`) maps Rust `Err(String)` → `ParseError { message }`. The sifr wrappers catch `ParseError` and re-raise `UrlError(e.message)`. `ParseError` is a confirmed builtin (`builtin_errors.rs:5`), and the identical pattern is already proven for base64 (line 447).

**Optional-arg codegen is validated end-to-end.** The new `is_optional_str`/`is_optional_int` helpers (`.clone()` for `str|None`, `.map(SifrIntBridge::from)` for `int|None`) are exercised by `network_http_url_query_percent.sifr`, which calls `build_url(..., port=8080)` etc. and passed the create-pr e2e suite — so `port.map(...)` genuinely rustc-compiles and runs, not just codegens.

**Feature derivation has no gap.** `features_for_stdlib_module` returning `&[]` for `sifr.url` is correct: the `url` feature is now derived from the `@rust(sifr_stdlib.url.*)` interop targets. Both the production path (`features_tests.rs:246-247`, asserting exactly `sifr_stdlib{features=["url"]}` with no direct `url`/`percent-encoding`) and the grouped-crate test-harness path (`fixture_cargo_toml.rs` + `fixture_dependency_paths.rs` + `harness_behavior_tests.rs`) are covered consistently.

**Platform-evidence fix is correct.** `check_platform_evidence.py` threads the manifest `timeout_seconds` through `run_and_check_duration` → `check_install_distribution_smoke` (replacing the hardcoded `timeout=60`), and the manifest bump to 300 aligns the subprocess timeout with the duration assertion. No other builtins are affected (they take `_timeout_seconds` and ignore it).

**`reject_bad_percent` boundary is correct** (`idx + 2 < bytes.len()` correctly requires two trailing hex digits — no off-by-one), and safe list indexing (`str | None`) means odd-length payloads degrade to `""` rather than panicking, upholding the no-panic guarantee.

### Non-blocking notes

1. **Validation closure.** No single uninterrupted `run_all_tests.sh --profile create-pr` completed *after* the harness fix — the original failure (`network_http_url_query_percent` missing `sifr_stdlib`) was root-caused and fixed, and the tail was validated piecewise. This is acceptable since the fix directly addresses the observed failure and every segment passed independently, but I'd recommend one clean uninterrupted run before merge for a clean audit trail.

2. **Retained `intrinsic_url()` model** (`sifr_stdlib_model/src/url.rs`) is now effectively dead for lowering (`intrinsic_names["_sifr.url"]` is empty per `stateless_private_codegen_tests.rs`), but its signatures were correctly updated to `ParseError`/`string_list` so it stays consistent. This mirrors the retained `intrinsic_regex()` — fine as a bootstrap fallback.

3. **`build_url_runtime_items` / `url_http_runtime.rs` is *not* dead** — it remains reachable through the still-compiler-owned HTTP surface (`needs_url_runtime` can be triggered by http preamble content), matching the ownership-doc note that HTTP helpers stay compiler-owned until the HTTP surface migrates. No action needed.

The implementation is acceptable for the milestone.
