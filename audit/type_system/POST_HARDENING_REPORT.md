# Post-Hardening Audit Report: Type System

**Date:** February 15, 2026  
**Scope:** 43 test files in `audit/type_system/`  
**Context:** Post Language Hardening Phase (Milestones 1–6 completed)

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 20 | 46.5% |
| **Fail (Sifr compile)** | 17 | 39.5% |
| **Fail (Rust compile)** | 6 | 14.0% |
| **Total** | 43 | 100% |

---

## Passing Tests (20)

These tests compile and run correctly after the Language Hardening phase:

| # | Test | Notes |
|---|------|-------|
| 01 | `01_basic_unions.sifr` | Union types, optionals, 3-way union |
| 02 | `02_literal_types_v2.sifr` | Literal types (v2 variant) |
| 04 | `04_classes_and_inheritance.sifr` | Classes, inheritance, operators |
| 08 | `08_newtypes.sifr` | Newtype pattern |
| 09 | `09_generics_basic.sifr` | Generic workaround (concrete types) |
| 10 | `10_generic_classes.sifr` | Generic class workaround |
| 11 | `11_generic_constraints.sifr` | Constraint workaround |
| 12 | `12_tuple_types.sifr` | Tuple types, unpacking, nesting |
| 13 | `13_collections_typed.sifr` | Typed list, dict, nested |
| 15 | `15_type_composition.sifr` | Class unions, composition |
| 18 | `18_recursive_types.sifr` | Flat recursive class |
| 26 | `26_multiple_use_after_print.sifr` | Use after print |
| 28 | `28_return_type_coercion.sifr` | int→float coercion |
| 29 | `29_string_methods_after_narrow.sifr` | `.upper()` after isinstance |
| 30 | `30_float_to_int_cast.sifr` | `int(float_val)` |
| 32 | `32_field_named_message.sifr` | Single class field access |
| 33 | `33_field_named_common.sifr` | Various field names |
| 35 | `35_class_reuse_after_method.sifr` | Multiple method calls |
| 37 | `37_3way_isinstance_no_field.sifr` | 3-way isinstance (no fields) |
| 40 | `40_print_then_field.sifr` | Print then field access |

---

## Failure Categories

### 1. Narrowing Over-Narrows to `Never`

**Error:** `cannot compare 'Never' and 'str' with ==`

After the first `if x == "value":` branch, the type system narrows the variable to `Never` in the `elif` branch, making further equality comparisons impossible.

| File | Description |
|------|-------------|
| `02_literal_types.sifr` | Literal types with `elif` equality chain |
| `03_type_narrowing.sifr` | isinstance, None, equality, truthiness |
| `24_elif_equality_chain.sifr` | Multi-branch `elif x == "val"` dispatch |

---

### 2. Attribute/Field Access on Narrowed Types Not Supported

**Error:** `attribute access '.field' not supported as expression`

Field access on union variants after narrowing in `elif isinstance` chains is not recognized as a valid expression. The codegen or type checker does not support attribute access in these contexts.

| File | Field | Description |
|------|-------|-------------|
| `06_discriminated_unions.sifr` | `.base` | 3+ variant discriminated unions |
| `16_exhaustive_matching.sifr` | `.intensity` | Exhaustive match on 3-way union |
| `20_complex_patterns.sifr` | `.message` | State machine, API patterns |
| `25_union_field_access_after_narrow.sifr` | `.name` | Field access in elif branch |
| `31_3way_isinstance_elif.sifr` | `.z` | 3-way isinstance field access |

---

### 3. Generic Type Parameters Not Supported

**Error:** `unknown type: 'T'`

Generic syntax (`def f[T](x: T)`, `class Box[T]`) is not recognized. The type system has no `TypeVar` or `Generic` variant.

| File | Description |
|------|-------------|
| `21_generic_functions_syntax.sifr` | `def f[T](x: T) -> T` |
| `22_generic_class_syntax.sifr` | `class Box[T]` |

---

### 4. Protocol Method Dispatch Not Working

**Error:** `type 'ProtocolName' has no method 'methodName'`

Protocols can be defined and classes can implement them, but using a protocol as a function parameter type and calling methods through it fails. The type checker does not recognize protocol methods.

