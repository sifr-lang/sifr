# wave_psp_b2 Review: Implementation Gaps and CPython Parity Quality

**Review Date:** 2026-03-17
**Reviewer:** Code Review
**Scope:** wave_psp_b2 (itertools, functools.reduce, operator, random, secrets)

---

## Executive Summary

The wave_psp_b2 implementation covers 5 CPython modules: `itertools`, `functools`, `operator`, `random`, and `secrets`. All pass tests execute successfully and fail tests correctly fail with expected errors. However, several gaps exist between traceability claims and shipped behavior, and some CPython parity surfaces lack test coverage.

---

## 1. Verified Working Components

### Pass Tests (All Execute Successfully)
| Test File | Status |
|-----------|--------|
| `phase_psp_b2_iterators_functional_randomness.sifr` | ✓ Pass |
| `cpython_random_subset.sifr` | ✓ Pass |
| `cpython_secrets_subset.sifr` | ✓ Pass |
| `cpython_itertools.sifr` | ✓ Pass |
| `stdlib_functools.sifr` | ✓ Pass |

### Fail Tests (All Correctly Fail)
| Test File | Expected Error | Status |
|-----------|---------------|--------|
| `phase_psp_b2_functools_partial_unsupported.sifr` | `module 'sifr.functools' has no member 'partial'` | ✓ Fail |
| `phase_psp_b2_operator_attrgetter_unsupported.sifr` | `module 'sifr.operator' has no member 'attrgetter'` | ✓ Fail |
| `phase_psp_b2_operator_methodcaller_unsupported.sifr` | `module 'sifr.operator' has no member 'methodcaller'` | ✓ Fail |
| `phase_psp_b2_random_choices_weights_unsupported.sifr` | `choices() got an unexpected keyword argument 'weights'` | ✓ Fail |
| `phase_psp_b2_secrets_token_urlsafe_unsupported.sifr` | `module 'sifr.secrets' has no member 'token_urlsafe'` | ✓ Fail |

---

## 2. Traceability Gaps

### 2.1 itemgetter Not Mentioned in Traceability

**Finding:** The `operator.itemgetter` function is implemented in `lib/sifr/operator.sifr` (lines 68-69) and tested indirectly via `operator.getitem`, but it is **not listed** in the "Reviewed upstream families" table of `wave_psp_b2_cpython_traceability.md`.

**Traceability Claim:**
> "`itemgetter(items, index)` remains a direct helper in this wave rather than the CPython callable factory."

**Shipped Behavior:**
- `itemgetter` is implemented and works: `itemgetter([1,2,3], 1)` returns `2`
- However, it's not explicitly tested in any wave b2 test file

**Impact:** Low - functionality works but not covered by traceability matrix.

### 2.2 Additional Implemented Functions Not in Traceability

The following functions are implemented but not listed in the traceability document:

| Module | Function | Implemented | Tested in Wave b2 |
|--------|----------|--------------|-------------------|
| `random` | `randint` | ✓ | ✗ |
| `random` | `uniform` | ✓ | ✗ |
| `random` | `gauss` | ✓ | ✗ |
| `random` | `sample` | ✓ | ✗ |
| `random` | `random` | ✓ | ✗ |
| `operator` | `itemgetter` | ✓ | ✗ |

---

## 3. CPython Parity Test Coverage Gaps

### 3.1 itertools Negative Path Coverage

**Finding:** No negative test cases for edge conditions in itertools functions:

| Function | Edge Case | CPython Behavior | Sifr Implementation | Tested? |
|----------|-----------|------------------|---------------------|---------|
| `product` | `repeat=-1` | Returns `[]` | Returns `[]` (line 149-150) | ✗ |
| `permutations` | `r > len(data)` | Returns `[]` | Returns `[]` (line 167-168) | ✗ |
| `combinations` | `r > len(data)` | Returns `[]` | Returns `[]` (line 186-187) | ✗ |
| `combinations_with_replacement` | Empty data | Returns `[]` | Returns `[]` (line 208-209) | ✗ |
| `islice` | `step <= 0` | Returns empty iterator | Returns `[]` (line 123-124) | ✗ |

