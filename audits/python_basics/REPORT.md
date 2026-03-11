# Post-Hardening Audit Report: Python Basics

**Date:** February 16, 2026  
**Scope:** 45 test files in `audit/python_basics/`  
**Context:** Post borrow-by-default phase

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 30 | 66.7% |
| **Fail (Sifr compile)** | 2 | 4.4% |
| **Fail (Rust compile)** | 10 | 22.2% |
| **Fail (runtime)** | 3 | 6.7% |
| **Total** | 45 | 100% |

---

## Changes Since Last Report

The previous report (February 15, 2026) had: **28 PASS**, 13 Sifr compile, 4 Rust compile, 0 runtime.

Net change: **+2 passing tests** (28 → 30). Sifr compile failures dropped dramatically (13 → 2) as many features were implemented, but Rust compile failures increased (4 → 10) due to borrow-by-default codegen regressions and newly-exposed Rust-level issues.

### Improvements (13 tests progressed)

| Test | Previous | Current | Notes |
|------|----------|---------|-------|
| 07_list_operations.sifr | FAIL (Sifr) | **PASS** | list.remove(), list + now supported |
| 14_functions_advanced.sifr | FAIL (Sifr) | **PASS** | Narrowing to Never fixed |
| 18_builtins.sifr | FAIL (Sifr) | **PASS** | range iteration now works |
| 21_scope_and_closures.sifr | FAIL (Sifr) | **PASS** | Nested functions / closures supported |
| 25_classes_basic.sifr | FAIL (Rust) | **PASS** | &mut self codegen fixed |
| 27_classes_static_class_methods.sifr | FAIL (Sifr) | **PASS** | cls parameter recognized |
| 43_real_world_todo_list.sifr | FAIL (Rust) | **PASS** | Borrow as mutable fixed |
| 15_list_comprehension.sifr | FAIL (Sifr) | FAIL (Runtime) | Progressed past Sifr compile |
| 16_dict_comprehension.sifr | FAIL (Sifr) | FAIL (Rust) | Progressed past Sifr compile |
| 29_context_managers.sifr | FAIL (Rust) | FAIL (Rust) | Different error: &String comparison |
| 36_global_constants.sifr | FAIL (Sifr) | FAIL (Runtime) | Progressed past Sifr compile |
| 44_real_world_calculator.sifr | FAIL (Sifr) | FAIL (Rust) | Progressed past Sifr compile |
| 45_real_world_matrix_ops.sifr | FAIL (Sifr) | FAIL (Runtime) | Progressed past Sifr compile |

### Regressions (5 tests that previously passed now fail)

| Test | Previous | Current | Root Cause |
|------|----------|---------|------------|
| 06_string_slicing.sifr | PASS | FAIL (Rust) | E0308 mismatched types |
| 10_control_flow_if.sifr | PASS | FAIL (Rust) | E0277 cannot multiply f64 by i64 |
| 12_loops_while.sifr | PASS | FAIL (Rust) | E0308 mismatched types |
| 30_assert.sifr | PASS | FAIL (Rust) | E0277 can't compare &String with String |
| 42_real_world_word_count.sifr | PASS | FAIL (Rust) | E0308 mismatched types |

---

## Passing Tests (30)

These tests compile and run correctly:

| # | Test | Notes |
|---|------|-------|
| 01 | `01_arithmetic_full.sifr` | Arithmetic operations |
| 02 | `02_comparison_operators.sifr` | Comparison operators |
| 03 | `03_boolean_logic.sifr` | Boolean logic |
| 04 | `04_string_methods.sifr` | String methods |
| 05 | `05_string_formatting.sifr` | String formatting |
| 07 | `07_list_operations.sifr` | List operations (NEW) |
| 08 | `08_dict_operations.sifr` | Dict operations |
| 09 | `09_tuple_operations.sifr` | Tuple operations |
| 11 | `11_loops_for.sifr` | For loops |
| 13 | `13_functions_basic.sifr` | Basic functions |
| 14 | `14_functions_advanced.sifr` | Advanced functions (NEW) |
| 18 | `18_builtins.sifr` | Builtins (NEW) |
| 19 | `19_augmented_assignment.sifr` | Augmented assignment |
| 20 | `20_unpacking.sifr` | Unpacking |
| 21 | `21_scope_and_closures.sifr` | Scope and closures (NEW) |
| 22 | `22_lambda_expressions.sifr` | Lambda expressions |
| 23 | `23_generators.sifr` | Generators |
| 24 | `24_error_handling.sifr` | Error handling |
| 25 | `25_classes_basic.sifr` | Basic classes (NEW) |
| 26 | `26_classes_inheritance.sifr` | Class inheritance |
| 27 | `27_classes_static_class_methods.sifr` | Static/class methods (NEW) |
| 28 | `28_decorators.sifr` | Decorators |
| 33 | `33_ternary_expression.sifr` | Ternary expression |
| 34 | `34_multiple_assignment.sifr` | Multiple assignment |
| 35 | `35_pass_statement.sifr` | Pass statement |
| 38 | `38_multiline_expressions.sifr` | Multiline expressions |
| 39 | `39_nested_data_structures.sifr` | Nested data structures |
| 40 | `40_real_world_fizzbuzz.sifr` | Real-world: FizzBuzz |
| 41 | `41_real_world_fibonacci.sifr` | Real-world: Fibonacci |
| 43 | `43_real_world_todo_list.sifr` | Real-world: Todo list (NEW) |

