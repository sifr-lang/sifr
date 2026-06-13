# Network HTTP M4 — Opus Review Pass 1

Reviewer: Claude Opus 4.7
Scope: branch `codex/network-http-m4-client-server` working tree, including untracked files. M4 candidate for HTTP Core Transport.

## Decision

**ACCEPT with non-blocking follow-ups.**

The HTTP/1.1 + HTTP/2 client/server transport substrate is correct, panic-free in user paths, properly feature-gated, and exercised by the three required loopback fixtures (HTTP/1 cleartext, HTTP/2 h2c, HTTPS h2). The dependency snapshot, contract tests, traceability doc, and Hyper-Util necessity note are coherent and match the phase contract. No findings rise to a blocker; the items below are worth addressing before the integration milestone (M5) or in a small fixup commit, but none block M4 merge.

## Blocking findings

None.

## Non-blocking notes (ordered by severity)

### N1 — `sifr.http_transport` placed in the public stdlib namespace
- `crates/sifr_stdlib/src/sources.rs:106` registers `sifr.http_transport` alongside the public stdlib modules.
- `lib/sifr/http_transport.sifr:1` self-describes as "internal HTTP transport harness surface", and the phase contract explicitly classifies the loopback transport harness as `test-only-harness` / "never a public ... module" (`issues/ad-hoc-production-network-http-platform-substrate.md:75`, `:600`, `:983`, `:1131`).
- Effect: any user `.sifr` file can `from sifr.http_transport import transport_http1_client_roundtrip_tcp` etc. and we are implicitly committing to that surface, even though the tuple shape is awkward and the harness is intended for internal validation only.
- Suggested follow-up (M5 hand-off is fine): either gate import behind a `_sifr.http_transport` private alias, restrict the source to tests, or add an explicit "this is a substrate harness; do not depend on it" marker that the importer surface tooling can enforce. At minimum, the M4 traceability row "Server accept/dispatch/shutdown substrate" should call out the public-namespace tension so it is not lost.

### N2 — Latent panic in `Duration::from_secs_f64` once timeouts are wired
- `crates/sifr_runtime/src/http.rs:47-52` validates `is_finite() && > 0.0` but does not bound the magnitude. `Duration::from_secs_f64` panics on overflow (~5.8e11 years).
- Currently unreachable because the codegen preamble hardcodes `0.0, false` for `seconds, has_timeout` (`crates/sifr_codegen/src/preamble/url_http_runtime.rs:500-509` and siblings), so user input never reaches it.
- Suggested follow-up: cap the value at a sane maximum (e.g. `u64::MAX as f64 / 2.0` seconds or a domain ceiling like 1 day) and return `Err("HTTP timeout is too large")` instead, so the path stays panic-free once M5 or a later milestone exposes user-controlled timeouts.

### N3 — `needs_http_runtime` detection relies on accidental string match
- `crates/sifr_codegen/src/lib_modules_and_codegen.rs:449-455` triggers the HTTP preamble when the lowered stdlib contains the substring `__sifr_http_` OR an intrinsic name starts with `http_`.
- Programs that import only `sifr.http_transport` (e.g. the H2 fixture) reference `__sifr_http1_*` / `__sifr_http2_*` — none of which contain `__sifr_http_`. The only reason the preamble is still emitted is the local variable name `__sifr_http_future` introduced by `boxed_async_http_helper_call` in `crates/sifr_codegen/src/intrinsics/registry/url_http.rs:28-35`. Rename that local and the H2 path silently stops emitting the runtime.
- Suggested follow-up: make the trigger explicit, e.g. add `function.starts_with("http1_") || function.starts_with("http2_")` to the OR chain, or check `stdlib_preamble.contains("__sifr_http1_") || ...contains("__sifr_http2_")`.

### N4 — Server-side response build failure is swallowed into a 500
- `crates/sifr_runtime/src/http.rs:301-307`: when `build_response` returns `Err(...)`, the closure produces a 500 with the error string in the body via `.unwrap_or_else(...)`. No tracing/log emission, no propagation up through the captured oneshot.
- This is not a panic — but a server fixture that constructs an invalid status/header set will look like a working request that returned 500, which is harder to debug than a typed transport error. Consider emitting a `tracing::warn!` here, or surface the failure via the oneshot (replace `Ok(request_parts)` with a recorded "response build failed" variant).

