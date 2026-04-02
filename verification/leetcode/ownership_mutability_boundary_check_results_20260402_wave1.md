# Ownership/Mutability Boundary Check-Only Validation (Wave 1, 2026-04-02)

- check pass: `24/47`
- check fail: `23/47`

## Top Remaining First Diagnostics

- `2`: unsupported operand type(s) for and: 'ListNode' and 'bool'
- `2`: return type mismatch: expected 'int', got 'float'
- `2`: unsupported operand type(s) for +: 'int | None' and 'int | None'
- `1`: return type mismatch: expected 'ListNode', got 'None | ListNode'
- `1`: parameter 'self' in function 'getIntersectionNode' is missing a type annotation
- `1`: list.append() argument type 'int | None' is not compatible with list element type 'int'
- `1`: list subscript assignment value type 'int | None' is not compatible with list element type 'int'
- `1`: argument 1 ('val') of function 'TreeNode': expected 'int', got 'int | None'
- `1`: list subscript assignment value type 'str' is not compatible with list element type 'int'
- `1`: min() with 2 arguments does not accept optional operands; got 'int | None' and 'int | None' (guard or unwrap first)
- `1`: unsupported operand type(s) for +: 'int' and 'int | None'
- `1`: unsupported operand type(s) for -: 'int' and 'int | None'
- `1`: unsupported operand type(s) for *: 'int' and 'int | None'
- `1`: function 'minimizeMax' must return a value of type 'int' on all control-flow paths
- `1`: duplicate function definition in module: 'largestPerimeter'
- `1`: use of moved value: 's1'
- `1`: unsupported operand type(s) for +: 'str' and 'Result[str, ValueError]'
- `1`: undefined variable: 'coins'
- `1`: min() takes 1 or 2 arguments
- `1`: duplicate function definition in module: 'findDifference'

## Per Fixture

