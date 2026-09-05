# Review: Ownership/Mutability Boundary Root-Cause Analysis (Pass 1)

Reviewer: agent
Date: 2026-04-02
Report under review: `issues/ownership-mutability-boundary-root-cause-2026-04-02.md`

---

## 1. Factual Correctness of Category Counts and Sub-Buckets

**Verdict: CORRECT**

All counts verified against both source artifacts:

| Sub-bucket | Report count | Taxonomy count | Breakdown count | Match |
|---|---|---|---|---|
| `immutable_parameter_mutation` | 31 | 31 | 31 (fixtures enumerated) | YES |
| `immutable_parameter_reassignment` | 11 | 11 | 11 (fixtures enumerated) | YES |
| `borrowed_parameter_escape_store` | 4 | 4 | 4 (fixtures enumerated) | YES |
| `borrowed_parameter_escape_return` | 2 | 2 | 2 (fixtures enumerated) | YES |
| **Total** | **48** | **48** | **48** | **YES** |

The `category_counts` field in `full_corpus_failure_taxonomy_20260402_live.json` confirms `ownership_and_mutability_boundary: 48`. Every fixture slug in the breakdown JSON maps 1:1 to a taxonomy entry with the same category and a diagnostic whose shape matches the declared sub-bucket.

High-frequency parameter names spot-checked:

- `nums` (11): confirmed -- 0016, 0046, 0075, 0179, 0312, 1498, 1838, 1968, 1984, 2616, 2971.
- `node` (4): confirmed -- 0002, 0141, 0160, 0234.
- `matrix` (2): confirmed -- 0048, 0073.
- `flowerbed` (2): confirmed -- 0605, 0605_v2.
- `root` (2): confirmed -- 0669, 0701.
- `grid` (2): confirmed -- 1020, 1254.
- `n` (2): confirmed -- 0191, 0263.
- `nums1` (2): confirmed -- 0088, 2215.
- `s` (2): confirmed -- 0006, 1888.
- `intervals` (2): confirmed -- 0252, 0435.

No count errors found.

---

## 2. Compiler-vs-Fixture Adaptation Split Under Sifr Principles

**Verdict: CORRECT, with one clarification needed**

The report's central claim -- that all 48 failures are fixture/source adaptation, not compiler defects -- is sound. The diagnostics are all correctly shaped: the compiler *is* enforcing the intended ownership/mutability rules, and the source code simply doesn't declare the required contracts.

**Clarification needed:** The report groups root causes into two buckets (`root_cause_a` = 42, `root_cause_b` = 6) but does not address the *overlap case* where a fixture needs both mutation **and** escape. Specifically:

- `0075_sort_colors`: classified under `borrowed_parameter_escape_return` (diagnostic: "cannot return borrowed parameter `nums`"). However, `sort_colors` is canonically an in-place sort -- the adapted fixture will likely need `own mut` (both ownership for return *and* mutability for the in-place swap). The report's compiler-work item #1 alludes to this (`own mut` suggestion) but does not quantify how many of the 6 escape fixtures may additionally require mutation.
- Similarly, `0669_trim_a_binary_search_tree` and `0701_insert_into_a_binary_search_tree` are classified as `immutable_parameter_mutation`, but tree-structure operations that mutate `root` often also return a (potentially different) root. After adding `mut`, these may surface a secondary escape error.

**Recommendation:** Add a note that the sub-bucket boundaries are based on the *first diagnostic emitted*, and that an estimated 3-5 fixtures may need compound annotations (`own mut`) that span both root causes. This does not change the adaptation-vs-compiler split, but affects remediation effort estimates.

---

## 3. Architectural Consistency with Explicit Mutability/Ownership Rules

**Verdict: CORRECT**

The report cites three architectural anchors:

1. Explicit `mut`/`own mut` for parameter reassignment/mutation (architecture.md:149)
2. Borrow-by-default parameter model with ownership and mutability as explicit axes (architecture.md:308-324)
3. Borrowed move-type params cannot escape unless explicitly owned or cloned (architecture.md:338)

These are the correct governing rules for every diagnostic shape observed in this bucket. The diagnostic messages themselves (`add mut`, `add own`, `store node.clone()`) are direct consequences of these rules.

The "do not loosen language semantics" decision is architecturally consistent: these are boundary-contract rules, not accidental restrictions. The LeetCode corpus is explicitly Python-origin code that treats mutability/aliasing as implicit -- exactly the mismatch Sifr's explicit model is designed to surface.

No architectural inconsistencies found.

---

## 4. Missing Root-Cause Dimensions or Remediation Gaps

### 4a. Cascading secondary errors after adaptation (MISSING)

The report mentions "suppress secondary noise from the same root cause" as a compiler quality improvement, but does not address the *reverse*: after a fixture is adapted (e.g., `mut` is added), **new errors in different categories may be unmasked**.

