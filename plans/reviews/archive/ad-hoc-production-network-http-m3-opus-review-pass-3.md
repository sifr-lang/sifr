I have enough to render the verdict.

---

## Verdict: **PASS**

The pass-2 blockers are remediated and the remediations do not introduce new blockers. M3 is acceptable to open as a PR after the standard validation gates record PASS.

---

## B1 — percent-encoded non-ASCII host bypass: **fully remediated**

`crates/sifr_codegen/src/preamble/url_http_runtime.rs:40-46` now percent-decodes the extracted authority host and rejects any decoded byte `>= 0x80`, ahead of `url::Url::parse`, so the IDNA leak is closed.

Walked through the three contract cases against the new code:

- `https://é.example/` → `host = "é.example"`, the literal-ASCII check at line 35 rejects (chars include non-ASCII).
- `https://%C3%A9.example/` → all literal chars ASCII (passes line 35), but `percent_decode_str("%C3%A9.example")` yields `0xC3 0xA9 ...`, line 41 rejects.
- `https://%61.example/ok` → literal ASCII (passes line 35), decoded bytes `"a.example"` are all ASCII (passes line 41), URL parses, and `host_str() = "a.example"` survives the post-parse `__sifr_url_validate_ascii_host` cleanly.

Fixture `crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr:43-46,80-86` locks all three behaviors plus `%2F` path preservation.

Defense-in-depth notes:
- `__sifr_url_validate_ascii_host` (lines 62-67) also runs the decoded-byte check, so `__sifr_url_build` is protected against a caller passing `%C3%A9.example` as a literal host arg.
- `__sifr_url_reject_bad_percent` already runs inside `__sifr_url_validate_ascii_host`, so malformed `%xx` in build paths is rejected before any decoding interpretation.

---

## B2 — path normalization traceability: **fully remediated**

`verification/stdlib/network_http_m3_url_header_cookie_traceability.md:10` now reads:

> Parsing applies WHATWG dot-segment removal for special schemes. Percent-encoded slash (`%2F`) is preserved as a segment byte, not a separator.

This matches what `url::Url::parse` actually does, eliminating the false "parsing does not silently normalize" claim. The fixture at `network_http_m3_url_query_percent.sifr:45-46` (`parse_url("https://example.com/a/%2F/b").get_path() == "/a/%2F/b"`) locks the `%2F` preservation guarantee, and lines 19 + 25 lock the dot-segment removal claim against the implementation.

---

## New blocking findings introduced by the fixes: **none**

I checked the new code paths for regressions:

1. The new `percent_decode_str` call only runs on a slice of the input string and on a host argument already vetted to be all-ASCII; no UTF-8 panic surface, no allocations beyond the host slice.
2. `__sifr_url_validate_ascii_host` now also runs the decoded-byte check; for `__sifr_url_from_parsed` the input is `parsed.host_str()` which is already decoded (no `%` chars), so the extra check is a harmless no-op there.
3. Inventory caps line up with `network_http_substrate_inventory.json:191-198`:
   - URL hard 65 536 ≡ `__SIFR_URL_MAX_BYTES`
   - Header name hard 1 024 ≡ `__SIFR_HEADER_NAME_MAX_BYTES`
   - Header value hard 65 536 ≡ `__SIFR_HEADER_VALUE_MAX_BYTES`
   - Header section hard 1 048 576 ≡ `__SIFR_HEADER_SECTION_MAX_BYTES`
4. `requirements.rs:31-48` now pairs URL→{url, percent-encoding} and HTTP→{http, cookie} both for explicit names and the fallback `url_`/`http_` prefix branches, so direct-intrinsic users get the same preamble dependencies the generated helpers actually call.
5. `__sifr_http_is_cookie_value_byte` byte ranges admit `=` (0x3D ∈ 0x3C..=0x5B) and reject `;` (0x3B is excluded), matching the fixture expectations on both `token=abc=def` accept and `bar; HttpOnly` reject.
6. `url_http_runtime.rs` is 471 lines, well under the 900-line cap.

---

## Remaining validation before PR / merge

The user-reported focused suite is complete for the implementation gate. The remaining items are workflow gates, not implementation blockers:

**Required before opening the PR (per `AGENTS.md` and the M0/M1/M2 ledger pattern):**
- `cargo clippy --workspace -- -D warnings` — not yet recorded in the M3 row (M0/M1/M2 all have it).
- `scripts/run_all_tests.sh --profile create-pr` — not yet recorded; M3 row should cite the `target/validation_lane_reports/create-pr.latest.json` path the same way M1/M2 do.
- `scripts/run_e2e_pass.sh` — the M3 row currently only records a 2-fixture selected manifest. The full pass corpus is the M1/M2 precedent and would catch any unintended cross-fixture regression from the URL/HTTP preamble.

**Required before milestone closure (after PR opens):**
- `scripts/run_all_tests.sh` (full merge gate), with the merge-lane report path recorded in the M3 row.

These are normal workflow gates, not implementation blockers.

---

## Bottom line

B1 (IDNA bypass) and B2 (traceability/path-normalization claim) are both fully closed. No new implementation blockers introduced. M3 is acceptable to open as a PR once `cargo clippy --workspace -- -D warnings`, `scripts/run_all_tests.sh --profile create-pr`, and `scripts/run_e2e_pass.sh` are run locally and recorded in the M3 row, with `scripts/run_all_tests.sh` required before merge.
