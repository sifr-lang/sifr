I have enough context. Here is the review.

---

# Wave 2.4 Codegen Review — Pass 2

**Verdict: approved**

The pass-1 nit on non-self simple field assignment has been correctly addressed by a resolved-type equality guard plus a new regression at `crates/sifr_codegen/src/lower_stmt/core_and_tuple_tests.rs:260-273`. The six Wave 2.4 rows are closed, the four Wave 2.5 rows remain `open` with `closes_in_subwave: "5"`, and the inventory/docs match the new 704/4/708 result. Ready for PR/merge from a code-review perspective.

## Blocking findings

None.

## Non-blocking nits

1. **Mutation-aware pattern still drops the historical "borrowed ⇒ never mut" invariant** — `crates/sifr_codegen/src/lower_stmt/condition_lowering.rs:17-23,110-116` and `crates/sifr_codegen/src/stmt_support_emitter/condition_lowering.rs:144-150`. Both predicates are now strictly `mutated_vars.contains(name)`; the prior negative guard on borrowed params is gone in the structured emitter too. `collect_mutated_vars` is reliable today, and `option_binding_value_expr_for_ir` at `stmt_support_emitter/condition_lowering.rs:129-142` still adds `.as_ref()` for borrowed params so the resulting `Some(mut x) = x.as_ref()` is well-typed (`mut x: &T`). Carried over from pass-1 — still worth a one-line comment near both predicates documenting why borrowed params are intentionally not negative-guarded anymore.

2. **Resolved-type equality is a structural compare across the full HIR `Type` shape** — `crates/sifr_codegen/src/lower_stmt/return_and_assignment_values.rs:170-172`. `resolve_alias_type(field_ty) != resolve_alias_type(value.ty())` works for the targeted cases (Option vs non-Option, Union shapes), but the check is conservative on intent and broad on cost: every non-self field assignment now performs a deep `Type` comparison. For Wave 2.4 scale this is fine, but if the simple field-assign path widens later, consider replacing the structural compare with an explicit predicate like `field_carries_storage_adaptation(field_ty, value.ty())`. Not a blocker; the current code's intent is documented by the inline comment.

3. **Redundant positive/negative assertion pair in the perf regression** — `crates/sifr_codegen/src/lib_codegen_tests/performance_codegen_tests.rs:370-377`. The `!generated.contains("let Some(mut pair) = pair else")` is strictly entailed by the positive `generated.contains("let Some(pair) = pair else")`. Pass-1 already noted this; kept for documentation intent. OK to leave.

## Review-question answers

1. **Scope correctness.** `jq` confirms exactly 6 rows with `proposed_pr_slice == "2.4"` are `closed` (`closes_in_subwave: "4"`) and 4 rows with `proposed_pr_slice == "2.5"` remain `open` (`closes_in_subwave: "5"`). Inventory totals: 48 closed / 4 open / 52 total. The four open rows are codegen-red-0012, -0029, -0037, -0045 — all `compiler-bug`, none mislabeled or hidden.

2. **Field/value type guard sufficiency.** The added check `resolve_alias_type(field_ty) != resolve_alias_type(value.ty())` at `return_and_assignment_values.rs:170-172` is sufficient for the targeted storage-adaptation cases:
   - Option-of-class field with non-Option leaf value → resolved types differ (Union vs Class) → simple path correctly defers, so the structured emitter handles `Some(Box::new(...))`.
   - Same-type Option-of-class field and value → resolved types match, so the simple path runs; this is fine because the Rust storage representation is identical on both sides (`Option<Box<Node>>` ↔ `Option<Box<Node>>`) and a plain move-assign type-checks.
   - The new `does_not_lower_field_assign_with_mismatched_value_type` regression at `core_and_tuple_tests.rs:260-273` pins the `Union(Int, None)` field vs `Int` value case, which is the canonical storage-adaptation mismatch the pass-1 nit called out.
   The combination of (a) `object != "self"`, (b) resolved-type equality, and (c) `try_lower_leaf_or_name_expr` (rejects non-leaf RHS via `does_not_lower_field_assign_with_non_leaf_value` at `core_and_tuple_tests.rs:244-258`) keeps the simple path correctly scoped.

3. **Mutation-aware option pattern.** Correct in both paths.
   - Simple lowering: `lower_stmt/condition_lowering.rs:17-23,110-116` keys patterns off `mutated_vars`.
   - Structured emitter: `stmt_support_emitter/condition_lowering.rs:144-150` mirrors the same predicate, and `option_binding_value_expr_for_ir` at `stmt_support_emitter/condition_lowering.rs:129-142` still wraps borrowed scrutinees with `.as_ref()`, so `let Some(mut x) = x.as_ref() else ...` would be `mut x: &T` (sound) if a borrowed param ever ends up in `mutated_vars`. For the `break_guard_unwraps_optional_tuple_before_indexing` fixture, `pair` is introduced via `HirStmt::Let` (not `Assign`) and only read via `pair[1]`, so `collect_mutated_vars` correctly excludes it and the new `Some(pair)` pattern is emitted.

4. **Inventory/docs consistency.** Verified:
   - `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json:5-10` reports `passed: 704, failed: 4, total: 708`, matching the local cargo run.
   - `red_blocker.failure_count: 4` at line 16.
   - All six Wave 2.4 rows show `status: "closed"`, `closes_in_subwave: "4"`.
   - All four Wave 2.5 rows show `status: "open"`, `closes_in_subwave: "5"`.
   - `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:384-401` Wave 2.4 notes correctly describe 704/4/708, reference pass-1 review, and document the mismatched-type guard regression.
   - `plans/issues/active/codegen-test-triage.md:3,11,22` consistently reports 704/4/708 and the Wave 2.4 → 2.5 progression.
   - Delta math checks out: previously 697/10/707 → 704/4/708 = +6 passes from row closures, -6 failures, +1 total from the new mismatched-type regression test.

5. **PR/merge readiness.** Yes, from a code-review perspective. The change is tightly scoped, the targeted contracts are well-covered by tests, no Wave 2.5 work bleeds in, and pass-1's only material nit has been addressed at the root cause.

## Verification notes

- `git status` and `git diff --stat` match the seven declared in-scope files plus the two untracked review notes; no collateral edits.
- `jq` over the inventory confirmed: 6 closed at `proposed_pr_slice == "2.4"`, 4 open at `proposed_pr_slice == "2.5"`, all four `compiler-bug` classification, totals 48 closed / 4 open / 52 total.
- `git diff` against HEAD on the test file confirms exactly one new test was added (`does_not_lower_field_assign_with_mismatched_value_type`); the other three field-assign tests already existed in HEAD (one of which — `lowers_simple_field_assign_for_non_self_target` — is the Wave 2.4 row codegen-red-0046 being closed).
- Cross-checked the structured emitter's `option_binding_value_expr_for_ir` to confirm it still wraps borrowed scrutinees with `.as_ref()`, which is what makes the new mutation-aware pattern sound even if `mutated_vars` ever overlaps `borrowed_params`.
- Did not re-run cargo tests or the verification harness; relied on the user-supplied local run logs (six target tests pass, suite at 704/4/708, `create-pr` profile green after the transient core_language rerun, fmt and file-size guardrails green).
