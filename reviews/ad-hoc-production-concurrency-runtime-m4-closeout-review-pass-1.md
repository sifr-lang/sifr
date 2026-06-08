RESULT: PASS

The docs-only closeout/classification diff correctly closes `milestone_concurrency_runtime_4` after PRs #2331 through #2400. All seven remediation items called out by the prior pre-closeout audit (`reviews/ad-hoc-production-concurrency-runtime-m4-closeout-audit-pass-1.md`) are addressed faithfully, no surface contradicts another, and no out-of-scope files are touched.

Scope verification (only docs / closeout artifacts modified)

- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` — checklist tick, PR-list "M4 closeout: in progress" entry, and a paired implementation/validation block.
- `verification/platform/supported_host_matrix.md` — single umbrella row (line 18) flipped to `supported | supported | host-limited`.
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md` — `Status:` line, `sifr.process.Status` row, sync `spawn`/`wait`/`Child.wait` row, sync `kill`/`Child.kill` row, CPython async-subprocess mapping disposition, two validation-lane rows (added `process_scoped_parent_cancel`), and the `Follow-up Boundaries` section.
- `reviews/ad-hoc-production-concurrency-runtime-m4-closeout-audit-pass-1.md` — the pre-closeout audit PASS artifact (this review's input).
- `reviews/ad-hoc-production-concurrency-runtime-m4-closeout-review-pass-1.md` — this review.

No code, no snapshots, no manifests, no fixtures — strictly docs/closeout artifacts.

Audit-item closure (vs. `reviews/ad-hoc-production-concurrency-runtime-m4-closeout-audit-pass-1.md`)

1. Traceability `Status:` opening (`concurrency_runtime_m4_process_traceability.md:5`): flipped from "In progress; ... remaining M4 subprocess lifecycle gaps are pending" to "Closed; ... No M4 subprocess lifecycle gaps remain on supported macOS/Linux hosts; non-Unix status semantics and Windows fixture coverage are intentionally host-limited and tracked in the supported-host matrix." ✓
2. `sifr.process.Status` row (`concurrency_runtime_m4_process_traceability.md:12`): "Cancellation status remains open for later lifecycle waves" removed; replaced with explicit scoped-parent-cancellation behavior plus an explicit host-limited note for non-Unix signal-equivalent status. `process_scoped_parent_cancel` added to the evidence column. ✓
3. Sync `spawn`/`wait`/`Child.wait` row (`concurrency_runtime_m4_process_traceability.md:23`): "parent cancellation evidence and non-Unix status semantics remain later M4 work" replaced with "Parent cancellation belongs to the scoped `ProcessHandle` path and is covered by `process_scoped_parent_cancel`; non-Unix status semantics remain host-limited in the supported-host matrix." ✓
4. Sync `kill`/`Child.kill` row (`concurrency_runtime_m4_process_traceability.md:24`): "Structured cancellation and non-Unix signal-status evidence remain later M4 work" replaced with "Structured cancellation is delivered through scoped process supervision and timeout-created process-group TERM-to-KILL escalation; non-Unix signal-status evidence remains intentionally host-limited in the supported-host matrix." ✓
5. `Follow-up Boundaries` (`concurrency_runtime_m4_process_traceability.md:47-54`): re-titled "Post-M4 host limits and future follow-ups"; first bullet explicitly classifies non-Unix signal status, non-Unix `terminate` semantics, and Windows subprocess fixtures as "intentionally host-limited in the supported-host matrix, not pending M4 implementation gaps." Optional text-error-handler bullet, sync `Child` drop honesty bullet, and stdlib re-export workload-metadata bullet retained as legitimate post-M4 future work. ✓
6. Supported-host matrix umbrella row (`supported_host_matrix.md:18`): flipped from `in-progress | in-progress | host-limited` to `supported | supported | host-limited`; "Termination escalation and non-Unix status semantics remain before this umbrella row can be marked supported" replaced with a pointer to the dedicated rows below plus an explicit "Non-Unix signal-equivalent status, terminate semantics, and Windows fixtures remain host-limited in those dedicated rows." Every dedicated subprocess row below (lines 19-32) already substantiates the macOS/Linux `supported` claim and carries the Windows host-limited classification. ✓
7. Execution ledger (`issues/...execution.md:35`, `:442`, `:526-536`): `milestone_concurrency_runtime_4` checkbox ticked; PR-list `M4 closeout: in progress.` entry inserted in PR-number order before `M5: pending.`; paired "M4 closeout classification implementation" and "M4 closeout classification targeted local validation" blocks recorded with the `git diff --check` + `scripts/run_all_tests.sh --profile create-pr` evidence (wall_time `145.47s`, e2e `114 passed/0 failed`, `cache_hits=28/30`, `report_signature=b11e218d104a7820`) consistent with the prior PR #2400 merge-ledger advisory pattern. M5 remains the next pending entry both in the checklist (line 36) and the PR list (line 443). ✓

Stale "remaining/pending M4" wording sweep on active surfaces

- `git grep -i "remaining M4|later M4|pending M4|M4.*pending"` across `verification/**/*.md` and the active section of the execution ledger returns only the audit artifact, the new "not pending M4 implementation gaps" remediation phrasing in `concurrency_runtime_m4_process_traceability.md:51`, and historical `reviews/*` summaries (allowed). No stale active-surface claims survive.
- `supported_host_matrix.md` has no `in-progress` token anywhere in the subprocess rows.

Cross-surface consistency

- macOS/Linux umbrella `supported` is consistent with every dedicated subprocess row at lines 19-32: child handle cleanup, async run/output, async output timeout, async spawn/wait, async owned pipes, scoped supervision, async kill/terminate, sync terminate, signal status, strict text, shell exec — all `supported | supported | host-limited` (except line 22 child handle cleanup which is `supported | supported | supported`, an additional cross-host strength that does not contradict the umbrella).
- Windows `host-limited` is consistent across umbrella + every dedicated row; no row silently promotes Windows to `supported`. Non-Unix terminate / signal-equivalent status are honestly host-limited everywhere they appear.
- `process_scoped_parent_cancel` propagates consistently: traceability `sifr.process.Status` evidence column, scoped `ProcessHandle` row, CPython async-subprocess mapping, both `Create PR` and `Merge` validation lane rows, and the `supported_host_matrix.md:27` scoped subprocess supervision row. The PR #2400 merge ledger entry at `issues/...execution.md:521-524` is preserved untouched.

Validation evidence

- `git diff --check` PASS (recorded).
- `scripts/run_all_tests.sh --profile create-pr` PASS; `wall_time=145.47s`; advisory: warm wall-time budget exceeded (consistent with the prior PR #2400 merge ledger pattern); platform golden `pass=6 skip=1`; create-pr e2e `114 passed, 0 failed`, `cache_hits=28/30`, `report_signature=b11e218d104a7820`. No regressions.

Non-blocking observations (do not block the closeout PR)

- `concurrency_runtime_m4_process_traceability.md:20`, `:22`, `:25` retain the pre-existing "until host-specific X is designed and fixture-backed" / "pending host-specific process-group design" wording in technical-notes columns for async `terminate`, sync `run_timeout`/`output_timeout`, and sync `terminate`. These describe current host-limited implementation behavior (not M4 lifecycle status), are consistent with the dedicated matrix rows' `host-limited` Windows classification, and the new `Follow-up Boundaries` section already states the permanent host-limited classification at line 51. Wording polish only — not a closeout blocker.
- `supported_host_matrix.md:18` umbrella row mentions "timeout TERM-to-KILL process-group cleanup" as a supported behavior backed by "dedicated rows below". The matrix has line 24 (async output timeout) but no dedicated row titled "subprocess timeout TERM-to-KILL". The supporting fixtures (`process_timeout_group_cleanup`, `process_async_timeout_group_cleanup`) are documented in the traceability validation lanes, so the claim is substantiated — just not by a matrix row with that exact title. Not a closeout blocker.
- `issues/...execution.md:442` "M4 closeout: in progress." follows the established sibling pattern and will be updated to the merged PR link + a "M4: complete." summary line (mirroring `:414-415` for M3) once this PR merges; the closeout merge ledger / review-loop entry blocks will be appended at that point. Standard post-merge bookkeeping, not a closeout-PR blocker.
- The closeout implementation block at `:526-536` does not yet cross-link to the audit (`reviews/ad-hoc-production-concurrency-runtime-m4-closeout-audit-pass-1.md`) or this review. The "M4 closeout classification review loop" entry will normally cite this review file once the review is recorded. Cosmetic — does not affect the PR's correctness.

Must remain host-limited / future (no M4 closure obligation, confirmed)

- Non-Unix (Windows) signal-equivalent status mapping.
- Non-Unix `terminate` semantics (currently returns typed unsupported `ProcessError`).
- Windows deterministic fixtures for every dedicated subprocess row.
- Optional subprocess text decoding error-handler arguments beyond strict.
- Sync `Child` drop: intentionally abandons observation (no kill/wait/descendant supervision claim).
- Stdlib re-export workload metadata mirroring (if/when a future stdlib re-exports a workload-annotated callable).

Conclusion

The closeout diff is honest, scope-clean, internally consistent, and validated. M4 is correctly marked complete; M5 remains the next pending entry both in the checklist and PR list. Open the docs-only closeout PR.
