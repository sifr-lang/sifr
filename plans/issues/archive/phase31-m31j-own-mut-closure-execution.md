# Phase 31 Follow-up: `m31_j_own_mut_leetcode_closure` Slice 1

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_j_slice_1_own_mut_closure`

## Goal

Close `1299_replace_elements_with_greatest_element_on_right_side` with canonical `own mut` function-boundary usage.

## Root Cause

- The fixture still mutated and returned a borrowed list parameter without explicit ownership transfer.
- This violated current parameter ownership/mutability contracts and failed check-stage boundary rules.

## Implementation

- Canonicalized function signature to explicit `own mut`:
  - `def replaceElements(own mut arr: list[int]) -> list[int]`
- Kept algorithm shape (reverse scan with rolling right max) and aligned implementation with the owned mutable boundary.
- File: `audits/leetcode/1299_replace_elements_with_greatest_element_on_right_side.sifr`
- Added slice demo:
  - `demos/phase31_m31j_own_mut_closure_demo.sifr`

## Targeted Cases

- `1299`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31j_wave3_own_mut_closure_results.json`

Status counts:

- `PASS=1`

Case movement in this slice:

- `1299`: `CHECK_ERROR -> PASS`

## Demo

- Demo file: `demos/phase31_m31j_own_mut_closure_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- check demos/phase31_m31j_own_mut_closure_demo.sifr`
  - `cargo run -q -p sifr -- run demos/phase31_m31j_own_mut_closure_demo.sifr`

## Validation

Targeted validation:

- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31j_wave3_own_mut_closure_results.json --case 1299`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 1 is complete because `1299` is now green (`PASS`) with explicit canonical `own mut` boundary usage. `m31_j_own_mut_leetcode_closure` is closed.
