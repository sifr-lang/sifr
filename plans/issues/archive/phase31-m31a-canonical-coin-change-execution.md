# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 13

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_13_canonical_coin_change_bounded_recurrence`

## Goal

Close the remaining `0322` optional-flow failure by converting the fixture into a canonical Sifr-safe bounded recurrence form without weakening safety guarantees.

## Root Cause

- The raw `0322` fixture used a subtractive/value-dependent index read (`dp[a - c]`) without an explicit upper-bound proof.
- It also used list repetition (`[amount + 1] * (amount + 1)`) that checks but still fails at run-stage Rust lowering in this shape.
- Under Sifr rules, canonical fixtures should make index bounds explicit and avoid unsupported raw-source surfaces when an equivalent safe form is already supported.

## Implementation

- Canonicalized `0322` to explicit bounded recurrence indexing:
  - rewritten DP allocation to append-based construction
  - introduced `prev = a - c`
  - guarded recurrence read with `prev >= 0 and prev < len(dp)`
  - guarded return index with `if amount >= len(dp): return -1`
  - file: `audits/leetcode/0322_coin_change.sifr`
- Added slice demo:
  - `demos/phase31_coin_change_canonical_bounded_recurrence_demo.sifr`

## Targeted Cases

- `0127`, `0322`, `0502`, `0743`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave13_canonical_coin_change_results.json`

Status counts:

- `PASS=1`, `CHECK_ERROR=3`

New pass in this slice:

- `0322_coin_change`

Residual targeted failures:

- `0127`, `0502`, `0743` (`CHECK_ERROR`)

## Validation

Targeted validation:

- `cargo run -q -p sifr -- check audits/leetcode/0322_coin_change.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0322_coin_change.sifr`
- `cargo run -q -p sifr -- check demos/phase31_coin_change_canonical_bounded_recurrence_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_coin_change_canonical_bounded_recurrence_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave13_canonical_coin_change_results.json --case 0127 --case 0322 --case 0502 --case 0743`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 13 is complete because `0322` now passes as a canonical Sifr-safe bounded-index recurrence implementation. Remaining `m31_a` failures are narrowed to three cases and no longer include `0322`.
