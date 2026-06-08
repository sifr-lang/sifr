# M5 resource cleanup helper diagnostics merge-ledger review — Pass 2

## Verdict: **FAIL**

The required pass-1 correction is **not applied**: PR #2423's merge-ledger bullet at `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:733` still asserts metrics that contradict the on-disk report it cites. Pass-1's blocking issue is unchanged.

## Findings

### PASS items

- **PR #2423 metadata**: `gh pr view 2423` returns `number=2423`, `title="Close M5 cleanup helpers as diagnostics"`, `state=MERGED`, `mergedAt=2026-06-08T19:00:13Z`, `headRefName=codex/concurrency-runtime-m5-resource-cleanup-next`, `baseRefName=main`, `mergeCommit.oid=efaf92ed58bc85e92a7f4f6aef2ed4488ae59e47`, `url=https://github.com/sifr-lang/sifr/pull/2423` — all consistent with the ledger row's `Merged as PR #2423 (efaf92ed58bc85e92a7f4f6aef2ed4488ae59e47) on 2026-06-08`.
- **PR URL replacement** at line 453 (`pending PR.` → `https://github.com/sifr-lang/sifr/pull/2423`) is correct and consistent with the milestone summary.
- **Scope wording**: the merge-ledger section is limited to "closing unsupported cleanup helpers as diagnostics" via the existing implementation/review rows; no new claim of cleanup-stack runtime or owned-close protocol implementation has been introduced. No scope overclaim beyond the diagnostic-fixtures slice.
- **Review-artifact cross-reference**: `reviews/...m5-resource-cleanup-diagnostics-review-pass-1.md` (the implementation review) exists and is correctly cited at line 728; this packet does not retroactively rename or relocate it.

### Blocking issue — pass-1 correction not applied (line 733)

Pass-1's required correction was to "rewrite the line-733 merge-ledger bullet to reflect the real on-disk metrics (`339.84s`, `cache_hits=34/34`, single advisory `warm wall-time budget exceeded`, no `53%` warm-cache hit-rate advisory)".

Verifying the current file (`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:733`) against the on-disk evidence:

| Field | Ledger row (line 733) claims | On-disk `create-pr.latest.json` (mtime `2026-06-08 20:57`) |
| --- | --- | --- |
| warm wall-time | `1373.86s` | `time.real_seconds: 339.84` (corroborated by `create-pr.latest.time` → `339.84 real`) |
| advisories | "warm wall-time budget exceeded" **and** "warm-cache hit rate below advisory target (`53%`, target `>=90%`)" — dual advisory | `advisories: ["warm wall-time budget exceeded"]` — single advisory |
| e2e cache hits | `cache_hits=18/34` | `e2e.cache_hits: 34` against `e2e.group_count: 34` (100% / 34 of 34) |
| report signature | `293aaf3695dc42f8` | (signature field not present in the on-disk JSON; the implementation row at line 724 uses the same signature, so this is shared with the implementation snapshot) |

The wall-time, advisory wording, warm-cache percentage, and cache-hit ratio in the merge-ledger row remain **byte-identical to the values pass-1 flagged as copy-pasted from PR #2418's earlier merge-ledger entry**. The on-disk report still shows the same `339.84s` / single advisory / `34/34` values that match PR #2423's implementation validation row at line 724 — i.e., no post-merge rerun has overwritten the report, and no separate attached evidence supports the line-733 figures. The cited file (`target/validation_lane_reports/create-pr.latest.json`) contradicts the cited metrics.

### Scope-creep observation (non-blocking for this packet's purpose, but worth flagging)

The packet additionally rewrites PR #2418's merge-ledger row (line 623) from the original `1373.86s` / dual advisory / `cache_hits=18/34` to `339.84s` / single advisory / `cache_hits=34/34`, and inserts new pass-3 (FAIL) and pass-4 (PASS) reviews of PR #2418's ledger. PR #2418 is already merged and previously had a passing pass-2 ledger review. The `reviews/...m5-signal-stream-ledger-review-pass-3.md` and `...pass-4.md` files exist on disk, so this is not an undocumented edit — but it represents a separate, retroactive correction stream for PR #2418, not part of the requested PR #2423 ledger correction. It does not block the present verdict, but it does mean two merge-ledger entries (PR #2418 line 623 vs. PR #2423 line 733) now cite the **same** `create-pr.latest.json` file with **different** metrics, which is internally inconsistent: only one of them can reflect the file's current contents, and the file currently shows `339.84s` (matching PR #2418's corrected row, not PR #2423's claimed row).

## Required correction before next review

Rewrite the line-733 merge-ledger bullet so it reflects the on-disk `target/validation_lane_reports/create-pr.latest.json`:

- warm wall-time: `339.84s`
- advisory: single — `warm wall-time budget exceeded` (drop the `53%` / `>=90%` warm-cache advisory wording)
- e2e cache hits: `cache_hits=34/34`
- keep `report_signature=293aaf3695dc42f8`, suite totals (`120 passed`, `0 failed`), and lane-step coverage as-is

Alternatively, if a distinct post-merge create-pr rerun was actually performed and produced the `1373.86s` / dual-advisory / `18/34` values, attach the corresponding report (e.g., a dated copy under `target/validation_lane_reports/`) and update the ledger row to cite that report rather than `create-pr.latest.json`. As currently written, line 733 cites a file whose contents contradict it.

## Honest pass-1 failure record

Pass-1 correctly identified the metric-overclaim/copy-paste from PR #2418's ledger and required a rewrite of line 733 to the on-disk values. That blocking issue is preserved verbatim by this pass-2 review and remains unaddressed by the packet under review. No scope creep beyond the cleanup-helper diagnostics slice has been introduced into the PR #2423 ledger itself; the blocking concern is solely the validation metrics/advisory wording mismatch with the cited report.
