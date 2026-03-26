# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 6

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_6_dict_membership_guard_narrowing`

## Goal

Remove optional-flow leakage for dict index reads when control flow already proves key presence via membership guards.

This slice is scoped to:

- `if key in dict: dict[key]`
- `if expr in dict.keys(): dict[expr]`
- post-guard reads after `if key not in dict: return ...`
- codegen parity so non-optional HIR dict indexes do not reintroduce optional types in generated Rust

It does not attempt to close the full remaining `m31_a` optional-flow backlog.

## Root Cause

- Guard tracking only modeled sequence length/index proofs and had no dict key-presence proof.
- Dict indexing always lowered to optional projection in codegen (`get(...).copied()/cloned()`), even when HIR had already proven a definite non-optional value type.

## Implementation

- Added `SequenceGuard::DictContains` and canonical key-expression tokens:
  - `crates/sifr_hir/src/lower/sequence_guards.rs`
- Added guard detection for:
  - `CmpOp::In` -> true-branch dict key presence
  - `CmpOp::NotIn` -> false-exit/post-guard key presence
  - `dict.keys()` guard source
  - `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
- Added guarded dict index narrowing:
  - `crates/sifr_hir/src/lower/guarded_index.rs`
- Updated simple dict index codegen to follow HIR result type:
  - optional HIR type -> keep optional projection
  - non-optional HIR type -> emit `expect` on projected option
  - `crates/sifr_codegen/src/lower_expr.rs`

## Regression Coverage

- HIR:
  - `test_dict_index_narrows_after_in_membership_guard`
  - `test_dict_index_narrows_after_keys_membership_guard_with_expression_key`
  - `test_dict_index_narrows_after_not_in_early_return_guard`
- Codegen:
  - `lowers_dict_index_to_optional_projection_for_optional_hir_type`
  - `lowers_dict_index_to_expect_for_non_optional_hir_type`
- Demo:
  - `demos/phase31_dict_membership_guard_demo.sifr`

## Targeted Cases

- `0001` `two_sum`
- `0523` `continuous_subarray_sum`
- `0560` `subarray_sum_equals_k`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave6_dict_membership_results.json`

Outcome:

- `PASS=2`
- `RUN_ERROR=1`

Confirmed passes:

- `0523` `continuous_subarray_sum`
- `0560` `subarray_sum_equals_k`

Follow-on reclassification:

- `0001` now fails only on raw fixture missing guaranteed return path under static return typing (`Vec<i64>` expected, fallthrough body returns unit in emitted Rust). This is a canonicalization/closure follow-on, not a dict-membership optional-narrowing failure.

## Validation

Targeted execution validation:

- `cargo test -p sifr_hir dict_index_narrows -- --nocapture`
- `cargo test -p sifr_codegen lowers_dict_index_to_optional_projection_for_optional_hir_type -- --nocapture`
- `cargo test -p sifr_codegen lowers_dict_index_to_expect_for_non_optional_hir_type -- --nocapture`
- `cargo run -q -p sifr -- check demos/phase31_dict_membership_guard_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_dict_membership_guard_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave6_dict_membership_results.json --case 0001 --case 0523 --case 0560`

Local validation gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 6 is complete because dict-membership flow proof now composes through HIR and codegen without optional leakage, and remaining targeted failure (`0001`) is outside this slice root cause.
