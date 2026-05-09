# INT-2B Module Const / Fixed-Width Fallback Cleanup — Review Pass 2

**Verdict:** Blockers found.

## Scope reviewed

Working-tree diff against `2b5bd78e` on `int-2b-module-const-fallback-cleanup`:

- [crates/sifr_hir/src/lower/fixed_width_fitting.rs](crates/sifr_hir/src/lower/fixed_width_fitting.rs)
- [crates/sifr_hir/src/lower/module_constants_lowering.rs](crates/sifr_hir/src/lower/module_constants_lowering.rs)
- [crates/sifr_hir/src/lower/simple_expr.rs](crates/sifr_hir/src/lower/simple_expr.rs)
- [crates/sifr_hir/src/lower/expressions_tests.rs](crates/sifr_hir/src/lower/expressions_tests.rs)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) (slice item: "Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable.")
- [internal_docs/integer_model.md](internal_docs/integer_model.md)

## Summary of intended changes

1. **Split const evaluation paths.** [fixed_width_fitting.rs:78-105](crates/sifr_hir/src/lower/fixed_width_fitting.rs:78) introduces `const_integer_value_for_export` (with `emit_budget_diagnostics: false`) and re-routes `remember_module_const_integer` through it, threading the flag down through `evaluate_left_shift`, `evaluate_pow`, and `reject_if_over_budget`. The validation entry point `const_integer_value` continues to emit budget diagnostics.
2. **Module-aware lowering.** [module_constants_lowering.rs:98-143](crates/sifr_hir/src/lower/module_constants_lowering.rs:98) replaces the `lower_integer_const_expr_simple` call sites with a new `lower_module_integer_const_expr` that resolves prior module-cached `Name`s, plus unary `+`/`-` and binops over them. `simple_expr.rs` exposes `negate_simple_expr` and `integer_binop_source` as `pub(super)` for reuse.
3. **Tests.** Three new HIR-level tests in [expressions_tests.rs:399-442](crates/sifr_hir/src/lower/expressions_tests.rs:399) cover (a) `LIMIT: int = 10 ** 5000` not erroring on the export path, (b) `LIMIT: int = BASE + 4` folding through to a `uint8` initializer, and (c) `NEGATIVE: int = -(BASE + 3)` folding through to an `int8` initializer.

## Blocker — internal compiler panic for over-budget `int` module constants

The `_for_export` flag muting suppresses `SIFR-INT-0004` for module-level `int` constants whose const evaluation exceeds the 4096-decimal-digit compile-time budget. With the prior code, the `remember_module_const_integer` call would re-evaluate and fail validation (because `validate_annotated_constant_initializer` returns `NotConst` for non-fixed-width `Type::Int`, leaving the budget guard in `remember_module_const_integer` as the only emitter). After the change, no diagnostic is emitted and lowering succeeds — but the resulting HIR shape is not codegen-safe.

Reproduction:

```sifr
# /tmp/limit_pow.sifr
LIMIT: int = 10 ** 5000

def main():
    print("ok")
```

- Before this branch: `cargo run -q -p sifr -- check /tmp/limit_pow.sifr` ⇒ `type error: integer literal exceeds compile-time evaluation budget: 5001 decimal digits (max 4096)`. Clean `SIFR-INT-0004`.
- After this branch: `cargo run -q -p sifr -- check /tmp/limit_pow.sifr` ⇒ `no errors found`, then `cargo run -q -p sifr -- emit /tmp/limit_pow.sifr` panics:
  ```
  thread 'main' panicked at crates/sifr_codegen/src/module_constants.rs:12:17:
  structured module constant emission missing for production path (LIMIT):
  unsupported module constant lowering shape: name=LIMIT, ty=Int,
  value=BinOp { left: IntLiteral(10), op: "**", right: IntLiteral(5000), ty: Int }
  internal compiler error: internal compiler panic during single-file code generation: ...
  ```

Variants that reproduce the same regression on this branch (all panic at codegen):

- `LIMIT = 10 ** 5000` (bare assignment, no annotation).
- `LIMIT: int = -(10 ** 5000)` (unary-negated over-budget).
- `LIMIT: int = 999999999999999999999999999999999999` (large literal exceeding `i64`, no `**` involved — unrelated to this PR but reachable from any over-budget module int annotation).

The `**` regression is the worst one because [fixed_width_fitting.rs:280-294](crates/sifr_hir/src/lower/fixed_width_fitting.rs:280) ran a *budget* gate that now no longer trips on the export path, while [crates/sifr_codegen/src/lower_expr.rs:1503-1519](crates/sifr_codegen/src/lower_expr.rs:1503) does not list `**` as a "safe simple binop". So `try_lower_simple_module_constant_item_result_impl` returns `Ok(None)` for primitive `Type::Int` with a `**` BinOp, and [crates/sifr_codegen/src/module_constants.rs:8-15](crates/sifr_codegen/src/module_constants.rs:8) escalates that to a panic.

