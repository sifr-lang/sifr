# Wave PSP-A2 Review: Gap Analysis and CPython Test Parity Quality

**Reviewer:** agent
**Date:** 2026-03-16
**Wave:** `wave_psp_a2` (Core Object Models: list/dict/set/tuple/str)
**Branch:** main
**Status:** Current State Assessment

---

## Executive Summary

Wave PSP-A2 implements core object model method argument normalization and builtin semantics for `list`, `dict`, `set`, `tuple`, and `str`. The implementation is functional and tests pass. However, there are actionable gaps in both implementation completeness and test parity fidelity that should be addressed.

---

## 1. Implementation Gaps (Actionable)

### 1.1 Missing Methods

| Method | Type | Status | Notes |
|--------|------|--------|-------|
| `dict.setdefault(key, default)` | dict | **NOT IMPLEMENTED** | Critical gap - this is a common dict operation |
| `list.remove(value)` with `start`/`stop` | list | **PARTIAL** | Accepts only value argument, unlike Python which supports start/stop bounds via indexing semantics |

### 1.2 Keyword Argument Normalization Gaps

The wave implements keyword argument normalization for certain methods but not consistently:

| Method | Keywords Normalized | Missing |
|--------|-------------------|---------|
| `list.index` | `start=`, `stop=` | ✅ Complete |
| `list.pop` | (positional only) | No keyword support (acceptable) |
| `dict.get` | `default=` | ✅ Complete |
| `dict.pop` | `default=` | ✅ Complete |
| `dict.update` | `**kwargs` | ✅ Complete |
| `tuple.index` | `start=` | Missing `stop=` keyword |
| `str.split` | `sep=`, `maxsplit=` | ✅ Complete |
| `str.replace` | `count=` | ✅ Complete |
| `set.update` | (variadic) | ✅ Complete |
| `set.intersection` | (variadic) | ✅ Complete |
| `set.difference` | (variadic) | ✅ Complete |

**Actionable Issue:** `set.intersection_update()`, `set.difference_update()`, `set.symmetric_difference_update()` methods exist in codegen but keyword argument normalization is not implemented (they reject keywords at compile time rather than normalizing).

### 1.3 Type Checking Gaps

From the HIR lowering code analysis:

1. **list.extend()** - Validates iterable element type compatibility ✅
2. **dict.update()** - Validates dict/iterable pairs ✅
3. **set.update()** - Validates iterable element types ✅
4. **Missing:** No compile-time validation for `dict.setdefault()` (method doesn't exist)

---

## 2. CPython Test Parity Quality Assessment

### 2.1 Test Files Overview

| Test File | Type | Purpose | Coverage Assessment |
|-----------|------|---------|---------------------|
| `phase_psp_a2_core_object_model_surface.sifr` | Pass | Basic functionality | Surface-level only |
| `cpython_core_object_model_subset.sifr` | Pass | CPython-derived subset | Limited subset |
| `phase_psp_a2_list_unexpected_keyword.sifr` | Fail | Error detection | Single case |
| `phase_psp_a2_dict_update_invalid_pairs.sifr` | Fail | Type error | Single case |
| `phase_psp_a2_dict_get_duplicate_default.sifr` | Fail | Duplicate arg | Single case |
| `phase_psp_a2_set_update_non_iterable.sifr` | Fail | Type error | Single case |
| `phase_psp_a2_str_replace_invalid_count.sifr` | Fail | Type error | Single case |
| `phase_psp_a2_tuple_index_invalid_bound.sifr` | Fail | Type error | Single case |

### 2.2 Coverage Fidelity Issues

#### Issue 1: Limited Edge Case Testing

The CPython test suite (`Lib/test/test_list.py`, `test_dict.py`, etc.) contains extensive edge cases. The Sifr port covers only a small fraction:

**list.index CPython tests (partial):**
- ✅ Basic index finding
- ✅ start/stop bounds
- ❌ ValueError on missing (adapted to return None - documented)
- ❌ IndexError on out-of-bounds (adapted - documented)

**Missing test coverage:**
- `list.pop()` with negative indices (tested in demo but not explicit test)
- `list.pop()` with empty list edge case
- `dict.update()` with mixed iterables and kwargs
- `dict.pop()` without default on missing key (should return None)
- `set.update()` with empty iterables
- `str.split()` with various sep values (None, empty string, whitespace)

#### Issue 2: Test Execution Verification

The `cpython_core_object_model_subset.sifr` file uses `assert_bool_vector_eq` which is a Sifr test utility, NOT a direct CPython test execution. This means:

- ❌ Tests are NOT actually running against CPython
- ❌ No side-by-side comparison with CPython behavior
- ✅ Sifr-specific assertions verify Sifr behavior is self-consistent

**Actionable Issue:** There is no automated verification that Sifr's adapted behavior matches CPython's actual behavior. The "CPython-derived" label is misleading - tests are adapted Sifr tests, not ported CPython tests.

#### Issue 3: Fail Test Coverage

Current fail tests cover single error cases but don't test:

- Multiple error conditions combined
- Error message specificity (CPython vs Sifr messages)
- Edge cases that should error but don't in CPython

### 2.3 Test Adaptation Quality

The adaptations are reasonable but not fully documented:

| Adaptation | Documentation | Notes |
|------------|---------------|-------|
| Return `T \| None` instead of raising | ✅ Documented | Correct for Sifr safety |
| Keyword argument support | ✅ Documented | Sifr convenience |
| Compile-time type errors | ✅ Documented | Replaces runtime errors |

**Gap:** No test verifies that error messages match expected CPython messages (they're Sifr-generated messages).

---

## 3. Demo Validation

```
$ cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr
["core", "x", "y"]
7
true
2
2
["alpha", "beta,gamma"]
bbaa
```

✅ Demo runs successfully and produces expected output.

---

## 4. Actionable Recommendations

### High Priority

1. **Implement `dict.setdefault(key, default)`**
   - Critical missing method
   - Used extensively in Python code
   - Should accept optional default keyword

2. **Add comprehensive test suite for dict.setdefault**
   - Missing key with default
   - Existing key (should return existing value, not default)
   - Without default argument

### Medium Priority

3. **Fix tuple.index to support `stop=` keyword**
   - Currently only supports `start=` keyword
   - Inconsistent with list.index behavior

4. **Add edge case tests for list.pop()**
   - Negative index handling
   - Out-of-bounds index
   - Empty list

5. **Add edge case tests for dict operations**
   - dict.update() with mixed iterables
   - dict.pop() without default on missing key

### Low Priority

6. **Clarify test naming and documentation**
   - Rename `cpython_core_object_model_subset.sifr` to avoid implying direct CPython test port
   - Add comments explaining which tests are "CPython-derived" (adapted) vs "CPython-ported" (if any)

7. **Add error message specificity tests**
   - Verify Sifr error messages are helpful
   - Compare with CPython error messages

---

## 5. Verification Commands

```bash
# Run demo
cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr

# Run pass tests
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr

# Run fail tests (should fail with type errors)
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_*.sifr
```

---

## 6. Conclusion

Wave PSP-A2 provides a solid foundation for core object model parity but has actionable gaps:

- **Implementation gaps:** Missing `dict.setdefault()`, incomplete tuple.index keyword support
- **Test parity gaps:** Limited edge case coverage, no direct CPython test execution, fail test coverage incomplete

The core functionality (keyword argument normalization, type-safe return values, compile-time error detection) is working correctly. The recommendations above would improve completeness and testing rigor.
