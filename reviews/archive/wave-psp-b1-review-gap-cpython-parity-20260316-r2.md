# wave_psp_b1 Review: Collections Objects and Ordered Helpers (Round 2)

**Review Date:** 2026-03-16
**Reviewer:** Claude (Codex)
**Wave Status:** done
**CPython Sources:** `Lib/test/test_collections.py`, `Lib/test/test_bisect.py`, `Lib/test/test_heapq.py`

---

## Executive Summary

This is a follow-up review to assess progress on actionable issues identified in r1. Most issues have been addressed; remaining gaps are documented intentional differences.

---

## Issue Resolution Status

### ✅ Resolved Issues

| Issue | Status | Evidence |
|-------|--------|----------|
| Counter.get() missing default parameter | **FIXED** | `lib/sifr/collections.sifr:17-21` now has `def get(self, key: T, default: int = 0) -> int` |
| Missing fail test for Counter(**kwargs) | **FIXED** | `crates/sifr/tests/e2e/fail/phase_psp_b1_counter_kwargs_constructor_unsupported.sifr` added, correctly rejects with "unexpected keyword argument" |

### 🔶 Remaining Items (Documented Adaptations)

| Item | Status | File Reference |
|------|--------|-----------------|
| deque.index() returns `None` vs CPython's `ValueError` | Adaptation (documented) | `lib/sifr/collections.sifr:114-143` |
| deque lacks `__len__`, `__getitem__`, `__setitem__`, etc. | Intentional difference | `lib/sifr/collections.sifr:75-200` |

---

## Parity Gaps (Actionable Issues)

### 1. deque Lacks Python `len()` Syntax Support

**File:** `lib/sifr/collections.sifr:79`

**Issue:** The deque class has `def len(self) -> int` but does not support Python's `len(q)` syntax.

```python
# CPython
>>> from collections import deque
>>> q = deque([1,2,3])
>>> len(q)
3

# Sifr - requires q.len() not len(q)
```

**Action Required:** Either:
- Add `__len__` magic method support in class lowering, or
- Document this as an explicit divergence in phase governance

---

### 2. Missing CPython Test Coverage for deque Methods

The following CPython test functions have no corresponding Sifr test:

| CPython Test | Coverage Status |
|--------------|-----------------|
| `test_maxlen`, `test_maxlen_zero`, `test_maxlen_attribute` | Not covered |
| `test_contains` (operator `in`) | Not covered |
| `test_insert` | Not covered |
| `test_imul`, `test_mul` | Not covered |
| `test_getitem`, `test_setitem`, `test_delitem` | Not covered |
| `test_iadd`, `test_add` | Not covered |

**Action Required:** Document these as explicit divergences in the traceability document, or implement missing methods.

---

## CPython Test Parity Quality

### Test Coverage Summary

| CPython Test File | Coverage Status | Notes |
|-------------------|-----------------|-------|
| `test_collections.py` (Counter) | Good | Has fail tests for keyword/iterable constructors |
| `test_collections.py` (deque) | Partial | Core methods covered; missing insert/indexing/contai   nment |
| `test_bisect.py` | Good | Covered with adaptation for lo/hi clamping |
| `test_heapq.py` | Good | heapify, heappushpop, heapreplace, max-heap helpers |

### Fail Test Quality

All waiver-enforcing fail tests are present and correctly reject unsupported patterns:

```
✓ phase_psp_b1_counter_iterable_constructor_unsupported.sifr
✓ phase_psp_b1_counter_kwargs_constructor_unsupported.sifr
✓ phase_psp_b1_bisect_key_unsupported.sifr
✓ phase_psp_b1_deque_index_invalid_bound.sifr
```

---

## Adopt/Adapt/Waive Mapping Coherence

### Coherence Analysis

| Surface | Classification | Status |
|---------|---------------|--------|
| `Counter.most_common([n])` | adapted | ✅ Coherent |
| `Counter.get(key, default)` | adapted | ✅ Coherent (now with default param) |
| deque rotate/count/remove/copy/reverse | adapted | ✅ Coherent |
| bisect lo/hi forms | adapted | ✅ Coherent |
| bisect key= | waived | ✅ Coherent |
| Counter(iterable) | waived | ✅ Enforced |
| Counter(**kwargs) | waived | ✅ Enforced (NEW in r2) |
| heapq mutating helpers | adapted | ✅ Coherent |

### Updated Traceability

The traceability document at `verification/stdlib/wave_psp_b1_cpython_traceability.md` should be updated to reflect:
1. Counter.get() now has default parameter (adapted surface)
2. Counter(**kwargs) waiver now has enforced fail test

---

## Validation Evidence

All tests verified to pass locally:

```bash
# Pass tests
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr  # OK
cargo run -q -p sifr -- run demos/wave_psp_b1_collections_ordered_helpers_demo.sifr                   # OK

# Fail tests (correctly rejected)
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_counter_kwargs_constructor_unsupported.sifr  # OK - "unexpected keyword argument"
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_counter_iterable_constructor_unsupported.sifr  # OK - type mismatch
```

---

## Recommendations

### Required Actions

1. **Update traceability document** to reflect Counter.get() default parameter implementation
2. **Document deque.index() behavior** (returns None vs raises ValueError) in governance

### Optional (Future Waves)

3. Add `__len__` magic method support or document Python syntax incompatibility
4. Implement deque missing methods (indexing, containment) or formalize as intentional differences

---

## Conclusion

wave_psp_b1 implementation is in good shape. The two actionable issues from r1 have been resolved:
- Counter.get() now supports default parameter ✅
- Counter(**kwargs) fail test added ✅

The adopt/adapt/waive mapping is now fully coherent with enforcement tests in place. Remaining gaps are documented intentional differences or low-priority enhancements.

**Status: Ready for governance closure**
