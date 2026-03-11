# Post-Hardening Audit Report: Type System

**Date:** February 16, 2026  
**Scope:** 43 test files in `audit/type_system/`  
**Context:** Post borrow-by-default phase (borrow-hardening + subsequent compiler changes)

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 23 | 53.5% |
| **Fail (Sifr compile)** | 4 | 9.3% |
| **Fail (Rust compile)** | 10 | 23.3% |
| **Fail (runtime)** | 6 | 14.0% |
| **Total** | 43 | 100% |

---

## Changes Since Last Report

The previous report (February 15, 2026) had: **20 PASS**, 17 Sifr compile, 6 Rust compile, 0 runtime.

Net result: **+3 passing** (20 → 23). Sifr compile failures dropped sharply (17 → 4) as generics, protocol dispatch, narrowing, and field access were fixed. However, new Rust codegen regressions and runtime failures appeared from the borrow-by-default transition.

### Improvements (15 tests progressed)

| Test | Previous | Current | What Changed |
|------|----------|---------|--------------|
| `03_type_narrowing.sifr` | FAIL (Sifr) — narrowing to Never | **PASS** | Narrowing fixed |
| `03_type_narrowing_v2.sifr` | FAIL (Rust) — Display for `()` | **PASS** | Codegen fixed |
| `06_discriminated_unions.sifr` | FAIL (Sifr) — field access + move | **PASS** | Field access and ownership fixed |
| `19_type_guards_custom.sifr` | FAIL (Rust) — Display for `()` | **PASS** | Codegen fixed |
| `21_generic_functions_syntax.sifr` | FAIL (Sifr) — unknown type T | **PASS** | Generic type parameters now supported |
| `22_generic_class_syntax.sifr` | FAIL (Sifr) — unknown type T | **PASS** | Generic type parameters now supported |
| `23_interface_as_param.sifr` | FAIL (Sifr) — protocol dispatch | **PASS** | Protocol method dispatch fixed |
| `25_union_field_access_after_narrow.sifr` | FAIL (Sifr) — field access | **PASS** | Narrowed field access fixed |
| `31_3way_isinstance_elif.sifr` | FAIL (Sifr) — field access | **PASS** | 3-way isinstance field access fixed |
| `02_literal_types.sifr` | FAIL (Sifr) — narrowing to Never | FAIL (Rust) | Progressed past Sifr compile; now hits &String comparison |
| `16_exhaustive_matching.sifr` | FAIL (Sifr) — field access | FAIL (Runtime) | Progressed past compile; binary not found |
| `20_complex_patterns.sifr` | FAIL (Sifr) — field access | FAIL (Runtime) | Progressed past compile; binary not found |
| `24_elif_equality_chain.sifr` | FAIL (Sifr) — narrowing to Never | FAIL (Rust) | Progressed past Sifr compile; Rust type errors |
| `34_protocol_param_dispatch.sifr` | FAIL (Sifr) — protocol dispatch | FAIL (Rust) | Progressed past Sifr compile; Rust mismatched types |
| `41_pass_class_to_fn.sifr` | FAIL (Sifr) — move | FAIL (Rust) | Progressed past Sifr compile; Rust &String comparison |

### Regressions (6 tests regressed)

| Test | Previous | Current | Root Cause |
|------|----------|---------|------------|
| `01_basic_unions.sifr` | PASS | FAIL (Rust) | &String vs String comparison (borrow-by-default codegen) |
| `02_literal_types_v2.sifr` | PASS | FAIL (Rust) | &String vs String comparison (borrow-by-default codegen) |
| `09_generics_basic.sifr` | PASS | FAIL (Runtime) | Could not run binary: No such file or directory |
| `26_multiple_use_after_print.sifr` | PASS | FAIL (Runtime) | Could not run binary: No such file or directory |
| `29_string_methods_after_narrow.sifr` | PASS | FAIL (Runtime) | Could not run binary: No such file or directory |
| `32_field_named_message.sifr` | PASS | FAIL (Runtime) | Could not run binary: No such file or directory |

---

## Passing Tests (23)

These tests compile and run correctly:

