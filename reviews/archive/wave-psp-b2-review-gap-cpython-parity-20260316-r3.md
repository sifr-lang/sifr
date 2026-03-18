# wave_psp_b2 Review: Implementation Gaps and CPython Parity Quality

**Review Date:** 2026-03-16
**Wave:** `wave_psp_b2` - Iterators, Functional Helpers, and Randomness
**Status:** Implemented (per execution ledger)

---

## Executive Summary

The wave_psp_b2 implementation covers the core surfaces for `itertools`, `functools`, `operator`, `random`, and `secrets` modules. The implementation is functional and passes validation tests. However, there are several concrete gaps and quality concerns that should be addressed to achieve production-grade parity.

---

## 1. Implementation Coverage Analysis

### 1.1 itertools.sifr (`lib/sifr/itertools.sifr`)

**Implemented:**
- `chain`, `repeat`, `take`, `flatten`, `pairwise`, `batched`
- `islice(start, stop, step)`
- `product(*iterables, repeat=)`
- `permutations`, `combinations`, `combinations_with_replacement`
- `starmap`
- `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`
- `zip_longest`, `count_from`, `cycle`

**Missing/Issues:**
| Function | Status | File Location | Issue |
|----------|--------|----------------|-------|
| `count` | Not implemented | `lib/sifr/itertools.sifr` | CPython's `count(start, step)` is infinite iterator; Sifr needs eager version or explicit waiver |
| `tee` | Not implemented | `lib/sifr/itertools.sifr` | Requires lazy iterator protocol; explicitly waived in traceability |
| `groupby` | Not implemented | `lib/sifr/itertools.sifr` | Requires lazy iterator protocol; needs explicit waiver |

**Eager vs Lazy Parity Note:** The implementation correctly uses eager `list[...]` materialization instead of lazy iterators. This is documented as intentional-diff in the traceability.

### 1.2 functools.sifr (`lib/sifr/functools.sifr`)

**Implemented:**
- `reduce(func, data, initial)`

**Missing:**
| Function | Status | File Location | Issue |
|----------|--------|----------------|-------|
| `partial` | Not implemented | `lib/sifr/functools.sifr` | Fail test exists: `phase_psp_b2_functools_partial_unsupported.sifr` |
| `cache`, `lru_cache` | Not implemented | `lib/sifr/functools.sifr` | Decorator infrastructure not available |
| `cmp_to_key` | Not implemented | `lib/sifr/functools.sifr` | Callable wrapper limitation |
| `cached_property` | Not implemented | `lib/sifr/functools.sifr` | Property descriptor not available |
| `total_ordering` | Not implemented | `lib/sifr/functools.sifr` | Class decorator not available |
| `wraps`, `update_wrapper` | Not implemented | `lib/sifr/functools.sifr` | Metaclass/decorator infrastructure needed |
| `singledispatch` | Not implemented | `lib/sifr/functools.sifr` | Complex descriptor protocol needed |

**Concern:** The `functools` module is severely undersurface. Only `reduce` is implemented out of 17 top-level exports. This is documented but warrants review for production readiness.

### 1.3 operator.sifr (`lib/sifr/operator.sifr`)

**Implemented:**
- Arithmetic: `add`, `sub`, `mul`, `floordiv`, `mod_val`, `neg`
- Comparison: `lt`, `le`, `eq`, `ne`, `ge`, `gt`
- Logical: `and_`, `or_`, `not_`
- Access: `getitem`, `itemgetter`, `contains`, `truth`

**Missing:**
| Function | Status | File Location | Issue |
|----------|--------|----------------|-------|
| `attrgetter` | Not implemented | `lib/sifr/operator.sifr` | Fail test: `phase_psp_b2_operator_attrgetter_unsupported.sifr` |
| `methodcaller` | Not implemented | `lib/sifr/operator.sifr` | Fail test: `phase_psp_b2_operator_methodcaller_unsupported.sifr` |
| `abs` | Not implemented | `lib/sifr/operator.sifr` | Should map to builtin |
| `concat` | Not implemented | `lib/sifr/operator.sifr` | Sequence concatenation |
| `delitem`, `setitem` | Not implemented | `lib/sifr/operator.sifr` | Mutation operators |
| `lshift`, `rshift` | Not implemented | `lib/sifr/operator.sifr` | Bit shift operators |
| `pow` | Not implemented | `lib/sifr/operator.sifr` | Power operator |
| `index`, `indexOf` | Not implemented | `lib/sifr/operator.sifr` | Index operations |
| `length_hint` | Not implemented | `lib/sifr/operator.sifr` | Protocol method |
| `matmul`, `imatmul` | Not implemented | `lib/sifr/operator.sifr` | Matrix multiplication |

