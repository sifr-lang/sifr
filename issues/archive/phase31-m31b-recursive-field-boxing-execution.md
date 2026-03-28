# Phase 31 Follow-up: `m31_b_destructuring_and_composite_lvalues` Slice 2

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_b_slice_2_recursive_optional_field_boxing_closure`

## Goal

Close the residual `0226` run-stage failure by fixing recursive optional-field assignment lowering at the compiler root cause.

## Root Cause

- HIR accepted assignments from `TreeNode` into recursive optional fields typed `TreeNode | None`.
- Rust lowering emitted direct assignment without coercion to recursive storage shape (`Option<Box<TreeNode>>`), causing run-stage rustc mismatches.
- This affected `0226` and any equivalent recursive-field assignment shape.

## Implementation

- Added field type metadata on HIR field-assignment statements:
  - `crates/sifr_hir/src/hir_nodes.rs`
  - `crates/sifr_hir/src/lower/statements.rs`
- Added codegen coercion for recursive optional-field assignments:
  - detect recursive field storage from class metadata + field type
  - coerce `T` -> `Some(Box::new(T))` where target field is recursive `T | None`
  - preserve existing `None` assignment behavior
  - files: `crates/sifr_codegen/src/stmt_support_emitter.rs`, `crates/sifr_codegen/src/lower_stmt.rs`

## Targeted Cases

- `0226`, `0295`, `0703`, `0997`, `1209`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31b_wave4_recursive_field_boxing_results.json`

Status counts:

- `NO_ORACLE=3`, `PASS=2`

Case movement in this slice:

- `0226`: `RUN_ERROR -> NO_ORACLE`
- `0295`, `0703`, `0997`, `1209`: remained green from slice 1

## Validation

Targeted validation:

- `cargo test -p sifr_hir -- test_tuple_unpack_allows_attribute_targets --nocapture`
- `cargo test -p sifr_codegen -- lowers_tuple_unpack_with_field_targets_to_temp_and_field_assigns --nocapture`
- `cargo run -q -p sifr -- check audits/leetcode/0226_invert_binary_tree.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0226_invert_binary_tree.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31b_wave4_recursive_field_boxing_results.json --case 0226 --case 0295 --case 0703 --case 0997 --case 1209`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 2 is complete because the residual recursive optional-field assignment mismatch is removed and all m31_b targeted ids are now green (`NO_ORACLE`/`PASS`). `m31_b_destructuring_and_composite_lvalues` is closed.
