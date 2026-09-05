# Phase 31 m31a Slice 2 Review: Same-Sequence Two-Pointer While Guard Narrowing

**Reviewer:** agent
**Date:** 2026-03-12
**Slice:** `m31_a_optional_narrowing_core` - slice 2 (two-pointer while guard narrowing)
**Status:** **NEEDS WORK** - Maintainability regression (exceeds HIR line limit)

---

## Executive Summary

The slice implements same-sequence two-pointer `while` guard narrowing, teaching the compiler to recognize explicit two-pointer loops where both pointers are bounded by construction (`left = 0`, `right = len(seq) - 1`, `while left < right`). The implementation is architecturally sound and correctly solves the root cause. However, the slice introduces a **maintainability regression**: `expressions.rs` now exceeds the 3800-line limit by 1 line.

---

## 1. Root Cause Analysis - Correct

### 1.1 Problem Statement

**Root Cause Hypothesis (from execution tracking):**
- Slice 1 only handled explicit bounds proofs tied directly to a sequence length expression (`i < len(seq)` or `for i in range(len(seq))`).
- Two-pointer algorithms often prove safety indirectly:
  - `left = 0`
  - `right = len(seq) - 1`
  - `while left < right:`
  - then index `seq[left]` and `seq[right]` after single-step pointer movement
- The current narrowing layer did not preserve these pointer-role facts.

### 1.2 Implementation Approach

The implementation correctly addresses this by:

1. **Pointer Role Tracking** (`sequence_pointers.rs`):
   - Tracks `ZeroBased` pointers: variables initialized from `0`
   - Tracks `EndPointer` variables: variables initialized from `len(seq) - 1`
   - Provides `same_sequence_two_pointer_loop()` to detect when both pointers target the same sequence

2. **Two-Pointer Guard Detection** (`sequence_guard_detection.rs`):
   - Detects `while left < right` where both variables are recognized as same-sequence pointers
   - Validates loop body preserves pointer discipline with single-step updates only (`+= 1` / `-= 1`)
   - Rejects unsupported pointer mutations (jumps, complex arithmetic)

3. **Guarded Index Integration** (`guarded_index.rs`):
   - Extends `has_guarded_sequence_index()` to recognize zero/end pointer locals
   - Applies narrowing only when non-empty flow is already proven (`min_length_guard > 0`)

---

## 2. Correctness Verification

### 2.1 Unit Tests

All 8 guarded index tests pass:

```
cargo test -q -p sifr_hir -- guarded_index
running 8 tests
........
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 46 filtered out; finished in 0.01s
```

Tests include:
- `test_two_pointer_while_reveals_element_type_after_single_step_updates` - Verifies narrowing works
- `test_two_pointer_while_with_pointer_jump_stays_optional` - Verifies unsafe jumps stay optional
- `test_non_empty_zero_and_end_pointers_reveal_element_type` - Verifies post-guard pointer reads

### 2.2 Demo and E2E

Demo runs successfully:
```
$ target/debug/sifr run demos/phase31_two_pointer_guard_demo.sifr
two_pointer_water_1 = 6
two_pointer_water_2 = 9
```

E2E test passes:
```
$ target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_two_pointer_while_guard_narrowing.sifr
(no output - assertions pass)
```

### 2.3 LeetCode Case

Primary target case `0042_trapping_rain_water` now passes:
```
$ target/debug/sifr check audits/leetcode/0042_trapping_rain_water.sifr
no errors found

$ target/debug/sifr run audits/leetcode/0042_trapping_rain_water.sifr
(no output - test cases pass)
```

---

## 3. Production-Path Codegen Behavior

### 3.1 Codegen Fixes Required

The slice exposed and fixed two production-path codegen gaps:

1. **Tuple-Unpack Fallback** (`stmt_support_emitter.rs` lines 3244-3255):
   - Added handling for `HirStmt::TupleUnpack` when simple lowering fails
   - Enables tuple unpacking like `l, r = 0, len(height) - 1`

2. **Numeric AugAssign in Nested Blocks** (`stmt_support_emitter.rs` lines 3323-3340):
   - Added handling for `HirStmt::AugAssign` when simple lowering fails
   - Enables statements like `l += 1`, `r -= 1`, `total += ...` in loop bodies

