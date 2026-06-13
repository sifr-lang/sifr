# Optional/None Flow and Narrowing Gap (38) Root Cause Breakdown

- Snapshot analyzed: `verification/leetcode/full_corpus_current_results_20260402_live_after_ownership_boundary_closure.json`
- Category source: `verification/leetcode/full_corpus_failure_taxonomy_20260402_live_after_ownership_boundary_closure_reclass.json`
- Deep diagnostics rerun: `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_rerun.json`
- Inventory: `verification/leetcode/optional_none_gap_38_root_cause_inventory_20260403.csv`
- Post-fix sweep (after 1980 + 0103 fixture closures): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_0103.tsv`
- Post-fix sweep (after 1980 + 0103 + 0047 fixture closures): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_0103_0047.tsv`
- Post-fix sweep (after 1980 + 0103 + 0047 + 0875 fixture closures): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_0103_0047_0875.tsv`
- Post-fix sweep (after 1980 + 0103 + 0047 + 0875 + 0057 fixture closures): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_0103_0047_0875_0057.tsv`
- Post-fix sweep (after 1980 + 0103 + 0047 + 0875 + 0057 + 0064 fixture closures): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_0103_0047_0875_0057_0064.tsv`
- Post-fix sweep (after 1980 + 0103 + 0047 + 0875 + 0057 + 0064 + 0139 + 0904 fixture closures): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_0103_0047_0875_0057_0064_0139_0904.tsv`
- Post-fix sweep (after 1980 + 0103 + 0047 + 0875 + 0057 + 0064 + 0139 + 0904 + 0977 + 0438 fixture closures): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_0103_0047_0875_0057_0064_0139_0904_0977_0438.tsv`
- Post-fix sweep (closure to 23 pass / 15 fail): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_closure_to_23_pass.tsv`
- Post-fix sweep (closure to 24 pass / 14 fail): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_closure_to_24_pass.tsv`
- Post-fix sweep (closure to 25 pass / 13 fail): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_closure_to_25_pass.tsv`
- Post-fix sweep (closure to 27 pass / 11 fail): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_closure_to_27_pass.tsv`
- Post-fix sweep (closure to 28 pass / 10 fail): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980_closure_to_28_pass.tsv`
- Post-fix sweep (final closure to 38 pass / 0 fail): `verification/leetcode/optional_none_gap_38_full_diagnostics_20260403_post_1980.tsv`
- Phase closure PR (2026-04-03): `https://github.com/sifr-lang/sifr/pull/1573`

## Decision Summary
- `compiler_fix`: `25`
- `both`: `12`
- `sifr_adaptation`: `1`

## Current Drift Check (2026-04-03 check-only, final closure sweep)
- Still failing: `0`
- Now passing at check stage: `38`
- Passes with warnings: `8`

## Largest Unresolved Subcategories (Current 0 Failures)
- none

## Language Adjustment Decision
- Keep core language principles unchanged (static safety, explicit Option/Result, no panic paths).
- `compiler_fix` items should be solved by stronger control-flow narrowing and dominated-index/container element refinement, not by weakening typing rules.
- `sifr_adaptation` item (`1980`) should be rewritten to explicit Option flow; this is canonical Sifr style, not a compiler loophole request.
- `both` items require both compiler precision and canonical fixture adaptation (stdlib surface/undefined-name cleanup) while keeping strict typing intact.

## Root Cause Clusters
- `cluster_container_element_optional_contamination`: `5`
- `cluster_index_based_optional_leakage`: `5`
- `cluster_two_pointer_index_optional_arithmetic`: `4`
- `cluster_binary_search_index_optional_arithmetic`: `3`
- `cluster_cfg_narrowing_for_builtin_min_max`: `3`
- `cluster_heap_pop_optional_unpack`: `3`
- `cluster_parse_result_and_optional_stack`: `3`
- `cluster_recursive_argument_optional_leakage`: `2`
- `cluster_recursive_constructor_nullable_arg`: `2`
- `cluster_string_index_optional_leakage`: `2`
- `cluster_none_comparison_and_matrix_indexing`: `1`
- `cluster_optional_sentinel_return_shape`: `1`
- `cluster_recursive_container_boundary`: `1`
- `cluster_recursive_nullable_return_contract`: `1`
- `cluster_stdlib_compat_counter_surface`: `1`
- `cluster_stdlib_compat_deque_surface`: `1`

