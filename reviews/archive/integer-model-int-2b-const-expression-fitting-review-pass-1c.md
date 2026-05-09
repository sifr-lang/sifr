# INT-2B const expression fitting — review pass 1c

Branch: `int-2b-const-expression-fitting`
Local validation reported by author: `scripts/run_all_tests.sh --profile quick` `report_signature=e1bf653aaa770517` (70.12s).

## Scope fit

In scope and implemented:
- New const-eval evaluator [`fixed_width_fitting.rs:95`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:95) covers integer/large integer literals, unary `+`/`-`, binary `+`/`-`/`*`/`//`/`%`/`<<`/`>>`/non-negative `**`, parentheses (transparently via the regular HIR shape), and same-module immutable integer constants.
- `validate_fixed_width_initializer` now returns `FixedWidthInitializerFit` (NotConst / Fits(HirExpr) / Rejected) so the caller can both suppress duplicate `TYPE_MISMATCH` and replace the HIR value with a folded literal ([`fixed_width_fitting.rs:10`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:10), [`statements.rs:1057`](crates/sifr_hir/src/lower/statements.rs:1057)).
- Folded value is `IntLiteral(i64)` when fitting, `LargeIntLiteral(decimal)` otherwise, so the existing codegen suffix path (`fixed_width_literal_expr_for_target`) round-trips into `255u8`, `-128i8`, `18446744073709551615u64` style emission.
- New `LowerCtx.const_integer_values: HashMap<String, BigInt>` stores evaluated module constants for later fitting ([`mod.rs:201`](crates/sifr_hir/src/lower/mod.rs:201)). Population happens in [`module_constants_lowering.rs`](crates/sifr_hir/src/lower/module_constants_lowering.rs), guarded by `error_count_before_initializer == error_count()` so rejected/erroring constants are never remembered.
- Module-constant collection refactored out of `lower_module_impl` into `module_constants_lowering::collect_module_constants`, plus `lower_expr_simple` was relocated from `classes.rs` into a new `simple_expr.rs` (alongside a binop-aware variant `lower_integer_const_expr_simple`). Pure rearrangement with one functional extension (binops in const-context only).
- Two-tier budget enforcement: an early `MAX_EXACT_SHIFT_OR_EXPONENT = 13_610` short-circuit for `<<` / `**` to avoid pathological allocation, plus a final `reject_if_over_budget` against `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET = 4096` on every evaluated result. `INT_EVAL_BUDGET_EXCEEDED` constant promoted to `pub(super)` for sharing with the parser visitor.
- Tests: four new HIR unit tests covering positive fold, module-const reference, out-of-range (`SIFR-INT-0001`), and over-budget (`SIFR-INT-0004`); two e2e fixtures (`fixed_width_const_expression_assignment.sifr`, `fixed_width_const_expression_out_of_range.sifr`).

Out of scope and **correctly** untouched:
- Function call arguments, returns, list/dict elements, generic specialization continue to use plain `is_assignable_to` — no fitting attempted there. The previous `test_fixed_width_call_argument_literal_is_not_implicitly_narrowed` guard is still in place.
- No cross-module const propagation; imported names never enter `const_integer_values`.

## Correctness review

**Statement-level branching is sound.** `lower_ann_assign` ([`statements.rs:1057-1074`](crates/sifr_hir/src/lower/statements.rs:1057)) now derives `fixed_width_not_const = matches!(_, NotConst)`, replaces `expr` only on `Fits(_)`, and continues to suppress the legacy `TYPE_MISMATCH` whenever fitting fired (whether Fits or Rejected). Behavior parity with the prior `Option<bool>` shape is preserved, with the bonus that `Fits(folded)` actually substitutes the HIR.

**Python-correct `//` and `%`.** `python_floor_div` and `python_mod` ([`fixed_width_fitting.rs:170-183`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:170)) handle the negative-remainder case correctly (matches CPython semantics). Zero-divisor short-circuits to `Unsupported` instead of dividing — no panic.

**Budget two-tier check.** `evaluate_left_shift` / `evaluate_pow` short-circuit on `shift > 13_610` / `exponent > 13_610` and `abs_left > 1`, then the result is digit-counted and rejected if > 4096. Threshold is calibrated to base 2 (`log10(2) * 13_610 ≈ 4096`), so `2 ** 13_610` (≈ 4096 digits) lands at the budget boundary and `2 ** 13_611` early-rejects without computing. For larger bases the conservatism is preserved by the post-evaluation digit check.

