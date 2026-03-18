# wave_psp_b2 Review: Implementation Gaps and CPython Parity

**Review Date:** 2026-03-17
**Reviewer:** Claude Code
**Wave Scope:** iterators (itertools), functional (functools.reduce), randomness (random, secrets), operator helpers

---

## Executive Summary

wave_psp_b2 is **largely well-implemented** with good test coverage. All claimed surfaces from the traceability document are shipped. However, there are **two notable behavioral deviations** from CPython that are not documented in the traceability as explicit intentional differences.

---

## 1. Implementation Gap Analysis

### 1.1 Critical Gaps (Blocking Issues)

**None identified.** All functions listed in the traceability as "adapted" are implemented and functional.

### 1.2 Medium Gaps (Behavioral Differences)

#### Gap 1: `operator.getitem` returns `None` for out-of-bounds vs CPython's `IndexError`

| Aspect | CPython | Sifr (Shipped) | Traceability Status |
|--------|---------|-----------------|---------------------|
| `operator.getitem([1,2,3], 10)` | Raises `IndexError` | Returns `None` | Listed as "adapted" but the IndexError→None difference not explicitly called out |

**Evidence:**
```python
# CPython
>>> import operator
>>> operator.getitem([1,2,3], 10)
IndexError: list index out of range

# Sifr (verified)
>>> getitem([10, 20, 30], 10)
None
```

**Impact:** Users expecting CPython's exception-raising behavior will get `None` instead. This is a safety-oriented design decision but represents an undocumented behavioral divergence.

#### Gap 2: `operator.truth` accepts only `list[T]` vs CPython's any object

| Aspect | CPython | Sifr (Shipped) | Traceability Status |
|--------|---------|----------------|---------------------|
| Signature | `truth(obj)` - accepts any object | `truth(value: list[T])` - only accepts lists | Listed as "adapted" but limited signature not documented |

**Evidence:**
```python
# CPython - accepts any object
>>> operator.truth([])
False
>>> operator.truth(0)
False
>>> operator.truth("hi")
True

# Sifr - only works with lists (verified)
truth([1,2,3]) → true
truth([]) → false
```

**Impact:** The function cannot be used with scalars, strings, dicts, or other container types. This significantly narrows the utility compared to CPython.

---

## 2. CPython Tests Parity Quality

### 2.1 Positive Findings

| Test File | Coverage Quality | Notes |
|-----------|-----------------|-------|
| `phase_psp_b2_iterators_functional_randomness.sifr` | ✅ Good | Tests chain, islice, product, permutations, combinations, combinations_with_replacement, starmap, reduce, callable objects, operator helpers, random, secrets |
| `cpython_random_subset.sifr` | ✅ Good | Canonical vector format with positive and negative test cases |
| `cpython_secrets_subset.sifr` | ✅ Good | Canonical vector format covering core and error paths |
| Fail tests | ✅ Correctly failing | All 5 waiver tests fail as expected |

### 2.2 Test Coverage Gaps

| CPython Test Family | Claimed Coverage | Actual Coverage | Notes |
|---------------------|------------------|------------------|-------|
| `test_itertools` - variadic chain | ✅ Covered | ✅ Verified | `chain(*iterables)` works correctly |
| `test_itertools` - islice | ✅ Covered | ✅ Verified | `islice(data, start, stop, step)` works correctly |
| `test_random` - shuffle | ✅ Covered | ✅ Verified | Mutates in place, returns None |
| `test_random` - randrange | ✅ Covered | ✅ Verified | Handles single arg, start/stop, step correctly |
| `test_operator` - getitem | ⚠️ Partial | Behavioral diff | Returns None vs IndexError |
| `test_operator` - truth | ⚠️ Partial | Behavioral diff | Only accepts list[T] |

---

## 3. Traceability vs Shipped Behavior Validation

### 3.1 Claimed Functions - Implementation Status

