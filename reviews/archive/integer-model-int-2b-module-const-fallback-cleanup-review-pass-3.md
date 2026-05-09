# INT-2B Module Const / Fixed-Width Fallback Cleanup — Review Pass 3

**Verdict:** Blockers found.

## Scope reviewed

Working-tree diff against `2b5bd78e` on `int-2b-module-const-fallback-cleanup`:

- [crates/sifr_hir/src/lower/module_constants_lowering.rs](crates/sifr_hir/src/lower/module_constants_lowering.rs)
- [crates/sifr_hir/src/lower/simple_expr.rs](crates/sifr_hir/src/lower/simple_expr.rs)
- [crates/sifr_hir/src/lower/expressions_tests.rs](crates/sifr_hir/src/lower/expressions_tests.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) (slice item: "Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable.")
- [reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-2.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-2.md)

## Pass-2 blocker resolution

The pass-2 blocker — `_for_export` muting of `SIFR-INT-0004` letting over-budget `int` module constants escape HIR and either panic codegen (`**`) or produce `rustc` errors (`<<`) — is closed. The revised diff drops the export-cache split entirely. `fixed_width_fitting.rs` is unchanged from the parent commit, and [module_constants_lowering.rs:54-59](crates/sifr_hir/src/lower/module_constants_lowering.rs:54) routes `remember_module_const_integer` through the original diagnostic-emitting `const_integer_value`. I re-traced the four pass-2 reproducers:

- `LIMIT: int = 10 ** 5000` ⇒ `evaluate_pow` → `reject_if_over_budget` emits one `SIFR-INT-0004`. `lower_module_with_externals` returns `Err`, so codegen never runs. Confirmed by `sifr check` and by [test_module_int_over_budget_const_expr_stays_hir_diagnostic](crates/sifr_hir/src/lower/expressions_tests.rs:466).
- `LIMIT = 10 ** 5000` (bare) ⇒ same path through `collect_bare_constant`/`remember_module_const_integer`, one `SIFR-INT-0004`.
- `LIMIT: int = -(10 ** 5000)` ⇒ `negate_module_integer_const_expr` wraps in `UnaryOp("-", BinOp, Int)`; const_integer_value's `"-"` arm propagates `Rejected` from the inner BinOp evaluation. One `SIFR-INT-0004`.
- `LIMIT: int = 1 << 99999` ⇒ `evaluate_left_shift` short-circuits via `MAX_EXACT_SHIFT_OR_EXPONENT`, emits one `SIFR-INT-0004`.

The pass-2 N1 (dead `negate_simple_expr` else arm), N2 (missing module-level fixed-width budget regression test), and N3 (codegen smoke for the new shape) are also addressed: [negate_module_integer_const_expr](crates/sifr_hir/src/lower/module_constants_lowering.rs:130) now uses `or_else`+`then`, [test_module_fixed_width_const_expression_budget_has_int_code_once](crates/sifr_hir/src/lower/expressions_tests.rs:435) pins the `uint8 = 10 ** 5000` single-emission, and [module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr:5) covers `BASE_LIMIT + 4` and `-(MAX_RETRIES + 10)` end-to-end.

## Criteria from the pass-3 brief

1. Module-level `int` over-budget const expressions should remain clean HIR diagnostics and not reach codegen — **met** (above).
2. Module-level fixed-width over-budget const expressions should emit exactly one budget diagnostic and no range/type follow-on — **met**: validate path emits `SIFR-INT-0004` and returns `Rejected`; the [error_count_before_initializer](crates/sifr_hir/src/lower/module_constants_lowering.rs:53) guard then skips `remember_module_const_integer`. Pinned by [test_module_fixed_width_const_expression_budget_has_int_code_once](crates/sifr_hir/src/lower/expressions_tests.rs:435).
3. Module constants may reuse prior const-evaluable names through unary/binop expressions without general expression lowering — **met functionally for `int` reuse**; see new blocker below for the `int`/fixed-width crossover variant.
4. The e2e module constants fixture should provide sufficient codegen smoke for the new name/binop/unary module const shapes — **met**: `BASE_LIMIT: int = 250` / `LIMIT: int = BASE_LIMIT + 4` exercises the BinOp+Name codegen path; `NEGATIVE_LIMIT: int = -(MAX_RETRIES + 10)` exercises the UnaryOp+BinOp+Name path; both have value assertions in `main`.

## New blocker — mixed `int` ⇄ fixed-width module constant references emit broken Rust

