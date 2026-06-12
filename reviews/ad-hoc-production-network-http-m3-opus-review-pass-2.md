I confirmed the IDNA leak. Now I have enough to write the review.

---

## Verdict: **FAIL**

The M3 implementation is structurally sound, but it has one concrete contract violation that the traceability document explicitly says the implementation must prevent. There is also a fixture/coverage gap that lets the bug ship undetected, and a documentation/contract inconsistency in the path-normalization story.

---

## Blocking findings

### B1. URL/IDNA guard is bypassed by percent-encoded non-ASCII host bytes

**Contract (`verification/stdlib/network_http_m3_url_header_cookie_traceability.md:9`):**
> "Generated helper rejects non-ASCII authority hosts **before `url` crate IDNA behavior can leak**"

**M0 contract (issues/...platform-substrate.md):**
> "Unicode host canonicalization/IDNA consumes the text/i18n-approved Unicode version from M2."

**Implementation, `crates/sifr_codegen/src/preamble/url_http_runtime.rs:10-31`:**

`__sifr_url_reject_non_ascii_authority_host` only inspects the **literal** input bytes — it slices the authority out and checks `host.chars().any(|ch| !ch.is_ascii())`. Then `__sifr_url_validate_ascii_host` is run again on `parsed.host_str()`, which by that point is already the punycoded ASCII form.

I confirmed empirically against `url = "2.5.8"`:

```
PARSED  'https://%C3%A9.example/' -> host_str=Some("xn--9ca.example")
PARSED  'https://é.example/'      -> host_str=Some("xn--9ca.example")
```

So `parse_url("https://%C3%A9.example/")` is accepted today and the IDNA-converted punycode form is stored as `host` (and serialized form) — i.e., the url crate's bundled UTS46/Unicode tables silently produced user-visible state from non-ASCII bytes. That is exactly the leak the M3 traceability promises to prevent, and it ships text-decoding/Unicode behavior ahead of any text/i18n sign-off.

The blocked-state fixture (`crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr:65-71`) only tests the literal-non-ASCII path, which is why this slipped through.

**Remediation (one of):**

1. In `__sifr_url_reject_non_ascii_authority_host`, percent-decode the host portion before the ASCII check. Concretely, replace the `host.chars().any(...)` check with: percent-decode `host` to a `Vec<u8>` and reject if any decoded byte ≥ 0x80. This still accepts pure-ASCII percent-encodings like `%61` → `a`.
2. Or, after parsing, compare `parsed.host_str()` (lowercased) against the input host portion (lowercased, with percent-decoding) and reject if they differ — that catches both IDNA punycoding and any other host rewriting.
3. Add a positive fixture asserting `parse_url("https://%C3%A9.example/")` produces a `UrlError` with the same "non-ASCII URL hosts are blocked" message as the literal-non-ASCII case, and a second fixture asserting `parse_url("https://%61.example/")` still parses (host decodes to `a.example` which is ASCII).

This must be fixed before opening M3 for review; the contract language is unambiguous and the M3 traceability explicitly cites this exact leak as the reason the guard exists.

---

### B2. M3 traceability claim about path normalization contradicts the implementation it ships

**Traceability (`verification/stdlib/network_http_m3_url_header_cookie_traceability.md:10`):**
> "Path normalization | **Parsing does not silently normalize**; explicit dot-segment helper preserves encoded slash/backslash boundaries."

**Fixture (`crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr:19, 25`):**
```python
parsed: Url = parse_url("https://user:pass@example.com:8443/a/../b?x=1&x=2#frag")
...
assert parsed.get_path() == "/b"
```

The fixture only passes because `url::Url::parse` for special schemes (http/https) does the WHATWG "remove dot segments" step during parsing. The traceability claim that "parsing does not silently normalize" is therefore stale or misleading — parsing DOES silently normalize the path for special schemes, and the test relies on it.

This is blocking because the traceability is the M3 contract artifact that downstream M4/M5 reviewers (and Phase 41 / HTTP client consumers) will read to understand the substrate guarantees. M4 may build behavior on the (false) assumption that `Url.path` is the raw input path.

