# wave_psp_e1 Review: CPython Parity Gap Analysis

**Date**: 2026-03-17
**Branch**: codex/python-builtin-std-parity-wave-e2
**Reviewer**: Claude Code

---

## Executive Summary

wave_psp_e1 ("strong-but-incomplete core modules") covers five CPython module families: `datetime`, `re`, `math`, `statistics`, and `hashlib`. This review assesses the current implementation against the traceability contract in `wave_psp_e1_cpython_traceability.md` and identifies any gaps between the documented adopt/adapt/waive decisions and actual implementation/test behavior.

**Status**: No actionable gaps found. All documented adaptations are correctly implemented and tested.

---

## 1. Verification Traceability Contract Compliance

### 1.1 Traceability Document Analysis

The `wave_psp_e1_cpython_traceability.md` defines:

**Reviewed upstream families (adapted state)**:
| CPython family | Surface | State |
|---|---|---|
| `Lib/test/test_datetime.py` | `timedelta`, `datetime/date/time` formatting, timestamp conversion | adapted |
| `Lib/test/test_re.py` | search/findall/split/sub/fullmatch behavior, flags | adapted |
| `Lib/test/test_math.py` | combinatorics, floating-point tolerance | adapted |
| `Lib/test/test_statistics.py` | mean/median/variance families | adapted |
| `Lib/test/test_hashlib.py` | hash object construction/update/hexdigest | adapted |

**Classified waivers (unsupported)**:
- Full timezone-aware/calendar object parity in `datetime`
- Full `re` Match/Pattern object matrix
- Decimal/Fraction-specific semantics in `math`/`statistics`
- SHA3/SHAKE constructor families in `hashlib`

---

## 2. Implementation vs. Traceability Gap Analysis

### 2.1 datetime Module

| Surface | Traceability Contract | Implementation | Gap? |
|---------|----------------------|----------------|------|
| `timedelta.total_seconds()` | Returns integral values (adapted) | Returns `int` | ✅ Matches |
| `timedelta` constructor | N/A | Only `days`, `seconds` params | N/A |
| Timestamp conversion | Typed error handling | `Result[str, ValueError]` | ✅ Matches |

**Evidence**:
- `lib/sifr/datetime.sifr:30-31` returns `int`
- `cpython_datetime_subset.sifr:11-13` asserts `total_seconds() == 86400` (int)
- `phase_psp_e1_core_modules_numeric_patterns_crypto.sifr:13` asserts `total_seconds() == 0` (int)

**Finding**: No gap. Implementation and tests align with traceability adaptation.

---

### 2.2 math Module

| Surface | Traceability Contract | Implementation | Gap? |
|---------|----------------------|----------------|------|
| `factorial(-1)` | Returns 0 (deterministic adaptation) | Returns 0 | ✅ Matches |
| `comb(5, 10)` | Returns 0 (deterministic adaptation) | Returns 0 | ✅ Matches |
| `perm(5, 10)` | Returns 0 (deterministic adaptation) | Returns 0 | ✅ Matches |

**Evidence**:
- `lib/sifr/math.sifr:4-6` returns 0 for `n < 0`
- `lib/sifr/math.sifr:41-45` returns 0 for invalid `comb` inputs
- `lib/sifr/math.sifr:61-65` returns 0 for invalid `perm` inputs
- `cpython_math_missing_surface_subset.sifr:75` asserts `factorial(-1) == 0`
- `phase_psp_e1_core_modules_numeric_patterns_crypto.sifr:66-67` asserts `comb(5, 10) == 0` and `perm(5, 10) == 0`

**Finding**: No gap. Implementation and tests align with traceability adaptation.

---

### 2.3 hashlib Module

| Surface | Traceability Contract | Implementation | Gap? |
|---------|----------------------|----------------|------|
| `digest()` | Alias to `hexdigest()` (hex string) | Returns `str` | ✅ Matches |
| SHA3/SHAKE | Placeholders raising typed errors | Raises `ValueError` | ✅ Matches |

**Evidence**:
- `lib/sifr/hashlib.sifr:28-30` returns `self.hexdigest()`
- `cpython_hashlib_object_model_subset.sifr:17` asserts `h.digest() == h.hexdigest()`
- `lib/sifr/hashlib.sifr:112-122` raises `ValueError` for SHA3/SHAKE

