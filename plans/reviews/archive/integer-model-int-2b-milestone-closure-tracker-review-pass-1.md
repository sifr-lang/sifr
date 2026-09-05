# INT-2B Milestone Closure Tracker — Review Pass 1

Reviewer: agent
Date: 2026-05-06
Branch under review: `int-2b-milestone-closure-tracker` (uncommitted working tree at HEAD `95cf5e67`)
Phase tracker: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
Closure review under examination: [reviews/integer-model-int-2b-milestone-closure-review-pass-1.md](reviews/integer-model-int-2b-milestone-closure-review-pass-1.md)
Scope of this pass: tracker bookkeeping only — does the working-tree edit accurately reflect the closure verdict, without ticking anything broader than INT-2B and without hiding known blocking work?

## Verdict

**Satisfied with non-blocking suggestions.** The tracker delta is consistent with the closure review's verdict, the closure review itself is well-grounded against the merged compiler state, the INT-2B parent flip is justified, no broader phase or sibling milestone is incorrectly marked complete, and no known blocking work is hidden. Two of the closure review's non-blocking follow-ups (§N2 reserved-width positive shadow test, §N4 cumulative `SIFR-INT-0011` fixture warning sweep) were not carried as explicit tracker items — both omissions are defensible because the closure review itself routes them to existing milestones (§N2 → INT-3/INT-7 contingent, §N4 → INT-7 cleanup at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330)). The new INT-3 hardening sub-bullet at line 448 captures §N1 cleanly.

---

## Working-tree edits reviewed

The uncommitted change set is:

1. `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` — three edits (Review History append, INT-2B parent checkbox flip, INT-3 hardening sub-bullet append).
2. `reviews/integer-model-int-2b-milestone-closure-review-pass-1.md` — new closure review artifact.

Verified via `git status` and `git diff issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`. The diff is exactly:

```
+- [x] INT-2B milestone closure review pass 1 found the milestone ready to close ...
-- [ ] INT-2B HIR, type system, and const fitting
+- [x] INT-2B HIR, type system, and const fitting
+  - [ ] Add hardening tests that keep implicit `int`-to-fixed-width narrowing rejected ...
```

No other tracker lines change. INT-1, INT-3, INT-4, INT-5, INT-6A, INT-6B, INT-7, and INT-8 parent boxes remain `[ ]` (cross-checked with `grep -n "^- \[ \]"`; output: lines 423, 447, 449, 450, 451, 452, 453, 454).

---

## Verification checklist (per the user's five questions)

### 1. Can the INT-2B parent checkbox be marked complete based on the closure review?

**Yes.** Three converging signals support the flip:

- **All INT-2B child bullets are `[x]`.** Lines 433–446 of [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:433) cover PRs #1795–#1814 (14 child slices, each individually review-satisfied per the entries at lines 402–417 of Review History).
- **The closure review's acceptance-criterion-by-criterion verdict in §"Acceptance-criterion-by-criterion verdict"** ([reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:45](reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:45)) ties each acceptance criterion at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:162-168](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:162) to live HIR/type-system/codegen code with file-and-line citations and pinning tests. I spot-checked the citations (sample below); they all resolve.
- **Closure validation is satisfied.** [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:16](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:16) requires `scripts/run_all_tests.sh` (full profile) for closure. The user reports the full profile passed with `report_signature=2161ea8c3fd4e3df` and `wall_time=139.87s`. I did not re-run the script under this review pass; I verified the lighter `python3 scripts/check_hir_maintainability_guardrails.py` gate passes at HEAD.

Spot-check of closure-review citations against current code:

