I have enough context. Now I'll write the review.

---

# Wave 2.4 Codegen Review — Pass 1

**Verdict: approved with nits**

The six Wave 2.4 rows are correctly closed by tightly scoped changes. Root behavior is right for the targeted contracts, the inventory and tracking docs are internally consistent, and the four remaining `proposed_pr_slice: 2.5` rows stay `open` with `closes_in_subwave: 5`. No blocking issues found.

## Blocking findings
None.

## Non-blocking nits

1. **Non-self simple field-assign lowering lacks a field/value-type guard** — `crates/sifr_codegen/src/lower_stmt/return_and_assignment_values.rs:159-177`. The new `try_lower_simple_field_assign_stmt` accepts any `object != "self"` and lowers `object.field = <leaf-or-name>` raw. If a non-self field carries a codegen-only storage adaptation (e.g. recursive `Box<T>` or option-wrapping on the field), the simple path would emit `node.field = value` without the corresponding `Box::new(...)` / `Some(...)` wrap. The Wave 2.4 regression at `lower_stmt/core_and_tuple_tests.rs:208-230` only covers `Type::Int → Type::Int`, and the full suite stays green, so this is plausibly unreachable from user code today — but the asymmetry (self path defers to structured emitter for exactly this reason per the inline comment) is worth either documenting or guarding on `field_ty == value.ty()` before a future recursive non-self field case lands.

2. **Pattern-mut heuristic dropped borrowed-param negative guard** — `lower_stmt/condition_lowering.rs:17-23,110-116` and `stmt_support_emitter/condition_lowering.rs:144-150`. The new predicate is purely `mutated_vars.contains(name)`. If a borrowed `&Option<T>` param ever lands in `mutated_vars` (e.g. local rebinding of the param name), the emitted pattern becomes `Some(mut x)` applied to a scrutinee that goes through `.as_ref()` (`option_binding_value_expr_for_ir` at `condition_lowering.rs:129-142` is unchanged). That still compiles — `mut x: &T` is sound — but it is a behavior change from the prior "borrowed ⇒ never mut" invariant; worth a short note in the wave summary.

3. **Performance-test assertion pair is good, but the negative assertion is redundant once `Some(pair)` is asserted** — `lib_codegen_tests/performance_codegen_tests.rs:370-377`. `generated.contains("let Some(pair) = pair else") && !generated.contains("let Some(mut pair) = pair else")` is belt-and-suspenders; the positive check is a strict substring so it already excludes the `mut` form. Not wrong, just extra. Leaving it does cleanly document the regression intent (no spurious `mut`), so OK to keep.

4. **Plan-doc PR link for Wave 2.3 references `pull/2564`, while `codegen-test-triage.md` also lists `pull/2564` for Wave 2.3** — consistent within this diff, but worth double-checking the actual merged PR number against GitHub before merging Wave 2.4 (this diff alone doesn't expose a mismatch).

## Review-question answers

1. **Scope correctness.** `jq` confirms exactly 6 rows with `proposed_pr_slice == "2.4"` flipped to `closed` (`closes_in_subwave: "4"`) and 4 rows with `proposed_pr_slice == "2.5"` remaining `open` (`closes_in_subwave: "5"`). Totals: 48 closed / 4 open / 52 total. No misclassification.

2. **Field-assignment behavior.** `object == "self"` still returns `None` to keep the structured emitter responsible for boxing/option adaptations; this matches the unchanged regression `does_not_lower_field_assign_on_self_target` at `core_and_tuple_tests.rs:233-242`. Non-self path uses `try_lower_leaf_or_name_expr` (rejects non-leaf RHS — confirmed by `does_not_lower_field_assign_with_non_leaf_value` at line 244-258). Correct for the targeted contract; see Nit 1 for the type-adaptation caveat.

3. **Mutation-aware option pattern.** Both simple (`lower_stmt/condition_lowering.rs:17-23,110-116`) and structured (`stmt_support_emitter/condition_lowering.rs:144-150`) paths now key off `mutated_vars`. `collect_mutated_vars` in `hir_analysis/queries/queries_impl.rs:300-461` covers Assign / AugAssign / mutating method calls / Subscript/Field assigns / Delete / async-with — comprehensive enough to be a sound proxy for "needs `mut` on the unwrapped binding." For the `break_guard_unwraps_optional_tuple_before_indexing` fixture, `pair` is introduced via `HirStmt::Let` (not `Assign`) and only read via `pair[1]`, so it correctly stays out of `mutated_vars` and the new `Some(pair)` pattern is emitted.

4. **Performance-test assertion update.** Correct. In the Wave 2.4 source the unwrapped `pair` is read-only (`previous_index = pair[1]`); the prior `Some(mut pair)` was a spurious mut from the inverse-of-borrowed heuristic. The new assertion pins both the desired shape and the absence of `mut`, which is the actual non-weakening (Nit 3 notes the redundancy but not weakening).

5. **Inventory and tracking docs.** `red_blocker.failure_count: 4` and `test_result: 703/4/707` match the reported local run. Wave 2.3 status moved to "merged in PR #2564" and Wave 2.4 added with `locally validated; review and PR pending`. `codegen-test-triage.md` updates the running per-wave progression consistently. The phase plan's prose ("close all 6 structured lowering compiler-bug rows") matches the JSON closure pattern.

## Verification notes

- `git diff --stat` matches the seven files declared in scope; no collateral edits.
- `jq` over `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: 6 closed at `proposed_pr_slice == "2.4"`, 4 open at `proposed_pr_slice == "2.5"`, totals 48/4/52 — consistent with the updated `red_blocker.failure_count` and `test_result`.
- Spot-checked all six Wave 2.4 regression sites (`lowers_simple_field_assign_for_non_self_target`, the five `return_assert_if_tests` option-narrowing tests) against the new lowering paths — assertions align with what the modified code now emits given empty `mutated_vars` / `borrowed_params`.
- Did not re-run cargo tests or the verification harness; relied on the user-supplied local run logs (six target tests pass, suite at 703/4/707, `create-pr` profile green after transient timeout).
- Untracked `plans/reviews/active/ad-hoc-world-class-verification-wave-2-4-review-pass-1.md` is a zero-length placeholder — not part of this scope, no action needed.
