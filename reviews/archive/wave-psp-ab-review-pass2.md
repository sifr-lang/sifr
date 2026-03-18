# Wave PSP-AB Review Pass 2: Implementation Quality and CPython Parity Closure

**Reviewer:** Claude Code
**Date:** 2026-03-16
**Waves:** `wave_psp_a1`, `wave_psp_a2`, `wave_psp_b1`, `wave_psp_b2`
**Status:** See individual wave sections

---

## Executive Summary

This review covers four waves in the PSP-A/B series that implement CPython builtin and stdlib surface parity:

| Wave | Scope | Status | Key Issues |
|------|-------|--------|------------|
| `wave_psp_a1` | Builtin constructors (`list`, `tuple`, `dict`, `set`) and callables (`sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `ord`, `chr`) | ⚠️ Bug confirmed | `range()` incorrectly accepts keyword arguments |
| `wave_psp_a2` | Core object models (`list`, `dict`, `set`, `tuple`, `str`) method argument normalization | ✅ Approved | None |
| `wave_psp_b1` | Collections/ordered helpers (`Counter`, `deque`, `bisect`, `heapq`) | ✅ Approved | See waivers |
| `wave_psp_b2` | Iterators/functional/randomness (`itertools`, `functools`, `operator`, `random`, `secrets`) | ✅ Approved | See waivers |

---

## Wave PSP-A1: Builtin Constructors and Callable Surface

### Status: PRODUCTION-READY WITH ONE CONFIRMED BUG

### Scope Verification

| Surface | Feature | Status | Notes |
|---------|---------|--------|-------|
| `list()` / `list(iterable)` | Constructor | ✅ Adopted | Empty and iterable-backed work |
| `list(sequence=...)` | Keyword rejection | ✅ Adapted | Rejects at compile time |
| `tuple()` / `tuple(list/tuple/str literal)` | Constructor | ✅ Adapted | Fixed-length typed values |
| `tuple(dynamic_iterable)` | Dynamic conversion | ⚠️ Waived | Explicitly documented as waived |
| `dict()` / `dict(iterable)` / `dict(**keywords)` | Constructor | ✅ Adapted | All forms work |
| `ord()` | Code point | ✅ Adapted | Literal folding + Result for variables |
| `chr()` | Character | ✅ Adapted | Literal folding + Result for variables |
| `sorted(iterable, key, reverse)` | Sorting | ✅ Adopted | Full keyword support |
| `reversed(sequence)` | Reversal | ✅ Adapted | Materializes as `list[T]` |
| `enumerate(iterable, start)` | Enumeration | ✅ Adopted | Both positional and keyword |
| `zip(*iterables)` | Zipping | ✅ Adopted | Variadic arity supported |
| `zip(..., strict=True)` | Strict mode | ⚠️ Waived | Explicitly deferred |
| `map(func, *iterables)` | Mapping | ✅ Adopted | Callable arity validation |
| `map(..., strict=True)` | Strict mode | ⚠️ Waived | Explicitly deferred |
| `range(start, stop, step)` | Range | 🔴 Bug | Keyword args incorrectly accepted |

### Identified Bug

**Bug: Range Keyword Arguments Incorrectly Accepted (High Severity)**

CPython behavior:
```python
>>> list(range(10, stop=20))
TypeError: range() takes no keyword arguments

>>> list(range(start=1, stop=10, step=2))
TypeError: range() takes no keyword arguments
```

Sifr current behavior (BUG):
```python
>>> list(range(start=1, stop=10, step=2))
[1, 3, 5, 7, 9]  # Silently accepts keywords!

>>> list(range(10, stop=20))
TypeError: range(): 'stop' was provided both positionally and as a keyword  # Correctly detects this case
```

**Analysis:**
- `range(start=1, stop=10, step=2)` - All keywords: Should error but doesn't ❌
- `range(10, stop=20)` - Positional + keyword: Correctly detects duplicate ✅

The implementation correctly catches the case where `stop` is provided both positionally and as a keyword, but it incorrectly allows keyword-only arguments.

**Recommended Fix:** Reject all keyword arguments to `range()` to match CPython's behavior.

### Test Coverage

**Pass Tests:**
- `phase_psp_a1_builtin_callable_surface.sifr` - Comprehensive coverage

**Fail Tests (all working correctly):**
- `phase_psp_a1_sorted_unexpected_keyword.sifr` - ✅
- `phase_psp_a1_map_callable_arity_mismatch.sifr` - ✅
- `phase_psp_a1_tuple_dynamic_list_shape.sifr` - ✅
- `phase_psp_a1_range_duplicate_stop_keyword.sifr` - ✅

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

### Waiver Coverage

| Surface | State | Rationale |
|---------|-------|------------|
| `tuple(dynamic_iterable)` | ✅ Waived | Sifr tuples are fixed-length typed values |
| `zip(strict=True)` | ✅ Waived | Deferred with iterator-family parity |
| `map(strict=True)` | ✅ Waived | Deferred with iterator-family parity |

### Pre-existing Issue (Not Introduced by This Wave)

Empty list literal `[]` as argument to functions like `zip()` causes codegen error:
```python
# This fails to compile:
result = zip([], [1, 2])

# Workaround:
empty: list[int] = []
result = zip(empty, [1, 2])  # Works
```

---

## Wave PSP-A2: Core Object Models and Builtin Semantics

### Status: APPROVED - Implementation complete with no critical issues

### Scope Verification

| Surface | Feature | Status | Notes |
|---------|---------|--------|-------|
| `list` | `pop(index)` | ✅ Adapted | Returns `T \| None` instead of raising |
| `list` | `index(value, start, stop)` | ✅ Adapted | Returns `int \| None` instead of raising |
| `list` | `extend(iterable)` | ✅ Adapted | Type validation for iterable compatibility |
| `list` | Unexpected keyword rejection | ✅ Adapted | Compile-time rejection |
| `dict` | `update(**kwargs)` | ✅ Adapted | Keywords converted to dict literal |
| `dict` | `update(iterable)` | ✅ Adapted | Validates iterable of key/value pairs |
| `dict` | `pop(key, default)` | ✅ Adapted | Default value statically typed |
| `dict` | `get(key, default)` | ✅ Adapted | Duplicate default detection |
| `set` | `update(*iterables)` | ✅ Adapted | Variadic iterable arguments |
| `set` | `intersection(*iterables)` | ✅ Adapted | Multiple iterables |
| `set` | `difference_update(*iterables)` | ✅ Adapted | Multiple iterables |
| `set` | `symmetric_difference_update(iterable)` | ✅ Adapted | Works correctly |
| `tuple` | `count(value)` | ✅ Adapted | Correct implementation |
| `tuple` | `index(value, start)` | ✅ Adapted | Returns `int \| None` |
| `tuple` | Bound typing | ✅ Adapted | Enforces `int` type |
| `str` | `split(sep, maxsplit)` | ✅ Adapted | Both positional and keyword |
| `str` | `replace(old, new, count)` | ✅ Adapted | `count < 0` means replace all |

### Waiver Coverage

| Surface | State | Rationale |
|---------|-------|------------|
| `bytes` / `bytearray` | ✅ Waived | Sifr has no first-class bytes type |

### Test Coverage

**Pass Tests:**
- `phase_psp_a2_core_object_model_surface.sifr` - Comprehensive coverage

**Fail Tests (all working correctly):**
- `phase_psp_a2_list_unexpected_keyword.sifr` - ✅
- `phase_psp_a2_dict_update_invalid_pairs.sifr` - ✅
- `phase_psp_a2_dict_get_duplicate_default.sifr` - ✅
- `phase_psp_a2_set_update_non_iterable.sifr` - ✅
- `phase_psp_a2_str_replace_invalid_count.sifr` - ✅
- `phase_psp_a2_tuple_index_invalid_bound.sifr` - ✅

### Demo Validation
```bash
$ cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr
["core", "x", "y"]
7
true
2
2
["alpha", "beta,gamma"]
bbaa
```

---

## Wave PSP-B1: Collections and Ordered Helpers

### Status: APPROVED - Implementation complete with appropriate waivers

### Scope Verification

| Surface | Feature | Status | Notes |
|---------|---------|--------|-------|
| `Counter` | `most_common([n])` | ✅ Adapted | Typed class, not dict subclass |
| `deque` | `rotate`, `count`, `remove`, `copy`, `reverse` | ✅ Adapted | Behavior matches CPython |
| `deque` | Invalid index bound typing | ✅ Adapted | Compile-time rejection |
| `bisect` / `bisect_left` | `lo`/`hi` forms | ✅ Adapted | Clamps safely instead of raising |
| `bisect` | Unsupported `key=` | ✅ Waived | Not supported in current signature model |
| `heapq` | `heappushpop`, `heapreplace` | ✅ Adapted | Empty replacement returns `None` |
| `heapq` | Max-heap helpers | ✅ Adapted | Shipped underscore helpers |

### Waiver Coverage

| Surface | State | Rationale |
|---------|-------|------------|
| `Counter(iterable)` / `Counter(**kwargs)` | ✅ Waived | No generic constructor overloading yet |
| `defaultdict(..., default_factory=..., **kwargs)` | ✅ Waived | Only positional factory/mapping forms |
| `heapq.merge(*iterables)` | ✅ Waived | Not fully wired through import path |
| `_heappush_max` / `_heappushpop_max` | ✅ Waived | Not exported |

### Test Coverage

**Pass Tests:**
- `phase_psp_b1_collections_ordered_helpers.sifr` - Comprehensive coverage

**Fail Tests (all working correctly):**
- `phase_psp_b1_bisect_key_unsupported.sifr` - ✅
- `phase_psp_b1_counter_iterable_constructor_unsupported.sifr` - ✅
- `phase_psp_b1_deque_index_invalid_bound.sifr` - ✅

### Demo Validation
```bash
$ cargo run -q -p sifr -- run demos/wave_psp_b1_collections_ordered_helpers_demo.sifr
[("delta", 2), ("alpha", 1), ("beta", 1)]
[0, 3, 1, 2]
3
1
2
```

---

## Wave PSP-B2: Iterators, Functional, and Randomness

### Status: APPROVED - Implementation complete with appropriate waivers

### Scope Verification

| Surface | Feature | Status | Notes |
|---------|---------|--------|-------|
| `itertools.chain` | Variadic | ✅ Adapted | Eager `list[...]` materialization |
| `itertools.islice` | `start, stop, step` | ✅ Adapted | Works correctly |
| `itertools.product` | `repeat=` | ✅ Adapted | Works correctly |
| `itertools.permutations` | - | ✅ Adapted | Works correctly |
| `itertools.combinations` | - | ✅ Adapted | Works correctly |
| `itertools.combinations_with_replacement` | - | ✅ Adapted | Works correctly |
| `itertools.starmap` | - | ✅ Adapted | Works correctly |
| `functools.reduce` | Higher-order callable | ✅ Adapted | User-defined `__call__` works |
| `operator.getitem` | - | ✅ Adapted | Works correctly |
| `operator.contains` | - | ✅ Adapted | Works correctly |
| `operator.truth` | - | ✅ Adapted | Works correctly |
| `random.shuffle` | Mutating | ✅ Adapted | Returns `None`, matches CPython |
| `random.randrange` | Variants | ✅ Adapted | Empty population raises `ValueError` |
| `random.choice` | - | ✅ Adapted | Works correctly |
| `random.choices` | - | ✅ Adapted | Works correctly |
| `random.getrandbits` | - | ✅ Adapted | Works correctly |
| `secrets.compare_digest` | - ✅ Adapted | Functional for `str` inputs |
| `secrets.randbits` | - | ✅ Adapted | Works correctly |
| `secrets.choice` | - | ✅ Adapted | Works correctly |
| `secrets.randbelow` | - | ✅ Adapted | Works correctly |
| `secrets.token_hex` | - | ✅ Adapted | Works correctly |

### Waiver Coverage

| Surface | State | Rationale |
|---------|-------|------------|
| Lazy iterator objects | ✅ Waived | Requires broader lazy-iterator runtime |
| `functools.partial` | ✅ Waived | Codegen limitations |
| `cmp_to_key` | ✅ Waived | Codegen limitations |
| `operator.attrgetter` | ✅ Waived | Not available in static typing |
| `operator.methodcaller` | ✅ Waived | Not available in static typing |
| Weighted `random.choices` | ✅ Waived | No deterministic stateful generators |
| `seed`, `getstate`, `setstate` | ✅ Waived | No stateful generator objects |
| `secrets.token_urlsafe` | ✅ Waived | No bytes type |
| Bytes-oriented `compare_digest` | ✅ Waived | No bytes type |

### Test Coverage

**Pass Tests:**
- `phase_psp_b2_iterators_functional_randomness.sifr` - Comprehensive coverage

**Fail Tests:** None (no error cases defined for this wave)

### Demo Validation
```bash
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

---

## Cross-Cutting Analysis

### CPython Test Porting Assessment

| Wave | CPython Tests Reviewed | Tests Ported | Coverage |
|------|------------------------|---------------|----------|
| `wave_psp_a1` | `test_builtin.py`, `test_list.py`, `test_dict.py`, `test_tuple.py`, `test_str.py`, `test_range.py` | Yes | Comprehensive |
| `wave_psp_a2` | `test_list.py`, `test_dict.py`, `test_set.py`, `test_tuple.py`, `test_str.py` | Yes | Comprehensive |
| `wave_psp_b1` | `test_collections.py`, `test_bisect.py`, `test_heapq.py` | Yes | Comprehensive |
| `wave_psp_b2` | `test_itertools.py`, `test_functools.py`, `test_operator.py`, `test_random.py`, `test_secrets.py` | Yes | Comprehensive |

### Negative/Waiver Coverage Assessment

All four waves have appropriate waiver documentation:

1. **wave_psp_a1**: Clear waivers for `tuple(dynamic_iterable)`, `zip(strict=True)`, `map(strict=True)`
2. **wave_psp_a2**: Clear waiver for `bytes`/`bytearray` (no first-class type)
3. **wave_psp_b1**: Clear waivers for `Counter` constructor overloads, `defaultdict` keywords, `heapq.merge`, max-heap helpers
4. **wave_psp_b2**: Clear waivers for lazy iterators, `functools.partial`, `operator.attrgetter/methodcaller`, weighted `random.choices`, `secrets.token_urlsafe`, bytes-oriented operations

### Parity Claim Accuracy

| Wave | Claim | Accuracy |
|------|-------|----------|
| `wave_psp_a1` | `range()` keyword args | 🔴 Inaccurate - incorrectly accepts keywords |
| `wave_psp_a2` | Method argument normalization | ✅ Accurate |
| `wave_psp_b1` | Collections helpers | ✅ Accurate |
| `wave_psp_b2` | Iterators/functional/randomness | ✅ Accurate |

---

## Recommendations

### Must Fix Before Production

1. **wave_psp_a1**: Fix `range()` keyword argument handling
   - Recommended: Reject all keyword arguments to `range()` to match CPython
   - This is a high-severity parity issue

### Documentation Updates Needed

1. **wave_psp_a1**: Document the `range()` keyword bug as a known issue until fixed
2. **wave_psp_a1**: Document empty list literal limitation as pre-existing issue
3. All waves: Ensure traceability matrices are kept up to date

### Future Work Considerations

1. Consider type inference for empty constructors (`dict()`, `list()`, `tuple()`) without type context
2. Implement `zip(strict=True)` and `map(strict=True)` in future wave
3. Consider lazy iterator runtime for full `itertools` parity

---

## Conclusion

| Wave | Status | Next Steps |
|------|--------|------------|
| `wave_psp_a1` | ⚠️ Bug to fix | Fix `range()` keyword handling before production |
| `wave_psp_a2` | ✅ Approved | Ready for production |
| `wave_psp_b1` | ✅ Approved | Ready for production |
| `wave_psp_b2` | ✅ Approved | Ready for production |

**Overall Assessment:** Three of four waves are production-ready. `wave_psp_a1` requires a small fix to `range()` keyword handling to achieve full CPython parity.
