# INT-2B const expression fitting — review pass 2

Branch: `int-2b-const-expression-fitting`
Local validation reported by author: `scripts/run_all_tests.sh --profile quick` `report_signature=e1bf653aaa770517` (102.26s).

## Summary

The four pass-1c findings flagged for action (F1, F2, F3, and the F6 coverage gaps) are addressed correctly. The fixes are minimal, reuse existing infrastructure, and are pinned by new unit tests. F4 (module-level remember-path SIFR-INT-0004 from `int` annotations), F5 (module-level Name/unary-on-non-literal not lowered by `lower_integer_const_expr_simple`), and F7 (folding bypasses Name-shape ownership tracking) are unchanged and remain documented as non-blocking by pass 1c.

## Pass-1c follow-up verification

### F1 — Scope-blind module-constant lookup → fixed

[`fixed_width_fitting.rs:109-118`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:109) now guards the `Name` arm with `is_shadowed_by_inner_scope`:

```rust
HirExpr::Name { name, .. } => {
    if is_shadowed_by_inner_scope(ctx, name) {
        return ConstIntegerValue::Unsupported;
    }
    return ctx.const_integer_values.get(name).cloned()
        .map_or(ConstIntegerValue::Unsupported, ConstIntegerValue::Value);
}
```

[`fixed_width_fitting.rs:239-246`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:239) implements the helper exactly as suggested:

```rust
fn is_shadowed_by_inner_scope(ctx: &LowerCtx, name: &str) -> bool {
    let frame_count = ctx.scope.frame_count();
    frame_count > 1
        && ctx.scope.lookup_in_frame_range(name, 1, frame_count - 1).is_some()
}
```

Verified against the scope model:
- `frames[0]` is the module scope, populated by `module_constants_lowering::collect_*` via `ctx.scope.define`. Skipping frame 0 means a same-named module constant does not register as a shadow of itself.
- Function bodies push frame 1 via `ctx.enter_function_scope` ([`function_scopes.rs:11-17`](crates/sifr_hir/src/lower/function_scopes.rs:11)), and parameters land there via `define_parameter`. So a parameter named `BASE` is detected.
- `if`/`else`/`while`/`for`/comprehension bodies all push their own frames ([`statements.rs:1700,1741,1770,1988,2005,2109,2178`](crates/sifr_hir/src/lower/statements.rs:1700)), so for-loop targets and nested-block locals are also covered.
- `lookup_in_frame_range(_, 1, 0)` returns `None` (start > end), so the `frame_count > 1` short-circuit isn't strictly necessary for safety but keeps the call shape obvious.

The fold-vs-mismatch ordering is correct in the test scenario:
- `BASE: int = 100` inside `main()` is processed via `lower_ann_assign`, which lowers the value (no fold), then `scope.define_explicit_local("BASE", Int)` — frame[1] now contains BASE.
- The next statement `value: uint8 = BASE + 1` lowers `BASE` as `HirExpr::Name { name: "BASE", ty: Int }`. `validate_fixed_width_initializer` calls `const_integer_value`, which sees the frame[1] BASE, returns `Unsupported`, and the binop falls through to `NotConst`. The TYPE_MISMATCH emission covers the whole `BASE + 1` range, which is exactly what the test asserts.

`test_fixed_width_const_expression_does_not_fold_shadowed_module_constant` ([`expressions_tests.rs:328-356`](crates/sifr_hir/src/lower/expressions_tests.rs:328)) pins this with the reproducer from the pass-1c finding and additionally asserts that `INT_FIXED_WIDTH_OUT_OF_RANGE` is *not* emitted (which would have been the silent-wrong-answer signature: stale module BASE=254 would have folded to 255, fitting uint8 cleanly).

The fix is conservative: any locally-bound `BASE` (even a mutable one) blocks the fold. That's the right call — only the regular HIR pipeline knows whether the local is itself a constant, and the fitter shouldn't try to second-guess that.