**No panics on user-triggered paths.** `BigInt::pow(u32)`, `<<` / `>>` with bounded shifts, parsing via `value.parse()`, and `i64::try_from(BigInt)` all return without panicking. `decimal_digit_count` handles 0 and negatives correctly. No `unwrap` / `expect` / `assert!` in the new evaluator.

**Folded HIR/codegen consistency.** `bigint_to_hir_integer_literal` produces `IntLiteral(i64)` when the value fits and `LargeIntLiteral(decimal)` otherwise. Negative values past `i64::MIN` are unreachable through fitting (no fixed-width target permits them), so `LargeIntLiteral` strings emitted by the fitter remain unsigned-canonical. `fixed_width_literal_expr_for_target` walks `IntLiteral` / `LargeIntLiteral` / unary `+`/`-` and suffixes correctly; the `-128i8` corner case still works.

**Same-module constant remember/use ordering.** `collect_module_constants` runs before function-body lowering, so `BASE: int = 250 + 4` is in `const_integer_values` by the time `value: uint8 = BASE + 1` evaluates. The error_count guard correctly prevents storing a value when `validate_annotated_constant_initializer` produced any diagnostic — out-of-range and over-budget module constants are not remembered.

## Findings

### F1 — Module-constant lookup is scope-blind (correctness, edge case)

[`fixed_width_fitting.rs:102-108`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:102):

```rust
HirExpr::Name { name, .. } => {
    return ctx
        .const_integer_values
        .get(name)
        .cloned()
        .map_or(ConstIntegerValue::Unsupported, ConstIntegerValue::Value);
}
```

`HirExpr::Name` carries only a `String`, and the lookup is keyed purely on that string. If a function-body local, parameter, comprehension binding, or `for` loop variable shadows a same-named module constant, the fitter folds the source-level reference using the *module* value rather than the bound value. The local frame in `ctx.scope.frames` correctly resolves the Name to the local at the regular-lowerer stage, but the const-eval pass ignores that and substitutes the stale module value.

Concrete reproducer (not currently in the test suite):

```python
BASE: int = 254

def main() -> uint8:
    BASE: int = 100        # shadow
    return BASE + 1        # in a uint8-annotated let, this would silently fold to 255
```

`value: uint8 = BASE + 1` here would emit `IntLiteral(255)` instead of either rejecting (no const ref to module BASE) or honoring the local 100. This is a silent wrong-answer if the user shadows. Same hazard for parameters (`def main(BASE: int) -> uint8: value: uint8 = BASE + 1`) and `for BASE in [...]`.

The minimal, mechanical fix uses the existing scope API ([`scope.rs:131,237`](crates/sifr_hir/src/scope.rs:131)): in the `Name` arm, before consulting `const_integer_values`, check whether `ctx.scope.lookup_in_frame_range(name, 1, ctx.scope.frame_count() - 1)` returns `Some(_)`; if so, the name is shadowed by an inner frame and the const value must not be folded. Two lines, no new infrastructure.

Severity: real correctness bug, low frequency in practice, no test coverage today. Worth either fixing inline or pinning the current behavior in a test (so the team knows what was decided) before landing.

### F2 — Duplicate `SIFR-INT-0004` for over-budget literal tokens

`integer_literal_diagnostics::validate_module_integer_literals` runs first ([`mod.rs:526`](crates/sifr_hir/src/lower/mod.rs:526)) and emits `SIFR-INT-0004` for any literal token whose canonical decimal length exceeds `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET`. The new `const_integer_value` for `LargeIntLiteral` ([`fixed_width_fitting.rs:98`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:98)) then parses the same string, falls through to `reject_if_over_budget`, and emits a *second* `SIFR-INT-0004` with the same code/message/range. Triggers for any annotated-fixed-width or remember-path module literal where the bare literal already exceeds budget, e.g.:

```python
LIMIT: uint8 = 100000…(5000-digit literal)…000   # parser-level emits, then fitter re-emits
LIMIT = 100000…(5000-digit literal)…000          # same on the bare path via remember_module_const_integer
```

Existing tests don't catch the duplicate (they all use `errors.iter().any(...)`).

Cheap fix: in the `LargeIntLiteral` arm, short-circuit when `value.trim_start_matches('-').len() > INTEGER_EVAL_DECIMAL_DIGIT_BUDGET` and return `Unsupported`. The parser-level diagnostic remains the canonical signal and the fitter doesn't compound it.

Severity: quality, not functional — the user still gets the right error. Worth fixing for noise reduction.

### F3 — Misleading "1 decimal digits" for early shift/pow rejection

