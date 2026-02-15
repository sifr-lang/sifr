# Sifr Type System Audit Report

**Date:** February 15, 2026
**Methodology:** 41 test files compiled and run against the Sifr compiler to probe type system features against TypeScript's capabilities.
**Compiler version:** Current `main` branch (ahead by 1 commit)

---

## Executive Summary

**17 PASS / 24 FAIL** out of 41 tests.

The type system has a solid foundation -- union types, basic narrowing, optional types, classes, inheritance, tuples, typed collections, and type aliases all work. However, there are **10 distinct categories of issues** that significantly limit the type system's expressiveness compared to TypeScript. The most critical gaps are: **no generics**, **broken `elif` chains**, **overly aggressive ownership/move semantics**, and **incomplete protocol dispatch**.

---

## What Works Well (PASS)

| # | Test | TypeScript Equivalent | Status |
|---|------|----------------------|--------|
| 01 | Basic unions (`int \| str`) | `number \| string` | PASS |
| 09 | Generics workaround (concrete types) | N/A (workaround) | PASS |
| 10 | Generic classes workaround (concrete) | N/A (workaround) | PASS |
| 11 | Generic constraints workaround | N/A (workaround) | PASS |
| 12 | Tuple types | `[string, number]` | PASS |
| 13 | Typed collections (list, dict, nested) | `Array<T>`, `Map<K,V>` | PASS |
| 15 | Type composition (class unions) | Discriminated unions | PASS |
| 18 | Recursive types (flat) | Recursive interfaces | PASS |
| 29 | String methods after narrowing | Type guards | PASS |
| 30 | Float to int cast | `Math.floor()` | PASS |
| 32 | Field named "message" (single class) | Property access | PASS |
| 33 | Various field names (single class) | Property access | PASS |
| 35 | Class reuse after method call | Object reuse | PASS |

---

## Issues Found

### Issue 1: NO GENERICS (Critical)

**Tests:** 21, 22
**TypeScript equivalent:** `function identity<T>(x: T): T`, `class Box<T>`
**Error:** `type error: unknown type: 'T'`

The `Type` enum has no `Generic`, `TypeVar`, or `TypeParam` variant. Generic syntax (`def f[T](x: T) -> T`) is not recognized. This is the **single biggest gap** vs TypeScript.

**Impact:** Cannot write reusable data structures or algorithms. Every generic pattern requires duplicating code for each concrete type (e.g., `IntBox`, `StrBox`, `FloatBox`).

**TypeScript features blocked:**
- Generic functions: `function map<T, U>(arr: T[], fn: (x: T) => U): U[]`
- Generic classes: `class Box<T> { value: T }`
- Generic constraints: `function longest<T extends { length: number }>(a: T, b: T): T`
- Generic interfaces: `interface Repository<T> { find(id: string): T }`
- Default type parameters: `class Container<T = string>`
- Conditional types: `type IsString<T> = T extends string ? true : false`
- Mapped types: `type Partial<T> = { [K in keyof T]?: T[K] }`
- `keyof`, `typeof`, `infer`, template literal types

---

### Issue 2: `elif` Equality Narrowing Broken (Critical)

**Tests:** 02, 03, 24
**Error:** `type error: cannot compare 'Never' and 'str' with ==`

After the first `if x == "value":` branch, the type system narrows the variable to `Never` in the `elif` branch, making further equality comparisons impossible. Simple `if/else` equality works, but any `elif` chain fails.

```python
# FAILS:
def route(method: str) -> str:
    if method == "GET":
        return "get"
    elif method == "POST":    # Error: cannot compare 'Never' and 'str'
        return "post"
```

**Impact:** Cannot write multi-branch dispatch on string values -- an extremely common pattern (routing, command handling, state machines).

---

### Issue 3: `elif isinstance` Codegen Bug in 3+ Unions (Critical)

**Tests:** 06, 16, 20, 25, 31, 37
**Symptoms:**
- Field access in `else` branch of 3+ union: `attribute access '.field' is not supported as an expression`
- Runtime: `elif isinstance` branches are **silently dropped** -- codegen only emits the first `if` and the `else`, skipping all `elif` arms

**Evidence from emitted Rust (test 37):**
```rust
// Input: if isinstance(val, A) ... elif isinstance(val, B) ... else ...
// Generated:
match val {
    AOrBOrC::A(val) => { return "A"; }
    _ => { return "C"; }  // B branch completely missing!
}
```

