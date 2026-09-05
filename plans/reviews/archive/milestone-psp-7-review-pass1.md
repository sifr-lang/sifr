# milestone_psp_7 Review Pass 1

**Reviewer:** agent
**Date:** 2026-03-16
**Scope:** Parity governance, exit closure inventory, and waiver governance for milestone_psp_7

---

## Executive Summary

The milestone_psp_7 governance and exit closure infrastructure has **partial foundations** but is **incomplete for final exit closure**. Key gaps exist in consolidated inventories, waiver indexing, and unified traceability across the 45 shipped `lib/sifr` modules. The milestone cannot reach exit closure until all 10 waves (a1-a2, b1-b2, c1-c2, d1-d2, e1-e2) are complete, but the governance framework itself needs consolidation work.

---

## 1. Inventory Correctness

### 1.1 What Exists

| Inventory Type | Location | Status | Coverage |
|---|---|---|---|
| Phase 30 Parity Matrix | `verification/stdlib/phase30_parity_matrix.md` | ✅ Active | ~30 modules with detailed behavior/classification/rationale/evidence |
| Wave Traceability (a1) | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | ✅ Active | Builtins: list, tuple, dict, ord, chr, sorted, reversed, enumerate, zip, map, range |
| Wave Traceability (a2) | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | ✅ Active | Object models: list, dict, set, tuple, str |
| Wave Traceability (b1) | `verification/stdlib/wave_psp_b1_cpython_traceability.md` | ✅ Active | collections, bisect, heapq |
| Wave Traceability (b2) | `verification/stdlib/wave_psp_b2_cpython_traceability.md` | ✅ Active | itertools, functools, operator, random, secrets |
| Wave Traceability (c1) | `verification/stdlib/wave_psp_c1_cpython_traceability.md` | ✅ Active | json, tomllib, csv, configparser |
| Wave Traceability (c2) | `verification/stdlib/wave_psp_c2_cpython_traceability.md` | ✅ Active | string, textwrap, base64, html, fnmatch, difflib, calendar |

### 1.2 What's Missing

| Inventory Type | Requirement | Gap Severity |
|---|---|---|
| **Canonical builtin parity inventory** | Required per spec (line 132) | 🔴 HIGH - Builtins only exist in wave a1 traceability, not as standalone inventory |
| **Canonical core object-model inventory** | Required per spec (line 133) | 🔴 HIGH - Core objects (list, dict, set, tuple, str, bytes) only in wave a2 |
| **Per-module closure inventory** | Required per spec (line 134) | 🟡 MEDIUM - Partial in phase30_matrix + wave files, but no unified view for all 45 modules |
| **Exit-gate closure summary** | Required per spec (line 136) | 🔴 HIGH - Does not exist |

### 1.3 Inventory Gaps Analysis

**Builtins gap**: The builtins surface (list, tuple, dict, set, str, int, float, bool, ord, chr, len, abs, min, max, sum, sorted, reversed, enumerate, zip, map, range, any, all) is documented in wave_a1 traceability but lacks a dedicated inventory file with the same rigor as phase30_parity_matrix.md (status, classification, rationale, owner, tracking_issue, revisit_rule, evidence columns).

**Core object-model gap**: Similar to builtins - covered in wave_a2 but not as a standalone inventory.

**45-module coverage gap**:
```
lib/sifr modules: argparse, base64, bisect, bytes, calendar, collections, configparser, csv,
datetime, difflib, env, fnmatch, functools, glob, graphlib, gzip, hashlib, heapq, html, io,
ipaddress, itertools, json, logging, math, operator, os, pathlib, platform, random, re,
secrets, shutil, statistics, string, subprocess, sys, tempfile, test, textwrap, time,
timeit, tomllib, uuid, zipfile
```

Only ~30 modules have detailed coverage in phase30_parity_matrix.md. Modules in later waves (d1-d2, e1-e2) are not yet implemented and therefore not inventoried.

---

## 2. Waiver Governance Completeness

### 2.1 What Exists

Each wave traceability file contains:
- **Adopt/Adapt/Waive table** with state and local evidence
- **Explicit waivers section** with rationale

| Wave | Waiver Table | Explicit Waivers |
|---|---|---|
| wave_psp_a1 | ✅ 11 entries | ✅ 2 (zip strict, map strict) |
| wave_psp_a2 | ✅ 10 entries | ✅ 1 (bytes/bytearray) |
| wave_psp_b1 | ✅ 5 entries | ✅ 3 (Counter constructors, defaultdict, heapq.merge) |
| wave_psp_b2 | ✅ 11 entries | ✅ Multiple (see file) |
| wave_psp_c1 | ✅ 9 entries | ✅ Multiple |
| wave_psp_c2 | ✅ 11 entries | ✅ 5 (string.Formatter, textwrap, fnmatch, difflib, calendar) |

### 2.2 What's Missing

| Governance Item | Requirement | Gap Severity |
|---|---|---|
| **Waiver index** | Required per spec: "Publish waiver index" (line 136) | 🔴 HIGH - No consolidated index of all waivers across waves |
| **Adopt/adapt/waive ledger** | Required per spec (line 135) | 🟡 MEDIUM - Exists in individual wave files but not consolidated |
| **Traceability matrix for every wave** | Required per spec (line 135) | 🟡 MEDIUM - Exists per-wave but no unified view |

### 2.3 Waiver Governance Gaps Analysis

