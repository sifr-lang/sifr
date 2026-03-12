# Phase 31 Follow-up: `m31_a_optional_narrowing_core` Slice 3

Status: complete
Started: 2026-03-12
Current slice: `m31_a_slice_3_sliding_window_left_pointer`

## Goal

Remove the next optional-narrowing root cause inside `type_system.optional_narrowing_and_union_ops` by teaching the compiler to recognize canonical sliding-window loops where a left pointer starts at `0`, a right pointer is driven by `for r in range(len(seq))`, and the left pointer only advances monotonically by single steps.

This slice is scoped to:

- right pointers proven by `range(len(seq))`
- left pointers initialized from `0`
- left pointers that only move by `+= 1` or equivalent assignment forms
- direct downstream indexing with the left pointer inside the loop body
- downstream codegen paths that still render proven non-optional string indexes as `Option[str]`

It does not attempt to solve:

- generalized affine proofs such as `seq[l + 2]`
- sliding windows with pointer resets or non-monotone movement
- reverse-range / descending-index proofs
- recurrence or constructed-sequence min-length proofs

## Targeted Cases

Primary slice candidates:

- `0003` `longest_substring_without_repeating_characters`
- `1456` `maximum_number_of_vowels_in_a_substring_of_given_length`

Secondary watch cases that may reclassify if the same proof shape helps:

- `0209` `minimum_size_subarray_sum`

## Root-cause Hypothesis

- Slice 1 covered direct `i < len(seq)` and `for i in range(len(seq))` guards.
- Slice 2 covered explicit same-sequence `while left < right` pointer loops.
- Canonical sliding-window programs use a different proof shape:
  - `l = 0`
  - `for r in range(len(seq)):`
  - optional inner conditions or `while` loops
  - `l` only advances by one step and never resets
- The compiler does not currently preserve the derived invariant that `l <= r` at each loop iteration, so `seq[l]` remains optional even though the loop structure keeps the pointer in range.
- One downstream codegen path still violates that proof even after type checking succeeds: proven non-optional string indexing used as a method-call argument can still render as `Option<String>`, which is what currently breaks `0003` during Rust compilation.

## Implemented Root-cause Fixes

- Extended range-based guard detection so canonical `for r in range(len(seq))` loops can add a guarded in-range fact for a left pointer when:
  - the pointer starts from `0`,
  - the pointer is actually used as `seq[l]` inside the loop body,
  - the body only permits single-step monotone increments, and
  - no later `seq[l]` reads happen after a branch that may increment `l`.
- Made the sliding-window candidate discovery robust for tuple-unpacked zero-based locals by accepting either:
  - an existing zero-based pointer fact, or
  - an inferred `LiteralInt(0)` start state on the indexed variable.
- Preserved safety by keeping the guard path narrow:
  - nested loop bodies do not gain the new proof,
  - pointer resets or jumps are still rejected,
  - post-merge reads after a possible increment stay optional.
- Removed the downstream `0003` run-path failure as a consequence of the improved proof:
  - `s[l]` in `charSet.remove(s[l])` now lowers through the existing proven string-index path instead of leaking an `Option<String>` into generated Rust.

## Regression Coverage

- HIR/type-check tests in `crates/sifr_hir/src/lower/guarded_index.rs`:
  - `test_sliding_window_left_pointer_reveals_element_type_before_single_step_increment`
  - `test_tuple_unpack_sliding_window_left_pointer_reveals_element_type`
  - `test_sliding_window_left_pointer_stays_optional_after_incremented_branch_merges`
- Positive E2E fixture:
  - `crates/sifr/tests/e2e/pass/phase31_sliding_window_left_pointer_narrowing.sifr`
- Demo:
  - `demos/phase31_sliding_window_left_pointer_demo.sifr`

## Acceptance Criteria

- `1456_maximum_number_of_vowels_in_a_substring_of_given_length` moves past the current left-pointer `str | None` failure.
- `0003_longest_substring_without_repeating_characters` no longer fails generated Rust compilation because of `Option<String>` leakage from a proven string index.
- Unsupported sliding-window pointer mutation shapes still remain optional instead of being narrowed unsafely.

## Validation Evidence

- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_sliding_window_left_pointer_narrowing.sifr`
- `target/debug/sifr run demos/phase31_sliding_window_left_pointer_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave3_results.json --case 0003 --case 0209 --case 1456`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Current Measured Outcome

Primary slice artifact:

- `verification/leetcode/phase31_m31a_wave3_results.json`

Observed movement across the targeted three-case slice:

- `PASS=2`, `CHECK_ERROR=1`, `RUN_ERROR=0`
- Newly passing cases:
  - `0003` `longest_substring_without_repeating_characters`
  - `1456` `maximum_number_of_vowels_in_a_substring_of_given_length`
- Confirmed reclassification signal:
  - `0209` `minimum_size_subarray_sum` no longer carries the left-pointer indexing failure and is now blocked by the unrelated `float('inf')` / branch-type mismatch.
