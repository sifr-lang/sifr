

## Code Review: Compiler Milestone M7

### OverviewThis PR fixes two codegen issues exposed by PR #2218's `.take()` optimization for owned recursive optional fields:

1. **Mutable locals**: Recursive-node locals from non-empty pops were immutable, but `.take()` requires `mut`
2. **Borrowed optional wrapping**: Helper calls like `nodeValue(node)` wrapped locals before reuse, moving them incorrectly

---

### 1. Correctness of Codegen Ownership/Mutability Rules

**`should_force_mutable_binding`** (`call_args_and_returns.rs:244-285`, `plain_call_args.rs:417-458`):
- `class_has_recursive_option_field` correctly identifies classes where a field's type is a Union containing the class itself
- The fix ensures local bindings of such classes are marked `mut`, enabling `.take()` on recursive fields
- Logic is sound and scoped to the specific ownership issue**Optional parameter wrapping** (`simple_dispatch_and_bindings.rs:66-86`, `expr_call_and_literal_helpers.rs:78-98`):
- Correctly distinguishes borrowed vs owned arguments: `borrowed_name_arg` tracks whether the name is in `borrowed_params`/`mut_borrowed_params` or has a `&` type
- For borrowed name args into optional params: clones before wrapping (`Clone(Box::new(lowered_arg))`)
- For owned args: moves directly (`lowered_arg`)

---

### 2. Borrowed vs Owned Optional Wrapping

**Structured path** (`simple_dispatch_and_bindings.rs:70-78`):
```rust
let wrapped_inner = if param_is_owned_rust_value && !borrowed_name_arg {
    lowered_arg  // move} else if matches!(hir_arg, HirExpr::Name { .. })
    && !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty)
{
    crate::RustExpr::Clone(Box::new(lowered_arg))  // clone
} else {
    Self::clone_non_copy_name_expr_for_ir(hir_arg, lowered_arg)
};
```

**Registry path** (`expr_call_and_literal_helpers.rs:82-90`):
```rust
if (!param_is_owned_rust_value || borrowed_name_arg)
    && !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty)
{
    lowered_arg = crate::RustExpr::MethodCall { // clone        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
        method: "clone".to_string(),
        args: vec![],
    };
}
```

Both paths correctly protect owned recursive optional wrapper behavior. Verified by `test_owned_recursive_option_field_moves_without_tail_clone` and `test_owned_recursive_option_field_take_preserves_parent_use`.

---

### 3. Mutable Binding Scope

`should_force_mutable_binding` is conservative:
- Only applies to classes with recursive optional fields (Union containing self)
- Does not affect iterators or defaultdict aliases (pre-existing cases)
- Does not hide deeper issues—enables the correct `.take()` pattern for child field moves

---

### 4. Test Coverage

| Test | Purpose |
|------|---------|
| `test_owned_recursive_option_field_moves_without_tail_clone` | PR #2218 regression |
| `test_owned_recursive_option_field_take_preserves_parent_use` | PR #2218 regression |
| `test_borrowed_recursive_option_field_still_clones` | Owned vs borrowed distinction |
| `test_local_recursive_node_binding_is_mutable_for_child_moves` | New: mutable binding fix |
| `test_borrowed_optional_wrapper_clones_recursive_node` | New: borrowed wrapper fix |

All5 tests pass. Validation output confirms generated code has `let mut node` and `node.left.take().map(...)`.

---

### 5. Scope Creep / Maintainability

- `classes_and_basics_codegen_tests.rs`: 768 lines (under 900-line cap)
- `recursive_node_codegen_tests.rs`:172 lines (new focused module)
- No unrelated changes; all modifications directly address the two codegen issues
- `git diff --check`, `check_hir_maintainability_guardrails.py`, `check_file_size_guardrails.py` all pass

---

### Validation Summary

- `cargo test -p sifr_codegen recursive_node_codegen_tests`: 5 passed
- `cargo test -p sifr_codegen recursive_option_field`: 3 passed
- `scripts/run_all_tests.sh --profile quick`: exit 0 (LSP stress is unrelated flaky test)
- `cargo build -p sifr`: exit 0
- Generated code verified: `let mut node`, `.take()` calls, cloned `nodeValue(&Some(...))`

---

**APPROVED**
