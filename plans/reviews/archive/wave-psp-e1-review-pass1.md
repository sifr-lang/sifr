# wave_psp_e1 Review Pass 1

**Wave**: `wave_psp_e1` — Strong-But-Incomplete Core Modules
**Modules**: datetime, re, math, statistics, hashlib
**Status**: Pending implementation
**Review Date**: 2026-03-16

---

## Executive Summary

This review analyzes the current implementation state of wave_psp_e1 modules in the Sifr codebase. The wave covers five Python standard library modules: `datetime`, `re`, `math`, `statistics`, and `hashlib`. Implementation files exist for all modules, along with CPython-derived test coverage. However, several actionable implementation issues and coverage gaps were identified that require remediation before the wave can be considered complete.

**Overall Assessment**: Issues found that require code changes before implementation can proceed.

---

## Module-by-Module Analysis

### 1. datetime.sifr

**Implementation Location**: `lib/sifr/datetime.sifr`

**Coverage**:
- Classes: `datetime`, `date`, `time`, `timedelta`, `timezone`
- Functions: `now()`, `today()`, `format_datetime()`, `from_timestamp()`
- Helper functions: `_is_leap_year()`, `_days_in_year()`, `_days_in_month()`

**Intrinsics Used**: `_sifr.datetime` (datetime_now, datetime_now_struct, datetime_format, datetime_from_timestamp)

**Test Coverage**:
- `crates/sifr/tests/e2e/pass/cpython_datetime.sifr` (28 assertions for timedelta)
- `crates/sifr/tests/e2e/pass/cpython_datetime_subset.sifr`

**Issues Found**:

| Severity | Issue | Location | CPython Behavior |
|----------|-------|----------|------------------|
| **HIGH** | `timedelta.total_seconds()` returns `int` | Line 30-31 | Returns `float` |
| MEDIUM | Missing `timedelta` microseconds parameter | Lines 22-28 | Supports microseconds |
| MEDIUM | Missing datetime arithmetic (e.g., `datetime + timedelta`) | N/A | Supported |
| MEDIUM | Missing comparison operators (`__lt__`, `__gt__`, etc.) | N/A | Supported |
| LOW | Missing `strftime()` / `strptime()` methods | N/A | Supported |

**Root Cause**: The `timedelta.total_seconds()` implementation returns `int` instead of `float`. This is a type signature mismatch with CPython.

```sifr
# Current (incorrect)
def total_seconds(self) -> int:
    return self._days * 86400 + self._seconds

# Should be
def total_seconds(self) -> float:
    return float(self._days * 86400 + self._seconds)
```

---

### 2. re.sifr

**Implementation Location**: `lib/sifr/re.sifr`

**Coverage**:
- Classes: `Match`, `Pattern`
- Functions: `search`, `search_match`, `sub`, `findall`, `split`, `compile`, `fullmatch`
- Flag constants: `IGNORECASE`, `MULTILINE`, `DOTALL`, `VERBOSE`

**Intrinsics Used**: `_sifr.regex` (re_match, re_find, re_replace, re_findall, re_split, re_find_start, re_find_end, re_match_flags, re_find_flags, re_replace_flags, re_findall_flags, re_split_flags)

**Test Coverage**:
- `crates/sifr/tests/e2e/pass/cpython_re.sifr` (47 assertions)
- `crates/sifr/tests/e2e/pass/cpython_re_subset.sifr`

**Issues Found**:

| Severity | Issue | Location | CPython Behavior |
|----------|-------|----------|------------------|
| MEDIUM | `fullmatch` uses manual `^pattern$` anchoring | Lines 120-127 | Native fullmatch support |
| MEDIUM | Missing `Match.group(n)` for captured groups | Lines 20-21 | Supports group capture |
| MEDIUM | Missing `Pattern.flags` attribute | Lines 74-110 | Exposes flags property |
| LOW | Missing `escape()` function | N/A | Supported |

**Root Cause**: The `fullmatch` implementation manually prepends `^` and appends `$` to simulate the behavior, which doesn't handle all edge cases (e.g., patterns with anchors already).