[`fixed_width_fitting.rs:194-198, 222-226`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:194):

```rust
if shift > MAX_EXACT_SHIFT_OR_EXPONENT && left != &BigInt::from(0) {
    emit_budget_exceeded(ctx, decimal_digit_count(left), range);
    return ConstIntegerValue::Rejected;
}
…
emit_budget_exceeded(ctx, decimal_digit_count(left), range);
```

The early-rejection path passes `decimal_digit_count(left)` (the *base*'s digit count, e.g., `1` for `2`, `2` for `16`). For `2 ** 100_000` the user sees:

```
integer literal exceeds compile-time evaluation budget: 1 decimal digits (max 4096)
```

…which is misleading — the offending magnitude is the result, not the base. An approximation `((digits(left) as u64) * (exponent as u64)).min(usize::MAX) as usize` (or `((exponent as f64) * (left.bits() as f64) * 0.301).ceil()` for bases > 1) would already be more honest. Alternatively, plumb a separate "would-exceed-budget" diagnostic that doesn't claim a specific digit count.

Existing tests pass because `10 ** 5000` falls through the early threshold (`5000 < 13_610`) and the *result* is digit-counted by `reject_if_over_budget` — that path already reports the correct `5001 decimal digits`.

Severity: message-quality polish.

### F4 — Module-level `int` annotations can newly trigger `SIFR-INT-0004` from the remember path

`remember_module_const_integer` is called for every annotated/bare module integer assignment that survives validation ([`module_constants_lowering.rs:52,86`](crates/sifr_hir/src/lower/module_constants_lowering.rs:52)). For a non-fitting target (`int`) where the initializer evaluates to > 4096 digits, the inner `reject_if_over_budget` still emits `SIFR-INT-0004`, even though the user didn't ask for fitting:

```python
LIMIT: int = 10 ** 5000   # at module scope, now errors with SIFR-INT-0004
COUNT       = 10 ** 5000  # same on the bare path
```

The same expression *inside a function body* with the same `int` annotation does not error today, because `validate_fixed_width_initializer` returns `NotConst` early before evaluating, and there's no remember step. This produces an inconsistent compile contract depending on declaration scope.

Defensible per the design ("any evaluated integer result" — `internal_docs/integer_model.md:101`), but the inconsistency is worth noting in the PR description, and arguably the remember path should silently fail (returning `Unsupported`) rather than emitting from a "we tried to be helpful" code path.

Severity: behavioral surprise, defensible by spec. Worth a note.

### F5 — Module-level Name references and unary-on-non-literal don't lower

[`simple_expr.rs`](crates/sifr_hir/src/lower/simple_expr.rs) has no `Expr::Name` arm, and `negate_simple_expr` only flips `IntLiteral` / `LargeIntLiteral` / `FloatLiteral` (not `BinOp` / `Name`). So the following module-level forms silently fail to lower as constants and are dropped from the module entirely (no `ctx.scope.define`, no `constants.push`):

```python
LIMIT: int    = 100
DERIVED: int  = LIMIT + 1     # dropped — Name unsupported in lower_integer_const_expr_simple
NEGATED: int  = -(2 + 3)      # dropped — negate_simple_expr can't wrap a BinOp
```

The legacy `lower_expr_simple` had the same gap, so this is a *limitation*, not a regression. The function-body path goes through the regular HIR lowerer, which produces `HirExpr::Name` and `HirExpr::UnaryOp { op: "-", operand: <any>, … }`, both of which `const_integer_value` handles. So `z: uint8 = LIMIT` *inside* `def main()` still folds.

If the team wants module-level chaining (and the design doc's `LIMIT: int = 200; z: uint8 = LIMIT` example does suggest it), `lower_integer_const_expr_simple` would need a `Name` arm that consults `ctx.const_integer_values` (or the prior collected constants), and `negate_simple_expr` would need to wrap arbitrary lowered HIR with `UnaryOp { op: "-", operand, ty: Int }` when the inner expression has type `Int`.

Severity: limitation, no regression, but a discoverability paper-cut (silent drop is worse than a diagnostic). Worth a follow-up note in the INT-2B follow-ups bullet.

### F6 — Test coverage gaps for documented features

The unit and e2e tests exercise `+`, `-`, `*`, `>>`, and `**`. Not exercised by any added test:

- `<<` (left shift) — coded path, no fixture.
- `//` and `%` — coded paths, no fixture (and `// 0` / `% 0` short-circuit to `Unsupported`, falling through to `TYPE_MISMATCH`, which is worth pinning so a future change to that arm doesn't silently regress).
- Negative const expressions, e.g., `value: int8 = -10 * 5` (= -50) and `value: int16 = -(100 + 27)`.
- Non-const-operand binop falling through to `TYPE_MISMATCH`, e.g., `value: uint8 = some_func() + 1` (currently relies on the existing `test_fixed_width_assignment_from_non_const_int_is_still_mismatch` for the bare `Name` case but not for binops with one non-const operand).
- Module-constant shadowing (see F1).
- A module-level fitting fixture, e.g., `LIMIT: uint8 = 250 + 4` consumed in `main()`, to round-trip the `lower_item.rs` codegen path through e2e.

The current e2e pass fixture exercises `BASE + 1`, `(1 + 2) * 40 + (20 >> 1)`, and `2 ** 12`; the e2e fail fixture exercises `2 ** 8` and `10 ** 5000`. Coverage is reasonable for the headline acceptance criteria but understates what's actually wired up.

Severity: gaps, not regressions. Each is a one-line fixture or one assertion.

### F7 — Folding bypasses Name-shape ownership tracking

After folding, [`statements.rs:1083-1101`](crates/sifr_hir/src/lower/statements.rs:1083) checks `if let HirExpr::Name { name, ty } = value { … mark_moved … }`. When the source was `value: uint8 = SOME_NAME` and `SOME_NAME` was a `Move`-ownership binding, the fold replaces the `Name` with an `IntLiteral` and the move-tracking branch never runs.

For `Type::Int` specifically, this is currently benign because exact `int` is value-semantic and source-level use shouldn't move the binding (per `internal_docs/integer_model.md` Compiler Architecture Impact §9). But the implication of folding before the move check is worth documenting; if INT-3 changes ownership semantics for any integer-typed binding, this elision could quietly change observable behavior.

Severity: latent, depends on future ownership policy. Note for the follow-ups bullet.

## Tests reviewed

- `test_fixed_width_const_expression_assignment_fits_and_folds` — covers `(1 + 2) * 40 + (20 >> 1) = 130` for uint8 and asserts `IntLiteral(130)` fold.
- `test_fixed_width_const_expression_uses_module_integer_constants` — `BASE: int = 250 + 4` then `BASE + 1` for uint8, asserts `IntLiteral(255)` fold; pins same-module const propagation in the no-shadowing case.
- `test_fixed_width_const_expression_out_of_range_has_int_code` — `2 ** 8` against uint8, pins SIFR-INT-0001 message and range, and asserts no shadow `TYPE_MISMATCH`.
- `test_fixed_width_const_expression_budget_has_int_code` — `10 ** 5000` against uint8, pins SIFR-INT-0004 message (`5001 decimal digits`) and asserts no shadow SIFR-INT-0001.
- `test_fixed_width_literal_assignment_fits` — updated to assert `IntLiteral(-128)` instead of `UnaryOp("-", IntLiteral(128))` after the fold, locking in the i8-MIN normalization.
- e2e `fixed_width_const_expression_assignment.sifr` (round-trips the module-const path + binop + pow through the compiler).
- e2e `fixed_width_const_expression_out_of_range.sifr` (col-pinned SIFR-INT-0001 and SIFR-INT-0004).

Coverage matches headline acceptance criteria from `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` lines 162-167 modulo the gaps listed in F6.

## Verdict

The implementation is well-structured: the evaluator is small, the budget enforcement is layered correctly, the FixedWidthInitializerFit refactor is clean, and the codegen suffix path is reused without churn. Core acceptance criteria (`x: uint8 = 100 + 27` ok, `x: uint8 = 256` rejects with SIFR-INT-0001, `10 ** 5000` rejects with SIFR-INT-0004, no implicit narrowing in calls/returns/lists) are met for the same-module case.

The most material issue is F1 (scope-blind Name lookup): a real silent-wrong-answer bug under module-constant shadowing, mitigated only by the rarity of that pattern in practice. The fix is two lines using the existing scope frame API. F2 (duplicate diagnostic) and F3 (misleading early-reject digit count) are quality issues; F4 is a defensible behavioral note; F5 is a pre-existing limitation; F6 is a test-coverage gap; F7 is latent.

Recommendation: address F1 (or, at minimum, add a regression test that pins the current scope-blind behavior so the team has explicitly chosen it), tighten the duplicate emission in F2, and add the `<<`/`//`/`%`/negative-const test cases from F6. None of the other findings are blocking.

VERDICT: CHANGES REQUESTED