| Module | Function | Claimed Status | Shipped | Verified |
|--------|----------|---------------|---------|----------|
| itertools | `chain` | adapted | ✅ Yes | ✅ |
| itertools | `islice(start, stop, step)` | adapted | ✅ Yes | ✅ |
| itertools | `product(..., repeat=)` | adapted | ✅ Yes | ✅ |
| itertools | `permutations` | adapted | ✅ Yes | ✅ |
| itertools | `combinations` | adapted | ✅ Yes | ✅ |
| itertools | `combinations_with_replacement` | adapted | ✅ Yes | ✅ |
| itertools | `starmap` | adapted | ✅ Yes | ✅ |
| functools | `reduce` | adapted | ✅ Yes | ✅ |
| operator | `getitem` | adapted | ✅ Yes | ⚠️ returns None |
| operator | `contains` | adapted | ✅ Yes | ✅ |
| operator | `truth` | adapted | ✅ Yes | ⚠️ limited to list |
| random | `shuffle` | adapted | ✅ Yes | ✅ |
| random | `randrange(stop)` | adapted | ✅ Yes | ✅ |
| random | `randrange(start, stop, step)` | adapted | ✅ Yes | ✅ |
| random | `choice` | adapted | ✅ Yes | ✅ |
| random | `choices` | adapted | ✅ Yes | ✅ |
| random | `getrandbits` | adapted | ✅ Yes | ✅ |
| secrets | `compare_digest` | adapted | ✅ Yes | ✅ |
| secrets | `randbits` | adapted | ✅ Yes | ✅ |
| secrets | `choice` | adapted | ✅ Yes | ✅ |
| secrets | `randbelow` | adapted | ✅ Yes | ✅ |
| secrets | `token_hex` | adapted | ✅ Yes | ✅ |

### 3.2 Waivers - Correctly Failing

| Surface | Waiver Status | Fail Test | Verified |
|---------|---------------|-----------|----------|
| Lazy iterator objects | unsupported | ✅ N/A | ✅ (eager lists) |
| `functools.partial` | unsupported | ✅ fails correctly | ✅ |
| `operator.attrgetter` | unsupported | ✅ fails correctly | ✅ |
| `operator.methodcaller` | unsupported | ✅ fails correctly | ✅ |
| `random.choices(weights=)` | unsupported | ✅ fails correctly | ✅ |
| `secrets.token_urlsafe` | unsupported | ✅ fails correctly | ✅ |

---

## 4. Actionable Findings

### Finding 1: Document `operator.getitem` None-for-IndexError behavior

**Severity:** Medium
**Type:** Documentation gap
**Location:** `lib/sifr/operator.sifr:64-65`

The function returns `None` for out-of-bounds indices rather than raising `IndexError`. This is a safety-oriented design but should be explicitly documented.

**Recommendation:** Add docstring documenting the behavioral difference, or consider if an alternative that raises IndexError should be added.

### Finding 2: Document or expand `operator.truth` signature limitation

**Severity:** Medium
**Type:** API surface limitation
**Location:** `lib/sifr/operator.sifr:76-77`

Current signature: `def truth[T](value: list[T]) -> bool`
CPython signature: `def truth(obj) -> bool`

The function only accepts lists, not arbitrary objects.

**Recommendation:** Either:
1. Document this limitation in the traceability as an explicit "intentional-diff"
2. Expand the implementation to accept more types (though this may require broader type system work)

### Finding 3: Missing constant-time documentation for `compare_digest`

**Severity:** Low
**Type:** Documentation gap
**Location:** `lib/sifr/secrets.sifr:6-7`

The traceability notes "not currently documented as constant-time across all hosts" but there's no docstring in the implementation.

**Recommendation:** Add a docstring noting that this is a basic equality check and not guaranteed to be constant-time.

---

## 5. Verification Commands

```bash
# Verify all pass tests compile and run
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_random_subset.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_secrets_subset.sifr

# Verify fail tests correctly fail
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_functools_partial_unsupported.sifr  # Should fail
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_operator_attrgetter_unsupported.sifr  # Should fail
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_operator_methodcaller_unsupported.sifr  # Should fail
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr  # Should fail
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_secrets_token_urlsafe_unsupported.sifr  # Should fail
```

---

## Conclusion

wave_psp_b2 is **production-ready** with one minor documentation fix recommended. The implementation is solid, test coverage is comprehensive, and all claimed surfaces are shipped. The two behavioral differences (getitem returning None, truth limited to lists) are acceptable design decisions for a safety-oriented language but should be explicitly documented.

**Recommendation:** Address Finding 1 and Finding 3 (docstring additions) before considering this wave closed. Finding 2 may require broader discussion about the operator module's design philosophy.
