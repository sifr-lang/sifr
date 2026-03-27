# Phase 31 Strategy Review - Pass 1

**Review Date:** 2026-03-14
**Source:** `verification/leetcode/phase31_current_full_results_after_m31a_wave5_rerun.json`
**Total Failing:** 35 cases out of 50

## Executive Summary

The current milestone structure in `phase31-ad-hoc-followup-milestones.md` is **mostly sound** but has **gaps and misclassifications** that need correction. This review validates each of the 35 failing cases against the actual error messages and identifies where:

1. Some milestones are too narrow (missing cases that belong to them)
2. Some milestones are too broad or incorrectly scoped
3. A new ad hoc phase may be needed for cases not covered by existing milestones

## Classification of All 35 Failing Cases

### Category 1: Needs Broader Ad Hoc Prerequisite (5 cases)

These require either `prereq_recursive_types` or `prereq_own_mut` before Phase 31 closure can happen.

| ID | Fixture | Root Cause | Current Milestone | Correct? |
|----|---------|------------|-------------------|----------|
| 0100 | same_tree | `.val` attribute access on recursive TreeNode | m31_e_recursive_tree_surface | ✅ Yes - depends on recursive_types |
| 0102 | binary_tree_level_order_traversal | `.left`/`.right` attribute access on recursive TreeNode | m31_e_recursive_tree_surface | ✅ Yes - depends on recursive_types |
| 0110 | balanced_binary_tree | `.left`/`.right` + Any indexing on recursive nodes | m31_e_recursive_tree_surface | ✅ Yes - depends on recursive_types |
| 0226 | invert_binary_tree | `.left`/`.right` attribute access on recursive TreeNode | m31_e_recursive_tree_surface | ✅ Yes - depends on recursive_types |
| 0235 | lowest_common_ancestor_of_a_binary_search_tree | `.val` attribute + missing self annotation | m31_e_recursive_tree_surface | ✅ Yes - depends on recursive_types |
| 1299 | replace_elements_with_greatest_element_on_right_side | ownership error - borrowed parameter | m31_j_own_mut_leetcode_closure | ✅ Yes - depends on prereq_own_mut |

**Note:** Cases 0100, 0102, 0110, 0226, 0235 all need `prereq_recursive_types`. Case 1299 needs `prereq_own_mut`. These correctly map to their respective prerequisites and dependent milestones.

---

### Category 2: Normal Phase 31 Milestone Work (28 cases)

These should be solved through existing Phase 31 milestones without additional prerequisites.

#### Container Literal Specialization (m31_g) - 6 cases

| ID | Fixture | Error | Analysis |
|----|---------|-------|----------|
| 0001 | two_sum | `dict[Any, Any]` indexing with int | ✅ Correctly assigned to m31_g |
| 0242 | valid_anagram | `int + Any` arithmetic | ✅ Correctly assigned to m31_g |
| 0424 | longest_repeating_character_replacement | `dict[Any, Any]` indexing | ✅ Correctly assigned to m31_g |
| 0523 | continuous_subarray_sum | `dict[Any, Any]` indexing | ✅ Correctly assigned to m31_g |
| 0560 | subarray_sum_equals_k | `dict[Any, Any]` indexing + Any arithmetic | ✅ Correctly assigned to m31_g |

**Issue:** The milestone says affected ids are `0001`, `0242`, `0424`, `0523`, `0560` - but only lists 5. There are 5 here that match. This looks correct.

#### Optional Flow Completion (m31_a) - 8 cases

| ID | Fixture | Error | Analysis |
|----|---------|-------|----------|
| 0053 | maximum_subarray | `int \| None` vs `int` | ✅ Correctly assigned to m31_a |
| 0127 | word_ladder | `None \| str` slicing | ✅ Correctly assigned to m31_a |
| 0238 | product_of_array_except_self | `int \| None` multiplication | ✅ Correctly assigned to m31_a |
| 0322 | coin_change | `int \| None` return | ✅ Correctly assigned to m31_a |
| 0502 | ipo | None unpacking + Any comparison | ⚠️ Partial - also has heapq/Any issues |
| 0743 | network_delay_time | None tuple unpacking | ✅ Correctly assigned to m31_a |
| 0746 | min_cost_climbing_stairs | `int \| None` return | ✅ Correctly assigned to m31_a |

**Issue:** The milestone says affected ids are `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746` - that's 7 cases. We have 7 matching here. But wait - 0502 also appears in the list, so that's actually correct.

However, note that 0502 also has heapq-related issues. It should remain in m31_a but may need coordination with m31_c stdlib work.

#### Nested Function Pipeline (m31_d) - 9 cases