### 1.4 random.sifr (`lib/sifr/random.sifr`)

**Implemented:**
- `choice`, `choices`, `randrange`, `randint`
- `shuffle`, `sample`, `getrandbits`
- `random`, `uniform`, `gauss`

**Missing:**
| Function | Status | File Location | Issue |
|----------|--------|----------------|-------|
| `seed` | Not implemented | `lib/sifr/random.sifr` | Deterministic state - explicitly waived |
| `getstate`, `setstate` | Not implemented | `lib/sifr/random.sifr` | State serialization - explicitly waived |
| `randbytes` | Not implemented | `lib/sifr/random.sifr` | Random bytes generation |
| Weighted `choices` | Not implemented | `lib/sifr/random.sifr` | Fail test: `phase_psp_b2_random_choices_weights_unsupported.sifr` |
| `Random` class | Not implemented | `lib/sifr/random.sifr` | Class-based API - explicitly waived |
| `SystemRandom` class | Not implemented | `lib/sifr/random.sifr` | Crypto random - partially available via secrets |
| Distribution functions | Not implemented | `lib/sifr/random.sifr` | `betavariate`, `gammavariate`, etc. |

### 1.5 secrets.sifr (`lib/sifr/secrets.sifr`)

**Implemented:**
- `compare_digest`, `token_hex`
- `randbelow`, `randbits`, `choice`

**Missing:**
| Function | Status | File Location | Issue |
|----------|--------|----------------|-------|
| `token_urlsafe` | Not implemented | `lib/sifr/secrets.sifr` | Fail test: `phase_psp_b2_secrets_token_urlsafe_unsupported.sifr` |
| `token_bytes` | Not implemented | `lib/sifr/secrets.sifr` | Bytes-oriented token generation |
| `SystemRandom` | Not implemented | `lib/sifr/secrets.sifr` | Class-based API |
| `DEFAULT_ENTROPY` | Not implemented | `lib/sifr/secrets.sifr` | Constant |

**Security Note:** The current `compare_digest` implementation is simple string equality (`a == b`), not constant-time. This is documented as "not currently documented as constant-time across all hosts" in traceability.

---

## 2. Adopt/Adapt/Waive Mapping Coherence

### 2.1 Current Classification

| Module | Adopted | Adapted | Waived | Coherence Issues |
|--------|---------|---------|--------|------------------|
| itertools | chain, islice, product, permutations, combinations, starmap | Eager list returns | lazy iterators, tee, groupby | **OK** |
| functools | reduce | None | partial, cache, lru_cache, cmp_to_key | **Gap: Over-waived** - Only 1/17 exports implemented |
| operator | getitem, contains, truth, basic math ops | None | attrgetter, methodcaller, matmul | **OK** |
| random | choice, choices, randrange, shuffle, getrandbits | ValueError on invalid input | seed, state, weights, distributions | **OK** |
| secrets | compare_digest, token_hex, randbelow, randbits, choice | ValueError on invalid input | token_urlsafe, token_bytes | **Concern: compare_digest not constant-time** |

### 2.2 Coherence Issues

1. **functools is severely undersurface:** Only `reduce` (1 of 17 exports) is implemented. While waivers exist, the ratio suggests incomplete wave coverage.

2. **compare_digest security concern:** The implementation `return a == b` is not constant-time and may leak timing information. This is a security-sensitive function that should either:
   - Be properly implemented with constant-time comparison
   - Be explicitly documented as not constant-time with security warnings

3. **Missing operator functions:** While `attrgetter` and `methodcaller` are correctly waived, other basic operators like `abs`, `concat`, `pow`, `lshift`, `rshift` are simply missing without explicit waiver documentation.

---

## 3. CPython Test Parity Quality

### 3.1 Test Coverage

| Module | CPython Test File | Sifr Test Coverage | Quality |
|--------|-------------------|-------------------|---------|
| itertools | `test_itertools.py` | `cpython_itertools_subset.sifr`, `phase_psp_b2_iterators_functional_randomness.sifr` | **Partial** - Core functions tested, lazy iterator tests absent |
| functools | `test_functools.py` | `stdlib_functools.sifr`, `phase_psp_b2_functools_partial_unsupported.sifr` | **Poor** - Only reduce tested |
| operator | `test_operator.py` | `cpython_random_subset.sifr` (uses getitem/contains/truth), `stdlib_operator.sifr` | **Partial** - Limited coverage |
| random | `test_random.py` | `cpython_random_subset.sifr`, `phase_psp_b2_iterators_functional_randomness.sifr` | **Good** - Core + error paths tested |
| secrets | `test_secrets.py` | `cpython_secrets_subset.sifr`, `phase_psp_b2_iterators_functional_randomness.sifr` | **Partial** - Basic functions tested |

