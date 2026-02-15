# Post-Hardening Audit Report: Python Basics

**Date:** February 15, 2026  
**Scope:** 45 test files in `audit/python_basics/`  
**Context:** Post Language Hardening Phase (Milestones 1–6 completed)

---

## Summary

| Status | Count | Percentage |
|--------|-------|-------------|
| **PASS** | 28 | 62.2% |
| **Fail (Sifr compile)** | 13 | 28.9% |
| **Fail (Rust compile)** | 4 | 8.9% |
| **Total** | 45 | 100% |

---

## Passing Tests (28)

These tests compile and run correctly after the Language Hardening phase:

| # | Test | Notes |
|---|------|-------|
| 01 | `01_arithmetic_full.sifr` | Arithmetic operations |
| 02 | `02_comparison_operators.sifr` | Comparison operators |
| 03 | `03_boolean_logic.sifr` | Boolean logic |
| 04 | `04_string_methods.sifr` | String methods |
| 05 | `05_string_formatting.sifr` | String formatting |
| 06 | `06_string_slicing.sifr` | String slicing |
| 08 | `08_dict_operations.sifr` | Dict operations |
| 09 | `09_tuple_operations.sifr` | Tuple operations |
| 10 | `10_control_flow_if.sifr` | Control flow (if) |
| 11 | `11_loops_for.sifr` | For loops |
| 12 | `12_loops_while.sifr` | While loops |
| 13 | `13_functions_basic.sifr` | Basic functions |
| 19 | `19_augmented_assignment.sifr` | Augmented assignment |
| 20 | `20_unpacking.sifr` | Unpacking |
| 22 | `22_lambda_expressions.sifr` | Lambda expressions |
| 23 | `23_generators.sifr` | Generators |
| 24 | `24_error_handling.sifr` | Error handling |
| 26 | `26_classes_inheritance.sifr` | Class inheritance |
| 28 | `28_decorators.sifr` | Decorators |
| 30 | `30_assert.sifr` | Assert statement |
| 33 | `33_ternary_expression.sifr` | Ternary expression |
| 34 | `34_multiple_assignment.sifr` | Multiple assignment |
| 35 | `35_pass_statement.sifr` | Pass statement |
| 38 | `38_multiline_expressions.sifr` | Multiline expressions |
| 39 | `39_nested_data_structures.sifr` | Nested data structures |
| 40 | `40_real_world_fizzbuzz.sifr` | Real-world: FizzBuzz |
| 41 | `41_real_world_fibonacci.sifr` | Real-world: Fibonacci |
| 42 | `42_real_world_word_count.sifr` | Real-world: Word count |

---

## Failure Categories

### 1. Comprehension over `range` Not Supported

**Error:** `cannot iterate over type 'range'` (in comprehension context)

`range` is not recognized as iterable in comprehension contexts. Regular `for` loops over `range` work (11_loops_for passes), but list comprehensions and similar constructs fail.

| File | Description |
|------|-------------|
| `15_list_comprehension.sifr` | List comprehension over `range` |
| `18_builtins.sifr` | Builtin iteration over `range` |

---

### 2. Dict/Set Comprehension Not Supported

**Error:** `unsupported expression type (dict comprehension not supported)` / `unknown generic type 'Set'`

Dict and set comprehensions are not implemented. Set type is also unknown.

| File | Description |
|------|-------------|
| `16_dict_comprehension.sifr` | Dict comprehension syntax |
| `17_set_comprehension.sifr` | Set comprehension; `Set` generic type unknown |

---

### 3. Nested Functions / Closures

**Error:** `unsupported statement type (nested function)`; `undefined variable 'x'`; `undefined function 'inner'`

Nested function definitions and closure capture are not supported. Inner functions cannot reference outer scope variables.

| File | Description |
|------|-------------|
| `21_scope_and_closures.sifr` | Nested function, closure over `x`, call to `inner` |

---

### 4. List Operations: `remove()` Missing, `+` Concatenation Not Supported

**Error:** `list has no method 'remove'`; `unsupported operand type(s) for +: 'list[int]' and 'list[int]'`

List methods and operators are incomplete. `list.remove()` is missing; list concatenation via `+` is not supported. Also: `undefined variable 'c'` (likely from a comprehension or loop scope).

| File | Description |
|------|-------------|
| `07_list_operations.sifr` | `list.remove()`, `list + list`, variable scope |

---

### 5. `cls` Parameter in `@classmethod`

**Error:** `undefined function 'cls'`

The `cls` parameter in `@classmethod` is not recognized. The type checker treats `cls` as a function call rather than the class reference parameter.

