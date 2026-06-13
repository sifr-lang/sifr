# M5 resource cleanup helper diagnostics merge-ledger review — Pass 3

## Verdict: **PASS**

The pass-1 / pass-2 blocker (line 733 validation metrics/advisory wording contradicting the cited report) is fully resolved. Remaining checks pass.

## Findings

### Pass-1/pass-2 blocker — FIXED

The merge-ledger row at `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:733` now reads `warm wall-time budget exceeded (339.84s, …)`, single advisory, `cache_hits=34/34`, `120 passed / 0 failed`, `report_signature=293aaf3695dc42f8`. Cross-checked against on-disk `target/validation_lane_reports/create-pr.latest.json`:

| Field | Ledger row (line 733) | On-disk report |
| --- | --- | --- |
| `time.real_seconds` | `339.84s` | `339.84` |
| `advisories` | single — `warm wall-time budget exceeded` | `["warm wall-time budget exceeded"]` |
| `e2e.cache_hits` / `group_count` | `34/34` | `34 / 34` |
| `report_signature` | `293aaf3695dc42f8` | field absent in JSON, but matches the implementation row at line 724 sharing the same run |

The byte-identical `1373.86s` / dual-advisory / `cache_hits=18/34` copy-paste from PR #2418's earlier ledger is gone.

### PR #2423 metadata — ACCURATE

`gh pr view 2423` returns `number=2423`, `state=MERGED`, `mergedAt=2026-06-08T19:00:13Z`, `mergeCommit.oid=efaf92ed58bc85e92a7f4f6aef2ed4488ae59e47`, `headRefName=codex/concurrency-runtime-m5-resource-cleanup-next`, `baseRefName=main`. Ledger row at line 732 (`PR #2423 (efaf92ed58bc85e92a7f4f6aef2ed4488ae59e47) on 2026-06-08`) and milestone-summary update at line 453 are consistent with this and with `git log` (`efaf92ed5 Merge pull request #2423 …`).

### Review history — HONEST

- The implementation review reference at line 728 (`...m5-resource-cleanup-diagnostics-review-pass-1.md`, `PASS`) exists on disk and was introduced with the PR itself.
- The packet also retroactively rewrites PR #2418's merge-ledger row (line 623) to match the on-disk `339.84s` / single-advisory / `34/34` numbers and amends pass-2's description to "the earlier warm-cache validation snapshot" (so the original pass-2 PASS no longer overclaims the final metrics), then records `...m5-signal-stream-ledger-review-pass-3.md` (`FAIL`) and `...pass-4.md` (`PASS`) — both files exist on disk. This is a faithful honest-history correction stream for PR #2418's ledger overclaim and resolves the cross-row inconsistency the pass-2 review flagged.
- The PR #2423 merge-ledger entry itself does not yet cite ledger-review pass-1 or pass-2 (both FAIL, on disk at `reviews/ad-hoc-production-concurrency-runtime-m5-resource-cleanup-diagnostics-ledger-review-pass-{1,2}.md`). The entry does **not make any false review-history claim** — it simply omits the citations. When this pass-3 result is recorded, pass-1 (FAIL), pass-2 (FAIL), and pass-3 (PASS) should be cited together to match the convention used for the signal-stream ledger. Treating this as a follow-up for the next ledger-update commit rather than a blocker, since no current text overclaims.

### Scope — NO OVERCLAIM

Lines 730–733 stay strictly within "closing unsupported cleanup helpers as diagnostics" with preserved `nullcontext(...)` support. No claim of cleanup-stack runtime, owned-close protocol, or async-cleanup implementation has been introduced. The retroactive PR #2418 ledger edits are limited to validation metrics and pass-2/3/4 review wording; they do not expand PR #2418's claimed scope beyond stream shape and lowering.

## Follow-up (non-blocking)

When recording this pass-3 result, add ledger-review citations to the line-730 section in the same shape as the signal-stream ledger (lines 624–626): pass-1 `FAIL` (metric/advisory copy-paste), pass-2 `FAIL` (correction not yet applied), pass-3 `PASS`. This keeps the cleanup-diagnostics ledger's review history symmetric with the signal-stream ledger's honest-failure record.
