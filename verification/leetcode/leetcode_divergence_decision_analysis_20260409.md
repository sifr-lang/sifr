# LeetCode Divergence Decision Analysis

Date: 2026-04-09
Source ranking: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Scope rule: focus on paired fixtures with `changed_total_lines >= 80`, then manually include important parity-debt exceptions below the cutoff where needed.

## Preconditions Before Using This Analysis

- Raw diff size is only a triage signal, not a calibrated severity score.
- Items near the cutoff, especially in the `70-90` changed-line band, require manual judgment with `similarity_ratio` and signed line delta before escalation.
- Public-surface changes and asymptotic regressions matter more than raw line count.
- Shared helper boilerplate can hide divergence because mirrored dead code cancels out of the diff.
- Some Python fixtures contain multiple full implementations, which inflates divergence artificially.
- "Python-like" pressure must be split into:
  - language/compiler ergonomics
  - stdlib/data-structure parity
  - explicit parity-debt rewrites

## Main Decision Categories

### 1. Should Have Parity, Rewrite Mainly

These are not just verbose Sifr ports. They should be rewritten toward the canonical Python problem shape once the necessary surface exists.

- `0023_merge_k_sorted_lists`
- `0133_clone_graph`
- `0148_sort_list`
- `0212_word_search_ii`
- `0295_find_median_from_data_stream`
- `0707_design_linked_list`

Why:

- `0023_merge_k_sorted_lists` changes the public input model from linked lists to `list[list[int]]`.
- `0133_clone_graph` changes the public model from object-graph cloning to adjacency-list copying.
- `0148_sort_list` replaces linked-list merge sort with flatten/sort/rebuild.
- `0212_word_search_ii` replaces trie/prefix-pruning with per-word board search.
- `0295_find_median_from_data_stream` replaces heap-based updates with sorted-array insertion and changes asymptotic behavior.
- `0707_design_linked_list` replaces a linked-list data-structure design with array-backed storage and loses the intended operation-cost profile.

### 2. Should Support Similar Python Features / Ergonomics

These are valid Sifr targets, but the language and stdlib should make the canonical solution shape much easier to express safely.

#### 2a. Recursive Node / Cursor Ergonomics

- `0002_add_two_numbers`
- `0019_remove_nth_node_from_end_of_list`
- `0021_merge_two_sorted_lists`
- `0025_reverse_nodes_in_k_group`
- `0061_rotate_list`
- `0083_remove_duplicates_from_sorted_list`
- `0086_partition_list`
- `0092_reverse_linked_list_ii`
- `0143_reorder_list`
- `0147_insertion_sort_list`
- `0160_intersection_of_two_linked_lists`
- `0203_remove_linked_list_elements`
- `0234_palindrome_linked_list`
- `0297_serialize_and_deserialize_binary_tree`
- `0450_delete_node_in_a_bst`
- `0513_find_bottom_left_tree_value`
- `0662_maximum_width_of_binary_tree`
- `0669_trim_a_binary_search_tree`
- `0876_middle_of_the_linked_list`
- `0894_all_possible_full_binary_trees`
- `1609_even_odd_tree`
- `1669_merge_in_between_linked_lists`
- `1721_swapping_nodes_in_a_linked_list`
- `2130_maximum_twin_sum_of_a_linked_list`

What should improve:

- narrowing after `is not None`
- compiler-preserved narrowing within a proven scope, including across rebinding when the new value is provably the same type; no user-side re-narrowing required
- easier safe field access on recursive nodes
- clearer cursor-style mutation patterns without weakening ownership

#### 2b. Collection / Index / Stdlib Ergonomics