For example, adding `mut` to a tree-node parameter may surface a `recursive_node_and_field_expression_surface` error (field access syntax) that was previously suppressed by the earlier ownership error. The remediation plan's step 3 ("reclassify any residuals as true secondary defects") handles this in principle, but the report should explicitly warn that **some of the 48 fixtures may migrate to other failure categories rather than passing outright**.

**Recommendation:** Add a subsection under "Execution-Ready Remediation Strategy" estimating secondary-defect exposure. Based on parameter overlap with the `recursive_node_and_field_expression_surface` category (at least `node`-bearing fixtures 0002, 0141, 0160, 0234, plus tree-root fixtures 0669, 0701), at least 6 fixtures are likely to surface secondary errors in other categories.

### 4b. `mut` vs local-copy decision framework (INCOMPLETE)

The report mentions "prefer local copy for scalar loop counters when that preserves cleaner public signatures" but does not provide a concrete decision rule. For the 11 `immutable_parameter_reassignment` fixtures, the choice between:

- Adding `mut` to the parameter, or
- Introducing `let mut local = param` at function entry

...depends on whether the caller's contract should reflect mutability. The report should specify the default policy (e.g., "prefer local copy for copy-type scalars; prefer `mut` annotation for collections that are mutated in-place").

### 4c. `own` vs `.clone()` decision framework (INCOMPLETE)

For the 6 escape fixtures, the report says "choose `own` vs `.clone()` based on whether value must escape" but this is circular -- both `own` and `.clone()` enable escape. The actual decision axis is **caller intent**: does the caller expect to retain access after the call? If yes, `.clone()` inside the callee; if no, `own` at the boundary. The report should clarify this.

### 4d. Batch-adaptation ordering risk (MINOR)

The remediation strategy processes mutation/reassignment (42) first, then escape (6). This is correct for efficiency but the report should note that the 6 escape fixtures are *independent* of the 42 and can be adapted in parallel -- there is no ordering dependency between sub-buckets.

---

## 5. Risks to Zero-Failing LeetCode Objective

### 5a. Net fixture count after adaptation (LOW RISK)

All 48 are adaptation-only. None require language changes. After adaptation, these fixtures should pass or migrate to a different (already-tracked) category. This bucket contributes no systemic risk to the zero-failing objective.

### 5b. Secondary defect migration (MEDIUM RISK)

As noted in 4a, an estimated 6+ fixtures may unmask errors in other categories after ownership/mutability adaptation. These are not new bugs -- they are pre-existing errors masked by the ownership error being emitted first. But they will inflate apparent counts in other categories temporarily. The remediation plan accounts for this but should set expectations.

### 5c. Diagnostic quality (LOW RISK)

The three compiler-quality improvements listed (compound `own mut` suggestion, local-copy guidance, cascade suppression) are ergonomic, not correctness-blocking. They should be prioritized for developer experience but are not prerequisites for the adaptation work.

### 5d. Fixture variant duplication (NEGLIGIBLE)

The bucket includes one fixture pair: `0605_can_place_flowers` and `0605_can_place_flowers_v2`, both with identical diagnostics. This is expected (variant fixtures) and not a classification error, but the adaptation should handle both consistently.

---

## Incorrect Claims

None found. All counts, sub-bucket assignments, diagnostic shapes, and architectural citations are factually accurate against the source data.

---

## Implementation-Readiness Verdict

**READY, with minor amendments required before execution.**

The report is factually sound, architecturally consistent, and the compiler-vs-adaptation split is correct. The following amendments should be made before using this report to drive batch adaptation:

### Required Changes

1. **Add compound-annotation note (Section: Current Decomposition or Root Cause):** State that sub-bucket membership is based on the first diagnostic emitted and that an estimated 3-5 fixtures (especially `0075_sort_colors`, `0669_trim_a_binary_search_tree`, `0701_insert_into_a_binary_search_tree`) may require compound `own mut` annotations spanning both root-cause dimensions.

2. **Add secondary-defect exposure estimate (Section: Execution-Ready Remediation Strategy):** Warn that at least 6 fixtures with `node`/`root` parameters are likely to surface errors in `recursive_node_and_field_expression_surface` or other categories after ownership/mutability adaptation. Step 3 handles reclassification but should set quantitative expectations.

3. **Sharpen the `mut` vs local-copy decision rule (Section: Remediation Strategy):** Replace the generic guidance with: "For copy-type scalars being rebound (`n`, `k`, `left`, `columnNumber`, `speed`, `s`), prefer local-copy pattern. For collections being sorted/mutated in-place (`nums`, `intervals`, `tokens`, etc.), prefer `mut` annotation."

4. **Sharpen the `own` vs `.clone()` decision rule (Section: Root Cause B):** Clarify: "`own` when the caller does not need the value after the call (typical for LeetCode); `.clone()` when the caller retains a reference (rare in this corpus -- likely 0 of 6)."

### Optional Improvements

- Note that escape sub-bucket (6) can be adapted in parallel with mutation sub-bucket (42) since there is no ordering dependency.
- Confirm that `0605_can_place_flowers` / `0605_can_place_flowers_v2` should receive identical adaptations.
