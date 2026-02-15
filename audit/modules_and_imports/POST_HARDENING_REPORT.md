# Post-Hardening Audit Report: Modules & Imports

**Date:** February 15, 2026  
**Scope:** 5 test files in `audit/modules_and_imports/`  
**Context:** Post Language Hardening Phase

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 4 | 80.0% |
| **Fail (Rust compile)** | 1 | 20.0% |
| **Total** | 5 | 100% |

---

## Passing Tests (4)

These tests compile and run correctly after the Language Hardening phase:

| # | Test | Notes |
|---|------|-------|
| 01 | `01_basic_import.sifr` | Basic import |
| 02 | `02_from_import.sifr` | From-import syntax |
| 04 | `04_multi_file.sifr` | Multi-file modules |
| 05 | `05_stdlib_imports.sifr` | Standard library imports |

---

## Failure Categories

### 1. Import Alias Codegen — Original Name Referenced in Generated Rust

**Error:** `cannot find function 'square_root' in this scope` (E0425)

When using `import X as Y`, the import alias codegen emits the alias correctly, but the generated Rust code still references the original name (e.g., `square_root`) instead of the alias. The stdlib function is thus not found because the alias name should be used in the emitted code.

| File | Description |
|------|-------------|
| `03_import_as.sifr` | Import alias; codegen emits alias but references original name in generated Rust |

---

## What Was Fixed by Language Hardening

The Language Hardening phase brought **4 tests** to passing. Notable improvements:

| Fix | Tests Unblocked |
|-----|-----------------|
| **Basic import** | `01_basic_import` — module import and member access |
| **From-import** | `02_from_import` — `from X import Y` syntax |
| **Multi-file modules** | `04_multi_file` — cross-file module resolution |
| **Stdlib imports** | `05_stdlib_imports` — standard library module imports |

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Blocks Real-World Usage

1. **Import alias codegen** (03) — `import X as Y` emits the alias but generated Rust still references the original name. Codegen must use the alias name when emitting function calls and references.

---

## Test File Index (Post-Hardening)

| File | Status | Root Cause |
|------|--------|------------|
| `01_basic_import.sifr` | PASS | — |
| `02_from_import.sifr` | PASS | — |
| `03_import_as.sifr` | FAIL (Rust) | Import alias codegen references original name |
| `04_multi_file.sifr` | PASS | — |
| `05_stdlib_imports.sifr` | PASS | — |
