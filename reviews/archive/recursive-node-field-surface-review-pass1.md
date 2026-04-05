# Review Pass 1: recursive_node_and_field_expression_surface (2026-04-04 Rerun)

- **Reviewer**: Claude Opus 4.6 (automated)
- **Date**: 2026-04-04
- **Scope**: All 34 fixture rows in `recursive_node_field_surface_20260404_inventory.csv`
- **Cross-referenced against**: full diagnostics (`recursive_node_field_34_diagnostics_20260404.txt`), taxonomy JSON (`full_corpus_failure_taxonomy_20260404_live_rerun1.json`), breakdown issue (`recursive-node-field-surface-breakdown-2026-04-04.md`)

---

## 1. Per-Fixture Verdicts

| # | fixture_slug | subcategory | resolution_path | verdict | notes |
|---|---|---|---|---|---|
| 1 | 0019_remove_nth_node_from_end_of_list | field_expression_access_unsupported | both | CORRECT | `.next` blocked + mut/own/optional fixture issues confirmed in diagnostics |
| 2 | 0021_merge_two_sorted_lists | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; nullable boundary + duplicate def + operand issues are all fixture-side or out-of-scope compiler lanes |
| 3 | 0025_reverse_nodes_in_k_group | field_expression_access_unsupported | both | CORRECT | `.next` blocked + borrowed param/moved value fixture issues |
| 4 | 0061_rotate_list | field_expression_access_unsupported | both | CORRECT | `.next` blocked (x3) + immutable param/borrowed param/moved value |
| 5 | 0083_remove_duplicates_from_sorted_list | field_expression_access_unsupported | both | CORRECT | `.val` blocked + borrowed param/moved value |
| 6 | 0092_reverse_linked_list_ii | field_expression_access_unsupported | both | CORRECT | `.next` blocked + attribute assignment target/missing annotation/undefined var |
| 7 | 0094_binary_tree_inorder_traversal | field_expression_access_unsupported | compiler_fix | CORRECT | `.right`+`.val` blocked; `list[int] vs list[Any]` is separate compiler typing gap; no fixture structural issues |
| 8 | 0101_symmetric_tree | field_expression_access_unsupported | compiler_fix | CORRECT | `.left`(x2)+`.right` blocked; return-path completeness and deque truthiness are both compiler gaps; no fixture structural rewrite needed |
| 9 | 0112_path_sum | field_expression_access_unsupported | both | CORRECT | `.left`(x2)+`.val` blocked + duplicate function def/missing annotations (fixture) |
| 10 | 0124_binary_tree_maximum_path_sum | nullable_function_boundary_signature | both | CORRECT | Nullable helper contract (fixture) + `max()` with optional operands (compiler) + unary not TreeNode (adaptation to explicit `is None`) |
| 11 | 0138_copy_list_with_random_pointer | nullable_function_boundary_signature | both | CORRECT | Resolution path is correct. **Subcategory concern**: dominant blockers are field expression access (`.next` x3, `.random`, `.val`) not the nullable boundary. See Section 2 for detail. |
| 12 | 0143_reorder_list | field_expression_access_unsupported | both | CORRECT | `.next` blocked + chained assignment/borrowed param/undefined vars/moved values |
| 13 | 0146_lru_cache | field_expression_access_unsupported | both | CORRECT | `.prev` blocked + chained assignments(x4)/missing annotations(x6)/no field 'cache'(x4)/no field 'right' -- massive fixture issues |
| 14 | 0147_insertion_sort_list | field_expression_access_unsupported | both | CORRECT | `.next`+`.val` blocked + borrowed param/moved value inside loop |
| 15 | 0148_sort_list | field_expression_access_unsupported | both | CORRECT | `.next`(x3) blocked + borrowed param(x2)/truthiness/moved value |
| 16 | 0199_binary_tree_right_side_view | field_expression_access_unsupported | both | CORRECT | `.left`+`.right`+`.val` blocked + moved values need `own` annotation (adaptation) |
| 17 | 0203_remove_linked_list_elements | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; nullable signature + borrowed param (`own`) + ListNode truthiness/comparison rewritable to canonical `is None` patterns; out-of-scope compiler truthiness issues tracked elsewhere |
| 18 | 0211_design_add_and_search_words_data_structure | field_expression_access_unsupported | both | CORRECT | `.children`(x3)+`.word` blocked + missing annotations/no field 'root'/undefined vars (fixture) |
| 19 | 0236_lowest_common_ancestor_of_a_binary_tree | quoted_forward_ref_boundary_mismatch | sifr_adaptation | **INCORRECT** | **Diagnostics show `.left` and `.right` field expression access errors requiring compiler support.** See Section 2. |
| 20 | 0297_serialize_and_deserialize_binary_tree | field_expression_access_unsupported | both | CORRECT | `.left`+`.right`+`.val` blocked + missing annotations(x4)/undefined vars(x4)/Any typing |
| 21 | 0450_delete_node_in_a_bst | field_expression_access_unsupported | both | CORRECT | `.left` blocked + immutable param mutation(x4)/borrowed param |
| 22 | 0513_find_bottom_left_tree_value | optional_node_in_container_elements | both | CORRECT | Container element typing (compiler) + `.left`+`.right`+`.val` field access (compiler) + duplicate function def (fixture) |
| 23 | 0572_subtree_of_another_tree | field_expression_access_unsupported | both | CORRECT | `.val` blocked + return-path completeness needs fixture-side explicit return |
| 24 | 0606_construct_string_from_binary_tree | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; nullable boundary + untyped vars + `unknown type: 'list'` -- all fixture-side |
| 25 | 0617_merge_two_binary_trees | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; nullable boundary + `not TreeNode` rewritable to `is None` (Sifr canonical pattern) -- all adaptation |
| 26 | 0662_maximum_width_of_binary_tree | optional_node_in_container_elements | both | CORRECT | Container element typing (compiler) + `unsupported operand for -: TreeNode` (fixture/compiler) |
| 27 | 0729_my_calendar_i | field_expression_access_unsupported | both | CORRECT | `.end` blocked + missing annotations(x5)/no field 'calendar' (fixture) |
| 28 | 0783_minimum_distance_between_bst_nodes | field_expression_access_unsupported | both | CORRECT | `.val` blocked + `nonlocal` mutation unsupported (compiler) + int/float return mismatch + moved value (fixture) |
| 29 | 0876_middle_of_the_linked_list | field_expression_access_unsupported | both | CORRECT | `.next`(x2) blocked + borrowed param needs `own` (adaptation) |
| 30 | 0894_all_possible_full_binary_trees | optional_return_variance_mismatch | sifr_adaptation | CORRECT | Pure annotation mismatch; canonicalize return type -- no compiler gap |
| 31 | 1609_even_odd_tree | field_expression_access_unsupported | both | CORRECT | `.left`+`.right`+`.val`(x2) blocked + deque truthiness needs Sifr-safe shaping |
| 32 | 1669_merge_in_between_linked_lists | field_expression_access_unsupported | both | CORRECT | `.next` blocked + immutable param(x2)/borrowed param/moved value(x2) |
| 33 | 1721_swapping_nodes_in_a_linked_list | field_expression_access_unsupported | both | CORRECT | `.next`(x2) blocked + borrowed param/undefined var/moved values(x5)/loop move |
| 34 | 2130_maximum_twin_sum_of_a_linked_list | field_expression_access_unsupported | both | CORRECT | `.next`(x2)+`.val` blocked + borrowed param |

