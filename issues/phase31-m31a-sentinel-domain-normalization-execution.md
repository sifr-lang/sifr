# Phase 31 Follow-up: `m31_a_optional_narrowing_core` Slice 4

Status: complete
Started: 2026-03-12
Current slice: `m31_a_slice_4_sentinel_domain_normalization`

## Goal

Remove the next optional/branch-flow root cause inside `type_system.optional_narrowing_and_union_ops` by normalizing canonical numeric infinity sentinels used in integer algorithms.

This slice is scoped to:

- `float("inf")`, `float("-inf")`, `float("infinity")`, and `float("-infinity")` string-literal sentinel constructors
- unannotated local accumulator variables initialized from those sentinels
- integer-domain algorithm patterns where the accumulator is later updated through `min(...)`, `max(...)`, direct integer assignment, or integer comparisons
- comparisons against the same sentinel in branch/return expressions

It does not attempt to solve:

- general `float(str)` parsing behavior
- arbitrary mixed int/float comparison rules
- `nan`-specific flow semantics
- non-local or cross-function sentinel inference

## Targeted Cases

Primary slice candidate:

- `0209` `minimum_size_subarray_sum`

Secondary parity probes that should move if the same root cause is fixed:

- `0334` `increasing_triplet_subsequence`
- direct lowered-shape probe for ordinary `float("3.14")` parsing

## Root-cause Hypothesis

- Sifr currently treats `float("inf")` exactly like `float("3.14")`: a fallible `Result[float, ParseError]`.
- Algorithmic Python code often uses `float("inf")` and `float("-inf")` as numeric sentinels even when the actual working domain is integer.
- That breaks in three ways:
  - the initializer gets the wrong type (`Result[...]` instead of a constant sentinel),
  - later `min(...)` / `max(...)` / comparison flow cannot collapse the accumulator onto the integer domain,
  - final `x if x != float("inf") else 0` style branches keep the sentinel poisoning the result type.

## Planned Root-cause Fix

- Recognize canonical infinity string literals in `float(...)` as sentinel constants, not parse operations.
- Track local pending sentinel accumulators during lowering.
- Resolve integer-domain sentinel variables when later numeric flow proves the accumulator is being used as an integer algorithmic sentinel.
- Patch the original initializer/codegen path so an integer-domain sentinel variable lowers to an integer sentinel constant instead of a float parse/value.

## Implemented Root-cause Fixes

- Split infinity sentinel handling into a dedicated lowering path in `crates/sifr_hir/src/lower/numeric_sentinels.rs`.
- Kept ordinary `float(str)` behavior intact:
  - non-sentinel string inputs still lower as `Result[float, ParseError]`
  - only canonical infinity spellings (`inf`, `-inf`, `infinity`, `-infinity`, with optional `+`) bypass string parsing
- Added local sentinel fact tracking on unannotated and annotated assignments so later numeric flow can refine the variable domain instead of freezing it as `float`.
- Resolved sentinel accumulator domains when later `min(...)`, `max(...)`, assignment, or comparison flow proves an integer-only algorithmic accumulator.
- Patched already-lowered initializer statements once the domain resolves so emitted Rust uses an integer sentinel constant (`i64::MAX` / `i64::MIN`) instead of a float value or parse call.
- Extracted the new min/max sentinel normalization and integer-overflow warning logic into dedicated lowering modules so `expressions.rs` stays under the HIR maintainability guardrail.
- Reviewer follow-up hardening fixed float-domain special literal rendering in codegen so unresolved float sentinels now emit valid Rust constants (`f64::INFINITY`, `f64::NEG_INFINITY`, `f64::NAN`) instead of invalid bare identifiers.

## Regression Coverage

- HIR/type-check tests in `crates/sifr_hir/src/lower/numeric_sentinels.rs`:
  - `test_regular_float_string_parse_remains_fallible`
  - `test_min_call_resolves_unannotated_infinity_sentinel_to_int`
  - `test_sentinel_comparison_branch_returns_int_after_resolution`
- Positive E2E fixture:
  - `crates/sifr/tests/e2e/pass/phase31_numeric_sentinel_domain_normalization.sifr`
- Demo:
  - `demos/phase31_numeric_sentinel_domain_demo.sifr`

## Acceptance Criteria

- `0209_minimum_size_subarray_sum` moves to `PASS`.
- The fix is rooted in compiler sentinel-domain normalization, not a case-specific special path.
- Direct `float(str)` parsing behavior for ordinary strings remains unchanged and fallible.
- Added regression coverage proves both:
  - infinity constructors stay fallible for normal parse inputs,
  - integer sentinel flows normalize correctly for algorithmic accumulators.

## Validation Evidence

- `cargo test -p sifr_hir numeric_sentinel`
- `cargo test -p sifr_codegen renders_special_float_literals_with_rust_constants`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_numeric_sentinel_domain_normalization.sifr`
- `target/debug/sifr run demos/phase31_numeric_sentinel_domain_demo.sifr`
- `target/debug/sifr run audits/leetcode/0334_increasing_triplet_subsequence.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave4_results.json --case 0209`
- `target/debug/sifr emit audits/leetcode/0209_minimum_size_subarray_sum.sifr`
- `cargo run -q -p sifr -- emit /tmp/phase31_float_inf_codegen_probe.sifr`
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Current Measured Outcome

Primary slice artifact:

- `verification/leetcode/phase31_m31a_wave4_results.json`

Observed movement across the targeted slice:

- `PASS=1`, `CHECK_ERROR=0`, `RUN_ERROR=0`
- Newly passing seeded case:
  - `0209` `minimum_size_subarray_sum`
- Confirmed parity probe:
  - `0334` `increasing_triplet_subsequence` now checks and runs with integer-domain sentinel lowering
- Confirmed codegen outcome:
  - emitted Rust for `0209` now uses `let mut res: i64 = 9223372036854775807 as i64;` and compares against the same integer sentinel rather than `f64::INFINITY`
- Confirmed reviewer-driven hardening:
  - unresolved float sentinels now emit valid Rust constants such as `let mut res: f64 = f64::INFINITY as f64;`
- Explicit non-goal confirmation:
  - this slice does not clear unrelated `Any`/optional-index failures such as `2017_grid_game`