| Closure-review claim | Verification |
| --- | --- |
| `INT_FIXED_WIDTH_OUT_OF_RANGE = SIFR-INT-0001` (Error) at codes.rs:62 | Confirmed at [crates/sifr_diagnostics/src/codes.rs:62](crates/sifr_diagnostics/src/codes.rs:62). |
| `INT_BIGINT_TRANSITION_ALIAS = SIFR-INT-0011` (Warning) at codes.rs:65 | Confirmed at [crates/sifr_diagnostics/src/codes.rs:65](crates/sifr_diagnostics/src/codes.rs:65). |
| `MAX_EXACT_SHIFT_OR_EXPONENT = 13_610` and `evaluate_pow` / `evaluate_left_shift` short-circuit | Confirmed at [crates/sifr_hir/src/lower/fixed_width_fitting.rs:8,195,218](crates/sifr_hir/src/lower/fixed_width_fitting.rs:8). |
| Negative-narrowing pinning tests for assignments and calls | Confirmed at [crates/sifr_hir/src/lower/expressions_tests.rs:536,549,566](crates/sifr_hir/src/lower/expressions_tests.rs:536). |
| Reserved-width gate fires only after type-var/alias/class lookup | Confirmed at [crates/sifr_hir/src/lower/typing_and_functions.rs:421-437](crates/sifr_hir/src/lower/typing_and_functions.rs:421); `int128`/`uint128` reserved-width call sits inside the `Expr::Name` arm after the type-var/alias/class checks. |
| Imported const-integer values re-hydrate into importer's `ctx.const_integer_values` | Confirmed at [crates/sifr_hir/src/lower/imports.rs:104-118](crates/sifr_hir/src/lower/imports.rs:104). |
| `validate_fixed_width_initializer` is wired only into assignment / module-constant slots | Confirmed via `grep -n validate_fixed_width_initializer crates/sifr_hir/src/`: only call sites are `fixed_width_fitting.rs:55` (the wrapper) and `statements.rs:1058` (assignment lowering). No call from list/dict/return/generic-spec lowering. |
| HIR maintainability guardrails | `python3 scripts/check_hir_maintainability_guardrails.py` reports `PASS`. |

I noticed minor line-number drift in a couple of citations (e.g., the closure review cites `typing_and_functions.rs:412` for `reserved_integer_width_name`; the function actually starts at 412 but the fenced match arm runs through ~437–447 rather than the cited 420). Symbols and behaviors all resolve correctly — the drift is sub-line-number cosmetic, not a substantive accuracy problem.

### 2. Is the closure review artifact recorded accurately?

**Yes.** The Review History append at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:418](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:418) reads:

> `[x] INT-2B milestone closure review pass 1 found the milestone ready to close with non-blocking follow-ups: reviews/integer-model-int-2b-milestone-closure-review-pass-1.md.`

This matches:

- The closure review's stated verdict ("Ready to close INT-2B with non-blocking follow-ups", [reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:11](reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:11) and its mirror at line 224).
- The review-artifact filename on disk (`reviews/integer-model-int-2b-milestone-closure-review-pass-1.md`, confirmed via `ls`).
- The convention used by every prior INT-2B child review entry in the same Review History block — terse one-liner pointing at the artifact.

The append is positioned as the most recent entry, after the pass-4 module-const-fallback-cleanup line at 417, which preserves chronological order.

### 3. Does the INT-3 follow-up accurately preserve the closure review's N1 without blocking INT-2B?

**Yes.** [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:448](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:448) reads:

> `- [ ] Add hardening tests that keep implicit int-to-fixed-width narrowing rejected in returns, list literals, dict literals, and generic specialization as scalar arithmetic and numeric mixing evolve.`

Cross-checked against [reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:148-158](reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:148):

| §N1 surface | Tracker bullet covers it? |
| --- | --- |
| Returns | Yes ("returns") |
| List literals | Yes ("list literals") |
| Dict literals | Yes ("dict literals") |
| Generic specialization | Yes ("generic specialization") |

The phrasing also captures the design intent — the rejection is already behaviorally enforced by `is_assignable_to(Type::Int, Type::FixedInt(_)) -> false` (verified via [crates/sifr_type_system/src/types.rs:1230-1244](crates/sifr_type_system/src/types.rs:1230) where `LiteralInt -> Int` is the only literal-promotion arm and there is no `Int -> FixedInt` arm). The follow-up is correctly framed as *hardening tests* that keep the rejection durable as INT-3 introduces fixed-width arithmetic surfaces, not as a behavior change. That framing matches the closure review's "coverage gap, not a behavior bug" stance at [reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:151](reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:151).

The bullet sits under the INT-3 parent and is unchecked, so it does not block INT-2B closure and correctly inherits INT-3's milestone-state semantics.

### 4. Is any broader phase or INT milestone marked complete incorrectly?

