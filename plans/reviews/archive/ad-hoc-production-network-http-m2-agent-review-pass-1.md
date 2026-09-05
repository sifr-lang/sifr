I have enough to render a verdict. Let me also quickly confirm the runtime test coverage of "empty flush after close_notify" and double close_notify, which the runtime test does cover, and verify the e2e fixture does not.

Based on the e2e fixture I already read (lines 56-71 of `network_http_m2_tls_loopback_split.sifr`), the public fixture only does a single `close_notify` followed by a rejected late write and a reader.close(). No double close_notify or post-close_notify flush. The runtime test `tls_loopback_split_close_notify_and_alpn` does cover those at line 692-694.

## Review of M2 TLS Runtime

**1. Result: FAIL.**

The implementation is largely sound — clean Sifr-native API, no rustls/tokio-rustls type leaks, correct feature gating, no production rcgen/webpki-roots/x509-parser, deterministic split halves, typed errors — but one fixture issue is hard-blocking.

**2. Blocking findings**

- **`crates/sifr/tests/e2e/pass/network_http_m2_tls_loopback_split.sifr:15-19` — TLS cert and key have a 24-hour validity window.** Decoded ASN.1 UTCTime fields are `NotBefore = 2026-06-12 01:34:31Z` and `NotAfter = 2026-06-13 01:34:31Z`. Today's date is 2026-06-12, so the fixture passes now, but rustls will reject the cert tomorrow at `01:34:31Z` and the e2e pass suite (including the merge gate) will start failing for everyone, on every host, without any code change. This is a time-bomb in a fixture that's supposed to be deterministic. Fix by either (a) regenerating the embedded PEM with a multi-year validity (10y is standard for test fixtures), or (b) generating certs at fixture run time via a dev/test-only helper (rcgen is already a dev-dep and the runtime tests use this pattern). Hard blocker — do not open PR until resolved.

**3. Non-blocking recommendations**

- **M2 DoD evidence gap in the public e2e fixture.** The phase contract explicitly echoes "repeated close_notify and empty-flush-after-close-notify fixture coverage" in M2 DoD. The runtime test `crates/sifr_runtime/src/tls.rs:692-694` covers it (`tls_write_half_close_notify` called twice followed by `tls_write_half_flush`), but the public `network_http_m2_tls_loopback_split.sifr` only does a single `close_notify` then a late-write rejection. Consider lifting both behaviors into the .sifr fixture so the public surface is what proves the DoD bullet.
- **`tls_stream_split` is silently lossy on unknown handles.** `crates/sifr_runtime/src/tls.rs:420-434` returns two freshly-allocated handles even when `lock(&STREAMS).remove(&handle)` returns `None`, leaving phantom halves that fail on first use. The Sifr-level contract says split is infallible, but a runtime-level invariant violation should at least be logged or asserted; today it produces confusing "handle is closed or unknown" errors with no provenance back to the bad split.
- **`mtls_rejects_missing_client_certificate` and `invalid_root_rejects_server_certificate`** use `assert!(server.is_err() || client.is_err(), ...)` (`crates/sifr_runtime/src/tls.rs:736-739`, `770-773`). Tighter assertions on the expected side (server-side error for missing client cert; client-side error for untrusted root) would prove the rejection path is the one being exercised, not the symmetric I/O EOF.
- **`fixture_dependency_paths.rs::tokio_dependency_spec()`** still omits `"net"` from the e2e-generated `Cargo.toml`. It works in practice because `sifr_runtime`'s `tls`/`net` feature transitively activates `tokio/net` via the workspace dep, but the explicit dep string is misleading for anyone reading generated test crates. Either add `"net"` here or document why it's intentionally omitted.
- **`tls_stream_read_chunk` and friends re-insert the stream on I/O error** (`crates/sifr_runtime/src/tls.rs:336-348`). After a TLS-layer error the session state is generally not usable; restoring the handle lets users keep poking at a poisoned stream and surfaces follow-on confusing errors. Behavior is documented as deterministic, but consider terminating the handle on any error other than benign `WouldBlock`-class ones, or document the restore explicitly.

**4. Validation gaps**

The M2 ledger itself lists the gates that have NOT been re-run since the final docs/version edits:
- `cargo clippy --workspace -- -D warnings` — not run for this candidate.
- `python3 scripts/check_hir_maintainability_guardrails.py` — not run after final edits.
- `scripts/run_all_tests.sh --profile create-pr` — not run.
- `scripts/run_all_tests.sh` (merge gate) — not run.

These are explicit "M2 remaining gates before PR" per `issues/ad-hoc-production-network-http-platform-substrate-execution.md:307-312` and must pass after the cert blocker is fixed and before opening the PR. The focused validation in the ledger (`cargo check`, `cargo test -p sifr_runtime --features tls tls::tests`, `cargo test -p sifr_stdlib --test network_http_dependency_snapshots`, `cargo run -- run` on both fixtures) all passed today only because today is 2026-06-12; rerunning the same fixture commands tomorrow against the same source will fail.

Recommended next step: regenerate the embedded PEM in the loopback fixture with a long-lived (≥10y) cert, then run the four ledger-required gates above. The rest of the M2 implementation looks ready.