### F2 — Duplicate `SIFR-INT-0004` for over-budget LargeIntLiteral → fixed

[`fixed_width_fitting.rs:98-108`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:98) now short-circuits before parsing:

```rust
HirExpr::LargeIntLiteral(value) => {
    if value.trim_start_matches('-').len()
        > super::integer_literal_diagnostics::INTEGER_EVAL_DECIMAL_DIGIT_BUDGET
    {
        return ConstIntegerValue::Rejected;
    }
    match value.parse() { ... }
}
```

Returning `Rejected` rather than `Unsupported` is the right choice here:
- `validate_fixed_width_initializer` propagates `Rejected → FixedWidthInitializerFit::Rejected`, which suppresses the legacy `TYPE_MISMATCH` in [`statements.rs:1057-1074`](crates/sifr_hir/src/lower/statements.rs:1057).
- The parser-level `validate_module_integer_literals` already emitted SIFR-INT-0004 (the AST visitor walks function bodies via `walk_stmt → visit_body`, so function-body literals are covered too). No second SIFR-INT-0004 is emitted.
- If the arm returned `Unsupported`, the user would see SIFR-INT-0004 *plus* TYPE_MISMATCH, which is louder than the original duplicate.

The remember-path is consistent: `remember_module_const_integer` only inserts on `ConstIntegerValue::Value`, so an over-budget literal isn't memoized but also doesn't re-emit.

`test_fixed_width_over_budget_literal_diagnostic_is_not_duplicated` ([`expressions_tests.rs:399-424`](crates/sifr_hir/src/lower/expressions_tests.rs:399)) asserts exactly one SIFR-INT-0004 (filter + `assert_eq!(len, 1)`) and no TYPE_MISMATCH for a 4097-digit literal in a uint8-annotated assignment. Direct pin.

### F3 — Misleading "1 decimal digits" in early shift/pow rejection → improved

[`fixed_width_fitting.rs:279-287`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:279) introduces conservative estimators:

```rust
fn approximate_left_shift_digits(left: &BigInt, shift: u32) -> usize {
    let bit_digits = u64::from(shift).saturating_mul(30_103) / 100_000 + 1;
    let bit_digits = usize::try_from(bit_digits).unwrap_or(usize::MAX);
    decimal_digit_count(left).saturating_add(bit_digits)
}

fn approximate_pow_digits(abs_left: &BigInt, exponent: u32) -> usize {
    decimal_digit_count(abs_left).saturating_mul(usize::try_from(exponent).unwrap_or(usize::MAX))
}
```

Spot-checks on the message values:
- `1 << 100_000` → `1 + 30_103 = 30_104` (vs. the pre-fix `1`).
- `2 ** 100_000` → `1 * 100_000 = 100_000` (actual ≈ 30_103). Conservative overestimate by ~3×.
- `9 ** 13_611` (just past the early threshold) → `1 * 13_611 = 13_611` (vs. actual ≈ 12_989). Slight overestimate, but >>4096 so the user reads "way over budget" loud and clear.

These are saturating to avoid overflow on very large `u32` operands, with `unwrap_or(usize::MAX)` guarding the cast for 32-bit hosts. No panic surface. The estimates are intentionally upper-bounds (so the user never sees a digit count that *understates* the magnitude). Net: the message is now informative instead of misleading.

The post-evaluation digit count in `reject_if_over_budget` is unchanged and continues to report the exact result digit count (`5001 decimal digits` for `10 ** 5000`, asserted by `test_fixed_width_const_expression_budget_has_int_code`).

### F6 — Test coverage gaps → addressed (mostly)

New coverage relative to pass 1c:

