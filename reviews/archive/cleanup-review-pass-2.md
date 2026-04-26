# PR #1106 Review: Cleanup of Internal Docs, Audits, and Duplicate Demos (Pass 2)

**Reviewer:** Claude Code
**Date:** 2026-03-11
**PR Title:** Clean up internal docs, audits, and duplicate demos
**Branch:** `codex/cleanup`
**URL:** https://github.com/sifr-lang/sifr/pull/1106

---

## Summary

PR #1106 performs a cleanup of the repository by:
1. Moving `.cursor/plans/main` to `internal_docs/`
2. Renaming `audit/` to `audits/`
3. Moving non-main `.cursor/plans` material into `issues/`
4. Removing superseded demo files with documentation

This review focuses on production-grade quality: paths, docs placement, reference integrity, review artifacts, merge readiness, and regressions.

---

## Issues Found

### 1. Broken References to `audit/` → `audits/` (MEDIUM-HIGH PRIORITY)

**17 occurrences across 5 files** in `internal_docs/phases/` still reference the old `audit/` path instead of the new `audits/` path:

| File | Broken References |
|------|-------------------|
| `internal_docs/phases/04_language_hardening.md` | 7 occurrences (e.g., `audit/type_system/`, `audit/stdlib/`, `audit/borrowing/`) |
| `internal_docs/phases/05_borrow_by_default.md` | 5 occurrences (e.g., `audit/borrowing/`) |
| `internal_docs/phases/06_stdlib_architecture.md` | 3 occurrences (`audit/STDLIB_PARITY_MASTER_REPORT.md`) |
| `internal_docs/phases/09_stdlib_safety_remediation.md` | 1 occurrence (`audit/STDLIB_PARITY_MASTER_REPORT.md`) |
| `internal_docs/phases/30_reliability_parity_and_performance_budgets.md` | 2 occurrences (`audit/stdlib/`) |

**Verification:** All referenced files exist in `audits/`:
- `audits/borrowing/` - exists ✓
- `audits/stdlib/01_math.sifr` - exists ✓
- `audits/STDLIB_PARITY_MASTER_REPORT.md` - exists ✓

**Recommendation:** Update all `audit/` references to `audits/` in these 5 files:
```bash
# Quick fix using replace_all
sed -i '' 's|audit/|audits/|g' internal_docs/phases/04_language_hardening.md
sed -i '' 's|audit/|audits/|g' internal_docs/phases/05_borrow_by_default.md
sed -i '' 's|audit/|audits/|g' internal_docs/phases/06_stdlib_architecture.md
sed -i '' 's|audit/|audits/|g' internal_docs/phases/09_stdlib_safety_remediation.md
sed -i '' 's|audit/|audits/|g' internal_docs/phases/30_reliability_parity_and_performance_budgets.md
```

---

### 2. Historical Review Files with Stale References (LOW PRIORITY - ACCEPTABLE)

**~50 files** in `reviews/` contain references to:
- `.cursor/plans/main` → Should be `internal_docs/`
- `audit/` → Should be `audits/`

**Analysis:** These are historical review artifacts that document past review sessions. They serve as audit trails for decision-making. While technically "broken" (pointing to non-existent paths), they are acceptable to leave as-is because:
1. They are explicitly historical records
2. Updating them adds no functional value
3. It would significantly inflate the diff size

**Recommendation:** Leave as-is. Consider adding a note in the repo documentation that historical review files may contain stale path references.

---

### 3. Demo Cleanup (RESOLVED - Verified)

The demo cleanup from pass-1 review was verified:

| Removed Demo | Replacement | Status |
|--------------|-------------|--------|
| `demos/m1_env_demo.sifr` | `demos/m30_1a_env_parity_demo/main.sifr` | Verified ✓ |
| `demos/m3_base64_demo.sifr` | `demos/m30_1a_base64_parity_demo/main.sifr` | Verified ✓ |
| `demos/m2_bytes_demo.sifr` | `demos/m30_1a_bytes_parity_demo/main.sifr` | Verified ✓ |
| `demos/m4_math_demo.sifr` | `demos/m30_1b_math_parity_demo/main.sifr` | Verified ✓ |
| `demos/m5_hashlib_demo.sifr` | `demos/m30_1a_hashlib_parity_demo/main.sifr` | Verified ✓ |
| `demos/milestone_codegen_quality_demo.*` | `demos/milestone_codegen_quality_v2_demo.sifr` | Verified ✓ |
| `demos/milestone_narrowing_v2_demo.sifr` | `demos/milestone_narrowing_v3_demo.sifr` | Verified ✓ |

The `issues/demo_cleanup_reasoning_report.md` was added and is comprehensive.

---

## Correctly Implemented

### ✅ Directory Structure
- `internal_docs/` - Contains architecture, roadmap, phases, verification
- `audits/` - Contains all audit materials (borrowing, leetcode, stdlib, etc.)
- `issues/` - Contains issue tracking and planning docs
- `verification/` - Retained at root for test infrastructure

### ✅ Path Updates in Active Files
- `AGENTS.md` - References `internal_docs/` ✓
- `README.md` - References `internal_docs/architecture.md` ✓
- `scripts/validate_phase15_backlog.py` - References updated ✓
- `scripts/phase_contract_gate_check.py` - References updated ✓
- `scripts/check_hir_maintainability_guardrails.py` - References updated ✓
- `scripts/check_sifr_driver_maintainability_guardrails.py` - References updated ✓

### ✅ Audit Folder Rename
- All 1181 files moved cleanly via git mv (0-byte diff for most)
- Internal references within audits/ are correct

### ✅ Build and Runtime Verification
```bash
cargo check -p sifr  # PASSED ✓
cargo run -q -p sifr -- run demos/m30_1a_env_parity_demo/main.sifr  # PASSED ✓
python3 scripts/validate_phase15_backlog.py  # PASSED ✓
python3 scripts/phase_contract_gate_check.py --phase 15 --check exit  # PASSED ✓
```

---

## Merge Readiness Assessment

| Criteria | Status | Notes |
|----------|--------|-------|
| Build works | ✅ PASS | cargo check passes |
| Tests pass (basic) | ✅ PASS | Demo runs correctly |
| Path references updated | ⚠️ PARTIAL | 17 broken refs in internal_docs/phases |
| Demo references fixed | ✅ PASS | From pass-1 review |
| Documentation complete | ✅ PASS | demo_cleanup_reasoning_report.md added |
| No regressions | ✅ PASS | All validations pass |

---

## Recommendation

**Condition for merge:** Fix the 17 broken `audit/` → `audits/` references in `internal_docs/phases/` files.

Once those references are updated, the PR will be production-grade ready. The historical review file references are acceptable to leave as-is.

---

## Validation Commands

```bash
# Verify build
cargo check -p sifr

# Verify demo replacement works
cargo run -q -p sifr -- run demos/m30_1a_env_parity_demo/main.sifr

# Verify scripts work
python3 scripts/validate_phase15_backlog.py
python3 scripts/phase_contract_gate_check.py --phase 15 --check exit

# Check for remaining audit/ references in active docs
grep -r "audit/" internal_docs/phases/
```

---

## Conclusion

PR #1106 is **close to merge-ready**. The main remaining issue is 17 broken path references in `internal_docs/phases/` files that need `audit/` → `audits/` updates. Once fixed, the repository will be in a production-grade state for this cleanup phase.
