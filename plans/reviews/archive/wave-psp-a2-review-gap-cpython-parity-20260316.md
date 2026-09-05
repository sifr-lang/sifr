# wave_psp_a2 Review: Implementation Gaps and CPython Test Parity

**Reviewer:** agent
**Date:** 2026-03-16
**Wave:** `wave_psp_a2` (milestone_psp_2)
**Branch:** Merged to main (commits f685e4847, 9049349ac, 2cd04f085)

---

## Executive Summary

wave_psp_a2 implements core object model method argument normalization for `list`, `dict`, `set`, `tuple`, and `str`. The implementation is complete and merged to main. However, there are several actionable issues related to test coverage depth and parity documentation that should be addressed.

---

## 1. Implementation Status

### Completed Features (per traceability matrix)

| Surface | Feature | State | Status |
|---------|---------|-------|--------|
| `list` | `pop(index)` | Adapted | ✅ Implemented |
| `list` | `index(value, start, stop)` | Adapted | ✅ Implemented |
| `list` | Unexpected keyword rejection | Adapted | ✅ Implemented |
| `dict` | `update(**kwargs/iterable)` | Adapted | ✅ Implemented |
| `dict` | `pop(key, default)` | Adapted | ✅ Implemented |
| `dict` | Duplicate default detection | Adapted | ✅ Implemented |
| `set` | `update(*iterables)` | Adapted | ✅ Implemented |
| `set` | `intersection(*iterables)` | Adapted | ✅ Implemented |
| `set` | `difference_update(*iterables)` | Adapted | ✅ Implemented |
| `set` | Non-iterable argument rejection | Adapted | ✅ Implemented |
| `tuple` | `count(value)` | Adapted | ✅ Implemented |
| `tuple` | `index(value, start)` | Adapted | ✅ Implemented |
| `tuple` | Bound typing | Adapted | ✅ Implemented |
| `str` | `split(sep, maxsplit)` | Adapted | ✅ Implemented |
| `str` | `replace(old, new, count)` | Adapted | ✅ Implemented |
| `bytes/bytearray` | Object model | Waived | ⚪ Not in scope |

---

## 2. Actionable Implementation Gaps

### Gap 1: Insufficient Edge Case Coverage in Pass Tests

**Severity:** Medium

The pass test file `phase_psp_a2_core_object_model_surface.sifr` contains 38 lines of test code. While it covers the basic functionality, it lacks comprehensive edge case testing:

- **Missing edge cases for `list.pop()`:**
  - `pop()` without argument (should return last element)
  - `pop(-1)` (negative index handling)
  - `pop()` on empty list (should return `None`)

- **Missing edge cases for `list.index()`:**
  - Index not found scenario (should return `None`)
  - `start > stop` behavior

- **Missing edge cases for `dict.update()`:**
  - Multiple kwargs: `dict.update(a=1, b=2, c=3)`
  - Mixed iterable and kwargs: `dict.update([("a", 1)], b=2, c=3)`

- **Missing edge cases for `str.split()`:**
  - `split()` without arguments (whitespace splitting)
  - `maxsplit=0` behavior
  - Empty string split

**Recommended Action:** Expand test coverage in `phase_psp_a2_core_object_model_surface.sifr` to include these edge cases.

---

### Gap 2: Missing Error Message Contract Tests

**Severity:** Medium

The fail tests in `crates/sifr/tests/e2e/fail/phase_psp_a2_*.sifr` verify that errors are raised, but there are no `.txt` expectation files to verify the actual error messages match expected patterns.

Currently, there is no verification that:
- Error messages contain the expected method name
- Error messages follow a consistent format
- The error types are appropriate

**Recommended Action:** Add `.txt` expectation files for fail tests to verify error message contracts, or document that error message content is intentionally not part of the parity contract.

---

### Gap 3: Keyword Adaptation Not Enforced by Tests

**Severity:** Low

The traceability document states that Sifr accepts keyword arguments for methods that are positional-only in CPython (e.g., `list.index(value, start=1)` and `dict.pop(key, default=7)`). However, there are no tests verifying that:

1. These keyword forms actually work as expected
2. CPython's positional-only forms still work (for cross-language compatibility)

**Recommended Action:** Add explicit tests for both keyword and positional forms to document the adaptation behavior.

---

## 3. CPython Test Parity Quality Assessment

### Coverage Fidelity

| Category | Assessment |
|----------|------------|
| Adopted surfaces | Full coverage of basic functionality |
| Adapted surfaces | Coverage present but thin on edge cases |
| Waived surfaces | Properly documented (bytes/bytearray) |

### Test Quality Issues

1. **Test file size is small:** The main pass test is only 38 lines. CPython's `test_list.py` has thousands of lines of test cases.

2. **No CPython-derived test vectors:** The tests are hand-written Sifr code rather than being derived from specific CPython test functions. This reduces confidence that the implementation matches CPython behavior.

3. **Missing consolidated CPython subset test:** While `cpython_core_object_model_subset.sifr` exists, it appears to be a minimal subset and may not exercise all the parity behaviors documented in the traceability matrix.

### Local Tests Enforcement

The local tests (`phase_psp_a2_*.sifr`) do enforce the claimed parity:

- ✅ Pass tests verify correct output for valid inputs
- ✅ Fail tests verify compile-time rejection of invalid inputs
- ✅ The demo file (`demos/wave_psp_a2_core_object_models_demo.sifr`) runs successfully

However, the depth of testing is insufficient to claim true CPython test parity.

---

## 4. Documentation Issues

### Issue 1: Traceability Matrix References Non-Existent Files

The traceability document references `crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr` which exists, but some of the test cases described may not be present in the file.

### Issue 2: Previous Review Acknowledged Disk Space Issue

The previous review (pass 1) noted:
> "The verification hardening suite encountered a disk space issue during this review, but this is an environment issue unrelated to the wave implementation."

This suggests the verification hardening suite was not run for this wave. This should be re-run to ensure full coverage.

---

## 5. Recommendations

### Must Fix (Before Close)

1. **Expand edge case coverage:** Add more test cases to `phase_psp_a2_core_object_model_surface.sifr` to cover:
   - Empty collection handling
   - Negative index handling
   - Boundary conditions (start=0, stop=len, etc.)

2. **Run verification hardening suite:** Re-run the full verification hardening suite to ensure no regressions.

### Should Consider

3. **Add error message expectations:** Create `.txt` files for fail tests or document the error message contract.

4. **Document keyword adaptation clearly:** Ensure the adaptation behavior (accepting keywords where CPython uses positional-only) is clearly documented in user-facing docs.

5. **Consider adding CPython-derived test vectors:** For higher confidence, consider adding tests derived from specific CPython test functions (e.g., `test_list.py::ListTest::test_pop`, `test_list.py::ListTest::test_index`).

---

## 6. Conclusion

wave_psp_a2 is **functionally complete** and the core implementation works correctly. However, there are **actionable gaps** in:

1. Test coverage depth (edge cases)
2. Error message contract verification
3. Complete verification hardening suite run

The implementation correctly follows the adopt/adapt/waive pattern with proper documentation of semantic differences. The primary concern is that the test suite is relatively shallow compared to CPython's test coverage, which could miss subtle behavioral differences.

**Verdict:** Implementation is complete but test coverage should be expanded before considering the wave fully hardened.