### 3.2 starmap Limitation Not Tested

**Finding:** The `starmap` implementation only supports functions with exactly 2 arguments:

```sifr
# lib/sifr/itertools.sifr line 223
def starmap(func: Callable[[A, B], R], pairs: list[tuple[A, B]]) -> list[R]:
```

**CPython Behavior:** `itertools.starmap` accepts any callable and iterates over any-length tuples.

**Sifr Limitation:** Only handles 2-argument functions.

**Test Coverage:** Only tested with 2-argument function (line 33 of phase_psp_b2 test). No test for single-argument or 3+-argument functions.

### 3.3 operator Additional Functions Not Tested

**Finding:** The following operator functions are implemented but not tested in wave b2:
- `add`, `sub`, `mul`, `floordiv`, `mod_val`, `neg`
- `lt`, `le`, `eq`, `ne`, `ge`, `gt`
- `and_`, `or_`, `not_`

These exist in `lib/sifr/operator.sifr` but are not part of the wave b2 test suite.

---

## 4. Security Consideration

### 4.1 compare_digest Not Constant-Time

**Finding:** The implementation uses simple string equality:

```sifr
# lib/sifr/secrets.sifr line 6-7
def compare_digest(a: str, b: str) -> bool:
    return a == b
```

**Traceability Note:** The document states: "compare_digest is functionally correct for str inputs in this wave, but it is not currently documented as constant-time across all hosts."

**Issue:** This is documented as a waiver, but there is no test or assertion verifying this behavior difference from CPython. The security implication should be explicitly tested or documented more prominently.

---

## 5. Missing CPython Functions

### 5.1 itertools Functions Not Implemented

| Function | CPython | Sifr | Notes |
|----------|---------|------|-------|
| `count` | Infinite iterator | ✗ | Different from `count_from` helper |
| `groupby` | Group iterator | ✗ | Requires lazy evaluation |
| `tee` | Cloned iterator | ✗ | Requires lazy evaluation |

### 5.2 random Functions Not Implemented (Waiver)

| Function | Waived | Guard Test |
|----------|--------|------------|
| `seed` | ✓ | ✗ |
| `getstate` | ✓ | ✗ |
| `setstate` | ✓ | ✗ |
| `Random` object | ✓ | ✗ |
| `SystemRandom` object | ✓ | ✗ |

### 5.3 secrets Functions Not Implemented (Waiver)

| Function | Waived | Guard Test |
|----------|--------|------------|
| `token_bytes` | ✓ | ✗ |
| `token_urlsafe` | ✓ | `phase_psp_b2_secrets_token_urlsafe_unsupported.sifr` |

---

## 6. Actionable Findings

### High Priority

1. **Add negative path tests for itertools** - Add test cases for:
   - `product` with negative repeat
   - `permutations` with r > len
   - `combinations` with r > len
   - `islice` with step <= 0

2. **Document starmap limitation in traceability** - The 2-argument limitation should be explicitly documented as a functional gap, not just an implementation detail.

### Medium Priority

3. **Add itemgetter to traceability matrix** - Update `wave_psp_b2_cpython_traceability.md` to include `itemgetter` in the reviewed families.

4. **Add test for compare_digest security behavior** - Either add a test documenting the non-constant-time behavior, or add documentation emphasizing this security difference.

5. **Extend test coverage for additional implemented functions** - Functions like `randint`, `uniform`, `sample` are implemented but not tested in wave b2.

### Low Priority

6. **Consider adding operator tests** - The basic arithmetic and comparison operators in `operator` module are not tested in wave b2.

---

## 7. Summary

| Category | Count |
|----------|-------|
| Pass tests verified | 5 |
| Fail tests verified | 5 |
| Traceability gaps | 2 |
| Missing test coverage | 5 |
| Missing implementations | 3 |

The wave_psp_b2 implementation is functionally correct and all tests pass. However, there are gaps in traceability documentation and some CPython parity surfaces lack test coverage. The most significant actionable items are adding negative path tests for itertools functions and documenting the starmap 2-argument limitation.