[lower_module_integer_const_expr](crates/sifr_hir/src/lower/module_constants_lowering.rs:98) accepts any prior name that lives in `ctx.const_integer_values` and synthesizes `HirExpr::Name { ty: Type::Int }` regardless of the source constant's actual scope type. Because `remember_module_const_integer` happily caches BigInt values for *fixed-width* module constants too (see the existing passing test [test_fixed_width_const_expression_uses_module_integer_constants](crates/sifr_hir/src/lower/expressions_tests.rs:313), which depends on that cache populating from `BASE: int = 250 + 4`), this new path will now resolve `Name`s for fixed-width-typed constants while reporting `Type::Int` to downstream codegen.

The result is that codegen renders Rust expressions that reference a fixed-width (`u8`/`i32`/...) const inside an `i64`-context expression, which `rustc` rejects.

Reproductions on the working tree (`sifr check` reports "no errors found"):

```sifr
BASE: uint8 = 250
ALIAS: int = BASE
def main():
    print(str(ALIAS))
```
`sifr emit` ⇒ `const BASE: u8 = 250u8;` / `const ALIAS: i64 = BASE;`
`sifr run` ⇒ `error[E0308]: mismatched types ... expected 'i64', found 'u8'`

```sifr
BASE: uint8 = 250
LIMIT: int = BASE + 4
def main():
    print(str(LIMIT))
```
`sifr emit` ⇒ `const BASE: u8 = 250u8;` / `const LIMIT: i64 = BASE + (4 as i64);`
`sifr run` ⇒ `error[E0277]: cannot add 'i64' to 'u8'` (also `E0308` x2).

```sifr
BASE: uint8 = 250
NEG: int = -BASE
def main():
    print(str(NEG))
```
`sifr emit` ⇒ `const BASE: u8 = 250u8;` / `const NEG: i64 = -BASE;`
`sifr run` ⇒ `error[E0308]: ... expected 'i64', found 'u8'` (and `E0600` since `-` is not defined on `u8`).

For comparison, on the parent commit `2b5bd78e` the alias case fails with `type error: undefined variable: 'ALIAS'` because the prior `lower_integer_const_expr_simple` did not handle `Name`s, so `ALIAS` was not collected as a module constant. The function-body equivalent (`def main(): value: int = BASE + 1`) still fails cleanly today with `unsupported operand type(s) for +: 'uint8' and 'int'` because regular expression lowering uses scope-resolved types — only this new module-level path bypasses that resolution.

### Why this matters for the slice

The slice's stated remit is "clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable." The pass-2 blocker called out a different reachable path that produced rustc errors / panics; pass 3 closes that one but introduces a new reachable path (mixed-type module const reuse) that produces rustc errors from valid-looking Sifr source. This contradicts AGENTS.md's "if it compiles, it works" commitment for the front-end gate (`sifr check` returns "no errors") and is exactly the class of issue the slice is meant to prevent.

The new pass-3 tests don't catch this because [test_module_constant_export_uses_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:399) and [test_module_constant_export_uses_unary_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:414) only assert on the *folded* `IntLiteral` produced by fixed-width fitting in a function body — they never look at the unfolded HIR shape that codegen sees for the mixed module-level case. The e2e fixture is all `int`-typed.

### Suggested resolution

Restrict the `Name` arm in `lower_module_integer_const_expr` to constants whose scope-resolved type is `Type::Int`. The scope is already populated by the time later constants are collected (each iteration calls `ctx.scope.define(var_name, ty)`), and `is_shadowed_by_inner_scope` already shows the lookup pattern. A minimal change:

```rust
Expr::Name(name) if ctx.const_integer_values.contains_key(name.id.as_str()) => {
    let scope_ty = ctx.scope.lookup(name.id.as_str()).cloned()?;
    if !matches!(scope_ty, Type::Int) {
        return None;
    }
    Some(HirExpr::Name { name: name.id.to_string(), ty: Type::Int })
}
```

