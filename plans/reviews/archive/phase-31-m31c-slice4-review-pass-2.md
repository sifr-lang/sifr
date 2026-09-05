# Phase 31 m31c Slice 4 Production-Readiness Review (Pass 2)

**Reviewer:** agent
**Date:** 2026-03-12
**Slice:** `m31_c_slice_4_private_heapq_max_compat`
**PR:** #1112

---

## Executive Summary

The private heapq max-heap compatibility slice is **PRODUCTION-READY** with minor documentation improvements recommended. The implementation is functionally correct, handles edge cases properly, and the export policy is correctly wired. The core algorithm is sound, with one minor documentation gap (missing docstrings on max-heap functions).

---

## 1. Edge Case Analysis

### Tested Edge Cases

| Edge Case | Input | Expected Output | Status |
|-----------|-------|-----------------|--------|
| Single element | `[42]` | `[42]` | ✅ PASS |
| All same elements | `[5, 5, 5, 5, 5]` | `[5, 5, 5, 5, 5]` | ✅ PASS |
| Negative numbers | `[-1, -5, -3, -10, -2]` | `[-1, -2, -3, -5, -10]` | ✅ PASS |
| Already sorted ascending | `[1, 2, 3, 4, 5]` | `[5, 4, 3, 2, 1]` | ✅ PASS |
| Reverse sorted | `[5, 4, 3, 2, 1]` | `[5, 4, 3, 2, 1]` | ✅ PASS |
| Replace on single element | `[10]` replace with `5` | returns `10`, heap=`[5]` | ✅ PASS |
| Replace with larger | `[3, 1]` replace with `100` | returns `3`, heap=`[100, 1]` | ✅ PASS |
| Float type | `[3.14, 2.71, 1.41, 1.73, 4.67]` | `[4.67, 3.14, 2.71, 1.73, 1.41]` | ✅ PASS |
| Float replace | `[3.5, 1.2]` replace with `5.5` | returns `3.5`, heap=`[5.5, 1.2]` | ✅ PASS |
| Empty heap pop | `[]` | `None` | ✅ PASS (from existing test) |
| Empty heap replace | `[]` replace with `8` | `None` | ✅ PASS (from existing test) |

### Edge Case Findings

**No issues found.** The algorithm correctly handles all tested edge cases including:
- Boundary conditions (empty, single element)
- Duplicate values
- Negative numbers
- Already-ordered inputs
- Type variations (int, float)

---

## 2. API Contract Verification

### Comparison with CPython heapq

| CPython Function | Sifr Implementation | Signature Match | Notes |
|------------------|-------------------|-----------------|-------|
| `_heapify_max(heap)` | `_heapify_max(mut data: list[T])` | ✅ | In-place, mut parameter |
| `_heappop_max(heap)` | `_heappop_max(mut heap: list[T])` | ✅ | Returns `T \| None` |
| `_heapreplace_max(heap, item)` | `_heapreplace_max(mut heap: list[T], own item: T)` | ✅ | Returns `T \| None`, uses `own` for ownership |

### Ownership Correctness

The `_heapreplace_max` function correctly uses `own item: T`:
- When the replacement item is stored into the heap (a mutable borrow), it takes ownership
- This is essential for Sifr's ownership semantics to work correctly with the codegen
- Matches the pattern used in other stdlib functions that store borrowed data

**Status:** ✅ CORRECT

---

## 3. Export Policy Assessment

### Policy Implementation

```rust
pub(crate) fn should_export_callable(module_name: &str, callable_name: &str) -> bool {
    !callable_name.starts_with('_')
        || matches!(
            (module_name, callable_name),
            (
                "sifr.heapq",
                "_heapify_max" | "_heappop_max" | "_heapreplace_max"
            )
        )
}
```

### Wiring Verification

The export policy is correctly wired through:
1. ✅ `bootstrap.rs` - stdlib function exports (line 78)
2. ✅ `bootstrap.rs` - function defaults exports (line 95)
3. ✅ `bootstrap.rs` - signature exports for codegen (line 230)
4. ✅ `bootstrap.rs` - generator function exports (line 290)
5. ✅ `project/exports.rs` - project export collection (line 19)
6. ✅ `project/exports.rs` - function defaults (line 36)

### Narrow Allowlist Assessment

**Status:** ✅ SECURE

The policy explicitly match-lists only the three intentional private helpers. This prevents unintended exposure of other underscore-prefixed stdlib functions while allowing the specific CPython-compatible max-heap surface.

---

## 4. Regression Analysis

### Local Validation Results

| Validation | Status |
|------------|--------|
| `scripts/run_all_tests.sh --profile quick` | ✅ PASS |
| `scripts/run_all_tests.sh` | ✅ PASS |
| Unit test `stdlib_heapq_exports_allowlisted_private_max_heap_helpers` | ✅ PASS |
| E2E test `phase31_heapq_max_private_compat.sifr` | ✅ PASS |
| Demo `phase31_heapq_max_compat_demo.sifr` | ✅ PASS |

### Regression Findings

**No regressions detected.**

- No compiler regressions introduced
- No stdlib regressions introduced
- Export policy change is narrowly scoped to heapq max-heap functions only

