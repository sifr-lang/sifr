# CPython Test Suite Portability to Sifr — Research & Findings

Date: 2026-02-16
Source: Compared Sifr stdlib tests against CPython test suite at `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/`

---

## Executive Summary

The CPython test suite is a massive, decades-old corpus of behavioral tests that can significantly benefit Sifr's stdlib quality. **50-65% of CPython test assertions are directly portable** to Sifr with a mechanical rewrite — no automated translator needed. The remaining ~35% tests Python-specific dynamic features (dunder protocols, subclassing builtins, pickling) that don't apply to Sifr and should be intentionally skipped.

The current Sifr stdlib has **~145 test assertions across 44 E2E test files**. The comparable CPython test files contain **~2,400+ assertions** for the same modules. Porting even the directly-applicable subset would increase Sifr's test coverage by **5-8x**.

Two small additions to `sifr.test` — `assert_almost_eq` for float comparison and `assert_raises` for error cases — would unlock the vast majority of portable tests.

---

## Current State: Sifr vs CPython Test Depth

### Sifr Stdlib Tests (as of 2026-02-16)

- **44 E2E test files** in `crates/sifr/tests/e2e/pass/stdlib_*.sifr`
- **~145 total `# expect-stdout:` assertions** across all files
- **17 assertions** use `assert_eq`/`assert_true` (only in `stdlib_test.sifr` and `stdlib_import_test.sifr`)
- Tests verify basic functionality with 2-9 assertions per module
- No edge case coverage (INF, NAN, empty inputs, boundary values)
- No float tolerance testing
- No error/exception testing

### CPython Test Suite (comparable modules)

| CPython Test File | Test Methods | Assertions | Lines |
|---|---|---|---|
| `test_math.py` | 40 | ~1,075 | 2,857 |
| `test_builtin.py` | 133 | ~781 | 3,059 |
| `test_statistics.py` | 233 | ~527 | 3,336 |
| `test_json/` (19 files) | ~80+ | ~300+ | ~3,000+ |
| `test_int.py` | 42 | ~200+ | 939 |
| `test_float.py` | 54 | ~250+ | 1,603 |
| `test_list.py` | 24+ | ~150+ | 387 |
| `test_dict.py` | 87 | ~400+ | 1,729 |
| `test_set.py` | 60+ | ~300+ | 1,500+ |
| `test_bool.py` | 20+ | ~100+ | 426 |
| `test_re.py` | 50+ | ~300+ | 2,000+ |
| `test_string.py` | 30+ | ~150+ | 1,000+ |

**Ratio: Sifr has ~145 assertions where CPython has ~4,500+ for the same functional surface.**

---

## CPython Test Patterns — Portability Analysis

### Pattern 1: Pure Value Assertions — DIRECTLY PORTABLE

**~40-50% of all CPython test lines.** These are function calls with literal arguments compared to expected values.

CPython:
```python
self.assertEqual(math.sqrt(16), 4.0)
self.assertEqual(math.gcd(0, 0), 0)
self.assertEqual(math.copysign(1, 42), 1.0)
self.assertEqual(math.lcm(120, 84), 840)
self.assertTrue(math.isnan(math.acos(float('nan'))))
self.assertFalse(math.isinf(42.0))
```

Sifr equivalent (no translator needed):
```python
from sifr.math import sqrt, copysign, isnan, isinf, acos, nan
from sifr.test import assert_eq, assert_true, assert_false

def main():
    assert_eq(sqrt(16.0), 4.0)
    assert_eq(copysign(1.0, 42.0), 1.0)
    assert_true(isnan(acos(nan)))
    assert_false(isinf(42.0))
    print("all math tests passed")
# expect-stdout: all math tests passed
```

**Verdict:** Mechanical 1:1 rewrite. `assertEqual` → `assert_eq`, `assertTrue` → `assert_true`, `assertFalse` → `assert_false`.

### Pattern 2: Float Tolerance Assertions — PORTABLE WITH ONE ADDITION

**~10-15% of math/statistics test lines.** CPython uses custom `ftest()` with ULP tolerance and `assertAlmostEqual`.

CPython:
```python
self.ftest('acos(-1)', math.acos(-1), math.pi)
self.ftest('cos(pi/2)', math.cos(math.pi/2), 0, abs_tol=math.ulp(1))
self.assertAlmostEqual(statistics.mean([1, 2, 3]), 2.0)
```

