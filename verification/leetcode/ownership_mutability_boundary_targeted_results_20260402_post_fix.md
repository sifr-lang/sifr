# Ownership/Mutability Boundary Targeted Validation (Post-Fix, 2026-04-02)

| Fixture | Check | Run | Notes |
|---|---|---|---|
| `0006_zigzag_conversion` | PASS | FAIL | build error: cargo build failed:    Compiling sifr_output v0.1.0 (/private/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr_run_cache_stage_68271_1775094465558407000/sifr_output) warning: unnecessary parentheses around `if` condition  --> src/main.rs:2:8   / 2 /     if ((numRows == (1 as i64)) / |
| `0075_sort_colors` | FAIL | SKIP | type error: tuple unpacking target must be a simple name or attribute type error: tuple unpacking target must be a simple name or attribute |
| `0002_add_two_numbers` | FAIL | SKIP | type error: return type mismatch: expected 'ListNode', got 'None / ListNode' type error: unsupported operand type(s) for or: 'ListNode' and 'bool' type error: use of moved value: 'dummy' |
| `0141_linked_list_cycle` | FAIL | SKIP | type error: unsupported operand type(s) for and: 'ListNode' and 'bool' |
| `0160_intersection_of_two_linked_lists` | FAIL | SKIP | type error: parameter 'self' in function 'getIntersectionNode' is missing a type annotation |
| `0234_palindrome_linked_list` | FAIL | SKIP | type error: unsupported operand type(s) for and: 'ListNode' and 'bool' type error: use of moved value: 'head' type error: use of moved value: 'head' type error: while condition must be bool or collection/string truthiness, got 'ListNode' |
| `0016_3sum_closest` | FAIL | SKIP | type error: return type mismatch: expected 'int', got 'float' type error: undefined variable: 'currentGap' type error: unsupported operand type(s) for +: 'int' and 'int / None' type error: unsupported operand type(s) for +: 'int' and 'int / None' |
| `0040_combination_sum_ii` | PASS | PASS | ok |
| `0046_permutations` | FAIL | SKIP | type error: list.append() argument type 'int / None' is not compatible with list element type 'int' type error: list.append() argument type 'int / None' is not compatible with list element type 'int' |
| `0048_rotate_image` | PASS | FAIL | build error: cargo build failed:    Compiling sifr_output v0.1.0 (/private/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr_run_cache_stage_68512_1775094467173277000/sifr_output) error[E0308]: mismatched types   --> src/main.rs:37:43    / 37 / ...                   *__elem = __nested_assign_valu |
| `0066_plus_one` | FAIL | SKIP | type error: while condition must be bool or collection/string truthiness, got 'int' |
| `0073_set_matrix_zeroes` | PASS | PASS | ok |
| `0088_merge_sorted_array` | FAIL | SKIP | type error: list subscript assignment value type 'int / None' is not compatible with list element type 'int' type error: list subscript assignment value type 'int / None' is not compatible with list element type 'int' type error: unsupported expression type |
| `0106_construct_binary_tree_from_inorder_and_postorder_traversal` | FAIL | SKIP | type error: argument 1 ('val') of function 'TreeNode': expected 'int', got 'int / None' type error: function 'buildTreeHelper' return type could not be inferred deterministically type error: undefined variable: 'idx' type error: undefined variable: 'idx' type error: undefined variable: 'inorderIndex |
| `0179_largest_number` | FAIL | SKIP | type error: list subscript assignment value type 'str' is not compatible with list element type 'int' type error: undefined function: 'cmp_to_key' |
| `0274_H_index` | PASS | PASS | ok |
| `0435_non_overlapping_intervals` | FAIL | SKIP | type error: for loop tuple target expects iterable elements of tuple type, got 'list[int]' |
| `0452_minimum_number_of_arrows_to_burst_balloons` | FAIL | SKIP | type error: min() with 2 arguments does not accept optional operands; got 'int / None' and 'int / None' (guard or unwrap first) |
| `0605_can_place_flowers` | PASS | PASS | ok |
| `0605_can_place_flowers_v2` | PASS | PASS | ok |
| `0669_trim_a_binary_search_tree` | PASS | FAIL | build error: cargo build failed:    Compiling sifr_output v0.1.0 (/private/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr_run_cache_stage_68885_1775094469261641000/sifr_output) warning: unnecessary parentheses around `if` condition   --> src/main.rs:42:8    / 42 /     if (root.val > high) {    |
| `0701_insert_into_a_binary_search_tree` | PASS | FAIL | build error: cargo build failed:    Compiling sifr_output v0.1.0 (/private/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr_run_cache_stage_69000_1775094469648899000/sifr_output) warning: unnecessary parentheses around `if` condition   --> src/main.rs:42:8    / 42 /     if (val > root.val) {     |
| `0881_boats_to_save_people` | FAIL | SKIP | type error: unsupported operand type(s) for +: 'int / None' and 'int / None' |
| `0948_bag_of_tokens` | FAIL | SKIP | type error: unsupported operand type(s) for +: 'int' and 'int / None' type error: unsupported operand type(s) for -: 'int' and 'int / None' |
| `1020_number_of_enclaves` | FAIL | SKIP | type error: function 'numEnclaves' must return a value of type 'int' on all control-flow paths type error: only single-generator generator expressions are supported |
| `1254_number_of_closed_islands` | PASS | PASS | ok |
| `1498_number_of_subsequences_that_satisfy_the_given_sum_condition` | FAIL | SKIP | type error: unsupported operand type(s) for +: 'int / None' and 'int / None' |
| `1700_number_of_students_unable_to_eat_lunch` | FAIL | SKIP | type error: list.append() argument type 'int / None' is not compatible with list element type 'int' |
| `1838_frequency_of_the_most_frequent_element` | FAIL | SKIP | type error: unsupported operand type(s) for -: 'int' and 'int / None' |
| `1958_check_if_move_is_legal` | FAIL | SKIP | type error: cannot unpack non-tuple type 'list[int]' type error: undefined variable: 'dr' type error: undefined variable: 'dr' |
| `1968_array_with_elements_not_equal_to_average_of_neighbors` | PASS | FAIL | [sifr-artifact-cache] namespace=run key=b6d8c46106fc61b3 cache_hit=true workspace=/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr_generated_artifact_cache/run/b6d8c46106fc61b3  thread 'main' (60634206) panicked at src/main.rs:52:5: assertion failed: (format!("{:?}",             rearrangeArray( |
| `1984_minimum_difference_between_highest_and_lowest_of_k_scores` | FAIL | SKIP | type error: return type mismatch: expected 'int', got 'float' type error: unsupported operand type(s) for -: 'int' and 'int / None' |
| `2300_successful_pairs_of_spells_and_potions` | FAIL | SKIP | type error: unsupported operand type(s) for *: 'int' and 'int / None' |
| `2402_meeting_rooms_iii` | FAIL | SKIP | type error: for loop tuple target expects iterable elements of tuple type, got 'list[int]' type error: return type mismatch: expected 'int', got 'int / None' |
| `2616_minimize_the_maximum_difference_of_pairs` | FAIL | SKIP | type error: function 'minimizeMax' must return a value of type 'int' on all control-flow paths type error: undefined variable: 'left' type error: undefined variable: 'left' type error: unsupported operand type(s) for -: 'int / None' and 'int / None' type error: unsupported operand type(s) for -: 'in |
| `2971_find_polygon_with_the_largest_perimeter` | FAIL | SKIP | type error: duplicate function definition in module: 'largestPerimeter' |
| `0005_longest_palindromic_substring` | FAIL | SKIP | type error: use of moved value: 's1' type error: use of moved value: 's2' |
| `0067_add_binary` | FAIL | SKIP | type error: if condition must be bool or collection/string truthiness, got 'int' type error: undefined variable: 'bitA' type error: undefined variable: 'char' type error: undefined variable: 'total' type error: undefined variable: 'total' type error: unsupported operand type(s) for -: 'Result[int, V |
| `0086_partition_list` | FAIL | SKIP | type error: chained assignment targets must be simple names type error: chained assignment targets must be simple names |
| `0168_excel_sheet_column_title` | FAIL | SKIP | type error: unsupported operand type(s) for +: 'str' and 'Result[str, ValueError]' |
| `0189_rotate_array` | FAIL | SKIP | type error: tuple unpacking target must be a simple name or attribute type error: tuple unpacking target must be a simple name or attribute type error: tuple unpacking target must be a simple name or attribute |
| `0191_number_of_1_bits` | FAIL | SKIP | type error: while condition must be bool or collection/string truthiness, got 'int' |
| `0263_ugly_number` | PASS | PASS | ok |
| `0312_burst_balloons` | FAIL | SKIP | type error: undefined variable: 'coins' type error: undefined variable: 'coins' type error: unsupported operand type(s) for *: 'int / None' and 'int / None' |
| `1383_maximum_performance_of_a_team` | FAIL | SKIP | type error: for loop tuple target expects iterable elements of tuple type, got 'list[int]' type error: sort() got an unexpected keyword argument 'reverse' |
| `1888_minimum_number_of_flips_to_make_the_binary_string_alternating` | FAIL | SKIP | type error: min() takes 1 or 2 arguments type error: return type mismatch: expected 'int', got 'float' |
| `2215_find_the_difference_of_two_arrays` | FAIL | SKIP | type error: duplicate function definition in module: 'findDifference' type error: type mismatch: cannot assign 'set[int]' to variable 'nums1' of type 'list[int]' type error: type mismatch: cannot assign 'set[int]' to variable 'nums2' of type 'list[int]' |

## Summary

- check pass: 12/47
- run pass: 7/47