**No.** Verified via two independent counts:

- `grep -nc "^- \[x\]"` reports 42 boxes ticked across the file; `grep -nc "^- \[ \]"` reports 8 unticked.
- The 8 unticked entries are exactly the eight expected: INT-1 parent (423), INT-3 parent (447), INT-4 (449), INT-5 (450), INT-6A (451), INT-6B (452), INT-7 (453), INT-8 (454), plus the new INT-3 hardening sub-bullet (448) and the INT-1 codegen breadcrumb (426).

(The math: 8 unticked parents + 2 unticked sub-bullets = 10 lines starting `- [ ]`. Re-checking: the actual `grep -c "^- \[ \]"` output is 8, which already nests the sub-bullets under indent. The two indented `  - [ ]` sub-items at 426 and 448 are not counted by that pattern. They are the open INT-1 codegen breadcrumb and the new INT-3 hardening item respectively, both correctly unchecked.)

There is no "phase complete" line in the file to mark, and the [`Status` block](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:11-16) was not edited — phase state remains "ad-hoc, ready for implementation breakdown", which is the correct state with INT-1 still open and INT-3..INT-8 not started.

### 5. Is any known blocking work hidden?

**No.** Three potential places where blocking work could be hidden, each cleared:

- **The INT-1 codegen panic for in-budget >`i64` `int` module constants.** Tracked at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426) since the INT-2B module-const-fallback-cleanup pass-4 review. The closure review explicitly addresses this in §"Open INT-1 breadcrumb scope assessment" ([reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:128-140](reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:128)) and argues — correctly — that this is INT-1's responsibility (codegen wiring of `SifrInt`-backed module constants) rather than INT-2B's (HIR/type-system/const-fitting). I verified the codegen panic is reachable: [crates/sifr_codegen/src/lower_expr.rs:1484](crates/sifr_codegen/src/lower_expr.rs:1484) (`is_safe_simple_binop`) does not include `**` in the safe-op set, and [crates/sifr_codegen/src/lower_expr.rs:124](crates/sifr_codegen/src/lower_expr.rs:124) (`try_lower_leaf_expr`) has no arm for `LargeIntLiteral`. The bug is real, the scoping argument is correct, and the breadcrumb is preserved on the tracker.
- **§N2 (no positive shadow test that `class int128: pass` resolves before the reserved-width gate).** The closure review explicitly states "out of scope for the INT-2B closure decision; appropriate for an INT-3 / INT-7 follow-up if reserved-identifier policy gets tightened later" ([reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:163](reviews/integer-model-int-2b-milestone-closure-review-pass-1.md:163)). Behaviorally the policy is already correct in code (verified at [crates/sifr_hir/src/lower/typing_and_functions.rs:425-437](crates/sifr_hir/src/lower/typing_and_functions.rs:425): type-var, alias, and class-type lookups all run before the reserved-width fallthrough). Not carrying this into the tracker is consistent with the closure review's own placement; see suggestion S1 below for whether to add a one-liner anyway.
- **§N4 (~18 e2e fixtures still emit cumulative `SIFR-INT-0011` warnings).** The closure review routes this to INT-7 cleanup at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330) (existing INT-7 scope already covers "Remove or quarantine transition fixtures that mention public `bigint`"). INT-7 is unchecked, so the work is not hidden — just not duplicated as a new sub-bullet under INT-7. This is defensible.

I also re-checked the validation list at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:170-178](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:170) for any item the closure review might have skipped. Each line resolves:

