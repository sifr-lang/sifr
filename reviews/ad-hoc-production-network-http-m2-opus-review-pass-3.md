I have enough to render a verdict. I verified the cert PEM ASN.1 dates (`NotBefore=2026-06-12T02:06:01Z`, `NotAfter=2126-05-19T02:06:01Z` — ~100y), the public fixture covers double `close_notify` and post-`close_notify` flush (lines 63-65), and the runtime + dependency surfaces match what the traceability and dependency-audit docs claim.

## Review of M2 TLS Runtime — Pass 3

**1. Result: PASS.**

The pass-1 blocking finding (24-hour public cert time-bomb) remains remediated; the public Sifr fixture now also covers the M2 DoD bullet (repeated `close_notify` and empty `flush()` after `close_notify`). I found no new blocking issues and observed two pass-2 non-blocking recommendations actually adopted in this candidate.

**2. Blocking findings**

None.

The pass-1 cert blocker at `crates/sifr/tests/e2e/pass/network_http_m2_tls_loopback_split.sifr:15-19` stays fixed: ASN.1 decode of the embedded PEM yields a `CA:FALSE` localhost/127.0.0.1 SAN leaf valid through 2126-05-19. The pass-2 follow-up gap (DoD coverage at the public fixture level) is closed at `crates/sifr/tests/e2e/pass/network_http_m2_tls_loopback_split.sifr:63-65`: `_client_notify` → `_client_notify_again` → `_client_closed_flush` exercises the runtime's idempotent close-notify and post-close-notify flush guards (`crates/sifr_runtime/src/tls.rs:524-536, 512-522`).

**3. Non-blocking findings / recommendations**

Adopted since pass 2 (verified):
- `crates/sifr_runtime/src/tls.rs:736-738` — mTLS test now tightens to `server.expect_err(...)` and `assert!(server_error.contains("TLS server handshake failed"))`, proving the rejection path.
- `crates/sifr_runtime/src/tls.rs:748-750` — `let unrelated_root_materials = materials();` clarifies the wrong-root construction.

Still standing (intentionally out of scope for M2 PR):
- `crates/sifr_runtime/src/tls.rs:420-434` — `tls_stream_split` returns freshly minted phantom halves when `STREAMS.remove(&handle)` is `None`. Pass-1/pass-2 already accepted this as deliberate to keep the Sifr-level `split()` contract infallible; phantom halves still surface the generic "handle is closed or unknown" on first use. Consider logging or asserting at the runtime invariant level in a later cleanup.
- `crates/sifr_runtime/src/tls.rs:333-348, 351-374` — `tls_stream_read_chunk` / `tls_stream_write` / `tls_stream_write_all` restore the stream on I/O error. After a TLS-layer error the session is usually poisoned; restoring the handle lets callers keep poking. Document or terminate-on-error in a follow-up.
- `crates/sifr_runtime/src/tls.rs:46-73` — `next_handle` (fallible, bails at `i64::MAX`) and `next_handle_infallible` (used by `tls_stream_split`, wraps to 1 on `i64::MAX`) are inconsistent. Practically unreachable, but a wrap could theoretically collide with very-low-numbered live handles. Worth aligning behavior eventually.
- `crates/sifr/tests/e2e_support/fixture_dependency_paths.rs:46-49` — `tokio_dependency_spec()` is now hardcoded with `"net"` for every fixture; harmless because tokio is dev-only here, but worth pruning when a non-network fixture exercises this helper.
- `lib/sifr/tls.sifr:88-94` — `TlsStream._closed` is set by `close(own self)` / `split(own self)` but never read (the `own self` consumption already forecloses re-use). Same dormant bookkeeping exists in `TlsReadHalf` / `TlsWriteHalf`. Not a correctness issue; could be dropped in a later sweep.

**4. Validation / documentation gaps**

The focused validations listed in the prompt (cargo check across crates with `tls`, `sifr_runtime` `tls::tests`, dependency-snapshot test, both M2 fixtures via `sifr run`, fixture-manifest M2 run, `cargo fmt --check`, both guardrail scripts) cover the surface that changed. The remaining required gates per `issues/ad-hoc-production-network-http-platform-substrate-execution.md:319-323` have NOT been re-run on this candidate and stay required before PR/merge:

- `cargo clippy --workspace -- -D warnings` — required baseline.
- `scripts/run_all_tests.sh --profile create-pr` — required PR gate.
- `scripts/run_all_tests.sh` — required merge gate before milestone closure.

The accidental full e2e pass run noted in the prompt is correctly demoted to "exploratory broad-suite advisory" in `verification/stdlib/network_http_m2_tls_traceability.md:38` and the execution ledger M2 broad-pass note. The IO and `bytes_conversion_errors` failures it surfaced are pre-existing and not in M2 scope. The two M2 fixtures themselves pass under the targeted manifest, which is the authoritative M2 e2e signal.

Recommended next step: run the three gates above (clippy → create-pr → merge gate). If they pass, open the M2 PR.
