# Review: Optional/Any Root-Cause Closure Phase (Pass 2)

Reviewer: Claude
Date: 2026-04-06
Phase document: `issues/ad-hoc-phase-optional-any-root-cause-closure-2026-04-06.md`
Prior review: `reviews/optional-any-root-cause-phase-review-pass1b.md`

---

## Verdict: Ready

All pass1b recommendations have been properly addressed. No remaining blockers. The phase plan is implementation-ready.

---

## Pass1b Recommendation Verification

| # | Recommendation | Status | Evidence |
|---|---|---|---|
| Edit 1 | Update `0787` rationale to note ON-1 index variant | Applied | JSON rationale now reads: "Optional index value (int | None) not narrowed before container indexing; note: container itself is concrete, issue is in index position (ON-1 variant)" |
| Edit 2 | Update `0909` rationale to flag secondary ON root cause | Applied | JSON rationale now reads: "...secondary: index type is int | None (potential ON root cause may surface after AU-2 closure)" |
| Edit 3 | Add suggested execution order to execution ledger | Applied | Execution ledger contains tiered ordering: W1/W2/A1 parallel, then W3, W4, W5, A2 in dependency order |
| Edit 4 | Add 53 non-targeted fixture regression clause to exit gates | Applied | Phase doc Full-corpus gate now includes: "the 53 non-targeted fixtures across all other taxonomy categories must not change status" |

All four exact edits from pass1b are confirmed applied. CSV and JSON remain mutually consistent after the rationale updates.

---

## Remaining Non-Blocking Refinements

### 1. W5 compiler/adaptation split criterion (carried from pass1b)

Pass1b Improvement 1 recommended adding a formal decision rule for determining which ON-4/ON-5 cases are compiler-closeable vs adaptation-requiring. This was not applied, but was explicitly marked non-blocking.

**Current status**: The W5 acceptance criteria still reads "compiler-owned part of ON-4 and ON-5 removed; remaining residuals are explicit adaptation candidates only" without a formal split rule.

**Recommendation**: This can be deferred to W5 implementation time. The implementer will need to define this criterion when evaluating ON-4/ON-5 cases against the CFG, and can document it then. Not a gate for starting implementation.

---

## Data Consistency Re-Check

| Check | Result |
|---|---|
| Phase doc fixture counts (30 ON + 28 AU = 58) | Pass |
| Resolution mode totals (51 + 6 + 1 = 58) | Pass |
| JSON `root_cause_counts` sum to 58 | Pass (15+6+3+4+2+4+16+5+1+1+1 = 58) |
| CSV row count (excluding header) = 58 | Pass |
| JSON and CSV mutually consistent after edits | Pass |
| Execution ledger workstreams match phase doc (W1-W5, A1-A2) | Pass |
| Execution order respects documented dependencies | Pass |
| Exit gates cover focused rerun, full-corpus, and policy | Pass |

---

## Remaining Blockers

None.

---

## Summary

The phase plan is ready to implement. All four pass1b edits were properly applied. The only outstanding item is a non-blocking refinement to formalize the W5 compiler/adaptation split criterion, which can be resolved during W5 implementation without affecting the start of Tier 1 workstreams (W1, W2, A1).