---

## 2. Corrections

### 2.1 INCORRECT: 0236_lowest_common_ancestor_of_a_binary_tree

**Current classification:**
- subcategory: `quoted_forward_ref_boundary_mismatch`
- resolution_path: `sifr_adaptation`

**Full diagnostics (from `recursive_node_field_34_diagnostics_20260404.txt`):**
```
type error: argument 1 ('root') of function 'lowestCommonAncestor': expected '"TreeNode"', got 'TreeNode'  (x3)
type error: argument 2 ('p') of function 'lowestCommonAncestor': expected '"TreeNode"', got 'int'  (x3)
type error: argument 3 ('q') of function 'lowestCommonAncestor': expected '"TreeNode"', got 'int'  (x3)
type error: attribute access '.left' is not supported as an expression; use as a method call    <-- COMPILER
type error: attribute access '.right' is not supported as an expression; use as a method call   <-- COMPILER
type error: bad operand type for unary not: '"TreeNode"'
type error: cannot compare '"TreeNode"' and 'None' with ==  (x3)
type error: cannot return borrowed parameter `root`
type error: return type mismatch: expected '"TreeNode"', got 'None'
type error: undefined variable: 'left'  (x2)
type error: undefined variable: 'right'
```

**Problem:** The diagnostics contain `attribute access '.left' is not supported as an expression` and `attribute access '.right' is not supported as an expression`. These are field expression access errors that require **compiler support** to resolve. Even after fixing the quoted forward refs and canonicalizing signatures, the fixture will remain blocked on `.left`/`.right` field reads.