Sifr equivalent (needs `assert_almost_eq`):
```python
from sifr.math import acos, cos, pi
from sifr.test import assert_almost_eq

def main():
    assert_almost_eq(acos(-1.0), pi, 0.00001)
    assert_almost_eq(cos(pi / 2.0), 0.0, 0.0000001)
```

**Blocker:** Sifr needs `assert_almost_eq(actual: float, expected: float, tolerance: float) -> None` added to `_sifr.test`. This is a single intrinsic addition — codegen emits `assert!((actual - expected).abs() < tolerance)`.

### Pattern 3: Data-Driven Loop Tests — DIRECTLY PORTABLE

**~10-15% of test lines.** CPython iterates over lists of test vectors.

CPython:
```python
for n in [10**5, 10**10, 10**20, 10**40]:
    self.assertEqual(math.ldexp(INF, -n), INF)
    self.assertEqual(math.ldexp(1., -n), 0.)

test_values = [
    ([], 0.0),
    ([0.0], 0.0),
    ([1e100, 1.0, -1e100, 1e-100, 1e50, -1.0, -1e50], 1e-100),
]
for vals, expected in test_values:
    actual = math.fsum(vals)
    self.assertEqual(actual, expected)
```

Sifr equivalent:
```python
from sifr.math import inf, isnan
from sifr.test import assert_eq, assert_true

def main():
    values: list[float] = [0.0, 1.0, -1.0, 3.14, -3.14]
    for v in values:
        assert_eq(v + 0.0, v)

    # Test NAN propagation
    assert_true(isnan(nan + 1.0))
    assert_true(isnan(nan * 0.0))
```

**Verdict:** Sifr supports `for` loops, lists, tuple unpacking, and list comprehensions. These port directly.

### Pattern 4: Exception/Error Assertions — PARTIALLY PORTABLE

**~15-20% of test lines.** Two subcategories:

#### 4a. TypeError assertions (~half) — SKIP, COMPILER HANDLES THESE

CPython:
```python
self.assertRaises(TypeError, math.sqrt)         # missing arg
self.assertRaises(TypeError, math.sqrt, "spam")  # wrong type
self.assertRaises(TypeError, math.ceil, 1, 2)    # too many args
```

**Sifr doesn't need these.** The compiler catches type errors and wrong argument counts at compile time. These tests validate CPython's runtime type checking — Sifr's type system makes them redundant. They could become E2E *fail* tests (verifying the compiler rejects them), but that's a different kind of test.

#### 4b. ValueError/OverflowError assertions (~half) — PORTABLE WITH ADDITION

CPython:
```python
self.assertRaises(ValueError, math.sqrt, -1)
self.assertRaises(ValueError, math.acos, 2.0)
self.assertRaises(OverflowError, math.exp, 1000000)
```

These test runtime domain errors. Portability depends on how Sifr handles these:
- If the intrinsic panics (current `.unwrap()` behavior): could test with `assert` that the result is NAN/INF instead
- If Sifr adds `Result[T, E]` returns for fallible math: needs `assert_raises` or Result-checking assertions

**Blocker:** Either (a) add `assert_raises` to `sifr.test`, or (b) convert these to NAN/INF checks where Rust's `f64` methods return NAN/INF instead of panicking (which is what Rust actually does for most math operations).

### Pattern 5: Dynamic/Dunder Protocol Tests — SKIP

**~15-20% of test lines.** These test Python's dynamic dispatch and are not applicable to Sifr.

CPython:
```python
# Custom __float__ protocol
self.assertEqual(math.ceil(FloatLike(+1.0)), +1.0)

# Custom __index__ protocol
self.assertEqual(lcm(MyIndexable(120), MyIndexable(84)), 840)

# Subclassing builtins
class subclass(list): pass
self.assertIs(type(u), subclass)

# Pickling
pickle.loads(pickle.dumps(iter([1, 2, 3])))

# eval() / repr() roundtrip
self.assertIs(eval(repr(False)), False)
```

**Verdict:** Skip entirely. These test CPython's object model, not mathematical/algorithmic correctness. Sifr's compiled nature means these features either don't exist or work differently by design.

### Pattern 6: Platform/Infrastructure Decorators — SKIP

**~5-10% of test lines.**

CPython:
```python
@requires_IEEE_754
@unittest.skipIf(sys.platform == 'win32' and platform.machine() in ('ARM', 'ARM64'), ...)
@cpython_only
@support.run_with_locale('LC_NUMERIC', 'en_US.UTF8')
```

