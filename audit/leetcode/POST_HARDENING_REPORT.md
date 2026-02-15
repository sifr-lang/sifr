# Post-Hardening Audit Report: LeetCode Problems

## Summary

| Metric | Count | % |
|--------|------:|--:|
| Total files | 411 | 100% |
| Pass (sifr check) | 38 | 9.2% |
| Fail (sifr check) | 373 | 90.8% |

Of the 38 that pass `sifr check`, **27 also compile and run** successfully through Rust, while **11 fail at Rust compilation**.

| Stage | Pass | Fail |
|-------|-----:|-----:|
| Sifr type-check | 38 | 373 |
| Rust compile (of 38) | 27 | 11 |
| **End-to-end pass** | **27** | — |

**End-to-end pass rate: 6.6% (27/411)**

---

## Sifr Compilation Failure Categories (373 files)

Errors are grouped by root cause. A single file may trigger multiple error types.

### Tier 1: High-Frequency Blockers

| # | Error Category | Occurrences | Root Cause |
|---|---------------|------------:|------------|
| 1 | `undefined function` | 128 | Nested functions (`def` inside `def`) not supported; also `set()` constructor, `tuple()` constructor, `backtrack`/`dfs` inner functions |
| 2 | `undefined variable` | 125 | Variables defined inside nested scopes or after unsupported statements become invisible |
| 3 | `unknown type` | 87 | Forward references to classes (e.g., `ListNode`, `TreeNode`, `Node`) not resolved when used as parameter/return types before the class is defined |
| 4 | `unsupported statement type` | 68 | Nested function definitions (`def` inside `def`) are not lowered to HIR |
| 5 | `parameter missing type annotation` | 54 | Nested function parameters can't be inferred; also class `__init__` params when class is defined after usage |
| 6 | `len() on optional/union type` | 37 | `len()` rejects `list[int] \| None` and similar union types; needs auto-narrowing or overload |
| 7 | `comparison on optional/union` | 36 | `<`, `>`, `<=`, `>=` not supported between `int \| None` and `int \| None` |
| 8 | `cannot index optional type` | 36 | Indexing `list[int] \| None` fails; needs narrowing or safe-index support |
| 9 | `type has no field` | 36 | Accessing fields on union types after narrowing; field access on classes not resolving |
| 10 | `for loop target must be simple name` | 34 | Tuple unpacking in `for i, v in enumerate(...)` not fully supported |

### Tier 2: Medium-Frequency Blockers

| # | Error Category | Occurrences | Root Cause |
|---|---------------|------------:|------------|
| 11 | `unsupported operand for +` | 29 | List concatenation (`list + list`), string + optional, operations on union types |
| 12 | `cannot compare with ==` | 23 | Comparing narrowed-to-`Never` types, or `int \| None` with `int` |
| 13 | `cannot iterate over type` | 19 | Range in comprehension context, iterating over `str \| None` or other union types |
| 14 | `attribute access not supported as expression` | 17 | Field access on narrowed union types emits error instead of resolving |
| 15 | `unsupported operand for -` | 15 | Subtraction on union/optional types |
| 16 | `bad operand for unary not` | 15 | `not list_var` (truthiness of collections) not supported |
| 17 | `subscript assignment target must be simple name` | 14 | `matrix[i][j] = val` (nested subscript assignment) not supported |
| 18 | `return type mismatch` | 14 | Functions returning `int` but body returns `float` (from division), or union mismatches |
| 19 | `dict.get() takes 1 argument` | 13 | `dict.get(key, default)` 2-arg form not supported |
| 20 | `augmented assignment target must be simple name` | 13 | `result[i] += val` (subscript augmented assignment) not supported |

### Tier 3: Lower-Frequency Issues

| # | Error Category | Occurrences | Root Cause |
|---|---------------|------------:|------------|
| 21 | `unsupported expression type` | 11 | Dict comprehension, set comprehension expressions |
| 22 | `unsupported operand for *` | 10 | Multiplication on union/optional types, list repetition |
| 23 | `type mismatch` | 10 | Assigning union to non-union variable, optional to concrete |
| 24 | `tuple unpacking target must be simple name` | 7 | `a, b = b, a` in some contexts |
| 25 | `cannot unpack non-tuple type` | 6 | Unpacking from `dict.items()` or other iterables |
| 26 | `cannot compare with !=` | 6 | Comparing optional/union types with `!=` |
| 27 | `list.pop() takes no arguments` | 3 | `list.pop(index)` with index argument |
| 28 | `list element type mismatch` | 3 | Heterogeneous list literals `[str, int]` |
| 29 | `abs() on optional` | 3 | `abs()` doesn't accept union types |
| 30 | Other (misc) | ~20 | `sum()` args, `zip()` args, `enumerate()` args, `reversed()` args, `//` operator, etc. |

---

## Rust Compilation Failures (11 of 38 that passed sifr check)

| File | Rust Error | Root Cause |
|------|-----------|------------|
| 0026_remove_duplicates_from_sorted_array.sifr | E0308 mismatched types | Codegen type mismatch |
| 0069_sqrtx.sifr | E0384 cannot assign twice to immutable variable | Variable reassignment not marked `mut` |
| 0080_remove_duplicates_from_sorted_array_ii.sifr | E0308 mismatched types | Codegen type mismatch |
| 0191_number_of_1_bits.sifr | E0308 mismatched types | Codegen type mismatch (bitwise ops) |
| 0231_power_of_two.sifr | E0428 name defined multiple times | Function overloading not supported |
| 0367_valid_perfect_square.sifr | E0308 mismatched types | Codegen type mismatch |
| 0441_arranging_coins.sifr | E0308 mismatched types | Codegen type mismatch |
| 1299_replace_elements.sifr | E0308 mismatched types | Codegen type mismatch |
| 1750_min_length_string.sifr | E0384 cannot assign twice to immutable | Variable reassignment not marked `mut` |
| 1963_min_swaps_balanced.sifr | E0384 cannot assign twice to immutable | Variable reassignment not marked `mut` |
| 1968_array_not_equal_avg.sifr | E0369 cannot multiply Vec by i64 | List repetition codegen incorrect |

