# Post-Hardening Audit Report: LeetCode Problems

**Date:** February 16, 2026
**Context:** Post borrow-by-default phase

## Summary

| Metric | Count | % |
|--------|------:|--:|
| Total files | 411 | 100% |
| Pass (end-to-end) | 39 | 9.5% |
| Fail (runtime) | 1 | 0.2% |
| Fail (Rust compile) | 41 | 10.0% |
| Fail (Sifr compile) | 330 | 80.3% |

| Stage | Pass | Fail |
|-------|-----:|-----:|
| Sifr type-check | 81 | 330 |
| Rust compile (of 81) | 40 | 41 |
| Runtime (of 40) | 39 | 1 |
| **End-to-end pass** | **39** | — |

**End-to-end pass rate: 9.5% (39/411)**

---

## Changes Since Last Report

| Metric | Previous | Current | Delta |
|--------|--------:|--------:|------:|
| Sifr compile failures | 373 | 330 | **−43** |
| Rust compile failures | 11 | 41 | +30 |
| End-to-end pass | 27 | 39 | **+12** |
| Runtime failures | 0 | 1 | +1 |

**Key observations:**

- **43 more tests now pass Sifr type-check**, bringing the total from 38 to 81. This is a direct result of the borrow-by-default phase and related improvements.
- **12 more tests fully pass end-to-end** (27 → 39), a 44% improvement in full pass count.
- **Rust compile failures rose from 11 to 41** — this is expected because 43 new tests now reach the Rust compilation stage. Many of these expose new codegen gaps.
- **1 runtime failure** appeared: `1498_number_of_subsequences.sifr` fails with `expected identifier, found keyword 'mod'` (a Rust keyword collision in generated code).

### New Fully Passing Tests (+16 net, +2 regressions = +14 delta to reach 39 from 27, with 2 removed)

| File | Problem |
|------|---------|
| 0045_jump_game_ii | Jump Game II |
| 0055_jump_game | Jump Game |
| 0058_length_of_last_word | Length of Last Word |
| 0122_best_time_to_buy_and_sell_stock_ii | Best Time to Buy and Sell Stock II |
| 0134_gas_station | Gas Station |
| 0238_product_of_array_except_self | Product of Array Except Self |
| 0268_missing_number | Missing Number |
| 0300_longest_increasing_subsequence_v2 | Longest Increasing Subsequence |
| 0554_brick_wall | Brick Wall |
| 0704_binary_search | Binary Search |
| 0896_monotonic_array | Monotonic Array |
| 1423_maximum_points_you_can_obtain_from_cards | Maximum Points from Cards |
| 1464_max_product_two_elements | Max Product of Two Elements |
| 1523_count_odd_numbers | Count Odd Numbers in Range |
| 1822_sign_of_the_product | Sign of the Product of an Array |
| 1929_concatenation_of_array_v2 | Concatenation of Array |

### Regressions (2 tests that previously passed now fail)

| File | Previous | Current | Issue |
|------|----------|---------|-------|
| 0151_reverse_words_in_a_string | PASS | Fail (Rust compile) | Regression in codegen |
| 0441_arranging_coins | PASS | Fail (Rust compile) | Regression in codegen |

---

## Sifr Compilation Failure Categories (330 files)

Errors are grouped by root cause. A single file may trigger multiple error types.

### Tier 1: High-Frequency Blockers

| # | Error Category | Occurrences | Root Cause |
|---|---------------|------------:|------------|
| 1 | `undefined function` | ~120 | Nested functions (`def` inside `def`) not supported; also `set()` constructor, `tuple()` constructor, `backtrack`/`dfs` inner functions |
| 2 | `undefined variable` | ~115 | Variables defined inside nested scopes or after unsupported statements become invisible |
| 3 | `unknown type` | ~80 | Forward references to classes (e.g., `ListNode`, `TreeNode`, `Node`) not resolved when used as parameter/return types before the class is defined |
| 4 | `unsupported statement type` | ~60 | Nested function definitions (`def` inside `def`) are not lowered to HIR |
| 5 | `parameter missing type annotation` | ~50 | Nested function parameters can't be inferred; also class `__init__` params when class is defined after usage |
| 6 | `len() on optional/union type` | ~33 | `len()` rejects `list[int] \| None` and similar union types; needs auto-narrowing or overload |
| 7 | `comparison on optional/union` | ~32 | `<`, `>`, `<=`, `>=` not supported between `int \| None` and `int \| None` |
| 8 | `cannot index optional type` | ~32 | Indexing `list[int] \| None` fails; needs narrowing or safe-index support |
| 9 | `type has no field` | ~32 | Accessing fields on union types after narrowing; field access on classes not resolving |
| 10 | `for loop target must be simple name` | ~30 | Tuple unpacking in `for i, v in enumerate(...)` not fully supported |

