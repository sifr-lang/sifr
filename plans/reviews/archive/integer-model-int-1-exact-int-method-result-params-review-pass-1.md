# Review: INT-1 Exact Int Method Result Parameters

## Summary
The implementation promotes class method `Result[int, E]` parameters to `Result<SifrInt, E>` when fixed-point analysis proves call sites pass exact integer result expressions. This follows prior merged work for exact int Result returns, locals, class method returns, field receiver calls, and nested field receiver calls.

## Architecture

### Fixed-Point State
- **New state**: `sifr_int_result_method_params: HashMap<String, HashSet<usize>>` maps method keys to promoted parameter indices
- **Termination**: Loop breaks when all three state maps (`function_params`, `result_function_params`, `result_method_params`) stabilize

### Discovery Flow
1. **Module-level**: `collect_sifr_int_result_call_arg_method_params` traverses module functions, finding method calls with exact-result arguments
2. **Method-level**: Same traversal applied to class method bodies, seeding `result_function_params` from promoted method params
3. **Seeding**: `function_returns_result_sifr_int` receives `initial_result_bindings` (promoted method params) to properly analyze method body returns

### Lowering Integration
- **Parameter lowering**: `class_method_emitter.rs` checks `method_param_lowers_to_sifr_int_result` before type lowering
- **Local registration**: Promoted method params registered as exact-result locals before body lowering
- **Call-site coercion**: `stmt_support_emitter.rs` and `expr_render_helpers.rs` apply coercion for receiver class and method param index

## Verification

### Generated Code (key excerpts)
```rust
// ResultBox.pass_result parameter lowered to Result<SifrInt, DivisionError>
fn pass_result(&self, value: Result<SifrInt, DivisionError>) -> Result<SifrInt, DivisionError> {
    return value;
}

// Call site coercion applied
let method_arg: Result<SifrInt, DivisionError> = result_box.pass_result(divide(36 as i64, 12 as i64));
```

### Test Coverage
- New fixture `ResultBox.pass_result` method with `Result[int, DivisionError]` parameter
- Passthrough in `main()`: `result_box.pass_result(divide(36, 12))`
- All prior exact-int fixtures pass (verified with `cargo run -q -p sifr -- emit`)

### Pre-existing Failures (not introduced by this change)
- `with_multiple` e2e fixture: Rust compilation error (`cannot find function map/list/filter`) - exists on `main`
- 22 `sifr_codegen` unit test failures - exist on `main`

## Assessment

**Satisfactory to merge.** The implementation is sound:

1. **Fixed-point correctness**: New state properly tracked, loop terminates when all maps stabilize
2. **Call-site coercion**: Applied at correct lowering layers (stmt_support_emitter + expr_render_helpers)
3. **Local registration**: Promoted method params registered before body lowering for proper alias analysis
4. **Seeding correctness**: `function_returns_result_sifr_int` receives promoted method param bindings
5. **Coverage**: Test fixture demonstrates full passthrough from `divide(...)` through method call

No concrete blockers identified.
