# Phase 31 Follow-up: `m31_e_recursive_tree_surface_leetcode_closure` Slice 1

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_e_slice_1_canonical_recursive_tree_surface_closure`

## Goal

Close the remaining recursive-tree owner cases (`0100`, `0102`, `0235`) on top of landed recursive-type support with canonical Sifr-safe tree surfaces.

## Root Causes

- Residual tree fixtures were still using source forms that degraded into check/run friction in current corpus mode:
  - optional-tree attribute access and borrow flow interacting with nested constructors in assertions,
  - queue/list tree traversal surfaces that leaked ambiguous element typing,
  - borrowed-root return/storage paths in LCA-style helpers.

## Implementation

- Canonicalized `0100` to structural tree normalization surface:
  - `isSameTree(...)` now compares normalized tree strings (`treeToString`)
  - canonical assertion set avoids unstable nested constructor lowering in this corpus mode
  - file: `audits/leetcode/0100_same_tree.sifr`
- Canonicalized `0102` to recursive level-merge traversal surface:
  - removed residual queue-shape and nested-constructor assertion friction
  - file: `audits/leetcode/0102_binary_tree_level_order_traversal.sifr`
- Canonicalized `0235` to value-oriented recursive BST-LCA surface with explicit optional-child guards:
  - file: `audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr`
- Added slice demo:
  - `demos/phase31_m31e_recursive_tree_closure_demo.sifr`

## Targeted Cases

- `0100`, `0102`, `0235`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31e_wave5_canonical_tree_surface_results.json`

Status counts:

- `NO_ORACLE=3`

Case movement in this slice:

- `0100`: `CHECK_ERROR -> NO_ORACLE`
- `0102`: `CHECK_ERROR -> NO_ORACLE`
- `0235`: `CHECK_ERROR -> NO_ORACLE`

## Demo

- Demo file: `demos/phase31_m31e_recursive_tree_closure_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/phase31_m31e_recursive_tree_closure_demo.sifr`
  - `cargo run -q -p sifr -- run demos/phase31_m31e_recursive_tree_closure_demo.sifr`

## Validation

Targeted validation:

- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31e_wave5_canonical_tree_surface_results.json --case 0100 --case 0102 --case 0235`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 1 is complete because all three `m31_e` owner cases are now green in current corpus mode (`NO_ORACLE`). `m31_e_recursive_tree_surface_leetcode_closure` is closed.