- `0130_surrounded_regions`
- `0150_evaluate_reverse_polish_notation`
- `0261_graph_valid_tree`
- `0269_alien_dictionary`
- `0286_walls_and_gates`
- `0355_design_twitter`
- `0394_decode_string`
- `0417_pacific_atlantic_water_flow`
- `0567_permutation_in_string`
- `0721_accounts_merge`
- `0743_network_delay_time`
- `0752_open_the_lock`
- `0778_swim_in_rising_water`
- `1203_sort_items_by_groups_respecting_dependencies`
- `1397_find_all_good_strings`
- `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`
- `1584_min_cost_to_connect_all_points`
- `1631_path_with_minimum_effort`
- `2092_find_all_people_with_secret`
- `2709_greatest_common_divisor_traversal`

What should improve:

- preserve proven non-Optional collection/index values across normal statement flow so fixtures do not need dead guard boilerplate
- safer owned collection helpers with minimal cloning and predictable ownership behavior
- stdlib parity where it materially unblocks canonical algorithms:
  - `heap`
  - `deque`
  - DSU / union-find helpers
  - trie-friendly dictionary ergonomics

### 3. Okay The Way They Are

These are high diff mostly because the Python side is noisy or redundant, not because the Sifr version is meaningfully wrong.

- `0104_maximum_depth_of_binary_tree`
- `0200_number_of_islands`
- `0516_longest_palindromic_subsequence`

Why:

- `0104_maximum_depth_of_binary_tree.py` includes multiple complete implementations and extra helper baggage, while the Sifr version already matches a clean recursive solution.
- `0200_number_of_islands.py` contains three full implementations in one file.
- `0516_longest_palindromic_subsequence.py` contains multiple solution families, while the Sifr version is a clean LCS-style solution.
- These are strong enough corpus-noise cases that they should not drive language priorities.

### 4. Acceptable Divergence Because Of An Intentional Architecture Boundary

These are not ideal only in the abstract, but the current divergence is acceptable because it sits behind a deliberate Sifr design boundary.

- `0673_number_of_longest_increasing_subsequence`

Why:

- The Python version uses mutable `nonlocal` closure state.
- Sifr intentionally does not support that architecture.
- The iterative rewrite preserves the same `O(n^2)` asymptotic behavior, so this is an architecture-boundary divergence rather than hidden parity loss.
- It should not be used as pressure to add mutable `nonlocal` support.

### 5. Needs Corpus Cleanup Before It Should Drive Design Priorities

This is a secondary label for fixtures whose primary classification is "okay as-is" but whose raw diff is dominated by noisy Python-side references.

Why:

- applies to the Category 3 fixtures (`0104_maximum_depth_of_binary_tree`, `0200_number_of_islands`, `0516_longest_palindromic_subsequence`), whose raw diffs are inflated by Python-side multi-implementation or helper baggage rather than Sifr-side divergence

## Practical Priority Order

1. Corpus normalization

- mark explicit non-canonical parity-debt fixtures clearly
- normalize helper-boilerplate noise in comparison scripts
- stop treating raw diff buckets as calibrated severity

2. Collection/index Optional-flow cleanup

- highest-leverage cheap win across many fixtures
- especially important for dead Optional-style guards on proven `list[T]` access

3. Recursive-field narrowing and cursor ergonomics

- especially for linked-list and tree rewiring/traversal
- keep rebinding/narrowing behavior compiler-proven rather than user-ceremonial

4. Stdlib primitives in unblock order

- `heap`
- DSU helpers
- `deque`
- trie ergonomics

5. Explicit parity-debt rewrites

- `0023_merge_k_sorted_lists`
- `0133_clone_graph`
- `0148_sort_list`
- `0212_word_search_ii`
- `0295_find_median_from_data_stream`
- `0707_design_linked_list`

This order is a work sequence, not a severity ranking. The rewrite cases remain the highest parity-risk items, but earlier cleanup and ergonomics work should make those rewrites cheaper and less noisy.

## Boundaries To Preserve

- Do not add Python-style truthiness coercions.
- Do not add implicit nullable access.
- Do not weaken ownership to emulate Python aliasing.
- Do not treat every high diff as a language problem before separating:
  - corpus noise
  - stdlib parity gaps
  - real language ergonomics gaps
  - explicit rewrite debt