**Finding**: No gap. Implementation and tests align with traceability adaptation.

---

### 2.4 statistics Module

| Surface | Traceability Contract | Implementation | Gap? |
|---------|----------------------|----------------|------|
| Error handling | Uses typed `StatisticsError` | `Result[..., StatisticsError]` | ✅ Matches |
| `mode()` | Supports `list[int]` | `list[int]` only | ✅ Matches (adapted) |

**Evidence**:
- `lib/sifr/statistics.sifr:4-5` defines `StatisticsError`
- All functions return `Result[T, StatisticsError]`
- `lib/sifr/statistics.sifr:138` signature: `def mode(data: list[int])`

**Finding**: No gap. Implementation aligns with traceability.

---

### 2.5 re Module

| Surface | Traceability Contract | Implementation | Gap? |
|---------|----------------------|----------------|------|
| Flags | Python-shaped constants | `IGNORECASE=2`, `MULTILINE=8`, etc. | ✅ Matches |
| Invalid pattern | Typed regex error | `Result[..., RegexError]` | ✅ Matches |

**Evidence**:
- `lib/sifr/re.sifr:4-8` flag constants match CPython values
- `phase_psp_e1_core_modules_numeric_patterns_crypto.sifr:49-55` tests invalid pattern rejection

**Finding**: No gap. Implementation aligns with traceability.

---

## 3. CPython Test Parity Quality

### 3.1 Test Coverage Matrix

| Module | CPython Source | Sifr Fixture | Status |
|--------|---------------|--------------|--------|
| datetime | test_datetime.py | cpython_datetime_subset.sifr | ✅ Adapting |
| re | test_re.py | cpython_re_subset.sifr | ✅ Adapting |
| math | test_math.py | cpython_math_*.sifr | ✅ Adapting |
| statistics | test_statistics.py | cpython_statistics_subset.sifr | ✅ Adapting |
| hashlib | test_hashlib.py | cpython_hashlib_*.sifr | ✅ Adapting |

### 3.2 E2E Validation Results

**Demo run**:
```
datetime.isoformat = 2026-03-16T12:00:00
datetime.from_timestamp(0) = 1970-01-01T00:00:00
re.search = 1200
math.comb(8, 3) = 56
math.isclose(0.1+0.2, 0.3) = true
statistics.mean = 5
hashlib.sha256 len = 64
```

**Finding**: Demo runs successfully. All core functionality works as expected.

---

## 4. Documentation Artifacts Status

| Document | Status |
|----------|--------|
| `wave_psp_e1_cpython_traceability.md` | ✅ Complete |
| Demo (`demos/wave_psp_e1_strong_core_modules_demo.sifr`) | ✅ Working |
| E2E fixtures | ✅ Present |
| Pass test: `phase_psp_e1_core_modules_numeric_patterns_crypto.sifr` | ✅ 20 assertions |

---

## 5. Findings Summary

### 5.1 Actionable Gaps: NONE

All surfaces documented in the traceability matrix are correctly implemented and tested. The implementation matches the documented adaptations.

### 5.2 CPython Parity Quality: HIGH

- **Adapted coverage**: All shipped surfaces have CPython-derived test coverage
- **Enforcement**: Tests explicitly validate adaptations (e.g., `factorial(-1) == 0`, `digest() == hexdigest()`)
- **Documented divergences**: All intentional differences are tracked in traceability

### 5.3 Verification Traceability Compliance: PASS

The implementation correctly follows the adopt/adapt/waive decisions documented in `wave_psp_e1_cpython_traceability.md`. No undocumented deviations were found.

---

## 6. Appendix: Evidence Files

### Pass Tests
- `crates/sifr/tests/e2e/pass/cpython_datetime_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_re_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_math_missing_surface_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_math_semantic_corrections_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr`
- `crates/sifr/tests/e2e/pass/phase_psp_e1_core_modules_numeric_patterns_crypto.sifr`

### Implementation Files
- `lib/sifr/datetime.sifr`
- `lib/sifr/re.sifr`
- `lib/sifr/math.sifr`
- `lib/sifr/statistics.sifr`
- `lib/sifr/hashlib.sifr`

### Demo
- `demos/wave_psp_e1_strong_core_modules_demo.sifr`
