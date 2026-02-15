# Sifr Type Inference Audit Report

**Date:** February 15, 2026
**Methodology:** 30 test files compiled and run against the Sifr compiler to probe type inference capabilities against TypeScript's inference engine.
**Scope:** Purely inference -- where the compiler should figure out types without the programmer spelling them out. Does not duplicate the type system audit (unions, narrowing, generics, protocols, etc.).

---

## Executive Summary

**18 PASS / 12 FAIL** out of 30 tests.

Sifr's type inference is **solid for the basics** -- it correctly infers variable types from literals, expressions, function calls, collection literals, comprehensions, map/filter, class constructors, loop variables, ternary expressions, walrus operators, builtins, zip/enumerate, and any/all. However, there are **7 distinct inference issues** that limit the experience compared to TypeScript.

---

## What Works Well (PASS)

| # | Test | What's Inferred | Status |
|---|------|----------------|--------|
| 02 | Variable from expression | `int` from `1 + 2`, `float` from `2.5 * 4.0`, `str` from concat, `bool` from comparison | PASS |
| 03 | Variable from function call | Type from return type annotation | PASS |
| 04 | Collection literal inference | `list[int]` from `[1,2,3]`, `dict[str,int]` from `{"a": 1}`, `tuple` from `(42, "hi")` | PASS |
| 06 | From method call | `str` from `.upper()`, `int` from `len()`, `list[int]` from `sorted()` | PASS |
| 07 | From conditional/ternary | `str` from `"yes" if True else "no"` | PASS |
| 08 | From list comprehension | `list[int]` from `[x*x for x in nums]`, `list[str]`, `list[bool]` | PASS |
| 09 | From map/filter | `list[int]` from `map(lambda x: x*2, nums)` | PASS |
| 10 | From class constructor | `Point` from `Point(3.0, 4.0)` | PASS |
| 11 | In for loop | Loop variable type from iterable element type | PASS |
| 13 | Full program, no var annotations | All locals inferred from function calls, arithmetic, builtins | PASS |
| 15 | From optional return | `str \| None` from function returning `str \| None` | PASS |
| 19 | Reassignment same type | `int` stays `int` after `x = 1; x = 2` | PASS |
| 21 | From builtin functions | `int` from `len()`, `abs()`, `min()`, `max()`, `sum()`; `str` from `str()`; `bool` from `bool()` | PASS |
| 22 | Nested collections | `list[list[int]]` from `[[1,2],[3,4]]`, `dict[str, list[int]]`, `list[tuple[str,int]]` | PASS |
| 25 | From walrus operator | `int` from `(n := len(items))` | PASS |
| 26 | Lambda param from context | Lambda param inferred as `int` from `list[int]` context in `map` | PASS |
| 28 | From zip/enumerate | `list[tuple[str,int]]` from `zip(names, ages)` | PASS |
| 29 | From any/all | `bool` from `any(...)` and `all(...)` | PASS |

---

## Issues Found

### Issue 1: No Return Type Inference (High)

**Test:** 05
**Error:** `type error: return type mismatch: expected 'None', got 'int'`

When a function omits the `-> ReturnType` annotation, the compiler defaults to `-> None` instead of inferring the return type from `return` statements. TypeScript infers return types from all return paths.

```python
# FAILS -- compiler assumes -> None
def add(a: int, b: int):
    return a + b  # Error: expected 'None', got 'int'

# WORKS -- explicit annotation
def add(a: int, b: int) -> int:
    return a + b
```

**TypeScript equivalent that works:**
```typescript
function add(a: number, b: number) { return a + b; }  // inferred as number
```

**Impact:** Every function must have an explicit return type annotation. This is the biggest ergonomic gap vs TypeScript, where return type annotations are almost always optional.

---

### Issue 2: `print(None)` Fails -- `()` Not Display-able (Medium)

**Test:** 01
**Rust error:** `` `()` cannot be formatted with the default formatter ``

Inferring a variable as `None` and then printing it fails because `None` maps to `()` in Rust, which doesn't implement `Display`.

```python
# FAILS at Rust build
e = None
print(e)  # Rust: println!("{}", e) where e: () -- () has no Display
```

**Impact:** Cannot print `None` values. This is a codegen issue -- the compiler should emit `println!("None")` or use `Debug` formatting for unit type.

---

### Issue 3: Union Return Values Not Wrapped in Enum Variant (High)

