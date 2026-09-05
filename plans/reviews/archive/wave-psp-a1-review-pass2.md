# wave_psp_a1 Review Pass 2: Builtin Constructors and Callable Surface

**Reviewer:** agent
**Date:** 2026-03-15
**Wave:** `wave_psp_a1` (milestone_psp_1)
**Status:** Production-ready with one confirmed bug

---

## Executive Summary

Wave PSP-A1 implements builtin constructors (`list()`, `tuple()`, `dict()`, `set()`) and callable surface (`sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `ord`, `chr`). The implementation is largely correct and all 416 tests pass.

**One critical bug was identified** in range keyword argument handling that causes CPython-incompatible behavior.

---

## Scope Verification

### Implemented Features (per traceability matrix)

| Surface | Status | Notes |
|---------|--------|-------|
| `list()` / `list(iterable)` | ✅ Adopted | Empty and iterable-backed constructor works |
| `list(sequence=...)` | ✅ Adapted | Rejects unsupported keyword at compile time |
| `tuple()` / `tuple(list/tuple/str literal)` | ✅ Adapted | Fixed-length typed values |
| `tuple(dynamic_iterable)` | ⚠️ Waived | Explicitly documented as waived |
| `dict()` / `dict(iterable)` / `dict(**keywords)` | ✅ Adapted | All forms work |
| `ord()` | ✅ Adapted | Literal folding + Result type for variables |
| `chr()` | ✅ Adapted | Literal folding + Result type for variables |
| `sorted(iterable, key, reverse)` | ✅ Adopted | Full keyword support |
| `reversed(sequence)` | ✅ Adapted | Materializes as `list[T]` |
| `enumerate(iterable, start)` | ✅ Adopted | Both positional and keyword |
| `zip(*iterables)` | ✅ Adopted | Variadic arity supported |
| `zip(..., strict=True)` | ⚠️ Waived | Explicitly deferred |
| `map(func, *iterables)` | ✅ Adopted | Callable arity validation |
| `map(..., strict=True)` | ⚠️ Waived | Explicitly deferred |
| `range(start, stop, step)` | 🔴 Bug | Keyword args incorrectly accepted |

---

## Verification Results

### Demo Validation

```bash
$ cargo run -q -p sifr -- run demos/wave_psp_a1_builtin_callable_surface_demo.sifr
=== constructors ===
["s", "i", "f", "r"]
(1, 2, 3)
{"demo": 2, "compiler": 1}
=== helpers ===
[1, 2, 3]
[3, 2, 1]
[3, 2, 1]
["r", "f", "i", "s"]
[(10, "a"), (11, "b")]
[(1, "a", true), (2, "b", false)]
[5, 7, 9]
[2, 5, 8]
=== ord/chr ===
65
B
```

### Test Suite Results

```
416 pass tests completed (416 passed, 0 failed)
verification ok: variants=64, failures=0, blocking_failures=0
```

### Fail Tests Correctly Detect Errors

| Test | Expected Error | Status |
|------|----------------|--------|
| `phase_psp_a1_sorted_unexpected_keyword.sifr` | `sorted() got an unexpected keyword argument 'bogus'` | ✅ |
| `phase_psp_a1_map_callable_arity_mismatch.sifr` | `map() callable expects 1 argument(s), got 2 iterable(s)` | ✅ |
| `phase_psp_a1_tuple_dynamic_list_shape.sifr` | `tuple() currently requires a tuple, list literal, or string literal` | ✅ |

---

## Identified Issues

### Bug 1: Range Keyword Arguments Incorrectly Accepted (High Severity)

**Issue:** `range()` incorrectly accepts keyword arguments (`start=`, `stop=`, `step=`) when CPython rejects them entirely.

**Evidence:**

```python
# CPython behavior:
>>> list(range(10, stop=20))
TypeError: range() takes no keyword arguments

>>> list(range(start=1, stop=10, step=2))
TypeError: range() takes no keyword arguments

# Sifr current behavior (BUG):
>>> list(range(10, stop=20))
[10, 11, 12, 13, 14, 15, 16, 17, 18, 19]  # Silently accepts and uses keyword!

>>> list(range(start=1, stop=10, step=2))
[1, 3, 5, 7, 9]  # Silently accepts keywords!
```

**Location:** `crates/sifr_hir/src/lower/builtin_calls.rs:581-699`

**Root Cause:**

1. The first loop (lines 594-601) assigns positional arguments by index:
   - `args[0]` → `start_expr`
   - `args[1]` → `stop_expr`
   - `args[2]` → `step_expr`

2. The keyword loop (lines 603-646) checks if a keyword's variable is already set:
   - `if start_expr.is_some() { error }` - only catches duplicate keywords
   - But doesn't check semantic validity (e.g., single positional should be `stop`, not `start`)

3. The conversion logic (lines 648-651) tries to handle single-arg case:
   ```rust
   (Some(stop), None, None) => (None, Some(stop), None),
   ```
   This correctly converts a single positional to `stop`, BUT this only applies when NO keywords are present.

**Recommended Fix:**

Option A (Recommended - full CPython parity):
- Reject all keyword arguments to `range()` entirely, matching CPython's behavior
- Simplifies implementation and ensures full compatibility

Option B (Current semantics with better validation):
- Add validation in keyword loop: if exactly 1 positional AND keyword `stop` is provided, error
- Track that single positional was converted to `stop`, then reject keyword `stop`

---

## Edge Case Testing

| Test Case | Expected | Result |
|-----------|----------|--------|
| `sorted()` without args | Error | ✅ Error: "missing required argument 'iterable'" |
| `enumerate()` without args | Error | ✅ Error: "takes 1 or 2 arguments" |
| `zip()` without args | Empty list | ✅ Returns `[]` |
| `map()` without iterables | Error | ✅ Error: "takes a callable followed by at least one iterable" |
| `reversed(42)` | Error | ✅ Error: "must be an iterable" |
| `range(10)` | [0..9] | ✅ Returns [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] |
| `range(5, 10)` | [5..9] | ✅ Returns [5, 6, 7, 8, 9] |
| `range(1, 7, 2)` | [1, 3, 5] | ✅ Returns [1, 3, 5] |
| `range(start=1, stop=7, step=2)` | TypeError | 🔴 Returns [1, 3, 5] (should error) |
| `range(10, stop=20)` | TypeError | 🔴 Returns [10,...,19] (should error) |
| `ord("A")` (literal) | 65 | ✅ Constant folds to 65 |
| `ord(variable)` | Result[int] | ✅ Returns Ok(codepoint) |
| `chr(65)` (literal) | "A" | ✅ Constant folds to "A" |
| `chr(variable)` | Result[str] | ✅ Returns Ok(char) |
| `dict()` without args | Empty dict | ⚠️ Requires type annotation |
| `list()` without args | Empty list | ⚠️ Requires type annotation |
| `tuple()` without args | Empty tuple | ⚠️ Requires type annotation |

---

## Pre-existing Issue (Not Introduced by This Wave)

### Empty List Literal Codegen

**Issue:** Using an empty list literal `[]` as an argument to functions like `zip()` causes a codegen error.

```python
# This fails to compile:
result = zip([], [1, 2])
```

**Workaround:**
```python
empty: list[int] = []
result = zip(empty, [1, 2])  # Works
```

**Note:** This is a pre-existing codegen limitation unrelated to wave_psp_a1.

---

## Code Quality

### Maintainability
- ✅ All lint checks pass: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`
- ✅ HIR maintainability guardrails pass: `python3 scripts/check_hir_mainability_guardrails.py`
- ✅ No monolithic files created by this wave

### Architecture
- HIR lowering: `crates/sifr_hir/src/lower/builtin_calls.rs`, `crates/sifr_hir/src/lower/expressions.rs`
- Codegen: `crates/sifr_codegen/src/lower_expr.rs`, `crates/sifr_codegen/src/stmt_support_emitter.rs`
- Clear separation between type checking (HIR) and code generation (codegen)

---

## Regression Coverage

### Pass Tests
- `crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr` - Comprehensive coverage

### Fail Tests
- `phase_psp_a1_sorted_unexpected_keyword.sifr` - Validates keyword rejection
- `phase_psp_a1_map_callable_arity_mismatch.sifr` - Validates arity checking
- `phase_psp_a1_tuple_dynamic_list_shape.sifr` - Validates tuple type restrictions

---

## Production Readiness Assessment

| Aspect | Status | Notes |
|--------|--------|-------|
| Correctness | ⚠️ | One bug in range keyword handling |
| Robustness | ✅ | Proper error handling for most edge cases |
| Edge cases | ⚠️ | Range keyword args not handled correctly |
| Test coverage | ✅ | 416 tests pass |
| CPython parity | ⚠️ | Range diverges from CPython |

---

## Recommendations

### Must Fix Before Production

1. **Fix range keyword arguments bug**
   - Recommended: Reject all keyword arguments to `range()` to match CPython
   - Alternative: Properly validate semantic correctness of positional + keyword combinations

### Documentation Updates Needed

1. Document the waiver for `range()` keyword arguments if maintaining current behavior
2. Document the empty list literal limitation as a known pre-existing issue

### Future Considerations

1. Consider type inference for empty constructors (`dict()`, `list()`, `tuple()`) without type context
2. Add explicit error messages for the range keyword rejection (when fixed)

---

## Conclusion

**Verdict:** PRODUCTION-READY WITH ONE BUG TO FIX

The wave implementation is largely correct and provides good Python parity for builtin constructors and callable surfaces. The identified bug in range keyword argument handling should be addressed before considering this wave fully production-ready, as it represents a semantic divergence from CPython that could cause confusion.

All core functionality works correctly:
- ✅ list(), tuple(), dict() constructors
- ✅ sorted(), reversed(), enumerate(), zip(), map()
- ✅ ord() and chr() with proper Result handling
- ✅ Proper error messages for invalid usage

The explicit waivers for `tuple(dynamic_iterable)`, `zip(strict=True)`, and `map(strict=True)` are appropriately documented and do not represent accidental parity gaps.

---

## Summary for Next Steps

1. **Fix the range keyword collision bug** - Reject all keyword arguments to `range()`
2. **Re-run tests** - Verify fix doesn't break existing functionality
3. **Update documentation** - Document any remaining divergences
4. **Mark as production-ready** - After fix, wave_psp_a1 can be considered production-grade