`check(B())` returns `"C"` instead of `"B"`.

**Impact:** Discriminated unions with 3+ variants are fundamentally broken. This is one of the most important TypeScript patterns (tagged unions / ADTs).

---

### Issue 4: Overly Aggressive Move Semantics (High)

**Tests:** 04, 08, 26, 40, 41
**Error:** `type error: use of moved value: 'x'`

Classes are marked as `Move` ownership, meaning:
- `print(obj)` moves the object, making it unusable afterward
- Passing a class to a function moves it
- Even accessing a field after `print()` fails

```python
# FAILS:
p: Point = Point(3.0, 4.0)
print(p)        # moves p
print(p.x)      # Error: use of moved value 'p'
```

Method calls on `self` work (test 35 passes), but any external use that takes the value by-value triggers a move. TypeScript has no ownership concept -- values are freely reusable.

**Impact:** Makes it very difficult to write normal application code. Users must carefully avoid using objects after printing or passing them to functions.

---

### Issue 5: Protocol Dynamic Dispatch Not Working (High)

**Tests:** 23, 34
**Error:** `type error: type 'HasArea' has no method 'area'`

Protocols can be **defined** and classes can **implement** them, but using a protocol as a **function parameter type** and calling methods through it fails. The type checker doesn't recognize that protocol types have the declared methods.

```python
# FAILS:
class HasArea(Protocol):
    def area(self) -> float:
        pass

def print_area(shape: HasArea) -> None:
    print(shape.area())  # Error: type 'HasArea' has no method 'area'
```

**Impact:** Protocols are essentially decorative -- they can't be used for polymorphism. This defeats the purpose of structural typing (TypeScript's core interface feature).

---

### Issue 6: Optional (`T | None`) Not Auto-Wrapped at Call Sites (High)

**Tests:** 36, 39
**Rust error:** `expected Option<String>, found String` / `expected Option<i64>, found i64`

When a function/constructor parameter is typed as `T | None`, passing a plain `T` value doesn't auto-wrap it in `Some(...)` in the generated Rust.

```python
# FAILS at Rust build:
class User:
    age: int | None
    def __init__(self, age: int | None):
        self.age = age

u: User = User("Alice", 30)  # Rust: expected Option<i64>, got i64
```

**Impact:** Optional fields in classes are unusable with non-None values unless the user somehow wraps them manually (which Sifr has no syntax for).

---

### Issue 7: `int / int` Returns `float`, No Integer Division (Medium)

**Tests:** 07, 28, 38
**Error:** `return type mismatch: expected 'int', got 'float'` / Rust: `expected f64, found i64`

Integer division (`a / b` where both are `int`) returns `float` in the type system, matching Python's behavior. But there's no `//` floor division operator, and no automatic coercion from `int` to `float` in return position.

```python
# FAILS:
def half(x: int) -> int:
    return x / 2  # Error: expected 'int', got 'float'

def divide(a: int, b: int) -> float:
    return a / b  # Rust error: expected f64, found i64
```

**Impact:** Basic arithmetic patterns are awkward. Need `int()` cast or explicit conversion.

---

### Issue 8: Narrowing Doesn't Propagate Through `elif isinstance` for Field Access (Medium)

**Tests:** 06, 16, 20, 25, 31
**Error:** `attribute access '.field' is not supported as an expression; use as a method call`

When narrowing through `elif isinstance` in a 3+ member union, the `else` branch doesn't properly narrow to the remaining type, so field access fails.

This is related to Issue 3 but manifests differently -- even when the type checker doesn't crash, it doesn't know the narrowed type in the `else` branch.

---

### Issue 9: Narrowing Doesn't Work for `int | str | None` with `elif isinstance` After `is None` Check (Medium)

**Test:** 19
**Error:** `type error: type 'None | str' has no method 'upper'`

After checking `if x is None:` and then `elif isinstance(x, int):`, the `else` branch should narrow to `str`, but the type system still sees `None | str`.

```python
# FAILS:
def process(x: int | str | None) -> str:
    if x is None:
        return "nothing"
    if isinstance(x, int):
        return f"number: {x + 1}"
    else:
        return f"text: {x.upper()}"  # Error: 'None | str' has no method 'upper'
```

---

### Issue 10: Reassignment of Narrowed Variable Fails in Codegen (Medium)

