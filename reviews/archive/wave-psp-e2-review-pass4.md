# wave_psp_e2 Review Pass 4

**Reviewer:** External Reviewer
**Scope:** Class-Heavy and Custom Cleanup (`argparse`, `ipaddress`, `uuid`, `graphlib`, `test`)
**Date:** 2026-03-16
**Status:** Not Ready for Production - Significant Contract Drift

---

## Executive Summary

This review identifies a **critical discrepancy** between the documented adapted parity contract and the actual implementation state of wave_psp_e2 on the main branch.

**Key Finding:** The wave_psp_e2 implementation described in the review passes (pass 1-3) was **never merged to main**. The main branch contains only the pre-wave, simplified implementations that predate the wave_psp_e2 effort. The review passes appear to document an implementation on a feature branch that was never integrated.

**Verdict:** Not ready for production. The current main branch does not reflect the wave_psp_e2 implementation described in the documentation.

---

## Current Implementation State (Main Branch)

### argparse.sifr
**File:** `lib/sifr/argparse.sifr` (62 lines)
**Last Modified:** Feb 16, 2026

| Feature | Status |
|---------|--------|
| `parse_flag()` | Implemented |
| `parse_option()` | Implemented |
| `parse_positional()` | Implemented |
| `ArgumentParser` class | NOT IMPLEMENTED |
| `--name=value` inline parsing | NOT IMPLEMENTED |
| `--` end-of-options marker | NOT IMPLEMENTED |

**Classification:** Minimal utility functions, not a parity surface

### ipaddress.sifr
**File:** `lib/sifr/ipaddress.sifr` (124 lines)
**Last Modified:** Feb 17, 2026

| Feature | Status |
|---------|--------|
| `is_valid_ipv4()` | Implemented (no leading-zero rejection) |
| `is_private()` | Implemented |
| `is_loopback()` | Implemented |
| `is_multicast()` | Implemented |
| `is_global()` | Implemented |
| `int_to_ip()` | Implemented |
| `IPv4Address` class | NOT IMPLEMENTED |
| `AddressValueError` | NOT IMPLEMENTED |
| IPv6 support | NOT IMPLEMENTED |
| Leading-zero rejection | NOT IMPLEMENTED |

**Classification:** Functional IPv4 helpers only

### uuid.sifr
**File:** `lib/sifr/uuid.sifr` (179 lines)
**Last Modified:** Mar 10, 2026

| Feature | Status |
|---------|--------|
| `uuid4()` intrinsic | Implemented |
| `UUID` class | Implemented |
| `uuid_from_hex()` | Implemented |
| `urn:uuid:` prefix handling | NOT IMPLEMENTED |
| `{...}` curly brace handling | NOT IMPLEMENTED |
| UUID1/3/5/6/7 | NOT IMPLEMENTED |

**Classification:** Basic UUID4 support only

### graphlib.sifr
**File:** `lib/sifr/graphlib.sifr` (64 lines)
**Last Modified:** Feb 17, 2026

| Feature | Status |
|---------|--------|
| `topological_sort()` function | Implemented |
| `TopologicalSorter` class | Implemented |
| `add_many()` method | NOT IMPLEMENTED |
| Sparse node filtering | NOT IMPLEMENTED |
| `Graph` class (Python 3.9+) | NOT IMPLEMENTED |

**Classification:** Basic topological sort only

### test.sifr
**File:** `lib/sifr/test.sifr` (84 lines)
**Last Modified:** Mar 16, 2026

| Feature | Status |
|---------|--------|
| `assert_eq/ne/true/false` | Implemented |
| `assert_almost_eq` | Implemented |
| `assert_gt/ge/lt/le` | Implemented |
| `assert_some/none` | Implemented |
| `assert_ok/err` | Implemented |
| `unittest.TestCase` | NOT IMPLEMENTED |

**Classification:** Assertion helpers only

---

## Review Pass Documentation vs. Reality

### Review Pass 1-3 Claims vs. Actual Implementation

| Claim (Pass 3) | Actual Main Branch |
|----------------|-------------------|
| argparse: `ArgumentParser` class with `--name=value` parsing | Only `parse_flag/option/positional` functions |
| ipaddress: Leading-zero rejection, `AddressValueError`, `IPv4Address` class | Basic functional API only |
| uuid: URN/curly-brace handling | Only basic hex parsing |
| graphlib: `add_many()`, sparse node filtering, `max_node=-1` | Only basic `add()` and `static_order()` |

### Missing Artifacts

The following artifacts are referenced in the review passes but **do not exist on main**:

1. **Traceability document**: `verification/stdlib/wave_psp_e2_cpython_traceability.md` - DOES NOT EXIST
2. **Wave demo**: `demos/wave_psp_e2_*.sifr` - DOES NOT EXIST
3. **Phase fail tests**: `phase_psp_e2_*.sifr` - DOES NOT EXIST
4. **CPython subset tests**: `cpython_argparse_subset.sifr`, `cpython_ipaddress_subset.sifr`, `cpython_graphlib_subset.sifr` - DO NOT EXIST

---

## Git History Analysis

```
414629757 Close wave_psp_e2 class-heavy parity surfaces (#1205)  <- NOT ON MAIN
a441b0dd7 Close wave_psp_e2 class-heavy parity surfaces            <- NOT ON MAIN
```

The commit that implements wave_psp_e2 exists on feature branches:
- `codex/python-builtin-std-parity-wave-e2`
- `codex/python-builtin-std-parity-wave-e2-review-pass1`
- `codex/python-builtin-std-parity-wave-e2-review-pass2`