---

### 3. math.sifr

**Implementation Location**: `lib/sifr/math.sifr`

**Coverage**:
- Imports 55 intrinsic functions from `_sifr.math`
- Pure Sifr implementations: `factorial`, `gcd`, `lcm`, `comb`, `perm`, `log_base`, `isclose`, `prod`
- Tuple-style adapters: `frexp_mantissa`, `frexp_exponent`, `modf_fractional`, `modf_integral`
- Alias: `pow`

**Intrinsics Used**: `_sifr.math` (full intrinsic coverage)

**Test Coverage**:
- `crates/sifr/tests/e2e/pass/cpython_math.sifr` (113 assertions)
- `crates/sifr/tests/e2e/pass/cpython_math_extended.sifr`
- `crates/sifr/tests/e2e/pass/cpython_math_parity_expanded_matrix.sifr`
- `crates/sifr/tests/e2e/pass/cpython_math_semantic_corrections_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_math_missing_surface_subset.sifr`

**Issues Found**:

| Severity | Issue | Location | CPython Behavior |
|----------|-------|----------|------------------|
| **HIGH** | `factorial(-n)` returns 0 | Lines 4-6 | Raises `ValueError` |
| **HIGH** | `comb(-1, 1)` returns 0 | Lines 41-43 | Raises `ValueError` |
| **HIGH** | `comb(5, -1)` returns 0 | Lines 41-43 | Raises `ValueError` |
| **HIGH** | `comb(5, 10)` returns 0 | Lines 44-45 | Raises `ValueError` |
| **HIGH** | `perm(-1, 1)` returns 0 | Lines 62-63 | Raises `ValueError` |
| **HIGH** | `perm(5, -1)` returns 0 | Lines 62-63 | Raises `ValueError` |
| **HIGH** | `perm(5, 10)` returns 0 | Lines 64-65 | Raises `ValueError` |
| MEDIUM | `isclose` missing default parameter values | Lines 76-99 | `rel_tol=1e-9, abs_tol=0.0` |
| LOW | Missing `factorial` with `mod` parameter | N/A | Python 3.8+ supports mod |

**Root Cause**: The `factorial`, `comb`, and `perm` functions return 0 for invalid inputs instead of raising `ValueError` as CPython does. This is a semantic deviation that should be fixed.

```sifr
# Current (incorrect)
def factorial(n: int) -> int:
    if n < 0:
        return 0  # Should raise ValueError

# Should be
def factorial(n: int) -> int:
    if n < 0:
        raise ValueError("factorial() not defined for negative integers")
```

---

### 4. statistics.sifr

**Implementation Location**: `lib/sifr/statistics.sifr`

**Coverage**:
- Functions: `mean`, `median`, `variance`, `pvariance`, `stdev`, `pstdev`, `fmean`, `harmonic_mean`, `geometric_mean`, `median_low`, `median_high`, `mode`, `multimode`, `quantiles`, `covariance`, `correlation`, `linear_regression`
- Imports from `sifr.math`: `sqrt`, `log`, `exp`

**Test Coverage**:
- `crates/sifr/tests/e2e/pass/cpython_statistics.sifr` (42 assertions)
- `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`

**Issues Found**:

| Severity | Issue | Location | CPython Behavior |
|----------|-------|----------|------------------|
| LOW | Missing `NormalDist` class | N/A | Python 3.8+ |
| LOW | Missing `multimode` float support | Lines 164-197 | Supports floats |
| LOW | `mode` only supports `list[int]` | Line 138 | Supports any hashable |

**Assessment**: statistics.sifr is in good shape with only minor coverage gaps.

---

### 5. hashlib.sifr

**Implementation Location**: `lib/sifr/hashlib.sifr`

**Coverage**:
- Algorithms: md5, sha1, sha224, sha256, sha384, sha512, blake2b, blake2s
- Classes: `HashObject`
- Functions: `new`, `algorithms_guaranteed`, `algorithms_available`, `file_digest`
- Constructor functions: `md5_obj`, `sha1_obj`, `sha224_obj`, `sha256_obj`, `sha384_obj`, `sha512_obj`, `blake2b_obj`, `blake2s_obj`
- SHA3 placeholders: `sha3_256_obj`, `sha3_512_obj`, `shake_128_obj`, `shake_256_obj` (raise ValueError)

