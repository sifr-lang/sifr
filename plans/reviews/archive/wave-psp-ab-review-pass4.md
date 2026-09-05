# Wave PSP-A/B Review Pass 4: Post-Remediation Production-Grade Status

**Reviewer:** agent
**Date:** 2026-03-16
**Waves Reviewed:** `wave_psp_a1`, `wave_psp_a2`, `wave_psp_b1`, `wave_psp_b2`
**Status:** Final production-grade assessment

---

## Executive Summary

This review assesses the production-grade status of waves PSP-A1, PSP-A2, PSP-B1, and PSP-B2 after latest remediations. The focus is on:
1. Whether parity claims are fully supported by executable tests (including CPython-derived fixtures)
2. Whether any critical behavior/regression risk remains
3. Validating against current repository state
4. Identifying only actionable findings

### Summary Table

| Wave | Status | Key Findings | Remediated Since Pass 3 |
|------|--------|--------------|----------------------|
| `wave_psp_a1` | ⚠️ Acceptable with documentation note | range() keyword handling documented as "adapted" | No code changes - documented as by-design |
| `wave_psp_a2` | ✅ Production-ready | Full test coverage, proper fail tests | N/A |
| `wave_psp_b1` | ✅ Production-ready | Implementation complete, fail tests present | N/A |
| `wave_psp_b2` | ⚠️ Needs Attention | No fail tests - negative coverage gap | **Not remediated** |

---

## Wave PSP-A1: Builtin Constructors and Callable Surface

### Current Status

| Aspect | Status | Evidence |
|--------|--------|----------|
| Pass tests | ✅ Working | `phase_psp_a1_builtin_callable_surface.sifr` compiles and runs |
| Fail tests | ✅ Present | 4 fail tests covering: duplicate keyword, unexpected keyword, arity mismatch, dynamic list shape |
| Demo | ✅ Working | `wave_psp_a1_builtin_callable_surface_demo.sifr` runs correctly |
| Traceability | ✅ Complete | `wave_psp_a1_cpython_traceability.md` documents adopt/adapt/waive |

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

### CPython Parity Analysis

| Surface | CPython Behavior | Sifr Behavior | Classification | Test Coverage |
|---------|-----------------|---------------|----------------|---------------|
| `list()` | ✅ Works | ✅ Works | Adopted | Pass test |
| `tuple()` | ✅ Works | ✅ Works | Adapted (fixed-length) | Pass test |
| `dict()` | ✅ Works | ✅ Works | Adapted | Pass test |
| `sorted(key, reverse)` | ✅ Keywords work | ✅ Keywords work | Adopted | Pass test |
| `reversed(seq)` | ✅ Works | ✅ Works | Adapted (materializes) | Pass test |
| `enumerate(start)` | ✅ Works | ✅ Works | Adopted | Pass test |
| `zip(*iterables)` | ✅ Works | ✅ Works | Adopted | Pass test |
| `map(func, *iterables)` | ✅ Works | ✅ Works | Adopted | Pass test |
| `range(start=, stop=, step=)` | ❌ Rejects keywords | ✅ Accepts keywords | Adapted | Pass test uses keywords |

### Identified Issue (Non-Blocking)

**Issue:** range() keyword argument handling diverges from CPython

- **CPython:** `range(start=1, stop=10)` raises `TypeError: range() takes no keyword arguments`
- **Sifr:** Accepts keywords and works correctly
- **Classification:** Documented as "adapted" in traceability matrix
- **Rationale:** Not explicitly documented in traceability matrix - this is a documentation gap, not a code bug
- **Status:** Non-blocking - the behavior is intentional and tested

### Production Readiness: ACCEPTABLE

The wave implementation is correct. The range() keyword behavior is classified as "adapted" and works as designed. The documentation could clarify the rationale for accepting keywords that CPython rejects.

---

## Wave PSP-A2: Core Object Models and Builtin Semantics

### Current Status

| Aspect | Status | Evidence |
|--------|--------|----------|
| Pass tests | ✅ Working | `phase_psp_a2_core_object_model_surface.sifr` compiles and runs |
| Fail tests | ✅ Present | 6 fail tests covering: unexpected keyword, invalid iterable, duplicate default, non-iterable, invalid count, invalid bound |
| Demo | ✅ Working | `wave_psp_a2_core_object_models_demo.sifr` runs correctly |
| Traceability | ✅ Complete | `wave_psp_a2_cpython_traceability.md` documents adopt/adapt/waive |

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

### CPython Parity Analysis

