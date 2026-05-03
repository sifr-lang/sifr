# Review: Diag-11 Tuple Unpack Raw HIR Diagnostic Migration

**Reviewer**: Code review
**Branch**: `codex/diag-11-raw-hir-tuple-unpack`
**Files**: `tuple_unpack.rs`, `expressions_tests.rs`, `check_diagnostic_transport_cleanup.py`

---

## Summary

Migration is **SATISFACTORY**. All project requirements are met. No raw `ctx.error(...)` remains, the dummy `unwrap_or_else` continuation is gone, primary ranges are correct, tests are comprehensive, and the guardrail is updated.

---

## Requirement Verification

### 1. No raw `ctx.error(...)` in migrated file

**Status**: PASS

All 10 error call sites in `tuple_unpack.rs` use `ctx.error_with_code_at(...)`. Confirmed via grep — zero raw `ctx.error` occurrences.

### 2. Dummy `unwrap_or_else` continuation removed

**Status**: PASS

The removed code was:

```rust
let star = star.unwrap_or_else(|| {
    ctx.error("star unpacking requires a starred expression".to_string());
    (
        "_".to_string(),
        sifr_type_system::Type::List(Box::new(elem_ty.clone())),
    )
});
```

Replaced with:

```rust
let Some(star) = star else {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
        "star unpacking requires a starred expression".to_string(),
        tuple.range(),
    );
    return None;
};
```

The fallback `("_", List(...))` dummy target that silently continued compilation with a synthetic binding is gone. Error now propagates via `return None`.

### 3. Diagnostic code taxonomy

**Status**: PASS

All 10 error sites in `tuple_unpack.rs` use `DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH`, consistent with the existing for-loop tuple target diagnostics in `statements.rs` (lines 1987, 2004).

The one case where `TYPE_MISMATCH` appears in the test file (`test_tuple_unpack_reassignment_type_mismatch_has_primary_range`) is correct — that error is about a type mismatch on rebinding an existing variable, not about unpack shape.

### 4. Primary range correctness

**Status**: PASS

| Error case | Primary range | Correct? |
|---|---|---|
| Attribute target not simple name | `value.range()` | points at the sub-expression that isn't a Name |
| Target not simple name or attribute | `elt.range()` | points at the invalid expression |
| Shape mismatch (wrong element count) | `tuple.range()` | points at the whole tuple target |
| Non-tuple being unpacked | `value.range()` | points at the non-tuple expression |
| Star unpacking non-list | `value.range()` | points at the non-list expression |
| Multiple starred | `starred.range()` | points at the duplicate `*tail` |
| Starred target not simple name | `starred.value.range()` | points at `values[0]` inside `*values[0]` |
| Non-name in trailing slot | `elt.range()` | points at `values[0]` after `*rest,` |
| Missing starred expression | `tuple.range()` | points at the whole tuple target |

### 5. Test coverage

**Status**: PASS — 4 new tests added

| Test | Source | Primary range target |
|---|---|---|
| `test_tuple_unpack_invalid_target_has_unpack_code` | `values[0], y = (1, 2)` | `values[0]` |
| `test_star_unpack_multiple_starred_targets_have_unpack_code` | `first, *rest, *tail = [1, 2, 3]` | `*tail` (whole starred expr) |
| `test_star_unpack_invalid_starred_target_has_unpack_code` | `first, *values[0] = [1, 2]` | `values[0]` (starred.value) |
| `test_star_unpack_invalid_trailing_target_has_unpack_code` | `first, *rest, values[0] = [1, 2]` | `values[0]` (trailing slot) |

All use `range_for` or `range_for_after_anchor` correctly. The `range_for_after_anchor` helper is used appropriately for cases where the needle appears after an anchor string that disambiguates it from other occurrences.

### 6. Guardrail updated

**Status**: PASS

`tuple_unpack.rs` added to `RAW_HIR_ERROR_FREE_FILES` in `check_diagnostic_transport_cleanup.py`.

---

## Phase Policy

**Status**: COMPLIANT

All diagnostics migrated are HIR-lowering phase errors (target-shape, star constraints, missing starred expression). No diagnostics belong to a later phase.

---

## Minor Observations (non-blocking)

1. **Message specificity**: The message "tuple unpacking attribute target must be rooted at a simple name" is new — no existing test covers it. This is acceptable as it was previously a raw error with no test, and the condition is an internal sanity check (the `Attribute` case already checks that `value` is a `Name`).

2. **`value.range()` vs `elt.range()` for attribute case**: At line 31, `value.range()` is used rather than `elt.range()` — this is correct because `value` is the receiver of the attribute (e.g., `foo.bar` → `foo` is the value), and the error message "tuple unpacking attribute target must be rooted at a simple name" describes what `value` should be.

---

## Conclusion

**Satisfied.** Ready for merge. No second pass required.