**Intrinsics Used**: `_sifr.crypto` (sha256, md5, sha1, sha512, sha224, sha384, blake2b, blake2s)

**Test Coverage**:
- `crates/sifr/tests/e2e/pass/stdlib_hashlib_intrinsics.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_hash.sifr`
- `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr`

**Issues Found**:

| Severity | Issue | Location | CPython Behavior |
|----------|-------|----------|------------------|
| **HIGH** | `HashObject.digest()` returns hex string | Lines 28-30 | Returns `bytes` |
| MEDIUM | Missing `update()` chaining support | Lines 22-23 | Returns `None`, enables chaining |
| LOW | SHA3 algorithms are placeholder errors | Lines 112-122 | Not yet implemented |

**Root Cause**: The `digest()` method returns a hex string (via `hexdigest()`) instead of raw bytes. This is an adaptation that should be explicitly documented. The current implementation:

```sifr
# Current (adaptation - needs documentation)
def digest(self) -> str:
    return self.hexdigest()  # Returns hex string, not bytes
```

---

## Coverage Quality Assessment

### Test Coverage Matrix

| Module | CPython Tests | Sifr Tests | Coverage |
|--------|---------------|------------|----------|
| datetime | test_datetime.py | cpython_datetime.sifr | Partial (timedelta focus) |
| re | test_re.py | cpython_re.sifr | Partial (basic operations) |
| math | test_math.py | cpython_math.sifr, cpython_math_*.sifr | Good |
| statistics | test_statistics.py | cpython_statistics.sifr | Good |
| hashlib | test_hashlib.py | stdlib_hashlib_*.sifr | Good |

### Missing Artifacts

1. **No demo file**: No `demos/wave_psp_e1_*_demo.sifr` exists
2. **No traceability document**: No `verification/stdlib/wave_psp_e1_cpython_traceability.md` exists

---

## Actionable Findings Summary

### Must Fix (Code Changes Required)

1. **math.sifr: factorial(-n)**: Change from returning 0 to raising `ValueError`
2. **math.sifr: comb(k<0 || k>n)**: Change from returning 0 to raising `ValueError`
3. **math.sifr: perm(k<0 || k>n)**: Change from returning 0 to raising `ValueError`
4. **datetime.sifr: timedelta.total_seconds()**: Change return type from `int` to `float`
5. **hashlib.sifr: digest() adaptation**: Document the hex-string adaptation in traceability

### Should Fix (Quality Improvements)

6. **datetime.sifr**: Add microseconds support to timedelta constructor
7. **datetime.sifr**: Add datetime arithmetic operators
8. **datetime.sifr**: Add comparison operators
9. **re.sifr**: Add Match.group(n) for captured groups
10. **math.sifr**: Add default values to isclose parameters

### Documentation Required

11. Create `verification/stdlib/wave_psp_e1_cpython_traceability.md`
12. Create `demos/wave_psp_e1_core_modules_demo.sifr`

---

## Validation Recommendations

Before the wave implementation proceeds, validate:

1. Run `factorial(-1)` - should raise ValueError (currently returns 0)
2. Run `comb(5, 10)` - should raise ValueError (currently returns 0)
3. Run `timedelta(0, 0).total_seconds()` - should return `0.0` (float), not `0` (int)
4. Verify hashlib digest returns hex string and document this adaptation

---

## Conclusion

The wave_psp_e1 modules have substantial implementation and test coverage, but several semantic deviations from CPython behavior were identified in the math module (factorial, comb, perm) and datetime module (timedelta.total_seconds). These are root-cause issues that should be fixed before the wave can be considered complete. The hashlib adaptation (digest returning hex string) should be explicitly documented in the traceability ledger.

**Review Status**: Issues found requiring code changes before implementation proceeds.