**Corrected classification:**
- subcategory: `quoted_forward_ref_boundary_mismatch` (unchanged -- first diagnostic is consistent)
- resolution_path: **`both`** (compiler must support `.left`/`.right` field expression access; fixture must fix quoted forward refs, nullable comparisons, and signature contracts)
- corrected rationale: "Quoted forward ref boundary mismatch requires fixture-side canonicalization (unquoted TreeNode, explicit nullable handling). Compiler must also support `.left`/`.right` field expression access. Both lanes required."

### 2.2 ADVISORY: 0138_copy_list_with_random_pointer (subcategory concern)

**Not marked incorrect** (methodology-consistent), but flagged for planning accuracy.

The subcategory `nullable_function_boundary_signature` follows the first-diagnostic convention. However, the diagnostics reveal **5 distinct field expression access errors** (`.next` x3, `.random`, `.val`) which are the dominant compiler blockers. For lane-planning purposes, this fixture is more accurately characterized as a `field_expression_access_unsupported` case that also has nullable boundary issues. The resolution_path `both` correctly captures this dual nature.

If subcategory classification is used for lane prioritization, consider reclassifying 0138 to `field_expression_access_unsupported` to avoid undercounting the field-expression compiler workload.

---

## 3. Corrected Aggregate Counts

### 3.1 Subcategory Counts (UNCHANGED)

The 0236 correction affects resolution_path only, not subcategory.

| subcategory | original | corrected |
|---|---|---|
| field_expression_access_unsupported | 24 | 24 |
| nullable_function_boundary_signature | 6 | 6 |
| optional_node_in_container_elements | 2 | 2 |
| optional_return_variance_mismatch | 1 | 1 |
| quoted_forward_ref_boundary_mismatch | 1 | 1 |
| **total** | **34** | **34** |

If advisory on 0138 is accepted (reclassify subcategory):

| subcategory | adjusted |
|---|---|
| field_expression_access_unsupported | 25 |
| nullable_function_boundary_signature | 5 |
| optional_node_in_container_elements | 2 |
| optional_return_variance_mismatch | 1 |
| quoted_forward_ref_boundary_mismatch | 1 |
| **total** | **34** |

### 3.2 Resolution Ownership Counts (CORRECTED)

| resolution_path | original | corrected | delta |
|---|---|---|---|
| both | 26 | **27** | +1 (0236 moved from sifr_adaptation) |
| sifr_adaptation | 6 | **5** | -1 (0236 moved to both) |
| compiler_fix | 2 | 2 | -- |
| **total** | **34** | **34** | -- |

### 3.3 Impact on Execution Recommendation

The breakdown issue lists Lane B (adaptation) fixtures as `0021, 0203, 0236, 0606, 0617, 0894`. With the correction:

- **Lane B (adaptation-only):** `0021, 0203, 0606, 0617, 0894` (5 fixtures, not 6)
- **Lane C (mixed/both):** gains `0236` (now 27 fixtures total requiring compiler closure before/alongside adaptation)

This means 0236 **cannot** be unblocked by adaptation alone -- it must wait for Lane A (compiler field-expression support) to land first.

---

## 4. Validation Summary

| metric | value |
|---|---|
| total rows reviewed | 34 |
| correct | 33 |
| incorrect | 1 (0236: resolution_path) |
| advisory flags | 1 (0138: subcategory suboptimal for planning) |
| error rate | 2.9% |
| first_diagnostic consistency (CSV vs diagnostics file vs taxonomy JSON) | 34/34 verified |
| subcategory count checksum | 24+6+2+1+1 = 34 PASS |
| resolution ownership count checksum (corrected) | 27+5+2 = 34 PASS |

---

## 5. Final Verdict

**NOT_READY**

One fixture (0236) has an incorrect `resolution_path` classification that materially affects lane planning. The fixture requires compiler field-expression support (`.left`, `.right`) and cannot be resolved by adaptation alone. The corrected resolution ownership shifts from 26/6/2 to 27/5/2 (both/sifr_adaptation/compiler_fix).

Apply the correction to 0236 and update the breakdown issue's Lane B fixture list and aggregate counts before proceeding to execution.
