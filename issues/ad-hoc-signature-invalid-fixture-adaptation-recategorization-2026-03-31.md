# Signature Adaptation Residual Recategorization (2026-03-31)

- Source plan: `issues/ad-hoc-signature-invalid-fixture-adaptation-plan-2026-03-31.md`
- Full rerun source: `verification/leetcode/full_corpus_current_results_20260331_live_after_signature_adaptation.json`

## Cleared By Signature Adaptation Alone

- `0034_find_first_and_last_position_of_element_in_sorted_array`
- `0044_wildcard_matching`
- `2348_number_of_zero_filled_subarrays`
- `2390_removing_stars_from_a_string`
- `0077_combinations`

## Residual Primary Categories

- `container specialization / collection shape`: 9 fixtures
- `other typed-boundary issue`: 7 fixtures
- `mutability boundary`: 6 fixtures
- `class field declaration boundary`: 2 fixtures
- `return-shape boundary`: 2 fixtures
- `run-stage build/codegen/runtime`: 2 fixtures
- `class/attribute surface mismatch`: 1 fixtures
- `stdlib surface mismatch`: 1 fixtures
- `stdlib symbol availability`: 1 fixtures

## Residual Fixture Mapping

| Fixture | New Status | Primary Category | First Error Line |
|---|---|---|---|
| `0018_4sum` | CHECK_ERROR | container specialization / collection shape | type error: argument 6 of callable 'findNsum': expected 'list[list[int]]', got 'list[Unknown]' |
| `0025_reverse_nodes_in_k_group` | CHECK_ERROR | class/attribute surface mismatch | type error: attribute access '.next' is not supported as an expression; use as a method call |
| `0131_palindrome_partitioning` | CHECK_ERROR | mutability boundary | type error: cannot reassign immutable parameter `l`: add `mut` to the parameter declaration |
| `0202_happy_number` | CHECK_ERROR | mutability boundary | type error: cannot reassign immutable parameter `n`: add `mut` to the parameter declaration |
| `0213_house_robber_ii` | CHECK_ERROR | return-shape boundary | type error: function 'rob' must return a value of type 'int' on all control-flow paths |
| `0252_meeting_rooms` | CHECK_ERROR | container specialization / collection shape | type error: cannot index type 'Any' with 'int' |
| `0253_meeting_rooms` | CHECK_ERROR | other typed-boundary issue | type error: '<' not supported between instances of 'int \| None \| None' and 'int \| None \| None' |
| `0271_encode_and_decode_strings` | CHECK_ERROR | other typed-boundary issue | type error: unsupported operand type(s) for +: 'int' and 'Result[int, ParseError]' |
| `0647_palindromic_substrings` | CHECK_ERROR | mutability boundary | type error: cannot reassign immutable parameter `l`: add `mut` to the parameter declaration |
| `0665_non_decreasing_array` | CHECK_ERROR | mutability boundary | type error: cannot mutate through immutable parameter `nums`: add `mut` to the parameter declaration |
| `0680_valid_palindrome_ii` | CHECK_ERROR | mutability boundary | type error: cannot reassign immutable parameter `i`: add `mut` to the parameter declaration |
| `0698_partition_to_k_equal_sum_subsets` | CHECK_ERROR | other typed-boundary issue | type error: cannot compare 'float' and 'int' with == |
| `0740_delete_and_earn` | CHECK_ERROR | other typed-boundary issue | type error: augmented subscript assignment is not supported for type 'Unknown' |
| `0946_validate_stack_sequences` | RUN_ERROR | run-stage build/codegen/runtime | build error: cargo build failed: |
| `2002_maximum_product_of_the_length_of_two_palindromic_subsequences` | CHECK_ERROR | stdlib surface mismatch | type error: max() takes 1 or 2 arguments |
| `2017_grid_game` | CHECK_ERROR | return-shape boundary | type error: function 'gridGame' must return a value of type 'int' on all control-flow paths |
| `2306_naming_a_company` | CHECK_ERROR | container specialization / collection shape | type error: cannot iterate over type 'set[Any] \| None' |
| `0706_design_hashmap` | CHECK_ERROR | class field declaration boundary | type error: type 'MyHashMap' has no field 'map' |
| `0721_accounts_merge` | CHECK_ERROR | class field declaration boundary | type error: type 'UnionFind' has no field 'par' |
| `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree` | CHECK_ERROR | other typed-boundary issue | type error: cannot compare 'list[list[int]]' and 'list[list[Any]]' with == |
| `0098_validate_binary_search_tree` | RUN_ERROR | run-stage build/codegen/runtime | build error: cargo build failed: |
| `0210_course_schedule_ii` | CHECK_ERROR | container specialization / collection shape | type error: cannot index type 'Unknown' with 'int' |
| `0286_walls_and_gates` | CHECK_ERROR | container specialization / collection shape | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' |
| `0332_reconstruct_itinerary` | CHECK_ERROR | other typed-boundary issue | type error: argument 1 of callable 'dfs': expected 'dict[str, list[str]]', got 'Unknown' |
| `0417_pacific_atlantic_water_flow` | CHECK_ERROR | other typed-boundary issue | type error: argument 4 of callable 'dfs': expected 'int', got 'int \| None' |
| `0752_open_the_lock` | CHECK_ERROR | container specialization / collection shape | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[str]' |
| `0909_snakes_and_ladders` | CHECK_ERROR | container specialization / collection shape | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' |
| `1239_maximum_length_of_a_concatenated_string_with_unique_characters` | CHECK_ERROR | stdlib symbol availability | type error: undefined function: 'Counter' |
| `1448_count_good_nodes_in_binary_tree` | CHECK_ERROR | mutability boundary | type error: cannot reassign immutable parameter `maxVal`: add `mut` to the parameter declaration |
| `2092_find_all_people_with_secret` | CHECK_ERROR | container specialization / collection shape | type error: cannot iterate over type 'Unknown \| None' |
| `2101_detonate_the_maximum_bombs` | CHECK_ERROR | container specialization / collection shape | type error: cannot index type 'Unknown' with 'int' |
