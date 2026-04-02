# Signature Adaptation Targeted Validation (2026-03-31)

- Scope: 36 fixtures from issues/ad-hoc-signature-invalid-fixture-adaptation-plan-2026-03-31.md
- Commands per fixture: `cargo run -q -p sifr -- check <fixture>`, then `cargo run -q -p sifr -- run <fixture>`

| Fixture | Check | Run | Notes |
|---|---|---|---|
| `0018_4sum` | FAIL | SKIP | type error: argument 6 of callable 'findNsum': expected 'list[list[int]]', got 'list[Unknown]' type error: cannot mutate through immutable parameter `nums`: add `mut` to the parameter declaration type error: return type mismatch: expected 'list[list[int]]', go |
| `0025_reverse_nodes_in_k_group` | FAIL | SKIP | type error: attribute access '.next' is not supported as an expression; use as a method call type error: attribute access '.next' is not supported as an expression; use as a method call type error: cannot reassign immutable parameter `curr`: add `mut` to the p |
| `0034_find_first_and_last_position_of_element_in_sorted_array` | PASS | PASS | ok |
| `0044_wildcard_matching` | PASS | PASS | ok |
| `0131_palindrome_partitioning` | FAIL | SKIP | type error: cannot reassign immutable parameter `l`: add `mut` to the parameter declaration  |
| `0202_happy_number` | FAIL | SKIP | type error: cannot reassign immutable parameter `n`: add `mut` to the parameter declaration type error: while condition must be bool or collection/string truthiness, got 'int'  |
| `0213_house_robber_ii` | FAIL | SKIP | type error: function 'rob' must return a value of type 'int' on all control-flow paths type error: max() takes 1 or 2 arguments  |
| `0252_meeting_rooms` | FAIL | SKIP | type error: cannot index type 'Any' with 'int' type error: sort() got an unexpected keyword argument 'key'  |
| `0253_meeting_rooms` | FAIL | SKIP | type error: '<' not supported between instances of 'int | None | None' and 'int | None | None'  |
| `0271_encode_and_decode_strings` | FAIL | SKIP | type error: unsupported operand type(s) for : 'int' and 'Result[int, ParseError]'  |
| `0647_palindromic_substrings` | FAIL | SKIP | type error: cannot reassign immutable parameter `l`: add `mut` to the parameter declaration type error: cannot reassign immutable parameter `r`: add `mut` to the parameter declaration  |
| `0665_non_decreasing_array` | FAIL | SKIP | type error: cannot mutate through immutable parameter `nums`: add `mut` to the parameter declaration type error: cannot mutate through immutable parameter `nums`: add `mut` to the parameter declaration  |
| `0680_valid_palindrome_ii` | FAIL | SKIP | type error: cannot reassign immutable parameter `i`: add `mut` to the parameter declaration type error: cannot reassign immutable parameter `j`: add `mut` to the parameter declaration  |
| `0698_partition_to_k_equal_sum_subsets` | FAIL | SKIP | type error: cannot compare 'float' and 'int' with == type error: sort() got an unexpected keyword argument 'reverse'  |
| `0740_delete_and_earn` | FAIL | SKIP | type error: augmented subscript assignment is not supported for type 'Unknown' type error: function 'deleteAndEarn' must return a value of type 'int' on all control-flow paths type error: undefined variable: 'dp' type error: undefined variable: 'store' type er |
| `0946_validate_stack_sequences` | PASS | FAIL | build error: cargo build failed:    Compiling sifr_output v0.1.0 (/private/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr_run_cache_stage_19652_1774915158678602000/sifr_output) warning: unnecessary parentheses around `while` condition   --> src/main.rs:6 |
| `2002_maximum_product_of_the_length_of_two_palindromic_subsequences` | FAIL | SKIP | type error: max() takes 1 or 2 arguments type error: unsupported operand type(s) for <<: 'int' and 'int | None'  |
| `2017_grid_game` | FAIL | SKIP | type error: function 'gridGame' must return a value of type 'int' on all control-flow paths type error: sum() argument must be an iterable with a statically-known element type, got 'list[int] | None' type error: sum() argument must be an iterable with a static |
| `2306_naming_a_company` | FAIL | SKIP | type error: cannot iterate over type 'set[Any] | None' type error: type 'None | set[Any]' has no method 'add'  |
| `2348_number_of_zero_filled_subarrays` | PASS | PASS | ok |
| `2390_removing_stars_from_a_string` | PASS | PASS | ok |
| `0706_design_hashmap` | FAIL | SKIP | type error: type 'MyHashMap' has no field 'map' type error: type 'MyHashMap' has no field 'map' type error: type 'MyHashMap' has no field 'map' type error: type 'MyHashMap' has no field 'map' type error: undefined variable: 'cur' type error: undefined variable |
| `0721_accounts_merge` | FAIL | SKIP | type error: type 'UnionFind' has no field 'par' type error: type 'UnionFind' has no field 'rank' type error: unsupported operand type(s) for : 'list[str | None]' and 'list[str]'  |
| `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree` | FAIL | SKIP | type error: cannot compare 'list[list[int]]' and 'list[list[Any]]' with == type error: cannot index type 'Any' with 'int' type error: for loop tuple target expects iterable elements of tuple type, got 'list[int]' type error: for loop tuple target expects itera |
| `0077_combinations` | PASS | PASS | ok |
| `0098_validate_binary_search_tree` | PASS | FAIL | build error: cargo build failed:    Compiling sifr_output v0.1.0 (/private/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr_run_cache_stage_24375_1774915186659822000/sifr_output) warning: unnecessary parentheses around `return` value   --> src/main.rs:46:1 |
| `0210_course_schedule_ii` | FAIL | SKIP | type error: cannot index type 'Unknown' with 'int' type error: for loop tuple target expects iterable elements of tuple type, got 'list[int]' type error: for-loop iterable must have a statically-known element type, got 'Any'  |
| `0286_walls_and_gates` | FAIL | SKIP | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' type error: cannot mutate through immutable parameter `rooms`: add `mut` to the parameter declaratio |
| `0332_reconstruct_itinerary` | FAIL | SKIP | type error: argument 1 of callable 'dfs': expected 'dict[str, list[str]]', got 'Unknown' type error: for loop tuple target expects iterable elements of tuple type, got 'list[str]' type error: for-loop iterable must have a statically-known element type, got 'Un |
| `0417_pacific_atlantic_water_flow` | FAIL | SKIP | type error: argument 4 of callable 'dfs': expected 'int', got 'int | None' type error: argument 4 of callable 'dfs': expected 'int', got 'int | None' type error: argument 4 of callable 'dfs': expected 'int', got 'int | None' type error: argument 4 of callable  |
| `0752_open_the_lock` | FAIL | SKIP | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[str]' type error: cannot unpack non-tuple type 'T' type error: list element type mismatch: expected 'str', got 'int' type error: undefined variable: 'wheel' type error: undefined variabl |
| `0909_snakes_and_ladders` | FAIL | SKIP | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' type error: cannot mutate through immutable parameter `board`: add `mut` to the parameter declaration type error: cannot unpack non-tuple type 'T' type error: if condition must be  |
| `1239_maximum_length_of_a_concatenated_string_with_unique_characters` | FAIL | SKIP | type error: undefined function: 'Counter' type error: undefined variable: 'c'  |
| `1448_count_good_nodes_in_binary_tree` | FAIL | SKIP | type error: cannot reassign immutable parameter `maxVal`: add `mut` to the parameter declaration  |
| `2092_find_all_people_with_secret` | FAIL | SKIP | type error: cannot iterate over type 'Unknown | None' type error: cannot iterate over type 'list[int] | None' type error: for loop tuple target expects iterable elements of tuple type, got 'list[int]' type error: undefined variable: 'visit' type error: undefin |
| `2101_detonate_the_maximum_bombs` | FAIL | SKIP | type error: cannot index type 'Unknown' with 'int' type error: cannot unpack non-tuple type 'list[int]' type error: cannot unpack non-tuple type 'list[int]' type error: for-loop iterable must have a statically-known element type, got 'Any' type error: undefine |

## Summary

- check pass: 7/36
- run pass: 5/36
- fixtures with failure in check or run: 31
