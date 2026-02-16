# Post-Hardening Audit Report: Lexical & Syntax

**Date:** February 16, 2026  
**Scope:** 7 test files in `audit/lexical_and_syntax/`  
**Context:** Post borrow-by-default phase

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 4 | 57% |
| **Fail (Sifr compile)** | 0 | 0% |
| **Fail (Rust compile)** | 3 | 43% |
| **Fail (Runtime)** | 0 | 0% |
| **Total** | 7 | 100% |

---

## Passing Tests (4)

| # | Test | Notes |
|---|------|-------|
| 01 | `01_numeric_literals.sifr` | Numeric literals |
| 02 | `02_string_literals.sifr` | String literals |
| 03 | `03_comments_whitespace.sifr` | Comments and whitespace |
| 07 | `07_assignment_forms.sifr` | Assignment forms |

---

## Regressions Since Last Report

**Previous report (February 15, 2026):** 7 PASS, 0 Fail — ALL PASSING.

**Current report:** 4 PASS, 3 Fail (Rust compile).

The following 3 tests previously passed and now fail at Rust compilation:

| Test | Previous | Current | Rust Error(s) |
|------|----------|---------|---------------|
| `04_line_continuation.sifr` | PASS | FAIL | E0308 mismatched types (2 errors) |
| `05_expression_precedence.sifr` | PASS | FAIL | E0368 binary += on Box\<dyn Any\>; E0308; E0599 Vec\<Box\<dyn Any\>> join |
| `06_bitwise_operators.sifr` | PASS | FAIL | E0384 cannot assign twice to immutable variable (l, r) |

---

## Root Cause Analysis

### 04_line_continuation.sifr — E0308 mismatched types

**Symptom:** Two E0308 mismatched types errors during Rust compilation.

**Root cause:** Borrow-by-default changes parameter and value semantics. The test uses `list[int]`, `dict[str, int]`, and multi-line expressions. Likely causes:

- **Collection type mismatch:** Codegen may emit `&Vec<i64>` or `&HashMap<String, i64>` where `Vec<i64>` or owned types are expected (or vice versa), due to convention propagation.
- **Borrow vs owned in print:** `print(items)` and `print(d)` pass collections; if `print` now receives borrowed args by default, the generated Rust may have `&T`/`T` mismatches at call sites.
- **Temporary lifetime / type inference:** Multi-line expressions like `(1 + 2 + 3 + 4 + 5)` or list/dict literals may produce types that no longer match the expected signatures after convention changes.

**Fix direction:** Align codegen for collection literals and `print` call sites with borrow-by-default conventions; ensure `list[int]` and `dict[str, int]` emit types consistent with parameter conventions.

---

### 05_expression_precedence.sifr — E0368, E0308, E0599

**Symptom:** E0368 (binary += on Box\<dyn Any\>), E0308 (mismatched types), E0599 (Vec\<Box\<dyn Any\>> join).

**Root cause:** Type inference falling back to `Any` / `Box<dyn Any>` for complex expressions:

- **Comparison chaining (`1 < x < 10`):** Chained comparisons may be inferred as `Box<dyn Any>` instead of `bool`, causing `+=` or other operations to be emitted on `Box<dyn Any>` (E0368).
- **Boolean precedence (`True or False and False`):** Logical expressions may lose their `bool` type and become `Any`, leading to E0308 when passed to `print` or used in conditions.
- **Vec\<Box\<dyn Any\>> join (E0599):** `print` may receive a `Vec<Box<dyn Any>>` (e.g. from a list of mixed or inferred types) and the generated code may call `.join()` or similar methods that don't exist on that type.

**Fix direction:** Preserve `bool` and `int` types through comparison chaining and logical expressions; avoid `Any` fallback for these. Ensure `print` call sites receive correctly typed arguments.

---

### 06_bitwise_operators.sifr — E0384 cannot assign twice to immutable variable

**Symptom:** E0384 — cannot assign twice to immutable variable (l, r).

**Root cause:** Reassigned variables are not emitted as `mut` in Rust. The test reassigns `flags`:

```sifr
flags: int = 0b0000
flags = flags | 0b0001  # set bit 0
flags = flags | 0b0100  # set bit 2
```

Codegen emits `let flags` instead of `let mut flags`. Rust rejects reassignment to immutable bindings. The error may refer to internal names (e.g. `l`, `r`) if the compiler uses different variable names in the generated code.

**Fix direction:** In codegen, mark any variable that is reassigned (appears on the left of `=`) as `mut` when emitting `let` bindings. This is a known pattern; similar fixes exist for iteration protocol (e.g. `03_unpacking_in_for.sifr`) and LeetCode audits.

---

## Failure Categories

| Category | Count | Tests |
|----------|-------|-------|
| **Rust compile — type mismatch (E0308)** | 2 | 04, 05 |
| **Rust compile — Box\<dyn Any\> ops (E0368, E0599)** | 1 | 05 |
| **Rust compile — immutable reassignment (E0384)** | 1 | 06 |

---

## Test File Index

| File | Status | Root Cause |
|------|--------|------------|
| `01_numeric_literals.sifr` | PASS | — |
| `02_string_literals.sifr` | PASS | — |
| `03_comments_whitespace.sifr` | PASS | — |
| `04_line_continuation.sifr` | FAIL (Rust) | E0308: type mismatch (borrow-by-default / collection conventions) |
| `05_expression_precedence.sifr` | FAIL (Rust) | E0368/E0308/E0599: Box\<dyn Any\> fallback for comparisons/booleans |
| `06_bitwise_operators.sifr` | FAIL (Rust) | E0384: reassigned variable not emitted as `mut` |
| `07_assignment_forms.sifr` | PASS | — |
