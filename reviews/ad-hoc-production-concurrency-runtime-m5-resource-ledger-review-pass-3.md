I've re-audited the docs-only diff after the create-pr rerun and the new review-loop bullets, against the report/log on disk, the merge commit, the prior pass-1 blocker, and the M5 merge-ledger convention.

---

**PASS**

**1. Final validation metrics in the ledger match the report/log on disk**

Verified the updated metrics on line 629 against `target/validation_lane_reports/create-pr.latest.json` and `.latest.log`:

- `time.real_seconds=132.53` ✓ → ledger `132.53s`.
- `advisories=["warm wall-time budget exceeded"]` (single entry after the rerun) ✓ → ledger now uses singular `advisory:` and quotes only that one advisory. The pass-2 report had a second advisory (`warm-cache hit rate below advisory target`) that no longer fires; the ledger correctly drops it.
- `budget.warm_wall_time_target_minutes=2` ✓ → ledger `warm target <=2m`.
- `e2e.cache_hits=31`, `e2e.group_count=32` ✓ → ledger `cache_hits=31/32`. `observations.cache_hit_rate=0.96875` is consistent with 31/32 and explains why the warm-cache advisory cleared on the rerun (prior pass had 28/32 = 0.875).
- `[sifr-e2e] report_signature=6dd646fdf4fc2cb4` ✓ (`.log:1454`).
- `[platform-golden] summary pass=6 skip=1` ✓ (`.log:1427`).
- `116 pass tests completed (116 passed, 0 failed)` ✓ (`.log:1455`).
- Lane-step enumeration in the ledger (guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden, create-pr e2e pass suite) is a representative subset of `lane_steps` — all 14 lane steps in the report are `status: pass`, matching the precedent set by PR #2407's ledger entry (line 601). No fabricated steps; no steps claimed pass that did not pass.
- Merge commit verified: `git show 5001b0985` → "Merge pull request #2409 from sifr-lang/codex/concurrency-runtime-m5-resource-foundation", `Date: Mon Jun 8 17:13:59 2026 +0200`. PR number, full SHA `5001b0985838a240a7adeb01adf6fa343970cb36`, and 2026-06-08 date all match.

**2. Pass-1 blocker remains fixed**

`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:447` still reads `- M5 resource nullcontext foundation: in progress.`. The "Implementation PRs" status block remains internally consistent with the sibling M5 rows at lines 445 (`M5 signal value-model foundation: in progress.`) and 446 (`M5 warnings global-filter rejection: in progress.`), preserving the convention set by PR #2406 (signal ledger) and PR #2408 (warnings ledger). `git diff` confirms the only tracked change in this branch remains the new merge-ledger subsection plus the two newly appended review-loop bullets — line 447 itself is not touched in the diff.

**3. Review-loop bullets accurately describe pass 1 FAIL and pass 2 PASS**

- Line 630 (pass-1 bullet): correctly records `FAIL`, names the actual cause ("top-level M5 status block had been promoted from `in progress.` to the PR URL, inconsistent with the accepted M5 foundation-slice ledger convention"), and notes the fix ("The status line was restored before the second review pass."). Matches `reviews/...m5-resource-ledger-review-pass-1.md:5-15`.
- Line 631 (pass-2 bullet): correctly records `PASS`, summarises what was verified ("status-block convention fix, PR #2409 merge commit/date, validation metrics and advisories, lane-step coverage, and no overclaim beyond the no-value `nullcontext()` slice"). Matches `reviews/...m5-resource-ledger-review-pass-2.md:7-25`. The bullet wisely paraphrases "validation metrics and advisories" rather than quoting the pass-2-era numbers (`221.95s`, `28/32`, two advisories) that the rerun has since superseded — so the ledger remains accurate against the current on-disk report even though pass-2's review artifact was written against the prior run.

**4. No remaining blockers**

- Branch scope is still docs-only: one tracked file modified (`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`), three untracked review artifacts (`reviews/...m5-resource-ledger-review-pass-{1,2,3}.md`) to stage alongside the ledger PR.
- No overclaim: the ledger subsection records only the merge fact and validation evidence; it does not claim `ResourceError`, `ExitStack`, `AsyncExitStack`, `closing`/`aclosing`, value-carrying generic nullcontext, cancellation cleanup reports, or async cleanup. The preceding implementation entry (lines 604-609) and the status row staying at `in progress.` both reinforce that this PR closes only the no-value nullcontext slice.
- No M4-style URL promotion anywhere in the M5 block.
- The pass-2 review's non-blocking note about appending review-loop bullets under the merge-ledger subsection has been actioned (lines 630-631), matching the warnings-ledger precedent at line 602.

**No remaining blockers for opening/merging this docs-only ledger PR.**
