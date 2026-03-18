# wave_psp_b2 Review: Implementation Gaps and CPython Parity Quality

**Review Date:** 2026-03-16
**Branch:** wave_psp_b2 (merged into main)
**Status:** MERGED

---

## Executive Summary

wave_psp_b2 covers iterators (itertools), functional helpers (functools), operators (operator), randomness (random), and cryptography-adjacent utilities (secrets). The wave is **functionally complete** with working implementations, but has several **test coverage gaps** and **documented limitations** that should be addressed.

---

## 1. Actionable Implementation Gaps

### 1.1 Missing CPython Feature Parity

| Feature | CPython Behavior | Sifr Behavior | Gap Severity | Notes |
| --- | --- | --- | --- | --- |
| `accumulate` initial param | `accumulate([1,2,3], initial=0)` returns `[0, 1, 3, 6]` | No `initial` parameter | Medium | Only works without initial value |
| `starmap` variadic args | Works with any function arity via *args | Fixed to 2-arg functions only | Medium | Only supports `Callable[[A, B], R]` |
| `compare_digest` timing safety | Constant-time comparison | Simple `a == b` (not constant-time) | **High** | Security-sensitive; documented as gap but needs warning |
| `random.choices` weights | Supports `weights=` parameter | Not supported | Medium | Fail test exists; correctly waived |
| `operator.attrgetter` | Returns callable that gets attributes | Not implemented | Waived | Correctly documented as unsupported |
| `operator.methodcaller` | Returns callable that calls methods | Not implemented | Waived | Correctly documented as unsupported |
| `functools.partial` | Returns callable with pre-filled args | Not implemented | Waived | Correctly documented as unsupported |
| `secrets.token_urlsafe` | Returns URL-safe base64 token | Not implemented | Waived | Correctly documented as unsupported |

### 1.2 Test Coverage Gaps

#### Missing Tests in CPython Subset Files

1. **product, permutations, combinations, combinations_with_replacement**: The traceability document claims these are "adapted," but:
   - `cpython_itertools.sifr` does NOT test these functions
   - `cpython_itertools_subset.sifr` does NOT test these functions
   - Only `phase_psp_b2_iterators_functional_randomness.sifr` has minimal assertions

   **Action needed:** Add dedicated assertions in `cpython_itertools_subset.sifr` for:
   - `product([1,2], [3,4])`
   - `permutations([1,2,3], 2)`
   - `combinations([1,2,3], 2)`
   - `combinations_with_replacement([1,2], 2)`

2. **starmap coverage**: Only tested in `phase_psp_b2_iterators_functional_randomness.sifr` with a simple 2-arg case. No negative-path tests or edge cases.

3. **operator tests**: The `operator.sifr` implementation includes `and_`, `or_`, `not_`, `floordiv`, `mod_val`, but these are NOT tested in the wave_psp_b2 specific tests. Only `getitem`, `contains`, `truth`, and `itemgetter` are tested.

4. **functools.reduce with callable objects**: The traceability claims "higher-order callable acceptance" is adapted, but there's no dedicated test for using user-defined `__call__` objects with `reduce`. The test in `phase_psp_b2_iterators_functional_randomness.sifr` only tests a simple function.

### 1.3 Stale/Dead Test Files

| File | Issue |
| --- | --- |
| `crates/sifr/tests/e2e/pass/stdlib_random.sifr` | Uses internal functions `random_int`, `random_float` which are not part of the public API (imported from `_sifr.crypto`). This test may be outdated or from a different wave. |

---

## 2. CPython Test Parity Quality Assessment

### 2.1 Adapted Coverage (Ported/Adapted)

| Module | Functions with Good Coverage | Coverage Quality |
| --- | --- | --- |
| **itertools** | `chain`, `islice`, `repeat`, `take`, `flatten`, `pairwise`, `batched` | Good - dedicated cpython_itertools.sifr and cpython_itertools_subset.sifr |
| **itertools** | `product`, `permutations`, `combinations`, `combinations_with_replacement`, `starmap` | **Weak** - only minimal assertions in main test |
| **random** | `shuffle`, `choice`, `choices`, `randrange`, `getrandbits` | Good - cpython_random_subset.sifr with positive and negative tests |
| **secrets** | `compare_digest`, `randbits`, `randbelow`, `token_hex`, `choice` | Good - cpython_secrets_subset.sifr |
| **operator** | `getitem`, `contains`, `truth` | Moderate - only basic positive tests |
| **functools** | `reduce` | Moderate - basic tests only |