| File | Protocol | Method | Description |
|------|----------|--------|-------------|
| `23_interface_as_param.sifr` | HasArea | `area` | Protocol as param type |
| `34_protocol_param_dispatch.sifr` | Describable | `describe` | Protocol param dispatch |

---

### 5. Ownership / Move Issues

**Error:** `use of moved value: 'x'`

Values are consumed by operations (e.g., iteration, function calls) and cannot be reused. Some cases were fixed by Milestone 3 (ownership v2), but others remain.

| File | Description |
|------|-------------|
| `06_discriminated_unions.sifr` | Use of moved value in union handling |
| `07_result_option.sifr` | Use of moved value: `items` |
| `41_pass_class_to_fn.sifr` | Use of moved value: `item` |

---

### 6. Return Type / Type Coercion Mismatches

**Error:** `return type mismatch: expected 'X', got 'Y'` or `type mismatch: cannot assign 'A' to 'B'`

Type checker or codegen does not correctly handle numeric coercion, optional assignment, or Result/Option return types.

| File | Error | Description |
|------|-------|-------------|
| `07_result_option.sifr` | Expected `Result[int, str]`, got `float` | Result type, try/except |
| `17_mapped_conditional_types.sifr` | Cannot assign `None \| str` to `str` | Partial-like pattern |
| `38_int_division_returns.sifr` | Expected `int`, got `float` | `int / int` returns float |

---

### 7. Callable Type Syntax Not Supported

**Error:** `parse error: Expected ':', found '(' (Callable type syntax not supported)`

The parser does not recognize `Callable[[int], int]` or similar function parameter type syntax.

| File | Description |
|------|-------------|
| `14_higher_order_functions.sifr` | Callback type syntax for higher-order functions |

---

### 8. Codegen Type Mismatches (Rust Compilation)

**Errors:** `() doesn't implement Display`, `cannot multiply f64 by i64`, `variable does not need to be mutable`, `mismatched types`

Generated Rust code has type errors, incorrect operator usage, or unnecessary mutability.

| File | Rust Error | Description |
|------|------------|-------------|
| `03_type_narrowing_v2.sifr` | `()` doesn't implement `std::fmt::Display` | 3-way union `is None` codegen |
| `05_protocols.sifr` | Cannot multiply `f64` by `i64` | Protocol method arithmetic |
| `19_type_guards_custom.sifr` | `()` doesn't implement `std::fmt::Display` | Sequential narrowing |
| `27_narrowing_reassign.sifr` | Variable does not need to be mutable | Narrowed variable reassignment |
| `36_optional_field_narrowing.sifr` | Mismatched types | Optional field in class |
| `39_class_with_optional_init.sifr` | Mismatched types | Optional in constructor |

---

## What Was Fixed by Language Hardening

The following improvements from Milestones 1–6 brought **20 tests** to passing (up from 17 pre-hardening):

| Milestone | Fixes | Tests Unblocked |
|-----------|-------|-----------------|
| **M1: Codegen fixes** | Tuple indexing, union return wrapping, int/int division, print(None), escaped quotes, narrowed reassignment, float*int cast, **= power, bool() collections, 3-way union is None | 28, 30, 37 |
| **M2: Narrowing v2** | Early-return narrowing, and-based narrowing, elif isinstance codegen, elif equality, sequential narrowing, len() on nested optionals | 37 |
| **M3: Ownership v2** | print consumes, string method moves, list mutation after use, tuple len after print, dunder operators consume, chained operations | 04, 08, 26, 40, 41 |
| **M4–M6** | Subscript mutation, iteration, builtins | (indirect improvements) |

**Notable gains:**
- **04, 08, 26, 40** — Ownership fixes allow reuse after print and method calls. (41 still fails with move on pass-to-function.)
- **28** — Return type coercion for int→float.
- **37** — 3-way `isinstance` codegen now emits all `elif` arms correctly.

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Blocks Real-World Usage

1. **Generic type parameters** — No `T`, `U`, etc. Support for `def f[T](x: T)` and `class Box[T]` is required for reusable abstractions.
2. **Protocol method dispatch** — Protocols as parameter types cannot call methods. Structural typing is effectively unusable.
3. **Narrowing over-narrows to `Never`** — `elif x == "val"` chains fail. Multi-branch string dispatch is a common pattern.
4. **Attribute access on narrowed types** — Field access in `elif isinstance` branches fails for 3+ unions. Discriminated unions are partially broken.