| Fixture | Check | Notes |
|---|---|---|
| `0006_zigzag_conversion` | PASS | ok |
| `0075_sort_colors` | PASS | ok |
| `0002_add_two_numbers` | FAIL | type error: return type mismatch: expected 'ListNode', got 'None / ListNode' type error: unsupported operand type(s) for or: 'ListNode' and 'bool' type error: use of moved value: 'dummy' |
| `0141_linked_list_cycle` | FAIL | type error: unsupported operand type(s) for and: 'ListNode' and 'bool' |
| `0160_intersection_of_two_linked_lists` | FAIL | type error: parameter 'self' in function 'getIntersectionNode' is missing a type annotation |
| `0234_palindrome_linked_list` | FAIL | type error: unsupported operand type(s) for and: 'ListNode' and 'bool' type error: use of moved value: 'head' type error: use of moved value: 'head' type error: while condition must be bool or collection/string truthiness, got 'ListNode' |
| `0016_3sum_closest` | FAIL | type error: return type mismatch: expected 'int', got 'float' type error: undefined variable: 'currentGap' type error: unsupported operand type(s) for +: 'int' and 'int / None' type error: unsupported operand type(s) for +: 'int' and 'int / None' |
| `0040_combination_sum_ii` | PASS | ok |
| `0046_permutations` | FAIL | type error: list.append() argument type 'int / None' is not compatible with list element type 'int' type error: list.append() argument type 'int / None' is not compatible with list element type 'int' |
| `0048_rotate_image` | PASS | ok |
| `0066_plus_one` | PASS | ok |
| `0073_set_matrix_zeroes` | PASS | ok |
| `0088_merge_sorted_array` | FAIL | type error: list subscript assignment value type 'int / None' is not compatible with list element type 'int' type error: list subscript assignment value type 'int / None' is not compatible with list element type 'int' type error: unsupported expression type |
| `0106_construct_binary_tree_from_inorder_and_postorder_traversal` | FAIL | type error: argument 1 ('val') of function 'TreeNode': expected 'int', got 'int / None' type error: function 'buildTreeHelper' return type could not be inferred deterministically type error: undefined variable: 'idx' type error: undefined variable: 'idx' type error: undefined variable: 'inorderIndexMap' |
| `0179_largest_number` | FAIL | type error: list subscript assignment value type 'str' is not compatible with list element type 'int' type error: undefined function: 'cmp_to_key' |
| `0274_H_index` | PASS | ok |
| `0435_non_overlapping_intervals` | PASS | ok |
| `0452_minimum_number_of_arrows_to_burst_balloons` | FAIL | type error: min() with 2 arguments does not accept optional operands; got 'int / None' and 'int / None' (guard or unwrap first) |
| `0605_can_place_flowers` | PASS | ok |
| `0605_can_place_flowers_v2` | PASS | ok |
| `0669_trim_a_binary_search_tree` | PASS | ok |
| `0701_insert_into_a_binary_search_tree` | PASS | ok |
| `0881_boats_to_save_people` | FAIL | type error: unsupported operand type(s) for +: 'int / None' and 'int / None' |
| `0948_bag_of_tokens` | FAIL | type error: unsupported operand type(s) for +: 'int' and 'int / None' type error: unsupported operand type(s) for -: 'int' and 'int / None' |
| `1020_number_of_enclaves` | PASS | ok |
| `1254_number_of_closed_islands` | PASS | ok |
| `1498_number_of_subsequences_that_satisfy_the_given_sum_condition` | FAIL | type error: unsupported operand type(s) for +: 'int / None' and 'int / None' |
| `1700_number_of_students_unable_to_eat_lunch` | PASS | ok |
| `1838_frequency_of_the_most_frequent_element` | FAIL | type error: unsupported operand type(s) for -: 'int' and 'int / None' |
| `1958_check_if_move_is_legal` | PASS | ok |
| `1968_array_with_elements_not_equal_to_average_of_neighbors` | PASS | ok |
| `1984_minimum_difference_between_highest_and_lowest_of_k_scores` | FAIL | type error: return type mismatch: expected 'int', got 'float' type error: unsupported operand type(s) for -: 'int' and 'int / None' |
| `2300_successful_pairs_of_spells_and_potions` | FAIL | type error: unsupported operand type(s) for *: 'int' and 'int / None' |
| `2402_meeting_rooms_iii` | PASS | ok |
| `2616_minimize_the_maximum_difference_of_pairs` | FAIL | type error: function 'minimizeMax' must return a value of type 'int' on all control-flow paths type error: undefined variable: 'left' type error: undefined variable: 'left' type error: unsupported operand type(s) for -: 'int / None' and 'int / None' type error: unsupported operand type(s) for -: 'int / None' and 'int / |
| `2971_find_polygon_with_the_largest_perimeter` | FAIL | type error: duplicate function definition in module: 'largestPerimeter' |
| `0005_longest_palindromic_substring` | FAIL | type error: use of moved value: 's1' type error: use of moved value: 's2' |
| `0067_add_binary` | PASS | ok |
| `0086_partition_list` | PASS | ok |
| `0168_excel_sheet_column_title` | FAIL | type error: unsupported operand type(s) for +: 'str' and 'Result[str, ValueError]' |
| `0189_rotate_array` | PASS | ok |
| `0191_number_of_1_bits` | PASS | ok |
| `0263_ugly_number` | PASS | ok |
| `0312_burst_balloons` | FAIL | type error: undefined variable: 'coins' type error: undefined variable: 'coins' type error: unsupported operand type(s) for *: 'int / None' and 'int / None' |
| `1383_maximum_performance_of_a_team` | PASS | ok |
| `1888_minimum_number_of_flips_to_make_the_binary_string_alternating` | FAIL | type error: min() takes 1 or 2 arguments type error: return type mismatch: expected 'int', got 'float' |
| `2215_find_the_difference_of_two_arrays` | FAIL | type error: duplicate function definition in module: 'findDifference' type error: type mismatch: cannot assign 'set[int]' to variable 'nums1' of type 'list[int]' type error: type mismatch: cannot assign 'set[int]' to variable 'nums2' of type 'list[int]' |
