PASS.

Verified end-to-end:

- **Fixtures honestly emit SIFR-NAME-0004 at col 27** for `ExitStack`, `AsyncExitStack`, `closing`, `aclosing` — confirmed by direct `cargo run -- check` against each of `crates/sifr/tests/e2e/fail/resource_{exitstack,async_exitstack,closing,aclosing}_unsupported.sifr`.
- **Implementation matches the diagnostics**: `lib/sifr/resource.sifr` exposes only `NullContext` and `nullcontext` — the four unsupported helpers are genuinely absent (no fake stubs), so the missing-member diagnostic is the true compiler behavior, not a synthetic gate.
- **`nullcontext(...)` support preserved without overclaim**: `crates/sifr/tests/e2e/pass/resource_nullcontext_basic.sifr` still covers no-value and value-carrying generic shapes; docs scope nullcontext to the synchronous `with` protocol only.
- **Future blockers recorded** in `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:19`, `verification/stdlib/concurrency_runtime_substrate_inventory.{md:29,json:288}`, and `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:712` — all three documents explicitly cite typed cleanup-error aggregation for cleanup stacks and an owned-close protocol preserving mutating/fallible close behavior for `closing`/`aclosing`. The shutdown traceability follow-up boundary (`...shutdown_traceability.md:55`) is updated consistently.
- **`supported_host_matrix.md:37`** lists the four new fixtures in the deterministic cleanup scopes row and keeps cancellation cleanup ordering as remaining M5 work — no overclaim.
- **Traceability table** (`...shutdown_traceability.md:48`) adds the four fixtures to the fail-suite column.
- **Validation evidence reproduced**: e2e fail suite reports `451 fail tests completed` with harness `test result: ok. 1 passed; 0 failed`. The two pre-existing ICE diagnostic captures (`cfg.rs:300`) printed by the harness do not fail the suite, matching the ledger note. JSON validity, `git diff --check`, and `scripts/check_file_size_guardrails.py` all PASS.

No blocking issues.
