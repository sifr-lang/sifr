# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 8

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_8_end_pointer_len_alias_while_guard`

## Goal

Remove optional-flow leakage for reverse while loops that use an end-pointer initialized from a len alias:

- `n = len(seq)`
- `i = n - 1`
- `while i >= 0: seq[i]`

## Root Cause

- End-pointer facts only recognized `len(seq) - 1` directly and did not resolve alias forms like `n - 1` when `n = len(seq)`.
- Guard detection did not treat `i >= 0` as an in-range proof for known end-pointers.
- As a result, `seq[i]` in such loops remained `T | None`.

## Implementation

- Extended sequence-pointer fact derivation:
  - alias-backed `len(...) - 1` now resolves through len-alias facts
  - file: `crates/sifr_hir/src/lower/sequence_pointers.rs`
- Extended true-guard detection:
  - `i >= 0` now emits an index-range guard when `i` is a known end-pointer
  - file: `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
- Added regression coverage:
  - `test_while_end_pointer_len_alias_reveals_element_type`
  - file: `crates/sifr_hir/src/lower/guarded_index.rs`
- Updated demo:
  - `demos/phase31_len_alias_range_guard_demo.sifr` now includes while-based reverse traversal.

## Targeted Cases

- `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave8_end_pointer_alias_results.json`

Status counts:

- `CHECK_ERROR=7` (count unchanged)

Meaningful movement:

- `0238_product_of_array_except_self` reduced optional arithmetic errors from 2 to 1:
  - `nums[i]` under `i = n - 1` + `while i >= 0` now narrows correctly via alias-backed end-pointer flow.
  - remaining `0238` failure is isolated to sized-local `result[i]` flow.

## Validation

Targeted validation:

- `cargo test -p sifr_hir while_end_pointer_len_alias_reveals_element_type -- --nocapture`
- `cargo run -q -p sifr -- check demos/phase31_len_alias_range_guard_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_len_alias_range_guard_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave8_end_pointer_alias_results.json --case 0053 --case 0127 --case 0238 --case 0322 --case 0502 --case 0743 --case 0746`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 8 is complete because alias-backed end-pointer while-loop indexing now narrows as definite when flow is proven safe. Remaining watched-case failures are narrower follow-ons outside this slice root cause.
