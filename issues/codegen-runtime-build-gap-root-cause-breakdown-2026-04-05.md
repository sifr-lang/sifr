# Codegen Runtime Build Gap Root Cause Breakdown (2026-04-05)

- Source results: `verification/leetcode/full_corpus_current_results_20260405_live_rerun1.json`
- Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260405_live_rerun1.json`
- Codegen runtime build failures analyzed: `58`
- Breakdown CSV: `verification/leetcode/codegen_runtime_build_gap_breakdown_20260405.csv`

## Root Cause Families
- `recursive_field_access_surface_leaks_to_codegen`: `21`
- `type_contract_mismatch_reaching_codegen`: `17`
- `non_standard_rust_build_failure_without_error_code`: `7`
- `ownership_move_reuse_in_generated_rust`: `6`
- `missing_or_mis_scoped_binding_in_codegen_output`: `3`
- `rust_build_contract_breakage_misc`: `3`
- `truthiness_not_lowering_to_bool_contract`: `1`

## Resolution Lanes
- `both`: `38`
- `compiler_fix`: `20`

## Rust Error Codes
- `E0308`: `34`
- `E0609`: `21`
- `NO_RUST_CODE`: `7`
- `E0382`: `6`
- `E0277`: `4`
- `E0282`: `2`
- `E0596`: `2`
- `E0369`: `1`
- `E0424`: `1`
- `E0425`: `1`
- `E0434`: `1`
- `E0502`: `1`
- `E0599`: `1`
- `E0600`: `1`
- `E0631`: `1`

## Per-Case Mapping
- `0002_add_two_numbers` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0006_zigzag_conversion` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0014_longest_common_prefix` | `E0382` | `ownership_move_reuse_in_generated_rust` | `compiler_fix`
- `0014_longest_common_prefix_v2` | `E0382` | `ownership_move_reuse_in_generated_rust` | `compiler_fix`
- `0019_remove_nth_node_from_end_of_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0020_valid_parentheses` | `E0600` | `truthiness_not_lowering_to_bool_contract` | `compiler_fix`
- `0021_merge_two_sorted_lists` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0025_reverse_nodes_in_k_group` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0046_permutations` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0048_rotate_image` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0051_n_queens` | `E0425` | `missing_or_mis_scoped_binding_in_codegen_output` | `compiler_fix`
- `0061_rotate_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0083_remove_duplicates_from_sorted_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0086_partition_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0092_reverse_linked_list_ii` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0101_symmetric_tree` | `E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0105_construct_binary_tree_from_preorder_and_inorder_traversal` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0106_construct_binary_tree_from_inorder_and_postorder_traversal` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0108_convert_sorted_array_to_binary_search_tree` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0124_binary_tree_maximum_path_sum` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0127_word_ladder` | `E0382` | `ownership_move_reuse_in_generated_rust` | `compiler_fix`
- `0138_copy_list_with_random_pointer` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0141_linked_list_cycle` | `E0277,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0143_reorder_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0147_insertion_sort_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0148_sort_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0160_intersection_of_two_linked_lists` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0189_rotate_array` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0203_remove_linked_list_elements` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0211_design_add_and_search_words_data_structure` | `E0277,E0382` | `ownership_move_reuse_in_generated_rust` | `compiler_fix`
- `0234_palindrome_linked_list` | `E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0304_range_sum_query_2d_immutable` | `E0277,E0282,E0424` | `missing_or_mis_scoped_binding_in_codegen_output` | `compiler_fix`
- `0394_decode_string` | `NO_RUST_CODE` | `non_standard_rust_build_failure_without_error_code` | `compiler_fix`
- `0417_pacific_atlantic_water_flow` | `E0434,E0596` | `missing_or_mis_scoped_binding_in_codegen_output` | `compiler_fix`
- `0435_non_overlapping_intervals` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0450_delete_node_in_a_bst` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0513_find_bottom_left_tree_value` | `NO_RUST_CODE` | `non_standard_rust_build_failure_without_error_code` | `compiler_fix`
- `0567_permutation_in_string` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0572_subtree_of_another_tree` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0617_merge_two_binary_trees` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0662_maximum_width_of_binary_tree` | `NO_RUST_CODE` | `non_standard_rust_build_failure_without_error_code` | `compiler_fix`
- `0669_trim_a_binary_search_tree` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0701_insert_into_a_binary_search_tree` | `E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `0703_kth_largest_element_in_a_stream` | `E0382` | `ownership_move_reuse_in_generated_rust` | `compiler_fix`
- `0729_my_calendar_i` | `E0277,E0596` | `rust_build_contract_breakage_misc` | `compiler_fix`
- `0783_minimum_distance_between_bst_nodes` | `E0369` | `rust_build_contract_breakage_misc` | `compiler_fix`
- `0838_push_dominoes` | `NO_RUST_CODE` | `non_standard_rust_build_failure_without_error_code` | `compiler_fix`
- `0876_middle_of_the_linked_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `0894_all_possible_full_binary_trees` | `E0308,E0599,E0631` | `type_contract_mismatch_reaching_codegen` | `both`
- `0912_sort_an_array` | `E0382` | `ownership_move_reuse_in_generated_rust` | `compiler_fix`
- `1203_sort_items_by_groups_respecting_dependencies` | `E0502` | `rust_build_contract_breakage_misc` | `compiler_fix`
- `1609_even_odd_tree` | `NO_RUST_CODE` | `non_standard_rust_build_failure_without_error_code` | `compiler_fix`
- `1669_merge_in_between_linked_lists` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `1721_swapping_nodes_in_a_linked_list` | `E0308,E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `1958_check_if_move_is_legal` | `E0282,E0308` | `type_contract_mismatch_reaching_codegen` | `both`
- `1968_array_with_elements_not_equal_to_average_of_neighbors` | `NO_RUST_CODE` | `non_standard_rust_build_failure_without_error_code` | `compiler_fix`
- `2130_maximum_twin_sum_of_a_linked_list` | `E0609` | `recursive_field_access_surface_leaks_to_codegen` | `both`
- `2215_find_the_difference_of_two_arrays` | `NO_RUST_CODE` | `non_standard_rust_build_failure_without_error_code` | `compiler_fix`