**Test:** 16
**Rust error:** `expected IntOrStr, found i64` -- suggests `try wrapping in IntOrStr::Int`

When a function returns a union type (`int | str`) and the inferred variable receives it, the codegen doesn't wrap return values in the generated enum variant.

```python
# FAILS at Rust build
def parse_input(s: str) -> int | str:
    if s == "42":
        return 42  # Rust: return 42_i64 -- should be IntOrStr::Int(42_i64)
    return s
```

**Impact:** Functions returning union types (inferred at call site) fail at Rust compilation. This is a codegen bug -- return values must be wrapped in the appropriate enum variant.

---

### Issue 4: `print(Result)` Fails -- Result Not Display-able (Medium)

**Test:** 17
**Rust error:** `` `Result<i64, String>` doesn't implement `std::fmt::Display` ``

When a `try` block infers the variable type from a `Result`-returning function, printing the value inside the `try` block fails because the inferred type is `Result<T, E>` rather than the unwrapped `T`.

```python
# FAILS at Rust build
try:
    val = parse_int("42")  # val inferred as Result[int, str]? Should be int
    print(val)             # Rust: println!("{}", val) where val: Result<i64, String>
except str as e:
    print(e)
```

**Impact:** Type inference inside `try` blocks doesn't properly unwrap `Result` to the success type. The variable should be inferred as `int`, not `Result[int, str]`.

---

### Issue 5: Tuple Literal Index Codegen Bug (Medium)

**Test:** 24
**Rust error:** `no field '0_' on type (String, i64)` -- suggests `pair.0` instead of `pair.0_i64`

When indexing a tuple with a literal integer (`pair[0]`), the codegen emits `pair.0_i64` instead of `pair.0`.

```python
pair: tuple[str, int] = ("hello", 42)
a = pair[0]  # Rust: pair.0_i64 -- should be pair.0
```

**Impact:** Tuple indexing with inferred result type fails at Rust build. The codegen appends the integer suffix `_i64` to the tuple field access.

---

### Issue 6: `int / int` Inferred as `float` but Codegen Emits `i64` (Medium)

**Test:** 27
**Rust error:** `expected f64, found i64`

When `total / len(items)` is computed (both `int`), the type system infers `float` but the codegen emits `i64` division, creating a mismatch when the result is assigned to an inferred `float` variable.

```python
total = sum(items)       # int
avg = total / len(items) # Sifr infers float, but Rust emits i64 / i64 = i64
```

**Impact:** Division between inferred `int` values doesn't properly coerce to `float` in the generated Rust.

---

### Issue 7: No Type Widening on Reassignment (Expected Behavior, but Worth Noting)

**Test:** 20
**Error:** `type error: type mismatch: cannot assign 'str' to variable 'x' of type 'int'`

Once a variable's type is inferred, it cannot be reassigned to a different type. This is **correct behavior** for a statically typed language (TypeScript also rejects this in strict mode), but worth documenting.

```python
x = 42       # inferred as int
x = "hello"  # Error: cannot assign str to int
```

**Note:** This is actually the right design choice for Sifr. TypeScript also rejects this unless the variable is explicitly typed as `number | string`.

---

### Existing Issues Surfaced Again (From Type System Audit)

These failures are caused by issues already documented in the type system audit, not new inference bugs:

| Test | Error | Root Cause |
|------|-------|------------|
| 12 | `use of moved value: 'doubled'` | Ownership/move semantics (Type System Issue 4) |
| 14 | `use of moved value: 'msg'` | Ownership/move semantics (Type System Issue 4) |
| 18 | `use of moved value: 'empty_list'` | Ownership/move semantics (Type System Issue 4) |
| 23 | `use of moved value: 'name'` | Ownership/move semantics (Type System Issue 4) |
| 30 | `use of moved value: 'parts'` | Ownership/move semantics (Type System Issue 4) |

These are all the same underlying problem: strings and collections are moved on use, preventing subsequent access. This is an ownership tracking issue, not an inference issue.

---

## Missing TypeScript Inference Features

