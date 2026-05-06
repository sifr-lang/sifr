# Review — INT-2B Transitive Const Re-export Tracker Update (pass 1)

Branch: `int-2b-transitive-reexport-tracker`
Reference: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), [reviews/integer-model-int-2b-transitive-reexport-doc-review-pass-1.md](integer-model-int-2b-transitive-reexport-doc-review-pass-1.md), [PR #1808](https://github.com/sifr-lang/sifr/pull/1808)
Reviewer scope: docs-only tracker update — verify the review-history row, checklist row, PR number, review-artifact path, and the trimmed remaining-follow-up wording. No code, doc-prose, or test changes are owed.

## Diff under review

`git status` shows a single dirty file:

```
M issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md
```

`git diff` is two hunks, both in `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`:

1. **Review History (line 414)** — one new `[x]` entry appended to the INT-2B run:

   ```
   + - [x] INT-2B transitive const re-export semantics documentation review satisfied: `reviews/integer-model-int-2b-transitive-reexport-doc-review-pass-1.md`.
   ```

2. **Implementation Checklist (lines 438-439)** — one new `[x]` bullet for PR #1808 inserted under INT-2B, and the prior carry-over `[ ]` bullet trimmed to drop the now-satisfied transitive-re-export follow-up:

   ```
   + - [x] Imported const-evaluable status is documented as local to the importing module, with no transitive const-value re-export unless the intermediate module defines its own public const-evaluable constant; review is satisfied and quick validation is passing: PR #1808.
   - - [ ] Carry remaining follow-ups from INT-2A/INT-2B reviews: decide reserved-name shadowing policy during `bigint` cleanup, clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable, and document or implement transitive re-export semantics for imported constants.
   + - [ ] Carry remaining follow-ups from INT-2A/INT-2B reviews: decide reserved-name shadowing policy during `bigint` cleanup and clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable.
   ```

No other files modified.

## Correctness checks

### Review-history row

- Path matches the on-disk artifact: `reviews/integer-model-int-2b-transitive-reexport-doc-review-pass-1.md` exists and ends with `VERDICT: SATISFIED` (line 56), so describing it as "review satisfied" is faithful.
- Naming convention is consistent with prior INT-2B history rows (`integer-model-int-2b-<slug>-review-pass-N.md`); the slug `transitive-reexport-doc` mirrors the merged branch name `int-2b-transitive-const-reexport-doc` in compact form.
- Placement at the end of the INT-2B history block (after the INT-0003 registry/e2e row at line 413) preserves chronological order — PR #1806 merged before PR #1808, so the new row appearing after it is correct.

### Implementation-checklist row

- The new bullet sits inside the INT-2B group between the INT-0003 e2e row (line 437, PR #1806) and the carry-over row (line 439). PR #1808 merged on 2026-05-06, after PR #1806, so chronological grouping holds.
- Wording matches the slice's actual scope. Per the review artifact (sections "Correctness vs. implementation" and "Closing the referenced follow-up"), PR #1808 added a single paragraph in `internal_docs/integer_model.md` documenting two facts: (a) imported const-evaluable status is local to the importing module, and (b) the only way to propagate a const value through a third module is for that module to redeclare its own public const-evaluable constant. The bullet captures both halves accurately and uses vocabulary already established in the doc (`const-evaluable`, `imported`, `public`).
- The closing fragment "review is satisfied and quick validation is passing: PR #1808." matches the boilerplate used by every other completed INT-2B bullet in the same list. PR number cross-checks against `gh pr view 1808`: state `MERGED`, mergeCommit `00ea3e39…`, base `main`, head `int-2b-transitive-const-reexport-doc`, title "Document const import re-export semantics" — all consistent.
- Local-validation claim is consistent with the merge commit's body, which records `report_signature=e1bf653aaa770517, wall_time=54.58s` for `scripts/run_all_tests.sh --profile quick`. That matches the values supplied in this task's context.

### Trimmed carry-over bullet

- The prior bullet enumerated three follow-ups: reserved-name shadowing, fixed-width diagnostic/fallback cleanup, and "document or implement transitive re-export semantics for imported constants." PR #1808 takes the *document* fork of that disjunction (the review's "Scope discipline" section calls this out explicitly), so removing the third clause from the open carry-over is correct. The remaining two clauses are unchanged in wording and ordering.
- The "or implement" half — i.e., extending the producer-side `lower_integer_const_expr_simple` gate to accept `Expr::Name` so transitive re-export becomes more than a documented bound — is intentionally not preserved as a separate open item. That matches note 2 of the review artifact, which flagged the producer/consumer asymmetry as "pre-existing… and consistent with the deferred 'or implement' half of the issue follow-up. Worth tracking but not a blocker." Whether to surface that as its own future-work bullet is a tracker-policy call; not adding it here is defensible because (a) the doc paragraph itself acknowledges the workaround constraint and (b) no other deferred-implementation items from prior reviews are tracked as standalone bullets in this checklist either. Calling this out in case the maintainer prefers to add a separate "future: extend producer-side const-expr gate to accept Name references" line — see "Notes" below.

### Cross-section consistency

- Validation matrix (lines 511-529 of the issue file, unaffected) is not owed a new row: this is a docs-only follow-up and the review artifact's "Consistency with surrounding doc" section confirms transitive re-export is a documented bound rather than a new validated capability.
- The INT-2B parent header at line 427 (`- [ ] INT-2B HIR, type system, and const fitting`) remains `[ ]` because the carry-over bullet is still open. Consistent — this slice does not close the milestone.
- No formatting drift: indentation (two-space child bullets), trailing periods, backtick usage on PR numbers vs. plain `PR #1808` style — all match neighbours.

## Notes / suggestions (non-blocking)

1. **Producer-side `Expr::Name` gap is now silently "documented as bound, never to be implemented."** Once the third clause is removed from the carry-over, there is no remaining tracker line that anticipates lifting the asymmetry between consumer-side fitting (which accepts `Expr::Name`) and producer-side `lower_integer_const_expr_simple` (which does not). If the eventual intention is to keep the bound permanently, this is fine. If it is to eventually implement it, a one-liner like "future: allow `MY_LIMIT: int = LIMIT` to participate in const-evaluable export so the documented intermediate-redeclaration workaround works for both literal and Name initializers" would prevent the item from falling off. Not a blocker for this tracker update; raise with the maintainer as a tracker-policy question.

2. **Bullet phrasing parallels the doc paragraph closely, which is good for traceability** but means it inherits the slight ambiguity flagged as note 1 in the doc-review artifact ("with `from other import LIMIT`" reading more like a mechanism than a trigger). The tracker bullet does not repeat that fragment, so the issue does not propagate. No action.

## Validation

Docs-only tracker update; no build, test, or lint validation is owed for this slice. The merged PR #1808 is the unit that carried `scripts/run_all_tests.sh --profile quick` (report_signature=`e1bf653aaa770517`, wall_time=54.58s). The tracker update reuses that signature implicitly via "quick validation is passing: PR #1808" — which matches the project's convention for tracker rows that document, rather than introduce, validated changes.

`git status` is clean apart from the single tracked-file modification under review, so there is no scope creep to flag.

## Verdict

VERDICT: SATISFIED