**Verdict:** Skip. Sifr compiles to Rust which has well-defined IEEE 754 behavior. Platform-specific and CPython-internal tests don't apply.

---

## Portability Summary by Pattern

| Pattern | % of CPython Tests | Portable? | Sifr Requirement |
|---|---|---|---|
| Pure value assertions | ~40-50% | **Yes, directly** | Nothing — use `assert_eq`/`assert_true` |
| Float tolerance assertions | ~10-15% | **Yes, with 1 addition** | Add `assert_almost_eq` to `sifr.test` |
| Data-driven loops | ~10-15% | **Yes, directly** | Nothing — `for` loops + lists work |
| TypeError assertions | ~8-10% | **Skip** | Compiler catches these |
| ValueError/domain assertions | ~8-10% | **Partially** | Add `assert_raises` or use NAN/INF checks |
| Dynamic/dunder protocol | ~15-20% | **Skip** | Not applicable to Sifr |
| Platform/infra decorators | ~5-10% | **Skip** | Not applicable |

**Total directly portable: ~60-80% of meaningful test assertions** (excluding the ~30% that test Python-specific features).

---

## Required Additions to `sifr.test`

### 1. `assert_almost_eq(actual: float, expected: float, tolerance: float) -> None`

**Priority: Critical.** Without this, no float-returning function can be properly tested.

Intrinsic definition in `_sifr.test`:
```
assert_almost_eq(actual: float, expected: float, tolerance: float) -> None
```

Codegen:
```rust
assert!((actual - expected).abs() < tolerance,
    "assert_almost_eq failed: {} != {} (tolerance {})", actual, expected, tolerance);
```

This unlocks testing for: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `log`, `log10`, `log2`, `sqrt`, `degrees`, `radians`, `hypot`, `atan2`, `copysign`, `fmod`, and all `statistics` functions.

### 2. `assert_almost_eq` with ULP tolerance (stretch)

For maximum CPython parity, a ULP-based comparison:
```
assert_ulp_eq(actual: float, expected: float, ulp_tolerance: int) -> None
```

This matches CPython's `ftest()` helper which compares within N ULPs (Units in the Last Place). Lower priority than absolute tolerance.

### 3. `assert_raises` (optional, lower priority)

For testing runtime errors:
```
assert_raises(callable: Callable[[], None]) -> None  # asserts that callable panics
```

This is harder to implement (needs `std::panic::catch_unwind` in codegen) and lower priority. Most ValueError tests can be converted to NAN/INF checks instead, since Rust's `f64` math functions return NAN/INF for domain errors rather than panicking.

---

## Module-by-Module Porting Guide

### Tier 1: Highest ROI (port first)

These modules have the most portable tests and the biggest gap between current Sifr coverage and CPython coverage.

#### `sifr.math` — 21 assertions today → ~300+ portable from CPython

Current Sifr tests: `stdlib_math.sifr` (7 assertions), `stdlib_math_expanded.sifr` (8), `stdlib_math_trig.sifr` (6)

CPython `test_math.py`: 40 test methods, ~1,075 assertions

**Directly portable test methods (with `assert_almost_eq`):**
- `testConstants` — pi, e, tau values
- `testAcos` — `acos(-1)` = pi, `acos(0)` = pi/2, `acos(1)` = 0, NAN propagation
- `testAsin` — `asin(-1)` = -pi/2, `asin(0)` = 0, `asin(1)` = pi/2
- `testAtan` — `atan(-1)` = -pi/4, `atan(0)` = 0, `atan(1)` = pi/4, INF handling
- `testAtan2` — 40+ assertions covering all quadrants, INF, NAN, zero signs
- `testCos` — `cos(0)` = 1, `cos(pi)` = -1, NAN propagation
- `testSin` — `sin(0)` = 0, `sin(pi/2)` = 1, NAN propagation
- `testTan` — basic values, NAN propagation
- `testCosh`, `testSinh`, `testTanh` — hyperbolic functions
- `testDegrees`, `testRadians` — angle conversion
- `testFloor`, `testCeil` — basic integer rounding
- `testTrunc` — truncation
- `testCopysign` — sign copying with INF, NAN, zero
- `testFmod` — float modulo
- `testHypot` — Pythagorean distance
- `testIsnan`, `testIsinf` — special value detection
- `testLog`, `testLog2`, `testLog10` — logarithms
- `testSqrt` — square root with edge cases

