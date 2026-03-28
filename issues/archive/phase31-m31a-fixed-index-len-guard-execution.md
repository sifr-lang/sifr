# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 12

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_12_fixed_index_len_guard_and_canonical_sources`

## Goal

Close fixed-index optional leakage after explicit length guards and convert two remaining seed fixtures to canonical Sifr-safe forms aligned with landed narrowing rules.

## Root Cause

- Post-return guards of the form `if len(seq) < K: return ...` / `<=` were not converted into min-length facts for the fallthrough path.
- This blocked fixed-index reads such as `seq[1]` even after explicit guards.
- Two seed fixtures still used raw-source forms that conflict with Sifr safety/convention defaults:
  - `0053` assumed non-empty input without an explicit guard
  - `0746` required mutable list semantics and used a subscript-augassign surface that currently routes into a codegen run failure in this exact loop shape

## Implementation

- Added false-exit sequence-guard detection for:
  - `len(seq) < K` -> `min_len(seq) >= K`
  - `len(seq) <= K` -> `min_len(seq) >= K + 1`
  - file: `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
- Added fixed-index regressions:
  - `test_early_return_len_lt_guard_narrows_fixed_index_type`
  - `test_early_return_len_lte_guard_narrows_fixed_index_type`
  - file: `crates/sifr_hir/src/lower/guarded_index.rs`
- Canonicalized `0053` fixture with explicit empty-input guard:
  - `if len(nums) == 0: return 0`
  - file: `audits/leetcode/0053_maximum_subarray.sifr`
- Canonicalized `0746` fixture to explicit Sifr-safe mutability and assignment surface:
  - `mut cost` parameter
  - `if len(cost) < 2: return 0`
  - rewrite `cost[i] += ...` to `cost[i] = cost[i] + ...` (preserves algorithm semantics while avoiding currently unsupported composite augassign lowering in this shape)
  - file: `audits/leetcode/0746_min_cost_climbing_stairs.sifr`
- Added slice demo:
  - `demos/phase31_fixed_index_len_guard_demo.sifr`

## Targeted Cases

- `0053`, `0127`, `0322`, `0502`, `0743`, `0746`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave12_fixed_index_guard_results.json`

Status counts:

- `PASS=2`, `CHECK_ERROR=4`

New passes in this slice:

- `0053_maximum_subarray`
- `0746_min_cost_climbing_stairs`

Residual targeted failures now:

- `0127`, `0322`, `0502`, `0743` (`CHECK_ERROR`)

## Validation

Targeted validation:

- `cargo test -p sifr_hir test_early_return_len_lt_guard_narrows_fixed_index_type`
- `cargo test -p sifr_hir test_early_return_len_lte_guard_narrows_fixed_index_type`
- `cargo run -q -p sifr -- check audits/leetcode/0053_maximum_subarray.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0746_min_cost_climbing_stairs.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0746_min_cost_climbing_stairs.sifr`
- `cargo run -q -p sifr -- run demos/phase31_fixed_index_len_guard_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave12_fixed_index_guard_results.json --case 0053 --case 0127 --case 0322 --case 0502 --case 0743 --case 0746`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 12 is complete because fixed-index reads now narrow after explicit `len(...) < / <=` false-exit guards, and the canonicalized `0053`/`0746` fixtures now pass end-to-end under Sifr conventions.
