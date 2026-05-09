# Review: INT-3 Fixed-Width Add APIs — PR #1868

## Summary

PR #1868 adds the first slice of fixed-width instance add APIs: `checked_add`,
`wrapping_add`, `saturating_add`, and `overflowing_add` for same-width fixed-width
operands only. This is the first quarter of the INT-3 fixed-width arithmetic APIs
milestone.

---

## What was reviewed

| File | Role |
|---|---|
| `crates/sifr_hir/src/lower/fixed_width_arithmetic_methods.rs` | HIR type resolution |
| `crates/sifr_hir/src/lower/expressions.rs` (+4/-3) | HIR dispatch hook for `Type::FixedInt` |
| `crates/sifr_hir/src/lower/expressions_tests.rs` (+51) | HIR unit tests |
| `crates/sifr_codegen/src/methods/fixed_width.rs` | Codegen lowering to Rust primitives |
| `crates/sifr_codegen/src/methods/mod.rs` (+5) | Method dispatch routing |
| `crates/sifr/tests/e2e/pass/fixed_width_add_apis.sifr` | E2E fixture |

---

## Semantic correctness against INT-3

| Requirement | Status | Notes |
|---|---|---|
| `checked_add` → `Result[fixed-width, OverflowError]` | ✅ | `Type::Result(Box::new(fixed_ty), Box::new(overflow_error_type(ctx)))` |
| `wrapping_add` → same fixed-width | ✅ | Returns `fixed_ty` directly |
| `saturating_add` → same fixed-width | ✅ | Returns `fixed_ty` directly |
| `overflowing_add` → `tuple[fixed-width, bool]` | ✅ | `Type::Tuple(vec![fixed_ty, Type::Bool])` |
| Mixed-width operands rejected | ✅ | `arg_ty.resolve_alias() != fixed_ty.resolve_alias()` triggers `TYPE_MISMATCH` |
| Codegen: no `.unwrap()`/`.expect()` in user paths | ✅ | All four methods use only safe Rust primitives (`checked_add`, `wrapping_add`, `saturating_add`, `overflowing_add`). `checked_add` chains `.ok_or_else` which returns `Result`, never panics. |
| OverflowError message content | ⚠️ Minor | Message reads "fixed-width integer addition overflow". Matches the e2e fixture expectation. Acceptable. |

### OverflowError type construction

`overflow_error_type` does a `.get()` into `ctx.class_types` and falls back to an
inlined `Type::Class` definition. This is the same fallback pattern used elsewhere in
the codebase. It is safe.

### Double `.to_string()` in codegen

In `lower_checked_add` (fixed_width.rs:18-22):

```rust
RustExpr::MethodCall {
    receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
        "fixed-width integer addition overflow".to_string(),  // <- first .to_string()
    ))),
    method: "to_string".to_string(),                          // <- second .to_string()
    args: vec![],
},
```

This produces `"fixed-width integer addition overflow".to_string().to_string()` in
generated Rust. The inner `.to_string()` is redundant but harmless — it calls
`&str.to_string()` which is `String::from(s)` with no double-allocation. The outer
`.to_string()` converts the resulting `String` to `String` again. The final emitted
code is correct but wasteful. Not a blocker.

---

## Type and codegen coverage

| Fixed-width variant | `checked_add` | `wrapping_add` | `saturating_add` | `overflowing_add` |
|---|---|---|---|---|
| `int8` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `int16` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `int32` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `int64` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `uint8` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `uint16` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `uint32` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `uint64` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `isize` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |
| `usize` | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ | HIR ✅ codegen ✅ |

All ten `FixedIntType` variants are dispatched through the same `resolve_fixed_width_method_type`
function and the same codegen path. No variant-specific branches exist, so coverage is uniform.

---

## No-panic guarantee

Rust primitive `checked_add` returns `Option<T>`. The codegen wraps it with
`.ok_or_else(|| OverflowError { ... })` which converts `None` → `Err`. No `unwrap`,
`expect`, or `unwrap_unchecked` anywhere in the lowered code. ✅

---

## Test coverage

| Test | What it covers |
|---|---|
| `test_fixed_width_add_apis_have_representation_preserving_types` | All four method return types match INT-3 specification |
| `test_fixed_width_add_api_rejects_mixed_width_argument` | `int8.wrapping_add(int16)` → `TYPE_MISMATCH` with correct diagnostic message and span |
| `fixed_width_add_apis.sifr` e2e | Overflow path (`high.checked_add(one)` → `OverflowError`), non-overflow path (`low.checked_add(one)` → `int8`), `wrapping_add`, `saturating_add`, `overflowing_add` |

The e2e fixture covers:
- `wrapping_add` overflow: `int8(127).wrapping_add(1)` → `int8(-128)`
- `saturating_add` overflow: `int8(127).saturating_add(1)` → `int8(127)`
- `overflowing_add` overflow: returns `(wrapped_value, true)`
- `checked_add` overflow path (try/except catches `OverflowError` with correct message)
- `checked_add` non-overflow path (try/except with empty message, value extracted correctly)

---

## Design observations

1. **No `sub`, `mul`, `div` yet.** The PR only covers `add` variants. The e2e fixture
   is named `fixed_width_add_apis` which is accurate. The design doc (integer_model.md
   line 174) calls out that "division, floor division, modulo, exponentiation, and shifts
   use the same no-silent-failure rule" — future slices.

2. **OverflowError struct is emitted into every generated binary.** The emitted
   `main.rs` contains a full `OverflowError` struct definition including `Debug`,
   `Clone`, `Display`, and `Error`. This is expected for the current codegen model
   (top-level struct emission). It is not specific to this PR.

3. **The 3-line deletion in `expressions.rs`** removes two stale doc-comment lines
   from `lower_lambda_with_context` and one unrelated blank line. This is cleanup,
   not functional change.

4. **`resolve_alias` usage is correct.** `fixed_ty.resolve_alias()` and
   `arg_ty.resolve_alias()` are compared. Since `Type::FixedInt` is not an alias
   type, `resolve_alias()` returns `self`, making the comparison equivalent to
   `arg_ty == fixed_ty` for the fixed-width case. This is the same pattern used
   throughout `check.rs`.

---

## Required changes

**None.** The implementation is semantically correct, type-safe, panic-free in
user paths, and consistent with the INT-3 design contract. The double `.to_string()`
in `lower_checked_add` is a cosmetic inefficiency but does not affect correctness.
