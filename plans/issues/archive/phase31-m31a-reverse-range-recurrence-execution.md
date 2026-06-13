# Phase 31 Follow-up: `m31_a_optional_narrowing_core` Slice 5

Status: complete
Started: 2026-03-13
Completed: 2026-03-13
Current slice: `m31_a_slice_5_reverse_range_recurrence_narrowing`

## Goal

Remove the next sound optional-flow root cause inside `type_system.optional_narrowing_and_union_ops` by proving affine `+1` recurrence reads safe when they are driven by reverse `range(...)` loops over sized local sequence constructions.

This slice is scoped to:

- reverse loops of the form `for i in range(len(seq) - 1, -1, -1)` and related `len(seq) - k` starts
- local list and matrix constructions whose lengths are derived from `range(len(anchor) + extra)`
- affine `index + 1` reads into those sized constructions
- nested matrix recurrence reads such as `dp[i + 1][j + 1]`
- downstream production codegen needed to execute those recurrence shapes without statement-emission or ownership-ordering failures

It does not attempt to solve:

- unguarded parameter indexing such as `nums[0]` on plain `list[int]` inputs
- subtractive/value-dependent offsets such as `dp[a - c]`
- arbitrary list repetition sizing from unconstrained integers

## Root Cause

Two distinct compiler gaps were coupled in the remaining recurrence failures:

1. HIR lowering could not prove `index + 1` safe under reverse `range(...)` loops over sized local constructions.
2. Even after the HIR proof, production codegen still mishandled the same programs by:
   - rejecting comprehension-backed local initializers in structured statement lowering,
   - rejecting dynamic subscript writes inside structured loop bodies,
   - evaluating recurrence RHS expressions after taking mutable list/matrix borrows,
   - lowering negative-step `range(...)` iterators with Rust `step_by(negative as usize)`, which produced empty loops.

This slice fixes those shared causes rather than individual LeetCode programs.

## Implementation

- Added sequence-shape fact tracking for local constructions sized from anchor lengths.
- Extended sequence-guard detection so reverse ranges such as `range(len(seq) - 1, -1, -1)` record an offset-safe guard window.
- Extended guarded-index narrowing to honor:
  - affine `+ literal` offsets,
  - local list constructions sized by `len(anchor) + extra`,
  - nested matrix recurrence reads such as `dp[i + 1][j + 1]`.
- Added structured IR lowering for list/dict/set comprehensions used as `let` initializers.
- Added structured IR lowering for dynamic `SubscriptAssign` and `NestedSubscriptAssign`.
- Fixed structured assignment ordering so RHS recurrence reads are materialized before taking mutable element borrows.
- Added a dedicated negative-step range iterator lowering path that emits reverse Rust iterators instead of invalid `step_by(-(1 as i64) as usize)` loops.

## Targeted Cases

Primary slice candidate:

- `1143` `longest_common_subsequence`

Reclassification probes:

- `0746` `min_cost_climbing_stairs`
- `0053` `maximum_subarray`
- `0322` `coin_change`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave5_results.json`

Targeted watched-set outcome:

- `PASS=1`
- `CHECK_ERROR=3`
- `RUN_ERROR=0`

Confirmed new pass:

- `1143` `longest_common_subsequence`

Confirmed reclassification boundaries:

- `0053` remains blocked by unsafe unguarded parameter head access (`nums[0]`)
- `0746` remains blocked by unsafe unguarded parameter head access on `cost`
- `0322` remains blocked by subtractive/value-dependent recurrence indexing and resulting optional arithmetic

Full warmed corpus rerun artifact:

- `verification/leetcode/phase31_current_full_results_after_m31a_wave5_rerun.json`

Full corpus state after slice 5:

- `PASS=15`
- `CHECK_ERROR=35`
- `RUN_ERROR=0`

Delta from the prior stable full snapshot:

- `1143` moved from `CHECK_ERROR` to `PASS`

## Validation

Targeted execution validation:

- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_reverse_range_recurrence_narrowing.sifr`
- `target/debug/sifr run demos/phase31_reverse_range_recurrence_demo.sifr`
- `target/debug/sifr run audits/leetcode/1143_longest_common_subsequence.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave5_results.json --case 0053 --case 0322 --case 0746 --case 1143`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_current_full_results_after_m31a_wave5_rerun.json`

Targeted compiler coverage added:

- `crates/sifr/tests/e2e/pass/phase31_reverse_range_recurrence_narrowing.sifr`
- `demos/phase31_reverse_range_recurrence_demo.sifr`
- `crates/sifr_codegen/src/lib_codegen_tests.rs`
- `crates/sifr_hir/src/lower/guarded_index.rs`

Pending pre-PR gate:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

This slice is complete because the reverse-range recurrence root cause is removed and the remaining watched failures are now cleanly outside slice scope. The unresolved cases are narrower non-goals that belong to later `m31_a` work:

- unsafe plain-parameter head indexing
- subtractive/value-dependent recurrence offsets