But **not on main branch**. The most recent main commit is `4ad100ee` which merges wave_psp_c2.

---

## Validation Evidence

### Tests that DO exist on main:
| Test File | Status |
|-----------|--------|
| `crates/sifr/tests/e2e/pass/stdlib_argparse.sifr` | PASS |
| `crates/sifr/tests/e2e/pass/stdlib_ipaddress.sifr` | PASS |
| `crates/sifr/tests/e2e/pass/stdlib_ipaddress_extended.sifr` | PASS |
| `crates/sifr/tests/e2e/pass/stdlib_graphlib.sifr` | PASS |
| `crates/sifr/tests/e2e/pass/stdlib_graphlib_class.sifr` | PASS |
| `crates/sifr/tests/e2e/pass/error_stdlib_graphlib.sifr` | PASS |
| `crates/sifr/tests/e2e/pass/stdlib_uuid_consolidated.sifr` | PASS |
| `crates/sifr/tests/e2e/pass/cpython_uuid_subset.sifr` | PASS |

### Tests that DON'T exist (referenced in reviews but missing):
- `cpython_argparse_subset.sifr`
- `cpython_ipaddress_subset.sifr`
- `cpython_graphlib_subset.sifr`
- `phase_psp_e2_*.sifr`
- `wave_psp_e2_class_heavy_custom_cleanup_demo.sifr`

---

## Traceability Contract Fidelity

### Current State: BROKEN

The wave_psp_e2 adapted parity contract cannot be evaluated because:

1. **No traceability document exists** - The contract is not formally documented
2. **Implementation does not match documented scope** - The review passes describe features that don't exist on main
3. **No phase-specific tests** - There are no `phase_psp_e2_*.sifr` test files to validate the contract
4. **No demo file** - The wave demo referenced in other waves is absent

---

## Regression Risk Assessment

### Current Risk: LOW (for existing simplified implementations)

The existing simplified implementations on main are stable:
- All 8 test files pass
- No known regressions
- Modules have been in production use since earlier phases

### Risk if wave_psp_e2 were to be merged: MEDIUM

If the feature branch implementation were to be merged:
- The changes are significant (new classes, new error types, new methods)
- No fail tests exist to validate error handling
- The sparse-node filtering change could affect existing graph users

---

## Correctness Findings

### Finding 1: Contract Drift - CRITICAL

**Severity:** Blocking

The review passes 1-3 document a wave_psp_e2 implementation that does not exist on the main branch. This creates a false impression of parity closure.

**Evidence:**
- Review pass 3 claims argparse has `ArgumentParser` class with inline option parsing
- Main branch only has `parse_flag()`, `parse_option()`, `parse_positional()` functions
- Review claims IPv4 address leading-zero rejection is implemented
- Main branch accepts "01.2.3.4" as valid

**Recommendation:**
1. Either merge the feature branch implementation to main
2. Or update the phase tracking to correctly reflect that wave_psp_e2 is not yet implemented on main

### Finding 2: Missing Traceability Document - HIGH

**Severity:** Non-blocking but required for contract fidelity

No `wave_psp_e2_cpython_traceability.md` exists in `verification/stdlib/`. Other waves have this document but wave_psp_e2 lacks it.

**Recommendation:**
Create the traceability document following the pattern of `wave_psp_c2_cpython_traceability.md`.

### Finding 3: Missing Wave Artifacts - MEDIUM

**Severity:** Non-blocking but standard expectation

- No `demos/wave_psp_e2_*.sifr` demo file
- No `phase_psp_e2_*.sifr` fail test cases
- No CPython subset tests for argparse, ipaddress, graphlib

**Recommendation:**
These artifacts are standard for wave production-readiness and should be added.

---

## Actionable Correctness Gaps

### 1. Merge wave_psp_e2 to main or update status

**Priority:** HIGH

The wave_psp_e2 implementation exists on feature branches but has not been merged to main. The phase execution tracking shows it as "pending" but the review passes document it as complete.

**Resolution path:**
- Option A: Merge PR #1205 (wave_psp_e2 closure) to main
- Option B: Update `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md` to correctly reflect that wave_psp_e2 is NOT yet on main

### 2. Create traceability document

**Priority:** MEDIUM

The wave needs a `verification/stdlib/wave_psp_e2_cpython_traceability.md` following the pattern of other waves.

### 3. Add wave-specific test artifacts

**Priority:** MEDIUM

Create fail tests and demo file per wave conventions.

---

## Conclusion

**Status:** NOT READY FOR PRODUCTION

The wave_psp_e2 implementation described in review passes 1-3 has NOT been merged to the main branch. The current main branch contains only the pre-wave simplified implementations that were present before the wave_psp_e2 effort began.

This creates a significant **traceability contract fidelity issue** - the documented contract and the actual implementation are not aligned.

**Immediate action required:** Either merge the wave_psp_e2 feature branch to main, or update the phase tracking to correctly reflect that this wave remains in "pending" status.

---

## Recommendation

**Do not approve as production-ready** until:

1. The wave_psp_e2 implementation is merged to main, OR
2. The phase tracking documents are updated to reflect that wave_psp_e2 is not yet on main, OR
3. The review passes are invalidated as describing a non-merged feature branch

**Code changes required:** Yes - merge from feature branch or update documentation.

---

*Review conducted against main branch commit: 4ad100ee*
