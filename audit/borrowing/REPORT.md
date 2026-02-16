# Post-Hardening Audit Report: Borrowing & Ownership

**Date:** February 16, 2026  
**Scope:** 50 test files in `audit/borrowing/`  
**Context:** Post milestone_borrow_hardening + subsequent compiler changes

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 29 | 58.0% |
| **Fail (Sifr compile — correct rejections)** | 12 | 24.0% |
| **Fail (Rust compile)** | 7 | 14.0% |
| **Fail (runtime)** | 2 | 4.0% |
| **Total** | 50 | 100% |

---

## Regressions Since Last Report

The previous report (February 15, 2026) had: **39 PASS**, 10 Sifr compile (correct), 1 pre-existing type issue, 0 Rust compile, 0 runtime.

The following **9 tests** previously passed but now fail:

| Test | Previous | Current | Root Cause |
|------|----------|---------|------------|
| 13_string_method_borrow.sifr | PASS | FAIL (Rust) | &String vs String comparison; mismatched types |
| 14_reassignment_resets_move.sifr | PASS | FAIL (Rust) | `()` doesn't implement Display; can't compare &String with String |
| 19_class_instance_move.sifr | PASS | FAIL (Rust) | use of moved value: c, s, t (class field destructuring) |
| 22_class_method_mut_self.sifr | PASS | FAIL (Rust) | mismatched types |
| 28_return_local_string.sifr | PASS | FAIL (Runtime) | empty output |
| 30_multiple_function_calls_same_var.sifr | PASS | FAIL (Rust) | mismatched types |
| 33_move_in_both_branches.sifr | PASS | FAIL (Rust) | String: Borrow<&String> not satisfied |
| 34_sorted_borrows.sifr | PASS | FAIL (Runtime) | could not run binary: No such file or directory |
| 42_chained_string_methods.sifr | PASS | FAIL (Rust) | dyn Any: Eq, Hash not satisfied; mismatched types |

---

## Rust Compilation Failures by Root Cause

The 7 Rust compilation failures group into the following patterns:

### 1. &String vs String comparison (borrow-by-default codegen regression)

Tests where the compiler generates `&String` but the Rust code expects or compares with `String`:

| Test | Error |
|------|-------|
| 13_string_method_borrow.sifr | E0277 can't compare &String with String; E0308 mismatched types |
| 14_reassignment_resets_move.sifr | E0277 can't compare &String with String |
| 33_move_in_both_branches.sifr | E0277 String: Borrow<&String> not satisfied |

### 2. Display / trait implementation

| Test | Error |
|------|-------|
| 14_reassignment_resets_move.sifr | E0277 `()` doesn't implement Display |

### 3. Class / struct codegen

| Test | Error |
|------|-------|
| 19_class_instance_move.sifr | E0382 use of moved value: c, s, t (class field destructuring) |
| 22_class_method_mut_self.sifr | E0308 mismatched types |

### 4. Mismatched types (generic / function signatures)

| Test | Error |
|------|-------|
| 30_multiple_function_calls_same_var.sifr | E0308 mismatched types |

### 5. dyn Any: Eq, Hash (chained method return types)

| Test | Error |
|------|-------|
| 42_chained_string_methods.sifr | E0277 dyn Any: Eq, Hash not satisfied; E0308 mismatched types |

---

## Borrow-by-Default Model

As of milestone_borrow_default, function parameters are **borrowed by default** (`&T`). The `own` keyword explicitly transfers ownership (`T`), and `mut` enables mutable borrowing (`&mut T`). Copy types (`int`, `float`, `bool`) always pass by value.

### Updated Tests (8 tests modified for `own` keyword)

| Test | Change | Reason |
|------|--------|--------|
| 08 | Added `own` to `consume()` param | Function consumes its argument |
| 09 | Added `own` to `consume()` param | Tests use-after-move via `own` |
| 16 | Added `own` to `consume()` param | Tests move-in-loop via `own` |
| 23 | Added `own` to `describe()` param | Function consumes class instance |
| 24 | Added `own` to `describe()` param | Tests use-after-move of class |
| 31 | Added `own` to `consume_str()` param | Tests str move into two functions |
| 32 | Added `own` to `consume()` param | Tests conditional move |
| 33 | Added `own` to `consume()` param | Tests move in both branches |

---

## Correctly Failing Tests (12)

All 12 Sifr compilation failures are **correct rejections**:

- **10 ownership violations:** 03, 05, 07, 09, 16, 20, 24, 26, 31, 40
- **1 type safety:** 44 (list.pop returns `int | None`, not `int`)
- **1 set move:** 47 (use of moved value: 's1' for set)