A second variant doesn't panic but emits unsound Rust:

```sifr
LIMIT: int = 1 << 99999
```

emits `const LIMIT: i64 = (1 as i64) << (99999 as i64);`, which `rustc` rejects:
```
error[E0080]: attempt to shift left by `99999_i64`, which would overflow
```
Previously this case also raised the clean `SIFR-INT-0004` at HIR. After this branch the diagnostic is silently moved to a downstream `rustc` error.

### Why this matters for the slice

The slice's stated remit is "clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable." Replacing a clean `SIFR-INT-0004` diagnostic with an internal-compiler panic on a non-fixed-width construct is the opposite of cleanup — it's a UX regression that contradicts the AGENTS.md commitment that "if it compiles, it works" and that there are no user-triggerable runtime panics. (The panic here is at compile time, not runtime, but it is still a `panic!` reached from valid-looking user input.)

The PR description correctly identifies that "const export-cache remembering for module-level int constants must not emit fixed-width budget diagnostics just because a module int constant is over budget." That intent is right *if* downstream codegen can represent the construct. Today it can't:
- `Type::Int` is still emitted as `i64` in module constant codegen ([crates/sifr_codegen/src/lower_item.rs:88-105](crates/sifr_codegen/src/lower_item.rs:88)) — the SifrInt-backed `int` from INT-1 is not yet wired into module-level constant lowering. INT-3/INT-1 wave 2 work is what unlocks this.
- `try_lower_leaf_or_name_expr_result` ([crates/sifr_codegen/src/lower_item.rs:71-79](crates/sifr_codegen/src/lower_item.rs:71)) doesn't accept `LargeIntLiteral` either.

Until that lands, suppressing the budget gate creates an unreachable-by-design path that explodes at codegen.

### Suggested resolutions (any one fixes the blocker)

- **Preferred**: keep `remember_module_const_integer` calling the diagnostic-emitting path (i.e., revert the `_for_export` split), at least until codegen learns to emit over-budget `int` module constants. The previous duplicate-diagnostic risk cited as the motivation for this split is already prevented by the `error_count_before_initializer` guard in [module_constants_lowering.rs:44-60](crates/sifr_hir/src/lower/module_constants_lowering.rs:44): `remember_module_const_integer` only runs when validation didn't error, so the only way it can re-emit is when validation didn't gate (i.e., when the target was non-fixed-width and we genuinely *do* want to error). Concretely, for `LIMIT: int = 10 ** 5000` the legacy single-emission of `SIFR-INT-0004` is the right behavior right now.
- **Alternative**: keep the export-path mute *only* when the const RHS is already representable by codegen (for example, leaf or simple-binop over leaves where the `**` operator is rejected at lowering, mirroring `is_safe_simple_binop`). This is more invasive and duplicates codegen invariants in HIR.
- **Alternative**: extend codegen to emit over-budget `int` module constants via the SifrInt runtime's big-integer constructor. That's a real INT-1/INT-3 surface and is out of scope for a "cleanup" slice.

Either way, the test `test_module_int_over_budget_const_expr_is_not_export_cache_diagnostic` ([expressions_tests.rs:398-412](crates/sifr_hir/src/lower/expressions_tests.rs:398)) needs to be reframed: it currently asserts the regression as if it were the desired behavior, but it only exercises HIR (`lower_source`) and so misses the downstream panic.

## Non-blocking findings

### N1 — Dead branch in `negate_module_integer_const_expr`

[module_constants_lowering.rs:130-143](crates/sifr_hir/src/lower/module_constants_lowering.rs:130):

```rust
fn negate_module_integer_const_expr(expr: HirExpr) -> Option<HirExpr> {
    if let Some(negated) = negate_simple_expr(expr.clone()) {
        return Some(negated);
    }
    if matches!(expr.ty(), Type::Int) {
        Some(HirExpr::UnaryOp { op: "-".to_string(), operand: Box::new(expr), ty: Type::Int })
    } else {
        negate_simple_expr(expr)
    }
}
```

The final `else` arm calls `negate_simple_expr` on an expression that already produced `None` from `negate_simple_expr` two lines up — `negate_simple_expr` only depends on the variant, not on context, so it returns `None` again. The arm is unreachable in practice and reads like a stray copy/paste. Either drop the branch:

```rust
fn negate_module_integer_const_expr(expr: HirExpr) -> Option<HirExpr> {
    negate_simple_expr(expr.clone()).or_else(|| {
        matches!(expr.ty(), Type::Int).then(|| HirExpr::UnaryOp {
            op: "-".to_string(),
            operand: Box::new(expr),
            ty: Type::Int,
        })
    })
}
```

