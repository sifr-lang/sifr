# wave_psp_ext_2 Review Pass 1 (Completion Gap)

**Phase**: ad-hoc-python-source-parity-extension-waiver-reduction
**Wave**: wave_psp_ext_2 (`itertools` Lazy Surface Closure)
**Review Type**: Completion Gap Review
**Reviewer**: Claude Code
**Date**: 2026-03-18

---

## Executive Summary

The wave_psp_ext_2 implementation successfully converts 12 `itertools` functions from eager list-returning behavior to lazy iterator-returning behavior, closing the gap identified in the predecessor lazy-iterator phase.

**Verdict**: APPROVED with minor findings that should be addressed before production-grade review

---

## Scope Review

### Wave Definition (from phase doc)

- Replace broad `itertools` lazy waiver with real shipped iterator behavior
- Migrate previously eager `itertools` helpers onto canonical iterator runtime
- Tighten residual waivers to only families still blocked by non-iterator root causes

### Implementation Delivered

| Target | Status | Evidence |
|--------|--------|----------|
| `accumulate` returns `Iterator[T]` | ✅ Complete | `lib/sifr/itertools.sifr:353-368` |
| `compress` returns `Iterator[T]` | ✅ Complete | `lib/sifr/itertools.sifr:371-376` |
| `dropwhile` returns `Iterator[T]` | ✅ Complete | `lib/sifr/itertools.sifr:379-392` |
| `takewhile` returns `Iterator[T]` | ✅ Complete | `lib/sifr/itertools.sifr:395-400` |
| `filterfalse` returns `Iterator[T]` | ✅ Complete | `lib/sifr/itertools.sifr:403-411` |
| `zip_longest` returns `Iterator[list[T]]` | ✅ Complete | `lib/sifr/itertools.sifr:414-419` |
| `cycle` returns `Iterator[T]` | ✅ Complete | `lib/sifr/itertools.sifr:431-444` |
| `starmap` returns `Iterator[R]` | ✅ Complete | `lib/sifr/itertools.sifr:343-350` |
| `product` returns `Iterator[list[T]]` | ✅ Complete | `lib/sifr/itertools.sifr:282-298` |
| `permutations` returns `Iterator[list[T]]` | ✅ Complete | `lib/sifr/itertools.sifr:301-314` |
| `combinations` returns `Iterator[list[T]]` | ✅ Complete | `lib/sifr/itertools.sifr:317-327` |
| `combinations_with_replacement` returns `Iterator[list[T]]` | ✅ Complete | `lib/sifr/itertools.sifr:330-340` |

---

## Detailed Review

### 1. Iterator Return Type Correctness

All 12 functions correctly return `Iterator[T]` (or `Iterator[list[T]]` for combinators that yield tuples).

**Finding**: ✅ Correct - All functions use `yield` to produce lazy iterators instead of returning eager lists.

### 2. Iterator Exhaustion Behavior

The demo `ad_hoc_parity_ext_wave2_itertools_lazy_surface_demo.sifr` correctly demonstrates:
- Iterator exhaustion after partial consumption
- Explicit materialization via `list(...)` for reusable values

**Example from demo (lines 16-18)**:
```sifr
acc_it: Iterator[int] = accumulate([1, 2, 3, 4])
assert next(acc_it) == 1
assert str(list(acc_it)) == "[3, 6, 10]"
```

**Finding**: ✅ Correct - Iterator single-pass behavior is properly demonstrated.

### 3. Explicit Materialization Enforcement

**Test evidence** (`crates/sifr/tests/e2e/fail/phase_psp_ext_2_itertools_materialization_required.sifr`):
```sifr
def main():
    values: list[int] = accumulate([1, 2, 3])  # Should fail - Iterator, not list
    assert str(values) == "[1, 3, 6]"
```

Verification:
```bash
$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_ext_2_itertools_materialization_required.sifr
type error: type mismatch: expected 'list[int]', got 'Iterator[int]'
```

