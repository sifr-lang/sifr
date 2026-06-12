I have enough context. Let me draft the review.

---

# Fable gap-fix remediation review — `codex/network-http-fable-gap-fixes`

**Verdict:** PASS with one borderline residual (not a re-FAIL).

All six Fable blockers from `reviews/ad-hoc-production-network-http-fable-full-phase-review-pass-1.md` are addressed and locally verified (cargo tests pass; fixture runs without panic). The remediation is root-cause oriented: a shared `timeouts` module, a snapshot-vs-generator equality test, and explicit phase amendments rather than cosmetic re-wording.

The single borderline below is a residual of the same class as B2 in places Fable did not enumerate verbatim. Because B2's Required-Fix explicitly scoped to "the inventory Error Taxonomy section and audit-doc error-mapping cells" — both fully rewritten — and the new amendment text covers "and similar variant paths," I do not call it a new blocker. But it is the one place where an honest reader can still derive a contradicted API claim.

## Findings, by severity

### F1. Borderline — residual variant references in `network_http_substrate_inventory.md` outside the rewritten taxonomy section
The Error Taxonomy section (lines 183-194) was correctly rewritten to flat classes, and an explicit 2026-06-12 Fable amendment was added stating "and similar variant paths" are not shipped. Audit doc is now clean (grepped: no `Error::`). However, the same inventory file still asserts ~20 structured variants as the API contract in tables that appear **before** the amendment:
- `verification/stdlib/network_http_substrate_inventory.md:51` — `ProtocolError::UnsupportedExtensionFrame`
- `:79,93` — `HeaderError::InvalidName`
- `:94` — `HeaderError::ObsFold`
- `:97` — `ProtocolError::ConflictingContentLength`, `BodyError::LengthMismatch`
- `:98` — `ProtocolError::AmbiguousBodyLength`
- `:107` — `BodyError::TrailersUnsupported`
- `:111-112` — `BodyError::Cancelled { direction, bytes_observed }`
- `:113,125` — `ProtocolError::StreamReset { code, bytes_observed }` nested in `HttpError`
- `:124` — `ProtocolError::PingFlood`
- `:127` — `ProtocolError::MalformedFrame { kind }`
- `:134-139` — `UrlError::TooLarge`, `HeaderError::TooLarge`, `BodyError::TooLarge`
- `:149` — `UrlError::InvalidPort`
- `:151` — `UrlError::InvalidPercentEncoding`

Verified by grep: none of these names exist in `lib/sifr/{net,tls,url,http}.sifr` (only flat classes with `message: str`). The B2 amendment language ("and similar variant paths") does cover them in spirit, and the Fable required-fix didn't enumerate these specific cells. Treating it as a non-blocker is defensible, but the cleanest closure either:
- (a) rewrites these cells to "maps to `ProtocolError` with PING-flood/RST/reset/extension-frame evidence," matching the audit-doc style; or
- (b) hoists the amendment above the security/header/HTTP-2/size-limit tables so readers see the disclaimer first.

### F2. Non-blocking — overflow regression fixture covers only the network path
`crates/sifr/tests/e2e/pass/network_http_m1_tcp_errors.sifr:21-27` exercises `connect_tcp` and `resolve_host` with `timeout=1e20`. TLS and HTTP transport paths share the same `timeout_duration` helper, so panic-safety is structurally guaranteed across all three. Unit coverage in `crates/sifr_runtime/src/timeouts.rs:20-39` is the authoritative panic-safety signal. Consider one matching e2e assertion for `connect_tls`/`http1_request_tcp` only if you want surface-level proof too — but not required.

### F3. Non-blocking — snapshot equality test compares only `production_dependencies`
`network_http_snapshot_json_matches_generated_dependency_output` (`crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs:115-155`) checks the `production_dependencies` array vs `generated_cargo_dependencies()` for all four snapshots. Drift in `required_features`, `must_not_include`, or `manifest_codegen_requirements` is still possible (the pre-existing ring-5 test catches some of `must_not_include` semantics, not all). The B3 root-cause fix is satisfied for the part Fable called out; tightening to a structural equality on the rest of each snapshot would harden it further.

### F4. Non-blocking — `cargo test -p sifr_runtime` without features is mostly noop
`scripts/run_all_tests.sh:373-377` runs both bare and `--features http`. Because `http → tls → net`, the `--features http` invocation already covers every feature-gated module. The bare run only exercises unconditional modules and the `[cfg(test)]`-gated `timeouts` unit tests. Harmless; trim if you care about merge-gate wall-clock.

### F5. Non-blocking — `MAX_RUNTIME_TIMEOUT_SECONDS = 86_400` is now applied to net/TLS paths that were previously uncapped
`crates/sifr_runtime/src/timeouts.rs:3` — a tightening, not a loosening. Aligns net/TLS with the prior `MAX_HTTP_TIMEOUT_SECONDS`. No reasonable user would pass `>24h` to a TCP/TLS timeout; calling this out only to acknowledge it's a behavior delta over `main`.

### F6. Non-blocking — execution ledger marks remediation rows as `in progress`
`issues/…-execution.md:451-454` is honest given the changes are uncommitted. Once this branch merges, those rows should flip to `pass`/`done` with PR links, and the phase contract status line in `issues/ad-hoc-production-network-http-platform-substrate.md:3` ("…remediation in progress") should drop the trailing clause.

## Per-area verification

