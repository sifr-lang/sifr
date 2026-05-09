# INT-2B Module Const / Fixed-Width Fallback Cleanup — Tracker Update Review Pass 1

**Verdict:** Satisfied with non-blocking suggestions.

## Scope reviewed

Tracker-only working-tree change against `main` (current HEAD `bb32a5cc`):

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)

Reference artifact:

- [reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md) — verdict "Satisfied with non-blocking suggestions" for the merged code change in PR #1814.
- Cited validation: `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=86.64s`).

The diff is a single-file tracker update with two hunks:

1. A new `Review History` line registering the pass-4 review artifact.
2. Replacing the open "Carry remaining follow-ups from INT-2A/INT-2B reviews" placeholder bullet with a checked summary line attributing closure to PR #1814.

No code, demos, fixtures, or other docs are touched.

## Verification

### 1. Review-history entry accurately records the pass-4 artifact

The added line at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:417](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:417) reads:

> `- [x] INT-2B module const/fixed-width fallback cleanup review pass 4 satisfied after addressing pass 2 and pass 3 blockers: reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md.`

Cross-checked against the artifact:

- File path resolves: `reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md` exists on disk (verified) and the linked filename is correct.
- Pass numbering is consistent with the pass-2 and pass-3 artifacts that the artifact itself references at [pass-4 review:16-17](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:16-17).
- The artifact's verdict at [pass-4 review:3](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:3) is "Satisfied with non-blocking suggestions." The tracker uses the shorter "satisfied" phrasing, which matches the established convention used elsewhere in this tracker's `Review History` section (e.g., `INT-2B fixed-width fail fixture marker cleanup review satisfied`, `INT-2B reserved-width shadowing policy documentation review satisfied`). The phrasing does not over-promise: the artifact still records pass-4 as a `Satisfied` verdict, and the tracker's review-history convention is binary (satisfied vs. blockers found). Consistent with prior entries for slices that landed with non-blocking N-items.
- "after addressing pass 2 and pass 3 blockers" is accurate: the pass-4 artifact at [pass-4 review:19-53](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:19-53) explicitly closes the pass-3 blocker (the scope-type guard for `Name`-arm reuse) and the pass-2 follow-up criteria.
- The entry is placed in chronological order at the bottom of the `Review History` list, immediately after the prior INT-2B fixed-width fail fixture marker review entry. Ordering is consistent with the existing pattern.

No fabricated or rephrased history. ✓

### 2. INT-2B checklist item closed by PR #1814 is marked complete with an accurate summary

Replacement of the open bullet at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:444](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:444):

Before:
> `- [ ] Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable.`

After:
> `- [x] Module constant integer fallback paths now preserve budget diagnostics for over-budget module int/fixed-width constants, support same-module int const reuse through names/unary/binops, reject mixed fixed-width-to-int const reuse before codegen, and smoke-test the new codegen shapes; review is satisfied and quick validation is passing: PR #1814.`

Each clause cross-checked against the pass-4 artifact's "Criteria from the pass-4 brief" section ([pass-4 review:55-60](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:55-60)):

- "preserve budget diagnostics for over-budget module int/fixed-width constants" → matches criterion 3 (single `SIFR-INT-0004` for `uint8 = 10 ** 5000` and `int = 10 ** 5000`), verified at [pass-4 review:59](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:59) and pinned by `test_module_fixed_width_const_expression_budget_has_int_code_once` and `test_module_int_over_budget_const_expr_stays_hir_diagnostic`. ✓
- "support same-module int const reuse through names/unary/binops" → matches criterion 1 ("met. The all-`int` paths are exercised by the two new HIR tests and by the e2e fixture"), [pass-4 review:57](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:57). ✓
- "reject mixed fixed-width-to-int const reuse before codegen" → matches criterion 2 ("scope-type guard ... refuses to synthesize an `int`-typed `Name` for a fixed-width source"), [pass-4 review:58](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:58). The "before codegen" wording is faithful: the guard fires at HIR lowering (`lower_module_integer_const_expr`) and the constant is dropped from `module.constants` so it never reaches codegen. ✓
- "smoke-test the new codegen shapes" → matches criterion 4 (e2e fixture at `crates/sifr/tests/e2e/pass/module_constants.sifr` with `BASE_LIMIT + 4` and `-(MAX_RETRIES + 10)` shapes), [pass-4 review:60](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:60). ✓
- "review is satisfied" → matches the artifact's `Verdict` at [pass-4 review:3](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:3) and at [pass-4 review:115](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:115). The "with non-blocking suggestions" qualifier is dropped, but as noted under §1 above, the tracker's convention is binary; the precedent set by other entries in this list (e.g., the const expression fitting pass 2 at line 406 also landed with non-blocking suggestions and is recorded as "satisfied") is being followed. ✓
- "quick validation is passing: PR #1814" → consistent with the user-provided context (`report_signature=e1bf653aaa770517`, `wall_time=86.64s`). The tracker convention does not capture report signatures, only the satisfied/passing fact and the PR number. ✓

