# PR #1106 Review: Cleanup of Internal Docs, Audits, and Duplicate Demos

**Reviewer:** agent
**Date:** 2026-03-11
**PR Title:** Clean up internal docs, audits, and duplicate demos

---

## Summary

PR #1106 performs a significant cleanup of the repository by:
1. Moving `.cursor/plans/main` to `internal_docs/`
2. Renaming `audit/` to `audits/`
3. Moving non-main `.cursor/plans` material into `issues/`
4. Removing superseded demo files with documentation

---

## Issues Found

### 1. Broken References to Removed Demo Files (HIGH PRIORITY)

The following files still reference the removed demo files but were **not updated** in the PR:

| File | Broken Reference(s) |
|------|---------------------|
| `reviews/ad-hoc-owned-crate-clippy-cleanup-production-grade-review.md` | `demos/m1_env_demo.sifr` |
| `reviews/phase-30-part-3-base64-review-2.md` | `demos/m3_base64_demo.sifr` |

**Recommendation:** Update these references to point to the replacement demos as documented in `issues/demo_cleanup_reasoning_report.md`:
- `demos/m1_env_demo.sifr` → `demos/m30_1a_env_parity_demo/main.sifr`
- `demos/m3_base64_demo.sifr` → `demos/m30_1a_base64_parity_demo/main.sifr`

---

### 2. Incomplete Path Reference Updates (MEDIUM PRIORITY)

**119 occurrences across 49 files** still contain references to `.cursor/plans/main` that were **not updated** to `internal_docs/`. These are almost entirely **historical review files** in `reviews/`.

**Affected categories:**
- Review files: ~47 files with ~117 occurrences
- `internal_docs/phases/15_baseline_reconciliation.md`: 1 occurrence (reference to deferred planning draft)
- `internal_docs/phases/30_reliability_parity_and_performance_budgets.md`: 1 occurrence (reference to deferred planning draft)

**Analysis:**
- The references in `internal_docs/phases/*` files are to **deferred planning drafts** that still exist in `.cursor/plans/` (e.g., Phase 36/37 planning). These are not part of the moved content and are intentionally left as-is.
- The review file references are historical artifacts that document past review sessions. While technically "broken" (pointing to non-existent paths), they serve as historical records and could be left as-is with a note, OR updated to point to the new locations.

**Recommendation:**
- For `internal_docs/phases/*` files: Leave as-is (these reference active planning drafts)
- For review files: Either:
  - (A) Leave as historical records (acceptable)
  - (B) Update to new paths (more work but cleaner)

---

### 3. Missing Verification File

The file `verification/phase31_leetcode_corpus_policy.md` appears to have been removed in the PR but was not accounted for in the summary.

**Status:** Need verification - check if this was intentional or needs restoration.

---

## Correctly Implemented

### ✅ Audit Folder Rename
The rename from `audit/` to `audits/` was done cleanly with `git mv` semantics (0 bytes changed for all files). All internal references within the audits folder and in modified issue files were properly updated.

### ✅ Internal Docs Move
- `.cursor/plans/main/` moved to `internal_docs/` with correct path updates
- `compiler_pipeline.html` moved and README link updated correctly
- All scripts that referenced the old paths were updated:
  - `scripts/validate_phase15_backlog.py`
  - `scripts/phase_contract_gate_check.py`
  - `scripts/check_hir_maintainability_guardrails.py`
  - `scripts/check_sifr_driver_maintainability_guardrails.py`

### ✅ Demo Cleanup Documentation
The `issues/demo_cleanup_reasoning_report.md` was added with clear documentation of:
- What demos were removed
- What their replacements are
- Why each was removed
- Duplicate fixtures found but not removed

### ✅ Removed Demos Have Valid Replacements
Verified that all replacement demos exist:
- `demos/m30_1a_env_parity_demo/`
- `demos/m30_1a_bytes_parity_demo/`
- `demos/m30_1a_base64_parity_demo/`
- `demos/m30_1b_math_parity_demo/`
- `demos/m30_1a_hashlib_parity_demo/`
- `demos/milestone_codegen_quality_v2_demo.sifr`
- `demos/milestone_narrowing_v3_demo.sifr`

### ✅ AGENTS.md and README.md Updates
Both files were correctly updated to reference `internal_docs/` instead of `.cursor/plans/main/`.

---

## Recommendations

### Must Fix Before Merge

1. **Update broken demo references** in:
   - `reviews/ad-hoc-owned-crate-clippy-cleanup-production-grade-review.md`
   - `reviews/phase-30-part-3-base64-review-2.md`

### Optional / Discussion

2. **Review file references**: Decide whether to update the ~47 review files with `.cursor/plans/main` references or leave them as historical records. If leaving as-is, consider adding a note in the repo README about historical reference handling.

3. **Verify removal of `verification/phase31_leetcode_corpus_policy.md`**: Confirm this was intentional.

---

## Validation Commands Run

```bash
# Verify internal_docs exists
ls internal_docs/

# Verify audits folder exists
ls audits/

# Verify removed demos don't exist
ls demos/m1_env_demo.sifr  # Should not exist in PR

# Verify replacement demos exist
ls demos/m30_1a_env_parity_demo/
```

---

## Conclusion

The PR is **mostly well-executed** with clean moves and proper documentation. The main issues are:
1. Two review files with broken demo references that need fixing
2. A large number of historical review files with outdated path references (can be left as-is or updated)

Once the broken demo references are fixed, the PR should be good to merge.