**Test:** 27
**Rust error:** `cannot assign twice to immutable variable`

When a narrowed optional variable is reassigned inside the narrowed branch, the generated Rust doesn't make the binding mutable.

```python
# Codegen bug:
def update(name: str | None) -> str:
    if name is not None:
        name = name.upper()  # Rust: cannot assign twice to immutable variable
        return name
```

---

### Issue 11: `float * int` Type Mismatch in Codegen (Medium)

**Test:** 05
**Rust error:** `cannot multiply f64 by i64`

When a protocol method returns `int` but the computation involves `float * float`, the codegen casts the result to `i64` incorrectly, or when `int()` is called on a float expression, the generated Rust has type mismatches.

```python
# Codegen bug:
def size(self) -> int:
    return int(3.14 * self.radius * self.radius)
    # Rust: (3.14_f64 * self.radius) * self.radius as i64
    # Should be: ((3.14_f64 * self.radius) * self.radius) as i64
```

---

### Issue 12: Higher-Order Function Type Syntax (Low)

**Test:** 14
**Error:** `parse error: Expected ':', found '(' at byte range 273..274`

The `lambda(int) -> int` syntax for function parameter types is not recognized by the parser. The existing `map` and `filter` builtins work with inline lambdas, but declaring a function that takes a callable as a parameter doesn't parse.

---

### Issue 13: 3-Way Union `is None` Check on Non-Optional Union (Low)

**Test:** 03 (v2)
**Rust error:** `no method named 'is_none' found for enum NoneOrIntOrStr`

When a 3+ member union includes `None` (e.g., `int | str | None`), the codegen generates a full enum (`NoneOrIntOrStr`) rather than `Option<IntOrStr>`, but then tries to call `.is_none()` on it, which doesn't exist.

---

## Missing TypeScript Features (Not Yet Implemented)

These are TypeScript features that have **no equivalent** in Sifr, not even a broken one:

| Feature | TypeScript Example | Status |
|---------|-------------------|--------|
| Generics | `<T>`, `<T extends U>` | Not implemented |
| Conditional types | `T extends U ? X : Y` | Not implemented |
| Mapped types | `{ [K in keyof T]: V }` | Not implemented |
| `keyof` operator | `keyof User` | Not implemented |
| `typeof` type operator | `typeof myVar` | Not implemented |
| Template literal types | `` `hello-${string}` `` | Not implemented |
| `infer` keyword | `T extends Array<infer U> ? U : never` | Not implemented |
| Index access types | `User["name"]` | Not implemented |
| Utility types | `Partial<T>`, `Required<T>`, `Pick<T,K>`, `Omit<T,K>`, `Record<K,V>` | Not implemented |
| Type assertions | `x as string` | Not implemented |
| Enum types | `enum Color { Red, Green, Blue }` | Not implemented (use union of classes) |
| `satisfies` operator | `config satisfies Config` | Not implemented |
| Custom type predicates | `function isString(x): x is string` | Not implemented |
| Recursive type aliases | `type Tree = { children: Tree[] }` | Not tested (classes work, aliases unknown) |
| Variadic tuple types | `[...T, string]` | Not implemented |
| `readonly` modifier | `readonly x: number` | Not implemented |
| `abstract` classes | `abstract class Shape` | Not implemented |

---

## Priority Ranking for Fixes

### Tier 1 -- Must Fix (Blocks Real-World Usage)

1. **Generics** -- Without generics, the language cannot express reusable abstractions. This is the #1 blocker.
2. **`elif isinstance` codegen for 3+ unions** -- Discriminated unions are a core feature but broken for 3+ variants.
3. **`elif` equality narrowing** -- Multi-branch string dispatch is a fundamental pattern.
4. **Overly aggressive move semantics** -- `print(obj)` consuming the object makes normal code impossible. Need auto-borrow or Clone.
5. **Protocol dynamic dispatch** -- Protocols as function parameters must work for structural typing to be useful.

### Tier 2 -- Should Fix (Significant Ergonomics)

6. **Optional auto-wrapping at call sites** -- `T | None` parameters must accept plain `T` values.
7. **`int / int` and numeric coercion** -- Need `//` operator or auto-coercion.
8. **Narrowing after `is None` + `isinstance` chain** -- Sequential narrowing must compose.
9. **Reassignment of narrowed variables** -- Codegen must emit `mut` bindings.
10. **`float * int` codegen** -- Operator precedence in `as` casts.

