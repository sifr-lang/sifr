# CPython itertools Test Coverage Gap Analysis

**Review Pass:** 1
**Date:** 2026-03-21
**Scope:** Compare Sifr's itertools implementation and tests against CPython Lib/test/test_itertools.py

---

## 1. Surface Coverage Summary

### 1.1 CPython itertools Public API (20 functions)

| Function | Sifr Status | Notes |
|----------|-------------|-------|
| `accumulate` | ✅ Shipped | |
| `batched` | ✅ Shipped | |
| `chain` | ✅ Shipped | |
| `chain.from_iterable` | ❌ Not shipped | See gap G1 |
| `combinations` | ✅ Shipped | |
| `combinations_with_replacement` | ✅ Shipped | |
| `compress` | ✅ Shipped | |
| `count` | ✅ Shipped | Bounded (10k prefix) |
| `cycle` | ✅ Shipped | Finite signature: `cycle(data, n)` |
| `dropwhile` | ✅ Shipped | |
| `filterfalse` | ✅ Shipped | |
| `groupby` | ❌ Not shipped | Known gap; see G2 |
| `islice` | ✅ Shipped | |
| `pairwise` | ✅ Shipped | |
| `permutations` | ✅ Shipped | |
| `product` | ✅ Shipped | |
| `repeat` | ✅ Shipped | |
| `starmap` | ✅ Shipped | Binary only (intentional) |
| `takewhile` | ✅ Shipped | |
| `tee` | ❌ Not shipped | Known gap; see G2 |
| `zip_longest` | ✅ Shipped | |

### 1.2 Sifr-Specific Extensions (not in CPython)

| Function | Notes |
|----------|-------|
| `take(n, iterable)` | Convenience function |
| `flatten(lists)` | Convenience function |
| `count_from(start, step, n)` | Bounded count for internal use |

---

## 2. Test Coverage Gap Analysis

### 2.1 Implemented Functions: Test Coverage Status

| Function | CPython Tests | Sifr Tests | Gap Severity |
|----------|---------------|-------------|--------------|
| `accumulate` | 135 tests total across all classes | ✅ Covered | **Medium** — Missing: custom operators (min, max, mul), initial parameter edge cases |
| `batched` | Full coverage (strict mode, errors) | ✅ Covered | **Low** — Missing: `strict=True` incomplete batch test |
| `chain` | Extensive | ✅ Covered | **Low** — Missing: `chain.from_iterable` |
| `combinations` | Extensive (overflow, tuple reuse) | ✅ Covered | **Medium** — Missing: overflow tests, tuple reuse tests |
| `combinations_with_replacement` | Extensive | ✅ Covered | **Medium** — Missing: overflow tests |
| `compress` | Good | ✅ Covered | **Low** |
| `count` | Extensive (step, threading, overflow) | ✅ Covered | **Medium** — Missing: threading tests, maxsize boundary tests |
| `cycle` | Good | ✅ Covered | **Low** — Different signature (finite vs infinite) |
| `dropwhile` | Good | ✅ Covered | **Low** |
| `filterfalse` | Good | ✅ Covered | **Low** |
| `islice` | Extensive | ✅ Covered | **Low** |
| `pairwise` | Extensive (reentrant) | ✅ Covered | **Medium** — Missing: reentrant tests |
| `permutations` | Extensive | ✅ Covered | **Medium** — Missing: overflow tests, tuple reuse tests |
| `product` | Extensive (overflow, tuple reuse) | ✅ Covered | **Medium** — Missing: overflow tests |
| `repeat` | Good | ✅ Covered | **Low** |
| `starmap` | Good | ✅ Covered | **Low** |
| `takewhile` | Good | ✅ Covered | **Low** |
| `zip_longest` | Extensive | ✅ Covered | **Medium** — Missing: tuple reuse tests |

---

## 3. Identified Gaps

### G1: `chain.from_iterable` — Not Implemented

**Status:** Not shipped
**CPython Test:** `test_chain_from_iterable`
**Impact:** High
**Rationale:** Common itertools pattern for flattening iterables. Sifr has `flatten` which serves a similar purpose but with different semantics (materializes input).

### G2: `groupby` — Not Implemented

**Status:** Not shipped (documented as unsupported)
**CPython Test:** Multiple tests in TestBasicOps, TestGC, TestVariousIteratorArgs
**Impact:** High
**Rationale:** Requires lazy iteration and internal state management that conflicts with current generator lowering. Already in fail test suite (`phase_psp_iter_fix_0_itertools_groupby_unsupported.sifr`).

### G3: `tee` — Not Implemented

**Status:** Not shipped (documented as unsupported)
**CPython Test:** Multiple tests including threading, deallocation, concurrent access
**Impact:** High
**Rationale:** Requires internal buffering and copy-on-write semantics that don't map well to Sifr's current model. Already in fail test suite (`phase_psp_iter_fix_0_itertools_tee_unsupported.sifr`).

