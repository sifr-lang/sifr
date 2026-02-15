# Sifr Python Basics Audit Report

**Date:** February 15, 2026
**Methodology:** 45 test files compiled and run against the Sifr compiler to probe core Python language functionality.
**Scope:** Everyday Python features -- arithmetic, strings, collections, control flow, functions, classes, comprehensions, generators, error handling, and real-world programs. Does not duplicate the type system or type inference audits.

---

## Executive Summary

**22 PASS / 23 FAIL** out of 45 tests.

The core language fundamentals are solid -- basic arithmetic, comparisons, boolean logic, string formatting, slicing, for/while loops, functions with defaults and recursion, unpacking, lambdas, generators, error handling, ternary expressions, assert, pass, decorators, and multiline expressions all work. However, there are **15 distinct issues** that prevent writing normal Python-style code, ranging from missing features to codegen bugs.

---

## What Works Well (PASS)

| # | Test | Python Feature | Status |
|---|------|---------------|--------|
| 02 | Comparison operators | `==`, `!=`, `<`, `>`, `<=`, `>=`, `is`, `in`, chained | PASS |
| 03 | Boolean logic | `and`, `or`, `not`, compound expressions | PASS |
| 05 | String formatting | f-strings, `str()`, string multiply, concatenation | PASS |
| 06 | String slicing | `s[0]`, `s[-1]`, `s[1:3]`, `s[::-1]`, `len()` | PASS |
| 10 | If/elif/else | Multi-branch, nested if, integer comparisons | PASS |
| 11 | For loops | `range()`, list iteration, enumerate, nested for | PASS |
| 12 | While loops | Basic while, break, continue, countdown | PASS |
| 13 | Functions basic | Positional args, defaults, tuple return, nested calls | PASS |
| 14 | Functions advanced | Keyword args, recursion (factorial, fibonacci, accumulator) | PASS |
| 20 | Unpacking | Tuple unpacking, swap, star unpacking | PASS |
| 22 | Lambda expressions | `map`, `filter`, chained lambdas | PASS |
| 23 | Generators | `yield`, generator with filter, fibonacci generator | PASS |
| 24 | Error handling | `Result`, `try/except`, `raise`, multiple try blocks | PASS |
| 26 | Class inheritance | `super().__init__()`, method override, field access | PASS |
| 28 | Decorators | `@log` pass-through decorator | PASS |
| 30 | Assert | `assert True`, `assert expr`, assert with message | PASS |
| 33 | Ternary expression | `x if cond else y`, nested ternary, in f-string | PASS |
| 35 | Pass statement | `pass` in function, class, if block | PASS |
| 38 | Multiline expressions | Parenthesized multiline, nested function calls | PASS |
| 40 | FizzBuzz | Real-world: elif chain with modulo | PASS |
| 41 | Fibonacci | Real-world: recursive + iterative, tuple swap in loop | PASS |

---

## Issues Found

### Issue 1: Mixed `int`/`float` Arithmetic Not Supported (Critical)

**Test:** 01
**Rust error:** `cannot add f64 to i64`

Python automatically promotes `int` to `float` in mixed arithmetic (`10 + 3.5` = `13.5`). Sifr does not -- the generated Rust tries `i64 + f64` which fails.

```python
# FAILS
print(10 + 3.5)    # Rust: 10_i64 + 3.5_f64 -- type mismatch
print(2 * 3.14)    # Rust: 2_i64 * 3.14_f64 -- type mismatch
```

**Impact:** Any computation mixing integers and floats fails. This is extremely common in Python.

---

### Issue 2: Unary `+` Operator Not Supported (Low)

**Test:** 01
**Rust error:** `expected expression, found +`

`+x` (unary plus) generates invalid Rust. Unary minus (`-x`) works fine.

---

### Issue 3: String Methods Trigger Move Semantics (High)

**Test:** 04
**Error:** `use of moved value: 'parts'`

Calling `.split()` on a string and then using the result (e.g., passing to `.join()`) fails because the result is moved on first use.

```python
# FAILS
parts = "a,b,c".split(",")
print(parts)                # moves parts
joined = ", ".join(parts)   # Error: use of moved value 'parts'
```