| Test | Error | Category |
|------|-------|----------|
| 03_move_str_use_after_assign.sifr | use of moved value: 's1' | Assignment move (str) |
| 05_move_list_use_after_assign.sifr | use of moved value: 'l1' | Assignment move (list) |
| 07_move_dict_use_after_assign.sifr | use of moved value: 'd1' | Assignment move (dict) |
| 09_use_after_move_into_function.sifr | use of moved value: 'nums' | Function move |
| 16_move_in_loop_consume.sifr | value 's' is moved inside loop body | Move-in-loop |
| 20_class_instance_use_after_move.sifr | use of moved value: 'p1' | Assignment move (class) |
| 24_use_class_after_function_move.sifr | use of moved value: 'p' | Function move (class) |
| 26_tuple_use_after_move.sifr | use of moved value: 't1' | Assignment move (tuple) |
| 31_str_move_into_two_functions.sifr | use of moved value: 's' | Double function move |
| 40_nested_collection_use_after_move.sifr | use of moved value: 'outer' | Assignment move (nested list) |
| 44_list_pop_mutates.sifr | type mismatch: expected 'int', got 'int \| None' | Safety (pop returns Option) |
| 47_set_use_after_move.sifr | use of moved value: 's1' | Assignment move (set) |

---

## Passing Tests (29)

01, 02, 04, 06, 08, 10, 11, 12, 15, 17, 18, 21, 23, 25, 27, 29, 32, 35, 36, 37, 38, 39, 41, 43, 45, 46, 48, 49, 50

---

## Runtime Failures (2)

| Test | Error |
|------|-------|
| 28_return_local_string.sifr | (empty output) |
| 34_sorted_borrows.sifr | could not run binary: No such file or directory |

---

## Test File Index (Post-Hardening)

| File | Status | Root Cause |
|------|--------|------------|
| 01_copy_types_reuse.sifr | PASS | — |
| 02_move_str_assignment.sifr | PASS | — |
| 03_move_str_use_after_assign.sifr | FAIL (Sifr) | Correct: assignment move |
| 04_move_list_assignment.sifr | PASS | — |
| 05_move_list_use_after_assign.sifr | FAIL (Sifr) | Correct: assignment move |
| 06_move_dict_assignment.sifr | PASS | — |
| 07_move_dict_use_after_assign.sifr | FAIL (Sifr) | Correct: assignment move |
| 08_move_into_function.sifr | PASS | — |
| 09_use_after_move_into_function.sifr | FAIL (Sifr) | Correct: function move |
| 10_copy_type_into_function.sifr | PASS | — |
| 11_print_borrows_not_moves.sifr | PASS | — |
| 12_len_borrows_not_moves.sifr | PASS | — |
| 13_string_method_borrow.sifr | FAIL (Rust) | &String vs String; mismatched types |
| 14_reassignment_resets_move.sifr | FAIL (Rust) | () Display; &String vs String |
| 15_move_in_loop.sifr | PASS | — |
| 16_move_in_loop_consume.sifr | FAIL (Sifr) | Correct: move-in-loop |
| 17_for_loop_borrows_collection.sifr | PASS | — |
| 18_multiple_for_loops_same_collection.sifr | PASS | — |
| 19_class_instance_move.sifr | FAIL (Rust) | use of moved value (class fields) |
| 20_class_instance_use_after_move.sifr | FAIL (Sifr) | Correct: assignment move |
| 21_class_method_self_borrow.sifr | PASS | — |
| 22_class_method_mut_self.sifr | FAIL (Rust) | mismatched types |
| 23_pass_class_to_function.sifr | PASS | — |
| 24_use_class_after_function_move.sifr | FAIL (Sifr) | Correct: function move |
| 25_tuple_copy_vs_move.sifr | PASS | — |
| 26_tuple_use_after_move.sifr | FAIL (Sifr) | Correct: assignment move |
| 27_list_append_after_use.sifr | PASS | — |
| 28_return_local_string.sifr | FAIL (Runtime) | empty output |
| 29_return_local_list.sifr | PASS | — |
| 30_multiple_function_calls_same_var.sifr | FAIL (Rust) | mismatched types |
| 31_str_move_into_two_functions.sifr | FAIL (Sifr) | Correct: double function move |
| 32_conditional_move.sifr | PASS | — |
| 33_move_in_both_branches.sifr | FAIL (Rust) | String: Borrow<&String> |
| 34_sorted_borrows.sifr | FAIL (Runtime) | binary not found |
| 35_enumerate_borrows.sifr | PASS | — |
| 36_zip_borrows.sifr | PASS | — |
| 37_list_comprehension_borrows.sifr | PASS | — |
| 38_dict_method_borrow.sifr | PASS | — |
| 39_nested_collection_move.sifr | PASS | — |
| 40_nested_collection_use_after_move.sifr | FAIL (Sifr) | Correct: assignment move |
| 41_fstring_borrows.sifr | PASS | — |
| 42_chained_string_methods.sifr | FAIL (Rust) | dyn Any: Eq, Hash; mismatched types |
| 43_list_sort_in_place.sifr | PASS | — |
| 44_list_pop_mutates.sifr | FAIL (Sifr) | Correct: safety (Option return) |
| 45_multiple_borrows_same_scope.sifr | PASS | — |
| 46_set_move_semantics.sifr | PASS | — |
| 47_set_use_after_move.sifr | FAIL (Sifr) | Correct: assignment move |
| 48_builtin_functions_borrow.sifr | PASS | — |
| 49_str_concat_creates_new.sifr | PASS | — |
| 50_augmented_assign_str.sifr | PASS | — |