### Tier 2: Medium-Frequency Blockers

| # | Error Category | Occurrences | Root Cause |
|---|---------------|------------:|------------|
| 11 | `unsupported operand for +` | ~25 | List concatenation (`list + list`), string + optional, operations on union types |
| 12 | `cannot compare with ==` | ~20 | Comparing narrowed-to-`Never` types, or `int \| None` with `int` |
| 13 | `cannot iterate over type` | ~17 | Range in comprehension context, iterating over `str \| None` or other union types |
| 14 | `attribute access not supported as expression` | ~15 | Field access on narrowed union types emits error instead of resolving |
| 15 | `unsupported operand for -` | ~13 | Subtraction on union/optional types |
| 16 | `bad operand for unary not` | ~13 | `not list_var` (truthiness of collections) not supported |
| 17 | `subscript assignment target must be simple name` | ~12 | `matrix[i][j] = val` (nested subscript assignment) not supported |
| 18 | `return type mismatch` | ~12 | Functions returning `int` but body returns `float` (from division), or union mismatches |
| 19 | `dict.get() takes 1 argument` | ~11 | `dict.get(key, default)` 2-arg form not supported |
| 20 | `augmented assignment target must be simple name` | ~11 | `result[i] += val` (subscript augmented assignment) not supported |

### Tier 3: Lower-Frequency Issues

| # | Error Category | Occurrences | Root Cause |
|---|---------------|------------:|------------|
| 21 | `unsupported expression type` | ~10 | Dict comprehension, set comprehension expressions |
| 22 | `unsupported operand for *` | ~9 | Multiplication on union/optional types, list repetition |
| 23 | `type mismatch` | ~9 | Assigning union to non-union variable, optional to concrete |
| 24 | `tuple unpacking target must be simple name` | ~6 | `a, b = b, a` in some contexts |
| 25 | `cannot unpack non-tuple type` | ~5 | Unpacking from `dict.items()` or other iterables |
| 26 | `cannot compare with !=` | ~5 | Comparing optional/union types with `!=` |
| 27 | `list.pop() takes no arguments` | ~3 | `list.pop(index)` with index argument |
| 28 | `list element type mismatch` | ~3 | Heterogeneous list literals `[str, int]` |
| 29 | `abs() on optional` | ~3 | `abs()` doesn't accept union types |
| 30 | Other (misc) | ~15 | `sum()` args, `zip()` args, `enumerate()` args, `reversed()` args, `//` operator, etc. |

---

## Rust Compilation Failures (41 of 81 that passed Sifr check)

