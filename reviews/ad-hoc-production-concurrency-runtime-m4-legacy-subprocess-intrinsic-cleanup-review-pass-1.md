## M4 Legacy Subprocess Intrinsic Cleanup — Review Pass 1

**In-scope code changes:**

- `crates/sifr_codegen/src/intrinsics/registry.rs:27,595-597` — removed `mod subprocess;` plus the three `subprocess_*` dispatch arms. Clean and surgical.
- `crates/sifr_codegen/src/intrinsics/registry/subprocess.rs` — file deleted in its entirety. No dangling re-exports.
- `crates/sifr_stdlib/src/sys_fs.rs:93-122` — removed `subprocess_run`, `subprocess_run_with_input`, and `subprocess_run_structured` `FunctionType` registrations from `intrinsic_sys()`. No collateral changes to sibling sys signatures.
- `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:4-15` — added `legacy_subprocess_intrinsics_are_not_lowered` negative guard; positive `lowers_subprocess_intrinsics_via_registry` test removed without orphans.
- `crates/sifr_stdlib/src/lib.rs:393-404` — added `legacy_subprocess_intrinsics_are_not_registered` negative guard against `_sifr.sys`.

**Remaining-references sweep:** `git grep -E "_sifr\.sys\.subprocess_run|subprocess_run_with_input|subprocess_run_structured"` outside guards, third_party, and `reviews/` returns no matches. The two matches in `reviews/ad-hoc-production-concurrency-runtime-m4-sync-process-review-pass-1.md` are historical archive notes and are allowed by scope. No live consumer remains.

**Public diagnostics intact:**
- `crates/sifr_stdlib/src/lib.rs:175,210-214` — `unsupported_legacy_stdlib_module("sifr.subprocess")` still returns `LegacyStdlibModule { suggested_module: "sifr.process", ... }`.
- `crates/sifr_stdlib/src/lib.rs:234` — `cpython_stdlib_reserved_suggestion("subprocess") -> "sifr.process"`.
- `crates/sifr/tests/e2e/fail/legacy_sifr_subprocess_removed.sifr` and `async_popen_unsupported.sifr` still expect typed errors for `from sifr.subprocess import …`. No silent adapter regression.

**Production process behavior:** none of the `process_*` dispatch arms in `registry.rs` were touched. User-supplied evidence shows `process_sync_output_text` and `process_signal_status` e2e tests PASS and the full create-pr profile passes (`96 passed`, `0 failed`).

**Documentation honesty:**
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md:51` — the obsolete "delete legacy `_sifr.sys.subprocess_*` paths" follow-up bullet is removed exactly because it is now done.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:691-708` — implementation record and validation ledger are accurate, including honest advisories about warm wall-time and cache-hit-rate misses.

**Non-blocking observations (not gating this slice):**
- Working tree contains unstaged modifications to `issues/ad-hoc-production-network-http-platform-substrate-execution.md` and `issues/ad-hoc-production-network-http-platform-substrate.md`, plus untracked `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-1.md` and `…-pass-2.md`. These are outside the M4 subprocess cleanup scope and should not be staged into this PR's commits.
- `reviews/ad-hoc-production-concurrency-runtime-m4-legacy-subprocess-intrinsic-cleanup-review-pass-1.md` is a 0-byte placeholder; consider filling it with the review record or removing the empty file before opening the PR.

The in-scope cleanup is correct, narrowly scoped, and preserves all required public behavior and diagnostics.

RESULT: PASS