### Rust Failure Categories:
- **Codegen type mismatch (E0308)**: 6 files — generated Rust has wrong types (likely `i64` vs `usize` or similar)
- **Variable mutability (E0384)**: 3 files — reassigned variables not emitted as `mut`
- **Function overloading (E0428)**: 1 file — multiple function definitions with same name
- **List repetition (E0369)**: 1 file — `list * n` codegen emits invalid Rust

---

## Fully Passing (27 end-to-end)

These 27 LeetCode problems compile through Sifr and produce correct Rust that compiles and runs:

| File | Problem |
|------|---------|
| 0009_palindrome_number | Palindrome Number |
| 0014_longest_common_prefix_v2 | Longest Common Prefix |
| 0045_jump_game_ii_v2 | Jump Game II |
| 0053_maximum_subarray_v2 | Maximum Subarray |
| 0055_jump_game_v2 | Jump Game |
| 0058_length_of_last_word_v2 | Length of Last Word |
| 0070_climbing_stairs | Climbing Stairs |
| 0121_best_time_to_buy_and_sell_stock_v2 | Best Time to Buy and Sell Stock |
| 0125_valid_palindrome | Valid Palindrome |
| 0134_gas_station_v2 | Gas Station |
| 0136_single_number | Single Number |
| 0151_reverse_words_in_a_string | Reverse Words in a String |
| 0152_maximum_product_subarray_v2 | Maximum Product Subarray |
| 0169_majority_element_v2 | Majority Element |
| 0190_reverse_bits | Reverse Bits |
| 0198_house_robber_v2 | House Robber |
| 0238_product_of_array_except_self_v2 | Product of Array Except Self |
| 0263_ugly_number | Ugly Number |
| 0392_is_subsequence | Is Subsequence |
| 0441_arranging_coins* | Arranging Coins |
| 0459_repeated_substring_pattern | Repeated Substring Pattern |
| 0509_fibonacci_number | Fibonacci Number |
| 0605_can_place_flowers_v2 | Can Place Flowers |
| 0704_binary_search_v2 | Binary Search |
| 1464_max_product_two_elements | Max Product of Two Elements |
| 1523_count_odd_numbers | Count Odd Numbers in Range |
| 1768_merge_strings_alternately_v2 | Merge Strings Alternately |
| 1929_concatenation_of_array_v2 | Concatenation of Array |

---

## What the Language Hardening Phase Fixed

The 10 milestones addressed many of the issues that previously blocked LeetCode problems:

- **M1 (codegen_fixes)**: Tuple indexing, int/int division, print(None), escaped quotes, float*int casts, **= power
- **M2 (narrowing_v2)**: Early-return narrowing, and-based narrowing, elif isinstance chains
- **M3 (ownership_v2)**: print() no longer consumes values, string method moves, list mutation after use
- **M4 (subscript_mutation)**: `list[i] = val`, `dict[key] = val`, `self.field += 1`
- **M5 (iteration_v2)**: String iteration, tuple unpack in for, dict.items() iteration
- **M6 (builtins_v2)**: max/min 2-arg, range 3-arg, sorted key=, pow(), list.pop(i)
- **M7 (syntax_expansion)**: Bitwise operators, chained assignment, unary +
- **M8 (recursive_types)**: ListNode, TreeNode with Box<T>
- **M9 (inference_v2)**: Return type inference for unannotated functions
- **M10 (stdlib_hardening)**: Set type, set methods, import aliases

---

## Top Remaining Issues (by impact on LeetCode coverage)

### Critical (would unblock 100+ problems each)

1. **Nested functions / closures** — 68 "unsupported statement type" + 128 "undefined function" (many are inner `def`s). This is the single biggest blocker.
2. **Forward class references** — 87 "unknown type" errors from `ListNode`, `TreeNode`, `Node` used before definition. Need forward declaration or two-pass resolution.
3. **Operations on union/optional types** — 36+29+15+10 = 90 errors from arithmetic/comparison on `T | None`. Need auto-narrowing after safe indexing or explicit unwrap.

### High (would unblock 30-60 problems each)

4. **Tuple unpacking in for loops** — 34 errors. `for i, v in enumerate(...)` pattern is extremely common in LeetCode.
5. **Parameter type inference** — 54 errors. Nested function params and some class methods lack annotations.
6. **len() on union types** — 37 errors. Very common pattern: `len(list_that_might_be_none)`.
7. **Dict comprehension** — Part of 11 "unsupported expression type" + others. Common in LeetCode solutions.

### Medium (would unblock 10-30 problems each)

8. **dict.get(key, default)** — 13 errors. Two-arg `get()` is extremely common.
9. **Nested subscript assignment** — 14 errors. `matrix[i][j] = val` pattern.
10. **Augmented subscript assignment** — 13 errors. `result[i] += val` pattern.
11. **bool(collection)** / truthiness — 15 errors. `not list_var` as emptiness check.
12. **Field access on narrowed types** — 17+36 errors. Accessing fields after isinstance narrowing.

### Low (would unblock <10 problems each)

13. **Variable mutability in codegen** — 3 Rust failures from missing `mut`
14. **Codegen type mismatches** — 6 Rust failures from wrong integer types
15. **list.pop(index)** — 3 errors (should already be supported per M6)
16. **List concatenation** — `list + list` not supported
17. **Function overloading** — 1 Rust failure from duplicate function names
