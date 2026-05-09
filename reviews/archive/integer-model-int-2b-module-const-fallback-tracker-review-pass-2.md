# INT-2B Module Const / Fixed-Width Fallback Cleanup — Tracker Update Review Pass 2

**Verdict:** Satisfied.

## Scope reviewed

Tracker-only working-tree change against `main` (current HEAD `bb32a5cc`):

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)

Reference artifacts:

- [reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md) — pass-4 code review for PR #1814; verdict "Satisfied with non-blocking suggestions" with N4 specifically calling out a pre-existing in-budget `LargeIntLiteral` codegen panic at [crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12).
- [reviews/integer-model-int-2b-module-const-fallback-tracker-review-pass-1.md](reviews/integer-model-int-2b-module-const-fallback-tracker-review-pass-1.md) — pass-1 tracker review; verdict "Satisfied with non-blocking suggestions" with N1 suggesting an explicit INT-1/INT-3 breadcrumb to preserve N4 visibility from the tracker.

The diff is a single-file tracker update with three hunks now (one more than pass-1):

1. New `Review History` line registering the pass-4 code-review artifact.
2. **New** open INT-1 sub-bullet recording the pre-existing in-budget `LargeIntLiteral` module-constant codegen panic.
3. Replacing the open "Carry remaining follow-ups from INT-2A/INT-2B reviews" placeholder bullet with a checked summary line attributing closure to PR #1814.

No code, demos, fixtures, or other docs are touched.

## Verification

### 1. New open INT-1 sub-bullet is an accurate scoped breadcrumb for pass-4 N4 / pass-1 N1

The newly added line at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425) reads:

> `- [ ] Wire module-level `int` constants whose in-budget values exceed `i64` through `SifrInt` codegen, removing the current module-constant production panic path tracked by the INT-2B module const/fixed-width fallback cleanup review.`

Cross-checked against the upstream pass-4 N4 ([pass-4 review:98-100](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:98-100)):

- The pass-4 reviewer described the panic as: a literal in budget but exceeding `i64` (e.g., `LIMIT: int = 999999999999999999999999999999999999`) survives `lower_module_integer_const_expr` → `lower_integer_const_expr_simple` → `LargeIntLiteral`, then `try_lower_simple_module_constant_item_result_impl` returns `Ok(None)`, and `emit_module_constants` panics at [crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12). I confirmed the panic site is still present and unchanged ([crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12) — `panic!("structured module constant emission missing for production path ({name}): {err}")`).
- The new bullet's framing ("module-level `int` constants whose in-budget values exceed `i64`") accurately captures the trigger condition: in-budget (so the `SIFR-INT-0004` path is not entered), `int`-typed (so fixed-width fitting is not entered), and exceeding `i64` (so the `IntLiteral(i64)` simple-lowering path is bypassed in favor of `LargeIntLiteral`). ✓
- The remediation language ("through `SifrInt` codegen") matches the pass-4 reviewer's framing of the fix as "tied to SifrInt wiring under INT-1/INT-3 wave 2" ([pass-4 review:100](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:100)). The pass-1 tracker-review's suggested wording ([pass-1 tracker review:106](reviews/integer-model-int-2b-module-const-fallback-tracker-review-pass-1.md:106)) was structurally equivalent ("Wire module-constant codegen to use `SifrInt` for in-budget integer literals exceeding `i64`, removing the [`emit_module_constants` panic path](crates/sifr_codegen/src/module_constants.rs:12)..."). The applied wording is slightly different — it omits the file:line reference and points back to the cleanup review by name — but it is faithful to both the upstream technical description and the pass-1 suggestion. ✓
- "removing the current module-constant production panic path tracked by the INT-2B module const/fixed-width fallback cleanup review" is the cross-reference: the named review (pass-4) is the artifact whose N4 documents the panic, so a future maintainer looking at this bullet has a one-hop path to the technical detail. ✓
- Placement under INT-1 is correct: INT-1's stated scope at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:96-106](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:96-106) covers `SifrInt` runtime substrate, codegen wiring, and "Generated code can construct, clone/reuse, compare, hash, and format `int` values through `SifrInt`" ([acceptance:110](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:110)). Module-constant `SifrInt` wiring fits squarely in that scope. INT-3 is also a defensible home (the pass-4 reviewer mentioned "INT-1/INT-3 wave 2") because INT-3 is where `Type::Int` arithmetic lowering through `SifrInt` lands ([INT-3 scope:184-193](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:184-193)), but the panic itself is in module-constant *emission* (codegen substrate), so INT-1 is the more direct fit. The pass-1 tracker review explicitly listed both as acceptable targets ("under INT-1 or INT-3"), so either placement is consistent with that suggestion. ✓
- The bullet is `[ ]` (open), correctly reflecting that this work is not yet done. ✓
- Scope is bounded — it does not bundle unrelated INT-1 work, does not widen into INT-3 arithmetic, and does not try to re-litigate the fixed-width fallback path that PR #1814 closed. ✓