**Not portable (skip):**
- `testCeil`/`testFloor` lines using `FloatLike`, `TestNoCeil` (dunder protocols)
- `testFsum` lines using `FloatLike` objects
- `testGcd`/`testLcm` lines using `MyIndexable` (custom `__index__`)
- All `assertRaises(TypeError, ...)` lines (compiler handles these)
- `testProd` lines using `Decimal`, `Fraction` types
- `testNextafter`, `testUlp` — Sifr doesn't expose these yet
- `test_testfile`, `test_mtestfile` — external test data file parsing

**Estimated portable assertions: ~300 out of ~1,075 (~28%)**
But these 300 cover the core mathematical correctness that matters.

#### `sifr.statistics` — 3 assertions today → ~100+ portable from CPython

CPython `test_statistics.py`: 233 test methods, ~527 assertions

**Directly portable (for `mean`, `median`, `variance`, `stdev`, `mode`):**
- Basic correctness: `mean([1,2,3])` = 2.0, `median([1,2,3])` = 2, etc.
- Edge cases: single-element lists, large values, negative values
- Float precision: tolerance-based comparison for variance/stdev

**Not portable:**
- `Decimal` and `Fraction` type tests (~40% of the file)
- `NormalDist` class tests
- `covariance`, `correlation`, `linear_regression` (not in Sifr yet)
- Pickling tests

**Estimated portable assertions: ~100 out of ~527 (~19%)**

#### `sifr.json` — 2 assertions today → ~50+ portable from CPython

CPython `test_json/` directory: 19 files

**Directly portable:**
- `test_decode.py`: Basic JSON parsing — strings, numbers, booleans, null, arrays, objects
- `test_pass1.py`, `test_pass2.py`, `test_pass3.py`: Valid JSON documents
- `test_fail.py`: Invalid JSON strings (if Sifr returns errors)
- `test_dump.py`: Basic serialization

**Not portable:**
- `test_speedups.py` — C extension tests
- `test_enum.py` — Python enum serialization
- `test_recursion.py` — recursive structure detection
- Custom encoder/decoder tests

**Estimated portable assertions: ~50 out of ~300+**

#### `sifr.re` — 6 assertions today → ~80+ portable from CPython

CPython `test_re.py`: 50+ test methods, ~300+ assertions

**Directly portable:**
- Basic pattern matching: `match`, `search`, `findall`, `split`, `sub`
- Character classes, quantifiers, anchors
- Simple group extraction

**Not portable:**
- Compiled pattern objects (`re.compile`)
- Flag combinations (`re.IGNORECASE | re.MULTILINE`)
- Match object methods (`.group()`, `.span()`, etc.)
- Named groups, lookahead/lookbehind
- `re.error` exception details

**Estimated portable assertions: ~80 out of ~300+**

### Tier 2: Medium ROI

#### `sifr.collections` (set operations) — 9 assertions → ~60+ portable

From CPython `test_set.py`: union, intersection, difference, symmetric_difference, membership, length, iteration.

#### `sifr.bisect` — 4 assertions → ~30+ portable

From CPython `test_bisect.py`: sorted insertion positions, boundary cases.

#### `sifr.heapq` — 4 assertions → ~30+ portable

From CPython `test_heapq.py`: heap invariant, push/pop sequences, nsmallest/nlargest.

#### `sifr.textwrap` — 5 assertions → ~40+ portable

From CPython `test_textwrap.py`: wrapping at width, dedent behavior, indent behavior.

#### `sifr.ipaddress` — 7 assertions → ~30+ portable

From CPython `test_ipaddress.py`: IPv4 validation, private/loopback/multicast classification.

#### `sifr.fnmatch` — 8 assertions → ~25+ portable

From CPython `test_fnmatch.py`: glob pattern matching, wildcard behavior.

### Tier 3: Lower ROI (fewer portable tests)

These modules have CPython tests that are heavily class-based or use features Sifr doesn't support:

- `sifr.argparse` — CPython tests are entirely class-based (`ArgumentParser`)
- `sifr.logging` — CPython tests use handler/formatter class hierarchy
- `sifr.pathlib` — CPython tests use `Path` class methods
- `sifr.datetime` — CPython tests use `datetime`/`timedelta` class arithmetic
- `sifr.csv` — CPython tests use `reader`/`writer` class objects
- `sifr.graphlib` — CPython tests use `TopologicalSorter` class

For these, the functional Sifr APIs are different enough that CPython tests serve more as **behavioral reference** than as directly portable test cases.

