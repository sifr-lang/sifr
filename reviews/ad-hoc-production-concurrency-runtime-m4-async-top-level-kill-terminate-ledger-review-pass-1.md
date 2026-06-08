## Review: M4 top-level async kill/terminate merge-ledger docs branch — **PASS**

### Blocking findings
None.

### Verification performed
- Merge SHA + date confirmed via `git log`: `a064cf3e5074ab81a61da455233369bafe340dc1`, dated `2026-06-08`. ✓
- PR #2378 link reused consistently in both the issue ledger (line 432) and traceability Status sentence. ✓
- All validation numbers in the new "Merge-ledger validation" bullet (line 1093) match the values provided: `wall_time=203.29s`, `pass=5 skip=2`, `106 passed/0 failed`, `cache_hits=27/27`, `report_signature=dc7d767be4dbcf7c`. ✓
- Wording for the warm wall-time advisory matches the established convention from the prior `184.34s` line (1087) — same phrasing, just the new number. ✓
- Referenced `reviews/...-review-pass-1.md` (without `-ledger`) exists; it was merged in commit `eb2ba70f9` as part of PR #2378. ✓
- No overclaiming: the traceability doc Status correctly stays `In progress;` and the M4 surface row still hedges with "Public async owned pipes, cancellation-safe observation, scoped process supervision, and shell async APIs remain later M4 work." The issues file likewise keeps `M4: in progress.` at line 433. ✓

### Non-blocking notes
- Changes are uncommitted working-tree edits (no commits ahead of `origin/main`). Branch needs a commit before a PR can be opened — flagging only because the scope question explicitly asks about PR-readiness.
- Untracked `reviews/...-ledger-review-pass-1.md` is present in the working tree but out of scope for the two-file ledger change; include or exclude deliberately when staging the commit.
- Minor: the traceability Status sentence is now ~20 PRs long. Not blocking, but at some point a one-paragraph rollup with the per-PR list moved to a sub-section would scale better. Defer until M4 closeout, not this PR.

Ledger branch is ready to commit and open as a docs-only PR.
