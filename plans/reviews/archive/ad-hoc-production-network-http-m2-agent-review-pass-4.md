# Review of M2 TLS Runtime — Pass 4

Branch: `codex/network-http-m2-tls-runtime`, tip `d4e2feb1feef13c7fd037d14301531915ed75b2a` ("Update M2 TLS validation contracts"). PR: https://github.com/sifr-lang/sifr/pull/2496. This pass re-checks the branch tip after the post-pass-3 follow-up commit; the pass-3 PASS remains in scope for the M2 TLS runtime itself.

## 1. Result: PASS.

No blocking findings on the final branch tip. The follow-up commit is scoped to validation contracts: it ratifies a test expectation already true of the generated Cargo.toml and adds `CARGO_TARGET_DIR` awareness to two verification helpers. It introduces no behavior change in the TLS runtime, codegen, intrinsics registry, public Sifr surface, or fixtures.

## 2. Blocking findings

None.

The follow-up commit `d4e2feb1f` touches exactly three files and each change is harmless:

- `crates/sifr/tests/e2e_support/harness_behavior_tests.rs:519-523` — adds `"net"` to the asserted tokio feature list. This is a contract sync: the e2e harness already emits the `"net"` feature unconditionally via `tokio_dependency_spec()` at `crates/sifr/tests/e2e_support/fixture_dependency_paths.rs:46-49`, and `generate_cargo_toml`'s `"tokio"` branch at `crates/sifr/tests/e2e_support/fixture_compilation.rs:432-434` delegates to that function. The asserted string now matches the rendered string verbatim. The pass-3 ledger had already accepted the hardcoded `"net"` as non-blocking; this commit just lifts that into the harness contract test. There is no divergence from the stdlib's conditional `tokio_dependency_spec` at `crates/sifr_stdlib/src/features.rs:649-656`, because user code never goes through the e2e harness function.
- `verification/performance/run_benchmarks.py:38-58, 347, 432-445, 448-460, 463-466` — replaces module-constant binary paths with `cargo_debug_dir()` / `frontend_bench_binary()` / `sifr_binary()` helpers that honor `CARGO_TARGET_DIR`. Pure refactor; no functional change for default `target/debug` runs, and the M2 reviewer's `CARGO_TARGET_DIR=target/codex-clippy` workflow is preserved.
- `verification/tooling/check_diagnostic_source_canonicalization_rules.py:166-174, 192, 212` — adds the same `cargo_debug_dir()` helper and routes the two `sifr_binary()` / `diagnostic_rendering_harness_binary()` lookups through it. Same refactor profile; the relative path is normalized against `REPO_ROOT` exactly as the benchmark runner does. No security or correctness impact.

I also re-verified the M2 TLS runtime surface against the pass-3 acceptance:

- `crates/sifr_runtime/src/tls.rs:524-536, 512-522` — idempotent `close_notify` and `flush`-after-`close_notify` semantics unchanged.
- `crates/sifr_runtime/src/tls.rs:711-744, 746-785` — mTLS missing-client and invalid-root tests still assert the failing side (`server.expect_err(...)`, `client.is_err()` plus contains check).
- `crates/sifr/tests/e2e/pass/network_http_m2_tls_loopback_split.sifr:14-19, 63-72` — long-lived (2026-06-12 → 2126-05-19) localhost cert/key bytes are unchanged; repeated `close_notify`, post-`close_notify` `flush`, and write-after-`close_notify` typed error coverage is still present.
- `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs:96-141` — generated M2 dependency snapshot still asserts `rustls = "=0.23.35"`, `tokio-rustls = "0.26.4"`, `rustls-pemfile = "2.2.0"`, `rustls-platform-verifier = { version = "0.7.0", default-features = false }`, and explicit absence of `rcgen` / `webpki-roots` / `x509-parser` from production deps.
- `crates/sifr_runtime/Cargo.toml:9-47` — `tls` feature still pulls `net` and the four optional TLS dependencies; production graph has no `rcgen` (dev-only).

## 3. Non-blocking findings / recommendations

All standing items are carried over from pass 3; none of them are introduced by this commit, and the ledger has already accepted them as out of scope for the M2 PR. Recorded here so the next milestone does not re-discover them:

- `crates/sifr_runtime/src/tls.rs:420-434` — `tls_stream_split` still mints phantom halves when `STREAMS.remove(&handle)` is `None`, to keep the Sifr `split()` contract infallible. Phantom halves surface a generic "handle is closed or unknown" on first use; a runtime invariant log/assert would tighten the contract in a later cleanup.
- `crates/sifr_runtime/src/tls.rs:333-374` — `tls_stream_read_chunk` / `tls_stream_write` / `tls_stream_write_all` restore the stream after I/O error. Rustls sessions are typically poisoned after a TLS-layer error; restoring the handle lets callers keep poking. Either terminate the handle on TLS error or document the poisoned semantics in a follow-up.
- `crates/sifr_runtime/src/tls.rs:46-73` — fallible `next_handle` (errors at `i64::MAX`) coexists with `next_handle_infallible` (wraps to 1). Unreachable in practice, but the wrap could theoretically collide with low-numbered live handles after `i64::MAX` exhaustion. Worth aligning.
- `crates/sifr/tests/e2e_support/fixture_dependency_paths.rs:46-49` — `tokio_dependency_spec()` is hardcoded with `"net"`. This pass now also locks the same string into the harness contract test. Harmless in practice but worth pruning when a non-network fixture exercises the helper.
- `lib/sifr/tls.sifr:46-94, 126-150` — `_closed` bookkeeping is set on `close(own self)` / `split(own self)` but never read; `own self` already forecloses re-use. Could be dropped in a later sweep.

## 4. Validation review

The validation set quoted in the prompt is sufficient for the M2 merge gate:

- `scripts/run_all_tests.sh --profile create-pr` PASS, report `target/validation_lane_reports/create-pr.latest.json`, advisory only (warm wall-time exceeded).
- `scripts/run_all_tests.sh` PASS, `target/validation_lane_reports/merge.latest.json`, `wall_time=799.89s`, advisory only (high e2e group skew); no `"failed"` / `"failure"` entries in the merge report.
- `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, file-size guardrail, HIR maintainability guardrail all PASS.
- Focused M2 runtime / stdlib / e2e checks PASS as enumerated in `issues/ad-hoc-production-network-http-platform-substrate-execution.md:299-315` and `verification/stdlib/network_http_m2_tls_traceability.md:22-39`.

The follow-up commit is also covered indirectly:

- The harness-contract test it edits (`test_generate_cargo_toml_required_tokio_uses_runtime_features` at `crates/sifr/tests/e2e_support/harness_behavior_tests.rs:516`) is included in `cargo test -p sifr` and ran inside the merge gate.
- `verification/performance/run_benchmarks.py` is exercised by `scripts/run_all_tests.sh` (merge profile includes the performance lane); the merge run passed.
- `verification/tooling/check_diagnostic_source_canonicalization_rules.py` is invoked from `scripts/run_all_tests.sh`; the merge run passed.

## 5. Acceptability

PR #2496 is acceptable to merge now.

The implementation candidate, the public Sifr surface, the fixture corpus, the dependency snapshots, the host matrix, and the validation gates have all passed at branch tip `d4e2feb1f`. The follow-up commit is a low-risk validation contract sync that did not regress any pass-3 evidence and that has been observed clean under the full merge gate. After merge, the M2 milestone closure on `issues/ad-hoc-production-network-http-platform-substrate-execution.md:22-26, 240-247` and the implementation merge ledger entry can land in the same PR or a fast follow-up.
