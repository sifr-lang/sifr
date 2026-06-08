## Verdict: FAIL

**Verified OK:**
- **PR #2418 metadata** matches GitHub exactly: state `MERGED`, mergeCommit `abdd8674b9a51dc88260782283b6f47c4c7791ff`, mergedAt `2026-06-08T17:40:16Z`, title "Add M5 signal stream lowering". Ledger date `2026-06-08` and SHA match.
- **Merge SHA on main** — `abdd8674b Add M5 signal stream lowering (#2418)` is present in `git log`.
- **Corrected metrics are internally consistent**: `cache_hits=18/34 ≈ 52.94% ≈ 53%`; both advisories (`warm wall-time budget exceeded`, `warm-cache hit rate below advisory target`) match the create-pr report excerpt in the packet; `120 passed, 0 failed` and `report_signature=293aaf3695dc42f8` are consistent with the same fixture set as prior runs.
- **Signal-stream entry scope**: the merge-ledger entry stays restricted to "stream shape and lowering" — no claim of deterministic delivery, non-Unix SIGTERM, or constants beyond SIGINT/SIGTERM.

**Blocking discrepancies:**

1. **Review reference no longer matches the metrics it cites.** Pass-2 (`reviews/...-ledger-review-pass-2.md`, committed in `d28bef225`) explicitly verified `cache_hits=34/34`, `warm 571.40s`, single advisory:
   > "merge-ledger re-run (`120 passed, 0 failed, cache_hits=34/34, sig=293aaf3695dc42f8`, warm `571.40s`)"

   The corrected ledger now reports `cache_hits=18/34`, `warm 1373.86s`, two advisories — yet still asserts pass-2 "verified final validation metrics and advisory wording". That claim is no longer true. `reviews/...-ledger-review-pass-3.md` exists but is **0 bytes** (untracked), so no committed review attests to the corrected metrics.

2. **Metric correction was made silently in an unrelated commit.** Commit `4b81bcb43` ("Close M5 cleanup helpers as diagnostics") amended the already-merged-ledger advisory list and numbers (571.40s/34/34/single advisory → 1373.86s/18/34/two advisories) while ostensibly adding cleanup-diagnostics docs. The commit subject does not signal that it also rewrites historical merge-ledger validation numbers, and no review pass corroborates the rewrite.

The merge-ledger entry's review reference must either be re-pointed to a populated pass-3 verifying the corrected metrics, or pass-2's wording in the ledger updated to reflect that it verified an earlier (now-superseded) snapshot.
