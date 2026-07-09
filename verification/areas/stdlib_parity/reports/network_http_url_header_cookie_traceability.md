# Network HTTP URL/header/cookie capability Traceability: URL, Header, And Cookie Primitives

Status: merged in PR #2497 at `9a3ee4d18a12ab6ddaa9174aebea591a891c4651`.

| Requirement | network/HTTP baseline capability decision | Acceptance evidence |
| --- | --- | --- |
| `sifr.url.Url` and `UrlQuery` | `production-public`, stable-public-api over ASCII and already-valid Sifr text. | `stdlib/sifr/url.sifr`; `_sifr.url` private Rust interop declarations in `stdlib/_sifr/url.sifr`; `sifr_stdlib::url` owns parser/percent/query helper behavior; fixture `network_http_url_query_percent.sifr`. |
| Percent helpers | Byte/ASCII/UTF-8 safe helpers accepted; named encodings blocked on text/i18n async-network capability. | `percent_encode`, `percent_decode`, `percent_encode_bytes`, and `percent_decode_bytes`; fixture covers UTF-8 text, raw bytes, and invalid percent escapes. |
| Host and authority validation | ASCII domain labels, IPv4, IPv6, and already-punycode accepted; non-ASCII blocked until text/i18n TLS capability; ports are decimal 0-65535. | Generated helper rejects non-ASCII authority hosts before `url` crate IDNA behavior can leak; fixture covers IPv4 parsing, IPv6 building, already-punycode host, and invalid port rejection. |
| Path normalization | Parsing applies WHATWG dot-segment removal for special schemes. Percent-encoded slash (`%2F`) is preserved as a segment byte, not a separator. | `normalize_path` helper and fixture coverage for parser dot-segment removal, explicit helper dot-segment removal, and encoded slash boundaries. |
| Query parsing/building | byte/ASCII/UTF-8 safe behavior only; sensitive query key redaction applies in observability. | `UrlQuery` stores ordered pairs; parse/build fixture covers duplicate keys, plus decoding, and empty/missing lookup behavior. |
| `sifr.http` header primitives | URL/header/cookie capability owns canonical `HeaderName`, `HeaderValue`, and `HeaderMap`: ASCII token names, lowercase canonical names, obs-fold rejection, OWS trim only, duplicate order preservation, and size caps from inventory. | `stdlib/sifr/http.sifr`; `_sifr.http` private Rust interop declarations in `stdlib/_sifr/http.sifr`; `sifr_stdlib::http` uses the `http` crate for validation behind the native boundary; fixture covers lowercase canonicalization, duplicate order, OWS trim, invalid names, obs-fold rejection, and inventory hard caps. |
| Cookie header parsing | header-level parse/build only. | Cookie header parse/build uses ordered `list[tuple[str, str]]`; fixture covers parse/build, embedded `=` in values, and invalid cookie syntax. Cookie jars, signing, private cookies, persistence, and percent-decoded user text remain unexposed. |

## Validation Evidence

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo check -p sifr_stdlib -p sifr_codegen` | PASS | Verifies intrinsic signatures, generated dependency features, and URL/HTTP lowerer compilation. |
| `cargo build -p sifr --bin sifr` | PASS | Rebuilt the CLI after adding `sifr.url`, `sifr.http`, and generated helper preambles. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_url_query_percent.sifr` | PASS | Public URL/query/percent fixture type-checks. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_header_cookie.sifr` | PASS | Public header/cookie fixture type-checks. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_url_query_percent.sifr` | PASS | Fresh generated build after reviewer remediation; covers URL parse/build, IPv4 parsing, IPv6 building, punycode host, percent-encoded ASCII host acceptance, literal and percent-encoded non-ASCII host blocked states, `%2F` path preservation, percent helpers, path normalization, query parse/build, and invalid port. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_header_cookie.sifr` | PASS | Fresh generated build after reviewer remediation; covers header name/value validation, lowercase canonicalization, OWS trim, duplicate order, obs-fold rejection, cookie header parse/build, and embedded `=` cookie values. |
| `cargo test -p sifr network_http_dependency_rules_tests -- --nocapture` | PASS | Verifies locked URL dependency specs, HTTP stdlib feature inference, Sifr-owned cookie-header handling without an external cookie crate, and generated-Rust dependency inference for URL/runtime crates and `sifr_stdlib::http`. |
| `SIFR_E2E_FIXTURE_MANIFEST=<url_header_cookie_fixtures> SIFR_E2E_CACHE_DIR=target/sifr_e2e_cache/url-header-cookie-focused SIFR_E2E_DISABLE_CACHE=0 cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` | PASS | Selected batch e2e run for `network_http_header_cookie` and `network_http_url_query_percent`; 2 passed, 0 failed, cache hits 0/2 after the IPv6 fixture change. |
| `cargo test -p sifr_stdlib --test network_http_dependency_snapshots -- --nocapture` | PASS | Verifies generated dependency snapshots through URL/header/cookie readiness, no external cookie crate for cookie-header helpers, and Ring 5 absence from URL/header/cookie production dependencies. |
| `cargo fmt --check` | PASS | Rust formatting check. |
| `cargo clippy --workspace -- -D warnings` | PASS | Workspace clippy gate passed after Opus pass-3 remediation. |
| `verification/runner/e2e/run_e2e_pass.sh` | PASS | Full e2e pass suite completed 138 pass fixtures with 0 failures; report signature `4ede7c71d86f381c`. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Authoritative create-pr validation passed; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. |
| `scripts/run_all_tests.sh` | PASS | Full merge-gate validation passed after the final review evidence check; report `target/validation_lane_reports/merge.latest.json`; all 14 lane steps passed, wall time 783.02s, hardening failures 0, advisory: high e2e group skew only. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | File-size guardrail passed with 2319 files under the 900-line limit. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | HIR maintainability guardrails passed. |

## CPython Evidence

Mine `urllib.parse`, `http.cookies`, and selected `http.client` header behavior. `urllib.request` and `http.cookiejar` remain rejection/defer evidence.