---

## Failure Categories

### Sifr Compilation Failures (2)

#### 1. Set Comprehension / Set Type Not Supported

**Error:** `unsupported statement type`, `unknown generic type 'Set'`, `undefined function 'Set'`

| File | Description |
|------|-------------|
| `17_set_comprehension.sifr` | Set comprehension syntax; `Set` generic type unknown |

#### 2. `del` Statement Limitations

**Error:** `del is only supported for collection items`

| File | Description |
|------|-------------|
| `31_del_statement.sifr` | `del` on variables or unsupported targets |

---

### Rust Compilation Failures (10)

#### 1. Mismatched Types (E0308)

Generated Rust code has type mismatches — typically `&String` vs `String`, or integer/float type conflicts from the borrow-by-default codegen changes.

| File | Error |
|------|-------|
| `06_string_slicing.sifr` | E0308 mismatched types |
| `12_loops_while.sifr` | E0308 mismatched types |
| `30_assert.sifr` | E0308 mismatched types (also &String comparison) |
| `42_real_world_word_count.sifr` | E0308 mismatched types |

#### 2. &String vs String Comparison (E0277)

Borrow-by-default generates `&String` references where Rust expects owned `String` for comparison operations.

| File | Error |
|------|-------|
| `29_context_managers.sifr` | E0277 can't compare &String with String |
| `30_assert.sifr` | E0277 can't compare &String with String |
| `44_real_world_calculator.sifr` | E0277 can't compare &String with String (4 occurrences) |

#### 3. Numeric Type Mismatch (E0277)

Arithmetic operations between different numeric types (f64 * i64) are not handled in codegen.

| File | Error |
|------|-------|
| `10_control_flow_if.sifr` | E0277 cannot multiply f64 by i64 |

#### 4. Borrow of Moved Value (E0382)

A value is used after being moved in the generated Rust code.

| File | Error |
|------|-------|
| `16_dict_comprehension.sifr` | E0382 borrow of moved value: `n` |

#### 5. Display Trait Not Implemented (E0277)

Result type used in print context without Display implementation.

| File | Error |
|------|-------|
| `32_walrus_operator.sifr` | E0277 `Result<i64, String>` doesn't implement Display |

#### 6. Type Annotations Needed (E0282)

Rust cannot infer types in generated conversion code.

| File | Error |
|------|-------|
| `37_type_conversions.sifr` | E0282 type annotations needed |

---

### Runtime Failures (3)

#### 1. Binary Not Found

Cargo build succeeds but the binary cannot be located or executed.

| File | Error |
|------|-------|
| `15_list_comprehension.sifr` | could not run binary: No such file or directory |
| `36_global_constants.sifr` | could not run binary: No such file or directory |

#### 2. Cargo Build Failure (Syntax)

Cargo build fails due to generated Rust code containing syntax errors.

| File | Error |
|------|-------|
| `45_real_world_matrix_ops.sifr` | cargo build failed: expected identifier, found keyword `mod` |

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Borrow-by-Default Regressions (5 tests)

These tests previously passed and broke due to the borrow-by-default codegen changes. Fixing the `&String` vs `String` and mismatched types patterns would recover them immediately.

1. **Mismatched types / &String comparison** (06, 10, 12, 30, 42) — Five previously-passing tests now fail with E0308/E0277 errors from borrow-by-default codegen.

### Tier 2 — Rust Codegen Issues (5 tests)

Tests that progressed past Sifr compilation but hit Rust-level issues. These represent new codegen challenges exposed by recent Sifr compiler improvements.

