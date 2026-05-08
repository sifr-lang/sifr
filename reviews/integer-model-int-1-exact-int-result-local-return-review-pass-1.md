# Review: INT-1 Local Result Binding Return Slice

## Verdict: SATISFIED

The slice is correct and ready for commit. No blockers identified.

---

## Change Summary

**`function_emitter.rs`:**
- `function_returns_result_sifr_int` now collects `Result[int, DivisionError]`-shaped local bindings via `collect_sifr_int_result_local_bindings`
- Returning one of those bindings marks the function as `Result<SifrInt, DivisionError>`
- Assignment invalidation removes promotion when the RHS is not a SifrInt Result expression

**`exact_int_floor_mod_result_return.sifr`:**
- `divide_local()`: `value = a // b`, `return value`
- `divide_local_again()`: `value = divide_local(a, b)`, `return value`
- `main` try-unwraps the chained return and asserts `3`

---

## Soundness Analysis

### Local Binding Collection (`collect_sifr_int_result_local_bindings`)

| Aspect | Assessment |
|--------|------------|
| `LOCAL_SCOPE_ONLY` traversal | **Correct** — nested functions are scope boundaries; no cross-contamination |
| Iterative collection | **Correct** — bindings accumulate during single left-to-right pass; order is sufficient |
| Type check via `is_result_int_type` | **Correct** — matches `Result[Int, DivisionError]` or `Result[LiteralInt, DivisionError]` |
| Assignment invalidation | **Correct** — `remove` only when `hir_expr_returns_sifr_int_result` returns false |
| `hir_expr_returns_sifr_int_result` matches | `//`, `%` binops; calls to `result_function_returns`; `HirExpr::Name` refs |

The collection order is sufficient: statements before a return statement are always visited before that return during `walk_stmts`, so by the time return analysis runs, all Let-bindings in scope have been collected.

### Assignment Invalidation

```
value: Result[int, DivisionError] = a // b  # inserted
value = other_result_var                   # kept (Name matches)
value = some_int                           # removed
value = a // b                             # re-inserted
```

The `if result_bindings.contains(name)` guard ensures only already-promoted bindings are invalidated. This correctly models the semantics: once a Result[int] binding holds a promoted value, reassigning from another promoted binding keeps it; reassigning from a non-promoted source removes it.

### Return Path

`function_returns_result_sifr_int` uses `LOCAL_SCOPE_ONLY` traversal with `hir_expr_returns_sifr_int_result(value, result_function_returns, &local_result_bindings)`. Since `local_result_bindings` is built by the same traversal order guarantee, the return expression sees the complete binding set.

---

## Missing Coverage (Acceptable for This Slice)

The stated scope is: *"a function returning a local Result[int, DivisionError] binding that was initialized from an exact floor/mod result or from another promoted Result-returning helper."*

The e2e test covers both sub-cases:
1. `divide_local()` — direct `//` assignment (floor/mod result)
2. `divide_local_again()` — chained promoted helper call

**Not covered** (out of scope per the stated intent):
- Reassignment patterns (e.g., `value = a // b; value = 42; return value`) — correctly handled by the invalidation logic, but not exercised
- Conditional returns from promoted bindings (e.g., `if cond: return value; return Err(...)`)
- Nested function returning a promoted local binding from its outer scope — requires `INCLUDE_NESTED_FUNCTIONS` and a separate tracking mechanism, out of scope for this slice

---

## Edge Case: Assignment Invalidation is Coarse

The invalidation check is:
```rust
HirStmt::Assign { name, value }
    if result_bindings.contains(name)
        && !hir_expr_returns_sifr_int_result(value, result_function_returns, &result_bindings)
```

This means assigning from any non-SifrInt-Result expression removes promotion, including:
- Function calls to helpers that *happen* to return a non-Result or `Result[non-int], _`
- Complex expressions containing a promoted binding name

Example:
```python
def helper() -> int: ...
value: Result[int, DivisionError] = a // b
value = helper()  # removed by invalidation, even though helper() is safe
return value      # still returns Result[int, DivisionError], not Result[SifrInt, DivisionError>
```

This is **coarse but correct** for this slice: the function returns `Result[int, DivisionError]` which is the correct type when the value isn't SifrInt. The Rust codegen will produce `Result<i64, DivisionError>` for the `int` arm, matching the declared Sifr type. The only consequence is lost precision in the promoted return set, not a type mismatch.

---

## Validation Results

```
cargo check -p sifr_codegen                   ✓
cargo test -p sifr_codegen function_emitter   ✓ (6 tests)
cargo test -p sifr_hir -- --skip test_e2e_pass ✓ (481 tests)
cargo test -p sifr -- --skip test_e2e_pass    ✓ (32 tests)
scripts/run_all_tests.sh --profile quick      ✓ (e2e: 24/24 passed)
SIFR_E2E_FIXTURE_MANIFEST=<manifest> cargo test -p sifr -- test_e2e_pass ✓ (exact_int_floor_mod_result_return)
cargo fmt --check                             ✓
```

Generated Rust output (emitted) confirms:
- `divide_local()` → `Result<SifrInt, DivisionError>`
- `divide_local_again()` → `Result<SifrInt, DivisionError>`
- `main` try-unwraps produce `SifrInt` in success branches

---

## Recommendation

**Satisfied.** The slice is minimal, correct, and sufficiently tested. Commit as-is.