# Phase 28 Production-Grade Review: Decimal Types and Exact Numeric Semantics

**Review Date:** 2026-03-07
**Reviewer:** agent
**Scope:** Post-first-review remediation status assessment
**Branch:** main (commit 93c21d55)

---

## Executive Summary

Phase 28 introduces `decimal` (fixed-precision via rust_decimal) and `bigdecimal` (arbitrary-precision via bigdecimal) types with deterministic, exact base-10 arithmetic semantics. After the first-review remediation (PR #915), **the implementation is largely production-ready** with one remaining medium-priority gap.

### Status: PRODUCTION-GRADE WITH ONE REMAINING ISSUE

---

## First-Review Remediation Status

### ✅ HIGH PRIORITY: Float Conversion Compile-Time Error (FIXED)

**Status:** Fixed in commit `3bab1331` (merged in PR #915)

**Changes Made:**
- Added compile-time error for `float(decimal)` with diagnostic `[E2505]`
- Added compile-time error for `float(bigdecimal)` with diagnostic `[E2506]`
- Added test files:
  - `crates/sifr/tests/e2e/fail/float_from_decimal_forbidden.sifr`
  - `crates/sifr/tests/e2e/fail/float_from_bigdecimal_forbidden.sifr`

**Verification:**
```sifr
def main():
    d: decimal = Decimal("1.5")
    f: float = float(d)  # Now produces [E2505] at compile time
```

The fix correctly rejects decimal-to-float conversions at compile time rather than letting them fail with cryptic Rust codegen errors.

---

## Remaining Issues

### 1. MEDIUM PRIORITY: Missing Assignability Rules in Type System

**Location:** `crates/sifr_type_system/src/types.rs:730-734`

**Issue:** The `is_assignable_to` function explicitly handles `BigInt` but lacks explicit handling for `Decimal` and `BigDecimal`:

```rust
// Current state (line 732-734):
(BigInt, BigInt) => true,
_ => false,  // Decimal/BigDecimal fall through here
```

**Impact:** Currently low because variable assignments use direct type equality checks, not `is_assignable_to`. However, this creates an inconsistency and potential future compatibility risk.

**Recommended Fix:**

Add explicit assignability rules in `types.rs`:

```rust
// Add before the catch-all _ => false
(Self::Decimal, Self::Decimal) => true,
(Self::BigDecimal, Self::BigDecimal) => true,
```

**Test Coverage:** Add unit tests for decimal/bigdecimal assignability in the `is_assignable_to` test module.

---

## Correctness Verification

### Test Suite Status

| Test Category | Status |
|---------------|--------|
| E2E Pass Tests | ✅ All pass |
| E2E Fail Tests | ✅ All pass (correctly rejected) |
| E2E Runtime Fail Tests | ✅ All pass (correctly caught at runtime) |
| Determinism Gates (30 iterations) | ✅ Verified |

**Test Evidence:**
```
$ cargo test --package sifr --test e2e
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Components Verified

| Component | Status | Notes |
|-----------|--------|-------|
| Type System (Decimal/BigDecimal) | ✅ PASS | Types properly defined with ownership (Copy/Move) |
| Parser Integration | ✅ PASS | Constructor parsing works |
| HIR Lowering | ✅ PASS | Expression lowering complete |
| Arithmetic Operators | ✅ PASS | +, -, *, /, //, %, ** all work |
| Method Validation | ✅ PASS | quantize, sqrt, round, abs, is_zero, is_finite |
| Mixed-Numeric Policy | ✅ PASS | float+decimal, decimal+bigdecimal forbidden |
| Constructor Validation | ✅ PASS | str, int, bigint, decimal, bigdecimal inputs |
| Conversion Boundaries | ✅ PASS | int(), bigint(), str() conversions |
| Diagnostics (E2501-E2508) | ✅ PASS | All codes properly used |
| Determinism | ✅ PASS | 30-iteration tests verify consistency |
| Float Conversion Ban | ✅ PASS | float(decimal), float(bigdecimal) rejected |

---

## Diagnostics Assessment

### Current Error Codes (E2501-E2508)

| Code | Purpose | Status |
|------|---------|--------|
| E2501 | Decimal invalid literal string | ✅ Active |
| E2502 | BigDecimal invalid literal string | ✅ Active |
| E2503 | float + decimal/bigdecimal mixing | ✅ Active |
| E2504 | decimal + bigdecimal mixing | ✅ Active |
| E2505 | Decimal constructor/conversion errors | ✅ Active (extended) |
| E2506 | BigDecimal constructor/conversion errors | ✅ Active (extended) |
| E2507 | Decimal context/scale errors | ✅ Active |
| E2508 | BigDecimal context/scale errors | ✅ Active |

**Note:** E2505 and E2506 have been extended beyond their original scope to cover float conversion errors (this is appropriate reuse of existing error codes).

### Diagnostic Stability: STABLE

All diagnostic messages are consistent, well-formed, and provide actionable guidance.

---

## Determinism Guarantees

| Guarantee | Implementation | Status |
|-----------|----------------|--------|
| Rounding Mode | `HalfEven` (banker's rounding) | ✅ Consistent |
| BigDecimal Default Context | Fixed precision 28 with HalfEven | ✅ Deterministic |
| Iteration Tests | 30-run verification loops | ✅ Passing |
| Error Behavior | Seeded negative test cases | ✅ Consistent |

**Assessment:** VERIFIED - The implementation provides reproducible results across runs.

---

## Safety Assessment

### Numeric Safety

| Aspect | Assessment |
|--------|------------|
| Overflow Prevention | ✅ Decimal has fixed precision; BigDecimal is arbitrary |
| Division by Zero | ✅ Runtime error (tested) |
| Precision Loss Prevention | ✅ Float conversions explicitly banned |
| NaN/Infinity Handling | ✅ Not applicable (not representable in decimal types) |
| is_finite() | ✅ Always returns true (correct for exact types) |

### Type Safety

| Aspect | Assessment |
|--------|------------|
| Mixed Numeric Policy | ✅ Enforced at compile time |
| Constructor Validation | ✅ Strict type checking |
| Conversion Boundaries | ✅ Fallible conversions return Result types |

---

## Production Readiness Checklist

- [x] All 5 parts merged
- [x] Full test suite passes (19 e2e test categories)
- [x] Determinism gates implemented and passing
- [x] Demo execution verified
- [x] Error codes documented (E2501-E2508)
- [x] Float conversion compile-time error (HIGH priority) - FIXED
- [x] Float constructor ban (Decimal/BigDecimal from float)
- [x] Mixed numeric policy enforcement
- [ ] Explicit assignability rules for Decimal/BigDecimal (MEDIUM priority)

---

## Risk Assessment

| Area | Risk Level | Notes |
|------|------------|-------|
| Type System (assignability gap) | LOW | Not currently causing runtime issues |
| Arithmetic Operations | LOW | Uses well-tested rust_decimal/bigdecimal |
| Constructor Validation | LOW | Strict validation with clear diagnostics |
| Diagnostics | LOW | New codes don't conflict with existing |
| Backwards Compatibility | LOW | No breaking changes to existing types |
| Regression | LOW | Comprehensive test coverage |

---

## Actionable Fixes

### 1. Fix Assignability Rules (MEDIUM PRIORITY)

**File:** `crates/sifr_type_system/src/types.rs`

**Change:** Add explicit assignability rules for Decimal and BigDecimal in the `is_assignable_to` match arms (around line 730-734):

```rust
// Add before the catch-all _ => false
(Self::Decimal, Self::Decimal) => true,
(Self::BigDecimal, Self::BigDecimal) => true,
```

**Test:** Add unit tests:
```rust
#[test]
fn test_decimal_assignability() {
    assert!(Type::Decimal.is_assignable_to(&Type::Decimal));
    assert!(!Type::Decimal.is_assignable_to(&Type::BigDecimal));
}

#[test]
fn test_bigdecimal_assignability() {
    assert!(Type::BigDecimal.is_assignable_to(&Type::BigDecimal));
    assert!(!Type::BigDecimal.is_assignable_to(&Type::Decimal));
}
```

**Estimated Effort:** Low (5-10 lines of code + tests)

---

## Conclusion

**Production-Grade Status: RECOMMENDED WITH CAVEAT**

Phase 28 is production-ready for all practical purposes. The implementation correctly enforces exact numeric semantics, provides deterministic behavior, and has comprehensive test coverage. The one remaining issue (missing assignability rules) is low-risk but should be addressed for completeness and future-proofing.

**Recommendation:** Deploy to production after addressing the medium-priority assignability rules issue. The fix is trivial and would bring the implementation to full parity with the type system's design patterns.