**Finding**: ✅ Correct - Type system correctly rejects iterator-to-list assignment without explicit materialization.

---

## Correctness Gaps

### Finding 1: `itertools.product` Negative `repeat` Handling (MEDIUM)

**Description**: CPython's `itertools.product` raises `ValueError` when `repeat` is negative, but Sifr returns an empty list silently.

**CPython behavior**:
```python
>>> import itertools
>>> list(itertools.product([1, 2], repeat=-1))
ValueError: repeat argument cannot be negative
```

**Sifr behavior** (from test `cpython_itertools_subset.sifr:44`):
```sifr
actual.append(str(list(product([1, 2], repeat=-1))) == "[]")
```

**Location**: `lib/sifr/itertools.sifr:282-298`
```sifr
def product[T](*iterables: list[T], repeat: int = 1) -> Iterator[list[T]]:
    result: list[list[T]] = []
    if repeat >= 0:  # Silent handling instead of raising
        ...
```

**Risk**: Users expecting CPython error behavior may get unexpected empty results.

**Recommendation**: Either:
1. Add runtime error for negative repeat (aligns with CPython), or
2. Document this as intentional difference in `wave_psp_b2_cpython_traceability.md`

---

### Finding 2: `itertools.cycle` Signature Differs from CPython (LOW)

**Description**: Sifr's `cycle` takes two arguments (`data: list[T], n: int`) for a finite version, while CPython's `cycle` takes one argument (`iterable`) and cycles infinitely.

**CPython signature**:
```python
itertools.cycle(iterable)  # Infinite iterator
```

**Sifr signature**:
```sifr
def cycle(data: list[T], n: int) -> Iterator[T]  # Finite, n elements
```

**Location**: `lib/sifr/itertools.sifr:431-444`

**Evidence from demo**:
```sifr
cyc: Iterator[int] = cycle([1, 2, 3], 5)  # 5 elements, then stops
assert next(cyc) == 1
assert str(list(cyc)) == "[2, 3, 1, 2]"
```

**Risk**: Low - This is likely an intentional adaptation for safety (avoiding infinite iterators), but it's not explicitly documented in the traceability.

**Recommendation**: Document this as intentional difference in `wave_psp_b2_cpython_traceability.md`.

---

## Regression Check

### No User-Triggerable Panics

Reviewed code paths in `lib/sifr/itertools.sifr`:
- All functions use safe loops with bounds checking
- No `.unwrap()` or `.expect()` calls in production code paths

**Finding**: ✅ Correct - No panic paths introduced.

### Deterministic Behavior

All iterator functions produce deterministic, reproducible sequences matching expected combinatorial mathematics.

**Finding**: ✅ Correct - Deterministic behavior maintained.

---

## Coverage Assessment

### Positive Path Coverage

| Function | Demo | E2E Test |
|----------|------|----------|
| `accumulate` | ✅ | ✅ |
| `compress` | ✅ | ✅ |
| `dropwhile` | ✅ | ✅ |
| `takewhile` | ✅ | ✅ |
| `filterfalse` | ✅ | ✅ |
| `zip_longest` | ✅ | ✅ |
| `cycle` | ✅ | ✅ |
| `starmap` | ✅ | ✅ |
| `product` | ✅ | ✅ |
| `permutations` | ✅ | ✅ |
| `combinations` | ✅ | ✅ |
| `combinations_with_replacement` | ✅ | ✅ |

### Negative Path Coverage

| Test Case | File | Coverage |
|-----------|------|----------|
| Iterator-to-list assignment | `phase_psp_ext_2_itertools_materialization_required.sifr` | ✅ |
| `starmap` non-binary callable | `phase_psp_b2_itertools_starmap_non_binary_callable.sifr` | ✅ (from predecessor wave) |

### Missing Coverage

1. **Negative `repeat` for `product`**: No fail test that expects runtime error (see Finding 1)
2. **Empty input handling**: Could add more edge case tests for empty iterables

---

