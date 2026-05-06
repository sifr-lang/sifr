# INT-2B fixed-width const literal fitting — review pass 1b

Branch: `int-2b-fixed-width-const-fitting`
Local validation: `scripts/run_all_tests.sh --profile quick` reported `report_signature=e1bf653aaa770517`.

## Scope fit

In scope and implemented:
- `crates/sifr_hir/src/lower/fixed_width_fitting.rs:8` — `validate_fixed_width_initializer` only fires when target resolves (alias-aware) to `Type::FixedInt(_)` and value is an integer literal / large integer literal / unary `+|-` literal; otherwise returns `None` to leave existing checks untouched.
- `crates/sifr_hir/src/lower/statements.rs:1057` — annotated assignment guards the legacy `TYPE_MISMATCH` branch with `fixed_width_fit.is_none()`, so a `Some(false)` from the fitter suppresses the duplicate generic mismatch.
- `crates/sifr_hir/src/lower/mod.rs:1106` — module-level annotated constants use `validate_annotated_constant_initializer`, which calls the same fitter and short-circuits on `Some(_)`.
- Codegen suffixing for fitting literals lands in three call sites: `lower_stmt.rs:4210` (simple let initializer), `stmt_support_emitter.rs:297` (local-coercion path), and `lower_item.rs:88` (module constant Result dispatcher).
- New diagnostic `SIFR-INT-0001` registered in `crates/sifr_diagnostics/src/codes.rs:62,744` with template/owner/representative fixture, plus matching `docs/errors/SIFR-INT-0001.md`, `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md` rows.
- E2E fixtures: `crates/sifr/tests/e2e/pass/fixed_width_literal_assignment.sifr` (uint8 max, int8 min, uint64 max) and `crates/sifr/tests/e2e/fail/fixed_width_literal_out_of_range.sifr` (col=23, col=32 for `256` and `-1`).
- Display name now formats `Type::FixedInt` in HIR diagnostic messages (`crates/sifr_hir/src/lower/diagnostics.rs:48`).

Out of scope and *correctly* not exercised:
- Function call arguments still go through plain `is_assignable_to` ([expressions.rs:1468](crates/sifr_hir/src/lower/expressions.rs:1468)); the unit test `test_fixed_width_call_argument_literal_is_not_implicitly_narrowed` pins this behavior.
- No const evaluator, arithmetic/shift/pow folding, return-position narrowing, or container-element fitting was added.

## Correctness review

**Statement-level branching is sound.** `lower_ann_assign` now layers `is_int_to_bigint`, `fixed_width_fit`, and `is_assignable_to` so that:
- fitting literal → suppress generic `TYPE_MISMATCH` (allows int → fixed-width);
- out-of-range literal → `SIFR-INT-0001` only, no duplicate `TYPE_MISMATCH` (asserted by `test_fixed_width_literal_assignment_out_of_range_has_int_code`);
- non-const int into fixed-width → fitter returns `None`, falls through to `TYPE_MISMATCH` (asserted by `test_fixed_width_assignment_from_non_const_int_is_still_mismatch`).

**`const_integer_value` and `fixed_range`** correctly normalize unary `+|-` and parse `LargeIntLiteral` as `BigInt`. Negative `LargeIntLiteral` is reachable only via `HirExpr::UnaryOp { op: "-", operand: LargeIntLiteral }` (see `classes.rs:1282`), and the recursive `op == "-"` arm handles that shape.

**Codegen suffixing.** `fixed_width_literal_expr_for_target` emits e.g. `255u8`, `-128i8`, `18446744073709551615u64`. Verified that `rustc` accepts `-128i8` (`MIN` corner case is special-cased — only `-129i8` triggers `overflowing_literals`). The fitter blocks anything else from reaching codegen, so the codegen path never has to emit literals outside the suffixed type's range.

**Alias handling.** Both validator (`target.resolve_alias()` in `fixed_width_fitting.rs:14`) and codegen (`crate::resolve_alias_type_for_plain_call` in `lower_expr.rs:37`) traverse `Type::Alias`, matching the e2e/HIR path used for typedefs.

**Module-constant emission path.** Production lowering goes through `try_lower_simple_module_constant_item_result` (see `module_constants.rs:27`), which is the variant updated with `fixed_width_literal_expr_for_target`. The non-`_result` `try_lower_simple_module_const_item` at `lower_item.rs:189` is *not* updated, but tests are the only callers — consider a brief follow-up to either route those tests through the result variant or mirror the helper, to avoid a future drift hazard. Not a regression today.

