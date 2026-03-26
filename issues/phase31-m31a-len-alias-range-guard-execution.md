# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 7

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_7_len_alias_range_guard_narrowing`

## Goal

Remove optional-flow leakage when loop bounds are expressed through local aliases of `len(sequence)` rather than direct `len(...)` calls.

This slice is scoped to:

- `n = len(seq)` then `for i in range(n): seq[i]`
- `n = len(seq)` then `for i in range(n - 1, -1, -1): seq[i]`

It does not attempt to solve:

- sized-local list growth proofs from append loops
- subtractive/value-dependent recurrence indexing (`a - c`)
- canonical source adaptation for mutability/ownership constraints

## Root Cause

- Range guard detection only recognized direct `len(seq)` in `range(...)`.
- Equivalent alias forms (`n = len(seq); range(n)`) did not produce `IndexVarInRange` guards.
- As a result, reads like `seq[i]` stayed `T | None` under bounds-safe loops.

## Implementation

- Added len-alias flow facts:
  - `crates/sifr_hir/src/lower/len_aliases.rs`
  - facts are set on `len(...)` assignments and propagated through simple alias assignments
- Wired assignment lowering to maintain len-alias facts:
  - `crates/sifr_hir/src/lower/statements.rs`
- Extended range-shape detection to resolve aliases as len anchors:
  - `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
  - applies to both forward and reverse range forms
- Added guarded-index regressions for forward and reverse alias-backed ranges:
  - `crates/sifr_hir/src/lower/guarded_index.rs`

## Regression Coverage

- `test_range_len_alias_list_index_reveals_element_type`
- `test_reverse_range_len_alias_list_index_reveals_element_type`

## Milestone Demo

- `demos/phase31_len_alias_range_guard_demo.sifr`

Demo validation:

- `cargo run -q -p sifr -- check demos/phase31_len_alias_range_guard_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_len_alias_range_guard_demo.sifr`

## Targeted Cases

- `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave7_len_alias_results.json`

Status counts:

- `CHECK_ERROR=7` (count unchanged)

Meaningful movement:

- `0238_product_of_array_except_self` reduced optional arithmetic failures from 3 to 2:
  - `nums[i]` under `for i in range(n)` now narrows via `n = len(nums)` alias flow.
  - remaining failures are isolated to sized-local `result[i]` flow, which is outside this slice scope.

## Validation

Targeted validation:

- `cargo test -p sifr_hir len_alias_list_index -- --nocapture`
- `cargo run -q -p sifr -- check demos/phase31_len_alias_range_guard_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_len_alias_range_guard_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave7_len_alias_results.json --case 0053 --case 0127 --case 0238 --case 0322 --case 0502 --case 0743 --case 0746`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 7 is complete because len-alias range bounds now produce the same index narrowing guarantees as direct `len(...)` bounds. Remaining optional failures in the watched set are narrower follow-ons and not this slice root cause.
