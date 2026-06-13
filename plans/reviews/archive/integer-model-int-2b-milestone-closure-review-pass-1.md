# INT-2B Milestone Closure — Review Pass 1

Reviewer: Claude Opus 4.7
Date: 2026-05-06
Branch under review: `main` at `95cf5e67` (post PR #1815, all INT-2B child bullets ticked)
Phase tracker: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
Canonical design: [internal_docs/integer_model.md](internal_docs/integer_model.md)
Scope of this pass: INT-2B (HIR, type system, and const fitting) — milestone closure readiness only.

## Verdict

**Ready to close INT-2B with non-blocking follow-ups.** Every stated INT-2B scope item, acceptance criterion, and validation entry on the tracker resolves to working compiler-owned behavior. The single open INT-1 breadcrumb (codegen wiring of in-budget >`i64` `int` module constants through `SifrInt`) is correctly scoped under INT-1, not INT-2B, and is the only place where an INT-2B-touched source surface still has a downstream gap. Several non-blocking gaps are listed in §"Non-blocking follow-ups" below; none of them are stated INT-2B requirements.

The argument for closure: every line of the INT-2B "Scope", "Acceptance criteria", and "Validation" sections in [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:148](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:148) is satisfied by code in `crates/sifr_hir`, `crates/sifr_type_system`, `crates/sifr_driver/src/project`, and `crates/sifr_driver/src/stdlib`. The argument against closure: certain validation-list items ("no implicit narrowing in returns / list literals / dict literals / generic specialization") rely on the underlying type-system reject path rather than dedicated negative tests. That coverage gap is real but does not block closure under the stated acceptance criteria — see §N1 for the recommended follow-up.

---

## Scope reviewed

I read each INT-2B child entry in the tracker against the merged compiler state, the canonical design doc, and existing HIR/driver/codegen sources. I also re-read the most recent INT-2B sub-PR review ([reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md)) to confirm pass-3/pass-4 blockers are closed and to inherit pass-4 follow-ups that the closure decision must take a stance on.

Files cross-referenced:

- [crates/sifr_type_system/src/types.rs](crates/sifr_type_system/src/types.rs:120)
- [crates/sifr_type_system/src/infer.rs](crates/sifr_type_system/src/infer.rs:27)
- [crates/sifr_hir/src/lower/typing_and_functions.rs:412](crates/sifr_hir/src/lower/typing_and_functions.rs:412)
- [crates/sifr_hir/src/lower/typevar_annotations.rs](crates/sifr_hir/src/lower/typevar_annotations.rs)
- [crates/sifr_hir/src/lower/fixed_width_fitting.rs](crates/sifr_hir/src/lower/fixed_width_fitting.rs)
- [crates/sifr_hir/src/lower/module_constants_lowering.rs](crates/sifr_hir/src/lower/module_constants_lowering.rs)
- [crates/sifr_hir/src/lower/integer_literal_diagnostics.rs](crates/sifr_hir/src/lower/integer_literal_diagnostics.rs)
- [crates/sifr_hir/src/lower/imports.rs:104](crates/sifr_hir/src/lower/imports.rs:104)
- [crates/sifr_hir/src/lower/mod.rs:455](crates/sifr_hir/src/lower/mod.rs:455)
- [crates/sifr_diagnostics/src/codes.rs:62](crates/sifr_diagnostics/src/codes.rs:62)
- [crates/sifr_driver/src/project/exports.rs](crates/sifr_driver/src/project/exports.rs)
- [crates/sifr_driver/src/project/frontend.rs](crates/sifr_driver/src/project/frontend.rs)
- [crates/sifr_driver/src/stdlib/bootstrap.rs](crates/sifr_driver/src/stdlib/bootstrap.rs)
- [crates/sifr_codegen/src/module_constants.rs](crates/sifr_codegen/src/module_constants.rs)
- [crates/sifr_codegen/src/lower_item.rs](crates/sifr_codegen/src/lower_item.rs)
- [crates/sifr_codegen/src/lower_expr.rs:36](crates/sifr_codegen/src/lower_expr.rs:36)

I did not re-run `scripts/run_all_tests.sh`; the closure decision is grounded in the merged code state plus the per-slice review trail (PR #1795–#1815) where each child PR was satisfied at quick-validation level.

---

## Acceptance-criterion-by-criterion verdict

### 1. "Unsuffixed literals infer as `int`"

**Met.** [crates/sifr_type_system/src/infer.rs:6](crates/sifr_type_system/src/infer.rs:6) maps `LiteralKind::Int` to `Type::Int`. The literal-budget visitor at [crates/sifr_hir/src/lower/integer_literal_diagnostics.rs:11](crates/sifr_hir/src/lower/integer_literal_diagnostics.rs:11) treats integer tokens as `int`-typed expressions; the only refinement is the `Type::LiteralInt(i64)` carry for small in-`i64` literals, which is `Type::Int.is_assignable_to(...)`-equivalent at every relevant boundary ([crates/sifr_type_system/src/types.rs:1239](crates/sifr_type_system/src/types.rs:1239)).

### 2. "`x: int = 10 ** 100` type-checks"

**Met at HIR/type-check.** Local-scope `x: int = 10 ** 100` parses, lowers to a BinOp of `IntLiteral(10)` and `IntLiteral(100)` typed `Type::Int`, and is accepted by the assignment checker because `Type::Int` is assignable to `Type::Int`. Module-scope `BIG: int = 10 ** 100` lowers via [lower_module_integer_const_expr](crates/sifr_hir/src/lower/module_constants_lowering.rs:98), is folded by [const_integer_value](crates/sifr_hir/src/lower/fixed_width_fitting.rs:95) → `BigInt(10^100)` (101 decimal digits, well under the 4096-digit budget), and is recorded in `ctx.const_integer_values` for downstream import fitting. `sifr check` accepts both shapes.

A downstream codegen panic still exists for module-scope `int` constants that survive HIR as `LargeIntLiteral` or non-leaf BinOps (see §B1 below); that is correctly flagged on the tracker as the open INT-1 breadcrumb at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425) and is out of INT-2B scope.

### 3. "`x: uint8 = 255` type-checks; `x: uint8 = 256` and `x: uint8 = -1` are compile errors with range diagnostics"

**Met.** [validate_fixed_width_initializer](crates/sifr_hir/src/lower/fixed_width_fitting.rs:16) compares the const-folded value to `fixed_range(U8)` and emits `DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE` (`SIFR-INT-0001`) with the expected `"integer value 256 does not fit target type uint8; valid range is 0..=255"` shape. Pinned by:

- [test_fixed_width_literal_assignment_fits](crates/sifr_hir/src/lower/expressions_tests.rs:212)
- [test_fixed_width_literal_assignment_out_of_range_has_int_code](crates/sifr_hir/src/lower/expressions_tests.rs:239)
- E2E pass: [crates/sifr/tests/e2e/pass/fixed_width_literal_assignment.sifr](crates/sifr/tests/e2e/pass/fixed_width_literal_assignment.sifr)
- E2E fail: [crates/sifr/tests/e2e/fail/fixed_width_literal_out_of_range.sifr](crates/sifr/tests/e2e/fail/fixed_width_literal_out_of_range.sifr) (now anchored by canonical top-level `expect-error` markers per PR #1812)

### 4. "`x: uint8 = 10 ** 5000` … fails with `SIFR-INT-0004`"

**Met.** [evaluate_pow](crates/sifr_hir/src/lower/fixed_width_fitting.rs:218) and [evaluate_left_shift](crates/sifr_hir/src/lower/fixed_width_fitting.rs:195) short-circuit via `MAX_EXACT_SHIFT_OR_EXPONENT = 13_610` and emit `INT_EVAL_BUDGET_EXCEEDED` (`SIFR-INT-0004`). [reject_if_over_budget](crates/sifr_hir/src/lower/fixed_width_fitting.rs:255) catches results that fit within the per-step approximation but exceed the 4096-decimal-digit final budget. Pinned by:

- [test_fixed_width_const_expression_budget_has_int_code](crates/sifr_hir/src/lower/expressions_tests.rs:379)
- [test_module_fixed_width_const_expression_budget_has_int_code_once](crates/sifr_hir/src/lower/expressions_tests.rs:451) (PR #1814; checks single-emission)
- [test_fixed_width_over_budget_literal_diagnostic_is_not_duplicated](crates/sifr_hir/src/lower/expressions_tests.rs:508)
- E2E fail: [crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr](crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr) (`SIFR-INT-0001` for `2 ** 8` into `uint8`, `SIFR-INT-0004` for `10 ** 5000`)

### 5. "No implicit narrowing in assignments, calls, returns, list literals, dict literals, or generic specialization"

**Met behaviorally; partially met by explicit negative tests.**

The behavioral rejection mechanism is `Type::Int.is_assignable_to(Type::FixedInt(_))` returning `false` ([crates/sifr_type_system/src/types.rs:1238-1243, 1382](crates/sifr_type_system/src/types.rs:1238)), which gates every source construct that boils down to an assignability check. `validate_annotated_constant_initializer` ([crates/sifr_hir/src/lower/fixed_width_fitting.rs:49](crates/sifr_hir/src/lower/fixed_width_fitting.rs:49)) is the const-fitting *override* that lets compile-proven constants pass into fixed-width *assignment* and *module-constant* slots only — it is intentionally not wired into call-arg / return / list-elt / dict-elt / generic-spec lowering paths, which means those surfaces correctly fall through to the type-mismatch rejection.

Pinned negative tests:

- Assignments: [test_fixed_width_assignment_from_non_const_int_is_still_mismatch](crates/sifr_hir/src/lower/expressions_tests.rs:536) and [test_fixed_width_assignment_from_non_const_binop_is_still_mismatch](crates/sifr_hir/src/lower/expressions_tests.rs:549).
- Calls: [test_fixed_width_call_argument_literal_is_not_implicitly_narrowed](crates/sifr_hir/src/lower/expressions_tests.rs:566).

Not directly pinned (but rejected by the underlying type system): returns, list literals, dict literals, and generic specialization. See §N1 for the follow-up — this is a coverage gap, not a behavior bug.

### 6. "`bigint` is gone from public docs/tests or emits intentional `SIFR-INT-0011` transition diagnostics only"

**Met via the second arm of the disjunction.** Every `bigint` user surface emits `INT_BIGINT_TRANSITION_ALIAS` (severity Warning; [crates/sifr_diagnostics/src/codes.rs:65](crates/sifr_diagnostics/src/codes.rs:65)). The emit sites cover:

- annotation name resolution: [crates/sifr_hir/src/lower/typing_and_functions.rs:439](crates/sifr_hir/src/lower/typing_and_functions.rs:439)
- `bigint(...)` constructor calls: [crates/sifr_hir/src/lower/expressions.rs:857](crates/sifr_hir/src/lower/expressions.rs:857)
- `isinstance(value, bigint)`: [crates/sifr_hir/src/lower/builtin_calls.rs:931](crates/sifr_hir/src/lower/builtin_calls.rs:931)
- `TypeVar(...)` positional/keyword bound and constraint forms: [crates/sifr_hir/src/lower/typevar_annotations.rs](crates/sifr_hir/src/lower/typevar_annotations.rs)
- PEP 695 function and class bounds with single-emission for class bounds (PR #1800)

The 11 dedicated tests at [crates/sifr_driver/src/tests/single_file_frontend.rs:255-388](crates/sifr_driver/src/tests/single_file_frontend.rs:255) pin every surface and the `Severity::Warning` invariant. The remaining `bigint`-using e2e fixtures in `crates/sifr/tests/e2e/{pass,fail}/` are intentional transition fixtures: each emits `SIFR-INT-0011` warnings rather than a fatal diagnostic, so the validation gate "emits intentional transition diagnostics only" is satisfied. The auto-generated [docs/errors/SIFR-INT-0011.md](docs/errors/SIFR-INT-0011.md) and [docs/errors/SIFR-TYPE-0006.md](docs/errors/SIFR-TYPE-0006.md) discuss `bigint` only in the context of the transition policy and the legacy mixed-arithmetic error — the surrounding diagnostic-codes index makes the deprecation explicit. There is no positive recommendation of `bigint` as a long-term type in any current public doc.

Eventual `bigint` removal is INT-7 cleanup ([issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330)), not INT-2B.

### 7. Reserved `int128` / `uint128`

**Met.** [crates/sifr_hir/src/lower/typing_and_functions.rs:435](crates/sifr_hir/src/lower/typing_and_functions.rs:435) emits `DiagnosticCode::INT_RESERVED_WIDTH_NAME` (`SIFR-INT-0003`) only after type-var, type-alias, and class lookups fail. The shadowing policy is documented at [internal_docs/integer_model.md:69](internal_docs/integer_model.md:69) (PR #1810). Pinned by:

- [test_reserved_integer_width_annotations_have_int_code](crates/sifr_hir/src/lower/type_alias_tests.rs:113)
- [test_nested_reserved_integer_width_annotations_have_int_code](crates/sifr_hir/src/lower/type_alias_tests.rs:138)
- E2E fail: [crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr](crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr) (PR #1806)

### 8. Imported immutable module constants carry const-evaluable values

**Met.** The export side runs through [collect_module_exports](crates/sifr_driver/src/project/exports.rs:6) and emits `external_defs.constant_integer_values[module][name]` for non-`_`-prefixed module constants whose `LoweringResult.constant_integer_values` map carries a folded value. The import side at [crates/sifr_hir/src/lower/imports.rs:104-118](crates/sifr_hir/src/lower/imports.rs:104) re-hydrates the BigInt into the importer's `ctx.const_integer_values` so subsequent fixed-width fitting in that importer sees the value through `validate_fixed_width_initializer`. The stdlib bootstrap mirrors the same surface at [crates/sifr_driver/src/stdlib/bootstrap.rs:147-153,336-340](crates/sifr_driver/src/stdlib/bootstrap.rs:147) using the public-only filter `collect_public_constant_integer_value_exports`.

Pinned by:

- Import propagation: [test_project_lowering_fits_imported_integer_constants](crates/sifr_driver/src/tests/project_graph.rs:602)
- Shadow rejection: [test_project_lowering_does_not_fold_shadowed_imported_integer_constant](crates/sifr_driver/src/tests/project_graph.rs:645)
- Stdlib end-to-end: [stdlib_integer_constants_fold_in_project_fixed_width_initializers](crates/sifr_driver/src/tests/stdlib_exports.rs:24) (real `sifr.logging.DEBUG` folding into `uint8` initializer)
- Same-module reuse: [test_module_constant_export_uses_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:399), [test_module_constant_export_uses_unary_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:413), and [test_module_constant_export_does_not_retype_fixed_width_name_as_int](crates/sifr_hir/src/lower/expressions_tests.rs:430).
- Documented "no transitive re-export" semantics at [internal_docs/integer_model.md:105](internal_docs/integer_model.md:105) (PR #1808).

### 9. HIR maintainability guardrails

**Met.** `python3 scripts/check_hir_maintainability_guardrails.py` reports `PASS`. The new lowering surface for fixed-width fitting and module-constant integer folding lives in dedicated small files ([fixed_width_fitting.rs](crates/sifr_hir/src/lower/fixed_width_fitting.rs) at 310 lines and [module_constants_lowering.rs](crates/sifr_hir/src/lower/module_constants_lowering.rs) at 142 lines), so the cap-per-file table in [scripts/check_hir_maintainability_guardrails.py](scripts/check_hir_maintainability_guardrails.py) did not need to grow to accommodate INT-2B's surface area.

---

## Open INT-1 breadcrumb scope assessment

The tracker's only open child item under any milestone touched by INT-2B is at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425), under INT-1:

> Wire module-level `int` constants whose in-budget values exceed `i64` through `SifrInt` codegen, removing the current module-constant production panic path tracked by the INT-2B module const/fixed-width fallback cleanup review.

The breadcrumb is **correctly scoped under INT-1, not INT-2B**:

- INT-1 owns runtime `SifrInt` and codegen wiring of generated Rust to `SifrInt` ([issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:92-122](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:92)). The current panic path is in `crates/sifr_codegen` ([crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12)), not `crates/sifr_hir` or `crates/sifr_type_system`.
- INT-2B owns "represent exact integer literals, fixed-width families, and const fitting in compiler-owned IR/type layers" ([issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:148-178](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:148)). The HIR layer correctly produces `LargeIntLiteral` / `BinOp{Int}` / `Type::Int` HIR for in-budget large `int` values; the value is also threaded into `LoweringResult.constant_integer_values` for cross-module fitting. Every INT-2B surface is downstream-decoupled from how codegen materializes the runtime representation.
- The INT-2B sub-PR review explicitly named this as an INT-1 item: pass-4 N4 in [reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md:99-100) flagged it "out of scope for this slice (tied to SifrInt wiring under INT-1/INT-3 wave 2), but worth tracking explicitly."

**Closure call:** the breadcrumb does not block INT-2B closure. The user-visible behavior is consistent with INT-2B's stated acceptance: `sifr check x: int = 10 ** 100` succeeds (which is what acceptance criterion #2 requires); the codegen-side panic that follows from `sifr build`/`sifr run` on a module-level large-`int` constant is INT-1's responsibility to retire when codegen migrates to `SifrInt`. See §B1 for the exact reproduction shapes a future INT-1 slice should use as its closure tests.

---

## Non-blocking follow-ups

These do not block INT-2B closure under the stated acceptance criteria, but each is something a future slice (INT-1 codegen wiring, INT-3 arithmetic, INT-7 cleanup) should pick up.

### N1 — No dedicated negative tests for implicit narrowing in returns / list literals / dict literals / generic specialization

INT-2B's validation list at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:170-178](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:170) calls for "Negative tests for implicit narrowing in every source construct listed above." The acceptance criterion lists six surfaces (assignments, calls, returns, list literals, dict literals, generic specialization). Today's HIR tests pin assignments and calls only ([test_fixed_width_assignment_from_non_const_int_is_still_mismatch](crates/sifr_hir/src/lower/expressions_tests.rs:536), [test_fixed_width_call_argument_literal_is_not_implicitly_narrowed](crates/sifr_hir/src/lower/expressions_tests.rs:566)). The other four surfaces are correctly rejected by the general `is_assignable_to(Int, FixedInt) -> false` rule, but the negative coverage is implicit rather than explicit.

This is a coverage gap, not a behavior bug. Recommended one-slice follow-up adds four small HIR (or single-file driver) tests:

- `def f() -> uint8: source: int = 1; return source` → expect `TYPE_MISMATCH` "expected 'uint8', got 'int'"
- `def main(): source: int = 1; xs: list[uint8] = [source]` → expect `TYPE_MISMATCH` (list-element conflict or top-level list-of-int vs list-of-uint8)
- `def main(): source: int = 1; ds: dict[str, uint8] = {"a": source}` → same shape
- a generic specialization shape, e.g. `def f[T](x: T) -> T: return x` then `value: uint8 = f(1)` (the latter should reject because the inferred `T` is `LiteralInt(1) | Int`, not `uint8`).

The validation gate is otherwise clear, so this is a hardening step rather than a closure blocker.

### N2 — No regression test that `class int128:` / `type int128 = ...` shadow the reserved-width diagnostic

[Pass-1 review O3 of the shadowing-policy slice](reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md) flagged this. The shadowing policy is documented at [internal_docs/integer_model.md:69](internal_docs/integer_model.md:69) and the implementation at [crates/sifr_hir/src/lower/typing_and_functions.rs:420-447](crates/sifr_hir/src/lower/typing_and_functions.rs:420) does the right thing (type-vars / aliases / class types resolve before the reserved-width gate fires), but no current test pins the positive shadow case. A one-liner adding `class int128: pass` followed by `value: int128 = int128()` and expecting no `SIFR-INT-0003` would close this. Out of scope for the INT-2B closure decision; appropriate for an INT-3 / INT-7 follow-up if reserved-identifier policy gets tightened later.

### N3 — Codegen panic path for in-budget >`i64` `int` module constants

Tracked at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425) as the open INT-1 item. See §B1 below for reproduction shapes.

### N4 — `bigint` e2e fixtures emit cumulative `SIFR-INT-0011` warning noise

Roughly 18 e2e pass/fail fixtures still use `bigint` annotations or constructors. Each is intentional transition coverage and emits `SIFR-INT-0011` warnings on every parse. The acceptance gate is met (severity is Warning, not Error, and the per-surface unit tests pin single-emission), but the fixtures will still produce warnings until INT-7 cleanup retires them. This is a known cost of the transition policy, not a closure blocker; INT-7 should sweep these together with the public-doc cleanup at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:330).

---

## Reproduction shapes for future INT-1 slice (informational)

Documented here so the eventual codegen slice has a precise closure target. Not load-bearing for the INT-2B closure decision.

### B1 — Module-level `int` constant codegen panic shapes

Both shapes pass HIR and panic at codegen. The HIR behavior is correct per INT-2B's scope; codegen wiring is INT-1's responsibility.

```sifr
# Shape A: bare LargeIntLiteral above i64
LIMIT: int = 999999999999999999999999999999999999

def main():
    print(str(LIMIT))
```

Trace: parser → `LargeIntLiteral("999...")` (PR #1792); HIR `lower_module_integer_const_expr` falls through to `lower_integer_const_expr_simple` ([crates/sifr_hir/src/lower/module_constants_lowering.rs:130](crates/sifr_hir/src/lower/module_constants_lowering.rs:130)) which returns the `LargeIntLiteral` directly. Codegen `try_lower_simple_module_constant_item_result_impl` ([crates/sifr_codegen/src/lower_item.rs:81](crates/sifr_codegen/src/lower_item.rs:81)) takes the `Type::Int` primitive arm; `fixed_width_literal_expr_for_target(Type::Int, …)` returns `None` (target is not `FixedInt`); `try_lower_leaf_or_name_expr_result` calls `try_lower_leaf_expr(LargeIntLiteral)` which has no arm for `LargeIntLiteral` ([crates/sifr_codegen/src/lower_expr.rs:124](crates/sifr_codegen/src/lower_expr.rs:124)) and returns `None`; the outer `else` returns `Ok(None)`; module_constants emit panics with "unsupported module constant lowering shape".

```sifr
# Shape B: BinOp evaluating to >i64 even though operand literals are small
BIG: int = 10 ** 100

def main():
    print(str(BIG))
```

Trace: HIR records `BinOp{IntLiteral(10), "**", IntLiteral(100)}` typed `Type::Int` and stores `BigInt(10^100)` in `ctx.const_integer_values["BIG"]`. Codegen `try_lower_leaf_expr(BinOp)` enters [the BinOp arm](crates/sifr_codegen/src/lower_expr.rs:190); `is_safe_simple_binop("**", Int, Int, Int)` returns `false` ([crates/sifr_codegen/src/lower_expr.rs:1503-1518](crates/sifr_codegen/src/lower_expr.rs:1503) — `**` is not in the safe-op set), so the BinOp arm yields `None`; same final panic.

The expected INT-1 fix routes `Type::Int` module constants through `SifrInt`-backed initialization (e.g., `static BIG: SifrInt = SifrInt::from_decimal_str("10000000...");` or a `LazyLock<SifrInt>` that calls into `sifr_runtime`). The existing `LoweringResult.constant_integer_values[name]` already provides the canonical decimal payload codegen needs. Codegen should use that map (already imported via `external_defs.constant_integer_values`) instead of re-evaluating from the BinOp HIR.

### B2 — Cross-module integer-value re-export semantics (informational)

Documented at [internal_docs/integer_model.md:105](internal_docs/integer_model.md:105) and pinned by [test_project_lowering_fits_imported_integer_constants](crates/sifr_driver/src/tests/project_graph.rs:602). Importing `from a import LIMIT` into module `b`, then re-exporting via `from a import LIMIT` in `c` and using it as a fixed-width fitting value works only when `c` imports directly from the defining module `a`. The INT-2B doc text takes the explicit "no transitive re-export of imported const values" stance — implementations should not need to walk the module graph for this. No follow-up needed; the policy is documented and tested.

---

## Validation matrix (cross-check against design doc §"Validation Matrix")

| Area | Coverage |
| --- | --- |
| Type inference: `list[int]`, `list[int32]`, contextual fixed-width literals | Inference works for `list[int]`; literal contextual fitting is wired only for `assignment` and `module-const` slots, deliberately not for list-element-of-fixed-width slots. The design example `d: list[int32] = [1, 2, 3]` is *not* yet a positive case in the implementation — the negative side is enforced by the general type system. This is consistent with INT-2B's stated scope (`literals, unary signs, basic integer arithmetic, shifts, …, immutable module constants`); container element fitting is not in the INT-2B scope text. Flag this for INT-3 / INT-4 follow-up if the team wants the design's `list[int32] = [1, 2, 3]` ergonomic. |
| Fixed-width scalars: fitting literals, fallible constructors, checked/wrapping/saturating APIs | Fitting literals: covered. Fallible constructors / checked-wrapping-saturating APIs: scoped to INT-1B (PR #1790) and INT-3 (not yet started). |
| Type inference: `T + T -> T` with fixed-width | INT-3 scope, not INT-2B. |
| Generic specialization narrowing | Behaviorally rejected; not pinned by a dedicated test. See §N1. |

---

## Final verdict

**Ready to close INT-2B with non-blocking follow-ups.**

INT-2B's stated scope, every acceptance criterion in [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:162-168](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:162), and the validation entries in [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:170-178](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:170) resolve to working, tested compiler behavior. The 14 INT-2B child PRs (#1795–#1814) are each individually review-satisfied, and the most recent pass-4 review's blockers are closed. The single open INT-1 breadcrumb at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425) is correctly scoped under INT-1 (codegen wiring of `SifrInt`-backed module constants), not INT-2B (HIR/type system/const fitting), and does not block INT-2B closure under the stated criteria. Non-blocking follow-ups N1–N4 above are tracking-only suggestions for the next slices.

Recommended action: tick the INT-2B parent line at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:431](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:431); add this review to the Review History section; carry N1 (or split it into per-construct sub-items) into INT-3 / INT-4 work; carry N3 into the existing INT-1 breadcrumb at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:425); and run the canonical `scripts/run_all_tests.sh` (full profile) closure validation per [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:16](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:16) before ticking the parent.