**Impact:** Cannot use string method results more than once. Already documented in type system audit as the move semantics issue, but it severely impacts basic string operations.

---

### Issue 4: List Mutation After Use Triggers Move (High)

**Test:** 07
**Error:** `use of moved value: 'nums'`

After calling `.append()` on a list, further operations on the same list fail.

```python
# FAILS
nums.append(9)
print(nums)       # moves nums
nums.insert(0, 0) # Error: use of moved value 'nums'
```

---

### Issue 5: Dict Subscript Assignment Not Supported (Critical)

**Test:** 08, 42
**Error:** `assignment target must be a simple name`

Cannot assign to dict keys with `d["key"] = value`. This is one of the most fundamental dict operations in Python.

```python
# FAILS
d["d"] = 4  # Error: assignment target must be a simple name
```

**Impact:** Cannot add or update dictionary entries. Dict is essentially read-only after creation.

---

### Issue 6: Tuple `len()` Triggers Move (Medium)

**Test:** 09
**Error:** `use of moved value: 'pair'`

Printing a tuple and then accessing it again fails due to move semantics.

---

### Issue 7: List Comprehension Over `range()` Fails (Critical)

**Test:** 15, 18
**Error:** `cannot iterate over type 'range'`

List comprehensions like `[x * x for x in range(6)]` fail because `range` is not recognized as iterable inside comprehensions. Regular `for x in range(n)` loops work fine.

```python
# FAILS
squares = [x * x for x in range(6)]  # Error: cannot iterate over type 'range'

# WORKS
for x in range(6):
    print(x * x)
```

**Impact:** A very common Python pattern is broken. List comprehensions only work with list iterables, not `range()`.

---

### Issue 8: Dict Comprehension Not Supported (High)

**Test:** 16
**Error:** `unsupported expression type`

Dict comprehensions (`{k: v for k, v in ...}`) are not recognized by the compiler.

```python
# FAILS
squares = {x: x * x for x in range(5)}  # Error: unsupported expression type
```

---

### Issue 9: Set Type / `from` Import Not Supported (Medium)

**Test:** 17
**Error:** `unsupported statement type` (for `from sifr.collections import Set`)

The `from X import Y` syntax is not supported. Sets may exist in the stdlib but cannot be imported this way.

---

### Issue 10: `pow()` Built-in Not Defined (Low)

**Test:** 18
**Error:** `undefined function: 'pow'`

The `pow(base, exp)` built-in is not available. The `**` operator works as a workaround.

---

### Issue 11: `**=` Augmented Power Assignment Codegen Bug (Medium)

**Test:** 19
**Rust error:** `expected i64, found f64`

`x **= 3` generates `(x as f64).powf(3 as f64)` which returns `f64`, but the variable is `i64`.

---

### Issue 12: Module-Level Variables / Global Constants Not Accessible (Critical)

**Test:** 21, 36
**Error:** `undefined variable: 'PI'` / `undefined variable: 'x'`

Variables defined at module level (outside `main()`) cannot be accessed from functions. Python supports module-level constants freely.

```python
# FAILS
PI: float = 3.14159

def circle_area(r: float) -> float:
    return PI * r * r  # Error: undefined variable 'PI'
```

**Impact:** Cannot define constants, configuration values, or shared state at module level. Every value must be passed as a function parameter.

---

### Issue 13: Nested Function Definitions Not Supported (Medium)

**Test:** 21
**Error:** `unsupported statement type`

Cannot define a function inside another function (closures/nested functions).

```python
# FAILS
def outer() -> int:
    def inner() -> int:  # Error: unsupported statement type
        return 5
    return inner()
```

---

### Issue 14: `@classmethod` with `cls` Not Supported (Medium)

**Test:** 27
**Error:** `undefined function: 'cls'`

Class methods decorated with `@classmethod` that use `cls(...)` to construct instances fail because `cls` is not recognized.

```python
# FAILS
@classmethod
def from_fahrenheit(cls, f: float) -> Temperature:
    return cls((f - 32.0) * 5.0 / 9.0)  # Error: undefined function 'cls'
```

---

### Issue 15: `with` Statement Variable Not Accessible (Medium)

**Test:** 29
**Rust error:** `cannot find value 'conn' in this scope`

The `with X as name:` syntax parses but the bound variable (`conn`) is not accessible inside the block.