| # | Test | Notes |
|---|------|-------|
| 03 | `03_type_narrowing.sifr` | isinstance, None, equality, truthiness |
| 03v2 | `03_type_narrowing_v2.sifr` | 3-way union `is None` codegen |
| 04 | `04_classes_and_inheritance.sifr` | Classes, inheritance, operators |
| 06 | `06_discriminated_unions.sifr` | 3+ variant discriminated unions |
| 08 | `08_newtypes.sifr` | Newtype pattern |
| 10 | `10_generic_classes.sifr` | Generic class workaround |
| 11 | `11_generic_constraints.sifr` | Constraint workaround |
| 12 | `12_tuple_types.sifr` | Tuple types, unpacking, nesting |
| 13 | `13_collections_typed.sifr` | Typed list, dict, nested |
| 15 | `15_type_composition.sifr` | Class unions, composition |
| 18 | `18_recursive_types.sifr` | Flat recursive class |
| 19 | `19_type_guards_custom.sifr` | Sequential narrowing |
| 21 | `21_generic_functions_syntax.sifr` | `def f[T](x: T) -> T` |
| 22 | `22_generic_class_syntax.sifr` | `class Box[T]` |
| 23 | `23_interface_as_param.sifr` | Protocol as param type |
| 25 | `25_union_field_access_after_narrow.sifr` | Field access in elif branch |
| 28 | `28_return_type_coercion.sifr` | int→float coercion |
| 30 | `30_float_to_int_cast.sifr` | `int(float_val)` |
| 31 | `31_3way_isinstance_elif.sifr` | 3-way isinstance field access |
| 33 | `33_field_named_common.sifr` | Various field names |
| 35 | `35_class_reuse_after_method.sifr` | Multiple method calls |
| 37 | `37_3way_isinstance_no_field.sifr` | 3-way isinstance (no fields) |
| 40 | `40_print_then_field.sifr` | Print then field access |

---

## Failure Categories

### 1. Sifr Compilation Failures (4)

Only 4 tests still fail at the Sifr compile stage (down from 17):

#### 1a. Return Type / Type Coercion Mismatches

| File | Error | Description |
|------|-------|-------------|
| `07_result_option.sifr` | Expected `Result[int, str]`, got `float` | Result type, try/except |
| `38_int_division_returns.sifr` | Expected `int`, got `float` | `int / int` returns float |

#### 1b. Callable Type Syntax Not Supported

| File | Error | Description |
|------|-------|-------------|
| `14_higher_order_functions.sifr` | Parse error: Expected `:`, found `(` | Callable type syntax not recognized |

#### 1c. Optional Assignment Mismatch

| File | Error | Description |
|------|-------|-------------|
| `17_mapped_conditional_types.sifr` | Cannot assign `None \| str` to `str` | Partial-like pattern |

---

### 2. Rust Compilation Failures (10)

The borrow-by-default transition introduced significant Rust codegen issues. The 10 failures group into the following patterns:

#### 2a. &String vs String Comparison (borrow-by-default codegen regression)

The compiler generates `&String` references but Rust comparison operators expect matching types. This is the most common regression pattern.

| File | Error Details |
|------|---------------|
| `01_basic_unions.sifr` | E0277 can't compare `&String` with `String`, E0308 (×3) |
| `02_literal_types.sifr` | E0277 can't compare `&String` with `String` (×3), E0308 |
| `02_literal_types_v2.sifr` | E0277 can't compare `&String` with `String`, E0308 |
| `41_pass_class_to_fn.sifr` | E0277 can't compare `&String` with `String` (×4) |

#### 2b. Mismatched Types (E0308)

Generated Rust code has type mismatches in assignments, returns, or match arms:

| File | Error Details |
|------|---------------|
| `27_narrowing_reassign.sifr` | E0308 mismatched types (×4) |
| `34_protocol_param_dispatch.sifr` | E0308 mismatched types (×2) |
| `36_optional_field_narrowing.sifr` | E0308, E0507 cannot move out of shared reference |
| `39_class_with_optional_init.sifr` | E0308 (×2), E0596 cannot borrow as mutable |

#### 2c. Arithmetic Type Mismatch

| File | Error Details |
|------|---------------|
| `05_protocols.sifr` | E0277 cannot multiply `f64` by `i64` (×2) |

#### 2d. Trait Bound Failures

| File | Error Details |
|------|---------------|
| `24_elif_equality_chain.sifr` | E0277 `dyn Any`: `Eq`, `Hash` not satisfied; E0308 |

---

### 3. Runtime Failures (6)

All 6 runtime failures share the same root cause: the compiled binary is not found at execution time. This suggests a build pipeline issue where Rust compilation succeeds but the output binary is not placed in the expected location.

| File | Error |
|------|-------|
| `09_generics_basic.sifr` | Could not run binary: No such file or directory |
| `16_exhaustive_matching.sifr` | Could not run binary: No such file or directory |
| `20_complex_patterns.sifr` | Could not run binary: No such file or directory |
| `26_multiple_use_after_print.sifr` | Could not run binary: No such file or directory |
| `29_string_methods_after_narrow.sifr` | Could not run binary: No such file or directory |
| `32_field_named_message.sifr` | Could not run binary: No such file or directory |

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Borrow-by-Default Codegen Regressions

