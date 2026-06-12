# Network HTTP M3 Traceability: URL, Header, And Cookie Primitives

Status: implementation candidate ready for Opus review.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| `sifr.url.Url` and `UrlQuery` | `production-public`, stable-public-api over ASCII and already-valid Sifr text. | `lib/sifr/url.sifr`; `_sifr.url` signatures in `crates/sifr_stdlib/src/url.rs`; generated helpers in `crates/sifr_codegen/src/preamble/url_http_runtime.rs`; fixture `network_http_m3_url_query_percent.sifr`. |
| Percent helpers | Byte/ASCII/UTF-8 safe helpers accepted; named encodings blocked on text/i18n M1. | `percent_encode`, `percent_decode`, `percent_encode_bytes`, and `percent_decode_bytes`; fixture covers UTF-8 text, raw bytes, and invalid percent escapes. |
| Host and authority validation | ASCII domain labels, IPv4, IPv6, and already-punycode accepted; non-ASCII blocked until text/i18n M2; ports are decimal 0-65535. | Generated helper rejects non-ASCII authority hosts before `url` crate IDNA behavior can leak; fixture covers IPv4 parsing, IPv6 building, already-punycode host, and invalid port rejection. |
| Path normalization | Parsing applies WHATWG dot-segment removal for special schemes. Percent-encoded slash (`%2F`) is preserved as a segment byte, not a separator. | `normalize_path` helper and fixture coverage for parser dot-segment removal, explicit helper dot-segment removal, and encoded slash boundaries. |
| Query parsing/building | byte/ASCII/UTF-8 safe behavior only; sensitive query key redaction applies in observability. | `UrlQuery` stores ordered pairs; parse/build fixture covers duplicate keys, plus decoding, and empty/missing lookup behavior. |
| `sifr.http` header primitives | M3 owns canonical `HeaderName`, `HeaderValue`, and `HeaderMap`: ASCII token names, lowercase canonical names, obs-fold rejection, OWS trim only, duplicate order preservation, and size caps from inventory. | `lib/sifr/http.sifr`; `_sifr.http` signatures in `crates/sifr_stdlib/src/http.rs`; generated helpers use `http` crate validation; fixture covers lowercase canonicalization, duplicate order, OWS trim, invalid names, obs-fold rejection, and inventory hard caps. |
| Cookie header parsing | header-level parse/build only. | Cookie header parse/build uses ordered `list[tuple[str, str]]`; fixture covers parse/build, embedded `=` in values, and invalid cookie syntax. Cookie jars, signing, private cookies, persistence, and percent-decoded user text remain unexposed. |

## Validation Evidence

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo check -p sifr_stdlib -p sifr_codegen` | PASS | Verifies intrinsic signatures, generated dependency features, and URL/HTTP lowerer compilation. |
| `cargo build -p sifr --bin sifr` | PASS | Rebuilt the CLI after adding `sifr.url`, `sifr.http`, and generated helper preambles. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr` | PASS | Public URL/query/percent fixture type-checks. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_m3_header_cookie.sifr` | PASS | Public header/cookie fixture type-checks. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr` | PASS | Fresh generated build after reviewer remediation; covers URL parse/build, IPv4 parsing, IPv6 building, punycode host, percent-encoded ASCII host acceptance, literal and percent-encoded non-ASCII host blocked states, `%2F` path preservation, percent helpers, path normalization, query parse/build, and invalid port. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m3_header_cookie.sifr` | PASS | Fresh generated build after reviewer remediation; covers header name/value validation, lowercase canonicalization, OWS trim, duplicate order, obs-fold rejection, cookie header parse/build, and embedded `=` cookie values. |
| `cargo test -p sifr network_http_dependency_contract_tests -- --nocapture` | PASS | Verifies locked URL/header/cookie dependency specs and generated-Rust dependency inference for `url`, `percent-encoding`, `http`, and `cookie`. |
| `SIFR_E2E_FIXTURE_MANIFEST=<M3 fixtures> SIFR_E2E_CACHE_DIR=target/sifr_e2e_cache/m3-focused SIFR_E2E_DISABLE_CACHE=0 cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` | PASS | Selected batch e2e run for `network_http_m3_header_cookie` and `network_http_m3_url_query_percent`; 2 passed, 0 failed, cache hits 0/2 after the IPv6 fixture change. |
| `cargo test -p sifr_stdlib --test network_http_dependency_snapshots -- --nocapture` | PASS | Verifies M0-M3 generated dependency snapshots and Ring 5 absence from M3 production dependencies. |
| `cargo fmt --check` | PASS | Rust formatting check. |
| `cargo clippy --workspace -- -D warnings` | PASS | Workspace clippy gate passed after Opus pass-3 remediation. |
| `scripts/run_e2e_pass.sh` | PASS | Full e2e pass suite completed 138 pass fixtures with 0 failures; report signature `4ede7c71d86f381c`. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Authoritative create-pr validation passed; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | File-size guardrail passed with 2319 files under the 900-line limit. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | HIR maintainability guardrails passed. |

## CPython Evidence

Mine `urllib.parse`, `http.cookies`, and selected `http.client` header behavior. `urllib.request` and `http.cookiejar` remain rejection/defer evidence.
