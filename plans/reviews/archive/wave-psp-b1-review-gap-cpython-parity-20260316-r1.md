# wave_psp_b1 Review: Collections Objects and Ordered Helpers

**Review Date:** 2026-03-16
**Reviewer:** agent (agent)
**Wave Status:** done
**CPython Sources:** `Lib/test/test_collections.py`, `Lib/test/test_bisect.py`, `Lib/test/test_heapq.py`

---

## Executive Summary

wave_psp_b1 covers `collections`, `bisect`, and `heapq` modules. The implementation achieves good parity for the closed surface but has several gaps in CPython test coverage and a few missing methods that should be documented as intentional divergences.

---

## Parity Gaps (Actionable Issues)

### 1. deque Missing `__len__` Python Syntax Support

**File:** `lib/sifr/collections.sifr:225`

**Issue:** The Sifr deque class implements `def len(self) -> int` but does not support Python's `len(q)` syntax. CPython deque objects are callable with `len()`.

**Evidence:**
```python
# CPython
>>> from collections import deque
>>> q = deque([1,2,3])
>>> len(q)
3

# Sifr - requires q.len() not len(q)
```

**Action Required:** Either:
- Add `__len__` method support in the class lowering, or
- Document this as an explicit divergence in the phase documentation

---

### 2. deque Missing Multiple CPython Methods

**File:** `lib/sifr/collections.sifr:178-324`

**Issue:** The following CPython deque methods are not implemented:

| CPython Method | Status | Priority |
|----------------|--------|----------|
| `insert(i, x)` | Missing | Medium |
| `__getitem__(i)` | Missing | High |
| `__setitem__(i, x)` | Missing | High |
| `__delitem__(i)` | Missing | High |
| `__mul__(n)` / `__imul__(n)` | Missing | Medium |
| `__add__(other)` / `__iadd__(other)` | Missing | Medium |
| `__contains__(x)` (`in` operator) | Missing | Medium |
| `__repr__()` | Missing | Low |
| `__hash__()` | Missing | N/A (intentionally - deques are mutable) |

**Action Required:** Either implement missing methods or classify as explicit divergences in the phase documentation.

---

### 3. Counter.get() Missing Default Parameter

**File:** `lib/sifr/collections.sifr:17-21`

**Issue:** Sifr's `Counter.get(key)` does not support the optional default parameter that CPython supports:

```python
# CPython
>>> from collections import Counter
>>> c = Counter('abc')
>>> c.get('x', -1)
-1

# Sifr - default parameter not supported
>>> c = Counter({'a': 1})
>>> c.get('x', -1)  # Would be a type error
```

**Action Required:** Add optional `default` parameter to Counter.get method:
```sifr
def get(self, key: T, default: int = 0) -> int:
```

---

### 4. Missing Explicit Fail Test for Counter(**kwargs)

**File:** `crates/sifr/tests/e2e/fail/phase_psp_b1_counter_iterable_constructor_unsupported.sifr`

**Issue:** The traceability document lists `Counter(**kwargs)` as waived, but there is no corresponding fail test to enforce this waiver.

**Current tests:**
- `phase_psp_b1_counter_iterable_constructor_unsupported.sifr` - tests `Counter(list)` (waived)
- Missing: test for `Counter(a=1, b=2)` (also waived)

**Action Required:** Add fail test:
```sifr
# Should fail - Counter(**kwargs) not supported
from sifr.collections import Counter

def main():
    counts: Counter[str] = Counter(a=1, b=2)
```

---

### 5. deque.index() Return Type Inconsistency

**File:** `lib/sifr/collections.sifr:286-309`

**Issue:** The `index()` method returns `int | None`, but CPython raises `ValueError` when the element is not found (not returns `None`).

```python
# CPython
>>> from collections import deque
>>> d = deque([1,2,3])
>>> d.index(5)
ValueError: 5 not in deque

# Sifr - returns None instead
```

**Action Required:** Either:
- Change to raise `ValueError` to match CPython, or
- Document as explicit adaptation (safe/panic-free behavior)

---

## CPython Test Parity Quality Assessment

### Test Coverage Summary

