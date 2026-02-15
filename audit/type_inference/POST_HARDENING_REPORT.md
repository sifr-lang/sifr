# Post-Hardening Audit Report: Type Inference

**Date:** February 15, 2026  
**Scope:** 30 test files in `audit/type_inference/`  
**Context:** Post Language Hardening Phase (Milestones 1–6 completed; M9 inference_v2 pending)

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 24 | 80.0% |
| **Fail (Sifr compile)** | 1 | 3.3% |
| **Fail (Rust compile)** | 5 | 16.7% |
| **Total** | 30 | 100% |

---

## Passing Tests (24)

These tests compile and run correctly after the Language Hardening phase:

| # | Test | Notes |
|---|------|-------|
| 02 | `02_variable_from_expression.sifr` | Arithmetic, concat, comparison, logic |
| 03 | `03_variable_from_function_call.sifr` | Infer from return type annotation |
| 04 | `04_collection_literal_inference.sifr` | list, dict, tuple literals |
| 06 | `06_infer_from_method_call.sifr` | .upper(), len(), sorted() |
| 07 | `07_infer_from_conditional.sifr` | Ternary expressions |
| 08 | `08_infer_from_comprehension.sifr` | List comprehensions |
| 09 | `09_infer_from_map_filter.sifr` | map/filter with lambda |
| 10 | `10_infer_from_class_constructor.sifr` | Class instantiation |
| 11 | `11_infer_in_for_loop.sifr` | Loop variable from iterable |
| 12 | `12_infer_chained_operations.sifr` | Chained filter/map/len |
| 13 | `13_infer_mixed_no_annotation.sifr` | Full program, no var annotations |
| 14 | `14_infer_from_fstring.sifr` | F-string result type |
| 15 | `15_infer_from_optional_return.sifr` | `str \| None` from function |
| 19 | `19_infer_reassignment_same_type.sifr` | Reassign same type |
| 21 | `21_infer_from_builtin_functions.sifr` | len, abs, min, max, sum, str, bool |
| 22 | `22_infer_nested_collection.sifr` | Nested list/dict/tuple |
| 23 | `23_infer_class_field_access.sifr` | Field type from class |
| 24 | `24_infer_from_index_access.sifr` | List/dict/tuple indexing |
| 25 | `25_infer_from_walrus.sifr` | Walrus operator |
| 26 | `26_infer_lambda_param_from_context.sifr` | Lambda param from collection type |
| 27 | `27_infer_multiline_no_annotations.sifr` | Multi-step computation |
| 28 | `28_infer_from_zip_enumerate.sifr` | zip/enumerate result types |
| 29 | `29_infer_from_any_all.sifr` | bool from any/all |
| 30 | `30_infer_from_string_ops.sifr` | String method return types |

---

## Failure Categories

### 1. Sifr: Reassignment to Different Type (Intentional)

**Error:** `type mismatch: cannot assign 'str' to variable 'x' of type 'int'`

Sifr enforces static typing — reassigning a variable to a different type after inference is intentionally disallowed. This matches TypeScript strict mode behavior.

| File | Description |
|------|-------------|
| `20_infer_reassignment_different_type.sifr` | `x = 42` then `x = "hello"` — correctly rejected |

---

### 2. Codegen: `print(None)` / Unit Type Display

**Error:** `E0308 mismatched types` or `dyn Any` / `()` doesn't implement `Display`

`None` maps to Rust's `()` (unit type), which does not implement `Display`. The codegen should emit `println!("None")` or use `Debug` formatting for unit type.

| File | Description |
|------|-------------|
| `01_variable_from_literal.sifr` | `e = None; print(e)` — unit type passed to `println!` |
| `05_return_type_inference.sifr` | Return type defaults to `None`; inferred `dyn Any` or mixed types don't implement `Display` |

---

### 3. Codegen: Union Return Value Wrapping

**Error:** `E0308 mismatched types` — expected `IntOrStr`, found `i64`

When a function returns a union type (`int | str`), return values must be wrapped in the generated enum variant. Codegen currently emits raw values.

| File | Description |
|------|-------------|
| `16_infer_from_union_return.sifr` | `parse_input` returns `int \| str`; `return 42` should emit `IntOrStr::Int(42_i64)` |

---

### 4. Codegen: Result Type in try Block / Display

**Error:** `Result<i64, String> doesn't implement Display`

Variables assigned from `Result`-returning functions inside `try` blocks are inferred as `Result[T, E]` rather than the unwrapped success type `T`. Printing such a value fails because `Result` does not implement `Display`.

| File | Description |
|------|-------------|
| `17_infer_from_result_return.sifr` | `val = parse_int("42")` in try block; `val` inferred as `Result[int, str]`; `print(val)` fails |

---

### 5. Inference + Codegen: Empty Collection Type

**Error:** `dyn Any: Eq` not satisfied, `dyn Any: Hash` not satisfied, `E0308 mismatched types`

