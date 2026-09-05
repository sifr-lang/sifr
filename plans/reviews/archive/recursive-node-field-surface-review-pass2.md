# Review Pass 2: recursive_node_and_field_expression_surface (2026-04-04 Rerun)

- **Reviewer**: agent (automated)
- **Date**: 2026-04-04
- **Input artifacts**:
  - `issues/recursive-node-field-surface-breakdown-2026-04-04.md`
  - `verification/leetcode/recursive_node_field_surface_20260404_inventory.csv`
  - `tmp/recursive_node_field_34_diagnostics_20260404.txt`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260404_live_rerun1.json`

---

## 1. Per-Fixture Verdicts (34 rows)

| # | fixture_slug | subcategory | resolution_path | verdict | notes |
|---|---|---|---|---|---|
| 1 | 0019_remove_nth_node_from_end_of_list | field_expression_access_unsupported | both | CORRECT | `.next` compiler gap + mut/own/optional adaptation needed |
| 2 | 0021_merge_two_sorted_lists | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; all issues (duplicate sig, nullable boundary, truthiness, borrowed) fixable by fixture rewrite |
| 3 | 0025_reverse_nodes_in_k_group | field_expression_access_unsupported | both | CORRECT | `.next` compiler gap + ownership adaptation |
| 4 | 0061_rotate_list | field_expression_access_unsupported | both | CORRECT | `.next` (x3) compiler gap + immutable/borrowed/moved adaptation |
| 5 | 0083_remove_duplicates_from_sorted_list | field_expression_access_unsupported | both | CORRECT | `.val` compiler gap + borrowed/moved adaptation |
| 6 | 0092_reverse_linked_list_ii | field_expression_access_unsupported | both | CORRECT | `.next` compiler gap + attribute assignment + type annotation adaptation |
| 7 | 0094_binary_tree_inorder_traversal | field_expression_access_unsupported | compiler_fix | CORRECT | `.right`+`.val` compiler gaps; `list[int]` vs `list[Any]` comparison is also compiler-side; no fixture rewrite needed |
| 8 | 0101_symmetric_tree | field_expression_access_unsupported | compiler_fix | CORRECT | `.left`+`.right` compiler gaps; deque truthiness and return-path analysis are compiler-side; standard BFS structure needs no rewrite |
| 9 | 0112_path_sum | field_expression_access_unsupported | both | CORRECT | `.left`+`.val` compiler gaps + duplicate function def + missing annotations are fixture-side |
| 10 | 0124_binary_tree_maximum_path_sum | nullable_function_boundary_signature | both | CORRECT | No field expression errors; nullable dfs boundary + `not TreeNode` truthiness + optional max() are mixed compiler/fixture |
| 11 | 0138_copy_list_with_random_pointer | nullable_function_boundary_signature | both | CORRECT | First diagnostic is nullable boundary; also has `.next`+`.random`+`.val` field expression errors and dict/truthiness issues; `both` correct |
| 12 | 0143_reorder_list | field_expression_access_unsupported | both | CORRECT | `.next` compiler gap + chained assignment + borrowed + undefined vars are adaptation |
| 13 | 0146_lru_cache | field_expression_access_unsupported | both | CORRECT | `.prev` compiler gap + heavy adaptation (chained assignment, missing annotations, missing class fields) |
| 14 | 0147_insertion_sort_list | field_expression_access_unsupported | both | CORRECT | `.next`+`.val` compiler gaps + borrowed/moved adaptation |
| 15 | 0148_sort_list | field_expression_access_unsupported | both | CORRECT | `.next` (x3) compiler gaps + borrowed/moved + ListNode truthiness adaptation |
| 16 | 0199_binary_tree_right_side_view | field_expression_access_unsupported | both | CORRECT | `.left`+`.right`+`.val` compiler gaps + truthiness + moved value adaptation |
| 17 | 0203_remove_linked_list_elements | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; nullable sig + comparison + borrowed + truthiness all fixable by fixture rewrite |
| 18 | 0211_design_add_and_search_words_data_structure | field_expression_access_unsupported | both | CORRECT | `.children`+`.word` compiler gaps + missing annotations + missing class fields are adaptation |
| 19 | 0236_lowest_common_ancestor_of_a_binary_tree | quoted_forward_ref_boundary_mismatch | sifr_adaptation | **INCORRECT** | **Diagnostics include `.left` and `.right` field expression access errors (compiler gap). Even after fixing quoted forward refs, field access errors remain. Must be `both`.** |
| 20 | 0297_serialize_and_deserialize_binary_tree | field_expression_access_unsupported | both | CORRECT | `.left`+`.right`+`.val` compiler gaps + missing annotations + Any typing + undefined vars are adaptation |
| 21 | 0450_delete_node_in_a_bst | field_expression_access_unsupported | both | CORRECT | `.left` compiler gap + immutable parameter mutation is adaptation |
| 22 | 0513_find_bottom_left_tree_value | optional_node_in_container_elements | both | CORRECT | Container element typing (compiler) + `.left`+`.right`+`.val` field expression + duplicate function def (adaptation) |
| 23 | 0572_subtree_of_another_tree | field_expression_access_unsupported | both | CORRECT | `.val` compiler gap + missing return path is adaptation |
| 24 | 0606_construct_string_from_binary_tree | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; nullable dfs boundary + Any typing + bare `list` type all fixable by adding annotations and explicit signatures |
| 25 | 0617_merge_two_binary_trees | nullable_function_boundary_signature | sifr_adaptation | CORRECT | No field expression errors; nullable boundary + `not TreeNode` truthiness fixable by rewriting to `is None` checks |
| 26 | 0662_maximum_width_of_binary_tree | optional_node_in_container_elements | both | CORRECT | Container element typing (compiler) + TreeNode subtraction is fixture issue |
| 27 | 0729_my_calendar_i | field_expression_access_unsupported | both | CORRECT | `.end` compiler gap + missing annotations + missing class fields are adaptation |
| 28 | 0783_minimum_distance_between_bst_nodes | field_expression_access_unsupported | both | CORRECT | `.val` compiler gap + nonlocal capture (compiler) + float/int return + moved value are mixed |
| 29 | 0876_middle_of_the_linked_list | field_expression_access_unsupported | both | CORRECT | `.next` (x2) compiler gap + borrowed parameter adaptation |
| 30 | 0894_all_possible_full_binary_trees | optional_return_variance_mismatch | sifr_adaptation | CORRECT | Single error: list covariance. Lists are invariant in Python typing; fixture annotation should be `list[TreeNode]` not `list[None \| TreeNode]` |
| 31 | 1609_even_odd_tree | field_expression_access_unsupported | both | CORRECT | `.left`+`.right`+`.val` compiler gaps + deque truthiness adaptation |
| 32 | 1669_merge_in_between_linked_lists | field_expression_access_unsupported | both | CORRECT | `.next` compiler gap + immutable/borrowed/moved adaptation |
| 33 | 1721_swapping_nodes_in_a_linked_list | field_expression_access_unsupported | both | CORRECT | `.next` (x2) compiler gap + borrowed/moved/loop-body-move adaptation |
| 34 | 2130_maximum_twin_sum_of_a_linked_list | field_expression_access_unsupported | both | CORRECT | `.next` (x2) + `.val` compiler gaps + borrowed adaptation |

---

## 2. Corrections

### Row 19: 0236_lowest_common_ancestor_of_a_binary_tree

| Field | Original | Corrected |
|---|---|---|
| resolution_path | `sifr_adaptation` | `both` |

**Reason**: The full diagnostics for this fixture include:

```
type error: attribute access '.left' is not supported as an expression; use as a method call
type error: attribute access '.right' is not supported as an expression; use as a method call
```

These are compiler-side field expression access gaps identical to the dominant pattern in this bucket. Even after resolving all quoted forward ref mismatches (adaptation), the `.left` and `.right` field expression errors will persist until the compiler supports node field reads in expression positions. Additionally, the fixture requires adaptation for: quoted forward ref removal, borrowed parameter ownership, return type and None comparison fixes. Therefore this fixture requires **both** compiler and adaptation work.

The subcategory (`quoted_forward_ref_boundary_mismatch`) remains correct -- it reflects the first diagnostic encountered, which is the quoted forward ref boundary error. The subcategory assignment convention (first-diagnostic-driven) is consistent across the inventory.

---

## 3. Corrected Aggregate Counts

### Subcategory Counts (UNCHANGED)

| Subcategory | Count |
|---|---|
| field_expression_access_unsupported | 24 |
| nullable_function_boundary_signature | 6 |
| optional_node_in_container_elements | 2 |
| optional_return_variance_mismatch | 1 |
| quoted_forward_ref_boundary_mismatch | 1 |
| **Total** | **34** |

No change -- the subcategory for row 19 was not affected; only the resolution_path was wrong.

### Resolution Ownership (CORRECTED)

| Resolution Path | Original | Corrected | Delta |
|---|---|---|---|
| both | 26 | **27** | +1 |
| sifr_adaptation | 6 | **5** | -1 |
| compiler_fix | 2 | 2 | 0 |
| **Total** | **34** | **34** | |

**Correction**: `0236_lowest_common_ancestor_of_a_binary_tree` moved from `sifr_adaptation` to `both`.

### Corrected sifr_adaptation set (5 fixtures)

| Fixture | Subcategory |
|---|---|
| 0021_merge_two_sorted_lists | nullable_function_boundary_signature |
| 0203_remove_linked_list_elements | nullable_function_boundary_signature |
| 0606_construct_string_from_binary_tree | nullable_function_boundary_signature |
| 0617_merge_two_binary_trees | nullable_function_boundary_signature |
| 0894_all_possible_full_binary_trees | optional_return_variance_mismatch |

### Corrected compiler_fix set (2 fixtures, unchanged)

| Fixture | Subcategory |
|---|---|
| 0094_binary_tree_inorder_traversal | field_expression_access_unsupported |
| 0101_symmetric_tree | field_expression_access_unsupported |

### Corrected both set (27 fixtures)

All remaining 27 fixtures, including the newly corrected `0236`.

---

## 4. Cross-Validation Notes

- All 34 fixture slugs in the CSV match exactly the 34 entries under `recursive_node_and_field_expression_surface` in the taxonomy JSON.
- All `first_diagnostic` values in the CSV match the first error line in the diagnostics file for each fixture.
- All `status` values are `CHECK_ERROR`, consistent with the taxonomy JSON.
- The taxonomy JSON `category_subcategory_counts.recursive_node_and_field_expression_surface.generic` = 34, matching the CSV row count.
- The breakdown document's subcategory and resolution ownership counts were internally consistent but had the 0236 resolution_path error propagated into the ownership totals.

---

## 5. Final Verdict

**NOT_READY**

One fixture (`0236_lowest_common_ancestor_of_a_binary_tree`) has an incorrect `resolution_path` classification. It is classified as `sifr_adaptation` but requires `both` due to `.left` and `.right` field expression access errors in the diagnostics. This error propagates into the resolution ownership aggregates (both: 26->27, sifr_adaptation: 6->5).

The corrected values are provided above. After applying the single correction, the analysis would be ready.

### Impact Assessment

The error is **low severity** for execution planning:
- Lane B (`sifr_adaptation` set) loses one fixture (0236) and shrinks from 6 to 5.
- Lane C (`both` set) gains one fixture (0236) and grows from 26 to 27.
- Lane A (`compiler_fix`) is unaffected.
- The fixture 0236 would have been caught during Lane B execution when field expression errors persisted post-adaptation, so the practical impact on delivery is a minor scheduling inefficiency rather than a correctness hazard.
