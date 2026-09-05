All verification is complete. Here is the follow-up full-phase review.

---

# agent High follow-up review — network/HTTP substrate gap-fix branch (`codex/network-http-agent-gap-fixes` working tree vs `main`)

## Verdict: **PASS** — all six pass-1 blockers are fixed at root cause, verified first-hand. The phase is honestly closeable once the standard merge mechanics below land. No blocking findings remain.

## Findings (none blocking)

**F1 — Merge mechanics still open, honestly recorded.** The three ledger remediation rows (`issues/ad-hoc-production-network-http-platform-substrate-execution.md:454-456`) say `in progress`, the contract status carries "remediation in progress" (`issues/ad-hoc-production-network-http-platform-substrate.md:3`), and no post-remediation full-gate run (`scripts/run_all_tests.sh --profile create-pr`) is recorded — pass-1 B4's required fix included recording a gate run. This is correct while the work is uncommitted, but closure is conditional on: running and recording the create-pr/merge gates, flipping the rows to done with the PR link, dropping the trailing status clause, adding ledger rows for the four review artifacts, and filling the 0-byte placeholder `reviews/ad-hoc-production-network-http-agent-full-phase-review-pass-2.md` with this review.

**F2 — Residual typed-nesting wording, pre-existing on main.** The audit doc's rustls row still says "nested `NetError`" and the tokio-rustls row "nested lower evidence" (`verification/stdlib/network_http_dependency_audit.md:15-16`), and the contract's M0/M2 planning sections retain "typed `TlsError`/nested `NetError` evidence" (`…substrate.md:557,565,886`). These are flat class names rather than variant paths, the dated amendment at `…substrate.md:690` now governs interpretation, and rewriting milestone-planning history would be falsification — so non-blocking. The two audit-doc cells could be reworded in a follow-up sweep for full consistency.

**F3 — M5 traceability under-claims the new enforcement.** The "Generated dependency snapshots" row's evidence cell (`network_http_m5_handoff_traceability.md:10`) predates the remediation and doesn't cite the new snapshot-equality test, even though that test is now what satisfies the row's "resolver-backed" requirement. Under-claiming, not contradiction.

**F4 — E2E overflow coverage is net-only, by design.** The fixture exercises `connect_tcp`/`resolve_host` with `timeout=1e20`; TLS and HTTP paths are covered structurally (same shared helper) plus unit tests. Matches agent F2's assessment; no action needed.

## Per-focus-area verification (all first-hand, not relayed from the agent reviews)

**1. Timeout overflow panic (B1) — fixed at root and tested.** `crates/sifr_runtime/src/timeouts.rs` rejects non-finite/non-positive values and caps at 86,400s — five orders of magnitude below `Duration::from_secs_f64` overflow — before conversion. All three modules route through it (`net.rs:93`, `tls.rs:88`, `http.rs:60`); the three bespoke validators are deleted, and a crate-wide grep confirms this is the **only** `from_secs_f64`/`Duration` construction in `sifr_runtime` production code (every remaining `unwrap` sits inside `#[cfg(test)]` modules starting at `tls.rs:557`/`http.rs:597`). Per-module error-message prefixes are preserved via the label parameter. I ran: bare `cargo test -p sifr_runtime` → 29 passed (includes the NaN/0/1e20 unit tests, runnable without features thanks to the `lib.rs` cfg), `--features http` → 36 passed, and the e2e fixture `network_http_m1_tcp_errors.sifr` with its new `timeout=1e20` assertions → exit 0, no panic.

**2. Error taxonomy (B2) — amendment explicitly recorded.** The contract's Typed-Errors section now lists exactly the 8 shipped flat classes with a dated 2026-06-12 amendment naming the unshipped variants (`…substrate.md:679-695`); the ledger carries a matching "Error taxonomy amendment — accepted for remediation" row. The inventory taxonomy table is rewritten to flat classes, and a `grep -rn "Error::"` across all phase artifacts returns only the two amendment texts that intentionally name the rejected variants. Notably, the `inventory.json` residual (`ProtocolError::UnsupportedExtensionFrame`) that agent pass-2 flagged as out-of-scope has been swept in this branch too — the inventory pair is now fully clean.

**3. Dependency evidence (B3) — regenerated and drift-proofed.** The snapshot JSON is regenerated (status `closed-audited`, source pointing at the enforcing test), and the new test `network_http_snapshot_json_matches_generated_dependency_output` compares each snapshot's `production_dependencies` against live `generated_cargo_dependencies()` output plus literal `required_features`/`must_not_include`/`status` assertions — I checked the `normalize_runtime_path` helper and confirmed it cannot mask a feature-set drift (a changed feature list falls through to the bare-path branch and mismatches). `cargo test -p sifr_stdlib` → 88 passed including the 9 snapshot tests. The audit doc is updated to closed/audited, the tokio row records provider-baseline `process`/`signal` inheritance (consistent with the contract's amended Ring 2 row and rejection table), the `http` row records spec 1.4.1 vs lockfile 1.4.2 — which I confirmed against `Cargo.lock` — the cookie row now matches the shipped Sifr-owned parser, and the rustls-platform-verifier transitives (pass-1 N6) are acknowledged.

**4. Validation gate (B4) — structurally closed.** `run_crate_tests` now runs `cargo test -p sifr_stdlib`, `cargo test -p sifr_runtime`, and `cargo test -p sifr_runtime --features http` (`scripts/run_all_tests.sh:370-377`), placed before the smoke/full mode branch and invoked unconditionally at line 472 — so both create-pr and merge profiles get them. All three commands pass locally. Recording a gate run in the ledger remains for merge (F1).

**5. Phase status (B5) — consistent.** Contract status is no longer `draft`; greps for `Status: draft`, `m4-implemented`, and `active audit` across all phase artifacts return nothing. Roadmap, ledger, and inventory JSON all agree on completed/audited with the in-flight remediation honestly flagged.

**6. Handoff/traceability (B6) — statement exists where cited.** `docs/network_http.md:45` and `internal_docs/network_http_architecture.md:35` now both carry the explicit single-runtime-worker-per-process boundary with the serving-scale deferral, the Phase 41 doc records the deferral, and the M5 traceability row was reworded to claim exactly what those documents now contain.

## Elegance assessment

This is the right shape of remediation, not patchwork: one shared validated conversion point instead of three copies (the class of bug can't recur in a fourth module), a test that makes snapshot drift structurally impossible rather than a one-time regeneration, and dated amendments that record the descope decision instead of quietly rewording evidence. The working tree contains exactly the remediation set — nothing unrelated crept in.

**PASS.** Remaining work is merge mechanics only (F1): run and record the authoritative gates, flip the ledger rows, drop the status trailing clause, and persist this review into the empty pass-2 artifact.
