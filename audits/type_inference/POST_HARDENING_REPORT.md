# Post-Hardening Audit Report: Type Inference

**Date:** February 16, 2026  
**Scope:** 30 test files in `audit/type_inference/`  
**Context:** Post borrow-by-default phase

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 19 | 63.3% |
| **Fail (Sifr compile)** | 1 | 3.3% |
| **Fail (Rust compile)** | 8 | 26.7% |
| **Fail (Runtime)** | 2 | 6.7% |
| **Total** | 30 | 100% |

---

## Passing Tests (19)

These tests compile and run correctly after the borrow-by-default phase:

| # | Test | Notes |
|---|------|-------|
| 02 | `02_variable_from_expression.sifr` | Arithmetic, concat, comparison, logic |
| 03 | `03_variable_from_function_call.sifr` | Infer from return type annotation |
| 04 | `04_collection_literal_inference.sifr` | list, dict, tuple literals |
| 06 | `06_infer_from_method_call.sifr` | .upper(), len(), sorted() |
| 08 | `08_infer_from_comprehension.sifr` | List comprehensions |
| 09 | `09_infer_from_map_filter.sifr` | map/filter with lambda |
| 10 | `10_infer_from_class_constructor.sifr` | Class instantiation |
| 11 | `11_infer_in_for_loop.sifr` | Loop variable from iterable |
| 12 | `12_infer_chained_operations.sifr` | Chained filter/map/len |
| 13 | `13_infer_mixed_no_annotation.sifr` | Full program, no var annotations |
| 18 | `18_infer_empty_collection.sifr` | Empty collection inference |
| 19 | `19_infer_reassignment_same_type.sifr` | Reassign same type |
| 21 | `21_infer_from_builtin_functions.sifr` | len, abs, min, max, sum, str, bool |
| 22 | `22_infer_nested_collection.sifr` | Nested list/dict/tuple |
| 23 | `23_infer_class_field_access.sifr` | Field type from class |
| 24 | `24_infer_from_index_access.sifr` | List/dict/tuple indexing |
| 26 | `26_infer_lambda_param_from_context.sifr` | Lambda param from collection type |
| 28 | `28_infer_from_zip_enumerate.sifr` | zip/enumerate result types |
| 30 | `30_infer_from_string_ops.sifr` | String method return types |

---

## Failure Categories

### 1. Sifr: Reassignment to Different Type (Intentional)

**Error:** `type mismatch: cannot assign 'str' to variable 'x' of type 'int'`

Sifr enforces static typing — reassigning a variable to a different type after inference is intentionally disallowed. This is by design.

| File | Description |
|------|-------------|
| `20_infer_reassignment_different_type.sifr` | `x = 42` then `x = "hello"` — correctly rejected |

---

### 2. Codegen: `print(None)` / Unit Type Display

**Error:** `E0308 mismatched types`

`None` maps to Rust's `()` (unit type), which does not implement `Display`. The codegen should emit `println!("None")` or use `Debug` formatting for unit type.

| File | Description |
|------|-------------|
| `05_return_type_inference.sifr` | Return type defaults to `None`; inferred `dyn Any` or mixed types don't implement `Display` |

---

### 3. Borrow-by-Default: `&String` vs `String` Comparison

**Error:** `E0277 String: Borrow<&String> not satisfied` / `can't compare &String with String`

Under borrow-by-default, parameters are passed as `&String` but comparison operators expect owned `String` or matching reference types. The codegen needs to dereference or adjust comparison logic for borrowed parameters.

| File | Description |
|------|-------------|
| `07_infer_from_conditional.sifr` | `String: Borrow<&String>` not satisfied in conditional |
| `15_infer_from_optional_return.sifr` | Can't compare `&String` with `String` |
| `16_infer_from_union_return.sifr` | Can't compare `&String` with `String`, plus E0308 mismatched types |
| `17_infer_from_result_return.sifr` | Can't compare `&String` with `String` (3 occurrences) |

---

### 4. Borrow-by-Default: Mismatched Types and Move Errors

**Error:** `E0308 mismatched types`, `E0507 cannot move out of shared reference`

Borrow-by-default introduces `&T` where `T` was expected, causing type mismatches. In some cases, code attempts to move a value out of a shared reference, which Rust forbids.

| File | Description |
|------|-------------|
| `25_infer_from_walrus.sifr` | E0308 mismatched types |
| `27_infer_multiline_no_annotations.sifr` | E0308 mismatched types + E0507 cannot move out of shared reference |
| `29_infer_from_any_all.sifr` | E0308 mismatched types |

---

### 5. Runtime Failures

| File | Error | Description |
|------|-------|-------------|
| `01_variable_from_literal.sifr` | `could not run binary: No such file or directory` | Binary not produced or path incorrect |
| `14_infer_from_fstring.sifr` | Empty output | Binary runs but produces no output |

---

## Regressions Since Last Audit (February 15, 2026)

