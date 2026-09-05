# Wave PSP-A2 Review: Gap Analysis and CPython Test Parity Quality

**Reviewer:** agent
**Date:** 2026-03-17
**Wave:** `wave_psp_a2` (Core Object Models: list/dict/set/tuple/str)
**Branch:** `codex/python-builtin-std-parity-wave-e2`
**Status:** Current State Assessment (R1)

---

## Executive Summary

Wave PSP-A2 implements core object model method argument normalization and builtin semantics for `list`, `dict`, `set`, `tuple`, and `str`. The implementation is functional and core tests pass. However, there are gaps in implementation completeness and some issues with the set mutation methods.

---

## 1. Implementation Gaps (Actionable)

### 1.1 Missing Methods

| Method | Type | Status | Notes |
|--------|------|--------|-------|
| `dict.setdefault(key, default)` | dict | **NOT IMPLEMENTED** | Critical gap - this is a common dict operation. Error: `type error: dict has no method 'setdefault'` |

### 1.2 Implemented but with Issues

| Method | Type | Status | Notes |
|--------|------|--------|-------|
| `set.intersection_update(iterable)` | set | **BUG** | Implemented in HIR but codegen generates non-mutable receiver. Error: `cannot borrow 's' as mutable, as it is not declared as mutable` |
| `set.difference_update(iterable)` | set | **BUG** | Same issue as intersection_update |
| `set.symmetric_difference_update(iterable)` | set | **Partial** | Recognized in HIR but generates a reassignment (`seen = seen.symmetric_difference(&__other).cloned().collect()`) instead of in-place mutation |

### 1.3 Keyword Argument Normalization Status

| Method | Keywords Normalized | Status |
|--------|-------------------|--------|
| `list.index` | `start=`, `stop=` | ✅ Complete |
| `list.pop` | (positional only) | ✅ Acceptable |
| `dict.get` | `default=` | ✅ Complete |
| `dict.pop` | `default=` | ✅ Complete |
| `dict.update` | `**kwargs` | ✅ Complete |
| `tuple.index` | `start=`, `stop=` | ✅ Complete (verified with test) |
| `str.split` | `sep=`, `maxsplit=` | ✅ Complete |
| `str.replace` | `count=` | ✅ Complete |
| `set.update` | (variadic) | ✅ Complete |
| `set.intersection` | (variadic) | ✅ Complete |
| `set.difference` | (variadic) | ✅ Complete |

---

## 2. Test Coverage Assessment

### 2.1 Test Files Overview

| Test File | Type | Status |
|-----------|------|--------|
| `phase_psp_a2_core_object_model_surface.sifr` | Pass | ✅ Running |
| `cpython_core_object_model_subset.sifr` | Pass | ✅ Running |
| `phase_psp_a2_list_unexpected_keyword.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_dict_update_invalid_pairs.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_dict_get_duplicate_default.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_set_update_non_iterable.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_str_replace_invalid_count.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_tuple_index_invalid_bound.sifr` | Fail | ✅ Detects error |

### 2.2 Demo Validation

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

## 3. CPython Test Parity Quality

### 3.1 Test Execution Verification

The `cpython_core_object_model_subset.sifr` file uses `assert_bool_vector_eq` which is a Sifr test utility. Tests are adapted Sifr tests, not directly ported CPython tests.

**Adaptations documented in traceability:**
- Return `T | None` instead of raising on misses/out-of-range (safe return)
- Keyword argument support (Sifr convenience)
- Compile-time type errors (replaces runtime errors)

### 3.2 Edge Case Coverage

| Feature | Coverage | Notes |
|---------|----------|-------|
| list.index with bounds | ✅ Tested | start/stop bounds work |
| list.pop | ✅ Tested | Negative indices handled |
| dict.update with mixed | ✅ Tested | Iterables and kwargs |
| dict.pop with default | ✅ Tested | |
| set operations | ✅ Tested | update, intersection, difference |
| tuple.index with bounds | ✅ Tested | Verified stop= works |
| str.split/replace | ✅ Tested | |

---

## 4. Codegen Analysis

### 4.1 Method Registry (mod.rs)

**List methods implemented:**
- append, extend, insert, clear, copy, reverse, sort, count, contains, pop, remove, index

**Dict methods implemented:**
- keys, values, items, update, clear, copy, contains, get, pop

**Set methods implemented:**
- add, remove, discard, contains, clear, copy, issubset, issuperset, isdisjoint, pop, union, intersection, difference, symmetric_difference

**Missing from Set (HIR recognized but not codegen):**
- intersection_update
- difference_update
- symmetric_difference_update

---

## 5. Actionable Recommendations

### High Priority

1. **Implement `dict.setdefault(key, default)`**
   - Critical missing method
   - Used extensively in Python code

2. **Fix set mutation methods codegen**
   - `set.intersection_update()` generates broken Rust code
   - `set.difference_update()` generates broken Rust code
   - Need to ensure proper mutable receiver in generated code

### Medium Priority

3. **Consistent set mutation behavior**
   - `symmetric_difference_update` works but uses reassignment pattern
   - Consider implementing proper in-place mutation for all _update methods

---

## 6. Verification Commands

```bash
# Run demo
cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr

# Run pass tests
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr

# Run fail tests (should fail with type errors)
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_*.sifr

# Test missing dict.setdefault (should fail)
echo 'def main():
    d: dict[str, int] = {"a": 1}
    d.setdefault("b", 2)' | cargo run -q -p sifr -- run -

# Test tuple.index with stop= (should work)
echo 'def main():
    t: tuple[int, int, int, int] = (1, 2, 3, 2)
    print(str(t.index(2, start=1, stop=3)))' | cargo run -q -p sifr -- run -
```

---

## 7. Conclusion

Wave PSP-A2 provides a solid foundation for core object model parity but has two main issues:

- **Critical gap:** `dict.setdefault()` is not implemented
- **Bug:** Set mutation methods (`intersection_update`, `difference_update`) generate broken Rust code

The keyword argument normalization is working correctly for all implemented methods. The test suite provides reasonable coverage for the implemented functionality.
