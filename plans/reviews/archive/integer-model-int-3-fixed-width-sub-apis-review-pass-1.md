# Review: PR #1869 — INT-3 Fixed-Width Subtraction APIs

**Commit**: 69de6e81
**PR**: https://github.com/sifr-lang/sifr/pull/1869
**Branch**: int-3-fixed-width-sub-apis
**Reviewer**: agent
**Date**: 2026-05-08

---

## Summary

The PR extends the explicit representation-preserving API pattern from the already-merged addition APIs (PR #1868) to subtraction. Four new instance methods are added per fixed-width type: `checked_sub`, `wrapping_sub`, `saturating_sub`, and `overflowing_sub`.

---

## Scope Coverage

### HIR typing — PASS

| Method | Return type |
|---|---|
| `checked_sub` | `Result[fixed-width, OverflowError]` |
| `wrapping_sub` | same fixed-width |
| `saturating_sub` | same fixed-width |
| `overflowing_sub` | `tuple[fixed-width, bool]` |

This matches the design in `internal_docs/integer_model.md` and is consistent with the addition APIs from PR #1868. The `resolve_fixed_width_method_type` function in `fixed_width_arithmetic_methods.rs` correctly handles all four new methods in a single match arm with a shared argument-width check at line 32.

### Mixed-width rejection — PASS

Line 32 of `fixed_width_arithmetic_methods.rs` uses `arg_ty.resolve_alias() != fixed_ty.resolve_alias()` to reject mixed-width arguments. The unit test `test_fixed_width_sub_api_rejects_mixed_width_argument` covers `wrapping_sub`. Because all four sub methods share the same match arm and width check, any mixed-width rejection bug would apply equally to all of them. The test is sufficient.

### Codegen — PASS

**`lower_checked_method`** (lines 21–53 of `fixed_width.rs`): produces `(receiver.checked_sub(rhs)).ok_or_else(|| OverflowError { message: "...".to_string() })`. This is the same safe pattern as `checked_add`. No `.unwrap()` or `.expect()` in user-triggerable paths. When Rust's `checked_sub` returns `None`, `ok_or_else` maps it to `Err(...)` without panicking. ✓

**`lower_primitive_method`** (lines 79–85): produces `(receiver).method(rhs)` directly. Wrapping/saturating/overflowing sub lower to Rust primitives without any fallible unwrap. ✓

### No-panic guarantee — PASS

The codegen for all four methods contains no `.unwrap()`, `.expect()`, or other data-dependent panics in user-triggerable paths. `checked_sub` uses `ok_or_else` which translates Rust `None` to a typed `Err` rather than panicking. The other three are direct primitive calls. ✓

### E2E fixture — PASS

`crates/sifr/tests/e2e/pass/fixed_width_sub_apis.sifr` exercises:
- `wrapping_sub(1)` on `uint8` from `0` → 255 (underflow, wraps) ✓
- `saturating_sub(1)` on `uint8` from `0` → 0 (underflow, saturates at min) ✓
- `overflowing_sub(1)` on `uint8` from `0` → `(255, True)` ✓
- `checked_sub(1)` on `uint8` from `0` → catches `OverflowError` with correct message ✓
- `checked_sub(1)` on `uint8` from `2` → succeeds without exception ✓

The fixture tests the full contract including the non-overflow happy path.

---

## Observations

1. **No codegen-level unit test for rendering**: The `mod.rs` test suite covers string, list, dict, set, decimal, and bytes method lowering, but has no snapshot test for fixed-width method lowering rendering. This is pre-existing and not a regression from this PR — the addition APIs from PR #1868 have the same gap. Not a blocker.

2. **Shared `lower_checked_method` helper**: Both `checked_add` and `checked_sub` use the same `lower_checked_method` function with a `method` parameter and a `message` parameter. This is the correct abstraction — DRY and the message is specificity-preserving. ✓

3. **Match arm grouping is clean**: `checked_add | checked_sub` and `overflowing_add | overflowing_sub` share arms while `_ => Some(fixed_ty)` catches wrapping and saturating variants. This mirrors the add APIs exactly and is easy to verify against the design. ✓

4. **`overflow_error_type` helper is shared**: Defined once at module level and used by both add and sub. Correct. ✓

---

## Required Changes

**None.** The PR is correct, complete, and safe to merge as the fixed-width subtraction API slice of INT-3.
