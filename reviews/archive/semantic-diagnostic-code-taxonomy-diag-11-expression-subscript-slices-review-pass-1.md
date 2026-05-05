# Review: diag-11 raw HIR expression subscript/slices

## Verdict: APPROVED with one required fix

The migration is structurally sound and all local validations pass. However, one
primary range choice is incorrect and must be corrected before merge.

---

## Required fixes

### 1. Unsupported slice receiver: primary range points to the wrong AST node

**File:** `crates/sifr_hir/src/lower/expressions.rs:1931`

**Current code:**
```rust
expression_diagnostics::type_mismatch(
    ctx,
    format!("cannot slice type '{}'", object_ty.display_name()),
    sub.value.range(),   // <-- points to the base expression (e.g. `value`)
);
```

**Problem:** For `value[0:1]` where `value: int`, `sub.value` is the base expression `value`.
The user writes `value[0:1]` — the `cannot slice type 'int'` diagnostic should highlight the
 offending expression that cannot be subscripted, which is `value[0:1]` as a whole (the
 subscript operation), not just the base `value`. Compare with the dict unpacking case:
`{**other}` uses `item.value.range()` correctly because `other` is the offending sub-expression.
But for a slice like `value[0:1]`, the slice syntax itself is the construct that is invalid on
 this type.

**Correct range:** `sub.slice.range()` — the `[0:1]` slice syntax is what is unsupported on `int`.
This aligns with the tuple slice errors which all use `sub.slice.range()`.

**Rationale:** The message says "cannot slice type 'int'" — the slice notation `[0:1]` on an `int`
is the offending construct. Pointing at just `value` misses the `[]` portion entirely and is
inconsistent with every other slice diagnostic in this same file which uses `sub.slice.range()`.

---

## What was verified

### Dict unpacking (**) — line 1764
- `expression_diagnostics::type_mismatch(ctx, msg, item.value.range())`
- `item.value` is the expression after `**` (e.g. `other` in `{**other}`) — correct.
- Primary range is appropriate.
- Test `test_dict_unpacking_has_type_code` covers this path.

### Tuple slicing — lines 1853–1903
All four cases use `sub.slice.range()`:
1. "tuple too large for slicing index computation" — line 1853
2. "tuple slice indices out of range" — lines 1876, 1884, 1892
3. "tuple slicing requires compile-time constant indices" — line 1903

Recovery behavior is preserved: `HirExpr::Slice { ... }` is still returned and type is `Type::Any`
so compilation continues cleanly. No fallback `.error()` calls remain.
- Test `test_tuple_slice_errors_have_type_codes` covers out-of-range and dynamic index cases.

### Unsupported slice receiver (fallback arm) — line 1931
- Currently uses `sub.value.range()` — **incorrect** (see fix above).
- All other slice paths use `sub.slice.range()`.
- Test `test_unsupported_slice_receiver_has_type_code` covers this path.

### Structured transport
- All four diagnostic sites use `expression_diagnostics::type_mismatch()` which delegates to
  `ctx.error_with_code_at(DiagnosticCode::TYPE_MISMATCH, ...)`.
- No raw `ctx.error(...)` fallback calls remain in any of the migrated paths.
- Primary ranges are recorded via the structured path.

### Test coverage
- `test_dict_unpacking_has_type_code` — verifies TYPE_MISMATCH code + correct primary range for `**other`.
- `test_tuple_slice_errors_have_type_codes` — verifies both out-of-range and dynamic-index cases.
- `test_unsupported_slice_receiver_has_type_code` — verifies `int[0:1]` error.

---

## Summary

| Diagnostic | Primary range used | Correct? |
|---|---|---|
| Dict unpacking `**` | `item.value.range()` | Yes |
| Tuple too large | `sub.slice.range()` | Yes |
| Tuple indices OOR | `sub.slice.range()` | Yes |
| Tuple dynamic indices | `sub.slice.range()` | Yes |
| Unsupported slice receiver | `sub.value.range()` | **No — must be `sub.slice.range()`** |

One line change required: line 1934 of `expressions.rs` — replace `sub.value.range()` with
`sub.slice.range()`.
