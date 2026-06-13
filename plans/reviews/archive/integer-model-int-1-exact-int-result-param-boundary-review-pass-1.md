

## Review: INT-1 Result Param Boundary Slice

**Verdict: SATISFIED**

### Summary

The slice correctly implements promotion of `Result[int, E]` parameters to `Result<SifrInt, E]` when call sites pass promoted result expressions. The generated Rust for the test fixture is valid and runs correctly.

### Detailed Findings

**1. Fixed point interaction with result function return promotion**

The loop at `function_emitter.rs:155–239` correctly converges when all three propagate:
- `function_returns` (plain int)
- `result_function_returns` (Result[int, E])
- `function_params` (plain int params)  
- `result_function_params` (Result[int, E] params)

The iteration order is sound: result params are discovered from result function returns, then result function returns are re-computed (including seeding from result params), then params are re-discovered. This mirrors the pattern used for plain int params/returns.

**2. Seeding `function_returns_result_sifr_int` with promoted result param names**

`sifr_int_result_function_params` is correctly threaded through to `function_returns_result_sifr_int` at line 1042. The seed at lines 1054–1059 correctly combines:
- Nested helper returns (via `collect_nested_sifr_int_result_function_returns`)
- Promoted result param passthrough bindings (via `collect_sifr_int_result_function_param_names` → `collect_sifr_int_result_local_bindings_with_initial`)

A function parameter that receives `Result<SifrInt, E>` and returns it directly (like `pass_result`) is correctly identified as needing promotion.

**3. Call argument coercion is correctly scoped**

The coercion at `expr_render_helpers.rs:268–270` fires only when:
- The called function has a promoted result param at that index
- The argument is a known `Result<SifrInt, E]` expression (local binding, promoted call, or `//`/`%`)

`coerce_result_int_expr_to_sifr_int_value` is conservative: it creates new `Ok(SifrInt::from_i64(...))` wrappers only for bare `Ok(...)` calls, and passes through already-promoted result expressions unchanged. This is correct.

**4. No state save/restore or nested-scope issues**

- `sifr_int_result_function_params` is a simple `HashMap<String, HashSet<usize>>`, populated monotonically in the fixed-point loop. No stack/unwind needed.
- `collect_sifr_int_result_call_arg_function_params` uses `TraversalConfig::LOCAL_SCOPE_ONLY`, correctly avoiding nested function bodies when discovering param→call-site bindings.
- The `collect_sifr_int_result_function_param_names` helper safely returns `HashSet::new()` when the function has no promoted result params.

**5. Test fixture correctness**

The generated `pass_result`:
```rust
fn pass_result(value: Result<SifrInt, DivisionError>) -> Result<SifrInt, DivisionError> {
    return value;
}
```
correctly receives `Result<SifrInt, DivisionError>` and returns `Result<SifrInt, DivisionError>`. Both test cases (`pass_result(result_arg)` and `pass_result(divide(18, 6))`) compile and run.

### Pre-existing e2e failures

The group-level compilation failures (`with_break`, `with_early_return`, etc.) are pre-existing — they fail identically without this slice due to missing Rust stdlib imports (`map`, `list`, `filter`). These are unrelated to INT-1.

### No blockers identified. The slice is acceptable for INT-1.
