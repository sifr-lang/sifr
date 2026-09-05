# Network HTTP M4 — agent Review Pass 2

Reviewer: agent
Scope: branch `codex/network-http-m4-client-server` working tree after pass-1 follow-up remediation. Verifies that pass-1 non-blocking notes N2, N3, N4, N5, and N6 are addressed without introducing a new blocker, that the HTTP/1 fixture scope fix preserves coverage, and that the M4 candidate is acceptable to open a PR after the standard validation surface listed in pass 1.

## Decision

**ACCEPT.**

All four code-level follow-ups (N2 bounded timeout, N3 explicit HTTP runtime preamble detection, N4 response-build error propagation, N6 connection-task abort on body-limit error) land cleanly and do not regress the pass-1 contract. N5 (HTTP/1 keep-alive deferral) and N1 (temporary public `sifr.http_transport` import path) are recorded as M5 handoff in `verification/stdlib/network_http_m4_http_transport_traceability.md:12-13` and rolled up in `issues/ad-hoc-production-network-http-platform-substrate-execution.md:188`. The HTTP/1 fixture scope fix is structurally correct in Sifr and does not reduce coverage. No new blocker.

The remaining validation gap before opening the PR is the unchanged pass-1 surface: `scripts/run_all_tests.sh --profile create-pr`, `cargo clippy --workspace -- -D warnings`, and the full `cargo test -p sifr --test e2e test_e2e_pass` (not just the three M4-only manifest entries) still must be run per `AGENTS.md` before pushing.

## Blocking findings

None.

## Verification of pass-1 follow-ups

- **N2 (timeout overflow)** — `crates/sifr_runtime/src/http.rs:16` introduces `MAX_HTTP_TIMEOUT_SECONDS = 86_400.0`. `timeout_duration` at `:49-57` now returns `Err("HTTP timeout is too large")` for any value above the cap, so `Duration::from_secs_f64` can no longer panic if M5 wires user-controlled timeouts through the existing `seconds, has_timeout` channel. Existing finite/positive guard is retained. Acceptable choice of ceiling (1 day) — well below the panic boundary, large enough not to constrain real timeouts.
- **N3 (string-match preamble detection)** — `crates/sifr_codegen/src/lib_modules_and_codegen.rs:449-458` now triggers `needs_http_runtime` when the lowered stdlib contains `__sifr_http1_`/`__sifr_http2_` or when any intrinsic name starts with `http1_`/`http2_`, in addition to the prior `__sifr_http_` / `http_` matches. The H2-only fixture path no longer relies on the accidental `__sifr_http_future` local-name match in `intrinsics/registry/url_http.rs`. The intrinsic-name check is sufficient on its own because `lower_http_intrinsic` registers entries named `http1_client_roundtrip_tcp`, `http1_server_respond_tcp`, etc. in `emitter.intrinsic_functions`.
- **N4 (server-side response build failure swallowed)** — `crates/sifr_runtime/src/http.rs:307-330` now matches `(request, &response_result)` and sends `Err("HTTP server response build failed: ...")` through the existing oneshot when request collection succeeded but response build failed. The peer still gets the 500 fallback for protocol-safety, but the harness caller now sees a typed transport error instead of a misleading "request OK, peer returned 500". `unwrap_or_else` is fed a cloned error string before the `Err` move — correct.
- **N5 (HTTP/1 keep-alive coverage trimmed)** — `verification/stdlib/network_http_m4_http_transport_traceability.md:13` now carries an explicit row: "HTTP/1 keep-alive | Deferred from the one-shot M4 loopback harness." with the M5 obligation. `issues/ad-hoc-production-network-http-platform-substrate-execution.md:188` rolls this up under the pass-1 review record. M0 acceptance no longer silently misses keep-alive — the deferral is on the M5 docket.
- **N6 (client connection task orphaned on body-limit error)** — `crates/sifr_runtime/src/http.rs:235-241` (HTTP/1) and `:278-285` (HTTP/2) now match `collect_limited`'s `Ok`/`Err`, and on the error path call `connection_task.abort()` before returning the typed error. The happy path still awaits the task so connection errors continue to surface. Behaviorally equivalent to the existing tests, which are unchanged.