Empty list `[]` and empty dict `{}` have no element type to infer. The compiler may default to `dyn Any` or a similar existential type, which does not satisfy `Eq`/`Hash` for `HashMap` keys or causes type mismatches when appending.

| File | Description |
|------|-------------|
| `18_infer_empty_collection.sifr` | `empty_list = []`; `empty_list.append(42)`; `empty_dict = {}` — inference and codegen for empty collections |

---

## What Was Fixed by Language Hardening

The following improvements from Milestones 1–6 brought **24 tests** to passing (up from 18 pre-hardening):

| Milestone | Fixes | Tests Unblocked |
|-----------|-------|-----------------|
| **M1: Codegen fixes** | Tuple indexing (`pair[0]` → `pair.0`), int/int division → float, print(None) for some paths | 24, 27 |
| **M3: Ownership v2** | print consumes, string method moves, list mutation after use, chained operations | 12, 14, 23, 30 |
| **M4–M6** | Subscript mutation, iteration, builtins | (indirect improvements) |

**Notable gains:**
- **12, 14, 23, 30** — Ownership fixes allow reuse after print and method calls; chained operations and f-strings work.
- **24** — Tuple index codegen fix (`pair.0` instead of `pair.0_i64`).
- **27** — int/int division now emits float coercion.

**Milestone 9: inference_v2** (pending) targets:
1. **Return type inference** — Infer `-> ReturnType` from all `return` statements instead of defaulting to `None` (would fix 05).
2. **Parameter type inference for nested functions** — Infer parameter types from usage context.
3. **Result unwrapping in try blocks** — Infer success type `T` instead of `Result[T, E]` (would fix 17).

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Blocks Real-World Usage

1. **Return type inference** (05) — Every function requires explicit `-> ReturnType`. Milestone 9 will address this.
2. **Union return wrapping** (16) — Functions returning `int | str` fail at Rust build. Codegen must wrap values in enum variants.
3. **Result unwrapping in try** (17) — Variables in try blocks inferred as `Result` instead of success type. Milestone 9 will address this.

### Tier 2 — Significant Ergonomics

4. **print(None) / unit type Display** (01) — Printing `None` fails. Codegen should emit `println!("None")` or use `Debug`.
5. **Empty collection inference** (18) — `[]` and `{}` need a sensible default (e.g., `list[Any]` with explicit annotation requirement, or `list[int]` from first append). Current `dyn Any` causes Eq/Hash/codegen issues.

### Tier 3 — By Design

6. **Reassignment to different type** (20) — Correctly rejected. No change needed.

---

## Test File Index (Post-Hardening)

| File | Status | Root Cause |
|------|--------|------------|
| `01_variable_from_literal.sifr` | FAIL (Rust) | print(None) / unit Display |
| `02_variable_from_expression.sifr` | PASS | — |
| `03_variable_from_function_call.sifr` | PASS | — |
| `04_collection_literal_inference.sifr` | PASS | — |
| `05_return_type_inference.sifr` | FAIL (Rust) | Return type inference, Display |
| `06_infer_from_method_call.sifr` | PASS | — |
| `07_infer_from_conditional.sifr` | PASS | — |
| `08_infer_from_comprehension.sifr` | PASS | — |
| `09_infer_from_map_filter.sifr` | PASS | — |
| `10_infer_from_class_constructor.sifr` | PASS | — |
| `11_infer_in_for_loop.sifr` | PASS | — |
| `12_infer_chained_operations.sifr` | PASS | — |
| `13_infer_mixed_no_annotation.sifr` | PASS | — |
| `14_infer_from_fstring.sifr` | PASS | — |
| `15_infer_from_optional_return.sifr` | PASS | — |
| `16_infer_from_union_return.sifr` | FAIL (Rust) | Union return wrapping |
| `17_infer_from_result_return.sifr` | FAIL (Rust) | Result Display / try unwrap |
| `18_infer_empty_collection.sifr` | FAIL (Rust) | Empty collection inference |
| `19_infer_reassignment_same_type.sifr` | PASS | — |
| `20_infer_reassignment_different_type.sifr` | FAIL (Sifr) | Intentional — static typing |
| `21_infer_from_builtin_functions.sifr` | PASS | — |
| `22_infer_nested_collection.sifr` | PASS | — |
| `23_infer_class_field_access.sifr` | PASS | — |
| `24_infer_from_index_access.sifr` | PASS | — |
| `25_infer_from_walrus.sifr` | PASS | — |
| `26_infer_lambda_param_from_context.sifr` | PASS | — |
| `27_infer_multiline_no_annotations.sifr` | PASS | — |
| `28_infer_from_zip_enumerate.sifr` | PASS | — |
| `29_infer_from_any_all.sifr` | PASS | — |
| `30_infer_from_string_ops.sifr` | PASS | — |