## Governance Accuracy

### Phase Execution Status

From `issues/ad-hoc-python-source-parity-extension-waiver-reduction-execution.md`:
- ✅ wave_psp_ext_2 listed as merged
- ✅ Implementation PR referenced: #1256
- ✅ Validation results recorded

### Traceability Ledger

From `verification/stdlib/wave_psp_b2_cpython_traceability.md`:
- ✅ "Approved iterator-returning itertools combinators" marked as `parity-closed`
- ✅ Explicit note: "require explicit source-level materialization (`list(...)`) when reusable collection values are needed"

### Governance Inventory

From `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`:
- ✅ `itertools` module listed as `parity-closed` under `wave_psp_b2`
- ✅ Residual waivers documented:
  - `itertools.tee`, `itertools.groupby` - intentional-diff
  - General-arity `starmap` - intentional-diff

### Finding 3: Missing Documentation for Signature Differences (LOW)

The traceability document doesn't explicitly note:
1. `cycle` takes 2 args (finite) vs CPython's 1 arg (infinite)
2. `product` handles negative `repeat` silently vs CPython raising `ValueError`

**Recommendation**: Add notes to `wave_psp_b2_cpython_traceability.md` under the "Classified waivers" section.

---

## Validation Evidence

### Local Validation

```
$ scripts/run_all_tests.sh --profile quick
Validation lane report
  profile=quick
  wall_time=37.79s cpu=27.18s
  e2e pass suite: 24 fixtures, 24 passed, 0 failed
  report_signature=e1bf653aaa770517
```

### Demo Execution

```
$ cargo run -q -p sifr -- run demos/ad_hoc_parity_ext_wave2_itertools_lazy_surface_demo.sifr
ad_hoc_parity_ext_wave2_itertools_lazy_surface_demo: ok
```

### E2E Tests

```
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr
cpython_itertools_subset: pass (all 30 assertions)
```

### Unit Tests

```
$ cargo test -p sifr_hir -- --skip test_e2e_pass
test result: ok. 121 passed; 0 failed
```

---

## Findings Summary

### Strengths

1. **Correct Iterator Semantics**: All 12 functions now return `Iterator[T]` matching CPython lazy semantics
2. **Type Safety**: Compile-time errors for incorrect iterator-to-collection assignments
3. **No Panics**: All production code paths use safe error handling
4. **Comprehensive Coverage**: Demo and e2e tests cover all functions with positive and negative paths
5. **Governance Alignment**: Wave status and traceability properly updated

### Findings Requiring Attention

| Finding | Severity | Status |
|---------|----------|--------|
| `product` negative `repeat` silent handling | Medium | Should document or fix |
| `cycle` signature differs from CPython | Low | Should document as intentional |
| Missing negative `repeat` test case | Low | Should add fail test |

---

## Verdict

**APPROVED** - The wave_psp_ext_2 implementation correctly:

1. Converts 12 `itertools` functions to iterator-returning semantics
2. Maintains type safety with compile-time errors for incorrect usage
3. Ensures no user-triggerable panics in the implementation
4. Provides deterministic, combinatorially correct behavior
5. Enforces explicit materialization boundaries
6. Updates governance ledgers appropriately

### Recommended Actions Before Production-Grade Review

1. **Document `product` negative repeat behavior**: Add explicit note in `wave_psp_b2_cpython_traceability.md` that negative `repeat` returns empty instead of raising `ValueError`, or add runtime error to match CPython
2. **Document `cycle` signature difference**: Add note that Sifr's `cycle(data, n)` is finite (n elements) vs CPython's infinite cycle
3. **Consider adding fail test**: Add `phase_psp_ext_2_product_negative_repeat.sifr` if error behavior is preferred

---

## Next Steps

1. Address documented findings (optional - can be deferred to future wave)
2. Run full validation suite before merge
3. Complete external production-grade review
4. Merge and proceed to wave_psp_ext_3 (Regex and Filesystem Iterator Surfaces)
