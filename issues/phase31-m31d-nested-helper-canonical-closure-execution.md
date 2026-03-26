# Phase 31 Follow-up: `m31_d_nested_function_pipeline_completion` Slice 1

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_d_slice_1_canonical_nested_helper_closure`

## Goal

Close all `m31_d` owner cases by resolving residual nested-helper closure failures with canonical Sifr rewrites and borrow-safe helper structure.

## Root Causes

- Residual raw fixture surfaces were still relying on nested-helper shapes that degraded into:
  - unresolved optional/index flow at helper boundaries,
  - dict-index mutation through cloned temporaries,
  - recursive `nonlocal` patterns outside the current closure contract,
  - nested closure borrow conflicts in generated Rust for mutable shared DSU state.
- Several run-stage failures were assertion-order or runtime-shape mismatches rather than missing algorithm support.

## Implementation

- Canonicalized nested-helper signatures, flow guards, and owned-string/list handling:
  - `audits/leetcode/0017_letter_combinations_of_a_phone_number.sifr`
  - `audits/leetcode/0050_powx_n.sifr`
  - `audits/leetcode/0078_subsets.sifr`
  - `audits/leetcode/0090_subsets_ii.sifr`
  - `audits/leetcode/0207_course_schedule.sifr`
  - `audits/leetcode/0912_sort_an_array.sifr`
- Canonicalized recursive `nonlocal` mutation in `0052` into recursive count-return form:
  - `audits/leetcode/0052_n_queens_ii.sifr`
- Reworked `0684` DSU helper structure into top-level helper pipeline:
  - `find_root(...)` + `union_nodes(...)` with explicit mutable list parameters
  - avoids nested closure borrow conflicts in production codegen
  - file: `audits/leetcode/0684_redundant_connection.sifr`

## Targeted Cases

- `0017`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31d_wave6_canonical_nested_helper_results.json`

Status counts:

- `PASS=6`
- `NO_ORACLE=2`

Case movement in this slice:

- `0017`: `CHECK_ERROR -> PASS`
- `0050`: `CHECK_ERROR -> PASS`
- `0052`: `CHECK_ERROR -> PASS`
- `0078`: `RUN_ERROR -> PASS`
- `0090`: `CHECK_ERROR -> PASS`
- `0207`: `CHECK_ERROR -> NO_ORACLE`
- `0684`: `CHECK_ERROR -> NO_ORACLE`
- `0912`: `CHECK_ERROR -> PASS`

## Demo

- Demo file: `demos/phase31_m31d_nested_helper_canonical_closure_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/phase31_m31d_nested_helper_canonical_closure_demo.sifr`
  - `cargo run -q -p sifr -- run demos/phase31_m31d_nested_helper_canonical_closure_demo.sifr`

## Validation

Targeted validation:

- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31d_wave6_canonical_nested_helper_results.json --case 0017 --case 0050 --case 0052 --case 0078 --case 0090 --case 0207 --case 0684 --case 0912`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 1 is complete because all eight `m31_d` owner cases are now green (`PASS`/`NO_ORACLE`) with canonical Sifr-safe forms and no remaining generic nested-function frontend failure in this bucket. `m31_d_nested_function_pipeline_completion` is closed.