| Operator / case | Test |
|---|---|
| `<<`, `//`, `%` | `test_fixed_width_const_expression_assignment_fits_and_folds` ([`expressions_tests.rs:281-311`](crates/sifr_hir/src/lower/expressions_tests.rs:281)) — `(1 << 6) + (9 // 2) + (9 % 2)` folds to `IntLiteral(69)` |
| Negative const expressions | Same test — `-10 * 5 → IntLiteral(-50)`, `-(100 + 27) → IntLiteral(-127)` |
| Module-constant shadowing | `test_fixed_width_const_expression_does_not_fold_shadowed_module_constant` |
| Non-const operand in binop fallback | `test_fixed_width_assignment_from_non_const_binop_is_still_mismatch` ([`expressions_tests.rs:439-454`](crates/sifr_hir/src/lower/expressions_tests.rs:439)) |
| Duplicate over-budget literal | `test_fixed_width_over_budget_literal_diagnostic_is_not_duplicated` |
| E2E roundtrip of new operators | [`fixed_width_const_expression_assignment.sifr`](crates/sifr/tests/e2e/pass/fixed_width_const_expression_assignment.sifr) — `(1 << 6) + (9 // 2) + (9 % 2)`, `-10 * 5`, `-(100 + 27)` |

Two minor coverage residuals worth noting (neither is a blocker):

1. **No test pins zero-divisor short-circuit.** `// 0` and `% 0` in a fixed-width const expression return `Unsupported` from `evaluate_integer_binop` and fall through to `TYPE_MISMATCH` (since result type is Int and target is fixed-width). A test like `value: uint8 = 5 // 0` would assert TYPE_MISMATCH (not a runtime divide-by-zero diagnostic). Without it, a future change to that arm could silently regress to "fold to 0 via panic" or similar.
2. **No module-level fitting fixture.** The pass test exercises module→function const propagation, but `LIMIT: uint8 = 250 + 4` consumed only at module scope (e.g., feeding a class default through `lower_item.rs`) is not present in any e2e fixture. The existing unit test `test_fixed_width_module_constant_out_of_range_has_int_code` covers the rejection path, but the success path through `lower_item.rs:88` is unverified end-to-end. Consider adding when an INT-2C-or-later milestone exercises module-scope fixed-width state.

### F4 / F5 / F7 — unchanged (per pass 1c, non-blocking)

- **F4** (module-level `LIMIT: int = 10 ** 5000` emits SIFR-INT-0004 from the `remember_module_const_integer` path, while the same expression inside a function body with `int` annotation does not): still present. The author's response did not address this and pass 1c flagged it as defensible by spec — worth a note in the PR description.
- **F5** (module-level `Name`/unary-on-non-literal silently dropped by `lower_integer_const_expr_simple`): still present. `simple_expr.rs` has no `Expr::Name` arm and `negate_simple_expr` only flips literal numerics. Not a regression.
- **F7** (folded `value: uint8 = SOME_NAME` short-circuits the `HirExpr::Name` move-tracking branch in [`statements.rs:1084-1101`](crates/sifr_hir/src/lower/statements.rs:1084)): unchanged. Benign for `Type::Int` per the integer model doc; document as a follow-up if INT-3 changes integer ownership semantics.

## New observations

### N1 — E2E fail fixture inline annotations are decorative, not enforced

[`crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr`](crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr) uses inline trailing-comment annotations:

```python
def main():
    too_wide: uint8 = 2 ** 8  # expect-error: SIFR-INT-0001 col=23
    too_large: uint8 = 10 ** 5000  # expect-error: SIFR-INT-0004 col=23
```

`parse_expect_error_line` ([`crates/sifr/tests/e2e.rs:614`](crates/sifr/tests/e2e.rs:614)) calls `line.strip_prefix("# expect-error:")`, which only matches when the comment is at the *start* of the line. The canonical form is illustrated by [`annotated_variable_requires_initializer.sifr:1`](crates/sifr/tests/e2e/fail/annotated_variable_requires_initializer.sifr:1):

```python
# expect-error[col=5]: SIFR-NAME-0006
```

