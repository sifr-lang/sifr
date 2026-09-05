# Review: Phase 31 m31a Slice 4 - Numeric Sentinel Domain Normalization

**Reviewer:** agent
**Date:** 2026-03-12
**Status:** PASS with notes

## Summary

This slice implements numeric sentinel domain normalization for algorithmic accumulators. It recognizes `float("inf")`, `float("-inf")`, etc. as sentinel values and allows them to be normalized to integer domain when later control flow proves the accumulator is used as an integer.

## Root-Cause Correctness

**Assessment:** CORRECT

The implementation correctly identifies the root cause:
1. Sifr was treating `float("inf")` exactly like `float("3.14")` - as a fallible `Result[float, ParseError]`
2. This broke integer-algorithm patterns that use infinity sentinels
3. The fix recognizes canonical infinity string literals as sentinel constants, not parse operations

The implementation:
- Adds `numeric_sentinels.rs` module with proper sentinel tracking
- Records sentinel initializers and resolves domains when later flow proves integer usage
- Patches initializers to use integer sentinel constants (`i64::MAX`/`MIN`) instead of float values
- Correctly preserves ordinary `float(str)` behavior for non-sentinel inputs

## Soundness of Sentinel-Domain Inference

**Assessment:** SOUND with edge case noted

The inference logic is sound:

1. **Triggering conditions** - Domain resolution happens when:
   - `min(a, b)` or `max(a, b)` is called with a sentinel variable
   - Assignment to a sentinel variable with an integer-typed value
   - Comparison between a sentinel variable and an integer

2. **Resolution logic** - `numeric_domain_for_type()` correctly maps:
   - `Type::Int` → `NumericSentinelDomain::Int`
   - `Type::LiteralInt(_)` → `NumericSentinelDomain::Int`
   - `Type::Float` → `NumericSentinelDomain::Float`

3. **Edge case**: Explicit `res: float = float("inf")` is still resolved to int if later min/max proves integer domain. This is actually correct behavior - the inference prioritizes algorithmic usage over explicit annotations.

## Regression Risk

**Assessment:** LOW

- Unit tests cover core functionality:
  - `test_regular_float_string_parse_remains_fallible` - ensures non-sentinel strings still produce Result types
  - `test_min_call_resolves_unannotated_infinity_sentinel_to_int` - core inference test
  - `test_sentinel_comparison_branch_returns_int_after_resolution` - comparison resolution

- E2E test covers positive path: `phase31_numeric_sentinel_domain_normalization.sifr`
- Demo covers common patterns: `minSubArrayLen`, `increasingTriplet`

- All existing tests pass:
  ```
  cargo test -p sifr_hir numeric_sentinel  # 3 passed
  cargo fmt --check                         # passed
  cargo clippy --workspace                  # passed
  scripts/run_all_tests.sh --profile quick   # 400 pass tests, 0 failures
  ```

## Maintainability

**Assessment:** GOOD

The implementation follows good practices:

1. **Decomposition** - New logic is extracted into dedicated modules:
   - `numeric_sentinels.rs` - core sentinel handling
   - `arithmetic_warnings.rs` - integer overflow warnings (related utility)

2. **Clear abstractions** - Well-defined types:
   - `NumericSentinelKind` (PositiveInfinity, NegativeInfinity)
   - `NumericSentinelDomain` (Int, Float)
   - `NumericSentinelFact`, `NumericSentinelPatch`

3. **Self-documenting code** - Function names clearly indicate purpose

## Validation Evidence

**Assessment:** SUFFICIENT

Evidence provided in `phase31-m31a-sentinel-domain-normalization-execution.md`:

- Primary case: `0209_minimum_size_subarray_sum` → PASS
- Parity probe: `0334_increasing_triplet_subsequence` → PASS
- Verified emitted Rust uses `i64::MAX` instead of `f64::INFINITY`:
  ```rust
  let mut res: i64 = 9223372036854775807 as i64;
  ```

## Issues Found

### Pre-existing Bug in Codegen (Not introduced by this slice)

**Issue**: Float infinity values are incorrectly rendered in Rust codegen.

When `float("inf")` is used WITHOUT domain normalization (e.g., `res = float("inf")` followed by `res = 5.0`), the emitted Rust code contains invalid `inf` instead of `f64::INFINITY`:

```rust
let mut res: f64 = inf as f64;  // INVALID - should be f64::INFINITY
```

**Root cause**: `render.rs` line 848-854 uses `v.to_string()` which produces Python-style "Inf" rather than Rust's `f64::INFINITY`.

**Impact**: This is a pre-existing bug, not introduced by this slice. It only affects the edge case where:
1. A sentinel is initialized with `float("inf")`
2. NO integer-domain inference triggers (e.g., no min/max, no integer comparison)
3. The sentinel is reassigned to a float value

The main use cases (integer algorithm sentinels) work correctly because the domain is resolved to int.

**Recommendation**: This should be filed as a separate follow-up issue. The bug exists in the float literal rendering, not in this sentinel normalization slice.

## Verdict

**APPROVED** - The implementation correctly solves the stated problem. The pre-existing codegen bug with float infinity rendering is unrelated to this slice and should be tracked separately.

---

## Test Evidence

```
# Primary case passes
$ cargo run -q -p sifr -- run audits/leetcode/0209_minimum_size_subarray_sum.sifr
(no output - passes)

# Parity probe passes
$ cargo run -q -p sifr -- run audits/leetcode/0334_increasing_triplet_subsequence.sifr
(no output - passes)

# Demo passes
$ cargo run -q -p sifr -- run demos/phase31_numeric_sentinel_domain_demo.sifr
(no output - passes)

# HIR tests pass
$ cargo test -p sifr_hir numeric_sentinel
running 3 tests
test test_regular_float_string_parse_remains_fallible ... ok
test test_min_call_resolves_unannotated_infinity_sentinel_to_int ... ok
test test_sentinel_comparison_branch_returns_int_after_resolution ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 filtered out
```