| ID | Fixture | Error | Analysis |
|----|---------|-------|----------|
| 0017 | letter_combinations_of_a_phone_number | dict indexing with Any, iterate over Any | ✅ Correctly assigned to m31_d |
| 0039 | combination_sum | Any >= int, list[Any] indexing | ✅ Correctly assigned to m31_d |
| 0050 | powx_n | missing type annotations on nested helper | ✅ Correctly assigned to m31_d |
| 0052 | n_queens_ii | Any return + missing annotations | ✅ Correctly assigned to m31_d |
| 0078 | subsets | Any >= int, list indexing, missing annotations | ✅ Correctly assigned to m31_d |
| 0090 | subsets_ii | list[Any] indexing, slicing Any | ✅ Correctly assigned to m31_d |
| 0207 | course_schedule | not Any, dict indexing with Any | ✅ Correctly assigned to m31_d |
| 0684 | redundant_connection | Any > Any, list indexing with Any | ✅ Correctly assigned to m31_d |
| 0912 | sort_an_array | missing annotations on merge helper | ✅ Correctly assigned to m31_d |

**Issue:** The milestone lists 9 cases: `0017`, `0039`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912`. That's exactly 9 and matches perfectly. Good.

#### Destructuring (m31_b) - 4 cases

| ID | Fixture | Error | Analysis |
|----|---------|-------|----------|
| 0295 | find_median_from_data_stream | tuple unpacking, missing fields | ✅ Correctly assigned to m31_b |
| 0703 | kth_largest_element_in_a_stream | tuple unpacking, missing fields | ✅ Correctly assigned to m31_b |
| 0997 | find_the_town_judge | for loop tuple target | ✅ Correctly assigned to m31_b |
| 1209 | remove_all_adjacent_duplicates_in_string_ii | augmented subscript target, None indexing | ⚠️ Partial - also has None indexing |

**Issue:** The milestone says affected ids are `0295`, `0703`, `0997`, `1209` - that's 4 cases and matches. However, 1209 also has `Any | None` indexing which relates to optional flow. This is a case where the milestone boundaries overlap slightly, but the primary failure is destructuring.

#### Local Name Binding (m31_h) - 1 case

| ID | Fixture | Error | Analysis |
|----|---------|-----|----------|
| 0015 | 3sum | `function > int`, `int + None` | ✅ Correctly assigned to m31_h |

This is correctly scoped. The issue is local shadowing causing the wrong type to be resolved.

#### Multi-Solution Fixture Canonicalization (m31_i) - 1 case

| ID | Fixture | Error | Analysis |
|----|---------|-------|----------|
| 0215 | kth_largest_element_in_an_array | None return, undefined heapify | ⚠️ NOT a multi-solution issue |

**CRITICAL ISSUE:** 0215 is listed under `m31_i_corpus_fixture_canonicalization_for_multi_solution_files` with ids `0215`, `1046`. But looking at the actual errors:
- 0215: `undefined function: 'heapify'` - this is a stdlib issue, not multi-solution
- 1046: `abs() argument must be numeric, got 'Any'` - this is typing + stdlib

These are **NOT multi-solution problems**. They should be:
- 0215: m31_c (stdlib) + m31_a (optional flow)
- 1046: m31_c (stdlib) + m31_g (container literal)

**This is a misclassification.** The milestone `m31_i` is incorrectly scoped.

#### Stdlib Module Parity (m31_c) - Remaining issues

| ID | Fixture | Error | Analysis |
|----|---------|-----|----------|
| 0215 | kth_largest_element_in_an_array | undefined heapify | Should be here, not m31_i |
| 1046 | last_stone_weight | abs(Any), heapq_max Any | Should be here, not m31_i |

The milestone `m31_c_stdlib_module_parity` is marked as **complete** in the document. However, 0215 and 1046 are still failing with stdlib-related errors. This suggests either:

1. The milestone was closed prematurely (before these deeper issues were surfaced), OR
2. These cases have issues beyond stdlib that kept them from passing even after the stdlib fix

Looking at the errors more carefully:
- 0215: `undefined function: 'heapify'` - this is the stdlib issue
- 1046: `abs() argument must be numeric, got 'Any'` - this is typing issue

The m31_c milestone closed with the statement: "all remaining watched-case failures are now downstream codegen/type-system work rather than `stdlib.python_module_surface`"

This is technically true - these are now **type-system** failures (Any leakage), not stdlib surface failures. The classification is correct but the milestone naming is confusing. These should be in m31_g (container literal) for the typing aspects.

---

### Category 3: Canonical Sifr Fixture Rewrite (1 case)

| ID | Fixture | Error | Current Milestone | Analysis |
|----|---------|-------|-------------------|----------|
| 0043 | multiply_strings | undefined variable 'digit', Result type arithmetic | m31_k_canonical_sifr_fixture_normalization | ✅ Correct |

This is correctly identified as a raw-source divergence requiring canonical Sifr rewrite due to intentional Sifr parse-safety guarantees.

---

### Category 4: Both Prerequisite AND Closure Step

No cases strictly require **both** a prerequisite AND a final Phase 31 closure step beyond what's already captured in Category 1. The tree cases (0100, 0102, 0110, 0226, 0235) will need:
1. `prereq_recursive_types` to land first
2. Then `m31_e_recursive_tree_surface_leetcode_closure` to close

This is already captured in the execution order.

---

## Issues Found

### 1. Misclassification: m31_i targets wrong cases

**Location:** `phase31-ad-hoc-followup-milestones.md:282-292`

**Problem:** The milestone `m31_i_corpus_fixture_canonicalization_for_multi_solution_files` lists `0215` and `1046` as affected ids, claiming they have "multiple alternative top-level solutions."

**Evidence:** The actual errors are:
- 0215: `undefined function: 'heapify'`, `return type mismatch: expected 'int', got 'int | None'`
- 1046: `abs() argument must be numeric, got 'Any'`

Neither of these errors is about multiple solutions. Both are about:
- 0215: stdlib `heapq.heapify` availability + optional flow (None return)
- 1046: container literal typing (Any leakage through heapq helpers)

**Recommendation:** Remove 0215 and 1046 from m31_i. They belong in:
- 0215: m31_a (optional flow) - the primary failure is `int | None` return
- 1046: m31_g (container literal) - the primary failure is Any typing

### 2. Gap: No milestone covers cases where stdlib + type-system intersect

**Problem:** Cases like 0215 and 1046 have compound failures:
- They need stdlib features (heapq) - supposedly "fixed" in m31_c
- They still fail due to Any leakage from container literals - supposedly in m31_g

But neither milestone explicitly owns the intersection.

**Recommendation:** This is actually fine - the failures will naturally fall into m31_g once the stdlib surface is fixed. No new milestone needed.

### 3. Ambiguity: m31_c closure may be too early

**Problem:** The document says m31_c is "complete" but cases with stdlib dependencies (0215, 1046) are still failing. The justification is "all remaining watched-case failures are now downstream codegen/type-system work."

This is technically correct but creates confusion about whether "complete" means "stdlib surface is done" or "all stdlib-related failures are resolved."

**Recommendation:** Clarify the definition of "complete" for m31_c. If it means "stdlib surface parity achieved" (even if downstream typing blocks remain), that's valid but should be explicitly stated.

### 4. Missing Case: 0017 appears in both m31_a and m31_d

Wait, let me re-check. 0017 has these errors:
- `cannot index type 'dict[str, str]' with 'Any'`
- `cannot index type 'str' with 'Any'`
- `cannot iterate over type 'Any'`
- `len() argument must be a string`

This is primarily a nested function issue (m31_d) where nested function inference produces Any. It doesn't appear in the m31_a list, so this is correct.

### 5. All Category 1 Cases Correctly Mapped

The 6 cases requiring prerequisites are correctly mapped:
- 5 tree cases → prereq_recursive_types → m31_e
- 1 ownership case → prereq_own_mut → m31_j

---

## Summary of Classifications

| Category | Count | Breakdown |
|----------|-------|-----------|
| 1. Needs broader ad hoc prerequisite | 6 | 5 tree (recursive_types) + 1 ownership (own_mut) |
| 2. Normal Phase 31 milestone work | 28 | m31_g(5) + m31_a(7) + m31_d(9) + m31_b(4) + m31_h(1) + m31_c(2)* |
| 3. Canonical Sifr rewrite | 1 | 0043 |
| 4. Both prerequisite + closure | 0 | Already covered by Cat 1 |

*m31_c cases (0215, 1046) are technically in m31_a and m31_g respectively per their primary errors

---

## Recommendations

### Must Fix

1. **Remove 0215 and 1046 from m31_i** - They are not multi-solution fixture issues
   - Move 0215 to m31_a_optional_flow_completion (it has `int | None` return)
   - Move 1046 to m31_g_container_literal (it has Any typing from container literals)

2. **Clarify m31_c completion criteria** - State explicitly that "complete" means stdlib surface parity, not complete LeetCode closure for stdlib-adjacent cases

### Confirm Current Structure Is Sound

3. **m31_g** (container literal) - Correctly scoped to 5 cases
4. **m31_a** (optional flow) - Correctly scoped to 7 cases (add 0215 per #1)
5. **m31_d** (nested function) - Correctly scoped to 9 cases
6. **m31_b** (destructuring) - Correctly scoped to 4 cases
7. **m31_h** (local binding) - Correctly scoped to 1 case
8. **m31_e** (tree closure) - Correctly depends on prereq_recursive_types
9. **m31_j** (own_mut closure) - Correctly depends on prereq_own_mut
10. **m31_k** (canonical rewrite) - Correctly scoped to 0043

### Execution Order Validation

The recommended execution order is:
1. prereq_recursive_types
2. prereq_own_mut
3. m31_g_container_literal
4. m31_a_optional_flow_completion
5. m31_b_destructuring
6. m31_d_nested_function
7. m31_e_recursive_tree
8. m31_h_local_name_binding
9. m31_j_own_mut
10. m31_k_canonical_sifr
11. m31_i (after fixing scope)

**This order is sound** - prerequisites first, then compiler improvements, then closure milestones.

---

## Conclusion

The Phase 31 milestone structure is **fundamentally sound** with one critical fix needed:

- **m31_i_corpus_fixture_canonicalization_for_multi_solution_files** incorrectly targets 0215 and 1046. These should be removed and reassigned to m31_a and m31_g respectively.

All other milestones are correctly scoped to their actual failing cases. The prerequisite dependencies (recursive_types, own_mut) are correctly identified and sequenced.
