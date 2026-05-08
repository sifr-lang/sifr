

---

## Review Verdict: Satisfied

### Summary

The changes correctly implement recursive lowering of structured field access receivers to enable `self.holder.calc.divide(...)` lowering without compile_error. The fix is minimal, focused, and composes correctly with existing field access and method call handling.

### Code Review

**`crates/sifr_codegen/src/expr_render_helpers.rs` lines 762-784**

The change adds a fallback to `self.lower_stmt_expr_for_ir(object)` when `try_lower_leaf_or_name_expr_result` returns `None`. For a nested field receiver like `self.holder.calc` (FieldAccess of FieldAccess of Name), the first call to `try_lower_structured_field_access_expr`:
1. `try_lower_leaf_or_name_expr_result(object)` → `None` (it's a FieldAccess, not a leaf/name)
2. Calls `self.lower_stmt_expr_for_ir(object)` → recursively lowers `self.holder.calc` to `self.holder.clone().calc`
3. Then applies `lower_field_access_expr_with_lowered_object` with the already-lowered receiver

This correctly produces `self.holder.clone().calc` for the nested field access.

**Composition with Result promotion**: The recursive lowering produces the correct `RustExpr` structure (`RustExpr::Field { expr: RustExpr::MethodCall { ... }, field: "calc" }`). When this is used as the receiver in `lower_stmt_expr_for_ir` for the MethodCall, it flows correctly into the Result promotion system from the previous PR.

**Safety**: The recursion terminates because each recursive call to `try_lower_structured_field_access_expr` works on the inner `object` field, which is structurally smaller in the AST. Eventually it reaches a `HirExpr::Name` which is handled by the first branch.

**`crates/sifr/tests/e2e/pass/exact_int_floor_mod_result_return.sifr`**

The test adds `Wrapper` class with `divide_nested_field` covering `self.holder.calc.divide(a, b)` returning `Result[int, DivisionError]`. The test:
- Constructs `Wrapper` from a `Holder`
- Calls `wrapper.divide_nested_field(30, 10)` expecting `3`
- Covers the missing production path (nested FieldAccess receiver → MethodCall → Result binding)

### Validated

- `cargo fmt --check` passes
- `cargo check -p sifr_codegen` passes  
- `cargo run -q -p sifr -- emit` confirms nested field generates `self.holder.clone().calc.divide(a, b)` with no compile_error
- E2E suite: 32 passed, 1 failed (pre-existing `callable_apply_twice` issue, not introduced by this change)
- Isolated nested field test confirms general case works (`self.inner.get()` → `self.inner.clone().get()`)

### No Blockers

The slice is acceptable for INT-1. The approach is sound, minimal, and the test coverage is appropriate.