| Feature | TypeScript Behavior | Sifr Status |
|---------|-------------------|-------------|
| Return type inference | Inferred from all return paths | **Not implemented** -- defaults to `None` |
| Contextual typing (callbacks) | `arr.map(x => x + 1)` infers `x` as element type | **Works** for built-in `map`/`filter` |
| Generic inference | `identity(42)` infers `T = number` | **Not applicable** -- no generics |
| Best common type | `[1, "hello"]` infers `(number \| string)[]` | **Not tested** -- likely fails (heterogeneous lists) |
| Control flow inference | Type narrows through if/else | **Works** (covered in type system audit) |
| Destructuring inference | `let { name, age } = user` infers types | **Not tested** -- Sifr has tuple unpacking but not object destructuring |
| `typeof` inference | `if (typeof x === "string")` narrows | **N/A** -- Sifr uses `isinstance` |
| Satisfies inference | `config satisfies Config` preserves literal types | **Not implemented** |
| `as const` inference | `[1, 2, 3] as const` infers readonly tuple | **Not implemented** |

---

## Priority Ranking

### Tier 1 -- Must Fix

1. **Return type inference** (Issue 1) -- Every function requiring explicit return type is a major ergonomic burden. TypeScript's biggest inference win is that return types are almost never needed.
2. **Union return value wrapping** (Issue 3) -- Functions returning union types silently fail at Rust build. Codegen must wrap return values in enum variants.

### Tier 2 -- Should Fix

3. **`print(None)` codegen** (Issue 2) -- Printing `None` should work.
4. **Result unwrapping in try blocks** (Issue 4) -- Variables in `try` blocks should be inferred as the success type, not `Result`.
5. **Tuple index codegen** (Issue 5) -- `pair[0]` should emit `pair.0`, not `pair.0_i64`.
6. **`int / int` codegen** (Issue 6) -- Division result type and generated Rust must agree.

### Tier 3 -- Nice to Have

7. **Type widening on reassignment** (Issue 7) -- Current behavior (reject) is correct for a strict language. No change needed.

---

## Test File Index

| File | Tests | Result |
|------|-------|--------|
| `01_variable_from_literal.sifr` | int, float, str, bool, None literals | FAIL (Issue 2) |
| `02_variable_from_expression.sifr` | Arithmetic, concat, comparison, logic | PASS |
| `03_variable_from_function_call.sifr` | Infer from return type | PASS |
| `04_collection_literal_inference.sifr` | list, dict, tuple literals | PASS |
| `05_return_type_inference.sifr` | Omitted return type annotation | FAIL (Issue 1) |
| `06_infer_from_method_call.sifr` | .upper(), len(), sorted() | PASS |
| `07_infer_from_conditional.sifr` | Ternary expressions | PASS |
| `08_infer_from_comprehension.sifr` | List comprehensions | PASS |
| `09_infer_from_map_filter.sifr` | map/filter with lambda | PASS |
| `10_infer_from_class_constructor.sifr` | Class instantiation | PASS |
| `11_infer_in_for_loop.sifr` | Loop variable from iterable | PASS |
| `12_infer_chained_operations.sifr` | Chained filter/map/len | FAIL (move semantics) |
| `13_infer_mixed_no_annotation.sifr` | Full program, no var annotations | PASS |
| `14_infer_from_fstring.sifr` | F-string result type | FAIL (move semantics) |
| `15_infer_from_optional_return.sifr` | `str \| None` from function | PASS |
| `16_infer_from_union_return.sifr` | `int \| str` from function | FAIL (Issue 3) |
| `17_infer_from_result_return.sifr` | `Result[int, str]` in try block | FAIL (Issue 4) |
| `18_infer_empty_collection.sifr` | Empty list/dict | FAIL (move semantics) |
| `19_infer_reassignment_same_type.sifr` | Reassign same type | PASS |
| `20_infer_reassignment_different_type.sifr` | Reassign different type | FAIL (Issue 7 -- correct) |
| `21_infer_from_builtin_functions.sifr` | len, abs, min, max, sum, str, bool | PASS |
| `22_infer_nested_collection.sifr` | Nested list/dict/tuple | PASS |
| `23_infer_class_field_access.sifr` | Field type from class | FAIL (move semantics) |
| `24_infer_from_index_access.sifr` | List/dict/tuple indexing | FAIL (Issue 5) |
| `25_infer_from_walrus.sifr` | Walrus operator | PASS |
| `26_infer_lambda_param_from_context.sifr` | Lambda param from collection type | PASS |
| `27_infer_multiline_no_annotations.sifr` | Multi-step computation | FAIL (Issue 6) |
| `28_infer_from_zip_enumerate.sifr` | zip/enumerate result types | PASS |
| `29_infer_from_any_all.sifr` | bool from any/all | PASS |
| `30_infer_from_string_ops.sifr` | String method return types | FAIL (move semantics) |