### N5 — HTTP/1 keep-alive coverage trimmed without an explicit follow-up
- The runtime always injects `Connection: close` on HTTP/1.1 responses when not already set (`crates/sifr_runtime/src/http.rs:159-163`). This is consistent with one-shot loopback fixtures, but the M0 acceptance line originally included "keep-alive" and the trimmed traceability row no longer mentions it (`verification/stdlib/network_http_m4_http_transport_traceability.md:6`).
- Suggested follow-up: either add a keep-alive fixture later in M4/M5, or record explicitly in the issue ledger that keep-alive validation is deferred and to which milestone, so it does not silently fall off the plan.

### N6 — Client connection task orphaned on body-limit error
- `crates/sifr_runtime/src/http.rs:230-237` (HTTP/1 client) and `:268-275` (HTTP/2 client): if `collect_limited` errors, `?` short-circuits and the previously spawned `connection_task` is never awaited. Tokio will reap it, but the task continues until it self-completes (it tries to drive the connection to completion against a dropped body). Cosmetic only; consider `.abort()` on the JoinHandle before returning the error.

### N7 — `version_label` fallback never reachable but documented as "HTTP/unknown"
- `crates/sifr_runtime/src/http.rs:116-126`: the runtime only ever builds HTTP/1.1 or HTTP/2, so the `_ => "HTTP/unknown"` arm is dead code today. Acceptable, but consider an `assert!` (programmer invariant per AGENTS.md) or `unreachable!` to make the contract explicit, or document the intent so it does not get misused later as "any version we receive back is mapped to a label."

## Validation gaps

The Codex-reported runs cover the core M4 paths well, but the following should be run before opening the PR per `AGENTS.md`:

- `scripts/run_all_tests.sh --profile create-pr` — the authoritative pre-PR gate; not in the reported run list.
- `cargo clippy --workspace -- -D warnings` — workspace lints (the new `crates/sifr_runtime/src/http.rs`, `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs`, and `crates/sifr_runtime/src/net.rs:294` `consume_stream_for_http` thin wrapper should clear pedantic warnings cleanly).
- `cargo fmt --check` was reported, but the snapshot of pre-existing `crates/sifr/tests/e2e_support/fixture_compilation.rs` removed many lines — confirm `git diff --check` shows no whitespace damage.
- Full `cargo test -p sifr --test e2e test_e2e_pass` (not just the three M4 fixtures with `SIFR_E2E_FIXTURE_MANIFEST`) — confirms M0-M3 fixtures still pass after the cookie-feature mapping change in `crates/sifr_codegen/src/intrinsics/registry.rs:467-475` and the dropped `cookie` crate dependency.
- One soak run with `cargo test -p sifr_stdlib --test network_http_dependency_snapshots` plus the M3 snapshots verifies the `cookie = "0.18.1"` removal does not break the M3 baseline assertions kept in the same file.

## Other observations (informational)

- The cookie-feature requirements migration (`crates/sifr_codegen/src/intrinsics/registry/requirements.rs:21-83`) cleanly removes the `cookie = "0.18.1"` direct dependency that was a leftover from M3, and the snapshot tests now assert the cookie crate is absent. `StdlibFeature::Cookie` still exists with an empty deps list — fine, but a future cleanup pass could remove the variant entirely.
- `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs` is a clean extraction of the dependency generator from `fixture_compilation.rs`. The two `match crate_name.as_str()` loops over `required_crates` (lines 154-179 and 181-297) could be merged into one, but this is style only.
- Pinned versions agree across `Cargo.toml`, `crates/sifr_stdlib/src/features.rs`, `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs`, and `verification/stdlib/network_http_dependency_snapshots.json`. `Cargo.lock` confirms `hyper 1.10.1`, `hyper-util 0.1.20`, `h2 0.4.14`.
- `verification/stdlib/hyper_util_necessity.md` is appropriately scoped — single-purpose justification, feature set, and the explicit "no public Sifr contract dependency" statement.
- The fixture certificate in `crates/sifr/tests/e2e/pass/network_http_m4_https_h2_loopback.sifr:16` is valid until 2126-05-19 — long-lived enough that fixture rot is not a near-term concern.
