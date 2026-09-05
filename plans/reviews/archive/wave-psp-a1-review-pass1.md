# Wave PSP-A1 Review: Builtin Constructors and Callable Surface

**Reviewer:** agent
**Date:** 2026-03-15
**Wave:** `wave_psp_a1` (milestone_psp_1)
**Status:** Generally complete with one identified bug

---

## Executive Summary

Wave PSP-A1 implements builtin constructors (`list()`, `tuple()`, `dict()`, `set()`) and callable surface (`sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `ord`, `chr`). The implementation is largely correct and passes all tests, with one identified bug in range keyword argument handling and one pre-existing codegen issue with empty list literals.

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
| `range(start, stop, step)` | ✅ Adapted | Keyword args supported |

---

## Verification Results

### Demo Validation

```bash
$ cargo run -q -p sifr -- run demos/wave_psp_a1_builtin_callable_surface_demo.sifr
=== constructors ===
["s", "i", "f", "r"]
(1, 2, 3)
{"compiler": 1, "demo": 2}
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

### Test Validation

All pass tests pass, and fail tests correctly detect expected errors:

| Test | Expected Error | Status |
|------|----------------|--------|
| `phase_psp_a1_sorted_unexpected_keyword.sifr` | `sorted() got an unexpected keyword argument 'bogus'` | ✅ |
| `phase_psp_a1_map_callable_arity_mismatch.sifr` | `map() callable expects 1 argument(s), got 2 iterable(s)` | ✅ |
| `phase_psp_a1_tuple_dynamic_list_shape.sifr` | `tuple() currently requires a tuple, list literal, or string literal` | ✅ |

### Full Test Suite

```bash
$ scripts/run_all_tests.sh --profile quick
# Result: All tests pass (416 pass, 0 fail)
```

---

## Identified Issues

### Bug 1: Range Positional/Keyword Argument Collision (Medium Severity)

**Issue:** When `range()` is called with a single positional argument followed by a keyword argument for the same parameter, it silently accepts the conflicting input instead of raising an error.

**Example:**
```python
# Sifr currently accepts this (BUG):
list(range(10, stop=20))  # Returns [10, 11, ..., 19]

# CPython correctly rejects:
# TypeError: range() takes no keyword arguments
```

**Location:** `crates/sifr_hir/src/lower/builtin_calls.rs:594-646`

**Root Cause:** The logic assigns the single positional argument to `start_expr` initially, then allows a keyword argument for the same parameter to override it without checking if the positional was already set.

**Recommended Fix:** Add validation to detect when a positional argument at index 0 is provided alongside a keyword argument for the first parameter (which should be `stop` when there's only one positional).

---

### Pre-existing Issue: Empty List Literal Codegen (Not introduced by this wave)

**Issue:** Using an empty list literal `[]` as an argument to functions like `zip()` causes a codegen error.

**Example:**
```python
# This fails to compile:
result = zip([], [1, 2])
```

**Workaround:** Use a typed empty list:
```python
empty: list[int] = []
result = zip(empty, [1, 2])  # Works
```

**Note:** This is a pre-existing codegen limitation unrelated to wave_psp_a1.

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
| `range(start=1, stop=7, step=2)` | [1, 3, 5] | ✅ Returns [1, 3, 5] |
| `ord("A")` (literal) | 65 | ✅ Constant folds to 65 |
| `ord(variable)` | Result[int] | ✅ Returns Ok(codepoint) |
| `chr(65)` (literal) | "A" | ✅ Constant folds to "A" |
| `chr(variable)` | Result[str] | ✅ Returns Ok(char) |
| `dict()` without args | Empty dict | ⚠️ Requires type annotation |
| `list()` without args | Empty list | ⚠️ Requires type annotation |
| `tuple()` without args | Empty tuple | ⚠️ Requires type annotation |

---

## Code Quality

### Maintainability
- All lint checks pass: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`
- HIR maintainability guardrails pass: `python3 scripts/check_hir_maintainability_guardrails.py`
- No monolithic files created by this wave

### Architecture
- HIR lowering: `crates/sifr_hir/src/lower/builtin_calls.rs`, `crates/sifr_hir/src/lower/expressions.rs`
- Codegen: `crates/sifr_codegen/src/intrinsic_method_emitters.rs`, `crates/sifr_codegen/src/lower_expr.rs`
- Clear separation between type checking (HIR) and code generation (codegen)

---

## Regression Coverage

### Pass Tests
- `crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr` - Comprehensive coverage of all implemented surfaces

### Fail Tests
- `phase_psp_a1_sorted_unexpected_keyword.sifr` - Validates keyword rejection
- `phase_psp_a1_map_callable_arity_mismatch.sifr` - Validates arity checking
- `phase_psp_a1_tuple_dynamic_list_shape.sifr` - Validates tuple type restrictions

---

## Conclusion

**Verdict:** APPROVED with one medium-severity bug to fix

The wave implementation is largely correct and provides good Python parity for builtin constructors and callable surfaces. The identified bug in range keyword argument handling should be addressed before closing the wave, but it does not block the overall milestone completion as it's an edge case.

The explicit waivers for `tuple(dynamic_iterable)`, `zip(strict=True)`, and `map(strict=True)` are appropriately documented and do not represent accidental parity gaps.

---

## Recommendations

1. **Fix the range keyword collision bug** - Should detect and reject cases like `range(10, stop=20)`
2. **Document the empty list literal limitation** - This is a pre-existing issue but should be documented for users
3. **Consider type inference for empty constructors** - `dict()`, `list()`, `tuple()` without type context could infer `dict[str, Any]` or similar defaults