## Verification of HTTP/1 fixture scope fix

`crates/sifr/tests/e2e/pass/network_http_m4_http1_loopback.sifr:60-105` keeps the response assertions and `server.join()` inside each `async with task.scope()` block:

- The original happy-path roundtrip + `assert response[...]` + `await server.join()` are inside `scope` (lines 60-74). Sifr's `task.scope()` requires spawned futures to be joined before scope exit, so this is the structurally correct shape and matches the rest of the codebase's async-with-scope fixtures.
- The independent-response-limit case (lines 82-96) and the body-from-bytes-too-large case (lines 99-105) are unchanged. All three assertions from pass 1 (`response[0] == 201`, `response[1] == "HTTP/1.1"`, `response[3] == b"pong-http1"`, plus the limit-independence boolean and the body-too-large boolean) are still present and still execute.

No coverage reduction. The fix is a scope-discipline correction, not a content cut.

## Non-blocking notes

- **N1 — `sifr.http_transport` public-namespace tension (carried)**: pass 1's N1 about the substrate harness sitting alongside public modules is now recorded as an M5 handoff in the traceability doc (`:12`) and is in practice protected at the lowering boundary — `crates/sifr_lowering/src/lower/mod_impl.rs:310-323` rejects the import unless `SIFR_E2E_ALLOW_HTTP_TRANSPORT_HARNESS` is set, and `crates/sifr/tests/e2e/fail/network_http_sifr_http_transport_internal.sifr:1-6` asserts that a user-shaped import without the env var emits `SIFR-IMPORT-0009`. Good current shape; the M5 task is to convert this from "env-var-gated public source" to a private alias or a test-only source so the lowering check becomes structural rather than ambient.
- **N7 — `version_label` `HTTP/unknown` arm still dead (carried from pass 1, informational)**: `crates/sifr_runtime/src/http.rs:121-131` is unchanged. Still acceptable as defensive default; remains a future cleanup.
- **Cookie-feature variant cleanup (carried)**: `StdlibFeature::Cookie` still exists with empty deps after the M3 `cookie` crate removal. Cosmetic only.

## Validation gaps before PR/merge

The reported pass-2 validation runs cover the focused M4 surface and the response-build error propagation path, but the pass-1 gaps are still open and required per `AGENTS.md` before opening the PR:

- `scripts/run_all_tests.sh --profile create-pr` — the authoritative pre-PR gate; still not in the reported pass-2 runs.
- `cargo clippy --workspace -- -D warnings` — the new code in `crates/sifr_runtime/src/http.rs` (timeout cap, body-limit abort branches, response-build error match) should clear pedantic lints; confirm before pushing.
- `cargo test -p sifr --test e2e test_e2e_pass` (full e2e pass suite without the M4-only manifest) — confirms the explicit `http1_`/`http2_` runtime preamble detection in `lib_modules_and_codegen.rs` does not change emission for M0-M3 fixtures, and confirms the cookie-feature-deps change and the `cookie` crate removal are still clean across all baseline fixtures.
- `scripts/run_all_tests.sh` — required for milestone closure (full merge gate). Not for pre-PR opening, but called out so it does not slip after PR review.

## Other observations (informational)

- The new `MAX_HTTP_TIMEOUT_SECONDS = 86_400.0` is a private constant of `crates/sifr_runtime/src/http.rs`. If M5 surfaces a stable Sifr-facing timeout, the cap should be promoted to the typed-error contract and quoted in the traceability table so user code can reason about it; not a pass-2 blocker.
- The `(request, &response_result)` match in `http_server_respond` formats the response-build error twice — once for the harness oneshot (`format!("HTTP server response build failed: {error}")`) and once for the peer 500 body (`Bytes::from(error)`). Both paths are correct and tested by inspection; no leak of the inner formatter into the wire body since the 500 just echoes the raw error string.
- The fail fixture `crates/sifr/tests/e2e/fail/network_http_sifr_http_transport_internal.sifr` is a useful safety net — it locks in the lowering-side rejection so a future namespace cleanup (the N1 M5 handoff) cannot silently regress.
- File-size, fmt, and HIR maintainability guardrails reported PASS — consistent with this review's inspection; no oversized first-party file in the touched set.