2. **&String comparison in context managers / calculator** (29, 44) — Same &String vs String pattern but in tests that were already failing at Sifr level.
3. **Borrow of moved value in dict comprehension** (16) — Dict comprehension now compiles in Sifr but generates Rust code with ownership issues.
4. **Walrus operator Display trait** (32) — Result type needs Display implementation for print.
5. **Type annotations needed** (37) — Rust type inference insufficient for conversion code.

### Tier 3 — Runtime / Binary Issues (3 tests)

Tests that pass both Sifr and Rust compilation but fail at runtime.

6. **Binary not found** (15, 36) — Build succeeds but binary location is incorrect.
7. **Generated `mod` keyword** (45) — Matrix ops generates invalid Rust syntax with `mod` keyword.

### Tier 4 — Sifr Limitations (2 tests)

Remaining Sifr-level compilation failures for unsupported language features.

8. **Set type / set comprehension** (17) — `Set` generic type not supported.
9. **`del` on non-collection targets** (31) — `del` restricted to collection items.

---

## Test File Index

| File | Status | Root Cause |
|------|--------|------------|
| `01_arithmetic_full.sifr` | PASS | — |
| `02_comparison_operators.sifr` | PASS | — |
| `03_boolean_logic.sifr` | PASS | — |
| `04_string_methods.sifr` | PASS | — |
| `05_string_formatting.sifr` | PASS | — |
| `06_string_slicing.sifr` | FAIL (Rust) | E0308 mismatched types — REGRESSION |
| `07_list_operations.sifr` | PASS | — (was Sifr fail) |
| `08_dict_operations.sifr` | PASS | — |
| `09_tuple_operations.sifr` | PASS | — |
| `10_control_flow_if.sifr` | FAIL (Rust) | E0277 cannot multiply f64 by i64 — REGRESSION |
| `11_loops_for.sifr` | PASS | — |
| `12_loops_while.sifr` | FAIL (Rust) | E0308 mismatched types — REGRESSION |
| `13_functions_basic.sifr` | PASS | — |
| `14_functions_advanced.sifr` | PASS | — (was Sifr fail: narrowing to Never) |
| `15_list_comprehension.sifr` | FAIL (Runtime) | could not run binary: No such file or directory |
| `16_dict_comprehension.sifr` | FAIL (Rust) | E0382 borrow of moved value: n |
| `17_set_comprehension.sifr` | FAIL (Sifr) | unsupported statement type, unknown Set type |
| `18_builtins.sifr` | PASS | — (was Sifr fail: range iteration) |
| `19_augmented_assignment.sifr` | PASS | — |
| `20_unpacking.sifr` | PASS | — |
| `21_scope_and_closures.sifr` | PASS | — (was Sifr fail: nested functions) |
| `22_lambda_expressions.sifr` | PASS | — |
| `23_generators.sifr` | PASS | — |
| `24_error_handling.sifr` | PASS | — |
| `25_classes_basic.sifr` | PASS | — (was Rust fail: &mut self) |
| `26_classes_inheritance.sifr` | PASS | — |
| `27_classes_static_class_methods.sifr` | PASS | — (was Sifr fail: cls parameter) |
| `28_decorators.sifr` | PASS | — |
| `29_context_managers.sifr` | FAIL (Rust) | E0277 can't compare &String with String |
| `30_assert.sifr` | FAIL (Rust) | E0277 &String comparison, E0308 — REGRESSION |
| `31_del_statement.sifr` | FAIL (Sifr) | del only supported for collection items |
| `32_walrus_operator.sifr` | FAIL (Rust) | E0277 Result<i64, String> doesn't implement Display |
| `33_ternary_expression.sifr` | PASS | — |
| `34_multiple_assignment.sifr` | PASS | — |
| `35_pass_statement.sifr` | PASS | — |
| `36_global_constants.sifr` | FAIL (Runtime) | could not run binary: No such file or directory |
| `37_type_conversions.sifr` | FAIL (Rust) | E0282 type annotations needed |
| `38_multiline_expressions.sifr` | PASS | — |
| `39_nested_data_structures.sifr` | PASS | — |
| `40_real_world_fizzbuzz.sifr` | PASS | — |
| `41_real_world_fibonacci.sifr` | PASS | — |
| `42_real_world_word_count.sifr` | FAIL (Rust) | E0308 mismatched types — REGRESSION |
| `43_real_world_todo_list.sifr` | PASS | — (was Rust fail: borrow as mutable) |
| `44_real_world_calculator.sifr` | FAIL (Rust) | E0277 can't compare &String with String (4×) |
| `45_real_world_matrix_ops.sifr` | FAIL (Runtime) | cargo build failed: expected identifier, found keyword `mod` |