## Findings

1. **Scope creep — module-constant generic mismatch (low risk, arguably a fix).** `validate_annotated_constant_initializer` ([fixed_width_fitting.rs:50](crates/sifr_hir/src/lower/fixed_width_fitting.rs:50)) now emits `TYPE_MISMATCH` for module constants whose value type is unassignable to the annotation (e.g. `LIMIT: int = "x"`). Previously this branch in `lower_module_impl` performed no type check at all. The new check is desirable, but the task description ("fixed-width fitting only…") doesn't enumerate it. No existing pass fixture trips on it (`module_constants.sifr` uses matching types), and `quick` validated clean. Worth a one-line callout in the PR description so reviewers expect the behavior change.

2. **Missing fixture: module-level out-of-range fail.** Unit test `test_fixed_width_module_constant_out_of_range_has_int_code` covers HIR, but `crates/sifr/tests/e2e/fail/fixed_width_literal_out_of_range.sifr` only exercises function-body sites. Adding a top-level case (e.g. `TOO_HIGH: uint8 = 256` at module scope) would lock in the module path through the e2e harness, mirroring the function-level coverage. Same-scope, low-effort.

3. **Missing fixture: module-level fitting emission.** `fixed_width_literal_assignment.sifr` only has function-body bindings. The codegen path `lower_item.rs:88` (suffixing for module-level fixed-width constants) is exercised only via unit tests — an e2e fixture like `LIMIT: uint8 = 255` consumed inside `main()` would round-trip the suffixed Rust output through the compiler and runtime. Optional but cheap.

4. **Pre-existing gap (out of scope, flagging only).** `lower_expr_simple` ([classes.rs:1277](crates/sifr_hir/src/lower/classes.rs:1277)) only collapses `UnaryOp::USub`, so `LIMIT: uint8 = +255` at module level silently fails to lower the value and the constant is dropped without a diagnostic. The fitter would happily accept it; the AST shape just never reaches the fitter at module scope. Not introduced here, but worth a tracking note since the scope says unary `+` literal forms are accepted.

5. **Pre-existing portability nit (out of scope).** `fixed_range` for `ISize`/`USize` uses host `isize::MIN/MAX`/`usize::MIN/MAX`. On a 64-bit dev host, this lets values that overflow a 32-bit cross-compile target slip through the fitter. Out of scope for INT-2B; fine to defer.

6. **Naming / drift.** `try_lower_simple_module_const_item` (non-result) at `lower_item.rs:189` is no longer in the production hot path but still has parallel logic. Either gate its tests to use the `_result` form or mirror the fixed-width arm so the fallback stays honest if someone re-enables it. Optional.

## Tests reviewed

- `test_fixed_width_literal_assignment_fits` — covers `uint8`/`int8`/`uint64` HIR shapes including `UnaryOp("-")` for `int8 = -128` and `LargeIntLiteral` for u64::MAX.
- `test_fixed_width_literal_assignment_out_of_range_has_int_code` — pins message, code, and primary range for the four canonical out-of-range cases (256, -1, 128, -129) and asserts no shadow `TYPE_MISMATCH` is emitted.
- `test_fixed_width_module_constant_out_of_range_has_int_code` — covers module-level uint8 = 256.
- `test_fixed_width_assignment_from_non_const_int_is_still_mismatch` — pins that `target: uint8 = source` (non-const int) keeps `TYPE_MISMATCH` and primary range on the value.
- `test_fixed_width_call_argument_literal_is_not_implicitly_narrowed` — pins that `take(1)` against `uint8` parameter keeps `TYPE_MISMATCH`.
- `lowers_fixed_width_literal_for_target_type` — pins codegen `255u8`, `-128i8`, `18446744073709551615u64`.

Coverage matches the stated invariants. The non-const-int and call-argument tests are the important "guard rails" that prevent silent narrowing creep, and they're both present.

## Verdict

The implementation is tight, the diagnostic plumbing is complete, and the guard tests cover the key boundary cases. The scope creep noted in finding 1 is benign and arguably desirable; findings 2–3 are nice-to-have e2e coverage, not blockers; findings 4–6 are pre-existing or out-of-scope.

VERDICT: SATISFIED