| File | Description |
|------|-------------|
| `27_classes_static_class_methods.sifr` | `@classmethod` with `cls` parameter |

---

### 6. Module-Level Constants Not Accessible

**Error:** `undefined variable 'PI'`, `'APP_NAME'`, `'MAX_RETRIES'`

Module-level constants (uppercase variables at top level) are not accessible. May be a scoping or name-resolution issue for top-level bindings.

| File | Description |
|------|-------------|
| `36_global_constants.sifr` | `PI`, `APP_NAME`, `MAX_RETRIES` at module level |

---

### 7. Narrowing Over-Narrows to `Never`

**Error:** `cannot compare 'Never' and 'str' with ==`; `unsupported operand type(s) for -: 'Never' and 'int'`

After branching, the type system narrows variables to `Never` in subsequent branches, making further comparisons or operations impossible.

| File | Description |
|------|-------------|
| `14_functions_advanced.sifr` | Arithmetic with `Never` after branching |
| `44_real_world_calculator.sifr` | String comparison with `Never` in calculator dispatch |

---

### 8. `del` Statement Limitations

**Error:** `del is only supported for collection items`

`del` is restricted to collection item deletion (e.g., `del lst[i]`). Deleting variables or other targets is not supported.

| File | Description |
|------|-------------|
| `31_del_statement.sifr` | `del` on variables or unsupported targets |

---

### 9. Walrus Operator with Optional Types

**Error:** `cannot compare 'int | None' and 'int' with ==`

After `:=` (walrus operator), the assigned variable may be `int | None`. Narrowing or comparison with `int` fails.

| File | Description |
|------|-------------|
| `32_walrus_operator.sifr` | Walrus in condition; comparison of optional with concrete type |

---

### 10. Methods Need `&mut self` (Rust Codegen)

**Error:** `cannot assign to 'self.count', which is behind a '&' reference`

Instance methods that mutate `self` are emitted with `&self` instead of `&mut self`. Rust rejects mutation through an immutable reference.

| File | Description |
|------|-------------|
| `25_classes_basic.sifr` | Method mutating `self.count` |

---

### 11. Context Managers: Variable Scope in `with` Block

**Error:** `cannot find value 'conn' in this scope`

Variables bound in a `with` block (e.g., `conn` from `with open(...) as conn`) are not in scope in the generated Rust code.

| File | Description |
|------|-------------|
| `29_context_managers.sifr` | `with ... as conn`; `conn` not found in scope |

---

### 12. Type Annotations Needed (Rust Inference)

**Error:** `type annotations needed (E0282)`

Rust cannot infer types in some generated code. Explicit type annotations or better codegen are required.

| File | Description |
|------|-------------|
| `37_type_conversions.sifr` | Type conversion expressions need annotations |

---

### 13. `len()` on Optional / Use of Moved Value

**Error:** `len() argument must be a string/list/dict/tuple, got 'list[int] | None'`; `use of moved value`

`len()` is called on an optional value without narrowing. Also, a value is used after being moved.

| File | Description |
|------|-------------|
| `45_real_world_matrix_ops.sifr` | `len()` on optional list; ownership/move issue |

---

### 14. Cannot Borrow as Mutable (Rust)

**Error:** `mismatched types`; `cannot borrow 'item' as mutable`

Generated Rust code attempts to borrow a value as mutable when it is not allowed.

| File | Description |
|------|-------------|
| `43_real_world_todo_list.sifr` | Mutating `item` in loop; borrow checker rejects |

---

## What Was Fixed by Language Hardening

The Language Hardening phase (Milestones 1–6) brought **28 tests** to passing. Notable improvements:

| Milestone | Fixes | Tests Unblocked |
|-----------|-------|-----------------|
| **M1: Codegen fixes** | Tuple indexing, int/int division, print(None), escaped quotes | Various |
| **M3: Ownership v2** | print consumes, string method moves, list mutation after use, chained operations | 04, 08, 12, 23, 39 |
| **M4–M6** | Subscript mutation, iteration, builtins | 11, 18 (partial), 22 |

**Notable gains:**
- **04, 08** — String and dict operations work after ownership fixes.
- **11, 12** — For/while loops and iteration.
- **22, 23** — Lambda expressions and generators.
- **24** — Error handling (try/except).
- **26, 28** — Class inheritance and decorators.
- **39** — Nested data structures.
- **40, 41, 42** — Real-world FizzBuzz, Fibonacci, word count.

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Blocks Real-World Usage