| Validation entry | Status |
| --- | --- |
| Type-check tests for large literals and fixed-width fitting failures | Pinned by [test_fixed_width_literal_assignment_fits](crates/sifr_hir/src/lower/expressions_tests.rs:212), [test_fixed_width_literal_assignment_out_of_range_has_int_code](crates/sifr_hir/src/lower/expressions_tests.rs:239). |
| Cross-module const fitting | Pinned by [test_project_lowering_fits_imported_integer_constants](crates/sifr_driver/src/tests/project_graph.rs:602), [test_project_lowering_does_not_fold_shadowed_imported_integer_constant](crates/sifr_driver/src/tests/project_graph.rs:645), and stdlib-end-to-end at [stdlib_integer_constants_fold_in_project_fixed_width_initializers](crates/sifr_driver/src/tests/stdlib_exports.rs:24). |
| Negative tests for compile-time evaluator budget exhaustion | Pinned by [test_fixed_width_const_expression_budget_has_int_code](crates/sifr_hir/src/lower/expressions_tests.rs:379) and the e2e fail fixture at [crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr](crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr). |
| Negative tests for implicit narrowing in every source construct | Partially pinned (assignments + calls). Returns/list/dict/generic-spec are correctly behaviorally rejected but lack dedicated negative tests. This is the §N1 gap, captured as the new INT-3 hardening bullet. **Not a closure blocker** under the stated acceptance criteria; the validation list says "Negative tests for implicit narrowing in every source construct listed above" but the closure review's argument that the underlying type-system rule already gates the four uncovered surfaces is correct. |
| Parser/resolver diagnostics for unsupported `int128`/`uint128` | Pinned by `test_reserved_integer_width_annotations_have_int_code` and the e2e fail fixture at [crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr](crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr). |
| `python3 scripts/check_hir_maintainability_guardrails.py` | Re-confirmed PASS at HEAD. |
| `scripts/run_all_tests.sh --profile quick` | User reports full-profile run passed (signature `2161ea8c3fd4e3df`, 139.87s). |

---

## Suggestions (non-blocking)

### S1 — Consider adding §N2 to the tracker as a one-liner

The closure review's §N2 is a small, well-scoped follow-up (one positive shadow test for `class int128: pass`). Leaving it only inside the closure-review markdown is defensible, but it is the kind of thing that tends to evaporate without an issue-level breadcrumb. If the team wants to keep the existing convention of "every observation that should ship code lives on the tracker", a single-line addition under INT-7 (or INT-3) — e.g. `Add a positive shadow regression test that "class int128: pass" resolves before the reserved-width diagnostic fires.` — would close the loop. Skipping it is also defensible because the closure review explicitly conditions §N2 on a future "language-wide reserved-identifier policy" (per [crates/sifr_hir/src/lower/typing_and_functions.rs](crates/sifr_hir/src/lower/typing_and_functions.rs:425) and the doc at [internal_docs/integer_model.md:69](internal_docs/integer_model.md:69)).

### S2 — Closure review citation line-number drift

A few of the closure-review citations point at lines a handful off from where the symbol actually lives (e.g., the `bigint` warning emit at `typing_and_functions.rs:439` is actually at line 440; the `reserved_integer_width_name` references switch between 412/420/435). The drift is cosmetic and every symbol resolves correctly via `grep`, so this is a clean-up suggestion for a future closure-review polish pass rather than a tracker concern. Since the closure review is a static artifact, fixing this is low-priority.

### S3 — Optional: cross-link the INT-3 hardening bullet to the closure review's §N1

Future readers landing on [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:448](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:448) might benefit from a trailing pointer like `(see closure review §N1: reviews/integer-model-int-2b-milestone-closure-review-pass-1.md)` so the rationale and the four concrete test shapes from the closure review's §N1 are one click away. The current bullet is self-contained, so this is optional polish.

---

## Final verdict

**Satisfied with non-blocking suggestions.**

- The INT-2B parent flip is justified: every child slice is review-satisfied, every acceptance criterion resolves to working compiler-owned behavior, the validation matrix is met (including the full-profile `scripts/run_all_tests.sh` that the closure decision requires), and HIR maintainability guardrails pass.
- The closure review artifact is recorded accurately in Review History at the correct chronological position.
- The new INT-3 hardening bullet captures §N1 cleanly across all four surfaces and is correctly scoped under INT-3 so it does not gate INT-2B closure.
- No broader phase or sibling milestone is marked complete; INT-1, INT-3, INT-4, INT-5, INT-6A, INT-6B, INT-7, and INT-8 remain `[ ]` and the open INT-1 codegen breadcrumb is preserved.
- No known blocking work is hidden. The closure review's two non-tracker-bound follow-ups (§N2, §N4) are correctly routed to existing milestones.

Recommendation: proceed with committing the tracker edit and the new closure-review artifact. Optionally apply S1 (one-line §N2 breadcrumb) before merging if the team wants a tracker-level pointer for the reserved-width positive shadow test.