### G4: Bounded `count()` — Design Adaptation

**Status:** Shipped with adaptation
**CPython Behavior:** Infinite iterator
**Sifr Behavior:** Bounded to 10,000 elements via `count_from`
**Impact:** Low (intentional)
**Rationale:** Current generator lowering materializes yields eagerly. The adaptation provides a usable prefix while preserving leading behavior.

### G5: Finite `cycle()` — Design Adaptation

**Status:** Shipped with adaptation
**CPython Behavior:** Infinite iterator (requires explicit take)
**Sifr Behavior:** Finite signature: `cycle(data, n)` returns n elements
**Impact:** Low (intentional)
**Rationale:** Aligns with bounded iterator model; more ergonomic for common use cases.

### G6: Binary-only `starmap()` — Design Adaptation

**Status:** Shipped with adaptation
**CPython Behavior:** Arbitrary-arity callable via `*args`
**Sifr Behavior:** Binary callable only: `starmap(func, pairs)`
**Impact:** Low (intentional)
**Already tested:** `phase_psp_b2_itertools_starmap_non_binary_callable.sifr` confirms rejection of non-binary callables.

---

## 4. Test-Specific Coverage Gaps (Medium Priority)

### 4.1 Overflow/Size Tests

CPython includes tests for overflow conditions on combinatoric functions:
- `test_combinations_overflow`
- `test_combinations_with_replacement_overflow`
- `test_permutations_overflow`
- `test_product_overflow`

**Sifr Status:** Not applicable — Sifr uses materialization which naturally avoids these overflow scenarios (pre-computes all results).

### 4.2 Tuple Reuse Implementation Detail

CPython tests for tuple reuse optimization:
- `test_combinations_tuple_reuse`
- `test_combinations_with_replacement_tuple_reuse`
- `test_permutations_tuple_reuse`
- `test_product_tuple_reuse`

**Sifr Status:** Not applicable — This is a CPython implementation detail specific to its C-level memory management.

### 4.3 Threading Tests

CPython tests iterators under threaded access:
- `test_count_threading`
- `test_count_with_step_threading`
- `test_tee_concurrent`

**Sifr Status:** Not applicable — Sifr doesn't support threading primitives in the same way; would require different testing approach.

### 4.4 Reentrant Iterator Tests

CPython tests pairwise reentrancy:
- `test_pairwise_reenter`
- `test_pairwise_reenter2`

**Sifr Status:** Missing test coverage — Should be added.

### 4.5 `batched` strict mode

CPython tests `strict=True` parameter:
- Incomplete batch raises ValueError when `strict=True`

**Sifr Status:** Partial — Sifr's `batched` returns `Result` but doesn't have `strict` parameter.

---

## 5. Summary: Real Gaps vs Intentional Differences

| Category | Count | Examples |
|----------|-------|----------|
| **Real Missing Features** | 2 | `groupby`, `tee` |
| **Intentional Design Differences** | 3 | `count` (bounded), `cycle` (finite), `starmap` (binary only) |
| **Convenience Additions** | 2 | `take`, `flatten`, `count_from` |
| **Implementation Details (N/A)** | 5+ | overflow, tuple reuse, threading |

---

## 6. Recommended Next-Wave Checklist

### Phase 1: Fill Test Coverage Gaps (Low-Hanging Fruit)

- [ ] Add `pairwise` reentrant tests (`test_pairwise_reenter`, `test_pairwise_reenter2` ports)
- [ ] Add `accumulate` custom operator tests (min, max, mul)
- [ ] Add `batched` edge case tests (various n values, empty input)
- [ ] Add `zip_longest` tuple reuse awareness test (document behavior)

### Phase 2: Address Known Gaps (Higher Effort)

- [ ] Evaluate `groupby` feasibility — requires lazy iteration support
- [ ] Evaluate `tee` feasibility — requires internal buffering model
- [ ] Implement `chain.from_iterable` — high-value common pattern

### Phase 3: Advanced Testing (Future)

- [ ] Add property-based tests for combinatoric functions (size, ordering invariants)
- [ ] Add performance/materialization boundary tests
- [ ] Document behavioral differences in user-facing docs

---

## 7. Conclusion

Sifr's itertools coverage is comprehensive for a first milestone. The core 18 shipped functions have good test coverage that validates CPython-compatible behavior. The primary gaps are:

1. **Two unimplemented functions** (`groupby`, `tee`) — known limitations with documented rationale
2. **Three intentional adaptations** — bounded `count`, finite `cycle`, binary-only `starmap`
3. **Test depth gaps** — reentrant tests, operator variations, edge cases

The test coverage is sufficient to validate the shipped surface. The recommended next wave focuses on adding reentrant tests and evaluating `chain.from_iterable` for implementation.