## Ownership x Cluster
- `compiler_fix`
`cluster_container_element_optional_contamination`: `5`
`cluster_index_based_optional_leakage`: `5`
`cluster_two_pointer_index_optional_arithmetic`: `3`
`cluster_binary_search_index_optional_arithmetic`: `2`
`cluster_cfg_narrowing_for_builtin_min_max`: `2`
`cluster_recursive_argument_optional_leakage`: `2`
`cluster_recursive_constructor_nullable_arg`: `2`
`cluster_recursive_container_boundary`: `1`
`cluster_recursive_nullable_return_contract`: `1`
`cluster_stdlib_compat_counter_surface`: `1`
`cluster_string_index_optional_leakage`: `1`
- `both`
`cluster_heap_pop_optional_unpack`: `3`
`cluster_parse_result_and_optional_stack`: `3`
`cluster_binary_search_index_optional_arithmetic`: `1`
`cluster_cfg_narrowing_for_builtin_min_max`: `1`
`cluster_none_comparison_and_matrix_indexing`: `1`
`cluster_stdlib_compat_deque_surface`: `1`
`cluster_string_index_optional_leakage`: `1`
`cluster_two_pointer_index_optional_arithmetic`: `1`
- `sifr_adaptation`
`cluster_optional_sentinel_return_shape`: `1`

## Reviewer Validation
- Claude review pass 1: `reviews/optional-none-gap-38-root-cause-breakdown-review-pass1.md`
- Claude review pass 2: `reviews/optional-none-gap-38-root-cause-breakdown-review-pass2.md` (contained a disputed owner-count claim).
- Claude review pass 3 reconciliation: `reviews/optional-none-gap-38-root-cause-breakdown-review-pass3.md`.
- Final reviewer-backed verdict: owner split is `compiler_fix=25`, `both=12`, `sifr_adaptation=1`.

