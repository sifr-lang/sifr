# Phase 31 Follow-up: `m31_a_optional_narrowing_core` Slice 1

Status: complete
Started: 2026-03-12
Current slice: `m31_a_slice_1_guarded_sequence_index_narrowing`

## Goal

Remove the first high-volume optional-narrowing root cause inside `type_system.optional_narrowing_and_union_ops` by teaching the compiler to treat sequence index results as definite values when control flow already proves the index is in range.

This slice is scoped to list/string indexing and immediate downstream use in expressions such as:

- string concatenation and string methods
- integer arithmetic and comparisons
- membership checks against known element types

It does not attempt to solve the entire optional bucket in one pass. Dict presence/key-flow and unrelated frontend/runtime gaps remain outside this slice.

## Targeted Cases

Primary slice candidates:

- `0014` `longest_common_prefix`
- `0042` `trapping_rain_water`
- `0053` `maximum_subarray`
- `1456` `maximum_number_of_vowels_in_a_substring_of_given_length`
- `1768` `merge_strings_alternately`

Secondary watch cases that may move if the same proof shapes are covered:

- `0198` `house_robber`
- `0238` `product_of_array_except_self`
- `0322` `coin_change`
- `0746` `min_cost_climbing_stairs`
- `1143` `longest_common_subsequence`

## Root-cause Hypothesis

- `index_result_type()` currently returns `T | None` for all list and string indexing, regardless of enclosing control flow.
- Existing narrowing infrastructure only tracks named variable type refinement (`x is not None`, truthiness, `isinstance`, equality). It does not carry non-type flow facts such as:
  - `i < len(seq)`
  - `while i < len(seq)`
  - `for i in range(len(seq))`
  - `if not seq: return` followed by `seq[0]`
- As a result, audit programs that are already bounds-safe in Python remain stuck on artificial `T | None` element types, which then poison arithmetic, comparisons, string operations, and returns.

## Implemented Root-cause Fixes

- Added lowering-time `SequenceGuard` tracking for locally proven bounds facts:
  - `MinLength { sequence, min_len }`
  - `IndexVarInRange { sequence, index_var }`
- Taught statement lowering to detect and preserve the first high-value proof shapes:
  - `while i < len(seq)`
  - `for i in range(len(seq))`
  - early-return non-empty guards such as `if len(seq) == 0: return ...`
- Refined `lower_subscript()` so guarded list/string indexing produces a definite element type instead of `T | None`.
- Preserved the safety contract by leaving unguarded indexing unchanged; unsupported cases still stay optional and type-check accordingly.
- Fixed the matching codegen root cause:
  - the simple statement lowerer no longer prematurely lowers proven non-optional list/string indexes through the legacy optional `.get(...).cloned()` path
  - structured codegen now emits direct list indexing and `chars().nth(...)/let-else` string access for proven-safe non-optional indexes
- Fixed the affected `let-else` rendering/macro rendering details needed for the structured string-index path.

## Regression Coverage

- HIR/type-check tests in `crates/sifr_hir/src/lower/expressions.rs`:
  - `test_guarded_string_index_in_while_reveals_str`
  - `test_range_len_list_index_reveals_element_type`
  - `test_early_return_non_empty_guard_reveals_element_type`
  - `test_early_return_non_empty_guard_let_uses_narrowed_index_type`
  - `test_unguarded_list_index_stays_optional`
- Codegen tests:
  - `crates/sifr_codegen/src/lower_stmt.rs`
    - `simple_let_declines_non_optional_list_index_to_allow_structured_lowering`
    - `simple_return_declines_non_optional_string_index_to_allow_structured_lowering`
  - `crates/sifr_codegen/src/lib_codegen_tests.rs`
    - `test_structured_stmt_path_wraps_non_optional_string_index_into_option_local`
    - `test_structured_stmt_path_handles_non_optional_string_index_return_expr`