Previous result: **24 PASS, 1 Fail (Sifr compile), 5 Fail (Rust compile)**.  
Current result: **19 PASS, 1 Fail (Sifr compile), 8 Fail (Rust compile), 2 Fail (Runtime)**.

**6 tests that previously passed now fail** — all related to the borrow-by-default phase:

| Test | Previous | Current | Root Cause |
|------|----------|---------|------------|
| `07_infer_from_conditional.sifr` | PASS | FAIL (Rust) | `String: Borrow<&String>` not satisfied — borrow-by-default codegen |
| `14_infer_from_fstring.sifr` | PASS | FAIL (Runtime) | Empty output — binary runs but produces nothing |
| `15_infer_from_optional_return.sifr` | PASS | FAIL (Rust) | `&String` vs `String` comparison — borrow-by-default codegen |
| `25_infer_from_walrus.sifr` | PASS | FAIL (Rust) | Mismatched types — borrow-by-default codegen |
| `27_infer_multiline_no_annotations.sifr` | PASS | FAIL (Rust) | Mismatched types + cannot move out of shared reference |
| `29_infer_from_any_all.sifr` | PASS | FAIL (Rust) | Mismatched types — borrow-by-default codegen |

**Pattern:** 5 of the 6 regressions are Rust compile failures caused by borrow-by-default introducing `&T` references where the generated code expects owned `T` values. The codegen needs to insert dereferences, adjust comparisons, or clone where appropriate.

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Borrow-by-Default Regressions (Blocks Previously Working Code)

1. **`&String` vs `String` comparisons** (07, 15, 16, 17) — Borrowed parameters can't be compared with owned strings. Codegen must dereference or adjust comparison operators.
2. **Mismatched types from borrows** (25, 27, 29) — Borrow-by-default introduces `&T` where `T` is expected. Codegen must insert derefs or clones.
3. **Cannot move out of shared reference** (27) — Code tries to move a value from behind `&`. Codegen must clone or restructure.

### Tier 2 — Pre-Existing Issues

4. **Return type inference** (05) — Functions default to `None` return type; `dyn Any` doesn't implement `Display`.
5. **Runtime: binary not found** (01) — Binary not produced or path incorrect for `print(None)` test.
6. **Runtime: empty output** (14) — F-string test binary runs but produces no output.

### Tier 3 — By Design

7. **Reassignment to different type** (20) — Correctly rejected. No change needed.

---

## Test File Index

| File | Status | Root Cause |
|------|--------|------------|
| `01_variable_from_literal.sifr` | FAIL (Runtime) | Binary not found |
| `02_variable_from_expression.sifr` | PASS | — |
| `03_variable_from_function_call.sifr` | PASS | — |
| `04_collection_literal_inference.sifr` | PASS | — |
| `05_return_type_inference.sifr` | FAIL (Rust) | Return type inference / Display |
| `06_infer_from_method_call.sifr` | PASS | — |
| `07_infer_from_conditional.sifr` | FAIL (Rust) | Borrow: `&String` comparison **[REGRESSION]** |
| `08_infer_from_comprehension.sifr` | PASS | — |
| `09_infer_from_map_filter.sifr` | PASS | — |
| `10_infer_from_class_constructor.sifr` | PASS | — |
| `11_infer_in_for_loop.sifr` | PASS | — |
| `12_infer_chained_operations.sifr` | PASS | — |
| `13_infer_mixed_no_annotation.sifr` | PASS | — |
| `14_infer_from_fstring.sifr` | FAIL (Runtime) | Empty output **[REGRESSION]** |
| `15_infer_from_optional_return.sifr` | FAIL (Rust) | Borrow: `&String` vs `String` **[REGRESSION]** |
| `16_infer_from_union_return.sifr` | FAIL (Rust) | Borrow: `&String` comparison + E0308 |
| `17_infer_from_result_return.sifr` | FAIL (Rust) | Borrow: `&String` comparison (×3) |
| `18_infer_empty_collection.sifr` | PASS | — |
| `19_infer_reassignment_same_type.sifr` | PASS | — |
| `20_infer_reassignment_different_type.sifr` | FAIL (Sifr) | Intentional — static typing |
| `21_infer_from_builtin_functions.sifr` | PASS | — |
| `22_infer_nested_collection.sifr` | PASS | — |
| `23_infer_class_field_access.sifr` | PASS | — |
| `24_infer_from_index_access.sifr` | PASS | — |
| `25_infer_from_walrus.sifr` | FAIL (Rust) | Borrow: mismatched types **[REGRESSION]** |
| `26_infer_lambda_param_from_context.sifr` | PASS | — |
| `27_infer_multiline_no_annotations.sifr` | FAIL (Rust) | Borrow: mismatched types + move error **[REGRESSION]** |
| `28_infer_from_zip_enumerate.sifr` | PASS | — |
| `29_infer_from_any_all.sifr` | FAIL (Rust) | Borrow: mismatched types **[REGRESSION]** |
| `30_infer_from_string_ops.sifr` | PASS | — |