The summary is accurate, scoped, and does not over-claim. ✓

### 3. No broader phase/milestone is marked complete incorrectly

Implementation checklist top-level state after the diff (verified at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:421-451](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:421-451)):

- INT-0 — `[x]` (unchanged, was already complete).
- INT-1 — `[ ]` (unchanged; sub-items 1789/1790 done, but the milestone as a whole has open scope).
- INT-2A — `[x]` (unchanged).
- INT-2B — `[ ]` (unchanged). All listed sub-items are now `[x]`, but the parent stays open. This is the correct call: the milestone's acceptance criteria at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:161-168](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:161-168) include "No implicit narrowing occurs in assignments, calls, returns, list literals, dict literals, or generic specialization" and "`bigint` is gone from public docs/tests or emits intentional `SIFR-INT-0011` transition diagnostics only" — the validation list also calls out "Negative tests for implicit narrowing in every source construct listed above" and "Cross-module const fitting tests for imported immutable constants". The slice-level checklist coverage is consistent with INT-2B *substantively* approaching closure, but the diff does not assert closure, which is correct given pre-existing follow-ups (see §4 N1) and the natural milestone-closure pass that should validate end-to-end.
- INT-3 through INT-8 — all `[ ]` (unchanged).

No phase/milestone closure is over-claimed. ✓

### 4. The update does not claim full phase closure or hide known out-of-scope future work

The diff:

- Adds one specific review-history entry for one specific review artifact. No omnibus or phase-closure claim.
- Replaces a single open checklist sub-item with a single checked sub-item attributed to one PR. The replacement does not collapse multiple open items, nor does it widen #1814's scope.
- Does not toggle the parent INT-2B milestone, INT-1, INT-3, or any other phase-level box.

Out-of-scope future work explicitly identified in the pass-4 review artifact:

- N1 (broader regression test for the user-visible diagnostic on mixed-type reuse) — non-blocking, optional follow-up.
- N2 (focused test for bare `-BASE` unary on a name) — non-blocking, optional follow-up.
- N3 (doc comment on `lower_module_integer_const_expr`/`negate_module_integer_const_expr`) — non-blocking, optional follow-up.
- N4 (pre-existing `LargeIntLiteral` codegen panic at [crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12) for in-budget literals exceeding `i64`) — out of scope for this slice; the reviewer notes this is "tied to SifrInt wiring under INT-1/INT-3 wave 2".

None of N1–N4 are surfaced as new tracker bullets, but N1–N3 are by design optional polish that "can land separately or stay as follow-ups; none of them gates merge" (pass-4 verdict, [pass-4 review:115](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:115)). N4 is implicitly carried by the still-open INT-1 and INT-3 milestones because it requires SifrInt wiring rather than further INT-2B work. This is a defensible read of the artifact, but see §Non-blocking findings N1 below — the previously-open catch-all bullet that this diff replaces was the *only* explicit INT-2B-side tracker line for "follow-ups from INT-2A/INT-2B reviews", and replacing it with a #1814-specific summary removes the only place where the pre-existing `LargeIntLiteral` codegen panic was visible from this milestone's vantage point. That isn't a closure-claim violation, but it is a tracker-discoverability nit.

No closure inflation. ✓

## Non-blocking findings

### N1 — Pre-existing `LargeIntLiteral` codegen panic loses its INT-2B-side tracker breadcrumb