- Positive E2E fixture:
  - `crates/sifr/tests/e2e/pass/phase31_guarded_sequence_index_narrowing.sifr`
- Demo:
  - `demos/phase31_guarded_sequence_index_demo.sifr`

## Validation Evidence

- `cargo build -q -p sifr`
- `cargo test -q -p sifr_hir test_guarded_string_index_in_while_reveals_str`
- `cargo test -q -p sifr_hir test_range_len_list_index_reveals_element_type`
- `cargo test -q -p sifr_hir test_early_return_non_empty_guard_reveals_element_type`
- `cargo test -q -p sifr_hir test_early_return_non_empty_guard_let_uses_narrowed_index_type`
- `cargo test -q -p sifr_hir test_unguarded_list_index_stays_optional`
- `cargo test -q -p sifr_codegen simple_let_declines_non_optional_list_index_to_allow_structured_lowering`
- `cargo test -q -p sifr_codegen simple_return_declines_non_optional_string_index_to_allow_structured_lowering`
- `cargo test -q -p sifr_codegen simple_compare_condition_wraps_proven_list_index_without_double_option`
- `cargo test -q -p sifr_codegen test_structured_stmt_path_wraps_non_optional_string_index_into_option_local`
- `cargo test -q -p sifr_codegen test_structured_stmt_path_handles_non_optional_string_index_return_expr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_guarded_sequence_index_narrowing.sifr`
- `target/debug/sifr run demos/phase31_guarded_sequence_index_demo.sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_constructor_compat.sifr`
- `target/debug/sifr check audits/leetcode/0014_longest_common_prefix.sifr`
- `target/debug/sifr check audits/leetcode/0198_house_robber.sifr`
- `target/debug/sifr check audits/leetcode/1768_merge_strings_alternately.sifr`
- `target/debug/sifr emit /tmp/phase31_head_or_zero.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave1_results.json --case 0014 --case 0042 --case 0053 --case 1456 --case 1768 --case 0198 --case 0238 --case 0322 --case 0746 --case 1143`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Current Measured Outcome

Primary slice artifact:

- `verification/leetcode/phase31_m31a_wave1_results.json`

Observed movement across the 10-case watch set:

- `PASS=3`, `CHECK_ERROR=7`, `RUN_ERROR=0`
- Newly passing cases:
  - `0014` `longest_common_prefix`
  - `0198` `house_robber`
  - `1768` `merge_strings_alternately`
- Confirmed root-cause removal:
  - the prior guarded-index `T | None` failures are removed from the passing cases above
  - `head_or_zero`-style early-return fixed indexing now emits direct Rust indexing instead of optional `.get(...).cloned()`
  - compare-condition lowering no longer produces `Option<Option<T>>` when a guarded list index is compared against an unguarded optional index
- Remaining failures are now narrower follow-on shapes:
  - `0042`, `0238`: arithmetic still fed by other unproven optional index paths
  - `0053`, `0322`, `0746`, `1143`: accumulator/return-flow paths still produce `int | None`
  - `1456`: sliding-window membership still sees `str | None` on a not-yet-proven index shape

## Remaining Slice-local Follow-on

- This slice only covers explicit local proof shapes already present in loop/early-return forms.
- The remaining watch-list failures need additional optional narrowing work for:
  - moving-index/two-pointer relationships
  - accumulator rebinding where the element proof is indirect
  - broader sliding-window and DP recurrence flows
- Those follow-on shapes remain inside `m31_a_optional_narrowing_core`, but outside this slice’s guard-tracking boundary.

## Definition Of Done

- The targeted guarded-index cases move past the current `T | None` element failures.
- Refined indexing only applies when the proof shape is explicit and local; unsafe/unproven indexing keeps the existing safe optional result.
- New regression coverage locks the guarded-index behavior in HIR/type-check and E2E tests.
- Remaining failures in the targeted cases are reclassified to narrower downstream work instead of generic optional-index errors.