### 3.2 Codegen Correctness

The codegen correctly:
- Emits Rust pattern let-statements for tuple unpacking
- Emits proper augmented assignment statements for pointer updates
- Uses the IR expression path for value lowering

---

## 4. Regression Analysis

### 4.1 Functional Regressions

**None detected.** The implementation:
- Correctly narrows safe two-pointer indices
- Correctly rejects unsafe pointer jumps
- Does not widen semantics unsafely

### 4.2 Maintainability Regression

**FAILURE:** `expressions.rs` exceeds line limit.

| Metric | Before | After | Limit | Status |
|--------|--------|-------|-------|--------|
| `expressions.rs` lines | 3784 | 3801 | 3800 | +1 over limit |

The slice added 17 lines to `expressions.rs` (the pointer fact recording in `lower_tuple_unpack_assign`).

**Fix Required:** Reduce `expressions.rs` by at least 1 line to comply with the 3800-line limit.

---

## 5. Safety and Soundness

### 5.1 Unsafe Widening Prevention

The implementation correctly prevents unsafe widening:

1. **Single-Step Validation** (lines 213-237 in `sequence_guard_detection.rs`):
   - Only accepts `+= 1` for left pointer
   - Only accepts `-= 1` for right pointer
   - Rejects jumps like `l += 2`

2. **Conservative Body Analysis**:
   - If any statement in the loop body doesn't preserve pointer discipline, the entire loop stays optional
   - Nested statements (if/elif/else) must all preserve pointer discipline

3. **Test Verification**:
   - `test_two_pointer_while_with_pointer_jump_stays_optional` confirms jumps remain `T | None`

### 5.2 Pointer Safety Proof

The narrowing is sound because:

1. **Initialization Guarantee**: Zero-based pointer starts at 0 (valid index if sequence non-empty)
2. **End Pointer Guarantee**: `len(seq) - 1` is the last valid index if sequence non-empty
3. **Loop Invariant**: `left < right` ensures both pointers stay in valid range
4. **Step Discipline**: Single-step updates preserve the invariant

---

## 6. Slice Scope Compliance

### 6.1 What the Slice Covers (Intentional)

- ✅ Pointers initialized from `0` and `len(seq) - 1`
- ✅ Loop conditions of the form `left < right`
- ✅ Loop bodies where pointers move by single safe steps (`+= 1` / `-= 1`)
- ✅ Direct downstream indexing with pointer variables inside the guarded loop body

### 6.2 What the Slice Correctly Excludes (Intentional)

- ❌ Sliding-window left pointers inferred from complex arithmetic
- ❌ Aliased length variables (`n = len(seq)`)
- ❌ Reverse `range(...)` proofs
- ❌ Generalized affine proofs (`seq[i + 2]`, `seq[a - c]`)

The exclusions are correct - these require more complex proof techniques beyond same-sequence two-pointer narrowing.

---

## 7. Summary and Recommendations

### 7.1 Summary

| Aspect | Status |
|--------|--------|
| Root Cause Resolution | ✅ Correct |
| Functional Correctness | ✅ Pass |
| Codegen Behavior | ✅ Correct |
| Unsafe Widening Prevention | ✅ Safe |
| Slice Scope | ✅ Compliant |
| **Maintainability** | ❌ **FAIL** - Line limit exceeded |

### 7.2 Required Actions

1. **Fix maintainability regression**: Reduce `expressions.rs` by at least 1 line
   - Option 1: Move some existing code out of `expressions.rs`
   - Option 2: Refactor existing code in `expressions.rs` to be more compact
   - Option 3: Move the new pointer recording logic to a different module

### 7.3 Validation Commands

After the fix, run:
```bash
scripts/run_all_tests.sh  # Full validation
scripts/run_all_tests.sh --profile quick  # Fast validation
```

---

## 8. Review Checklist

- [x] Root cause correctly identified and addressed
- [x] Implementation matches stated scope (no overreach)
- [x] Unit tests pass
- [x] Demo runs correctly
- [x] E2E test passes
- [x] LeetCode case passes
- [x] No functional regressions
- [x] Unsafe widening prevented
- [x] Codegen handles production paths
- [ ] **Maintainability: expressions.rs under 3800 lines**