### 1. Runtime timeout overflow (B1) — fixed at root
- Shared helper `timeouts::timeout_duration(seconds, label)` (`crates/sifr_runtime/src/timeouts.rs:5-13`) caps at `MAX_RUNTIME_TIMEOUT_SECONDS = 86_400.0` before calling `Duration::from_secs_f64`, eliminating the overflow panic Fable cited.
- All three modules now call it: `net.rs:93`, `tls.rs:88`, `http.rs:60`. The bespoke `timeout_duration` in `http.rs` and the `is_finite() && > 0.0`-only checks in `net.rs`/`tls.rs` are deleted (`git diff` confirms).
- E2E regression: `network_http_m1_tcp_errors.sifr:21-27` adds `timeout=1e20` assertions for `connect_tcp` and `resolve_host`; `cargo run` of the fixture succeeds.
- Unit regression: `timeouts::tests::rejects_non_finite_non_positive_and_overflow_sized_timeouts` covers NaN/0/1e20 — passes locally.
- The `lib.rs` cfg `#[cfg(any(feature = "net", feature = "tls", feature = "http", test))]` lets the unit tests run even without features.

### 2. Snapshot-vs-generator drift (B3) — fixed with root-cause test
- `network_http_dependency_snapshots.json` regenerated to match `generated_cargo_dependencies()` output. Status flipped from `m4-implemented` to `closed-audited`. Source line now points at the test that enforces equality.
- New test `network_http_snapshot_json_matches_generated_dependency_output` (lines 115-155 of the test crate) decodes the JSON and asserts equality against the normalized generator output for all four snapshots. I traced the expected ordering through `generated_cargo_dependencies`/`STDLIB_FEATURE_SPECS`/the `StdlibFeature` enum order — the assertions are correct. `cargo test -p sifr_stdlib --test network_http_dependency_snapshots` → 9 passed.
- Audit doc status updated to `closed, audited through M5`; `http` row honestly records "generated spec 1.4.1, lockfile 1.4.2"; tokio row now states process/signal are inherited from the provider baseline (matching the `tokio_dependency_spec` logic at `features.rs:805-812`); cookie row recorded as rejected (matches `COOKIE_DEPS: &[]` at `features.rs:147`); the rustls-platform-verifier transitive-roots note Fable's N6 asked for is also present.

### 3. Authoritative validation gate (B4) — fixed
`scripts/run_all_tests.sh:370-377` adds `cargo test -p sifr_stdlib`, `cargo test -p sifr_runtime`, and `cargo test -p sifr_runtime --features http`. The latter actually exercises the gated network/TLS/HTTP modules. Both crates' test suites pass locally.

### 4. Closure-evidence honesty (B2 + B5 + B6) — fixed where Fable specified
- Phase contract status (`issues/…-platform-substrate.md:3`) flipped from `draft` to "completed, audited; post-closure Fable High gap review remediation in progress."
- Phase contract Typed-Errors section (lines 679-695) trimmed to the 8 shipped classes and carries the explicit Fable amendment. Tokio row (line 360) acknowledges provider-baseline `process`/`signal` inheritance. Body/header-size row (line 730) replaces `TooLargeError` with the owning class plus deterministic evidence.
- Substrate inventory: Error Taxonomy table fully rewritten; specific cells Fable cited (`TlsError::Cancelled`, `TlsError::TooLarge`, `BodyError::TooLarge`, `HttpError::ConnectionClosing`, `TlsError::Transport(NetError)` etc.) replaced with flat-class + evidence wording. See F1 for residual same-class cells.
- M5 traceability (`network_http_m5_handoff_traceability.md:13`): row reworded to "Protocol/runtime ready for one runtime worker per Sifr process; multi-core throughput deferred…" and the documents list now matches what those documents actually say.
- Public doc (`docs/network_http.md:45`) and architecture doc (`internal_docs/network_http_architecture.md:35`) now carry the explicit single-runtime-worker-per-process boundary statement Fable's B6 required.

### 5. Traceability of the gap-remediation pass — partial
- Execution ledger has a new "Post-closure Fable High gap review" table (`issues/…-execution.md:447-454`) listing the Fable artifact and the three in-progress remediation rows. Phase contract amendment cross-references the same date.
- The placeholder `reviews/ad-hoc-production-network-http-fable-gap-fixes-opus-review-pass-1.md` is 0 bytes — presumably the slot this very review fills. Once written, ledger and traceability would benefit from an explicit row pointing at it (mirroring how the M5 row points at the M5 reviewer artifact).

## What's solid (verified, not just claimed)

- `cargo test -p sifr_runtime --features http` → 36 passed (includes `timeouts`, the `http1_malformed_response`, body-limit, h2 SETTINGS/HPACK/GOAWAY/RST_STREAM, full TLS loopback split + mTLS reject + invalid-root fixtures).
- `cargo test -p sifr_stdlib --test network_http_dependency_snapshots` → 9 passed.
- The new e2e fixture runs end-to-end (`cargo run -p sifr -- run …network_http_m1_tcp_errors.sifr` succeeded; assertion failures would have non-zero-exited and broken caching).
- Hand-traced `generated_cargo_dependencies` for each of the four snapshot IDs reproduces the JSON's `production_dependencies` order exactly, including the sorted-BTreeSet ordering of `required_features` for `http-transport`.
- All `Error::Variant` mentions Fable verbatim cited (B2 inventory section + audit tokio/socket2 rows + the `:140` cell) have been rewritten. No `Error::` paths remain in the audit doc, the phase contract, the architecture doc, or the public doc — only in the inventory tables noted under F1.

## Suggested follow-ups before merge (none blocking)

1. Sweep the remaining inventory variant references (F1) in one pass, or hoist the amendment above the security/header/HTTP-2/size-limit tables. This is the one place an unhelpful reader can still derive a contradicted contract.
2. After merge: flip the three "in progress" rows in the execution ledger and the trailing clause in the contract status to "done," and add a ledger row pointing at the gap-fixes review artifact.
3. Optional: tighten the snapshot-equality test to also compare `required_features` / `must_not_include` arrays so any drift in those fields is caught.