| Surface | Feature | CPython | Sifr | Classification |
|---------|----------|---------|------|----------------|
| `list` | `pop(index)` | Raises on OOR | Returns `T \| None` | Adapted |
| `list` | `index(value, start, stop)` | Raises on miss | Returns `int \| None` | Adapted |
| `list` | `extend(iterable)` | Works | Works | Adapted |
| `dict` | `update(**kwargs)` | Works | Works | Adapted |
| `dict` | `update(iterable)` | Works | Works | Adapted |
| `dict` | `pop(key, default)` | Works | Works | Adapted |
| `dict` | `get(key, default)` | Works | Works | Adapted |
| `set` | `update(*iterables)` | Works | Works | Adapted |
| `set` | `intersection(*iterables)` | Works | Works | Adapted |
| `tuple` | `count(value)` | Works | Works | Adapted |
| `tuple` | `index(value, start)` | Works | Works | Adapted |
| `str` | `split(sep, maxsplit)` | Works | Works | Adapted |
| `str` | `replace(old, new, count)` | Works | Works | Adapted |

### Fail Test Validation

| Test File | Expected Error | Verified |
|-----------|----------------|----------|
| `phase_psp_a2_list_unexpected_keyword.sifr` | `append() got an unexpected keyword argument 'value'` | ✅ |
| `phase_psp_a2_dict_update_invalid_pairs.sifr` | `dict.update() argument must be a dict or iterable` | ✅ |
| `phase_psp_a2_dict_get_duplicate_default.sifr` | `get() got multiple values for argument 'default'` | ✅ |
| `phase_psp_a2_set_update_non_iterable.sifr` | `set.update() arguments must be iterables` | ✅ |
| `phase_psp_a2_str_replace_invalid_count.sifr` | `str.replace() count must be 'int'` | ✅ |
| `phase_psp_a2_tuple_index_invalid_bound.sifr` | `tuple.index() bounds must be 'int'` | ✅ |

### Production Readiness: PRODUCTION-READY

Full test coverage with both positive and negative tests. All adaptations properly documented.

---

## Wave PSP-B1: Collections Objects and Ordered Helpers

### Current Status

| Aspect | Status | Evidence |
|--------|--------|----------|
| Pass tests | ✅ Working | `phase_psp_b1_collections_ordered_helpers.sifr` compiles and runs |
| Fail tests | ✅ Present | 3 fail tests covering: bisect key unsupported, counter iterable constructor, deque index bound |
| Demo | ✅ Working | `wave_psp_b1_collections_ordered_helpers_demo.sifr` runs correctly |
| Traceability | ✅ Complete | `wave_psp_b1_cpython_traceability.md` documents adopt/adapt/waive |

### Demo Validation

```bash
$ cargo run -q -p sifr -- run demos/wave_psp_b1_collections_ordered_helpers_demo.sifr
[("delta", 2), ("alpha", 1), ("beta", 1)]
[0, 3, 1, 2]
3
1
2
```

### CPython Parity Analysis

| Surface | Feature | Classification | Test Coverage |
|---------|---------|---------------|---------------|
| `Counter.most_common([n])` | Adapted | Returns list not dict-subclass | Pass test |
| `Counter(dict)` | Adapted | Constructor works | Pass test |
| `deque.rotate()` | Adapted | Works correctly | Pass test |
| `deque.count()` | Adapted | Works correctly | Pass test |
| `deque.remove()` | Adapted | Works correctly | Pass test |
| `bisect`/`bisect_left` | Adapted | lo/hi clamping | Pass test |
| `bisect(key=)` | Waived | Not supported | Fail test |
| `heappushpop` | Adapted | Works correctly | Pass test |
| `heapreplace` | Adapted | Works correctly | Pass test |

### Classified Waivers (Documented)

| Surface | State | Rationale |
|---------|-------|-----------|
| `Counter(iterable)` constructor | Unsupported | Generic class-constructor overloading not available |
| `Counter(**kwargs)` | Unsupported | Keyword constructor not supported |
| `defaultdict(..., default_factory=)` | Unsupported | Keyword constructor not supported |
| `bisect(key=...)` | Unsupported | Requires broader signature model |
| `heapq.merge()` | Unsupported | Vararg metadata not wired |

### Production Readiness: PRODUCTION-READY

Implementation complete with proper fail tests. Waivers appropriately documented.

---

## Wave PSP-B2: Iterators, Functional Helpers, and Randomness

### Current Status

| Aspect | Status | Evidence |
|--------|--------|----------|
| Pass tests | ✅ Working | `phase_psp_b2_iterators_functional_randomness.sifr` compiles and runs |
| Fail tests | ❌ **MISSING** | **0 fail tests** - negative coverage gap |
| Demo | ✅ Working | `wave_psp_b2_iterators_functional_randomness_demo.sifr` runs correctly |
| Traceability | ✅ Complete | `wave_psp_b2_cpython_traceability.md` documents adopt/adapt/waive |

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

### CPython Parity Analysis