---

## External Test Data Files

CPython ships pre-computed test data that is framework-independent and immediately usable:

### `Lib/test/mathdata/math_testcases.txt`

Contains thousands of input/output pairs for math functions in a simple text format:
```
-- sqrt
sqrt0000 sqrt 0.0 -> 0.0
sqrt0001 sqrt 1.0 -> 1.0
sqrt0002 sqrt 4.0 -> 2.0
...
```

This file alone could generate hundreds of test assertions for `sifr.math` by parsing it and generating Sifr test code.

### `Lib/test/mathdata/cmath_testcases.txt`

Complex math test data. Not applicable to Sifr (no complex number support), but useful reference for when/if complex numbers are added.

### `Lib/test/test_json/` pass/fail JSON documents

The `test_pass*.py` and `test_fail.py` files contain curated valid and invalid JSON strings that can be used directly as test inputs for `sifr.json`.

---

## Recommended Approach

### No translator needed

The CPython tests are written in a style that maps almost 1:1 to Sifr's test pattern. The `unittest.TestCase` class structure is just boilerplate wrapping simple assertion calls. Strip the class/method wrapper and you have exactly what Sifr's `assert_eq` / `assert_true` pattern does.

A developer reading CPython tests can port the applicable lines by hand. The mechanical mapping is:

| CPython | Sifr |
|---|---|
| `self.assertEqual(a, b)` | `assert_eq(a, b)` |
| `self.assertTrue(x)` | `assert_true(x)` |
| `self.assertFalse(x)` | `assert_false(x)` |
| `self.assertNotEqual(a, b)` | `assert_ne(a, b)` |
| `self.ftest(name, got, expected)` | `assert_almost_eq(got, expected, 0.00001)` |
| `self.assertAlmostEqual(a, b)` | `assert_almost_eq(a, b, 0.0000001)` |
| `self.assertRaises(TypeError, f, x)` | *skip — compiler catches this* |
| `self.assertRaises(ValueError, f, x)` | `assert_true(isnan(f(x)))` or `assert_raises(...)` |
| `self.assertIs(a, b)` | *skip — identity not meaningful in compiled lang* |
| `self.assertIn(x, container)` | *express as boolean check* |

### Step 1: Add `assert_almost_eq` to `sifr.test`

Single intrinsic addition. Unblocks all float testing.

### Step 2: Port `test_math.py` value assertions

Start with the highest-value module. Port ~300 assertions covering all 33 exported functions. This alone would be a 14x increase in math test coverage.

### Step 3: Port `test_statistics.py` for the 5 existing functions

Port ~100 assertions for `mean`, `median`, `variance`, `stdev`, `mode`.

### Step 4: Port remaining Tier 1 modules

`json`, `re`, `collections`, `bisect`, `heapq`, `textwrap`, `ipaddress`, `fnmatch`.

### Step 5: Mine `mathdata/math_testcases.txt`

Parse the external test data file and generate comprehensive test vectors for all math functions.

---

## Estimated Impact

| Metric | Current | After Porting |
|---|---|---|
| Total stdlib test assertions | ~145 | ~800-1,000 |
| Math function coverage | 15/33 functions tested | 33/33 functions tested |
| Edge case coverage | None (no INF/NAN/boundary) | Comprehensive |
| Float precision testing | None | Full (with `assert_almost_eq`) |
| Modules with >10 assertions | 3 | 15+ |

---

## What NOT to Port (and Why)

1. **Dunder protocol tests** (`__float__`, `__index__`, `__hash__`, `__len__` overrides) — Sifr doesn't have Python's dynamic dispatch
2. **Subclassing builtin types** (`class MyInt(int)`) — Sifr's type system is different
3. **Pickling/unpickling tests** — Not applicable
4. **`eval()`/`repr()` roundtrip tests** — Sifr doesn't have `eval()`
5. **Locale-dependent tests** — Sifr compiles to Rust with fixed locale behavior
6. **GC/memory/refcount tests** — Rust's ownership model handles this
7. **`sys.maxsize` boundary tests** — Sifr uses `i64`, not arbitrary precision
8. **`Decimal`/`Fraction` type tests** — Sifr doesn't have these types
9. **CPython-internal tests** (`@cpython_only`, C extension tests) — Implementation-specific

These represent ~30-40% of CPython test lines and are correctly excluded — they test Python's runtime, not the mathematical/algorithmic behavior that Sifr shares with Python.