```python
# FAILS at Rust build
with Connection("db") as conn:
    result = conn.query()  # Rust: cannot find value 'conn'
```

---

### Issue 16: `del` Only Works on Collection Items (Low)

**Test:** 31
**Error:** `del is only supported for collection items`

`del x` (delete a variable) is not supported. Only `del d["key"]` and `del lst[i]` work.

---

### Issue 17: Chained Assignment Not Supported (Low)

**Test:** 34
**Error:** `multiple assignment targets not supported yet`

`x = y = z = 0` is not supported. Must assign each variable separately.

---

### Issue 18: `bool()` on Collections Codegen Bug (Medium)

**Test:** 37
**Rust error:** `` `Vec<i64>` doesn't implement `std::fmt::Display` ``

`bool([1, 2])` and `bool([])` fail because the codegen doesn't properly handle truthiness conversion for collections.

---

### Issue 19: `self.field += value` Not Supported (High)

**Test:** 25, 43
**Error:** `augmented assignment target must be a simple name`

Cannot use augmented assignment on class fields. `self.count += 1` fails.

```python
# FAILS
def increment(self) -> None:
    self.count += 1  # Error: augmented assignment target must be a simple name
```

**Impact:** Cannot write mutable classes with increment/decrement patterns. Must use `self.count = self.count + 1` as a workaround (if that even works).

---

### Issue 20: `elif` Equality Chain in Functions Returning Result (Known)

**Test:** 44
**Error:** `cannot compare 'Never' and 'str' with ==`

Already documented in type system audit. The calculator example hits the `elif` equality narrowing bug.

---

### Issue 21: Safe Indexing in Nested Operations (Medium)

**Test:** 45
**Error:** `len() argument must be a string, list, dict, or tuple, got 'list[int] | None'`

`a[i][j]` returns `list[int] | None` from the first index, and then `len()` doesn't accept an optional type. Need to unwrap the first index before using the result.

---

## Priority Ranking

### Tier 1 -- Must Fix (Blocks Normal Python Code)

1. **Mixed int/float arithmetic** (Issue 1) -- `10 + 3.5` must work. Python's most basic feature.
2. **Dict subscript assignment** (Issue 5) -- `d["key"] = value` is fundamental to dict usage.
3. **List comprehension over range()** (Issue 7) -- `[x for x in range(n)]` is one of Python's most used patterns.
4. **Module-level variables** (Issue 12) -- Constants and shared state must be accessible from functions.
5. **`self.field += value`** (Issue 19) -- Mutable class fields are essential for OOP.

### Tier 2 -- Should Fix (Significant Ergonomics)

6. **Move semantics on collections/strings** (Issues 3, 4, 6) -- Using a value more than once must work.
7. **Dict comprehension** (Issue 8) -- Common Python pattern.
8. **Nested functions** (Issue 13) -- Closures and helper functions inside functions.
9. **`@classmethod` with `cls`** (Issue 14) -- Factory methods are a common pattern.
10. **`with` statement variable binding** (Issue 15) -- Context managers must bind the variable.

### Tier 3 -- Nice to Have

11. **`from X import Y`** (Issue 9) -- Import syntax.
12. **`pow()` built-in** (Issue 10) -- `**` works as workaround.
13. **`**=` codegen** (Issue 11) -- Power augmented assignment.
14. **`del` on variables** (Issue 16) -- Rarely needed.
15. **Chained assignment** (Issue 17) -- `x = y = 0`.
16. **`bool()` on collections** (Issue 18) -- Truthiness conversion.
17. **Unary `+`** (Issue 2) -- Rarely used.

---

## Test File Index

