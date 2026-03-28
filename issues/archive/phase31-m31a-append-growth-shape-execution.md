# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 9

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_9_append_growth_shape_propagation`

## Goal

Close the remaining optional-index blocker in `0238` by proving that local lists grown by one append per bounded iteration are safe for guarded indexed reads.

## Root Cause

- Even after alias and end-pointer narrowing, `result[i]` remained optional in `0238`.
- The compiler did not carry any shape fact for lists populated with:
  - `result = []`
  - `for i in range(n): result.append(...)`
- Therefore guarded index checks could not prove `i` is in-bounds for `result`.

## Implementation

- Added append-growth sequence-shape inference:
  - module: `crates/sifr_hir/src/lower/append_growth_shapes.rs`
  - records `SizedByAnchor` for single-append loop bodies over proven range anchors
- Integrated append-growth shape recording into for-loop lowering:
  - file: `crates/sifr_hir/src/lower/statements.rs`
- Added regression coverage:
  - `test_append_growth_shape_allows_index_under_alias_guard`
  - file: `crates/sifr_hir/src/lower/guarded_index.rs`
- Expanded the phase demo to include append-growth indexing under alias-backed guards:
  - `demos/phase31_len_alias_range_guard_demo.sifr`

## Targeted Cases

- `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave9_append_growth_results.json`

Status counts:

- `PASS=1`
- `CHECK_ERROR=6`

Confirmed new pass:

- `0238_product_of_array_except_self`

## Validation

Targeted validation:

- `cargo test -p sifr_hir append_growth_shape_allows_index_under_alias_guard -- --nocapture`
- `cargo run -q -p sifr -- check demos/phase31_len_alias_range_guard_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_len_alias_range_guard_demo.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0238_product_of_array_except_self.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0238_product_of_array_except_self.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave9_append_growth_results.json --case 0053 --case 0127 --case 0238 --case 0322 --case 0502 --case 0743 --case 0746`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 9 is complete because append-growth shape propagation removed the last `0238` optional-index failure and moved the case to `PASS`. Remaining `m31_a` failures are now narrower and outside this slice root cause.