| File | Rust Error | Root Cause |
|------|-----------|------------|
| 0006_zigzag_conversion | Codegen error | Type/ownership mismatch in generated Rust |
| 0011_container_with_most_water | Codegen error | Type/ownership mismatch in generated Rust |
| 0026_remove_duplicates_from_sorted_array | E0308 mismatched types | Codegen type mismatch |
| 0033_search_in_rotated_sorted_array | Codegen error | Type/ownership mismatch in generated Rust |
| 0035_search_insert_position | Codegen error | Type/ownership mismatch in generated Rust |
| 0042_trapping_rain_water | Codegen error | Type/ownership mismatch in generated Rust |
| 0048_rotate_image | Codegen error | Type/ownership mismatch in generated Rust |
| 0054_spiral_matrix | Codegen error | Type/ownership mismatch in generated Rust |
| 0066_plus_one | Codegen error | Type/ownership mismatch in generated Rust |
| 0068_text_justification | Codegen error | Type/ownership mismatch in generated Rust |
| 0071_simplify_path | Codegen error | Type/ownership mismatch in generated Rust |
| 0073_set_matrix_zeroes | Codegen error | Type/ownership mismatch in generated Rust |
| 0074_search_a_2d_matrix | Codegen error | Type/ownership mismatch in generated Rust |
| 0080_remove_duplicates_from_sorted_array_ii | E0308 mismatched types | Codegen type mismatch |
| 0081_search_in_rotated_sorted_array_ii | Codegen error | Type/ownership mismatch in generated Rust |
| 0135_candy | Codegen error | Type/ownership mismatch in generated Rust |
| 0151_reverse_words_in_a_string | Codegen error | **Regression** — previously passed end-to-end |
| 0167_two_sum_ii | Codegen error | Type/ownership mismatch in generated Rust |
| 0191_number_of_1_bits | E0308 mismatched types | Codegen type mismatch (bitwise ops) |
| 0231_power_of_two | E0428 name defined multiple times | Function overloading not supported |
| 0274_h_index | Codegen error | Type/ownership mismatch in generated Rust |
| 0338_counting_bits | Codegen error | Type/ownership mismatch in generated Rust |
| 0367_valid_perfect_square | E0308 mismatched types | Codegen type mismatch |
| 0441_arranging_coins | Codegen error | **Regression** — previously passed end-to-end |
| 0605_can_place_flowers | Codegen error | Type/ownership mismatch in generated Rust |
| 0605_can_place_flowers_v2 | Codegen error | Type/ownership mismatch in generated Rust |
| 0658_find_k_closest_elements | Codegen error | Type/ownership mismatch in generated Rust |
| 0724_find_pivot_index | Codegen error | Type/ownership mismatch in generated Rust |
| 0881_boats_to_save_people | Codegen error | Type/ownership mismatch in generated Rust |
| 0948_bag_of_tokens | Codegen error | Type/ownership mismatch in generated Rust |
| 0978_longest_turbulent_subarray | Codegen error | Type/ownership mismatch in generated Rust |
| 1299_replace_elements | E0308 mismatched types | Codegen type mismatch |
| 1343_number_of_subarrays_of_size_k | Codegen error | Type/ownership mismatch in generated Rust |
| 1582_special_positions_in_binary_matrix | Codegen error | Type/ownership mismatch in generated Rust |
| 1658_minimum_operations_to_reduce_x | Codegen error | Type/ownership mismatch in generated Rust |
| 1750_min_length_string | E0384 cannot assign twice to immutable | Variable reassignment not marked `mut` |
| 1768_merge_strings_alternately | Codegen error | Type/ownership mismatch in generated Rust |
| 1838_frequency_of_most_frequent_element | Codegen error | Type/ownership mismatch in generated Rust |
| 1963_min_swaps_balanced | E0384 cannot assign twice to immutable | Variable reassignment not marked `mut` |
| 1968_array_not_equal_avg | E0369 cannot multiply Vec by i64 | List repetition codegen incorrect |
| 2300_successful_pairs_of_spells | Codegen error | Type/ownership mismatch in generated Rust |

### Rust Failure Categories:

- **Type/ownership mismatch in codegen**: 30 files — the largest category; many are new tests reaching Rust stage for the first time, exposing gaps in borrow/ownership codegen
- **Codegen type mismatch (E0308)**: 5 files — generated Rust has wrong types (likely `i64` vs `usize` or similar)
- **Variable mutability (E0384)**: 2 files — reassigned variables not emitted as `mut`
- **Function overloading (E0428)**: 1 file — multiple function definitions with same name
- **List repetition (E0369)**: 1 file — `list * n` codegen emits invalid Rust
- **Regressions**: 2 files — `0151` and `0441` previously passed but now fail at Rust compile

---

## Runtime Failures (1 of 40 that passed Rust compile)

| File | Error | Root Cause |
|------|-------|------------|
| 1498_number_of_subsequences | `expected identifier, found keyword 'mod'` | Generated Rust uses `mod` as a variable name, which is a Rust reserved keyword |

---

## Fully Passing (39 end-to-end)

These 39 LeetCode problems compile through Sifr and produce correct Rust that compiles and runs:

