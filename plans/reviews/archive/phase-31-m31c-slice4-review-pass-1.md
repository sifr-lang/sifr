# Phase 31 m31_c Slice 4 Review: Private heapq Max-Heap Compatibility

**Reviewer:** agent
**Date:** 2026-03-12
**Slice:** `m31_c_slice_4_private_heapq_max_compat`
**PR:** #1112
**Status:** ✅ APPROVED

---

## Executive Summary

Slice 4 of the `m31_c_stdlib_module_parity` milestone correctly implements the private `heapq` max-heap compatibility layer. The implementation addresses the root cause by adding the missing stdlib surface and wiring it through the export policy. All validation evidence passes, and the remaining failures are correctly identified as downstream non-stdlib issues.

---

## 1. Slice Completion Against Documented Scope

### Documented Scope (from execution report)

The slice was required to:

1. Add pure-Sifr max-heap helpers to `sifr.heapq`:
   - `_sift_down_max`
   - `_sift_up_max`
   - `_heapify_max`
   - `_heappop_max`
   - `_heapreplace_max`

2. Mark `_heapreplace_max`'s replacement item as `own` for ownership/codegen rules

3. Add a shared export-policy helper in `sifr_driver` that allowlists only the intentional private `sifr.heapq` max-heap surface

4. Wire that allowlist through:
   - Project export collection
   - Stdlib function/default export collection
   - Stdlib signature export collection used by codegen/compat imports

### Implementation Delivered

| Requirement | Delivered | Evidence |
|------------|-----------|----------|
| Max-heap helpers in heapq.sifr | ✅ | `lib/sifr/heapq.sifr` lines 70-182 |
| `_heapreplace_max` item as `own` | ✅ | Line 176: `own item: T` |
| Export policy allowlist | ✅ | `export_policy.rs` lines 1-10 |
| Wired through bootstrap | ✅ | `bootstrap.rs` lines 78, 95, 230, 290 |
| Wired through project exports | ✅ | `project/exports.rs` lines 19, 36 |

**Scope Coverage:** COMPLETE ✅

---

## 2. Regressions

### Local Validation Results

| Validation | Result |
|------------|--------|
| `scripts/run_all_tests.sh --profile quick` | ✅ PASS |
| `scripts/run_all_tests.sh` | ✅ PASS |
| Unit test `stdlib_heapq_exports_allowlisted_private_max_heap_helpers` | ✅ PASS |
| E2E test `phase31_heapq_max_private_compat.sifr` | ✅ PASS |
| Demo `phase31_heapq_max_compat_demo.sifr` | ✅ PASS |

### Regression Assessment

- No compiler regressions introduced
- No stdlib regressions introduced
- Export policy change is narrowly scoped to heapq max-heap functions only

**Regressions:** NONE ✅

---

## 3. Root-Cause Coverage

### Root Cause Analysis

**Original Problem:**
- `sifr.heapq` implemented only public min-heap helpers
- Private max-heap helpers (`_heapify_max`, `_heappop_max`, `_heapreplace_max`) were not implemented
- The stdlib export policy stripped ALL leading-underscore callables, preventing even if-they-existed implementations from being visible

**Root-Cause Fix:**
1. Implemented the pure-Sifr max-heap algorithms (matching CPython behavior)
2. Added narrow export-policy allowlist that exposes only the three intentional private helpers
3. Wired the policy through all stdlib export collection paths

### Evidence of Root-Cause Resolution

**Before (would fail at import):**
```
undefined variable: 'heapq'
```

**After (resolves import, fails on downstream type issues):**
```
type error: parameter 'stones' in function 'lastStoneWeight' is missing a type annotation
type error: argument 1 ('data') of function '__compat_sifr_heapq__heapify_max': expected 'list[T]', got 'Any'
```

The presence of `__compat_sifr_heapq__heapify_max` in the error message confirms the function is now resolving through the compat import path. The remaining errors are type-system issues (not stdlib surface issues).

**Root-Cause Coverage:** COMPLETE ✅