### 3.2 Test Quality Concerns

1. **Insufficient edge case testing:** The test coverage focuses on happy paths. Error handling edge cases (empty inputs, boundary conditions) are partially covered but not comprehensive.

2. **Missing negative path tests:** Several CPython error conditions are not tested in Sifr:
   - `itertools` overflow conditions
   - `operator` type errors
   - `functools` reduce edge cases

3. **Test organization:** Tests are split across multiple files (`phase_psp_b2_*.sifr`, `cpython_*_subset.sifr`, `stdlib_*.sifr`), making it difficult to understand complete coverage.

---

## 4. Actionable Issues with File-Level Precision

### Critical Issues

| # | File | Issue | Recommendation |
|---|------|-------|-----------------|
| C1 | `lib/sifr/functools.sifr` | Only `reduce` implemented out of 17 exports | Either expand coverage or document why 16/17 are waived |
| C2 | `lib/sifr/secrets.sifr:6-7` | `compare_digest` uses `a == b`, not constant-time | Implement constant-time comparison or document security limitation |

### High Priority Issues

| # | File | Issue | Recommendation |
|---|------|-------|-----------------|
| H1 | `lib/sifr/operator.sifr` | Missing basic operators (abs, pow, lshift, rshift, concat) | Add implementations or document explicit waivers |
| H2 | `lib/sifr/random.sifr` | Missing `randbytes` function | Add implementation or document waiver |
| H3 | `lib/sifr/itertools.sifr` | Missing `count` function | Add eager version or document waiver |
| H4 | `verification/stdlib/wave_psp_b2_cpython_traceability.md` | `compare_digest` not documented as non-constant-time | Add explicit security disclaimer |

### Medium Priority Issues

| # | File | Issue | Recommendation |
|---|------|-------|-----------------|
| M1 | `crates/sifr/tests/e2e/pass/cpython_random_subset.sifr` | Test for weighted choices exists in CPython but not adapted | Add weighted choices test or strengthen waiver |
| M2 | `crates/sifr/tests/e2e/pass/cpython_secrets_subset.sifr` | Missing `token_bytes` coverage | Add tests or strengthen waiver |
| M3 | `lib/sifr/functools.sifr` | No test file specifically for reduce edge cases | Add comprehensive reduce tests |

---

## 5. Validation Status

Demo execution:
```
$ cargo run -q -p sifr -- run demos/wave_psp_b2_iterators_functional_randomness_demo.sifr
chain(*iterables) = [1, 2, 3, 4]
islice(start, stop, step) = [20, 40]
product(repeat=2) = [[1, 1], [1, 2], [2, 1], [2, 2]]
permutations(r=2) = [[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]]
combinations(r=2) = [[1, 2], [1, 3], [2, 3]]
starmap(add, pairs) = [5, 9]
callable object direct = 8
shuffle(mut items) len = 5
choice(items) ok = true
choices(items, k=3) len = 3
randrange(10) ok = true
secrets.compare_digest = true
secrets.token_hex(4) len = 8
secrets.randbits(16) ok = true
```

**Status:** Passes

---

## 6. Recommendations Summary

1. **Address C1 (functools undersurface):** Expand functools coverage or add comprehensive waiver documentation explaining why 16 of 17 exports are not implemented.

2. **Address C2 (compare_digest security):** Either implement constant-time comparison or add prominent security warnings in documentation.

3. **Strengthen adopt/adapt/waive mapping:** Add explicit waivers for all missing operator functions and document the security implications of non-constant-time compare_digest.

4. **Improve test coverage:** Add more comprehensive edge case testing, especially for error paths.

5. **Consider future lazy iterator support:** The current eager-only implementation is documented as intentional-diff, but users should be aware this limits interoperability with lazy iterator-consuming code.

---

## Conclusion

wave_psp_b2 is functional and passes validation, but has notable gaps in surface coverage (especially functools) and a security-sensitive issue with `compare_digest`. The adopt/adapt/waive mapping is mostly coherent but needs strengthening with explicit waivers for missing operator functions and security disclaimers for non-constant-time operations.
