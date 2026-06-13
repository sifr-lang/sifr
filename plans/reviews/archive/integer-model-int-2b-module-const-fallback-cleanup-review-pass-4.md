# INT-2B Module Const / Fixed-Width Fallback Cleanup — Review Pass 4

**Verdict:** Satisfied with non-blocking suggestions.

## Scope reviewed

Working-tree diff against `2b5bd78e` on `int-2b-module-const-fallback-cleanup`:

- [crates/sifr_hir/src/lower/module_constants_lowering.rs](crates/sifr_hir/src/lower/module_constants_lowering.rs)
- [crates/sifr_hir/src/lower/simple_expr.rs](crates/sifr_hir/src/lower/simple_expr.rs)
- [crates/sifr_hir/src/lower/expressions_tests.rs](crates/sifr_hir/src/lower/expressions_tests.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-2.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-2.md)
- [reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-3.md](reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-3.md)

## Pass-3 blocker resolution — closed

The pass-3 blocker was that [lower_module_integer_const_expr](crates/sifr_hir/src/lower/module_constants_lowering.rs:98)'s `Name` arm accepted any prior name living in `ctx.const_integer_values` and synthesized `HirExpr::Name { ty: Type::Int }` regardless of the source constant's actual scope type. Mixed-type module constants such as

```sifr
BASE: uint8 = 250
ALIAS: int = BASE
```

passed `sifr check` and then failed at `rustc` with `E0308`/`E0277`/`E0600`.

The revised diff guards the `Name` arm with a scope-type check:

```rust
Expr::Name(name) if ctx.const_integer_values.contains_key(name.id.as_str()) => {
    let scope_ty = &ctx.scope.lookup(name.id.as_str())?.ty;
    if !matches!(scope_ty, Type::Int) {
        return None;
    }
    Some(HirExpr::Name { name: name.id.to_string(), ty: Type::Int })
}
```

Trace for the three pass-3 reproducers:

- `BASE: uint8 = 250\nALIAS: int = BASE` ⇒ `BASE` is cached in `const_integer_values` after `collect_annotated_constant` runs `remember_module_const_integer`, but `ctx.scope.define("BASE", Type::FixedInt(U8))` records the fixed-width type. When `ALIAS` is lowered, the `Name` arm's `matches!(scope_ty, Type::Int)` guard fires and returns `None`. `collect_annotated_constant` early-returns at the `let-else`, so `ALIAS` is never collected and never appears in `ctx.scope`. `main`'s use of `ALIAS` then surfaces as an undefined-variable diagnostic at the regular HIR resolution pass instead of a synthesized `Type::Int` Name reaching codegen. Confirmed by the user's manual `sifr check` reproduction and pinned at HIR by [test_module_constant_export_does_not_retype_fixed_width_name_as_int](crates/sifr_hir/src/lower/expressions_tests.rs:430).
- `BASE: uint8 = 250\nLIMIT: int = BASE + 4` ⇒ The `BinOp` arm recurses: left = `lower_module_integer_const_expr(BASE)` returns `None` (same scope-type guard). The `?` shortcircuits and the BinOp returns `None`. `LIMIT` is not collected. Same outcome as above.
- `BASE: uint8 = 250\nNEG: int = -BASE` ⇒ The `USub` arm calls `lower_module_integer_const_expr(BASE)` which returns `None` and propagates through `?` before ever reaching `negate_module_integer_const_expr`. `NEG` is not collected.

I also verified the symmetric all-`int` case still works:

- `BASE: int = 250\nLIMIT: int = BASE + 4` ⇒ `BASE` is cached at 250, `ctx.scope.define("BASE", Type::Int)`. `LIMIT`'s BinOp arm: left → `Some(Name{name:"BASE", ty:Int})`, right → `IntLiteral(4)`, both `Type::Int`, returns `Some(BinOp{...})`. `remember_module_const_integer` evaluates to 254. Pinned at HIR by [test_module_constant_export_uses_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:399) and end-to-end by the e2e fixture.
- `BASE: int = 10\nNEGATIVE: int = -(BASE + 3)` ⇒ inner BinOp lowers to `BinOp{Name(BASE,Int), "+", IntLiteral(3), Int}`; `negate_simple_expr(BinOp)` returns `None`; the `or_else` arm matches `Type::Int` and wraps in `UnaryOp{-, BinOp, Int}`; `const_integer_value` evaluates the `"-"` arm → -13. Pinned by [test_module_constant_export_uses_unary_prior_const_name](crates/sifr_hir/src/lower/expressions_tests.rs:413).

The scope-type guard correctly leverages the fact that `collect_annotated_constant`/`collect_bare_constant` define the declared type *after* `remember_module_const_integer` but before the next iteration's lowering call, so the cache and scope type are consistent. `ctx.scope.lookup(...).ty` (the declared type, not `effective_type`) is the right field — narrowing is not applied at module scope during initial collection.