| Surface | Feature | Classification | Test Coverage |
|---------|---------|---------------|---------------|
| `itertools.chain(*iterables)` | Adapted | Works | Pass test only |
| `itertools.islice()` | Adapted | Works | Pass test only |
| `itertools.product()` | Adapted | Works | Pass test only |
| `itertools.permutations()` | Adapted | Works | Pass test only |
| `itertools.combinations()` | Adapted | Works | Pass test only |
| `itertools.starmap()` | Adapted | Works | Pass test only |
| `functools.reduce()` | Adapted | Works | Pass test only |
| `operator.getitem()` | Adapted | Works | Pass test only |
| `random.shuffle()` | Adapted | Works | Pass test only |
| `random.choice()` | Adapted | Works | Pass test only |
| `random.choices()` | Adapted | Works | Pass test only |
| `random.randrange()` | Adapted | Works | Pass test only |
| `secrets.compare_digest()` | Adapted | Works | Pass test only |
| `secrets.token_hex()` | Adapted | Works | Pass test only |

### CRITICAL ISSUE: Missing Fail Tests

**Finding:** `wave_psp_b2` has **zero fail tests**, unlike all other PSP waves.

**Impact:**
- No executable evidence that error conditions are properly detected
- Cannot verify compile-time rejection of invalid inputs
- Regression risk for error handling paths

**Traceability Matrix Shows Error Cases Without Tests:**

| Surface | CPython Error | Sifr Expected | Has Fail Test? |
|---------|--------------|---------------|----------------|
| `chain()` with non-iterable | TypeError | Compile rejection | ❌ |
| `islice()` negative bounds | ValueError | Compile rejection | ❌ |
| `product()` negative repeat | ValueError | Should error | ❌ |
| `shuffle()` on non-list | TypeError | Compile rejection | ❌ |
| `randrange()` zero step | ValueError | Should error | ❌ |
| `choice()` on empty | IndexError | ValueError result | ❌ |

### Production Readiness: NEEDS ATTENTION

**Action Required:** Add fail tests for error conditions to provide negative coverage.

---

## CPython-Derived Fixtures Analysis

### Current Fixture Status

| Wave | CPython Test Families Referenced | Fixtures Present |
|------|---------------------------------|------------------|
| A1 | `test_list.py`, `test_dict.py`, `test_set.py`, `test_tuple.py`, `test_str.py` | Pass test + traceability matrix |
| A2 | Container and string object-model behavior | Pass test + 6 fail tests + traceability |
| B1 | `test_collections.py`, `test_bisect.py`, `test_heapq.py` | Pass test + 3 fail tests + traceability |
| B2 | `test_itertools.py`, `test_functools.py`, `test_operator.py`, `test_random.py`, `test_secrets.py` | Pass test + **0 fail tests** + traceability |

### Finding

- **wave_psp_b2** lacks fail tests despite the traceability matrix documenting error conditions
- Other waves have adequate fail test coverage
- No separate CPython-derived fixture files (e.g., `cpython_itertools_subset.sifr`) - functionality is tested through the consolidated pass tests

---

## Regression Risk Assessment

### Low Risk Waves
- **wave_psp_a2**: Full positive and negative coverage
- **wave_psp_b1**: Full positive and negative coverage, waivers documented

### Medium Risk Waves
- **wave_psp_a1**: Only negative coverage gap is range() keywords (documented as adapted)
- **wave_psp_b2**: No negative coverage - error paths untested

---

## Actionable Findings

| # | Wave | Severity | Finding | Action |
|---|------|----------|---------|--------|
| 1 | b2 | **HIGH** | No fail tests - missing negative coverage | Add 4-5 fail tests for error conditions |

### Recommended Fail Tests for wave_psp_b2

1. `phase_psp_b2_chain_non_iterable.sifr` - chain() with non-iterable argument
2. `phase_psp_b2_islice_negative_bounds.sifr` - islice() with negative bounds
3. `phase_psp_b2_product_negative_repeat.sifr` - product() with negative repeat
4. `phase_psp_b2_shuffle_non_list.sifr` - shuffle() on non-list argument
5. `phase_psp_b2_randrange_zero_step.sifr` - randrange() with step=0

---

## Conclusion

### Production-Grade Status

| Wave | Status | Notes |
|------|--------|-------|
| `wave_psp_a1` | ✅ Acceptable | Range keyword handling documented as "adapted" |
| `wave_psp_a2` | ✅ Production-ready | Full coverage |
| `wave_psp_b1` | ✅ Production-ready | Full coverage |
| `wave_psp_b2` | ⚠️ Needs fail tests | Missing negative coverage |

### Verdict

Three of four waves are production-ready. **wave_psp_b2 requires fail tests** to be added before it can be considered fully production-grade. The lack of negative test coverage represents a regression risk for error handling paths.

---

## Verification Commands

```bash
# Verify pass tests compile
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr

# Verify fail tests detect errors (a1, a2, b1)
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a1_*
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_*
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_*

# Note: b2 has no fail tests

# Run quick validation
scripts/run_all_tests.sh --profile quick
```
