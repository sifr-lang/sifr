

Looking at this diff for DIAG-10 error-tainted poisoned bindings and related canonical diagnostics, I'll analyze systematically.

## Severity: Medium

### `crates/sifr_hir/src/lower/expressions.rs:327-332` — Unary operations not covered by cascade suppression

The diff adds poisoned binding checks to `lower_binop` but not to `lower_unary`. If a poisoned binding is used in a unary expression (e.g., `-s` where `s` is poisoned), the suppression mechanism won't fire:

```rust
// lower_unary (not shown in diff) - no is_poisoned_binding check
pub(super) fn lower_unary(unary: &ExprUnary, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;
    // ...
}
```

## Severity: Low/Missing Coverage

### `crates/sifr_hir/src/lower/expressions_tests.rs:189-210` — Test gap: cascade suppression only verified for binary ops

The test `test_poisoned_initializer_binding_suppresses_followup_operator_cascade` only covers:
- `s = xs[0] + xs[0]` (failed initializer with binop)
- `return s + 1` (single subsequent binop)

Missing test cases for cascade suppression:
1. **Chained operators**: `s = xs[0] + xs[0]; t = s + s + s` — to verify suppression holds across multiple references
2. **Unary suffix**: `s = xs[0] + xs[0]; return -s` — to verify unary operations don't bypass suppression
3. **Non-operator expressions**: `s = xs[0] + xs[0]; print(s)` — subscript/call use of poisoned binding

### `crates/sifr_hir/src/lower/statements.rs:1319-1335` — `failed_initializer_taint` panic path is an internal invariant

If `ctx.error_taint_since()` returns `None` (which should be impossible if errors were properly emitted), the code panics with `INTERNAL_COMPILER_PANIC`. This is a reasonable invariant but undocumented.

## Severity: Informational

### `crates/sifr_hir/src/scope.rs:27-35` — `ErrorTaint` is a zero-sized marker

`ErrorTaint(())` carries no information beyond "an error was emitted." This is intentional (suppression doesn't need to distinguish which error poisoned a binding), but worth noting for future extensibility.

### `crates/sifr_hir/src/lower/mod.rs:275-280` — `error_taint_since` has subtle edge case

```rust
fn error_taint_since(&self, previous_error_count: usize) -> Option<ErrorTaint> {
    (self.errors.len() > previous_error_count)
        .then_some(self.last_error_taint)
        .flatten()
}
```

If multiple errors are emitted between the call to `error_count()` and `error_taint_since()`, this returns the **most recent** error's taint, not the one that occurred first. This is likely correct behavior, but the semantics should be documented.

## What's Working Correctly

1. **Canonical diagnostic codes**: `TYPE_CONTAINER_ELEMENT_CONFLICT`, `TYPE_MISMATCH` are properly used throughout
2. **ErrorTaint flow**: `error()` and `error_with_code_at()` properly track `last_error_taint`
3. **Poison propagation**: `seed_binding_after_failed_initializer` → `define_poisoned_local` → `is_poisoned_binding` chain is sound
4. **Binop suppression logic**: The check `.any(|expr| matches!(expr, HirExpr::Name { name, .. } if ctx.is_poisoned_binding(name)))` correctly short-circuits

## Summary

**Blocking findings**: No blocking bugs identified.

**Residual test gaps**: The cascade suppression mechanism is only tested for binary operations. Unary operations and non-operator expression types (function calls, subscripts) using poisoned bindings are not covered by existing tests.