| File | Problem |
|------|---------|
| 0009_palindrome_number | Palindrome Number |
| 0014_longest_common_prefix_v2 | Longest Common Prefix |
| 0045_jump_game_ii_v2 | Jump Game II |
| 0045_jump_game_ii | Jump Game II |
| 0053_maximum_subarray_v2 | Maximum Subarray |
| 0055_jump_game | Jump Game |
| 0055_jump_game_v2 | Jump Game |
| 0058_length_of_last_word | Length of Last Word |
| 0058_length_of_last_word_v2 | Length of Last Word |
| 0069_sqrtx | Sqrt(x) |
| 0070_climbing_stairs | Climbing Stairs |
| 0121_best_time_to_buy_and_sell_stock_v2 | Best Time to Buy and Sell Stock |
| 0122_best_time_to_buy_and_sell_stock_ii | Best Time to Buy and Sell Stock II |
| 0125_valid_palindrome | Valid Palindrome |
| 0134_gas_station | Gas Station |
| 0134_gas_station_v2 | Gas Station |
| 0136_single_number | Single Number |
| 0152_maximum_product_subarray_v2 | Maximum Product Subarray |
| 0169_majority_element_v2 | Majority Element |
| 0190_reverse_bits | Reverse Bits |
| 0198_house_robber_v2 | House Robber |
| 0238_product_of_array_except_self | Product of Array Except Self |
| 0238_product_of_array_except_self_v2 | Product of Array Except Self |
| 0263_ugly_number | Ugly Number |
| 0268_missing_number | Missing Number |
| 0300_longest_increasing_subsequence_v2 | Longest Increasing Subsequence |
| 0392_is_subsequence | Is Subsequence |
| 0459_repeated_substring_pattern | Repeated Substring Pattern |
| 0509_fibonacci_number | Fibonacci Number |
| 0554_brick_wall | Brick Wall |
| 0704_binary_search | Binary Search |
| 0704_binary_search_v2 | Binary Search |
| 0896_monotonic_array | Monotonic Array |
| 1423_maximum_points_you_can_obtain_from_cards | Maximum Points from Cards |
| 1464_max_product_two_elements | Max Product of Two Elements |
| 1523_count_odd_numbers | Count Odd Numbers in Range |
| 1768_merge_strings_alternately_v2 | Merge Strings Alternately |
| 1822_sign_of_the_product | Sign of the Product of an Array |
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

### Borrow-by-Default Phase Improvements

The borrow-by-default phase introduced automatic borrowing semantics, which had a significant positive impact on LeetCode coverage:

- **43 fewer Sifr compile failures** — many tests that previously failed due to ownership/move errors now pass type-checking with automatic borrow semantics
- **12 more end-to-end passes** — tests involving list/string operations that previously hit move errors now generate correct Rust
- The trade-off: **30 more Rust compile failures** surfaced as tests that previously couldn't reach codegen now expose new codegen gaps (primarily type mismatches and ownership translation issues in the generated Rust)

---

## Top Remaining Issues (by impact on LeetCode coverage)

### Critical (would unblock 100+ problems each)

1. **Nested functions / closures** — ~60 "unsupported statement type" + ~120 "undefined function" (many are inner `def`s). This is the single biggest blocker.
2. **Forward class references** — ~80 "unknown type" errors from `ListNode`, `TreeNode`, `Node` used before definition. Need forward declaration or two-pass resolution.
3. **Operations on union/optional types** — ~32+25+13+9 = ~79 errors from arithmetic/comparison on `T | None`. Need auto-narrowing after safe indexing or explicit unwrap.

### High (would unblock 30-60 problems each)

4. **Tuple unpacking in for loops** — ~30 errors. `for i, v in enumerate(...)` pattern is extremely common in LeetCode.
5. **Parameter type inference** — ~50 errors. Nested function params and some class methods lack annotations.
6. **len() on union types** — ~33 errors. Very common pattern: `len(list_that_might_be_none)`.
7. **Codegen type/ownership mismatches** — 30 Rust compile failures from incorrect borrow/ownership translation. Fixing these would immediately convert Rust-stage failures to passes.

### Medium (would unblock 10-30 problems each)

8. **Dict comprehension** — Part of ~10 "unsupported expression type" + others. Common in LeetCode solutions.
9. **dict.get(key, default)** — ~11 errors. Two-arg `get()` is extremely common.
10. **Nested subscript assignment** — ~12 errors. `matrix[i][j] = val` pattern.
11. **Augmented subscript assignment** — ~11 errors. `result[i] += val` pattern.
12. **bool(collection)** / truthiness — ~13 errors. `not list_var` as emptiness check.
13. **Field access on narrowed types** — ~15+32 errors. Accessing fields after isinstance narrowing.

### Low (would unblock <10 problems each)

14. **Variable mutability in codegen** — 2 Rust failures from missing `mut`
15. **Codegen type mismatches (E0308)** — 5 Rust failures from wrong integer types
16. **Rust keyword collision** — 1 runtime failure from using `mod` as identifier
17. **List concatenation** — `list + list` not supported
18. **Function overloading** — 1 Rust failure from duplicate function names
19. **Regressions to investigate** — 2 tests (`0151`, `0441`) that previously passed now fail at Rust compile