### Tier 2 — Significant Ergonomics

5. **Ownership / move issues** — Remaining cases in 06, 07, 41 block Result handling and union processing.
6. **Return type / coercion mismatches** — `int / int` returns float; Result/Option return types; optional assignment.
7. **Optional auto-wrapping** — `T | None` parameters do not accept plain `T` (36, 39).
8. **Codegen type mismatches** — Display for `()`, float×int arithmetic, mutability, optional field/init (03_v2, 05, 19, 27, 36, 39).

### Tier 3 — Nice to Have

9. **Callable type syntax** — `Callable[[int], int]` for higher-order function parameters (14).

---

## Test File Index (Post-Hardening)

| File | Status | Root Cause |
|------|--------|------------|
| `01_basic_unions.sifr` | PASS | — |
| `02_literal_types.sifr` | FAIL (Sifr) | Narrowing to Never |
| `02_literal_types_v2.sifr` | PASS | — |
| `03_type_narrowing.sifr` | FAIL (Sifr) | Narrowing to Never |
| `03_type_narrowing_v2.sifr` | FAIL (Rust) | Display for `()` |
| `04_classes_and_inheritance.sifr` | PASS | — |
| `05_protocols.sifr` | FAIL (Rust) | f64 × i64 |
| `06_discriminated_unions.sifr` | FAIL (Sifr) | Field access, move |
| `07_result_option.sifr` | FAIL (Sifr) | Return type, move |
| `08_newtypes.sifr` | PASS | — |
| `09_generics_basic.sifr` | PASS | — |
| `10_generic_classes.sifr` | PASS | — |
| `11_generic_constraints.sifr` | PASS | — |
| `12_tuple_types.sifr` | PASS | — |
| `13_collections_typed.sifr` | PASS | — |
| `14_higher_order_functions.sifr` | FAIL (Sifr) | Callable syntax |
| `15_type_composition.sifr` | PASS | — |
| `16_exhaustive_matching.sifr` | FAIL (Sifr) | Field access |
| `17_mapped_conditional_types.sifr` | FAIL (Sifr) | Type coercion |
| `18_recursive_types.sifr` | PASS | — |
| `19_type_guards_custom.sifr` | FAIL (Rust) | Display for `()` |
| `20_complex_patterns.sifr` | FAIL (Sifr) | Field access |
| `21_generic_functions_syntax.sifr` | FAIL (Sifr) | Generics |
| `22_generic_class_syntax.sifr` | FAIL (Sifr) | Generics |
| `23_interface_as_param.sifr` | FAIL (Sifr) | Protocol dispatch |
| `24_elif_equality_chain.sifr` | FAIL (Sifr) | Narrowing to Never |
| `25_union_field_access_after_narrow.sifr` | FAIL (Sifr) | Field access |
| `26_multiple_use_after_print.sifr` | PASS | — |
| `27_narrowing_reassign.sifr` | FAIL (Rust) | Mutability |
| `28_return_type_coercion.sifr` | PASS | — |
| `29_string_methods_after_narrow.sifr` | PASS | — |
| `30_float_to_int_cast.sifr` | PASS | — |
| `31_3way_isinstance_elif.sifr` | FAIL (Sifr) | Field access |
| `32_field_named_message.sifr` | PASS | — |
| `33_field_named_common.sifr` | PASS | — |
| `34_protocol_param_dispatch.sifr` | FAIL (Sifr) | Protocol dispatch |
| `35_class_reuse_after_method.sifr` | PASS | — |
| `36_optional_field_narrowing.sifr` | FAIL (Rust) | Mismatched types |
| `37_3way_isinstance_no_field.sifr` | PASS | — |
| `38_int_division_returns.sifr` | FAIL (Sifr) | Return type |
| `39_class_with_optional_init.sifr` | FAIL (Rust) | Mismatched types |
| `40_print_then_field.sifr` | PASS | — |
| `41_pass_class_to_fn.sifr` | FAIL (Sifr) | Move |

