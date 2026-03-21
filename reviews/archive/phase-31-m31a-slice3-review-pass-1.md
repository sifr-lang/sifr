# Phase 31 m31a Slice 3 Review (Sliding-Window Left-Pointer Narrowing)

**Review ID:** phase-31-m31a-slice3-review-pass-1
**Date:** 2026-03-12
**Commit:** 211dcda3
**Feature:** Canonical sliding-window left-pointer narrowing

## Summary

Slice 3 of the `m31_a_optional_narrowing_core` milestone implements the compiler's ability to recognize and prove safe canonical sliding-window loops where a left pointer starts at 0, a right pointer is driven by `for r in range(len(seq))`, and the left pointer only advances monotonically by single steps. The implementation is **correct and safe**.

## What Was Implemented

### Guarded-Index Narrowing (PR #1119)

The implementation adds new logic in `sequence_guard_detection.rs` that extends range-based guard detection to handle sliding-window left pointers:

1. **Range sequence extraction** (`range_sequence_name`): Extracts the sequence name from `range(len(seq))` calls.

2. **Sliding-window pointer detection** (`detect_sliding_window_pointer_guards`): Discovers candidate left pointers in the loop body that:
   - Are used as `seq[l]` subscript indices
   - Are different from the right pointer variable
   - Are either zero-based (via `is_zero_based_pointer`) or have an inferred `LiteralInt(0)` start state (via `effective_type`)
   - Only undergo single-step monotone increments (`+= 1` or `l = l + 1`)

3. **Pointer mutation analysis** (`loop_body_preserves_sliding_window_pointer`): Analyzes the loop body to ensure:
   - No nested loops gain the new proof
   - Pointer resets or jumps are rejected
   - Post-branch reads after possible increments remain optional

### Tuple-Unpacked Zero-Based Locals

The implementation supports both explicit zero-based pointers and inferred zero-based pointers through type information:

- **Explicit**: `l = 0` or `l: int = 0` - tracked via `set_zero_based_pointer` in `sequence_pointers.rs`
- **Inferred**: `l, total = 0, 0` - tracked via `effective_type(left_var)` returning `Type::LiteralInt(0)`

This dual-path approach ensures robustness for both explicit initialization and tuple unpacking patterns common in sliding-window code.

### Safety of the New Proof

The implementation is correctly scoped to preserve safety:

1. **Monotonicity enforcement**: Only accepts `+= 1` or `l = l + 1` mutations
2. **Branch sensitivity**: After a branch that may increment the pointer, subsequent reads stay optional
3. **Nested loop isolation**: Nested loops do not inherit the proof
4. **No affine proofs**: Does not attempt generalized affine proofs like `seq[l + 2]`

The negative test `test_sliding_window_left_pointer_stays_optional_after_incremented_branch_merges` explicitly verifies that reads after a potential increment branch remain optional.

## Regression Coverage

### Unit Tests (`crates/sifr_hir/src/lower/guarded_index.rs`)

1. `test_sliding_window_left_pointer_reveals_element_type_before_single_step_increment` - Verifies type revelation before increment
2. `test_tuple_unpack_sliding_window_left_pointer_reveals_element_type` - Verifies tuple-unpacked pointers work
3. `test_sliding_window_left_pointer_stays_optional_after_incremented_branch_merges` - Verifies unsafe narrowing is prevented

### E2E Tests

- `phase31_sliding_window_left_pointer_narrowing.sifr` - Full sliding-window patterns including:
  - Tuple-unpacked zero-based pointers (`l, res, total = 0, 0, 0`)
  - Direct string indexing (`s[l] in vowels`)
  - `set.remove(s[l])` method call argument (the downstream codegen path that was fixed)

### Demo

- `demos/phase31_sliding_window_left_pointer_demo.sifr` - Two LeetCode-style problems:
  - `maxVowels` - Sliding window vowel count
  - `lengthOfLongestSubstring` - Sliding window with inner while loop

## Validation Results

### Local Test Suite
```
scripts/run_all_tests.sh --profile quick: PASS
scripts/run_all_tests.sh: PASS
```

### LeetCode Verification (Wave 3)

| Case ID | Problem | Status | Notes |
|---------|---------|--------|-------|
| 0003 | longest_substring_without_repeating_characters | PASS | Previously blocked by `Option<String>` codegen leakage |
| 1456 | maximum_number_of_vowels_in_a_substring_of_given_length | PASS | Previously blocked by `str \| None` in `in` operator |
| 0209 | minimum_size_subarray_sum | CHECK_ERROR | Reclassified: no longer left-pointer failure, now `float('inf')` branch-type mismatch |

**Result:** `PASS=2`, `CHECK_ERROR=1`, `RUN_ERROR=0` (targeted 3 cases)

## Scope Assessment Against Remaining Backlog

The slice correctly targets the canonical sliding-window pattern and does **not** attempt to solve:

- Generalized affine proofs (`seq[l + 2]`)
- Sliding windows with pointer resets or non-monotone movement
- Reverse-range / descending-index proofs
- Recurrence or constructed-sequence min-length proofs

### Remaining m31a Backlog (After Slice 3)

Based on the execution log in `phase31-ad-hoc-followup-milestones.md`:

- **Slice 1** solved: guarded sequence-index (explicit `while i < len(seq)`, `for i in range(len(seq))`)
- **Slice 2** solved: same-sequence two-pointer `while left < right`
- **Slice 3** solved: canonical sliding-window left-pointer

**Remaining failure patterns** in the optional narrowing bucket are concentrated in:
- Constructed-sequence proofs
- Recurrence patterns
- Broader arithmetic/branch-type proof gaps

This aligns with the documented scope in the execution report: "remaining optional failures are now concentrated in constructed-sequence, recurrence, and broader arithmetic/branch-type proof gaps rather than guarded-index, same-sequence two-pointer, or canonical sliding-window left-pointer roots."

## Quality Observations

1. **Well-scoped**: The implementation correctly limits itself to the canonical sliding-window pattern
2. **Safe by design**: The pointer mutation analysis ensures no unsafe narrowing
3. **Good test coverage**: Both positive cases (proved narrowing) and negative cases (preserved optionality) are covered
4. **Robust detection**: Supports both explicit zero-based initialization and tuple-unpacked inference via `effective_type`
5. **Proper documentation**: Execution report clearly documents what's solved and what's intentionally left for future work

## Issues Found

None. The implementation is sound and the tests pass.

## Recommendation

**APPROVE** - The slice is ready for merge. It correctly implements canonical sliding-window left-pointer narrowing with proper safety guarantees, comprehensive test coverage, and clear scope boundaries against the remaining Phase 31 backlog.
