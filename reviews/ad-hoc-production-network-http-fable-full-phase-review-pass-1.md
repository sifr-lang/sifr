All verifications are complete. The dependency audit doc is itself stale ("active audit through M3" at phase exit) and names error variants (`NetError::{Io, Timeout, Cancelled, Closed}`, `NetError::UnsupportedHostOption`) that don't exist in the implementation. I have everything needed for the verdict.

---

# Full-Phase Review: Ad Hoc Production Network and HTTP Platform Substrate

Reviewer: Fable High · Date: 2026-06-12 · Scope: phase contract, execution ledger, closure artifacts, implementation on `main` (HEAD `5cf577061`)

## Verdict: **FAIL** — the phase is substantially well-executed, but it cannot remain closed as-is. One no-panic-contract violation and a cluster of closure-evidence contradictions must be fixed before the "completed, audited" status is honest.

---

## Blocking findings

### B1. User-triggerable runtime panic in net/tls timeout paths (no-panic contract violation)

`crates/sifr_runtime/src/net.rs:80-85` and `crates/sifr_runtime/src/tls.rs:75-79` validate a user-supplied timeout only with `is_finite() && > 0.0`, then call `Duration::from_secs_f64(seconds)`, which **panics on overflow** for finite positive values (e.g. `timeout=1e20`). A Sifr program passing a large `timeout=` float — possibly derived from config or external input — panics instead of receiving a typed `NetError`/`TlsError`. The HTTP module already closes exactly this hole with `MAX_HTTP_TIMEOUT_SECONDS = 86_400.0` (`crates/sifr_runtime/src/http.rs:16,49-56`); net and tls were left uncapped. This directly violates the phase guarantee at `issues/ad-hoc-production-network-http-platform-substrate.md:705` and `:1081`, and contradicts the final review's claim that the no-panic guarantee is satisfied (`reviews/ad-hoc-production-network-http-final-opus-review-pass-2.md:3,39`).

**Required fix (root cause):** unify timeout validation across the three runtime modules — one shared helper that caps or uses `Duration::try_from_secs_f64` and returns the typed error, plus a regression fixture (e.g. extend `network_http_m1_tcp_errors.sifr` with an overflowing timeout). Note the workspace clippy `unwrap_used`/`expect_used` lints cannot catch implicitly-panicking std calls, which is why this survived the panic gates (see N2).

### B2. Closed inventory and dependency audit record an error taxonomy that was never implemented

The phase contract requires 13 typed errors with nested evidence preservation (`issues/ad-hoc-production-network-http-platform-substrate.md:679-700`, e.g. `HttpError::Tls(TlsError::Transport(NetError))`). The shipped implementation has 8 flat message-string classes (`lib/sifr/net.sifr:26`, `tls.sifr:32,36`, `url.sifr:15`, `http.sifr:14-26`); `DnsError`, `ConnectError`, `TimeoutError`, `TooLargeError`, and `CancelledError` exist **nowhere** in `lib/sifr/`, `crates/sifr_runtime/`, `crates/sifr_codegen/src/preamble/`, or `crates/sifr_stdlib/` (verified by grep, zero hits). Yet the **closed** closure artifacts still assert the structured taxonomy as fact:

- `verification/stdlib/network_http_substrate_inventory.md:184-192` — "`DnsError` … nests in `NetError::Dns`", "`TooLargeError` … parser/body size-cap evidence"; `:140` — "`TooLargeError` nested in `BodyError`".
- `verification/stdlib/network_http_dependency_audit.md:7` — "`NetError::{Io, Timeout, Cancelled, Closed}`, higher layers nest `NetError`"; socket2 row — "`NetError::UnsupportedHostOption`".

The execution ledger records no amendment descoping the taxonomy to flat message classes. Either this is an untracked implementation gap against the contract, or an unrecorded descope whose closure evidence now misdescribes the shipped API — both block honest closure.

**Required fix (root cause):** decide explicitly. Either (a) implement the contracted variant-based taxonomy (the elegant outcome — message-string copying like `tls.sifr:189` does not preserve typed nested evidence), or (b) record a reviewed phase amendment in the ledger accepting the flat taxonomy, and rewrite the inventory Error Taxonomy section and audit-doc error-mapping cells to describe what actually exists. Cosmetic re-wording without the recorded decision is not acceptable.

### B3. Dependency evidence chain is internally contradictory and stale

Checklist requirement: snapshots and audits match the implementation. They don't:

- `verification/stdlib/network_http_dependency_snapshots.json` entry `network-runtime-core` shows tokio **without** `process`/`signal`, **with** `bytes`, and **omits** `sifr_runtime` — while the actual generator output (asserted by `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs:81-88`) is `sifr_runtime(net)` + tokio **with** `process`/`signal` + `tracing`, no `bytes`. The JSON's own `http-transport` entry contradicts its `network-runtime-core` entry on the tokio feature set.
- The JSON's `status` is `m4-implemented` and the audit doc opens with "Status: active audit through M3 implementation candidate. M4-M5 must update exact lockfile versions…" (`network_http_dependency_audit.md:3`) — neither was updated at M5/closure.
- The audit doc's tokio row (`:7`) claims "no … `process`, `signal` … for network feature", but every generated network manifest carries them (inherited concurrency-provider baseline, widened in the M1 commit). The provenance is defensible; the audit doc's claim as written is false for shipped binaries and the baseline inheritance is acknowledged nowhere.
- Minor: `http` documented/pinned as 1.4.1 (`network_http_dependency_audit.md:19`) but Cargo.lock resolves 1.4.2.

**Required fix:** regenerate the snapshot JSON from real generator output (ideally make the snapshot test diff the JSON against `generated_cargo_dependencies()` so it can never drift again — that's the root cause), update both status lines to closed/audited, and amend the tokio row to state that `process`/`signal` come from the provider baseline and the network feature adds `net`/`io-util` only.

### B4. The phase's enforcement tests are outside the authoritative validation gate

`scripts/run_all_tests.sh` (`run_crate_tests`, lines 345-383) runs `cargo test` for sifr_diagnostics, sifr_lowering, sifr_syntax, sifr_frontend, sifr_analysis, sifr_lsp, sifr_package, sifr, and `sifr_driver --lib` — **never** `cargo test -p sifr_stdlib` or `-p sifr_runtime`. The dependency snapshot tests (this phase's ring-isolation enforcement) and the runtime TLS/HTTP unit tests run only when invoked by hand, as they were during milestones. Since "CI mirrors these exact scripts," a regression in `features.rs` dependency emission would now pass the merge gate silently. The phase contract itself lists `cargo test -p sifr_stdlib` as required validation (M0 `:803`, M5 `:1066`). Functional runtime coverage does flow through the e2e pass fixtures, which softens but does not close this hole.

**Required fix:** add `cargo test -p sifr_stdlib` and `cargo test -p sifr_runtime` to `run_crate_tests` in `scripts/run_all_tests.sh` and record a gate run in the ledger.

### B5. Phase contract still says `Status: draft`

`issues/ad-hoc-production-network-http-platform-substrate.md:3` reads `Status: draft`, while the ledger says "completed, audited" (`…-execution.md:5`), the roadmap says "completed, audited" (`internal_docs/roadmap.md:73`), and the inventory JSON says `closed`. Sibling closed phases use `Status: completed …` / `Status: complete`. Trivial fix, but a phase whose own contract header says draft is not consistently closed.

### B6. M5 traceability claims handoff documentation that doesn't exist where it says it does

`verification/stdlib/network_http_m5_handoff_traceability.md:13` states that `docs/network_http.md` (among others) "record[s] the handoff and serving-scale deferral." `docs/network_http.md` contains **no** serving-scale or single-runtime-worker statement (verified by grep). The explicit "single-runtime-worker per process until the follow-up closes" boundary — required by M4 DoD (`…substrate.md:1020`) — appears only in the phase issue itself; `internal_docs/network_http_architecture.md:34,38` carries adjacent deferral language but not the per-process worker boundary, and the public doc carries nothing.

**Required fix:** add the single-runtime-worker-per-process boundary statement to `docs/network_http.md` (and ideally the architecture doc), or correct the traceability row to cite only documents that actually contain the record. Given Phase 41 is the consumer, putting the statement in the public doc is the root-cause fix.

---

## Non-blocking observations

- **N1 — Rejection-test coverage gaps:** all main CPython-shaped modules are rejected with stable `SIFR-IMPORT-0008/0009` diagnostics and `from X import y` fixtures, but no fixtures exercise the bare `import socket` / `import sifr.socket` statement form, and submodules `http.cookies`, `http.cookiejar`, `urllib.error`, `urllib.robotparser` rely on root-level matching without dedicated fixtures. The compiler handles these paths (`crates/sifr_lowering/src/lower/mod_impl.rs:711-730`); they're just untested forms.
- **N2 — Panic-scan blind spot:** the generated-code panic scan walks only the generated crate's `src/` (`verification/generated_code_quality/generated_code_quality.py:482-483`); `sifr_runtime` is covered only by clippy `unwrap_used`/`expect_used`, which misses implicitly-panicking std calls — exactly how B1 slipped through. Consider a panic-pattern lint pass over `sifr_runtime` network modules. Also, the quality manifest has no explicit M4 transport entries (mitigated by whole-constant preamble emission), and the M5 ledger honestly records that the broad demos-mode quality run stalled and a targeted scan was the authoritative signal.
- **N3 — Contract/API drift, minor:** `TcpReadHalf.close`/`TcpWriteHalf.close` (`lib/sifr/net.sifr:91,115`) and `TlsReadHalf.close` (`tls.sifr:137`) are synchronous, where the contract declares them `async` (`…substrate.md:458,462,534`); contract's keyword-only `*` markers aren't reflected. Worth a one-line recorded deviation.
- **N4 — Documentation depth:** ~20 implemented public functions (`resolve_host`, percent helpers, `parse_cookie_header`, config aliases, etc.) and all alias pairs are undocumented; docs are overview-level with no API reference. The docs consistently under-claim rather than over-claim, which is the right direction.
- **N5 — Architecture doc omissions:** no async-counterpart/`@blocking_io` policy section and no supported-host-matrix/host-limited reference in `internal_docs/network_http_architecture.md`; the Phase 41 doc states the multipart deferral but is silent on WebSocket/compression/HTTP3 needing separate phases (those exclusions live only in `docs/network_http.md:58`).
- **N6 — Unacknowledged transitives:** `rustls-platform-verifier 0.7.0` pulls `webpki-root-certs` (same Mozilla roots as the rejected `webpki-roots`, target-gated), `rustls-native-certs`, `security-framework`, `openssl-probe`, `jni` — none acknowledged in the audit doc, which asserts "no webpki-roots fallback." Direct-dependency policy is honored; the audit doc should note the verifier's own transitive root-store path.
- **N7 — Repo state:** the only uncommitted change on `main` is the empty placeholder `reviews/ad-hoc-production-network-http-fable-full-phase-review-pass-1.md` (0 bytes, untracked). All closure PRs #2494–#2500 are merged; no phase files changed after closure commit `5cf577061`.

## What is solid (verified, not just claimed)

- **Public/private boundary is exemplary.** `sifr.http_transport` was removed from `STDLIB_SOURCES` (commit `8ea028076`), is gated behind `allow_http_transport_harness_imports` defaulting to `false` (`crates/sifr_lowering/src/lower/mod_context.rs:167`, gate at `mod_impl.rs:310-323`), with a negative fixture proving `SIFR-IMPORT-0009` and an explicit `# sifr-e2e-allow-http-transport-harness` opt-in for harness fixtures. No Rust crate types leak from any public signature in `lib/sifr/{net,tls,url,http}.sifr`.
- **No substitute runtime model.** Zero local cancellation tokens, thread pools, shutdown coordinators, or diagnostic buses in production network code; cancellation/timeout uses `tokio::time::timeout` on provider primitives; the only `tokio::spawn` is the standard hyper connection driver with typed error mapping (`http.rs:224,268`).
- **Remote-data arithmetic is safe.** All size handling is `try_from`/`saturating_add` against hard caps (body limits `http.rs:172-185`, header section caps and URL/query caps in `url_http_runtime.rs`); lock poisoning is absorbed, never unwrapped.
- **Dependency rings hold at the manifest level.** Ring 5 crates are dev-only or absent; rejected crates (reqwest/axum/webpki-roots/native-tls/tower) appear nowhere as direct deps; `hyper_util_necessity.md` exists as required; all 16 spot-checked version pins match Cargo.lock.
- **Demos, e2e fixtures, milestone evidence, and the serving-scale follow-up issue all exist and are wired into validation lanes** (demos via `verification/generated_code_quality/manifest.json:12-14`, executed by `run_all_tests.sh:328,336`).

## Evidence inspected

Phase contract (full, 1140 lines), execution ledger, `internal_docs/roadmap.md:73`, both Opus review artifacts, M0–M5 traceability docs, `network_http_substrate_inventory.{md,json}`, `network_http_dependency_audit.md`, `network_http_dependency_snapshots.json` + its test crate, `docs/network_http.md`, `docs/stdlib_imports.md`, `internal_docs/network_http_architecture.md`, Phase 41 doc, `lib/sifr/{net,tls,url,http}.sifr`, `crates/sifr_runtime/src/{net,tls,http}.rs`, codegen preambles, `crates/sifr_stdlib/src/{lib,sources,features}.rs`, lowering import gates and diagnostics, e2e pass/fail fixtures, `scripts/run_all_tests.sh`, generated-code-quality scan and manifest, Cargo.toml/Cargo.lock, and git history through `5cf577061`. Six parallel investigations covered status consistency, API boundary, CPython rejection, panic safety, dependency rings, and docs/demos; every blocking finding above was then re-verified first-hand at the cited lines.

## Final conclusion

The substrate itself is close to the contract's intent — the boundary enforcement, provider consumption, and resource-limit discipline are genuinely production-grade. But the phase is **not safely closed today**: one real user-triggerable panic contradicts the headline guarantee (B1), and the closure evidence chain — error taxonomy (B2), dependency snapshots/audit (B3), validation gate (B4), contract status (B5), and the M5 traceability's handoff claim (B6) — contains contradictions that an audit must not leave standing. All six blockers are tightly scoped (one small runtime fix plus evidence/gate corrections, no new scope). After they land with a recorded re-validation pass, the phase will deserve its "completed, audited" status; until then, FAIL.
