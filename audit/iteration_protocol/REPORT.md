# Iteration Protocol Audit Report

**1 PASS / 4 FAIL** out of 5 tests.

## Issues Found

### Issue 1: Cannot Iterate Over Strings
**Test:** 01 | `for ch in "abc":` gives `cannot iterate over type 'str'`. Python allows character-by-character iteration over strings.

### Issue 2: Cannot Iterate Over Dicts
**Test:** 01 | `for key in d:` gives `cannot iterate over type 'dict[str, int]'`. Python iterates over dict keys by default.

### Issue 3: Tuple Unpacking in For Loops Not Supported
**Test:** 03 | `for name, val in pairs:` gives `for loop target must be a simple name`. Cannot destructure tuples in for loop headers. This blocks `for k, v in dict.items()` and `for i, x in enumerate(list)` patterns.

### Issue 4: Comprehension Over `range()` Fails (Known)
**Test:** 05 | `[x * x for x in range(5)]` gives `cannot iterate over type 'range'`. Already documented in python_basics audit.

### Issue 5: Generator with Tuple Swap Hangs
**Test:** 04 | `fibonacci` generator using `a, b = b, a + b` inside a while loop causes an infinite loop or hang at runtime. The first two generators work fine.

## What Works
- `enumerate()`, `zip()`, `reversed()`, `sorted()` all work correctly
- Basic generators with `yield` work
- For loops over lists and ranges work