The current fixture's annotations are therefore not parsed — `expected` is empty for this fixture, so `test_e2e_fail` only asserts that compilation fails (any diagnostic counts). The codes (SIFR-INT-0001, SIFR-INT-0004) are *not* enforced by the e2e harness. The unit tests `test_fixed_width_const_expression_out_of_range_has_int_code` and `test_fixed_width_const_expression_budget_has_int_code` do assert codes/messages/ranges, so coverage exists in spirit.

Also note: the column claims in the inline annotations don't match. Line 2 has the literal at column 23 (correct), but line 3's literal `10 ** 5000` starts at column 24 (after `too_large: uint8 = `), not 23. Since neither column is enforced, the discrepancy is invisible today.

Recommendation: convert the inline annotations to canonical `# expect-error[col=N]: SIFR-INT-NNNN` lines preceding each test line, or remove them so the file doesn't suggest enforcement that isn't happening. Severity: documentation/polish, not functional.

### N2 — F3 estimator is correct under saturating arithmetic; covered

The new estimator helpers use `u64::saturating_mul(30_103)` and `usize::saturating_mul`, so even pathological `u32::MAX` exponents won't overflow and won't panic. `usize::try_from(u32).unwrap_or(usize::MAX)` is fine on 64-bit hosts and saturates cleanly on 32-bit. No new panic surface. Confirmed by inspection.

## Panics / no-user-path violations

None. Spot-checked all new code:
- `fixed_width_fitting.rs:99` — `value.trim_start_matches('-').len()` is total.
- `fixed_width_fitting.rs:201,212,224` — `non_negative_u32` returns `None` for negative or oversize, no panic.
- `fixed_width_fitting.rs:280-281` — `saturating_mul`, `unwrap_or` fallback.
- `fixed_width_fitting.rs:286` — `saturating_mul`, `unwrap_or` fallback.
- `fixed_width_fitting.rs:290` — `i64::try_from(BigInt)` fallback to `LargeIntLiteral`.
- No new `assert!`/`unwrap`/`expect` on user-triggerable paths.

## Validation independent of author report

I did not re-run the suites; the author reports `cargo fmt --check`, targeted `cargo test -p sifr_hir fixed_width`, `cargo clippy -p sifr_hir -p sifr_codegen -- -D warnings`, the e2e fail suite, the e2e pass fixture (`run`-mode, output `255 / 130 / 4096 / 69 / -50 / -127`), the HIR maintainability guardrails, and `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, 102.26s). The signature is unchanged from pass 1c, suggesting these changes are localized and don't regress the broader suite.

The pass-fixture output values match by-hand evaluation:
- `BASE + 1 = 254 + 1 = 255` ✓
- `(1 + 2) * 40 + (20 >> 1) = 120 + 10 = 130` ✓
- `2 ** 12 = 4096` ✓
- `(1 << 6) + (9 // 2) + (9 % 2) = 64 + 4 + 1 = 69` ✓
- `-10 * 5 = -50` ✓
- `-(100 + 27) = -127` ✓

## Verdict

All pass-1c blockers are resolved with minimal, well-scoped changes. The F1 fix uses the existing scope frame API and is correctly conservative; F2 silently drops the duplicate without weakening the existing TYPE_MISMATCH suppression; F3 swaps the misleading base-digit count for a saturating, conservative result-digit estimate; F6 closes the headline test gaps for `<<`, `//`, `%`, negative const expressions, shadowing, non-const binop fallback, and duplicate over-budget literals.

Two non-blocking polish items in this pass:
- N1: convert the inline e2e fail annotations to the canonical leading-line form, or drop them — they currently don't enforce the codes/columns they appear to.
- F4 (carryover from pass 1c): briefly note in the PR description the module-level `int` annotation behavior so reviewers know the inconsistency is intentional and bounded to that scope.

Neither is required to land this slice. The implementation is ready for PR.

VERDICT: SATISFIED