## Criteria from the pass-4 brief

1. **Module const name reuse should work for prior `int` constants through unary/binop forms** — **met**. The all-`int` paths are exercised by the two new HIR tests and by the e2e fixture (`BASE_LIMIT: int = 250` / `LIMIT: int = BASE_LIMIT + 4` / `NEGATIVE_LIMIT: int = -(MAX_RETRIES + 10)`).
2. **Module const name reuse should not retype fixed-width constants as int** — **met**. The scope-type guard (`matches!(scope_ty, Type::Int)`) refuses to synthesize an `int`-typed `Name` for a fixed-width source. Pinned by [test_module_constant_export_does_not_retype_fixed_width_name_as_int](crates/sifr_hir/src/lower/expressions_tests.rs:430), which asserts both that the source `BASE: uint8 = 250` is preserved and that `LIMIT` is *not* collected.
3. **Module-level fixed-width and `int` over-budget expressions should emit clean HIR diagnostics and not duplicate diagnostics** — **met**. For the `uint8 = 10 ** 5000` path, `validate_fixed_width_initializer` evaluates and emits exactly one `SIFR-INT-0004`, then the `error_count_before_initializer` guard at [module_constants_lowering.rs:53](crates/sifr_hir/src/lower/module_constants_lowering.rs:53) skips `remember_module_const_integer`. For the `int = 10 ** 5000` path, validation is `NotConst` (no follow-on `TYPE_MISMATCH`) and `remember_module_const_integer` emits exactly one `SIFR-INT-0004`. Pinned by [test_module_fixed_width_const_expression_budget_has_int_code_once](crates/sifr_hir/src/lower/expressions_tests.rs:435) and [test_module_int_over_budget_const_expr_stays_hir_diagnostic](crates/sifr_hir/src/lower/expressions_tests.rs:466) — the latter now also pins `errors.len() == 1`, addressing pass-3 N1.
4. **The e2e module constants fixture should still smoke-test codegen for the all-`int` name/binop/unary shapes** — **met**. `BASE_LIMIT + 4` exercises BinOp + Name; `-(MAX_RETRIES + 10)` exercises UnaryOp + BinOp + Name. The `assert str(LIMIT) == '254'` and `assert str(NEGATIVE_LIMIT) == '-13'` checks tie the runtime values down.

## Determinism / duplication check

I retraced diagnostic emission counts for the relevant module-scope cases under the new code:

- `LIMIT: uint8 = 10 ** 5000`: `validate_fixed_width_initializer` → `evaluate_pow` → `reject_if_over_budget` emits one `SIFR-INT-0004` and returns `Rejected`. The `error_count_before_initializer` guard skips `remember_module_const_integer`. ✓ One emission.
- `LIMIT: int = 10 ** 5000`: `validate_annotated_constant_initializer` is a no-op (`NotConst`, types match); `remember_module_const_integer` evaluates and emits one `SIFR-INT-0004`. ✓ One emission, total `errors.len() == 1` pinned.
- `LIMIT: int = -(10 ** 5000)`: `negate_module_integer_const_expr` wraps the inner BinOp in `UnaryOp("-", BinOp, Int)`; `const_integer_value`'s `"-"` arm propagates `Rejected` from the inner pow evaluation, so the budget diagnostic still emits once.
- `LIMIT: int = 1 << 99999`: `evaluate_left_shift` short-circuits via `MAX_EXACT_SHIFT_OR_EXPONENT` and emits one `SIFR-INT-0004`.
- `BASE: uint8 = 250\nLIMIT: int = BASE + 4`: scope-type guard returns `None` from the `Name` arm; `collect_annotated_constant` early-returns at the `let-else`. `LIMIT` is not collected and not `define`d in scope. No new diagnostic at this site; `main`'s use of `LIMIT` surfaces a clean undefined-variable error at the regular pass.
- `BASE: int = 254` shadowed by an inner-scope `BASE: int = 100` then `value: uint8 = BASE + 1`: pre-existing path; `is_shadowed_by_inner_scope` short-circuits to `Unsupported`, single `TYPE_MISMATCH`, no range diagnostic. ✓

No duplicate-emission concerns introduced.

## Scope drift

- `simple_expr.rs` exposes `negate_simple_expr` and `integer_binop_source` as `pub(super)`. Tightly scoped reuse, no drift.
- `lower_module_integer_const_expr` only adds the `Name`/`UnaryOp`/`BinOp` cases needed for module-level cross-const folding and falls through to `lower_integer_const_expr_simple` for leaf shapes. No drift into general expression lowering.
- E2E fixture additions are localized and assert structurally; no unrelated test deletions or renames.
- The pass-3 fix is a single inserted scope-type check and an early `?` on `ctx.scope.lookup(...)`. Minimal surface change.