The breadcrumb is accurate, scoped, and addresses pass-1 N1 directly. ✓

### 2. Review history accurately records the satisfied Claude review artifact for #1814

The added line at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:417](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:417) reads:

> `- [x] INT-2B module const/fixed-width fallback cleanup review pass 4 satisfied after addressing pass 2 and pass 3 blockers: reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md.`

Cross-checked against the artifact:

- File path resolves: I verified the file exists on disk at `reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md`.
- Pass numbering ("pass 4 ... after addressing pass 2 and pass 3 blockers") is consistent with the artifact's reference list at [pass-4 review:16-17](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:16-17), which links the pass-2 and pass-3 reviews, and with the artifact's "Pass-3 blocker resolution — closed" section at [pass-4 review:19-53](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:19-53).
- The artifact's verdict at [pass-4 review:3](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:3) and [pass-4 review:115](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:115) is "Satisfied with non-blocking suggestions." The tracker uses the shorter "satisfied" phrasing, consistent with the existing `Review History` convention used for siblings such as [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:406](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:406) (`INT-2B fixed-width const expression fitting review pass 2 satisfied after addressing blockers`) and [:416](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:416) (`INT-2B fixed-width fail fixture marker cleanup review satisfied`). The tracker convention is binary; the artifact still records "Satisfied" as the verdict, and the non-blocking N-items remain enumerated in the artifact itself. Not concealment. ✓
- Chronological placement at the end of the `Review History` list, immediately after the prior INT-2B fixed-width fail fixture marker review entry, is consistent with the existing pattern. ✓

No fabricated or rephrased history. ✓

### 3. INT-2B checklist item closed by PR #1814 is marked complete with an accurate summary

Replacement of the open bullet at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:445](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:445):

Before:
> `- [ ] Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable.`

After:
> `- [x] Module constant integer fallback paths now preserve budget diagnostics for over-budget module int/fixed-width constants, support same-module int const reuse through names/unary/binops, reject mixed fixed-width-to-int const reuse before codegen, and smoke-test the new codegen shapes; review is satisfied and quick validation is passing: PR #1814.`

Each clause cross-checked against the pass-4 artifact's "Criteria from the pass-4 brief" section ([pass-4 review:55-60](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:55-60)):

- "preserve budget diagnostics for over-budget module int/fixed-width constants" → matches criterion 3 (single `SIFR-INT-0004` for `uint8 = 10 ** 5000` and `int = 10 ** 5000`), pinned by `test_module_fixed_width_const_expression_budget_has_int_code_once` and `test_module_int_over_budget_const_expr_stays_hir_diagnostic` ([pass-4 review:59](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:59)). ✓
- "support same-module int const reuse through names/unary/binops" → matches criterion 1 ("met. The all-`int` paths are exercised by the two new HIR tests and by the e2e fixture", [pass-4 review:57](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:57)). ✓
- "reject mixed fixed-width-to-int const reuse before codegen" → matches criterion 2 ("scope-type guard ... refuses to synthesize an `int`-typed `Name` for a fixed-width source", [pass-4 review:58](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:58)). The "before codegen" wording is faithful: the guard fires at HIR lowering (`lower_module_integer_const_expr`) and the constant is dropped from `module.constants` before codegen runs. ✓
- "smoke-test the new codegen shapes" → matches criterion 4 (e2e fixture at `crates/sifr/tests/e2e/pass/module_constants.sifr` with `BASE_LIMIT + 4` and `-(MAX_RETRIES + 10)` shapes, [pass-4 review:60](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:60)). ✓
- "review is satisfied and quick validation is passing: PR #1814" → consistent with the artifact's verdict and with the user-provided merge context (`report_signature=e1bf653aaa770517`, `wall_time=86.64s`); the tracker convention does not capture report signatures or wall times. ✓

The summary is accurate, scoped, and does not over-claim. ✓

### 4. No broader phase/milestone is marked complete incorrectly

Implementation-checklist top-level state after the diff (verified at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:421-452](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:421-452)):

- INT-0 — `[x]` (unchanged, already complete).
- INT-1 — `[ ]` (unchanged). The new sub-bullet at [:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425) is `[ ]`, so the parent stays correctly open. ✓
- INT-2A — `[x]` (unchanged).
- INT-2B — `[ ]` (unchanged). All listed sub-items at [:431-445](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:431-445) are now `[x]`, but the parent stays open. This is the correct call: the milestone's acceptance criteria at [:161-168](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:161-168) include "No implicit narrowing occurs in assignments, calls, returns, list literals, dict literals, or generic specialization" and the validation list at [:170-178](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:170-178) calls out broader negative tests; the slice-level checklist is consistent with INT-2B substantively approaching closure but does not assert closure, which is correct given the open INT-1 breadcrumb above and the natural milestone-closure pass that should validate end-to-end.
- INT-3 through INT-8 — all `[ ]` (unchanged).

