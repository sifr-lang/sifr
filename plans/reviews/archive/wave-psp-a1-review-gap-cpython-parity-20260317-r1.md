# wave_psp_a1 Review: CPython Parity Gap Analysis

**Reviewer:** agent
**Date:** 2026-03-17
**Wave:** `wave_psp_a1` (milestone_psp_1 / builtin callable surface)
**Status:** Incomplete - two bugs identified

---

## Executive Summary

wave_psp_a1 implements builtin constructors (`list()`, `tuple()`, `dict()`) and callable surface (`sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `ord`, `chr`). Two bugs were identified:

1. **REGRESSION**: `range()` keyword arguments still incorrectly accepted
2. **NEW FINDING**: `zip()` with single iterable has codegen bug

---

## Verified Working Features

| Surface | CPython Behavior | Sifr Status | Evidence |
|---------|------------------|-------------|----------|
| `list()` / `list(iterable)` | Works | ✅ Works | Demo passes |
| `tuple()` / `tuple(list literal)` / `tuple(str literal)` | Works | ✅ Works | Demo passes |
| `dict()` / `dict(iterable)` / `dict(**keywords)` | Works | ✅ Works | Demo passes |
| `sorted(iterable, key, reverse)` | Works | ✅ Works | Demo passes |
| `reversed(sequence)` | Works | ✅ Works | Demo passes |
| `enumerate(iterable, start)` | Works | ✅ Works | Demo passes |
| `zip(*iterables)` 2+ args | Works | ✅ Works | Demo passes |
| `map(func, *iterables)` | Works | ✅ Works | Demo passes |
| `ord()` | Works | ✅ Works | Demo passes |
| `chr()` | Works | ✅ Works | Demo passes |
| `range(stop)` | Works | ✅ Works | Demo passes |
| `range(start, stop)` | Works | ✅ Works | Demo passes |
| `range(start, stop, step)` | Works | ✅ Works | Demo passes |
| `range(10, stop=20)` | TypeError | ✅ Correctly rejected | Fail test passes |

---

## Identified Issues

### Bug 1: range() Keyword Arguments Still Incorrectly Accepted (REGRESSION)

**Issue**: `range()` incorrectly accepts keyword arguments when CPython rejects them entirely.

**Evidence**:
```python
# CPython:
>>> list(range(start=1, stop=10, step=2))
TypeError: range() takes no keyword arguments

# Sifr (BUG - still present):
>>> list(range(start=1, stop=10, step=2))
[1, 3, 5, 7, 9]  # Incorrectly works!
```

**Partial Progress**: The case `range(10, stop=20)` (positional + keyword for same param) is now correctly rejected:
```
type error: range(): 'stop' was provided both positionally and as a keyword
```

But the all-keyword form `range(start=1, stop=10, step=2)` is still incorrectly accepted.

**Location**: `crates/sifr_hir/src/lower/builtin_calls.rs:581-702`

**Root Cause**: The implementation allows keyword arguments and maps them to positional parameters, but CPython rejects ALL keyword arguments to `range()`.

**Recommended Fix**: Reject all keyword arguments to `range()` to match CPython behavior:
```rust
if !call.arguments.keywords.is_empty() {
    ctx.error("range() does not accept keyword arguments".to_string());
    return None;
}
```

---

### Bug 2: zip() with Single Iterable Codegen Bug (NEW FINDING)

**Issue**: `zip()` with a single iterable fails at code generation time.

**Evidence**:
```python
# Sifr - FAILS at codegen:
def main():
    result = list(zip([1, 2, 3]))
    print(result)

# Error:
error[E0308]: mismatched types
expected `Vec<i64>`, found `Vec<(i64,)>`
```

**Expected (CPython)**:
```python
>>> list(zip([1, 2, 3]))
[(1,), (2,), (3,)]
```

**Root Cause**: The HIR type is correctly inferred as `list[tuple[int]]`, but the codegen incorrectly collects into `Vec<int>` instead of `Vec<(int,)>` for single-element tuples.

**Test Gap**: No test coverage for single-argument `zip()`.

**Location**: Likely in `crates/sifr_codegen/src/lower_expr.rs` - needs investigation

**Recommended Fix**: Add test case first, then fix codegen to handle single-element tuple types correctly.

---

## Test Coverage Gaps

| Gap | CPython Reference | Status |
|-----|-------------------|--------|
| `range(start=1, stop=10)` | test_range.py:46-52 | Not tested - should error |
| `zip([1, 2, 3])` | test_builtin.py:2125-2140 | Not tested - codegen bug |
| `set()` constructor | test_builtin.py:2557+ | Not in scope (not documented) |

---

## Fail Test Verification

| Test | Expected Error | Status |
|------|----------------|--------|
| `phase_psp_a1_range_duplicate_stop_keyword.sifr` | `range(): 'stop' was provided both positionally and as a keyword` | ✅ Passes |
| `phase_psp_a1_sorted_unexpected_keyword.sifr` | `sorted() got an unexpected keyword argument 'bogus'` | ✅ Passes |
| `phase_psp_a1_map_callable_arity_mismatch.sifr` | `map() callable expects 1 argument(s), got 2 iterable(s)` | ✅ Passes |
| `phase_psp_a1_tuple_dynamic_list_shape.sifr` | `tuple() currently requires a tuple, list literal...` | ✅ Passes |

---

## Traceability Matrix Discrepancy

The traceability document claims:
> "keyword `range(start=..., stop=..., step=...)` ... Sifr intentionally normalizes keyword forms for `range(...)` as a typed ergonomics adaptation"

**Reality**: This is NOT an intentional adaptation - it's a bug. CPython rejects all keyword arguments to `range()`. The documentation should be updated to reflect this is a bug, not an adaptation.

---

## Recommendations

### Must Fix Before Closing

1. **Fix range keyword arguments**
   - Reject all keyword arguments to `range()` to match CPython
   - Add fail test: `range(start=1, stop=10)` should error

2. **Fix zip() single iterable codegen**
   - Add test case for `zip([1, 2, 3])`
   - Fix codegen to produce correct tuple types

### Documentation Updates

1. Update traceability: remove claim that range keyword normalization is "intentional adaptation" - it's a bug
2. Add note about `set()` constructor not being in scope

---

## Conclusion

**Status**: Two bugs block closing this wave:

1. 🔴 **range() keyword bug** - Partial fix applied, but all-keyword form still broken
2. 🔴 **zip() single-arg codegen bug** - New finding, blocks single-argument zip()

All other features work correctly. Fix these two issues before marking wave complete.