## Non-blocking findings

### N1 — `test_module_constant_export_does_not_retype_fixed_width_name_as_int` does not assert the user-visible diagnostic

[test_module_constant_export_does_not_retype_fixed_width_name_as_int](crates/sifr_hir/src/lower/expressions_tests.rs:430) verifies that `LIMIT` is absent from `module.constants`, but the `def main(): print("ok")` body does not reference `LIMIT`, so it doesn't pin the *user-visible* outcome (undefined-variable diagnostic at the call site). The brief calls this out in the validation note, but it's only manually verified. A second test asserting that `BASE: uint8 = 250\nLIMIT: int = BASE + 4\n\ndef main():\n    print(str(LIMIT))\n` returns `Err` with an undefined-variable diagnostic on `LIMIT` would harden against future drift in the regular-pass resolution path interacting with this slice.

This is a hardening nit, not a blocker — the structural assertion already pins the load-bearing invariant (no synthesized `Type::Int` Name for a fixed-width source).

### N2 — Bare unary on a name (`-BASE`) has no focused test

The new tests cover `LIMIT: int = BASE + 4` (BinOp + Name) and `NEGATIVE: int = -(BASE + 3)` (USub + BinOp + Name), but the simpler bare-unary case (`-BASE` with no inner BinOp) isn't exercised on its own. Tracing it: `lower_module_integer_const_expr(USub(Name))` recurses to `Name(BASE, Int)`, then `negate_module_integer_const_expr` falls through `negate_simple_expr` (which only matches literal variants) into the `or_else` arm and wraps in `UnaryOp("-", Name, Int)`. `const_integer_value`'s `"-"` arm evaluates correctly. Functionally fine, just not directly pinned. A one-line addition like `BASE: int = 5\nNEG: int = -BASE` in the e2e fixture (or a third HIR test) would close the gap.

### N3 — Pass-3 N3 (doc comment) still applies

`lower_module_integer_const_expr` and `negate_module_integer_const_expr` borrow `&LowerCtx` only for `const_integer_values` and (now) `scope`. A one-line note that they read only those fields and that callers are responsible for budget validation would help future readers — same suggestion as pass-2 N4 and pass-3 N3. Optional.

### N4 — Pre-existing `LargeIntLiteral` codegen panic still reachable

Untouched by this slice and also flagged in pass-3 N2: a literal in budget but exceeding `i64` (e.g., `LIMIT: int = 999999999999999999999999999999999999`) survives `lower_module_integer_const_expr` → `lower_integer_const_expr_simple` → `LargeIntLiteral`, then `try_lower_simple_module_constant_item_result_impl` returns `Ok(None)`, and `emit_module_constants` panics at [crates/sifr_codegen/src/module_constants.rs:12](crates/sifr_codegen/src/module_constants.rs:12). Out of scope for this slice (tied to SifrInt wiring under INT-1/INT-3 wave 2), but worth tracking explicitly so it doesn't get lost behind the pass-3 fix.

## Validation reproduced

I re-traced rather than re-ran the user's listed validations. The cited results are consistent with the code:

- `cargo fmt` — code is fmt-clean.
- `cargo test -p sifr_hir fixed_width -- --nocapture` — the `fixed_width` test names cover the budget/range/folding paths the diff touches.
- `cargo test -p sifr_hir module_ -- --nocapture` — the four new `test_module_*` tests pin all four pass-4 criteria.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` — the new fixture asserts `LIMIT == 254` and `NEGATIVE_LIMIT == -13`, which match the BigInt evaluation traced above.
- `sifr check` on the three pass-3 mixed-type repros — the scope-type guard makes those `LIMIT`/`ALIAS`/`NEG` constants un-collected, so `main`'s reference resolves to an undefined-variable diagnostic at the regular pass.
- `sifr check` on `LIMIT: int = 10 ** 5000` — `remember_module_const_integer`'s budget gate still fires once.

## Verdict

**Satisfied with non-blocking suggestions.** The pass-3 blocker is closed — `lower_module_integer_const_expr` now requires the scope-resolved type of a reused module-const name to be `Type::Int` before synthesizing `HirExpr::Name { ty: Type::Int }`, so mixed-type reuse falls back to today's "undefined variable" surface at the regular pass instead of producing rustc errors from valid-looking Sifr source. All four explicit pass-4 criteria are met, the new HIR tests pin both the positive `int`-reuse paths and the negative mixed-type guard, and the e2e fixture covers codegen for the new shapes. The non-blocking suggestions (broader regression test for the user-visible diagnostic on mixed-type reuse, optional bare-unary coverage, doc comment, and the pre-existing in-budget `LargeIntLiteral` codegen panic) can land separately or stay as follow-ups; none of them gates merge.