Before this diff, [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) carried the open bullet "Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable." That phrasing was a deliberate catch-all for unresolved INT-2A/INT-2B fallout, and the pass-4 review's N4 finding explicitly called out the in-budget `LargeIntLiteral` codegen panic at [crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12) and said "worth tracking explicitly so it doesn't get lost behind the pass-3 fix" ([pass-4 review:100](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:100)).

After this diff the catch-all is gone — replaced by a #1814-specific summary — and the codegen panic is not separately tracked under INT-2B, INT-1, or INT-3. The reviewer's framing of N4 ("tied to SifrInt wiring under INT-1/INT-3 wave 2") is reasonable, but neither INT-1 nor INT-3 currently has a tracker line that would trip on it. A future maintainer reading the tracker would see no remaining INT-2B follow-ups and no specific INT-1/INT-3 sub-bullet for the panic, so the breadcrumb that the pass-4 reviewer asked for is, in practice, only preserved by the review artifact itself.

Non-blocking because:

- The pass-4 artifact remains in `reviews/` and is now linked from `Review History`, so the panic is not lost in the absolute sense.
- The panic is genuinely outside this slice's scope and outside INT-2B's representation/const-fitting scope.

Suggested fix (optional, can land separately): add a one-line bullet under INT-1 or INT-3 such as "Wire module-constant codegen to use `SifrInt` for in-budget integer literals exceeding `i64`, removing the [`emit_module_constants` panic path](crates/sifr_codegen/src/module_constants.rs:12) that's currently reachable from valid `int` constants." That would preserve the cross-reference the pass-4 reviewer asked for without expanding this PR's intent.

### N2 — "review is satisfied" elides the "with non-blocking suggestions" qualifier

The new tracker bullet says "review is satisfied" while the artifact's verdict is "Satisfied with non-blocking suggestions." This matches the established convention used in surrounding entries (e.g., [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:443](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:443)) and is therefore not a regression, but it does mean the tracker reader has to open the artifact to see that there are documented non-blocking N-items. Not a blocker; flagged only because the user prompt asked specifically that the update "not hide known out-of-scope future work" — the qualifier is the convention's chosen abbreviation, not concealment.

### N3 — `report_signature` and `wall_time` not captured

The merged-PR validation context (`report_signature=e1bf653aaa770517`, `wall_time=86.64s`) is not reflected in the tracker. The tracker's convention does not currently capture either, so this is consistent with prior entries; flagged only for completeness given the user prompt explicitly cited those values.

## Determinism / scope-drift check

- Diff touches a single file (`issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`) with two narrow hunks. No code, demos, fixtures, snapshots, or other docs modified.
- No reordering of unrelated bullets or list items; both hunks add/replace lines in place at the expected positions.
- No phase headers, milestone titles, acceptance criteria, validation commands, or design-summary content edited.
- Review-history line follows the existing pattern: `[x] <slice description> review<#>...: reviews/<filename>.md.`
- Implementation-checklist line follows the existing pattern: `[x] <past-tense outcome>; review is satisfied and quick validation is passing: PR #<n>.`

No drift. ✓

## Verdict

**Satisfied with non-blocking suggestions.** The two tracker hunks faithfully record what PR #1814 closed and the pass-4 review artifact that approved it. The new `Review History` entry points at a real, satisfied artifact whose contents match the summary line. The replacement implementation-checklist bullet is scoped to PR #1814's actual deliverables and does not reach beyond them: budget-diagnostic preservation, same-module `int` const reuse via names/unary/binops, mixed-type reuse rejection at HIR before codegen, and the e2e codegen smoke-test fixture all map back to the artifact's four pass-4 criteria. No phase or milestone box is incorrectly marked complete — INT-2B remains open, and no INT-3+ box is touched. The only nit (N1) is that the previously-open catch-all bullet was the lone INT-2B-side breadcrumb for the pre-existing `LargeIntLiteral` codegen panic flagged in the pass-4 review's N4, and replacing it with a #1814-specific summary leaves that panic tracked only by the review artifact itself; an optional one-line bullet under INT-1 or INT-3 would restore that breadcrumb without expanding this update's scope. None of N1–N3 gate this tracker change.
