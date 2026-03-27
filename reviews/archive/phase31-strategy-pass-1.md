# Phase 31 Strategy Synthesis Review — Pass 1

Reviewer: Claude Opus 4.6
Date: 2026-03-26
Input: `issues/phase31-strategy-synthesis-review.md` (dated 2026-03-24)
Evidence: `verification/leetcode/phase31_current_full_results_20260321.json`

## Overall Assessment

The synthesis is structurally sound. The bucket taxonomy (canonical fixture adaptation, container specialization, optional-flow, destructuring, nested follow-ons, iterator/comparable residuals) accurately reflects the real failure families in the results JSON. The decision to treat `own mut` and nested functions as consumed prerequisites is correct and well-evidenced.

However, there are **four material errors** and **three minor classification gaps** that should be corrected before the synthesis is used as an execution plan.

---

## Material Errors

### 1. Recursive type prerequisite is DONE, not pending

**Current synthesis claim**: "the only clearly remaining broad prerequisite for the Phase 31 seed corpus is the recursive-type phase" (line 52); section "Broad Prerequisite Still Relevant" treats `prereq_recursive_types` as future work.

**Evidence**: `issues/ad-hoc-full-recursive-type-feature.md` shows all 6 parts completed and merged by 2026-03-13 (PRs #1122–#1127). The results JSON is dated 2026-03-21 — 8 days after the phase merged. Cases 0100, 0102, 0235 still fail.

**Correct classification**: The recursive type prerequisite has already landed. The remaining tree-case failures (0100, 0102, 0235) are now **`m31_e` closure work**, not blocked on a missing prerequisite. The synthesis should move `prereq_recursive_types` from "Broad Prerequisite Still Relevant" to "Phases Already Consumed" alongside `own mut` and nested functions, and reclassify 0100/0102/0235 as normal closure bugs that need investigation under `m31_e` to determine what residual gaps the recursive type phase left behind.

**Impact**: This changes the recommended execution order. `m31_e` no longer needs to wait for a prerequisite — it can start immediately. The remaining attribute-access and type-resolution errors on tree cases represent genuine gaps in the recursive-type implementation that the closure milestone must diagnose and fix (or send back to the recursive-type phase with concrete gap reports, as the follow-up milestones doc already anticipates).

### 2. Case-by-case table contradicts milestone assignments for 0110 and 0226

**0110**: The case-by-case table says `normal closure | bool/local-state follow-on`. But 0110 (`balanced_binary_tree`) uses `TreeNode` recursive types, a nested `dfs` helper, and tuple destructuring. It is correctly listed in `m31_e` affected IDs (line 255). The table row should say `prerequisite closure + normal closure | recursive types (primary) + nested/destructuring follow-on`.

**0226**: The case-by-case table says `canonical fixture adaptation + closure | ownership + destructuring follow-on`. But 0226 (`invert_binary_tree`) is a `TreeNode` recursive-type case. Its `cannot return borrowed parameter 'root'` error is an ownership issue ON a recursive type, and it also has tuple-swap destructuring. It is correctly listed in `m31_e` affected IDs. The table row should say `prerequisite closure + canonical adaptation | recursive types (primary) + ownership + destructuring`.

**Impact**: Without this fix, someone reading only the case-by-case table would not know these are tree cases and might try to fix them without first confirming the recursive-type surface works end-to-end.

### 3. Pass count regression not analyzed

**Current synthesis claim**: "PASS=13, CHECK_ERROR=36, RUN_ERROR=1" from the 2026-03-21 rerun.

**Evidence**: The follow-up milestones doc execution log records "PASS=15, CHECK_ERROR=35, RUN_ERROR=0" from the m31_a wave 5 rerun on 2026-03-13. Between 2026-03-13 and 2026-03-21, the pass count dropped from 15 to 13 and a new RUN_ERROR appeared (0078).

**Missing analysis**: Two cases regressed from PASS to failure between the two snapshots. The synthesis identifies 0078 as a runtime regression but does not identify or explain the other regression(s). The synthesis should:
- Identify which 2 cases regressed (diff the two pass lists).
- Determine whether the regressions were caused by later phase merges (nested functions landed 2026-03-15, which is between the two snapshots).
- Record whether the regressions are expected side effects of stricter checking or genuine bugs.

### 4. `0424` has a compound failure misclassified as pure container specialization

**Current synthesis**: 0424 is in bucket 3 (container specialization) only.

**Actual errors**:
```
cannot index type 'dict[Any, Any]' with 'str'
undefined variable: 'r'
unsupported operand type(s) for +: 'int' and 'Any'
```

The `undefined variable: 'r'` is a **name-binding issue** — `r` is likely used as a loop variable or sliding-window pointer. This is NOT a container-specialization failure; it belongs in `m31_h_local_name_binding_and_shadowing` or needs its own investigation.

**Correction**: 0424 should be classified as `container specialization + name binding` and listed in both `m31_g` and `m31_h` (or a new name-binding investigation item). The container-specialization fix alone will not make 0424 pass.

---

## Minor Classification Gaps

### A. Float-int comparison is a shared root cause across 0050 and 0295

Both cases fail with `cannot compare 'float' and 'int' with ==`. The synthesis classifies 0050 as "numeric/typing cleanup" and 0295 as "destructuring/class-surface follow-on" without noting they share a common type-system restriction.

**Recommendation**: Add a cross-cutting note that float-int comparison is a type-system gap affecting at least two cases. It is small enough to fix once rather than treating each case as independent cleanup.

### B. `0052` label is misleading

The synthesis says "residual nested unsupported subshape" under bucket 5 ("residual nested-function follow-on bugs"). But 0052's error is `recursive nested function 'backtrack' cannot mutate captured state with nonlocal yet` — this is an **explicitly documented unsupported shape boundary** from the nested function phase, not a bug. The distinction matters for planning: this case needs the nested function phase to expand its supported boundary, not a bug fix.

**Recommendation**: Relabel 0052 as "nested-function phase unsupported boundary" rather than "residual nested unsupported subshape". In planning, this case should be tracked as a nested-function phase scope expansion request, not a follow-on bug.

### C. `1209` has a dual failure that spans two milestones

Errors:
```
augmented subscript assignment target must be a simple name
cannot index type 'Any | None' with 'int'
```

The synthesis classifies this under bucket 6 (destructuring/class-surface) and assigns it to `m31_b`. The `augmented subscript assignment` error is indeed `m31_b` scope, but the `Any | None` indexing error overlaps with container specialization / type inference. After `m31_b` fixes the composite lvalue issue, a container-specialization residual may remain.

**Recommendation**: Note the dual ownership in the case-by-case table.

---

## Cases Confirmed Correctly Classified

The following classifications are correct and well-supported by the evidence:

| Bucket | Cases | Assessment |
| --- | --- | --- |
| Canonical `mut` adaptation | 0007, 0009, 0015 (partial), 0043 (partial), 0090, 0127 (partial), 0151, 0215 (partial), 0746 (partial), 0912, 1299 | Correct. These fixtures need explicit `mut`/`own mut` rewrites. |
| Container specialization | 0001, 0242, 0523, 0560 | Correct. All show `dict[Any, Any]` from empty-literal inference gaps. |
| Optional-flow | 0053, 0238, 0322 | Correct. All show `int | None` arithmetic/return failures. |
| Destructuring | 0295 (partial), 0703, 0743, 0997 | Correct. Tuple unpacking and class-field resolution. |
| Multi-solution canonicalization | 0215, 1046 | Correct. These need fixture normalization before further diagnosis. |
| Nested follow-on | 0017, 0207, 0684 | Correct as post-nested-phase residuals. |
| Recursive tree closure | 0100, 0102, 0235 | Correctly scoped to `m31_e`, but prerequisite status is wrong (see Material Error 1). |

---

## Corrected Case-by-Case Table (changes only)

| ID | Current classification | Corrected classification | Corrected primary owner |
| --- | --- | --- | --- |
| `0100` | prerequisite + closure | **normal closure** (prereq landed) | `m31_e` recursive tree residual |
| `0102` | prerequisite + closure | **normal closure** (prereq landed) | `m31_e` recursive tree residual |
| `0110` | normal closure: bool/local-state | **normal closure: recursive type + nested + destructuring** | `m31_e` (primary) + `m31_b` + `m31_d` |
| `0226` | canonical adaptation + closure: ownership + destructuring | **normal closure: recursive type + ownership + destructuring** | `m31_e` (primary) + `m31_b` |
| `0235` | prerequisite + closure | **normal closure** (prereq landed) | `m31_e` recursive tree residual |
| `0424` | normal closure: container specialization | **normal closure: container specialization + name binding** | `m31_g` + `m31_h` |

---

## Recommended Corrections to the Synthesis

1. Move `prereq_recursive_types` to "Phases Already Consumed". Update the execution order to unblock `m31_e` immediately.
2. Fix the case-by-case table rows for 0110, 0226, 0100, 0102, 0235, 0424.
3. Add a section analyzing the 15→13 pass regression between 2026-03-13 and 2026-03-21.
4. Add a cross-cutting note on float-int comparison affecting 0050 and 0295.
5. Relabel 0052 from "residual bug" to "phase boundary expansion".

No other classifications require correction. The remaining buckets and milestone scoping are accurate.
