# wave_psp_a1 Review: CPython Parity Gap Analysis

**Reviewer:** Claude Code
**Date:** 2026-03-16
**Wave:** `wave_psp_a1` (milestone_psp_1 / ad-hoc Python source parity)
**Status:** PRODUCTION-READY - No actionable implementation gaps remaining

---

## Executive Summary

wave_psp_a1 implements builtin constructors (`list()`, `tuple()`, `dict()`, `set()`) and callable surface (`sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `ord`, `chr`). The implementation is complete and all tests pass. **The bug previously identified in review passes 1 and 2 (range keyword argument handling) has been fixed.**

---

## 1. Implementation Gap Assessment

### Previously Identified Bug (NOW FIXED)

**Bug: Range Keyword Argument Collision**
- Status: **FIXED** ✅
- Location: `crates/sifr_hir/src/lower/builtin_calls.rs:581-699`
- Verification:
  ```bash
  $ cargo run -q -p sifr -- run /tmp/test_range_kw.sifr
  type error: range(): 'stop' was provided both positionally and as a keyword
  ```

### Current Implementation Status

| Surface | Implementation Status | Notes |
|---------|----------------------|-------|
| `list()` / `list(iterable)` | ✅ Complete | Empty and iterable-backed constructor works |
| `list(sequence=...)` | ✅ Complete | Keyword rejection works correctly |
| `tuple()` / `tuple(list/tuple/str literal)` | ✅ Complete | Fixed-length typed values |
| `tuple(dynamic_iterable)` | ✅ Waived | Explicitly documented as waived |
| `dict()` / `dict(iterable)` / `dict(**keywords)` | ✅ Complete | All forms work |
| `ord()` | ✅ Complete | Literal folding + Result type |
| `chr()` | ✅ Complete | Literal folding + Result type |
| `sorted(iterable, key, reverse)` | ✅ Complete | Full keyword support |
| `reversed(sequence)` | ✅ Complete | Materializes as `list[T]` |
| `enumerate(iterable, start)` | ✅ Complete | Both positional and keyword |
| `zip(*iterables)` | ✅ Complete | Variadic arity supported |
| `zip(..., strict=True)` | ✅ Waived | Explicitly deferred |
| `map(func, *iterables)` | ✅ Complete | Callable arity validation |
| `map(..., strict=True)` | ✅ Waived | Explicitly deferred |
| `range(start, stop, step)` | ✅ Complete | Adapted - keywords accepted |

### No Actionable Gaps

All surfaces from the traceability matrix are implemented. The explicit waivers are:
1. `tuple(dynamic_iterable)` - Sifr tuples are fixed-length typed values
2. `zip(..., strict=True)` - Deferred with iterator family parity work
3. `map(..., strict=True)` - Deferred with iterator family parity work

---

## 2. CPython Test Parity Quality

### Traceability Matrix Coverage

| Surface | CPython Source | State | Local Evidence | Parity Enforcement |
|---------|---------------|-------|----------------|-------------------|
| `list()` / `list(iterable)` | test_list.py:15-24 | adopted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `list(sequence=...)` | test_list.py:51 | adapted | builtin_calls.rs | ✅ Compile-time rejection |
| `tuple()` / `tuple(...)` | test_tuple.py:30-38 | adapted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `tuple(dynamic_iterable)` | test_tuple.py:34-38 | waived | phase_psp_a1_tuple_dynamic_list_shape.sifr | ✅ Fail test |
| `dict()` / `dict(...)` | test_dict.py:37, 382-389 | adapted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `ord()` | test_builtin.py:1714-1739 | adapted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Result type |
| `chr()` | test_builtin.py:1727-1739 | adapted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Result type |
| `sorted(...)` | test_builtin.py:2771-2793 | adopted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `reversed(sequence)` | test_list.py:185-214 | adapted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `enumerate(...)` | test_builtin.py:2157-2158 | adopted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `zip(*iterables)` | test_builtin.py:2125-2140 | adopted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `zip(..., strict=True)` | test_builtin.py:2181-2286 | waived | traceability.md | ✅ Explicit waiver |
| `map(func, *iterables)` | test_builtin.py:1323-1355 | adopted | phase_psp_a1_builtin_callable_surface.sifr | ✅ Full |
| `map(..., strict=True)` | test_builtin.py:1395-1504 | waived | traceability.md | ✅ Explicit waiver |
| `range(start, stop, step)` | test_range.py:46-52 | adapted | phase_psp_a1_builtin_callable_surface.sifr | ⚠️ Keywords accepted |

### Test Coverage Verification

**Pass Tests:**
- `crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr` - Comprehensive coverage
- `crates/sifr/tests/e2e/pass/cpython_builtins_subset.sifr` - CPython-derived subset fixture

**Fail Tests:**
- `phase_psp_a1_sorted_unexpected_keyword.sifr` - ✅ Validates keyword rejection
- `phase_psp_a1_map_callable_arity_mismatch.sifr` - ✅ Validates arity checking
- `phase_psp_a1_tuple_dynamic_list_shape.sifr` - ✅ Validates tuple restrictions
- `phase_psp_a1_range_duplicate_stop_keyword.sifr` - ✅ Validates range keyword collision

### Local Tests Enforce Claimed Parity

| Claim | Evidence | Status |
|-------|----------|--------|
| list() keyword rejected | `list(sequence=[1,2,3])` errors | ✅ Verified |
| range keyword collision detected | `range(10, stop=20)` errors | ✅ Verified |
| range keywords work | `range(start=1, stop=7, step=2)` returns [1,3,5] | ✅ Verified |
| zip strict rejected | `zip([1,2], strict=True)` errors | ✅ Verified |
| map arity validated | `map(inc, [1,2], [3,4])` with 1-arg function errors | ✅ Verified |

---

## 3. Verified Adaptations (Documented Divergences)

The following are intentional adaptations documented in the traceability matrix:

1. **range() keyword arguments** - CPython rejects all keywords (`TypeError: range() takes no keyword arguments`), but Sifr accepts `start=`, `stop=`, `step=` keywords as a typed ergonomics adaptation. This is marked as "adapted" in the traceability matrix.

2. **tuple(dynamic_iterable)** - Sifr requires tuple literals or string literals. Dynamic list-to-tuple conversion requires explicit constructor.

3. **zip/map strict=True** - Not yet implemented, explicitly waived.

---

## 4. Test Results

```
$ scripts/run_all_tests.sh --profile quick
24 pass tests completed (24 passed, 0 failed)
verification ok: variants=64, failures=0, blocking_failures=0
```

---

## 5. Conclusion

**Verdict: PRODUCTION-READY**

wave_psp_a1 is complete with no actionable implementation gaps:

1. ✅ All surfaces from the traceability matrix are implemented
2. ✅ The range keyword collision bug has been fixed
3. ✅ All pass and fail tests work correctly
4. ✅ Local tests enforce claimed parity
5. ✅ Explicit waivers are properly documented

The intentional adaptation for `range()` keyword arguments (accepting keywords that CPython rejects) is documented and appropriate for Sifr's typed ergonomics goals.

---

## Recommendations

None - the wave is production-ready.