Falling back to `None` here will leave the constant un-collected (today's behavior on `2b5bd78e`), so a downstream "undefined variable" / "unsupported operand" diagnostic surfaces instead of a generated Rust type mismatch. Add a regression test (HIR-level: assert lowering succeeds on a clean `BASE: int = 250\nLIMIT: int = BASE + 4` *and* that lowering of `BASE: uint8 = 250\nLIMIT: int = BASE + 4` does not produce a `LIMIT` entry in `module.constants` — or alternatively reports a clean diagnostic).

A more complete alternative would be to also store the source type alongside the BigInt in `const_integer_values` and propagate it into the synthesized `Name`'s `ty`, but that is more invasive and isn't required for this slice; the simple "only reuse when source is `Type::Int`" cut is sufficient.

## Non-blocking findings

### N1 — Module int over-budget test does not pin total error count

[test_module_int_over_budget_const_expr_stays_hir_diagnostic](crates/sifr_hir/src/lower/expressions_tests.rs:466) asserts there is exactly one `INT_EVAL_BUDGET_EXCEEDED` but does not also assert that no other diagnostic is emitted. Trace shows none today (validate is a no-op for `Type::Int` annotations and `value_ty == declared_type`), but adding either an `errors.len() == 1` check or a "no `TYPE_MISMATCH` follow-on" assertion (parallel to [test_module_fixed_width_const_expression_budget_has_int_code_once:455](crates/sifr_hir/src/lower/expressions_tests.rs:455)) would be a cheap pinning win.

### N2 — Pre-existing `LargeIntLiteral` codegen panic is still reachable from this PR's surface

Untouched by this slice, but worth re-flagging: a literal that fits within the 4096-decimal-digit budget but exceeds `i64` (e.g. `LIMIT: int = 999999999999999999999999999999999999`) survives `lower_module_integer_const_expr` → `lower_integer_const_expr_simple` → `LargeIntLiteral`, then `try_lower_simple_module_constant_item_result_impl` returns `Ok(None)` (since `try_lower_leaf_expr` does not handle `LargeIntLiteral`), which `emit_module_constants` escalates to a `panic!` at [module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12). Pass 2 noted this as out of scope for the cleanup slice and tied it to the SifrInt wiring under INT-1/INT-3. Worth tracking explicitly so it doesn't get lost.

### N3 — Naming/doc nit (optional)

`lower_module_integer_const_expr` and `negate_module_integer_const_expr` borrow `LowerCtx` only for `const_integer_values`. The pass-2 N4 suggestion to add a one-line note that they read only the const-integer cache and that callers handle budget validation still applies. Small, can be skipped.

## Determinism / duplication check

I traced the diagnostic emission count for the relevant module-scope cases:

- `LIMIT: uint8 = 10 ** 5000` (over-budget fixed-width): validate path emits one `SIFR-INT-0004`, `error_count_before_initializer` guard skips `remember_module_const_integer`. ✓ One emission.
- `LIMIT: int = 10 ** 5000` (over-budget int): validate is a no-op (NotConst), `remember_module_const_integer` emits one `SIFR-INT-0004`. ✓ One emission.
- `LIMIT: int = -(10 ** 5000)`: same path; `Rejected` propagates through the `"-"` arm of `const_integer_value`, single emission. ✓
- `LIMIT: int = BASE + 4` with prior `BASE: int = 250`: validate is a no-op, `remember_module_const_integer` evaluates to `254`, no diagnostic. ✓
- Shadowed module constant via inner-scope rebind ([test_fixed_width_const_expression_does_not_fold_shadowed_module_constant](crates/sifr_hir/src/lower/expressions_tests.rs:329)): `is_shadowed_by_inner_scope` short-circuits to `Unsupported`, validate produces a single `TYPE_MISMATCH`, no range diagnostic. ✓

No duplicate-emission concerns introduced. The duplicate-emission risk that originally motivated the (pass-2) `_for_export` split was, as pass 2 already noted, already prevented by the `error_count_before_initializer` guard, so reverting the split was safe.

## Scope drift

- `simple_expr.rs` exposes `negate_simple_expr` and `integer_binop_source` as `pub(super)`. Tightly scoped reuse, no drift.
- `lower_module_integer_const_expr` is a contained replacement of the prior `lower_integer_const_expr_simple` call site for module constants. It only adds `Name`/`UnaryOp`/`BinOp` cases needed for module-level cross-const folding and falls through to `lower_integer_const_expr_simple` for the pre-existing leaf shapes. No drift.
- E2E fixture additions are localized to the existing `module_constants.sifr` and assert structurally; no unrelated test deletions or renames.

## Validation reproduced

- `cargo fmt` — clean.
- `cargo test -p sifr_hir fixed_width -- --nocapture` — passing locally.
- `cargo test -p sifr_hir module_ -- --nocapture` — passing locally.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` — passing.
- `cargo run -q -p sifr -- check /tmp/limit_pow.sifr` (with `LIMIT: int = 10 ** 5000`) ⇒ clean `SIFR-INT-0004`, codegen not invoked.

The new blocker was reproduced via the three mixed-type cases above; none of these are exercised by the in-tree test suite.

## Verdict

**Blockers found.** Pass 2's blocker is closed and the four explicit pass-3 criteria are met for the all-`int` shapes the brief calls out. However, the same `lower_module_integer_const_expr` `Name` arm that enables `int` reuse silently re-routes mixed-type module constants (e.g. `int = uint8_const`, `int = uint8_const + N`, `int = -uint8_const`) into codegen with a synthesized `Type::Int` Name, producing rustc errors from valid-looking Sifr source on the front-end gate. Constrain the `Name` arm to scope-`Type::Int` constants (or otherwise refuse to fold across types) and add a HIR-level regression test before merging. Once that lands, the slice is otherwise in good shape.