### 2.2 Waived Coverage (Documented Gaps)

The following are correctly documented as waivers with fail tests:

| Waived Feature | Fail Test | Documentation |
| --- | --- | --- |
| `functools.partial` | `phase_psp_b2_functools_partial_unsupported.sifr` | ✅ Traceability |
| `operator.attrgetter` | `phase_psp_b2_operator_attrgetter_unsupported.sifr` | ✅ Traceability |
| `operator.methodcaller` | `phase_psp_b2_operator_methodcaller_unsupported.sifr` | ✅ Traceability |
| `random.choices` weights | `phase_psp_b2_random_choices_weights_unsupported.sifr` | ✅ Traceability |
| `secrets.token_urlsafe` | `phase_psp_b2_secrets_token_urlsafe_unsupported.sifr` | ✅ Traceability |

### 2.3 Coverage Fidelity Issues

1. **compare_digest security claim**: The traceability document states "compare_digest is functionally correct for str inputs in this wave, but it is not currently documented as constant-time across all hosts." This is misleading - the implementation `return a == b` is **NOT** constant-time and should be documented as a **known security gap**, not just a documentation issue.

2. **Eager vs Lazy Iterators**: The wave correctly documents that itertools uses eager `list[...]` materialization instead of lazy iterators. This is a design decision, but tests don't explicitly verify this behavior difference.

3. **Error handling consistency**: Some functions return `Result[T, ValueError]` (e.g., `randrange`, `choice`), while others raise directly (e.g., `choices` with empty list). This inconsistency should be documented.

---

## 3. Recommendations

### High Priority

1. **Add security warning for compare_digest**: The current implementation is vulnerable to timing attacks. Either:
   - Document as a known security limitation
   - Implement constant-time comparison (using Rust's `subtle` crate)

2. **Add missing itertools tests**: Add `product`, `permutations`, `combinations`, `combinations_with_replacement` to `cpython_itertools_subset.sifr`

### Medium Priority

3. **Add starmap edge case tests**: Test with edge cases (empty list, single pair)

4. **Fix or remove stdlib_random.sifr**: The test uses non-public API functions

5. **Document accumulate limitation**: Note that `initial` parameter is not supported

6. **Add operator.and_/or_/not_ tests**: These functions exist but aren't tested in wave_psp_b2 specific tests

### Low Priority

7. **Consider adding accumulate with initial**: This would improve CPython parity

---

## 4. Test Execution Status

Tests compile and run successfully:
- `phase_psp_b2_iterators_functional_randomness.sifr` ✅
- `cpython_random_subset.sifr` ✅
- `cpython_secrets_subset.sifr` ✅
- `cpython_itertools.sifr` ✅
- `cpython_itertools_subset.sifr` ✅
- `stdlib_functools_reduce.sifr` ✅
- `stdlib_operator.sifr` ✅

Note: Full e2e test suite takes >120s to run; individual test execution verified.

---

## 5. Files Reviewed

### Implementation
- `lib/sifr/itertools.sifr` (349 lines)
- `lib/sifr/functools.sifr` (12 lines)
- `lib/sifr/operator.sifr` (78 lines)
- `lib/sifr/random.sifr` (82 lines)
- `lib/sifr/secrets.sifr` (45 lines)

### Tests
- `crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`
- `crates/sifr/tests/e2e/pass/cpython_itertools.sifr`
- `crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_random_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_secrets_subset.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_functools_reduce.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_operator.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_random.sifr` (flagged as potentially stale)

### Fail Tests
- `crates/sifr/tests/e2e/fail/phase_psp_b2_functools_partial_unsupported.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_b2_operator_attrgetter_unsupported.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_b2_operator_methodcaller_unsupported.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_b2_secrets_token_urlsafe_unsupported.sifr`

### Documentation
- `verification/stdlib/wave_psp_b2_cpython_traceability.md`