1. **&String vs String comparison** — The most widespread regression. Borrow-by-default generates `&String` references but equality comparisons fail because Rust cannot compare `&String` with `String` directly. Affects 4 Rust-compile failures and caused 2 regressions from PASS. Fix: dereference or use `.as_str()` in generated comparison code.
2. **Binary not found at runtime** — 6 tests compile but the binary is missing. Likely a build output path issue introduced during the borrow-by-default transition. Caused 4 regressions from PASS.

### Tier 2 — Pre-Existing Rust Codegen Issues

3. **Mismatched types (E0308)** — Multiple tests generate Rust code with type mismatches in assignments, returns, or match arms (27, 34, 36, 39).
4. **Arithmetic type mismatch** — `f64 × i64` in protocol method codegen (05).
5. **Trait bounds not satisfied** — `dyn Any` missing `Eq`/`Hash` (24).
6. **Move/borrow conflicts** — Cannot move out of shared reference (36), cannot borrow as mutable (39).

### Tier 3 — Sifr Compile Issues (Stable)

7. **Return type mismatches** — `int / int` returns float (07, 38). Needs int-division or coercion support.
8. **Optional assignment** — `None | str` cannot assign to `str` (17).
9. **Callable type syntax** — `Callable[[int], int]` not parsed (14).

---

## Test File Index

| File | Status | Root Cause |
|------|--------|------------|
| `01_basic_unions.sifr` | FAIL (Rust) | &String vs String comparison |
| `02_literal_types.sifr` | FAIL (Rust) | &String vs String comparison |
| `02_literal_types_v2.sifr` | FAIL (Rust) | &String vs String comparison |
| `03_type_narrowing.sifr` | PASS | — |
| `03_type_narrowing_v2.sifr` | PASS | — |
| `04_classes_and_inheritance.sifr` | PASS | — |
| `05_protocols.sifr` | FAIL (Rust) | f64 × i64 |
| `06_discriminated_unions.sifr` | PASS | — |
| `07_result_option.sifr` | FAIL (Sifr) | Return type mismatch |
| `08_newtypes.sifr` | PASS | — |
| `09_generics_basic.sifr` | FAIL (Runtime) | Binary not found |
| `10_generic_classes.sifr` | PASS | — |
| `11_generic_constraints.sifr` | PASS | — |
| `12_tuple_types.sifr` | PASS | — |
| `13_collections_typed.sifr` | PASS | — |
| `14_higher_order_functions.sifr` | FAIL (Sifr) | Callable syntax |
| `15_type_composition.sifr` | PASS | — |
| `16_exhaustive_matching.sifr` | FAIL (Runtime) | Binary not found |
| `17_mapped_conditional_types.sifr` | FAIL (Sifr) | Optional assignment |
| `18_recursive_types.sifr` | PASS | — |
| `19_type_guards_custom.sifr` | PASS | — |
| `20_complex_patterns.sifr` | FAIL (Runtime) | Binary not found |
| `21_generic_functions_syntax.sifr` | PASS | — |
| `22_generic_class_syntax.sifr` | PASS | — |
| `23_interface_as_param.sifr` | PASS | — |
| `24_elif_equality_chain.sifr` | FAIL (Rust) | dyn Any: Eq/Hash not satisfied |
| `25_union_field_access_after_narrow.sifr` | PASS | — |
| `26_multiple_use_after_print.sifr` | FAIL (Runtime) | Binary not found |
| `27_narrowing_reassign.sifr` | FAIL (Rust) | Mismatched types |
| `28_return_type_coercion.sifr` | PASS | — |
| `29_string_methods_after_narrow.sifr` | FAIL (Runtime) | Binary not found |
| `30_float_to_int_cast.sifr` | PASS | — |
| `31_3way_isinstance_elif.sifr` | PASS | — |
| `32_field_named_message.sifr` | FAIL (Runtime) | Binary not found |
| `33_field_named_common.sifr` | PASS | — |
| `34_protocol_param_dispatch.sifr` | FAIL (Rust) | Mismatched types |
| `35_class_reuse_after_method.sifr` | PASS | — |
| `36_optional_field_narrowing.sifr` | FAIL (Rust) | Mismatched types, move out of ref |
| `37_3way_isinstance_no_field.sifr` | PASS | — |
| `38_int_division_returns.sifr` | FAIL (Sifr) | Return type mismatch |
| `39_class_with_optional_init.sifr` | FAIL (Rust) | Mismatched types, mutability |
| `40_print_then_field.sifr` | PASS | — |
| `41_pass_class_to_fn.sifr` | FAIL (Rust) | &String vs String comparison |
