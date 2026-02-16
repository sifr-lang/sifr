# Post-Hardening Audit Report: Standard Library

**Date:** February 16, 2026  
**Scope:** 10 test files in `audit/stdlib/`  
**Context:** Post borrow-by-default phase

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 6 | 60.0% |
| **Fail (Sifr compile)** | 1 | 10.0% |
| **Fail (Rust compile)** | 3 | 30.0% |
| **Total** | 10 | 100% |

---

## Passing Tests (6)

These tests compile and run correctly after the borrow-by-default phase:

| # | Test | Notes |
|---|------|-------|
| 01 | `01_math.sifr` | Math module (log, sin, cos, tan, abs_val) |
| 03 | `03_string_full.sifr` | String module (full API) |
| 04 | `04_re.sifr` | Regex module |
| 06 | `06_io.sifr` | IO module (write_text, read_text, exists) |
| 07 | `07_time.sifr` | Time module |
| 08 | `08_env.sifr` | Env module (env_get, env_set) |

---

## Failure Categories

### 1. Collections Module — `Set` Not in `sifr.collections`

**Error:** `module 'sifr.collections' has no member 'Set'`

`Set` is a builtin type in Sifr, not a member of `sifr.collections`. Tests expecting `collections.Set` need to use the builtin `Set` instead, or the collections module needs a re-export.

| File | Description |
|------|-------------|
| `05_collections.sifr` | Set is builtin, not in collections module |

---

### 2. JSON Module — Duplicate Definitions and Type Errors in Generated Rust

**Error:** `E0428 countBits defined multiple times`, `E0369 cannot multiply Vec<...> by i64`

The test now passes Sifr compilation but the generated Rust has duplicate function definitions and invalid operations (multiplying a Vec by an integer).

| File | Description |
|------|-------------|
| `02_json.sifr` | Rust codegen: duplicate definitions, invalid Vec multiplication |

---

### 3. Borrow-by-Default Codegen — Cannot Borrow as Mutable Behind `&` Reference

**Error:** `E0596 cannot borrow *flowerbed as mutable, as it is behind a & reference`

Parameters are now passed by immutable reference (`&`) under borrow-by-default, but the test code attempts to mutate them. The codegen needs to detect mutation and use `&mut` references, or the test patterns need adjustment.

| File | Description |
|------|-------------|
| `09_random.sifr` | Cannot borrow `*flowerbed` as mutable (behind `&` reference) |
| `10_hash_encoding.sifr` | Cannot borrow `*flowerbed` as mutable (behind `&` reference) |

> **Note:** The "flowerbed" variable name in both errors suggests the test files may have been modified, or there is a codegen issue where borrow-by-default passes parameters as `&` but the generated code tries to mutate them.

---

## Improvements Since Last Audit (February 15, 2026)

Previous result: **3 PASS, 7 Fail (all Sifr compile)**.  
Current result: **6 PASS, 1 Fail (Sifr compile), 3 Fail (Rust compile)**.

**3 new tests now fully pass**, and 3 more progressed past Sifr compilation to Rust compilation:

| Test | Previous | Current | What Changed |
|------|----------|---------|--------------|
| `01_math.sifr` | FAIL (Sifr) — missing log, sin, cos, tan, abs_val | **PASS** | Math module members added to stdlib |
| `06_io.sifr` | FAIL (Sifr) — missing write_text, read_text, exists | **PASS** | IO module members added to stdlib |
| `08_env.sifr` | FAIL (Sifr) — env_get, env_set not found | **PASS** | Env module API names aligned |
| `02_json.sifr` | FAIL (Sifr) — json_dumps type restriction | FAIL (Rust) | Progressed past Sifr; Rust codegen issues remain |
| `09_random.sifr` | FAIL (Sifr) — random_choice not generic | FAIL (Rust) | Progressed past Sifr; borrow-by-default mutation issue |
| `10_hash_encoding.sifr` | FAIL (Sifr) — missing md5 | FAIL (Rust) | Progressed past Sifr; borrow-by-default mutation issue |

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Blocks Real-World Usage

1. **JSON codegen** (02) — Generated Rust has duplicate definitions and invalid Vec operations. Codegen needs to handle json_dumps for arbitrary types correctly.
2. **Borrow-by-default mutation** (09, 10) — Parameters passed as `&` but code mutates them. Codegen must detect mutation and emit `&mut` or adjust the ownership model for these patterns.

### Tier 2 — Significant Ergonomics

3. **Collections / Set** (05) — `Set` is builtin; test needs update or collections module needs `Set` re-export for compatibility.

---

## Test File Index

| File | Status | Root Cause |
|------|--------|------------|
| `01_math.sifr` | PASS | — |
| `02_json.sifr` | FAIL (Rust) | Duplicate definitions, invalid Vec ops |
| `03_string_full.sifr` | PASS | — |
| `04_re.sifr` | PASS | — |
| `05_collections.sifr` | FAIL (Sifr) | Set not in collections module |
| `06_io.sifr` | PASS | — |
| `07_time.sifr` | PASS | — |
| `08_env.sifr` | PASS | — |
| `09_random.sifr` | FAIL (Rust) | Borrow-by-default: cannot mutate `&` param |
| `10_hash_encoding.sifr` | FAIL (Rust) | Borrow-by-default: cannot mutate `&` param |