---

## 5. Documentation Gap Analysis

### Missing Documentation

**Issue Found:** Max-heap helper functions lack docstrings

| Function | Has Docstring | Notes |
|----------|---------------|-------|
| `_sift_down` | ❌ None | Internal helper |
| `_sift_up` | ❌ None | Internal helper |
| `_sift_down_max` | ❌ None | Internal helper |
| `_sift_up_max` | ❌ None | Internal helper |
| `heapify` | ✅ Yes | "Convert list to a min-heap in-place. O(n) time." |
| `heappush` | ✅ Yes | "Push item onto the heap in-place. O(log n) time." |
| `heappop` | ✅ Yes | "Pop and return the smallest item. Heap is modified in-place. O(log n) time. Returns None if the heap is empty." |
| `_heapify_max` | ❌ None | Public compat function - needs docstring |
| `_heappop_max` | ❌ None | Public compat function - needs docstring |
| `_heapreplace_max` | ❌ None | Public compat function - needs docstring |

### Recommendation

Add docstrings to the three public max-heap functions for consistency with min-heap functions:

```sifr
def _heapify_max[T: Comparable](mut data: list[T]) -> None:
    """Convert list to a max-heap in-place. O(n) time."""

def _heappop_max[T: Comparable](mut heap: list[T]) -> T | None:
    """Pop and return the largest item. Heap is modified in-place. O(log n) time.
    Returns None if the heap is empty."""

def _heapreplace_max[T: Comparable](mut heap: list[T], own item: T) -> T | None:
    """Pop and return the largest item, then push item onto heap.
    Returns None if the heap is empty. O(log n) time."""
```

---

## 6. Test Coverage Assessment

### Existing Coverage

| Test File | Type | Coverage |
|-----------|------|----------|
| `phase31_heapq_max_private_compat.sifr` | E2E | `_heapify_max`, `_heappop_max`, `_heapreplace_max`, empty heap |
| `phase31_heapq_max_compat_demo.sifr` | Demo | Integration test with assertions |
| `stdlib_exports.rs` | Unit | Export policy verification |

### Coverage Gaps Identified

**Minor gaps (non-blocking):**
1. No explicit test for float type usage (verified manually - works correctly)
2. No explicit test for max-heap with different comparable types
3. Internal helpers `_sift_down_max` and `_sift_up_max` not tested directly (but they are implementation details)

### Recommendation

The existing coverage is adequate for the slice scope. The internal sift helpers are implementation details of the public max-heap functions which are thoroughly tested.

---

## 7. Hidden Issues Search

### Potential Hidden Issues Investigated

| Issue | Finding |
|-------|---------|
| Algorithm correctness | ✅ Verified - inverts comparison correctly |
| Ownership model | ✅ Correct - uses `own` for replacement item |
| Type parameter bounds | ✅ Correct - uses `T: Comparable` |
| Return type handling | ✅ Correct - returns `T \| None` for empty heap |
| Mutation semantics | ✅ Correct - uses `mut` for in-place operations |
| Edge case: duplicate max values | ✅ Works correctly |
| Edge case: negative numbers | ✅ Works correctly |
| Edge case: single element | ✅ Works correctly |
| Export policy bypass risk | ✅ Narrow allowlist prevents bypass |

---

## 8. Downstream Case Progress

### Targeted Cases Status

| Case | Previous Status | Current Status | Blocked By |
|------|-----------------|---------------|------------|
| `1046_last_stone_weight` | `undefined variable: 'heapq'` | `CHECK_ERROR` (type annotations) | Downstream typing issues |
| `2971_find_polygon_with_the_largest_perimeter` | `undefined variable: 'heapq'` | `CHECK_ERROR` (optional arithmetic) | Downstream typing issues |

### Assessment

The slice correctly resolves the stdlib surface blocker. The remaining failures are downstream type-system and codegen issues, appropriately classified as follow-up work for `m31_a_optional_narrowing_core` and `m31_b_destructuring_target_lowering`.

---

## 9. Recommendations

### Required (Production Blocking)

None. The implementation is functionally complete and correct.

### Recommended (Non-Blocking)

1. **Add docstrings** to max-heap functions for consistency with min-heap API
2. **Consider adding** float type to the existing E2E test for complete type coverage

---

## 10. Conclusion

| Criterion | Status |
|-----------|--------|
| Edge cases handled | ✅ PASS |
| API contract correct | ✅ PASS |
| Export policy wired | ✅ PASS |
| No regressions | ✅ PASS |
| Documentation complete | ⚠️ Minor gap (docstrings) |
| Test coverage adequate | ✅ PASS |
| Ownership semantics correct | ✅ PASS |

**Overall Assessment: PRODUCTION-READY**

The implementation correctly addresses the root cause and is production-ready. The only recommendation is to add docstrings for API consistency, which is a minor documentation improvement rather than a functional issue.

---

## Action Items

| Priority | Item | Owner |
|----------|------|-------|
| Low | Add docstrings to `_heapify_max`, `_heappop_max`, `_heapreplace_max` | Future follow-up |
| Low | Add float type coverage to E2E test | Future follow-up |

---

*Review completed 2026-03-12*