| CPython Test File | Coverage Status | Notes |
|-------------------|-----------------|-------|
| `test_collections.py` (Counter) | Partial | Missing: `fromkeys`, keyword constructor, dict subclass behavior |
| `test_collections.py` (deque) | Partial | Missing: insert, indexing, multiplication, containment |
| `test_bisect.py` | Partial | Missing: `key=` function (correctly waived) |
| `test_heapq.py` | Good | Covered: heapify, heappushpop, heapreplace, max-heap helpers |

### Missing CPython Test Porting

The following CPython test functions have no corresponding Sifr test:

**From test_deque.py:**
- `test_maxlen`, `test_maxlen_zero`, `test_maxlen_attribute` - maxlen edge cases
- `test_contains` - `in` operator
- `test_insert` - insert method
- `test_imul`, `test_mul` - multiplication
- `test_getitem`, `test_setitem`, `test_delitem` - indexing operations
- `test_iadd`, `test_add` - concatenation

**From test_collections.py (Counter):**
- `Counter.fromkeys()` class method
- Dict subclass behavior tests

**Action Required:** For gaps that cannot be addressed in the current wave, document as explicit divergences.

---

## Adopt/Adapt/Waive Mapping Coherence

### Analysis

The traceability document at `verification/stdlib/wave_psp_b1_cpython_traceability.md` provides a mapping. Reviewing coherence:

| Surface | Classification | Coherence | Notes |
|---------|---------------|------------|-------|
| `Counter.most_common([n])` | adapted | OK | Works as documented |
| deque rotate/count/remove/copy/reverse | adapted | OK | Works as documented |
| bisect lo/hi forms | adapted | OK | Clamping behavior documented |
| bisect key= | waived | OK | Waiver rationale is sound |
| Counter(iterable) | waived | ISSUE | Missing fail test for `Counter(**kwargs)` |
| heapq mutating helpers | adapted | OK | None-returning for empty documented |

### Coherence Issues

1. **Waiver enforcement incomplete:** `Counter(**kwargs)` is listed as waived but lacks a fail test.

2. **Adapted surfaces not fully documented:** The `deque.index()` returning `None` vs CPython's `ValueError` is an adaptation that should be explicitly documented.

---

## Recommendations

### High Priority

1. **Add Counter.get default parameter** (`lib/sifr/collections.sifr:17`)
2. **Add fail test for Counter(**kwargs)** to enforce waiver
3. **Document deque.index() behavior difference** as adaptation

### Medium Priority

4. **Implement deque missing methods** (indexing, insert, containment) or document as explicit divergences
5. **Add deque `__len__` method** or document Python syntax incompatibility

### Low Priority

6. **Port additional CPython tests** for deque edge cases (maxlen, etc.)

---

## Validation Evidence

All tests verified to pass locally:

```bash
# Pass tests
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr  # OK
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_collections.sifr                      # OK
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_bisect.sifr                             # OK
cargo run -q -p sifr -- run demos/wave_psp_b1_collections_ordered_helpers_demo.sifr                   # OK

# Fail tests (correctly rejected)
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_bisect_key_unsupported.sifr   # OK - "unexpected keyword argument 'key'"
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_counter_iterable_constructor_unsupported.sifr  # OK - type mismatch
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_deque_index_invalid_bound.sifr  # OK - type error
```

---

## Files Modified in This Wave

- `lib/sifr/collections.sifr`
- `lib/sifr/bisect.sifr`
- `lib/sifr/heapq.sifr`
- `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_b1_bisect_key_unsupported.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_b1_counter_iterable_constructor_unsupported.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_b1_deque_index_invalid_bound.sifr`
- `verification/stdlib/wave_psp_b1_cpython_traceability.md`

---

## Conclusion

wave_psp_b1 achieves good functional parity for the closed surface but has several gaps in method completeness (especially deque) and CPython test coverage. The most actionable issues are:

1. Counter.get missing default parameter
2. Missing fail test for Counter(**kwargs)
3. deque.index() behavior difference needs documentation

The adopt/adapt/waive mapping is mostly coherent but needs the missing fail test to fully enforce the waiver classification.