### Tier 3 -- Nice to Have (TypeScript Parity)

11. Higher-order function type syntax
12. 3-way union `is None` codegen
13. Conditional types, mapped types, utility types
14. `keyof`, `typeof`, `infer`
15. Enum types, abstract classes, readonly

---

## Test File Index

| File | Tests | Result |
|------|-------|--------|
| `01_basic_unions.sifr` | Union types, optionals, 3-way union | PASS |
| `02_literal_types.sifr` | Literal types, elif equality | FAIL (Issue 2) |
| `03_type_narrowing.sifr` | isinstance, None, equality, truthiness | FAIL (Issue 2) |
| `04_classes_and_inheritance.sifr` | Classes, inheritance, operators | FAIL (Issue 4) |
| `05_protocols.sifr` | Protocol definition, implementation | FAIL (Issue 11) |
| `06_discriminated_unions.sifr` | 3+ variant discriminated unions | FAIL (Issue 3, 8) |
| `07_result_option.sifr` | Result type, try/except | FAIL (Issue 7) |
| `08_newtypes.sifr` | Newtype pattern | FAIL (Issue 4) |
| `09_generics_basic.sifr` | Generic workaround (concrete) | PASS |
| `10_generic_classes.sifr` | Generic class workaround | PASS |
| `11_generic_constraints.sifr` | Constraint workaround | PASS |
| `12_tuple_types.sifr` | Tuple types, unpacking, nesting | PASS |
| `13_collections_typed.sifr` | Typed list, dict, nested | PASS |
| `14_higher_order_functions.sifr` | Callback type syntax | FAIL (Issue 12) |
| `15_type_composition.sifr` | Class unions, composition | PASS |
| `16_exhaustive_matching.sifr` | Exhaustive match on 3-way union | FAIL (Issue 3) |
| `17_mapped_conditional_types.sifr` | Partial-like pattern | FAIL (Issue 9) |
| `18_recursive_types.sifr` | Flat recursive class | PASS |
| `19_type_guards_custom.sifr` | Sequential narrowing | FAIL (Issue 9) |
| `20_complex_patterns.sifr` | State machine, API patterns | FAIL (Issue 3) |
| `21_generic_functions_syntax.sifr` | `def f[T](x: T)` | FAIL (Issue 1) |
| `22_generic_class_syntax.sifr` | `class Box[T]` | FAIL (Issue 1) |
| `23_interface_as_param.sifr` | Protocol as param type | FAIL (Issue 5) |
| `24_elif_equality_chain.sifr` | `elif x == "val"` | FAIL (Issue 2) |
| `25_union_field_access_after_narrow.sifr` | Field access in elif | FAIL (Issue 8) |
| `26_multiple_use_after_print.sifr` | Use after print | FAIL (Issue 4) |
| `27_narrowing_reassign.sifr` | Reassign narrowed var | FAIL (Issue 10) |
| `28_return_type_coercion.sifr` | int->float coercion | FAIL (Issue 7) |
| `29_string_methods_after_narrow.sifr` | `.upper()` after isinstance | PASS |
| `30_float_to_int_cast.sifr` | `int(float_val)` | PASS |
| `31_3way_isinstance_elif.sifr` | 3-way isinstance field access | FAIL (Issue 3) |
| `32_field_named_message.sifr` | Single class field access | PASS |
| `33_field_named_common.sifr` | Many field names | PASS |
| `34_protocol_param_dispatch.sifr` | Protocol param dispatch | FAIL (Issue 5) |
| `35_class_reuse_after_method.sifr` | Multiple method calls | PASS |
| `36_optional_field_narrowing.sifr` | Optional field in class | FAIL (Issue 6) |
| `37_3way_isinstance_no_field.sifr` | 3-way isinstance (no fields) | PASS* (wrong output) |
| `38_int_division_returns.sifr` | `int / int -> int` | FAIL (Issue 7) |
| `39_class_with_optional_init.sifr` | Optional in constructor | FAIL (Issue 6) |
| `40_print_then_field.sifr` | Print then field access | FAIL (Issue 4) |
| `41_pass_class_to_fn.sifr` | Pass to function then reuse | FAIL (Issue 4) |

\* Test 37 passes (exit code 0) but produces **wrong output**: `check(B())` returns `"C"` instead of `"B"` due to Issue 3.
