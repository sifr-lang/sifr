# Phase 31 Follow-up: `m31_a_optional_narrowing_core` Slice 2

Status: complete
Started: 2026-03-12
Current slice: `m31_a_slice_2_two_pointer_while_guard_narrowing`

## Goal

Remove the next optional-narrowing root cause inside `type_system.optional_narrowing_and_union_ops` by teaching the compiler to recognize explicit same-sequence two-pointer `while` loops where both pointers are already bounded by construction.

This slice is intentionally limited to:

- pointers initialized from `0` and `len(seq) - 1`
- loop conditions of the form `left < right`
- loop bodies where `left` and `right` only move by single safe steps (`+= 1` / `-= 1`)
- direct downstream indexing with those pointer variables inside the guarded loop body

It does not attempt to solve:

- sliding-window left pointers inferred from more complex arithmetic relationships
- aliased length variables (`n = len(seq)`)
- reverse `range(...)` proofs
- generalized affine proofs such as `seq[i + 2]` or `seq[a - c]`

## Targeted Cases

Primary slice candidate:

- `0042` `trapping_rain_water`

Secondary watch cases that may reclassify if the same proof shape helps:

- `1456` `maximum_number_of_vowels_in_a_substring_of_given_length`
- `0215` `kth_largest_element_in_an_array`

## Root-cause Hypothesis

- Slice 1 only handled explicit bounds proofs tied directly to a sequence length expression such as `i < len(seq)` or `for i in range(len(seq))`.
- Two-pointer algorithms often prove safety indirectly:
  - `left = 0`
  - `right = len(seq) - 1`
  - `while left < right:`
  - then index `seq[left]` and `seq[right]` after single-step pointer movement
- The current narrowing layer does not preserve these pointer-role facts, so the type checker still sees `seq[left]` and `seq[right]` as `T | None` even when the loop condition plus pointer construction makes those indexes safe.

## Planned Root-cause Fix

- Track simple same-sequence pointer facts in lowering:
  - start pointer initialized to `0`
  - end pointer initialized to `len(seq) - 1`
- Detect `while left < right` loops where both variables are recognized as pointers into the same sequence.
- Only apply guarded indexing when the loop body preserves the pointer discipline with single-step monotone updates.
- Reuse the existing guarded-index narrowing/codegen path once the body-local sequence guards are available.

## Implemented Root-cause Fixes

- Added lowering-time pointer-role tracking for:
  - zero-based pointer locals initialized from `0`
  - end-pointer locals initialized from `len(seq) - 1`
- Extended guarded indexing so zero/end pointer locals become definite element reads once non-empty flow is already proven.
- Added explicit same-sequence two-pointer `while left < right` guard detection with body-shape validation:
  - only single-step pointer movement (`+= 1` / `-= 1`, or equivalent assignment form) is accepted
  - unsupported pointer mutation shapes stay optional
- Fixed the missing post-`if not seq: return` non-empty propagation so the same non-empty proof applies to pointer setup reads.
- Landed the downstream production-path codegen fixes exposed by the slice:
  - tuple-unpack fallback for structured statement emission now accepts tuple values lowered through the IR expression path
  - nested statement-block lowering now supports numeric `AugAssign` emission, unblocking two-pointer loop bodies such as `l += 1`, `r -= 1`, and `res += ...`

## Regression Coverage

- HIR/type-check tests in `crates/sifr_hir/src/lower/guarded_index.rs`:
  - `test_two_pointer_while_reveals_element_type_after_single_step_updates`
  - `test_two_pointer_while_with_pointer_jump_stays_optional`
  - `test_non_empty_zero_and_end_pointers_reveal_element_type`
- Positive E2E fixture:
  - `crates/sifr/tests/e2e/pass/phase31_two_pointer_while_guard_narrowing.sifr`
- Demo:
  - `demos/phase31_two_pointer_guard_demo.sifr`

## Acceptance Criteria

- `0042_trapping_rain_water` moves past the current `int | None` two-pointer index failures.
- New regression coverage proves that same-sequence `left/right` pointer loops refine direct indexing to concrete element types.
- Unsafe or unsupported pointer-update shapes remain rejected rather than silently narrowed.

## Validation Evidence

- `cargo test -q -p sifr_hir guarded_index`
- `cargo build -q -p sifr`
- `target/debug/sifr check audits/leetcode/0042_trapping_rain_water.sifr`
- `target/debug/sifr run audits/leetcode/0042_trapping_rain_water.sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_two_pointer_while_guard_narrowing.sifr`
- `target/debug/sifr run demos/phase31_two_pointer_guard_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave2_results.json --case 0014 --case 0042 --case 0053 --case 1456 --case 1768 --case 0198 --case 0238 --case 0322 --case 0746 --case 1143`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Current Measured Outcome

Primary slice artifact:

- `verification/leetcode/phase31_m31a_wave2_results.json`

Observed movement across the 10-case watch set:

- `PASS=4`, `CHECK_ERROR=6`, `RUN_ERROR=0`
- Newly passing case:
  - `0042` `trapping_rain_water`
- Confirmed root-cause removal:
  - same-sequence two-pointer `while` loops now type-check when the pointer construction and body updates preserve explicit safety
  - zero/end pointer setup reads after `if not seq: return` now lower as concrete element values instead of `T | None`
  - the previously exposed tuple-unpack / nested-`AugAssign` production-path emit gaps are removed for the slice fixture and audit case
- Remaining watch-set follow-on failures stay narrower than this slice:
  - `1456`: sliding-window left-pointer proof is still indirect
  - `0053`: first-element initialization still lacks an explicit non-empty proof
  - `0238`, `0322`, `0746`, `1143`: reverse-range / constructed-sequence / recurrence paths remain outside this slice