**Remediation:** rewrite that traceability row to state actual behavior:
> "Parsing applies WHATWG dot-segment removal for special schemes (http/https/ws/wss/ftp/file). Percent-encoded slash (`%2F`) is preserved as a segment byte, not a separator. The `normalize_path` helper applies the same dot-segment algorithm to opaque/relative inputs."

Add a fixture line asserting that `%2F` survives parse (e.g., `parse_url("https://h/a/%2F/b")` → `get_path() == "/a/%2F/b"`) so the "encoded slash boundary" guarantee is locked, not just stated.

---

## Non-blocking observations (worth recording before PR)

1. **`HeaderName` silently lowercases (`crates/sifr_codegen/src/preamble/url_http_runtime.rs:204-210`).** `http::HeaderName::from_bytes` accepts uppercase tokens and `parsed.as_str()` returns the canonical lowercased form, so `header_name("Content-Type").get_value() == "content-type"`. This is correct for HTTP/2 but is a silent transform that M4 consumers should expect. Add a fixture row asserting it explicitly so it's a documented invariant rather than incidental behavior.

2. **No `HeaderMap.get(name)` (`lib/sifr/http.sifr:41-52`).** Substrate-only, but every caller will iterate `items()` and re-implement case-insensitive lookup. Either expose a `get_all(name)` returning a list (cheap, contract-clean) or explicitly defer to M4 with a comment in the traceability.

3. **No per-value byte cap.** The `1024` header-count cap is hard-coded (`url_http_runtime.rs:227`), but individual `HeaderValue` byte length is unbounded. M0 explicitly tasks M3 with "size caps from inventory"; confirm the inventory's value-length cap and either enforce it now or record it as a deliberate M4 deferral in the traceability.

4. **Cookie parser uses naïve `;` split (`url_http_runtime.rs:240-252`).** This is fine for header-level parsing per RFC 6265 (cookies don't have semicolons in values without quoting, which the substrate doesn't try to support), but a fixture row asserting a single-cookie input with an embedded `=` (e.g., `parse_cookie_header("token=abc=def")` → `[("token", "abc=def")]`) would lock the behavior.

5. **`percent_encode` always uses `NON_ALPHANUMERIC`.** That's the most-aggressive set — `/` becomes `%2F`. Acceptable for substrate; a note in `sifr.url` docstrings would prevent users from reaching for it as a "URL-safe" encoder.

6. **`UrlError` message strings are matched as substrings by fixtures.** Locks message text into the public test contract. Consider adding a stable error-kind field on `UrlError` so M4 can branch on kind rather than `in e.message`.

---

## Validation gaps remaining before opening the PR

The M3 validation table in the execution ledger lists targeted checks only. The repo's `AGENTS.md` is explicit:

> Before considering any task done, run local validation on your changes:
> ```
> scripts/run_all_tests.sh --profile create-pr   # Fast signal — use for PRs
> scripts/run_all_tests.sh                       # Merge gate — default
> ```

Neither command appears in the M3 row of `issues/...execution.md` (compare to the M1 and M2 rows, both of which record `scripts/run_all_tests.sh --profile create-pr` PASS and `scripts/run_all_tests.sh` PASS with report paths and advisories). Specifically still owed before opening the PR:

- `cargo clippy --workspace -- -D warnings` (M3 ledger row absent; M0/M1/M2 all have it).
- `scripts/run_all_tests.sh --profile create-pr` with the report path under `target/validation_lane_reports/create-pr.latest.json`.
- Full `scripts/run_e2e_pass.sh` (M1 row recorded "138 pass tests"; M3 only recorded a 2-fixture selected manifest, which is not equivalent).
- Full merge-gate `scripts/run_all_tests.sh` is required *before milestone closure*, but the M3 row should at least cite it as pending — currently it isn't listed at all, which mirrors a stale-evidence pattern.

---

## Bottom line

Fix B1 (the IDNA bypass — change the guard to look at percent-decoded bytes, add the two fixture cases) and B2 (correct the path-normalization claim in the M3 traceability and add the `%2F`-preservation fixture). After those land and the standard `create-pr` and clippy validation lanes record PASS, M3 is acceptable to open as a PR. Without the B1 fix, M3 ships behavior that the M3 contract explicitly forbids, and the existing fixture set will not catch it.