## Per-Case Decisions
- `0002_add_two_numbers` | owner=`compiler_fix` | cluster=`cluster_recursive_nullable_return_contract`
current status: pass (exit=0)
snapshot: type error: return type mismatch: expected 'ListNode', got 'None | ListNode'
current check: no errors found
rationale: return ListNode vs None|ListNode should follow declared nullable linked-list contract
- `0046_permutations` | owner=`compiler_fix` | cluster=`cluster_container_element_optional_contamination`
current status: pass (exit=0)
snapshot: type error: list.append() argument type 'int | None' is not compatible with list element type 'int'
current check: no errors found
rationale: list element type poisoned by optional index read despite dominated loop bounds
- `0047_permutations_ii` | owner=`compiler_fix` | cluster=`cluster_stdlib_compat_counter_surface`
current status: pass (exit=0)
snapshot: type error: argument 1 ('source') of function '__compat_sifr_collections_Counter': expected 'None | dict[T, int]', got 'list[int]'
current check: no errors found
rationale: Counter compatibility signature/inference mismatch cascades into optional/Any contamination
- `0057_insert_interval` | owner=`compiler_fix` | cluster=`cluster_cfg_narrowing_for_builtin_min_max`
current status: pass (exit=0)
snapshot: type error: min() with 2 arguments does not accept optional operands; got 'int | None' and 'int | None' (guard or unwrap first)
current check: no errors found
rationale: min() sees Optional operands that should be narrowed to concrete interval endpoints
- `0064_minimum_path_sum` | owner=`both` | cluster=`cluster_cfg_narrowing_for_builtin_min_max`
current status: pass (exit=0)
snapshot: type error: min() with 2 arguments does not accept optional operands; got 'float | None' and 'float | None' (guard or unwrap first)
current check: no errors found
rationale: optional DP cell leakage plus numeric/int-vs-float contract cleanup needed
- `0088_merge_sorted_array` | owner=`compiler_fix` | cluster=`cluster_container_element_optional_contamination`
current status: pass (exit=0)
snapshot: type error: list subscript assignment value type 'int | None' is not compatible with list element type 'int'
current check: no errors found
rationale: subscript assignment propagates int|None into list[int] under bounded indices
- `0103_binary_tree_zigzag_level_order_traversal` | owner=`compiler_fix` | cluster=`cluster_recursive_container_boundary`
current status: pass (exit=0)
snapshot: type error: cannot compare 'list[list[int]]' and 'None' with ==
current check: no errors found
rationale: tree queue/list operations leak None|TreeNode despite control-flow guards
- `0105_construct_binary_tree_from_preorder_and_inorder_traversal` | owner=`compiler_fix` | cluster=`cluster_index_based_optional_leakage`
current status: pass (exit=0)
snapshot: type error: unsupported operand type(s) for +: 'int | None' and 'int'
current check: no errors found
rationale: indexed tree-split arithmetic leaks int|None where index is dominated as present
- `0106_construct_binary_tree_from_inorder_and_postorder_traversal` | owner=`compiler_fix` | cluster=`cluster_recursive_constructor_nullable_arg`
current status: pass (exit=0)
snapshot: type error: argument 1 ('val') of function 'TreeNode': expected 'int', got 'int | None'
current check: no errors found
rationale: TreeNode constructor receives int|None because recursive split index not stabilized
- `0108_convert_sorted_array_to_binary_search_tree` | owner=`compiler_fix` | cluster=`cluster_recursive_constructor_nullable_arg`
current status: pass (exit=0)
snapshot: type error: argument 1 ('val') of function 'TreeNode': expected 'int', got 'int | None'
current check: no errors found
rationale: TreeNode constructor argument should be concrete under midpoint bounds
- `0139_word_break` | owner=`compiler_fix` | cluster=`cluster_container_element_optional_contamination`
current status: pass (exit=0)
snapshot: type error: list subscript assignment value type 'bool | None' is not compatible with list element type 'bool'
current check: no errors found
rationale: memo table updates retain bool|None instead of refining to bool
- `0150_evaluate_reverse_polish_notation` | owner=`both` | cluster=`cluster_parse_result_and_optional_stack`
current status: pass (exit=0)
snapshot: type error: return type mismatch: expected 'int', got 'Result[int, ParseError] | None'
current check: warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values
rationale: parse Result plus optional stack pop semantics require explicit adaptation and better typing flow
- `0261_graph_valid_tree` | owner=`compiler_fix` | cluster=`cluster_index_based_optional_leakage`
current status: pass (exit=0)
snapshot: type error: argument 1 of callable 'union': expected 'int', got 'int | None'
current check: no errors found
rationale: union/find arguments leak int|None from list access despite valid edge iteration shape
- `0287_find_the_duplicate_number` | owner=`compiler_fix` | cluster=`cluster_index_based_optional_leakage`
current status: pass (exit=0)
snapshot: type error: cannot index type 'list[int]' with 'None | int'
current check: no errors found
rationale: cycle-detection indices remain Optional at dominated indexing sites
- `0304_range_sum_query_2d_immutable` | owner=`both` | cluster=`cluster_none_comparison_and_matrix_indexing`
current status: pass (exit=0)
snapshot: type error: cannot compare 'None' and 'int' with ==
current check: no errors found
rationale: None/int comparison and nested indexing indicate mixed fixture-shape adaptation plus compiler narrowing gaps
- `0329_longest_increasing_path_in_a_matrix` | owner=`compiler_fix` | cluster=`cluster_recursive_argument_optional_leakage`
current status: pass (exit=0)
snapshot: type error: argument 3 of callable 'dfs': expected 'int', got 'int | None'
current check: no errors found
rationale: recursive dfs args leak int|None despite checked coordinate bounds
- `0394_decode_string` | owner=`both` | cluster=`cluster_parse_result_and_optional_stack`
current status: pass (exit=0)
snapshot: type error: type 'None | str' has no method 'isdigit'
current check: no errors found
rationale: isdigit and arithmetic receive None/Result from stack+parse operations without explicit adaptation
- `0417_pacific_atlantic_water_flow` | owner=`compiler_fix` | cluster=`cluster_recursive_argument_optional_leakage`
current status: pass (exit=0)
snapshot: type error: argument 4 of callable 'dfs': expected 'int', got 'int | None'
current check: no errors found
rationale: dfs call arguments leak int|None from bounded grid traversal indices
- `0438_find_all_anagrams_in_a_string` | owner=`compiler_fix` | cluster=`cluster_string_index_optional_leakage`
current status: pass (exit=0)
snapshot: type error: 'in' operator: element type 'str | None' is not compatible with collection element type 'str'
current check: no errors found
rationale: char extraction under bounded window leaks str|None into dict/in checks
- `0452_minimum_number_of_arrows_to_burst_balloons` | owner=`compiler_fix` | cluster=`cluster_cfg_narrowing_for_builtin_min_max`
current status: pass (exit=0)
snapshot: type error: min() with 2 arguments does not accept optional operands; got 'int | None' and 'int | None' (guard or unwrap first)
current check: no errors found
rationale: min() optional operands should narrow from sorted interval access
- `0567_permutation_in_string` | owner=`both` | cluster=`cluster_parse_result_and_optional_stack`
current status: pass (exit=0)
snapshot: type error: ord() argument must be 'str', got 'str | None'
current check: no errors found
rationale: ord receives str|None and parse/result pollution requires explicit adaptation + better narrowing
- `0778_swim_in_rising_water` | owner=`both` | cluster=`cluster_heap_pop_optional_unpack`
current status: pass (exit=0)
snapshot: type error: cannot unpack non-tuple type 'None | list[int | None]'
current check: warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values
rationale: heap pop/unpack Optional tuple/list semantics need explicit non-empty adaptation and container typing improvements
- `0802_find_eventual_safe_states` | owner=`compiler_fix` | cluster=`cluster_index_based_optional_leakage`
current status: pass (exit=0)
snapshot: type error: cannot iterate over type 'list[int] | None'
current check: no errors found
rationale: iteration over graph[i] leaks list[int]|None despite bounded i loop
- `0875_koko_eating_bananas` | owner=`both` | cluster=`cluster_binary_search_index_optional_arithmetic`
current status: pass (exit=0)
snapshot: type error: return type mismatch: expected 'int', got 'int | None'
current check: no errors found
rationale: binary-search math uses int|None intermediates; needs narrowing + canonical adaptation cleanup
- `0881_boats_to_save_people` | owner=`compiler_fix` | cluster=`cluster_two_pointer_index_optional_arithmetic`
current status: pass (exit=0)
snapshot: type error: unsupported operand type(s) for +: 'int | None' and 'int | None'
current check: no errors found
rationale: two-pointer bounded indices still typed as Optional in arithmetic
- `0904_fruit_into_baskets` | owner=`compiler_fix` | cluster=`cluster_container_element_optional_contamination`
current status: pass (exit=0)
snapshot: type error: dict subscript assignment key type 'int | None' is not compatible with dict key type 'int'
current check: no errors found
rationale: dict key receives int|None from bounded array index access
- `0948_bag_of_tokens` | owner=`compiler_fix` | cluster=`cluster_two_pointer_index_optional_arithmetic`
current status: pass (exit=0)
snapshot: type error: unsupported operand type(s) for +: 'int' and 'int | None'
current check: no errors found
rationale: two-pointer score math polluted by Optional index value under guarded bounds
- `0977_squares_of_a_sorted_array` | owner=`both` | cluster=`cluster_two_pointer_index_optional_arithmetic`
current status: pass (exit=0)
snapshot: type error: abs() argument must be numeric, got 'int | None'
current check: warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values
rationale: abs receives int|None; plus fixture local-name hygiene needed in canonical form
- `1203_sort_items_by_groups_respecting_dependencies` | owner=`both` | cluster=`cluster_stdlib_compat_deque_surface`
current status: pass (exit=0)
snapshot: type error: argument 1 ('items') of function '__compat_sifr_collections_deque': expected 'None | list[T]', got 'Iterator[int]'
current check: no errors found
rationale: deque compat expects list but gets iterator and optional/index contamination follows
- `1397_find_all_good_strings` | owner=`both` | cluster=`cluster_string_index_optional_leakage`
current status: pass (exit=0)
snapshot: type error: cannot index type 'str' with 'int | None'
current check: warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values
rationale: string indexing with int|None plus helper-shape adaptation required for deterministic typing
- `1423_maximum_points_you_can_obtain_from_cards` | owner=`compiler_fix` | cluster=`cluster_index_based_optional_leakage`
current status: pass (exit=0)
snapshot: type error: unsupported operand type(s) for -: 'int | None' and 'int | None'
current check: no errors found
rationale: window arithmetic leaks Optional from bounded card index access
- `1498_number_of_subsequences_that_satisfy_the_given_sum_condition` | owner=`compiler_fix` | cluster=`cluster_two_pointer_index_optional_arithmetic`
current status: pass (exit=0)
snapshot: type error: unsupported operand type(s) for +: 'int | None' and 'int | None'
current check: warning: int left shift (<<) with non-constant shift amount may overflow i64 at runtime; consider using bigint
rationale: two-pointer arithmetic sees Optional where bounds imply concrete ints
- `1584_min_cost_to_connect_all_points` | owner=`both` | cluster=`cluster_heap_pop_optional_unpack`
current status: pass (exit=0)
snapshot: type error: cannot unpack non-tuple type 'None | list[int]'
current check: no errors found
rationale: heap tuple unpack receives Optional; requires explicit non-empty adaptation and comparable typing closure
- `1631_path_with_minimum_effort` | owner=`both` | cluster=`cluster_heap_pop_optional_unpack`
current status: pass (exit=0)
snapshot: type error: cannot unpack non-tuple type 'None | tuple[int, int, int]'
current check: warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values
rationale: priority-queue pop/unpack Optional tuple leakage plus return-type stabilization needed
- `1700_number_of_students_unable_to_eat_lunch` | owner=`compiler_fix` | cluster=`cluster_container_element_optional_contamination`
current status: pass (exit=0)
snapshot: type error: list.append() argument type 'int | None' is not compatible with list element type 'int'
current check: no errors found
rationale: queue/list append path leaks int|None into list[int]
- `1838_frequency_of_the_most_frequent_element` | owner=`compiler_fix` | cluster=`cluster_binary_search_index_optional_arithmetic`
current status: pass (exit=0)
snapshot: type error: unsupported operand type(s) for -: 'int' and 'int | None'
current check: warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values
rationale: window arithmetic leaks Optional from bounded sorted array indices
- `1980_find_unique_binary_string` | owner=`sifr_adaptation` | cluster=`cluster_optional_sentinel_return_shape`
current status: pass (exit=0)
snapshot: type error: if expression branches have incompatible types: 'None' and 'str'
current check: no errors found
rationale: Python-style None sentinel branch in str-return helper should be rewritten to explicit Option flow
- `2300_successful_pairs_of_spells_and_potions` | owner=`compiler_fix` | cluster=`cluster_binary_search_index_optional_arithmetic`
current status: pass (exit=0)
snapshot: type error: unsupported operand type(s) for *: 'int' and 'int | None'
current check: warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values
rationale: binary-search multiplication uses int|None from bounded potions index
