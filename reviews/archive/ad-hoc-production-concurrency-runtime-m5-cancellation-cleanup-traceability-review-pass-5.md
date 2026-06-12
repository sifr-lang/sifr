## FAIL

**Blocker: M6 IPC stream-helper merge ledger entry from main is NOT preserved.**

`git merge-base origin/main HEAD` = `019fd05a5` (the "Add M6 IPC stream helpers" code commit), but origin/main is now at `a060d87f9` "Record M6 IPC stream helpers merge ledger" — one commit ahead. The `origin/main..HEAD` diff therefore deletes that ledger commit's content:

- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` — deletes the entire `M6 typed IPC stream read/write merge ledger:` block (PR #2447, merge commit `019fd05a55…`, merged `2026-06-09T00:22:23Z`). See diff `@@ -1058,14 +1080,6 @@` (8 lines removed: ledger header + PR / merge commit / merged-at / scope / merge-ledger-validation bullets).
- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-transport-ledger-review-pass-1.md` — deleted in full (15 lines).

Branch must be rebased onto `origin/main` (`a060d87f9`) before this addendum can land. This was the exact regression flagged by pass-4 against an earlier base; it has reappeared because main moved forward after the rebase to `019fd05a5`.

**Items that pass (no blockers):**

- M5 remains closed: `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:5` still `Status: Closed`; `issues/...execution.md:459` still `- M5: complete.`; addendum recorded as a discrete `pending PR` line at `:458`.
- `cancellation_cleanup_runs` honestly recorded as merge/create-pr evidence: added at `verification/validation_lanes/merge_e2e_manifest.json:71`; create-pr/merge rows in the traceability doc updated; fixture claim correctly bounded to language-level cleanup ordering at `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:18` and `verification/platform/supported_host_matrix.md:37`.
- ExitStack/AsyncExitStack/closing/aclosing remain unsupported diagnostics: the four fail fixtures untouched; traceability row at `concurrency_runtime_m5_shutdown_traceability.md:20` only sharpens the future-support wording without weakening the diagnostic.
- Issue validation metrics match the specified latest runs (`issues/...execution.md` lines 27-29 of the diff): file-size `2253`, merge `136 passed / 0 failed`, `cache_hits=41/41`, `report_signature=dc77a4a9bb841f30`; create-pr `124 passed / 0 failed`, `cache_hits=37/37`, `report_signature=530c89bb7012eeb0`, wall `143.52s`, slowest `platform_golden` `39543ms` — all match verbatim.