---

## 4. Milestone Tracking Accuracy

### Execution Report Claims vs. Observed

| Claim | Verification |
|-------|--------------|
| Targeted six-case status: `PASS=2`, `CHECK_ERROR=3`, `RUN_ERROR=1` | ✅ Matches `phase31_m31c_wave4_results.json` |
| `1046` moves past heapq undefined error into deeper typing | ✅ Verified - errors now about `Any`/annotations |
| `2971` resolves private heapq, fails on optional arithmetic | ✅ Verified - error is `int \| None` arithmetic |
| Remaining blockers are downstream codegen/type work | ✅ Confirmed - `0003` codegen panic, others type errors |

### Milestone Closure Appropriateness

The execution report correctly identifies:
- The stdlib surface blocker is removed
- Remaining failures are downstream (not stdlib)
- Watch list cases properly reclassified

**Milestone Tracking:** ACCURATE ✅

---

## 5. Tests and Demos

### Regression Coverage

| Test | Type | Status |
|------|------|--------|
| `phase31_heapq_max_private_compat.sifr` | E2E | ✅ PASS |
| `phase31_heapq_max_compat_demo.sifr` | Demo | ✅ PASS |
| `stdlib_heapq_exports_allowlisted_private_max_heap_helpers` | Unit | ✅ PASS |

### Test Quality Assessment

**E2E Test (`phase31_heapq_max_private_compat.sifr`):**
- Tests `_heapify_max`, `_heappop_max`, `_heapreplace_max` directly
- Validates max-heap ordering behavior
- Tests edge cases (empty heap)
- Uses explicit type annotations
- **Quality:** GOOD ✅

**Demo (`phase31_heapq_max_compat_demo.sifr`):**
- Tests the same functions with assertions
- Uses `assert_bool_vector_eq` for validation
- **Quality:** GOOD ✅

**Unit Test (`stdlib_exports.rs`):**
- Verifies allowlisted functions exist in compiled stdlib
- Explicitly checks for `_heapify_max`, `_heappop_max`, `_heapreplace_max`
- **Quality:** GOOD ✅

### Missing Coverage

**Potential gap:** No explicit test for:
- `_sift_down_max` and `_sift_up_max` as internal helpers (but they are implementation details)
- Interaction with other stdlib modules

**Assessment:** Coverage is adequate for the slice scope. The internal sift helpers are implementation details of the public max-heap functions which are tested.

---

## 6. Additional Observations

### Design Quality

1. **Narrow Export Policy:** The `should_export_callable` function uses an explicit match statement rather than a broader pattern, ensuring only the intended private functions are exposed. This is good security/correctness hygiene.

2. **Ownership Correctness:** The `_heapreplace_max` function correctly marks the `item` parameter as `own`, matching Sifr's ownership semantics when storing back into the heap.

3. **Algorithm Correctness:** The max-heap implementations correctly invert the comparison logic from the min-heap implementations.

### Consistency with Other Slices

The slice follows the same pattern as previous m31_c slices:
- Clear root-cause hypothesis
- Targeted case selection
- Narrow implementation
- Proper reclassification of downstream failures

---

## 7. Definition of Done Assessment

| Done Criterion | Status |
|----------------|--------|
| Private heapq max functions resolve through compat path | ✅ |
| Export policy stays narrow/intentional | ✅ |
| Seeded case `1046` moves past undefined-symbol failure | ✅ |
| Direct parity probe proves max-heap behavior | ✅ |

---

## Conclusion

**Slice 4 Implementation: APPROVED ✅**

The implementation is complete, correct, and properly scoped. It:
- Addresses the documented root cause
- Introduces no regressions
- Includes adequate test coverage
- Correctly identifies downstream failures as non-stdlib work

The remaining failures in the watch list are correctly classified as downstream type-system and codegen work, appropriately owned by other follow-up milestones (`m31_a_optional_narrowing_core`, `m31_b_destructuring_target_lowering`).

---

## Recommendations

None required. The implementation is sound and ready for merge.