1. **Comprehension over `range`** (15, 18) — List comprehensions and builtin iteration over `range` fail. Very common Python idiom.
2. **Nested functions / closures** (21) — Inner functions and closure capture are unsupported. Limits higher-order patterns.
3. **`&mut self` for methods** (25) — Instance methods that mutate state fail at Rust compile. Core OOP behavior.
4. **Narrowing over-narrows to `Never`** (14, 44) — Multi-branch dispatch (e.g., calculator) breaks. Affects control flow.

### Tier 2 — Significant Ergonomics

5. **Dict/set comprehensions** (16, 17) — Dict and set comprehensions plus `Set` type. Common data transformation.
6. **List operations** (07) — `list.remove()`, list `+` concatenation. Basic list manipulation.
7. **`cls` in `@classmethod`** (27) — Class methods cannot use `cls`. Factory patterns affected.
8. **Module-level constants** (36) — Top-level constants not accessible. Configuration and constants pattern.
9. **Context managers** (29) — `with` block variable scope. Resource management pattern.

### Tier 3 — Edge Cases / Nice to Have

10. **Walrus operator with optional** (32) — Comparison after `:=` with optional type.
11. **`del` statement** (31) — Only collection-item `del` supported.
12. **`len()` on optional / move** (45) — Optional handling and ownership in matrix ops.
13. **Borrow as mutable** (43) — Todo list mutation in loop.
14. **Type annotations** (37) — Rust type inference in conversion code.

---

## Test File Index (Post-Hardening)

| File | Status | Root Cause |
|------|--------|------------|
| `01_arithmetic_full.sifr` | PASS | — |
| `02_comparison_operators.sifr` | PASS | — |
| `03_boolean_logic.sifr` | PASS | — |
| `04_string_methods.sifr` | PASS | — |
| `05_string_formatting.sifr` | PASS | — |
| `06_string_slicing.sifr` | PASS | — |
| `07_list_operations.sifr` | FAIL (Sifr) | list.remove(), list +, undefined var |
| `08_dict_operations.sifr` | PASS | — |
| `09_tuple_operations.sifr` | PASS | — |
| `10_control_flow_if.sifr` | PASS | — |
| `11_loops_for.sifr` | PASS | — |
| `12_loops_while.sifr` | PASS | — |
| `13_functions_basic.sifr` | PASS | — |
| `14_functions_advanced.sifr` | FAIL (Sifr) | Narrowing to Never |
| `15_list_comprehension.sifr` | FAIL (Sifr) | range in comprehension |
| `16_dict_comprehension.sifr` | FAIL (Sifr) | Dict comprehension |
| `17_set_comprehension.sifr` | FAIL (Sifr) | Set comprehension, Set type |
| `18_builtins.sifr` | FAIL (Sifr) | range iteration |
| `19_augmented_assignment.sifr` | PASS | — |
| `20_unpacking.sifr` | PASS | — |
| `21_scope_and_closures.sifr` | FAIL (Sifr) | Nested functions, closures |
| `22_lambda_expressions.sifr` | PASS | — |
| `23_generators.sifr` | PASS | — |
| `24_error_handling.sifr` | PASS | — |
| `25_classes_basic.sifr` | FAIL (Rust) | &mut self |
| `26_classes_inheritance.sifr` | PASS | — |
| `27_classes_static_class_methods.sifr` | FAIL (Sifr) | cls parameter |
| `28_decorators.sifr` | PASS | — |
| `29_context_managers.sifr` | FAIL (Rust) | with block scope |
| `30_assert.sifr` | PASS | — |
| `31_del_statement.sifr` | FAIL (Sifr) | del limitations |
| `32_walrus_operator.sifr` | FAIL (Sifr) | Walrus + optional |
| `33_ternary_expression.sifr` | PASS | — |
| `34_multiple_assignment.sifr` | PASS | — |
| `35_pass_statement.sifr` | PASS | — |
| `36_global_constants.sifr` | FAIL (Sifr) | Module-level constants |
| `37_type_conversions.sifr` | FAIL (Rust) | Type annotations |
| `38_multiline_expressions.sifr` | PASS | — |
| `39_nested_data_structures.sifr` | PASS | — |
| `40_real_world_fizzbuzz.sifr` | PASS | — |
| `41_real_world_fibonacci.sifr` | PASS | — |
| `42_real_world_word_count.sifr` | PASS | — |
| `43_real_world_todo_list.sifr` | FAIL (Rust) | Borrow as mutable |
| `44_real_world_calculator.sifr` | FAIL (Sifr) | Narrowing to Never |
| `45_real_world_matrix_ops.sifr` | FAIL (Sifr) | len() on optional, move |
