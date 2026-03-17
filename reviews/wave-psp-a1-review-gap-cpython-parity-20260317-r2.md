# wave_psp_a1 Review: CPython Parity Gap Analysis (Revision 2)

**Reviewer:** Claude Code
**Date:** 2026-03-17
**Wave:** `wave_psp_a1` (milestone_psp_1 / builtin callable surface)
**Status:** Incomplete - one bug persists, one previously-reported bug is now fixed

---

## Executive Summary

wave_psp_a1 implements builtin constructors (`list()`, `tuple()`, `dict()`) and callable surface (`sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `ord`, `chr`).

**Key Findings:**
1. **PROGRESS**: `zip()` with single iterable bug (from r1) is now FIXED ✅
2. **REGRESSION PERSISTS**: `range()` keyword arguments still incorrectly accepted - partial fix applied but incomplete

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
| `zip([1, 2, 3])` (single iterable) | Works | ✅ Works | Now fixed! |
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

>>> list(range(stop=10))
TypeError: range() takes no keyword arguments

# Sifr (BUG - still present):
>>> list(range(start=1, stop=10, step=2))
[1, 3, 5, 7, 9]  # Incorrectly works!

>>> list(range(stop=10))
[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]  # Incorrectly works!
```

**Partial Progress**: The case `range(10, stop=20)` (positional + keyword for same param) is now correctly rejected:
```
type error: range(): 'stop' was provided both positionally and as a keyword
```

But the all-keyword form `range(start=1, stop=10, step=2)` is still incorrectly accepted.

**Location**: `crates/sifr_hir/src/lower/builtin_calls.rs:581-702`

**Root Cause**: The implementation allows keyword arguments and maps them to positional parameters, but CPython rejects ALL keyword arguments to `range()`.

**Recommended Fix**: Reject all keyword arguments to `range()` to match CPython:
```rust
if !call.arguments.keywords.is_empty() {
    ctx.error("range() does not accept keyword arguments".to_string());
    return None;
}
```

---

### Good News: Bug 2 from r1 is FIXED

**Issue**: `zip()` with single iterable had codegen bug (reported in r1)

**Verification**:
```python
# Sifr now works correctly:
>>> list(zip([1, 2, 3]))
[(1,), (2,), (3,)]

# Matches CPython exactly:
>>> list(zip([1, 2, 3]))
[(1,), (2,), (3,)]
```

**Status**: ✅ FIXED

---

## Pre-existing Issues (Not Introduced by This Wave)

These are codegen limitations that existed before wave_psp_a1:

| Issue | Example | CPython | Sifr |
|-------|---------|---------|------|
| Empty list literal in zip | `zip([], [1,2])` | Works | Codegen error |
| Empty zip() | `zip()` | `[]` | Codegen error |
| Empty enumerate | `enumerate([])` | `[]` | Codegen error |
| Empty reversed | `reversed([])` | `[]` | Codegen error |
| Empty sorted | `sorted([])` | `[]` | Codegen error |

**Workaround**: Use explicitly typed empty lists:
```python
empty: list[int] = []
result = zip(empty, [1, 2])  # Works
```

**Note**: These are documented in pass2 review as pre-existing issues and should remain out of scope for this wave.

---

## Test Coverage Gaps

| Gap | CPython Reference | Status |
|-----|-------------------|--------|
| `range(start=1, stop=10)` | test_range.py:46-52 | Not tested - should error but doesn't |
| `range(stop=10)` | test_range.py:46-52 | Not tested - should error but doesn't |
| `zip([1, 2, 3])` | test_builtin.py:2125-2140 | ✅ Now works (fixed) |
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
   - Update test: `range(start=1, stop=7, step=2)` in demo/tests should use positional args

### Documentation Updates

1. Update traceability: remove claim that range keyword normalization is "intentional adaptation" - it's a bug
2. The partial fix (`range(10, stop=20)` now errors) is good but incomplete

---

## Conclusion

**Status**: One bug blocks closing this wave:

1. 🔴 **range() keyword bug** - Partial fix applied (positional+keyword case), but all-keyword form still broken

All other features work correctly, including the previously-reported `zip()` single iterable bug which is now fixed.

---

## Summary Table

| Bug | Status | Notes |
|-----|--------|-------|
| range() keyword args | 🔴 Incomplete | Partial fix applied, all-keyword form still broken |
| zip() single iterable | ✅ Fixed | Was broken in r1, now works |
| Empty list literal codegen | ⚠️ Pre-existing | Not in scope for this wave |