**No consolidated waiver index**: Each wave has its own waiver tracking, but there's no single document that:
- Lists all waived surfaces across all waves
- Provides a global waiver count by state (adopted/adapted/waived/unsupported)
- Links waivers to their source wave traceability

**Inconsistent waiver terminology**: Some files use `waived`, some use `unsupported`. The spec defines states: `done`, `intentional-diff`, `unsupported`, `host-limited`, `open`. Current files use `adopted`, `adapted`, `waived`, `unsupported` which may need mapping to spec taxonomy.

---

## 3. Traceability Fidelity

### 3.1 What Exists

| Traceability Layer | Status |
|---|---|
| Phase 30 matrix (detailed per-behavior with evidence) | ✅ Comprehensive |
| Wave traceability files (per-wave adopt/adapt/waive) | ✅ Well-maintained |
| CPython harvest inputs documented | ✅ All waves |
| Local regression/demo evidence linked | ✅ All waves |

### 3.2 Traceability Gaps

| Gap | Severity | Notes |
|---|---|---|
| No unified ledger across all waves | 🟡 MEDIUM | Each wave is traceable in isolation but no cross-wave summary |
| Missing revisit rules for some waivers | 🟡 MEDIUM | Phase30 matrix has revisit rules; wave files have varying completeness |
| No host-limited classification | 🟡 LOW | Spec mentions `host-limited` but not used in current files |
| Documentation alignment pending | 🟡 MEDIUM | Spec requires aligning architecture.md, roadmap.md, phase docs to closed state (line 137) - not yet done |

---

## 4. Execution Status Impact

The milestone_psp_7 cannot reach exit closure until:

| Wave | Status | Impact |
|---|---|---|
| wave_psp_a1 | ✅ Done | - |
| wave_psp_a2 | ✅ Done | - |
| wave_psp_b1 | ✅ Done | - |
| wave_psp_b2 | ✅ Done | - |
| wave_psp_c1 | ✅ Done | - |
| wave_psp_c2 | ⚠️ In Progress | Validation evidence present but PR not yet merged |
| wave_psp_d1 | ⏳ Pending | Not started |
| wave_psp_d2 | ⏳ Pending | Not started |
| wave_psp_e1 | ⏳ Pending | Not started |
| wave_psp_e2 | ⏳ Pending | Not started |

**Current completion: 5/10 waves (50%)**

---

## 5. Actionable Findings

### 5.1 HIGH Priority (Required for Exit Closure)

1. **Create canonical builtin parity inventory**
   - Extract builtins from wave_a1 traceability
   - Add status, classification, rationale, owner, tracking_issue, revisit_rule, evidence columns
   - Align with phase30_parity_matrix.md format

2. **Create canonical core object-model inventory**
   - Extract object models from wave_a2 traceability
   - Add same columns as builtin inventory

3. **Create waiver index**
   - Consolidate all waivers from wave_a1 through wave_c2 (and future waves)
   - Map current terminology (adopted/adapted/waived/unsupported) to spec taxonomy (done/intentional-diff/unsupported/host-limited/open)

4. **Create exit-gate closure summary**
   - Document final parity state for all completed modules
   - Provide cross-wave ledger view

### 5.2 MEDIUM Priority (Quality of Governance)

5. **Align waiver terminology**
   - Map `adopted` → `done` (parity)
   - Map `adapted` → `intentional-diff`
   - Map `waived`/`unsupported` → `unsupported` or `host-limited`
   - Ensure consistent usage across all wave traceability files

6. **Update documentation alignment**
   - Verify `internal_docs/architecture.md` reflects current parity state
   - Verify `internal_docs/roadmap.md` phase status is accurate
   - Verify public docs match actual support

### 5.3 LOW Priority (Future-Proofing)

7. **Add host-limited classification where applicable**
   - Some Sifr-specific limitations (e.g., no raw file descriptors) could use this classification

---

## 6. Validation Approach

To validate the inventory correctness:

1. **Check all 45 lib/sifr modules are covered** - cross-reference ls output with inventories
2. **Verify each inventory row has evidence** - each `done` or `intentional-diff` should link to demo/regression
3. **Verify waiver rationale is non-empty** - no "TBD" or missing rationale

To validate waiver governance:

1. **Count total waivers per wave** - compare to explicit waiver sections
2. **Check revisit rules exist** - each non-done entry should have revisit rule
3. **Verify owner assignment** - who is responsible for each open item

To validate traceability:

1. **Verify CPython test inputs are documented** - each wave should list source test files
2. **Verify regression evidence exists** - link to e2e pass/fail files

---

## 7. Review Conclusion

| Area | Current State | Readiness for Exit Closure |
|---|---|---|
| Inventory Correctness | Partial (phase30 matrix + wave files) | 🔴 NOT READY - Missing builtin/object-model dedicated inventories |
| Waiver Governance | Per-wave complete | 🔴 NOT READY - Missing waiver index |
| Traceability Fidelity | Per-wave excellent | 🟡 PARTIAL - No unified cross-wave ledger |

**Recommendation:** Do not proceed to exit closure until:
1. All 10 waves are complete (currently 50% done)
2. Dedicated builtin and object-model inventories are created
3. Waiver index is consolidated
4. Exit-gate closure summary is published
5. Documentation alignment is verified

---

## Appendix: Key File References

- Execution tracking: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md`
- Phase spec: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
- Phase30 matrix: `verification/stdlib/phase30_parity_matrix.md`
- Wave traceability: `verification/stdlib/wave_psp_*_cpython_traceability.md`
- lib/sifr modules: 45 `.sifr` files in `lib/sifr/`
