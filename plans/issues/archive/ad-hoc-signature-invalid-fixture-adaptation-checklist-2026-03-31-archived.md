# Signature Adaptation Fixture Checklist (2026-03-31)

- Source plan: `issues/ad-hoc-signature-invalid-fixture-adaptation-plan-2026-03-31.md`
- Targeted validation artifact: `verification/leetcode/signature_adaptation_targeted_results_20260331.md`
- Full rerun artifact: `verification/leetcode/full_corpus_current_results_20260331_live_after_signature_adaptation.json`

| Fixture | Batch | Signatures Adapted | check | run | Baseline Status | Post-Adapt Status |
|---|---|---|---|---|---|---|
| `0018_4sum` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0025_reverse_nodes_in_k_group` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0034_find_first_and_last_position_of_element_in_sorted_array` | A | yes | PASS | PASS | CHECK_ERROR | PASS |
| `0044_wildcard_matching` | A | yes | PASS | PASS | CHECK_ERROR | PASS |
| `0131_palindrome_partitioning` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0202_happy_number` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0213_house_robber_ii` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0252_meeting_rooms` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0253_meeting_rooms` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0271_encode_and_decode_strings` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0647_palindromic_substrings` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0665_non_decreasing_array` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0680_valid_palindrome_ii` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0698_partition_to_k_equal_sum_subsets` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0740_delete_and_earn` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0946_validate_stack_sequences` | A | yes | PASS | FAIL | CHECK_ERROR | RUN_ERROR |
| `2002_maximum_product_of_the_length_of_two_palindromic_subsequences` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `2017_grid_game` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `2306_naming_a_company` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `2348_number_of_zero_filled_subarrays` | A | yes | PASS | PASS | CHECK_ERROR | PASS |
| `2390_removing_stars_from_a_string` | A | yes | PASS | PASS | CHECK_ERROR | PASS |
| `0706_design_hashmap` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0721_accounts_merge` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree` | A | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0077_combinations` | B | yes | PASS | PASS | CHECK_ERROR | PASS |
| `0098_validate_binary_search_tree` | B | yes | PASS | FAIL | CHECK_ERROR | RUN_ERROR |
| `0210_course_schedule_ii` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0286_walls_and_gates` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0332_reconstruct_itinerary` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0417_pacific_atlantic_water_flow` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0752_open_the_lock` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `0909_snakes_and_ladders` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `1239_maximum_length_of_a_concatenated_string_with_unique_characters` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `1448_count_good_nodes_in_binary_tree` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `2092_find_all_people_with_secret` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |
| `2101_detonate_the_maximum_bombs` | B | yes | FAIL | SKIP | CHECK_ERROR | CHECK_ERROR |

## Totals

- fixtures in scope: 36
- cleared by signature adaptation alone (check+run pass): 5
- residual failures after signature adaptation: 31