No phase/milestone closure is over-claimed. ✓

### 5. The update does not claim full phase closure or hide known out-of-scope future work

The diff:

- Adds one specific review-history entry for one specific review artifact. No omnibus or phase-closure claim.
- Replaces a single open checklist sub-item with a single checked sub-item attributed to one PR. The replacement does not collapse multiple open items, nor does it widen #1814's scope.
- Adds one open INT-1 sub-bullet that explicitly preserves the pre-existing `LargeIntLiteral` codegen panic as a known out-of-scope follow-up. This directly addresses pass-1 N1, which flagged that the previously-open catch-all bullet was the only INT-2B-side breadcrumb for that panic.
- Does not toggle the parent INT-2B milestone, INT-1, INT-3, or any other phase-level box.

Out-of-scope future work explicitly identified in the pass-4 review artifact:

- N1 (broader regression test for the user-visible diagnostic on mixed-type reuse) — non-blocking, optional follow-up.
- N2 (focused test for bare `-BASE` unary on a name) — non-blocking, optional follow-up.
- N3 (doc comment on `lower_module_integer_const_expr`/`negate_module_integer_const_expr`) — non-blocking, optional follow-up.
- N4 (pre-existing in-budget `LargeIntLiteral` codegen panic) — **now explicitly tracked** by the new INT-1 sub-bullet at [:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425).

The pass-1 tracker-review verdict noted that N4 was the only pass-4 follow-up not preserved in the tracker after the diff; that gap is now closed. N1–N3 remain unrepresented in the tracker, which is consistent with the established convention for non-blocking artifact-level N-items (the artifact carries them; the tracker captures the binary outcome).

No closure inflation. ✓

## Determinism / scope-drift check

- Diff touches a single file (`issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`) with three narrow hunks. No code, demos, fixtures, snapshots, or other docs modified.
- No reordering of unrelated bullets or list items; all hunks add/replace lines in place at the expected positions.
- No phase headers, milestone titles, acceptance criteria, validation commands, or design-summary content edited.
- Review-history line follows the existing pattern: `[x] <slice description> review<#>...: reviews/<filename>.md.`
- Implementation-checklist closure line follows the existing pattern: `[x] <past-tense outcome>; review is satisfied and quick validation is passing: PR #<n>.`
- New INT-1 open bullet follows the prevailing open-bullet style for INT-1 sub-items (concise scope statement, no PR or review attribution since the work is not yet done).

No drift. ✓

## Comparison against pass-1 tracker-review findings

| Pass-1 finding | Status in pass-2 |
| --- | --- |
| N1 — Pre-existing `LargeIntLiteral` codegen panic loses its INT-2B-side tracker breadcrumb | **Closed.** The new INT-1 sub-bullet at [:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425) preserves the breadcrumb under INT-1, matching the pass-1 review's suggested fix. The new bullet's wording is structurally equivalent to the pass-1 suggestion. |
| N2 — "review is satisfied" elides the "with non-blocking suggestions" qualifier | Unchanged. As pass-1 noted, this is the established tracker convention and not a regression. The artifact still preserves the qualifier. |
| N3 — `report_signature` and `wall_time` not captured | Unchanged. Tracker convention does not capture these values; consistent with all prior entries. |

Pass-1's load-bearing non-blocking suggestion (N1) has been addressed; the remaining two were convention notes and remain consistent with the rest of the tracker.

## Verdict

**Satisfied.** All three tracker hunks faithfully record what PR #1814 closed, the pass-4 review artifact that approved it, and the pre-existing in-budget `LargeIntLiteral` codegen panic that the pass-4 reviewer asked be tracked separately. The new INT-1 sub-bullet is an accurate, scoped breadcrumb for pass-4 N4 / pass-1 N1: it correctly identifies the trigger condition (module-level `int` constants whose in-budget values exceed `i64`), points the reader at the right artifact for technical detail, lands under the right milestone (INT-1, where `SifrInt` codegen wiring sits), and stays `[ ]` open. The `Review History` entry points at a real, satisfied artifact whose contents match the summary line. The replacement implementation-checklist bullet is scoped to PR #1814's actual deliverables and does not reach beyond them. No phase or milestone box is incorrectly marked complete — INT-2B remains open, INT-1 remains open with a now-explicit follow-up, and no INT-3+ box is touched. The pass-1 tracker review's only load-bearing non-blocking suggestion has been addressed; the remaining two pass-1 nits were convention-level and remain consistent with existing tracker entries. No new findings.
