# Post-Hardening Audit Report: Standard Library

**Date:** February 15, 2026  
**Scope:** 10 test files in `audit/stdlib/`  
**Context:** Post Language Hardening Phase

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 3 | 30.0% |
| **Fail (Sifr compile)** | 7 | 70.0% |
| **Total** | 10 | 100% |

---

## Passing Tests (3)

These tests compile and run correctly after the Language Hardening phase:

| # | Test | Notes |
|---|------|-------|
| 03 | `03_string_full.sifr` | String module (full API) |
| 04 | `04_re.sifr` | Regex module |
| 07 | `07_time.sifr` | Time module |

---

## Failure Categories

### 1. Math Module — Missing Members

**Error:** `module 'sifr.math' has no member 'log', 'sin', 'cos', 'tan', 'abs_val'`

The `sifr.math` module does not expose `log`, `sin`, `cos`, `tan`, or `abs_val`. These common math functions are missing from the stdlib stub or implementation.

| File | Description |
|------|-------------|
| `01_math.sifr` | Math module missing log, sin, cos, tan, abs_val |

---

### 2. JSON Module — `json_dumps` Type Restriction

**Error:** `argument 1 of 'json_dumps' expected 'str', got 'int'/'bool'/'float'`

`json_dumps` only accepts `str` as its first argument, not arbitrary types (int, bool, float). The function should accept any JSON-serializable type and serialize it to a string.

| File | Description |
|------|-------------|
| `02_json.sifr` | json_dumps only accepts str, not arbitrary types |

---

### 3. Collections Module — `Set` Moved to Builtin

**Error:** `module 'sifr.collections' has no member 'Set'`

`Set` is now a builtin type in Sifr, not a member of `sifr.collections`. Tests or code expecting `collections.Set` need to use the builtin `Set` instead.

| File | Description |
|------|-------------|
| `05_collections.sifr` | Set is now builtin, not in collections |

---

### 4. IO Module — Missing Members

**Error:** `module 'sifr.io' has no member 'write_text', 'read_text', 'exists'`

The `sifr.io` module does not expose `write_text`, `read_text`, or `exists`. These file I/O helpers are missing from the stdlib.

| File | Description |
|------|-------------|
| `06_io.sifr` | IO module missing write_text, read_text, exists |

---

### 5. Env Module — API Name Mismatch

**Error:** `module 'sifr.env' has no member 'env_get', 'env_set'`

The env module uses different API names than expected. The test expects `env_get` and `env_set`; the actual API may use different names (e.g., `get`, `set` or `getenv`, `setenv`).

| File | Description |
|------|-------------|
| `08_env.sifr` | API name mismatch — env_get, env_set not found |

---

### 6. Random Module — `random_choice` Not Generic

**Error:** `random_choice expected 'list[int]', got 'list[str]'`

`random_choice` is typed to accept only `list[int]`, not generic lists. It should accept `list[T]` for any `T` and return an element of that type.

| File | Description |
|------|-------------|
| `09_random.sifr` | random_choice not generic; rejects list[str] |

---

### 7. Hash Module — Missing `md5`

**Error:** `module 'sifr.hash' has no member 'md5'`

The `sifr.hash` module does not expose `md5`. The MD5 hashing function is missing from the stdlib.

| File | Description |
|------|-------------|
| `10_hash_encoding.sifr` | Hash module missing md5 |

---

## What Was Fixed by Language Hardening

The Language Hardening phase brought **3 tests** to passing. Notable improvements:

| Fix | Tests Unblocked |
|-----|-----------------|
| **String module** | `03_string_full` — full string API |
| **Regex module** | `04_re` — regex support |
| **Time module** | `07_time` — time operations |

---

## Remaining Issues (Prioritized by Impact)

### Tier 1 — Blocks Real-World Usage

1. **Math module** (01) — Missing log, sin, cos, tan, abs_val. Core numeric operations.
2. **JSON module** (02) — json_dumps should accept arbitrary JSON-serializable types, not just str.
3. **IO module** (06) — Missing write_text, read_text, exists. Basic file I/O.

### Tier 2 — Significant Ergonomics

4. **Collections / Set** (05) — Set is builtin; tests need update or collections needs Set re-export for compatibility.
5. **Env module** (08) — API name mismatch; align env_get/env_set with implementation.
6. **Random module** (09) — random_choice should be generic over list element type.
7. **Hash module** (10) — Missing md5 for hash/encoding use cases.

---

## Test File Index (Post-Hardening)

| File | Status | Root Cause |
|------|--------|------------|
| `01_math.sifr` | FAIL (Sifr) | Math module missing members |
| `02_json.sifr` | FAIL (Sifr) | json_dumps type restriction |
| `03_string_full.sifr` | PASS | — |
| `04_re.sifr` | PASS | — |
| `05_collections.sifr` | FAIL (Sifr) | Set now builtin |
| `06_io.sifr` | FAIL (Sifr) | IO module missing members |
| `07_time.sifr` | PASS | — |
| `08_env.sifr` | FAIL (Sifr) | Env API name mismatch |
| `09_random.sifr` | FAIL (Sifr) | random_choice not generic |
| `10_hash_encoding.sifr` | FAIL (Sifr) | Hash module missing md5 |
