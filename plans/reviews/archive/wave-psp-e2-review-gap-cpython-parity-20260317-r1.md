# wave_psp_e2 Review: Implementation Gaps and CPython Parity Quality

**Date:** 2026-03-17
**Reviewer:** agent (agent)
**Status:** CRITICAL GAPS IDENTIFIED - Regression from main

---

## Executive Summary

wave_psp_e2 targets "class-heavy and custom cleanup" surfaces covering 5 CPython test families: `argparse`, `ipaddress`, `uuid`, `graphlib`, and `test` (unittest-style assertions).

**CRITICAL FINDING:** The wave_psp_e2 branch (`codex/python-builtin-std-parity-wave-e2`) contains **significant regressions** from the current main branch. Multiple features that exist and work correctly in main have been **removed** in the wave_psp_e2 implementation.

---

## 1. Critical Implementation Gaps (Regressions from Main)

### 1.1 argparse — CRITICAL REGRESSION

**Issue:** The wave_psp_e2 branch has **removed** core argparse functionality that exists in main.

| Feature | Status in Main | Status in wave_psp_e2 | Gap |
|---------|----------------|----------------------|-----|
| Inline options (`--option=value`) | ✅ Working | ❌ Removed | `_split_inline_option` function removed |
| Double-dash (`--`) handling | ✅ Working | ❌ Removed | `force_positional` logic removed |
| Option-like token detection | ✅ Working | ❌ Removed | `_is_option_like_token` function removed |
| Skip option-like values | ✅ Working | ❌ Removed | Logic to detect `--option` as value removed |

**Files Affected:**
- `lib/sifr/argparse.sifr`: Removed ~60 lines of working functionality

**CPython Parity Impact:**
- CPython argparse supports `--option=value` format — no longer supported
- CPython argparse supports `--` to stop option parsing — no longer supported
- Missing option fallback when next token looks like an option — broken

**Test Coverage Gap:**
- `cpython_argparse_subset.sifr` tests reduced from 19 assertions to 7 assertions
- Removed tests: `collect_option_token_shape_actual()`, `collect_missing_option_value_actual()`

---

### 1.2 ipaddress — CRITICAL REGRESSION

**Issue:** The wave_psp_e2 branch has **removed** critical IPv4 classification functionality.

| Feature | Status in Main | Status in wave_psp_e2 | Gap |
|---------|----------------|----------------------|-----|
| Leading-zero rejection | ✅ Working | ❌ Removed | `01.2.3.40` now passes validation |
| `is_link_local()` | ✅ Working | ❌ Removed | 169.254.0.0/16 detection missing |
| `is_reserved()` | ✅ Working | ❌ Removed | 240.0.0.0/4 detection missing |
| Full private range handling | ✅ Complete | ❌ Incomplete | Missing 100.64.0.0/10, 198.18/15, 198.51.100/24, 203.0.113.0/24 |
| `is_global()` correct logic | ✅ Working | ❌ Broken | Simplified logic doesn't handle CGN (100.64.0.0/10) |

**Files Affected:**
- `lib/sifr/ipaddress.sifr`: Removed `_in_ipv4_range()`, `_is_private_ipv4_value()`, `is_link_local()`, `is_reserved()` and IPv4Address methods

**CPython Parity Impact:**
- CPython rejects leading-zero IPv4 addresses — Sifr now accepts them
- CPython classifies 169.254.x.x as link-local — Sifr classification broken
- CPython classifies 100.64.0.0/10 as not global — Sifr incorrectly reports as global

---

### 1.3 graphlib — CRITICAL REGRESSION

**Issue:** The wave_psp_e2 branch has **broken** sparse node graph handling.

| Feature | Status in Main | Status in wave_psp_e2 | Gap |
|---------|----------------|----------------------|-----|
| Empty graph handling | ✅ Working (`max_node=-1`) | ❌ Broken (`max_node=0`) | Empty graph crashes |
| Explicit node tracking | ✅ Working | ❌ Removed | `nodes` list removed |
| Sparse node filtering | ✅ Working | ❌ Removed | `_filter_order()` removed |
| `static_order()` filtering | ✅ Returns explicit nodes only | ❌ Returns all nodes | Leaks undeclared nodes |

**Files Affected:**
- `lib/sifr/graphlib.sifr`: Changed initialization, removed filtering logic

**CPython Parity Impact:**
- CPython TopologicalSorter only returns explicitly-added nodes in `static_order()` — Sifr now returns all nodes in range

