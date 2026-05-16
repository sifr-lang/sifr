Looking at the fix for the Trees group blocker.

## Findings

### 1. Correctness of Root-Cause Fix

**Yes, this is correct.** The fix addresses the exact mismatch:

- `nodes: List<TreeNode>` iterates to `TreeNode` items
- `treeToString` expects `&Option<TreeNode>` (the `T | None` union)
- Without fix: direct call `treeToString(item)` — type error
- With fix: closure `|__sifr_map_item| treeToString(&Some(__sifr_map_item))` — correct

The adaptation chain is sound:
1. `Some` wrapping when param is `T | None` but arg is `T`
2. Shared borrow `&` added per `ParamConvention::borrow()` and non-Copy param
3. Result: `&Some(__sifr_map_item)` matching the expected `&Option<TreeNode>`

### 2. Regression Risks

**Low risk.** The fix is opt-in via early returns:

```rust
fn lower_simple_map_callable_expr(callable: &HirExpr, iter: &HirExpr) -> Option<RustExpr> {
    let lowered_callable = try_lower_simple_callable_expr(callable)?;
    let Some((param_types, conventions)) = simple_callable_param_info(callable) else {
        return Some(lowered_callable);  // ← existing behavior preserved
    };
    if param_types.len() != 1 || conventions.len() != 1 {
        return Some(lowered_callable);  // ← existing behavior preserved
    }
    // ... adaptation logic only reached for single-param callables with known sig
}
```

Existing paths are preserved for:
- Non-closure callables
- Multi-arg callables
- Callables without resolvable param info
- Any case where adaptation isn't needed

**Minor observation**: `is_option_like_simple` and `resolve_alias_type` are used in `adapt_simple_map_callable_arg`. If `resolve_alias_type(param_ty)` returns a type that doesn't unwrap cleanly, the `Some` wrapping won't trigger — but this would only happen for malformed types, and the fallback path would emit the old (incorrect) code. Given the test coverage and Trees group passing, this case isn't occurring.

### 3. Test Coverage

The unit test is focused and correctly asserts the closure structure with `Some` wrapping and shared borrow. The pattern matched is exactly the emitted code shown in verification:
```rust
map(|__sifr_map_item| treeToString(&Some(__sifr_map_item)))
```

No additional test rounds needed for this fix.

---

## Recommendation

**This fix is satisfactory to proceed.** The implementation is correct, the fix is narrow and safe, and it has passed all verification gates (unit test, build, Trees group full pass).