| File | Tests | Result |
|------|-------|--------|
| `01_arithmetic_full.sifr` | `+`, `-`, `*`, `/`, `//`, `%`, `**`, unary, mixed int/float | FAIL (Issues 1, 2) |
| `02_comparison_operators.sifr` | `==`, `!=`, `<`, `>`, `<=`, `>=`, `is`, `in`, chained | PASS |
| `03_boolean_logic.sifr` | `and`, `or`, `not`, compound, short-circuit | PASS |
| `04_string_methods.sifr` | upper, lower, strip, split, join, replace, find, startswith | FAIL (Issue 3) |
| `05_string_formatting.sifr` | f-strings, `str()`, string multiply, concatenation | PASS |
| `06_string_slicing.sifr` | Indexing, negative index, slicing, step, reverse, len | PASS |
| `07_list_operations.sifr` | append, insert, pop, remove, sort, reverse, count, slice | FAIL (Issue 4) |
| `08_dict_operations.sifr` | get, keys, values, items, subscript assign, del, pop, clear | FAIL (Issue 5) |
| `09_tuple_operations.sifr` | Indexing, unpacking, len, nested, multiple return | FAIL (Issue 6) |
| `10_control_flow_if.sifr` | if/elif/else, nested if, multi-branch | PASS |
| `11_loops_for.sifr` | range, list iteration, enumerate, nested for | PASS |
| `12_loops_while.sifr` | while, break, continue, countdown | PASS |
| `13_functions_basic.sifr` | Positional, defaults, tuple return, nested calls | PASS |
| `14_functions_advanced.sifr` | Keyword args, recursion, accumulator | PASS |
| `15_list_comprehension.sifr` | `[x for x in range()]`, filter, nested | FAIL (Issue 7) |
| `16_dict_comprehension.sifr` | `{k: v for ...}` | FAIL (Issue 8) |
| `17_set_comprehension.sifr` | Set type, `from` import | FAIL (Issue 9) |
| `18_builtins.sifr` | len, abs, min, max, sum, sorted, reversed, enumerate, zip, range, conversions, round, pow | FAIL (Issues 7, 10) |
| `19_augmented_assignment.sifr` | `+=`, `-=`, `*=`, `//=`, `%=`, `**=` | FAIL (Issue 11) |
| `20_unpacking.sifr` | Tuple unpack, swap, star unpack | PASS |
| `21_scope_and_closures.sifr` | Module-level vars, nested functions, shadowing | FAIL (Issues 12, 13) |
| `22_lambda_expressions.sifr` | map, filter, chained lambdas | PASS |
| `23_generators.sifr` | yield, generator with filter, fibonacci | PASS |
| `24_error_handling.sifr` | Result, try/except, raise, multiple blocks | PASS |
| `25_classes_basic.sifr` | Fields, methods, `__str__`, mutable state | FAIL (Issue 19) |
| `26_classes_inheritance.sifr` | super(), method override, field access | PASS |
| `27_classes_static_class_methods.sifr` | @staticmethod, @classmethod | FAIL (Issue 14) |
| `28_decorators.sifr` | @log pass-through | PASS |
| `29_context_managers.sifr` | `with X as name:`, nested with | FAIL (Issue 15) |
| `30_assert.sifr` | assert, assert with message | PASS |
| `31_del_statement.sifr` | del dict key, del variable | FAIL (Issue 16) |
| `32_walrus_operator.sifr` | `:=` in if and while | FAIL (walrus + safe indexing interaction) |
| `33_ternary_expression.sifr` | `x if cond else y`, nested, in f-string | PASS |
| `34_multiple_assignment.sifr` | `a, b, c = 1, 2, 3`, chained `x = y = 0` | FAIL (Issue 17) |
| `35_pass_statement.sifr` | pass in function, class, if | PASS |
| `36_global_constants.sifr` | Module-level constants | FAIL (Issue 12) |
| `37_type_conversions.sifr` | int(), float(), str(), bool() | FAIL (Issue 18) |
| `38_multiline_expressions.sifr` | Parenthesized multiline, nested calls | PASS |
| `39_nested_data_structures.sifr` | List of dicts, dict of lists, matrix | FAIL (Issue 4) |
| `40_real_world_fizzbuzz.sifr` | FizzBuzz | PASS |
| `41_real_world_fibonacci.sifr` | Recursive + iterative fibonacci | PASS |
| `42_real_world_word_count.sifr` | Word frequency counter | FAIL (Issue 5) |
| `43_real_world_todo_list.sifr` | Class-based todo app | FAIL (Issues 19, codegen) |
| `44_real_world_calculator.sifr` | Calculator with elif dispatch | FAIL (Issue 20) |
| `45_real_world_matrix_ops.sifr` | Matrix addition | FAIL (Issues 4, 21) |
