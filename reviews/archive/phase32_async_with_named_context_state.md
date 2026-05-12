# Code Review: Phase 32 Named Async Context State Slice

**Review scope:** User-defined `async with` direct-name lowering and nested cleanup order fixture.

**Branch:** `phase32-async-with-named-context-state`
**Base:** PR #2028 user-defined async context manager normal-exit support

---

## Summary

The slice makes two targeted changes:

1. **`stmt_support_emitter.rs`**: For `HirAsyncWithKind::UserDefined` where `context` is a `HirExpr::Name`, codegen calls `name.__aenter__()` and `name.__aexit__()` directly instead of materializing a `__sifr_async_cm` temporary.

2. **`queries.rs`**: Named user-defined async-with context variables are marked as mutated so the binding emits as `mut` in Rust.

---

## Detailed Findings

### 1. `stmt_support_emitter.rs` — Direct-name lowering (lines 7495–7525)

**What changed:**
```rust
if let HirExpr::Name { name, .. } = context {
    // Generates: name.__aenter__().await?
    // and: name.__aexit__(&AsyncExitCause::Normal).await?
    ...
    return Ok(Some(RustStmt::Block(stmts)));
}
// Fallback: existing __sifr_async_cm temporary path
```

**Soundness assessment:**

- **Rust borrow rules**: `__aenter__` and `__aexit__` methods take `&mut self`. The generated binding `let mut outer = ...` gives Rust a `mut` binding, satisfying the method receiver requirement. Correct.
- **Sifr ownership**: The HIR `HirAsyncWithKind::UserDefined` carries `enter_value_ty`, `enter_error_ty`, and `exit_error_ty` — the binding's ownership tracking flows through these type annotations. The `context` name is never consumed or reassigned within the `async with` body; only methods are called on it. The change is semantically sound.
- **Non-name path preserved**: Complex expressions (e.g., `get_cm()`) still use the `__sifr_async_cm` temporary fallback. No regression for PR #2028 behavior.
- **No ownership regression**: The direct-name path does not move or consume the context variable — it borrows it for method calls. The original binding remains in scope with its value intact for post-block use (verified by fixture assertions).

**Generated Rust for named case** (`async_with_nested_cleanup_order.sifr`):
```rust
let mut outer: OrderedResource = OrderedResource::new(1);
{
    let outer_value = outer.__aenter__().await?;  // borrows &mut outer
    assert!(outer.entered);                        // visible inside block
    assert!(!outer.exited);
    outer.__aexit__(&AsyncExitCause::Normal).await?;  // borrows &mut outer
}
assert!(outer.exited);  // visible after block — state preserved
```

This is exactly what the fixture asserts. The pattern is sound.

---

### 2. `queries.rs` — Mutated marking for named async-with context (lines 282–291)

**What changed:**
```rust
HirStmt::AsyncWith {
    kind: sifr_hir::HirAsyncWithKind::UserDefined {
        context: HirExpr::Name { name, .. },
        ..
    },
    ..
} => {
    mutated.borrow_mut().insert(name.clone());
}
```

**Soundness assessment:**

- **Placement**: This is in `collect_mutated_vars` under `on_stmt`, which runs during function-level analysis (see `function_emitter.rs:968` — `self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures)`). It correctly marks the variable in the function's mutation set.
- **Effect**: When `self.mutated_vars.contains(&name)` is true, local binding codegen emits `let mut name` instead of `let name`. Verified at `lib.rs:2320`, `stmt_support_emitter.rs:5851`, `lower_stmt.rs:643`.
- **No over-marking**: The mutation is scoped to the immediate `HirExpr::Name` context variable — not the body, not unrelated bindings.
- **No undesirable side effects**: Marking a variable as mutated only affects `let` binding mutability in the local scope where `collect_mutated_vars` was called. It does not affect:
  - Parameter conventions (those use `effective_nested_param_convention` which runs separately for nested functions)
  - Borrow rules for method receivers
  - Task boundary sendability
- **Correctness of the marking**: Calling `__aenter__` and `__aexit__` mutates the receiver (`&mut self`), so marking the context variable as mutated is semantically accurate. Rust requires `let mut` on the binding for `&mut self` method calls.

---

### 3. Fixture `async_with_nested_cleanup_order.sifr`

**Coverage:**

| Assertion | What it verifies |
|---|---|
| `assert outer.entered` inside outer block | State visible inside block |
| `assert not outer.exited` before inner exit | Outer not exited before inner |
| `assert not outer.exited` inside inner block | LIFO: outer still active during inner |
| `assert inner.exited` after inner block | Inner exited |
| `assert not outer.exited` after inner block | Outer still active |
| `assert outer.exited` after outer block | Outer exited |
| `return self.value` from `__aenter__` | State preserved across entry value capture |

**LIFO order verification**: The generated Rust emits `inner.__aexit__()` before `outer.__aexit__()` — matching Python/Sifr cleanup semantics. The fixture asserts LIFO ordering at each checkpoint.

**Abnormal-exit deferral**: The fixture does not test cancellation/timeout/exception exit paths. Per milestone_async_7a, abnormal-exit cleanup is intentionally deferred. The fixture scope is appropriate for this slice.

**Fixture sufficiency**: The single nested fixture with explicit state assertions covers the core named-context semantics. Additional coverage (e.g., deeper nesting, re-nesting) can come in follow-up slices.

---

### 4. Non-regression relative to PR #2028

| Scenario | PR #2028 | This slice | Status |
|---|---|---|---|
| Named context `async with cm as x` | __sifr_async_cm temp | Direct `cm.__aenter__()` | Improved |
| Expression context `async with f() as x` | __sifr_async_cm temp | __sifr_async_cm temp | Preserved |
| `async with task.scope()` | Built-in handling | Built-in handling | Unchanged |
| Abnormal-exit cleanup | Deferred | Deferred | Unchanged |
| `async_with_basic.sifr` fixture | Passes | Passes | Regression: none |

**Validation evidence**: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/async_with_basic.sifr` and `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/async_with_nested_cleanup_order.sifr` both pass.

---

## Concerns

None identified. The changes are:

1. **Semantically sound**: Direct borrow method calls via `&mut self` receivers are valid Rust.
2. **Ownership-correct**: The context variable is not moved or consumed — it remains accessible after the block.
3. **Locally scoped**: Mutation marking only affects the immediate context binding's `let mut` annotation.
4. **Non-regressive**: Non-name contexts, built-in forms, and abnormal-exit behavior are preserved.

---

## Recommendation

The slice correctly implements named user-defined async context manager state preservation across normal exit. The LIFO cleanup order is demonstrated by the fixture. No changes required.

---

## Review Checklist

- [x] Direct-name lowering is sound for Rust borrow rules
- [x] Mutation marking has no undesirable side effects
- [x] Fixture covers named-context state preservation
- [x] Fixture does not mask deferred abnormal-exit cleanup
- [x] No regression relative to PR #2028

---

REVIEW_STATUS: SATISFIED