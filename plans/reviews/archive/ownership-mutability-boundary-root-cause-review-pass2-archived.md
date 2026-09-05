# Review: Ownership/Mutability Boundary Root-Cause Analysis (Pass 2)

Reviewer: agent
Date: 2026-04-02
Report under review: `issues/ownership-mutability-boundary-root-cause-2026-04-02.md`
Prior review: `reviews/ownership-mutability-boundary-root-cause-review-pass1.md`

---

## Pass 1 Required Changes: Verification

### 1. First-diagnostic overlap and compound `own mut` note

**Status: ADDRESSED**

The report now includes a dedicated paragraph under "Current Decomposition" (lines 25-26):

> *"Note on overlap: sub-bucket assignment is based on the first emitted diagnostic. A subset of fixtures (estimated 3-5, including 0075_sort_colors, 0669_trim_a_binary_search_tree, 0701_insert_into_a_binary_search_tree) likely need compound boundary annotations (own mut) spanning both mutability and ownership dimensions."*

This matches the pass1 request exactly: it states the first-diagnostic basis, gives the 3-5 estimate, names the specific fixtures, and references `own mut` as the compound annotation.

### 2. Secondary-defect exposure estimate

**Status: ADDRESSED**

Remediation strategy step 3 (line 123) now includes:

> *"expectation: at least 6 fixtures with node/root-style surfaces may unmask secondary categories (for example, recursive node/field-expression surfaces) after ownership/mutability fixes."*

This sets the quantitative expectation requested in pass1 (at least 6 fixtures, naming the likely target category).

### 3. Concrete `mut` vs local-copy decision rule

**Status: ADDRESSED**

The remediation strategy (lines 120-121) now provides concrete rules:

- Copy-type scalar rebinding (`n`, `k`, `left`, `columnNumber`, `speed`, `s`): prefer `let mut local = param`
- Collection/object in-place edits (`nums`, `intervals`, `tokens`, `matrix`, tree roots): prefer explicit parameter `mut` (or `own mut` when escaping)

This replaces the generic guidance from the original report with the specific policy pass1 requested.

### 4. Concrete `own` vs `.clone()` decision rule

**Status: ADDRESSED**

Root cause B (lines 83-85) now specifies:

- `own`/`own mut`: when caller relinquishes the value (expected default for most LeetCode fixtures)
- `.clone()`: when caller must retain independent access after the call

This resolves the circularity noted in pass1 by anchoring the decision on caller intent rather than escape semantics.

---

## Pass 1 Optional Improvements: Verification

### Parallel adaptation note

**Status: ADDRESSED**

Line 124: *"Escape (6) and mutation/reassignment (42) workstreams are independent and can be adapted in parallel."*

### 0605 variant consistency note

**Status: NOT ADDRESSED (negligible)**

No explicit note that `0605_can_place_flowers` and `0605_can_place_flowers_v2` should receive identical adaptations. This remains negligible -- the fixtures share identical diagnostics and the adaptation is mechanical.

---

## Remaining Gaps

None of material concern. All four required changes from pass1 are fully incorporated. The single unaddressed optional item (0605 variant note) is negligible and does not affect execution readiness.

---

## Final Implementation-Readiness Verdict

**READY FOR EXECUTION.**

The report is factually correct, architecturally sound, and now includes all amendments required by pass1:
- Sub-bucket overlap is documented with specific fixtures and compound annotation guidance
- Secondary-defect exposure is quantified with a concrete estimate
- `mut` vs local-copy policy is concrete and parameter-type-specific
- `own` vs `.clone()` decision is anchored on caller intent
- Parallel workstream independence is explicit

No further amendments are needed. The report can be used as-is to drive batch adaptation of the 48 `ownership_and_mutability_boundary` fixtures.
