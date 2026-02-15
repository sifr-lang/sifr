# Post-Hardening Audit Report: Iteration Protocol

**Date:** February 15, 2026  
**Scope:** 5 test files in `audit/iteration_protocol/`  
**Context:** Post Language Hardening Phase (Milestone 5: iteration_v2 completed)

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 2 | 40.0% |
| **Fail (Sifr compile)** | 2 | 40.0% |
| **Fail (Runtime)** | 1 | 20.0% |
| **Total** | 5 | 100% |

---

## Passing Tests (2)

These tests compile and run correctly after the Language Hardening phase:

| # | Test | Notes |
|---|------|-------|
| 01 | `01_for_over_collections.sifr` | For loops over strings, dicts, lists, ranges |
| 02 | `02_enumerate_zip_reversed.sifr` | `enumerate()`, `zip()`, `reversed()` |

---

## Failure Categories

### 1. For Loop Tuple Unpacking Limitations

**Error:** `for loop target must be a simple name` (3 occurrences)

Tuple unpacking in `for` loop targets is not supported for all patterns. The compiler requires a simple name as the loop target; destructuring patterns like `for name, val in pairs:` or `for k, v in d.items()` fail.

| File | Description |
|------|-------------|
| `03_unpacking_in_for.sifr` | Tuple unpacking in for loop (3 occurrences) |

---

### 2. Comprehension over Range

**Error:** `cannot iterate over type 'range'`

`range` is not recognized as iterable in comprehension contexts. Regular `for` loops over `range` work (01 passes), but list comprehensions and similar constructs fail.

| File | Description |
|------|-------------|
| `05_comprehension_types.sifr` | List comprehension over `range` |

---

### 3. Dict Comprehension Not Supported

**Error:** `unsupported expression type (dict comprehension)`

Dict comprehensions are not implemented. The compiler rejects the syntax.

| File | Description |
|------|-------------|
| `05_comprehension_types.sifr` | Dict comprehension syntax |

---

### 4. Generator Output Mismatch

**Error:** Runs but output may not match expected

The program compiles and runs, but the output differs from the expected behavior. Outputs `[0, 1, 2, 3, 4]` and `[0, 1, 4, 9, 16]` — semantics may be correct but expected output format or values differ.

| File | Description |
|------|-------------|
| `04_generator_protocol.sifr` | Generator protocol; output mismatch |

---

## What Was Fixed by Language Hardening (Milestone 5: iteration_v2)

The Language Hardening phase (Milestone 5: iteration_v2) brought **2 tests** to passing. Notable improvements:

| Fix | Tests Unblocked |
|-----|-----------------|
| **String iteration** | `for ch in "abc":` — emit `.chars()`; `str` accepted as iterable |
| **Dict iteration** | `for key in d:` — iterate over dict keys |
| **For loops over collections** | 01_for_over_collections now passes |

**Notable gains:**
- **01** — For loops over strings, dicts, lists, and ranges now work.
- **02** — `enumerate()`, `zip()`, `reversed()` continue to work correctly.

**Still pending from iteration_v2 scope:**
- Tuple unpacking in for (`for name, val in pairs`)
- Comprehension over `range`
- Dict comprehension
- `for k, v in dict.items()` (combines dict iteration + tuple unpacking)

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Blocks Real-World Usage

1. **For loop tuple unpacking** (03) — `for i, x in enumerate(...)`, `for k, v in d.items()` are common Python idioms. Blocks 17+ LeetCode problems per audit.

### Tier 2 — Significant Ergonomics

2. **Comprehension over range** (05) — `[x * x for x in range(5)]` is a very common pattern. List comprehensions over `range` fail.

3. **Dict comprehension** (05) — `{k: v for k, v in ...}` not supported. Common data transformation pattern.

### Tier 3 — Output Verification

4. **Generator output mismatch** (04) — Runs but output may not match expected. May be a test expectation issue or subtle generator semantics. Lower priority than compile failures.

---

## Test File Index (Post-Hardening)

| File | Status | Root Cause |
|------|--------|------------|
| `01_for_over_collections.sifr` | PASS | — |
| `02_enumerate_zip_reversed.sifr` | PASS | — |
| `03_unpacking_in_for.sifr` | FAIL (Sifr) | Tuple unpacking in for |
| `04_generator_protocol.sifr` | FAIL (Runtime) | Output mismatch |
| `05_comprehension_types.sifr` | FAIL (Sifr) | range in comprehension, dict comprehension |