---

### 1.4 uuid — CRITICAL REGRESSION

**Issue:** The wave_psp_e2 branch has **removed** UUID format parsing functionality.

| Feature | Status in Main | Status in wave_psp_e2 | Gap |
|---------|----------------|----------------------|-----|
| URN format (`urn:uuid:...`) | ✅ Working | ❌ Removed | No longer recognized |
| Curly brace format (`{...}`) | ✅ Working | ❌ Removed | No longer recognized |
| Helper functions | ✅ Working | ❌ Removed | `_substring()`, `_starts_with()` gone |

**Files Affected:**
- `lib/sifr/uuid.sifr`: Removed normalization logic in `_canonical_uuid_text()`

**CPython Parity Impact:**
- CPython uuid accepts `urn:uuid:...` — Sifr rejects it
- CPython uuid accepts `{...}` — Sifr rejects it

---

### 1.5 test (unittest assertions) — No Issues

The `sifr.test` module appears unchanged between main and wave_psp_e2. No gaps identified.

---

## 2. Traceability vs. Contract

The traceability document (`verification/stdlib/wave_psp_e2_cpython_traceability.md`) states:

| Module | Contract | Wave_e2 Status | Gap |
|--------|----------|----------------|-----|
| argparse | "inline option values, end-of-options positional mode, and missing-option fallback" | ❌ All three removed | **CONTRACT VIOLATION** |
| ipaddress | "IPv4 classification was aligned with CPython special-range behavior... 100.64/10 and 192.0.0.9/.10 exceptions" | ❌ 100.64/10 not handled, `is_link_local`, `is_reserved` missing | **CONTRACT VIOLATION** |
| graphlib | "explicit added nodes and no longer leaks undeclared intermediary nodes" | ❌ Now leaks undeclared nodes | **CONTRACT VIOLATION** |
| uuid | "supports... `urn:uuid:...` and `{...}` forms" | ❌ Both removed | **CONTRACT VIOLATION** |

---

## 3. Test Coverage Analysis

### 3.1 Test Files Affected

| Test File | Main Assertions | wave_e2 Assertions | Gap |
|-----------|-----------------|---------------------|-----|
| `cpython_argparse_subset.sifr` | 19 | 7 | -12 (63% reduction) |
| `cpython_ipaddress_subset.sifr` | 20 | Unknown (missing methods) | Methods not tested |
| `cpython_graphlib_subset.sifr` | Unknown | Unknown (filtering removed) | Behavior change not tested |

### 3.2 Demo File

`demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr` uses features that no longer work:
- `is_link_local()` — Method doesn't exist
- Inline option parsing (`--mode=inline`) — No longer supported

---

## 4. Validation Results

### 4.1 Attempted Execution

```
$ cargo run -q -p sifr -- run demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr

Error: type error: class 'IPv4Address' has no method 'is_link_local'
```

The demo **does not run** with the wave_psp_e2 implementation.

---

## 5. Root Cause Analysis

The wave_psp_e2 branch appears to have been created from a baseline that was **earlier than main**, causing the implementation to regress existing functionality. This is the opposite of what a wave implementation should do — it should build upon the current main branch, not replace working code with incomplete implementations.

---

## 6. Recommendations

### 6.1 Immediate Actions Required

1. **Revert argparse changes**: Restore inline option handling, double-dash support, and option-like token detection
2. **Restore ipaddress functions**: Re-add `is_link_local()`, `is_reserved()`, leading-zero rejection, and full private range handling
3. **Restore graphlib filtering**: Re-add explicit node tracking and filtering in `static_order()`
4. **Restore uuid parsing**: Re-add URN and curly brace format support
5. **Update tests**: Restore full test coverage for all features

### 6.2 Before Merging

1. Run `demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr` — must pass
2. Verify all 19 argparse assertions pass
3. Verify all ipaddress assertions pass (including leading-zero rejection)
4. Verify graphlib sparse node handling works correctly
5. Verify uuid URN and curly brace formats work

---

## 7. Conclusion

**Status:** NOT READY FOR MERGE

The wave_psp_e2 implementation contains **critical regressions** that:
1. Remove features that already work in main
2. Violate the adopt/adapt/waive contract documented in the traceability
3. Break the demo file
4. Reduce test coverage by 60%+ for argparse alone

The wave_psp_e2 branch must be rebased onto the current main branch and the missing features restored before this wave can be considered for merge.
