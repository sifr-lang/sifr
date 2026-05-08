

Based on my detailed review, I found one blocker and several non-blocking observations.

---

## Review: INT-1 Integer Model — Exact Int Floor Mod Result Return

### Blocker: Nested-Function Recursive Return Type (`function_emitter.rs:464-467`)

When emitting a recursive `RustStmt::LocalFn` (nested function with a self-reference), the `ret` field ignores `nested_returns_sifr_int_result`. Only `nested_returns_sifr_int` is checked:

```rust
ret: if nested_returns_sifr_int {
    Some(RustType::Named("SifrInt".to_string()))
} else {
    self.lower_function_return_type(func, false)  // ← result_int path not considered
},
```

`lower_function_return_type` does check `sifr_int_result_function_returns` (line 630-636), but at this point in `emit_nested_function`, the nested function's name **has already been inserted** into that set (lines 385-388). So this call will find it and return the correct type — but only by accident, through the side-effect of lines 385-388. This is fragile.

**Fix**: Extend the conditional at line 464 to also check `nested_returns_sifr_int_result`:

```rust
ret: if nested_returns_sifr_int {
    Some(RustType::Named("SifrInt".to_string()))
} else if nested_returns_sifr_int_result {
    Some(result_int_return_type_to_sifr_int(&func.return_type))
} else {
    self.lower_function_return_type(func, false)
},
```

This makes the recursive nested-function return-type path explicit rather than relying on the lookup succeeding because of a prior side-effect.

---

### Non-blocking observations

**1. `function_returns_result_sifr_int` is narrower than `hir_function_returns_sifr_int_with_extra_forced_and_shadowed`**

`hir_function_returns_sifr_int` builds a forced-locals set that includes bindings from try-unwrapping promoted functions (`collect_sifr_int_forced_locals`). `function_returns_result_sifr_int` only checks direct `//`/`%` and calls. This is intentional: `Result[int, DivisionError]` functions shouldn't force locals through try-unwrapping *inside* the function (the return is the promotion point). However, if a future case emerges where a `Result[int, DivisionError]` function does `x = promoted_func(); return x`, detection will silently miss it until a fixed-point loop iteration picks it up. The fixed-point loop mitigates this, but it's worth documenting the asymmetry.

**2. Module-level fixed-point loop (`register_sifr_int_function_returns`) correctly handles mutual recursion**

Both `function_returns` and `result_function_returns` are part of the fixed-point convergence check (line 208-210), and the second pass over `result_function_returns` (lines 177-185) is ordered after the first pass over `function_returns`. This means cross-function mutual recursion between plain-SifrInt and Result-SifrInt functions is handled correctly.

**3. `coerce_result_int_expr_to_sifr_int_value` is appropriately narrow**

The coercion only rewrites `Ok(...)` (with recursive `coerce_expr_to_sifr_int_value` on the arg) and passes through SifrInt Result expressions unchanged. This avoids over-coercion. The `is_sifr_int_result_expr` predicate on line 1470 acts as an identity guard.

**4. No test for try-unwrapping inside a promoted `Result[int, DivisionError]` function body**

The e2e fixture covers direct `//`/`%` returns, chained calls, error handling, and mixed `Ok(1)`/`Ok(checked_floor_div(...))` branches — but no case where a promoted function does a try-unwrap on an earlier call and returns the unwrapped value. This is the gap described in observation #1 above.

**5. `is_result_int_division_error_type` and `is_result_int_type` are nearly identical**

Both check `Result(ok: Int|LiteralInt, err: DivisionError)` / `Result(ok: Int|LiteralInt, err: _)`. They should share a helper to avoid drift if the type system evolves.

---

### Verdict

**Not ready to merge** due to the nested-function recursive return type fragility. The code currently works by accident (the side-effect lookup succeeds), but it's not semantically sound. The fix is a two-line change on `function_emitter.rs:464-467`.

All other aspects — state save/restore, fixed-point convergence, Ok branch coercion, caller/try-unwrapping consistency, and e2e coverage — are correct. Once the nested-function fix is applied, this slice is acceptable for INT-1.