…or, if the intent is "wrap in UnaryOp only when the operand is `int`", say so directly without the dead arm.

### N2 — Missing module-level fixed-width budget regression test

The PR adds [test_module_int_over_budget_const_expr_is_not_export_cache_diagnostic](crates/sifr_hir/src/lower/expressions_tests.rs:398) but doesn't add a positive `SIFR-INT-0004` assertion at module scope. The existing [test_fixed_width_const_expression_budget_has_int_code](crates/sifr_hir/src/lower/expressions_tests.rs:378) only covers function-body assignments. Given that the fix in this slice deliberately changes the budget-emission behavior at module scope, a pinning test like

```rust
let source = "LIMIT: uint8 = 10 ** 5000\n\ndef main():\n    print(\"ok\")\n";
```

asserting the diagnostic is emitted exactly once and points at `10 ** 5000` would prevent silent regressions on the still-load-bearing fixed-width path. (I confirmed manually via `sifr check` that the diagnostic still fires today.)

### N3 — Tests do not cover the codegen surface for the new module-constant shape

[test_module_constant_export_uses_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:414) and [test_module_constant_export_uses_unary_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:429) verify HIR, but the new constant exports `BASE: int = 250` followed by `LIMIT: int = BASE + 4` only become genuinely useful when codegen renders the BinOp HIR through the safe-simple-binop path. I sanity-checked end-to-end with `sifr emit` (`const BASE: i64 = 250 as i64;` / `const LIMIT: i64 = BASE + (4 as i64);`), so the happy path works today — but a focused codegen snapshot test (or e2e fixture in `verification/`) would prevent later codegen restrictions from silently breaking the slice's externally-visible promise.

### N4 — Naming nit

`lower_module_integer_const_expr` and `negate_module_integer_const_expr` borrow `LowerCtx` only for the `const_integer_values` map. Passing `&LowerCtx` (already done) is fine, but the function reads as if it should grow to manage scope. Worth adding a one-line doc comment that the only piece of `ctx` it reads is the const-integer cache and that the caller is responsible for budget validation. (Optional — the call sites in `collect_annotated_constant` / `collect_bare_constant` are short enough that this is not load-bearing.)

## Determinism / duplication check

I traced the flow for both the fix-target and the existing fixed-width path:

- `LIMIT: uint8 = 10 ** 5000`: `validate_fixed_width_initializer` runs `const_integer_value` (emit=true) → `evaluate_pow` → `reject_if_over_budget` emits `SIFR-INT-0004` once and returns `Rejected`. `validate_annotated_constant_initializer` returns `None`. The `error_count_before_initializer` guard in `collect_annotated_constant` skips `remember_module_const_integer`, so no second emission. ✓ One diagnostic, deterministic. Manual `sifr check` confirms.
- `LIMIT: int = 250 + 4`: validation early-returns `NotConst`; `remember_module_const_integer` evaluates and caches `254`. ✓
- `LIMIT: int = BASE + 4` (with prior `BASE: int = 250`): identical to above, with `BASE` resolved through `const_integer_values`. ✓
- `BASE: int = 254` shadowed by inner-scope `BASE: int = 100` then `value: uint8 = BASE + 1`: `is_shadowed_by_inner_scope` short-circuits to `Unsupported`, validate produces a `TYPE_MISMATCH`. Existing test [test_fixed_width_const_expression_does_not_fold_shadowed_module_constant](crates/sifr_hir/src/lower/expressions_tests.rs:328) still passes. ✓

No diagnostic-duplication concerns introduced by the split.

## Scope drift

- `simple_expr.rs` exposes two helpers as `pub(super)` (`negate_simple_expr`, `integer_binop_source`). Tightly scoped reuse, no drift.
- The `lower_module_integer_const_expr` recursion is a small, contained replacement of the prior call. It does not pull in general expression lowering; it only adds the Name/UnaryOp/BinOp cases needed for module-level cross-const folding. No drift.
- No unrelated test deletions or renames.

## Validation reproduced

- `cargo fmt` — clean.
- `cargo test -p sifr_hir fixed_width -- --nocapture` — 13/13 passing locally.
- `cargo test -p sifr_hir module_ -- --nocapture` — 8/8 passing locally.

These do not exercise codegen, which is where the blocker lands.

## Verdict

**Blockers found.** The `_for_export` muting of `SIFR-INT-0004` turns previously-clean compile errors into either an internal compiler panic (`**` over budget, `LargeIntLiteral` over `i64`) or a `rustc` error (`<<` past `i64`) for module-level `int` constants. The slice's intent is sound but cannot land safely until codegen can emit over-budget `int` module constants, or until the export path stops accepting expressions that codegen will reject. Pick one of the suggested resolutions before merging, and add the missing module-level fixed-width budget test and a codegen smoke check to keep this from re-regressing.
