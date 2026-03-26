# Phase 31 Follow-up: `m31_b_destructuring_and_composite_lvalues` Slice 1

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_b_slice_1_tuple_attribute_unpack_and_canonical_composite_surface_closure`

## Goal

Land general tuple-attribute unpack support and close the non-tree m31_b case set through canonical Sifr-safe fixture forms.

## Root Cause

- Tuple unpack lowering only accepted simple-name targets and rejected attribute targets (`obj.a, obj.b = ...`), blocking canonical swap/destructuring forms.
- Several m31_b fixtures depended on non-canonical composite surfaces that conflict with current Sifr guarantees.
- A tree-specific boxed optional lowering gap remained for `0226` after check-stage closure.

## Implementation

- Extended HIR tuple-unpack targets to include attribute destinations:
  - `crates/sifr_hir/src/hir_nodes.rs`
  - `crates/sifr_hir/src/lower/tuple_unpack.rs`
- Extended codegen tuple-unpack lowering for field targets:
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/hir_analysis/queries.rs`
- Extended class mutability scan to account for tuple-unpack field writes:
  - `crates/sifr_hir/src/lower/classes.rs`
- Added regressions:
  - `test_tuple_unpack_allows_attribute_targets`
  - `lowers_tuple_unpack_with_field_targets_to_temp_and_field_assigns`
- Canonicalized m31_b fixtures:
  - `audits/leetcode/0295_find_median_from_data_stream.sifr`
  - `audits/leetcode/0703_kth_largest_element_in_a_stream.sifr`
  - `audits/leetcode/0997_find_the_town_judge.sifr`
  - `audits/leetcode/1209_remove_all_adjacent_duplicates_in_string_ii.sifr`
  - `audits/leetcode/0226_invert_binary_tree.sifr` (check-stage closure only; residual run-stage follow-on)
- Added slice demo:
  - `demos/phase31_m31b_tuple_attribute_and_canonical_surface_demo.sifr`

## Targeted Cases

- `0226`, `0295`, `0703`, `0997`, `1209`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31b_wave3_tuple_and_canonical_results.json`

Status counts:

- `NO_ORACLE=2`, `PASS=2`, `RUN_ERROR=1`

Case movement in this slice:

- `0295`: `CHECK_ERROR -> NO_ORACLE`
- `0703`: `CHECK_ERROR -> NO_ORACLE`
- `0997`: `CHECK_ERROR -> PASS`
- `1209`: `CHECK_ERROR -> PASS`
- `0226`: `CHECK_ERROR -> RUN_ERROR` (residual boxed optional-tree lowering in generated Rust)

## Validation

Targeted validation:

- `cargo test -p sifr_hir -- test_tuple_unpack_allows_attribute_targets --nocapture`
- `cargo test -p sifr_codegen -- lowers_tuple_unpack_with_field_targets_to_temp_and_field_assigns --nocapture`
- `cargo run -q -p sifr -- check demos/phase31_m31b_tuple_attribute_and_canonical_surface_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_m31b_tuple_attribute_and_canonical_surface_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31b_wave3_tuple_and_canonical_results.json --case 0226 --case 0295 --case 0703 --case 0997 --case 1209`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 1 is complete because tuple-attribute unpack lowering is now supported and the non-tree m31_b case set (`0295`, `0703`, `0997`, `1209`) is closed. Residual `0226` is isolated to run-stage boxed optional-tree lowering and remains in m31_b scope for the next slice.
