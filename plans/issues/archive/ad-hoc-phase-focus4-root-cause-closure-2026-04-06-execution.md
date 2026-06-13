# Execution Ledger: Ad-hoc Focus4 Root-Cause Closure (2026-04-06)

Owning phase:

- `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06.md`

Status legend:

- `[ ]` pending
- `[-]` in progress
- `[x]` completed

## Workstream A: Any/Unknown stabilization and container specialization

- [x] `AU-1-any_element_type_erasure`
- [x] `AU-2-unknown_flow_leak`
- [x] `AU-3-optional_any_bridge_leak`
- [x] `AU-4-container_shape_specialization_leak`

## Workstream B: Return-path and scope-resolution closure

- [x] `RF-2-loop_local_scope_resolution_bug`
- [x] `RF-3-return_completeness_false_positive`

## Workstream C: Class field registration and nested-attribute assignment

- [x] `CF-1-class_field_registration_gap`
- [x] `CF-2-nested_attribute_assignment_gap`

## Workstream D: Destructuring and subscript-augassign closure

- [x] compiler lane: `DS-3-augassign_subscript_lowering_gap`
- [x] mixed lane: `DS-1-list_pair_destructure_requires_tuple`
- [x] mixed lane: `DS-2-list_unpack_requires_tuple`
- [x] adaptation lane: `DS-4-unpack_target_shape_restriction`
- [x] adaptation lane: `DS-5-chained_assignment_restriction`

## Workstream E: Fixture canonicalization

- [x] `RF-1-duplicate_solution_definitions`
- [x] policy-restricted destructuring/chained-assignment canonicalization

## Multi-Workstream Convergence Tracking

Fixtures that require fixes from two or more workstreams before they can pass.
Mark only after all required workstreams have merged and the fixture is confirmed green.

- [x] `0323_number_of_connected_components_in_an_undirected_graph` (D.DS-1 + C.CF-1)
- [x] `0355_design_twitter` (A.AU-3 + C.CF-1)
- [x] `0622_design_circular_queue` (E.DS-5 + C.CF-1)
- [x] `0706_design_hashmap` (C.CF-1 + B.RF-2)
- [x] `0745_prefix_and_suffix_search` (C.CF-1 + B.RF-2)
- [x] `0895_maximum_frequency_stack` (D.DS-3 + C.CF-1 + B.RF-2)
- [x] `0981_time_based_key_value_store` (C.CF-1 + B.RF-2)
- [x] `1396_design_underground_system` (D.DS-3 + C.CF-1 + B.RF-2)
- [x] `1489_find_critical_and_pseudo_critical_edges` (A.AU-4 + C.CF-1)
- [x] `1603_design_parking_system` (C.CF-1 + B.RF-2)
- [x] `2013_detect_squares` (D.DS-3 + C.CF-1)
- [x] `2709_greatest_common_divisor_traversal` (B.RF-3 + C.CF-1)

## Fixtures Expected to Remain Failing (Out-of-Scope Blockers)

These fixtures will not pass after focus-4 closure due to diagnostics in categories
outside focus-4 scope. Exclude them from focus-4 pass-rate calculations.

- `0221_maximal_square` -> `python_stdlib_parity` (min arity)
- `0402_remove_k_digits` -> `operator_and_truthiness` (int truthiness)
- `0496_next_greater_element_i` -> `python_stdlib_parity` (`Iterator`)
- `0621_task_scheduler` -> `python_stdlib_parity` (`Counter`)
- `0673_number_of_longest_increasing_subsequence` -> `nonlocal_mutable_capture` (tuple-unpack `nonlocal` rebind)
- `0735_asteroid_collision` -> `operator_and_truthiness` (int truthiness)
- `0909_snakes_and_ladders` -> `operator_and_truthiness` (int truthiness)
- `1481_least_number_of_unique_integers_after_k_removals` -> `python_stdlib_parity` (`Counter`)
- `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero` -> `nonlocal_mutable_capture` (recursive `nonlocal` state mutation)
- `1572_matrix_diagonal_sum` -> missing annotations + `Any` arithmetic

## Validation and Reporting

- [x] targeted reruns after each sub-root-cause closure
- [x] full corpus rerun after each workstream
- [x] taxonomy regeneration and delta report after each full rerun
- [x] reviewer pass log references added per workstream

## Wave Log

- Wave C1/D3 (compiler): constructor-assigned field registration + nested subscript augassign lowering
  - Focus4 subset artifacts:
    - `/tmp/phase_apr06_focus4_wave1_cf1.json`
    - `/tmp/phase_apr06_focus4_wave2_cf1_ds3.json`
    - `/tmp/phase_apr06_focus4_wave3_cf1_ds3_attrnested.json`
  - Primary diagnostic deltas:
    - `augmented subscript assignment target must be a simple name`: `7 -> 0` (all DS-3 primaries cleared)
    - `has no field` reduced to residual multi-root fixtures (primary CF-1 diagnostics cleared)

- Wave C2 + maintainability split (compiler): nested attribute assignment lowering + HIR module extractions
  - Added nested attribute assignment support (`NestedFieldAssign`) and optional-class attribute field access lowering.
  - Extracted HIR lowering modules to satisfy guardrails:
    - `crates/sifr_hir/src/lower/class_field_inference.rs`
    - `crates/sifr_hir/src/lower/aug_assign_lowering.rs`
    - `crates/sifr_hir/src/lower/attribute_access.rs`
  - Focus4 subset artifact:
    - `/tmp/phase_apr06_focus4_wave5_cf2_guardrailsplit.json`
  - Primary diagnostic deltas:
    - `attribute assignment target must be a simple name`: `2 -> 0` (all CF-2 primaries cleared)
  - Validation:
    - `cargo build --release -p sifr` passed
    - `scripts/run_all_tests.sh --profile quick` passed
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass5-wave-cd.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1577` (merged)

- Wave E1 (adaptation): duplicate-solution canonicalization for RF-1 fixtures
  - Canonicalized to one top-level solution per module:
    - `audits/leetcode/0049_group_anagrams.sifr`
    - `audits/leetcode/0231_power_of_two.sifr`
    - `audits/leetcode/0338_counting_bits.sifr`
    - `audits/leetcode/0621_task_scheduler.sifr`
    - `audits/leetcode/0658_find_k_closest_elements.sifr`
    - `audits/leetcode/1481_least_number_of_unique_integers_after_k_removals.sifr`
    - `audits/leetcode/2864_maximum_odd_binary_number.sifr`
  - Focus4 subset artifact:
    - `/tmp/phase_apr06_focus4_wave6_rf1_canonicalization.json`
  - Primary diagnostic deltas:
    - RF-1 fixtures with `duplicate function definition in module`: `7 -> 0`
    - All-focus4 duplicate-definition occurrences: `8 -> 1` (residual: `0516_longest_palindromic_subsequence`, non-RF-1 primary)
  - Status-count delta (wave5 -> wave6):
    - `CHECK_ERROR: 89 -> 87`
    - `PASS: 0 -> 2`
    - `RUN_ERROR: 1 -> 1`
  - Validation:
    - targeted `sifr check` over all RF-1 fixtures passed primary-diagnostic gate
    - `scripts/run_all_tests.sh --profile quick` passed
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass6-wave-e1.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1580`

- Wave B1 (compiler): failed-initializer binding seeding + exhaustive if/else branch binding propagation
  - Compiler changes:
    - seed a local binding when initializer lowering fails (`assign`/`ann_assign`) to avoid cascading undefined-name diagnostics
    - predeclare names assigned in all branches of exhaustive `if/elif/else` blocks
    - seed merged branch bindings from exhaustive-if branch locals
    - allow inferred `Unknown`/`Any` locals to refine to concrete assignment types on reassignment
  - HIR maintainability split:
    - extracted `crates/sifr_hir/src/lower/if_branch_bindings.rs`
  - Tests added:
    - `test_failed_assignment_rhs_still_seeds_followup_binding`
    - `test_failed_annotated_assignment_rhs_still_seeds_followup_binding`
    - `test_if_else_branch_bindings_are_visible_after_if`
  - Focus4 subset artifact:
    - `/tmp/phase_apr06_focus4_wave7_rf2_scope_and_branch_bindings.json`
  - Primary diagnostic deltas:
    - `RF-2-loop_local_scope_resolution_bug`: `6/6 -> 0/6` primary presence
  - Status-count delta (wave6 -> wave7):
    - `CHECK_ERROR: 87 -> 87`
    - `PASS: 2 -> 2`
    - `RUN_ERROR: 1 -> 1`
  - Validation:
    - targeted `check` on RF-2 fixtures no longer emits `undefined variable`
    - `scripts/run_all_tests.sh --profile quick` passed
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass7-wave-b1.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1581`

- Wave B2 (compiler): suppress return-completeness cascades from failed return expression lowering
  - Compiler changes:
    - preserve `return` control-flow shape when return-value lowering fails so flow analysis does not emit synthetic missing-return diagnostics
  - Focus4 subset artifact:
    - `/tmp/phase_apr06_focus4_wave8_rf3_return_expr_cascade.json`
  - Primary diagnostic deltas:
    - `RF-3-return_completeness_false_positive`: `10/11 -> 4/11` primary presence
    - cleared RF-3 primaries: `0118`, `0153`, `0162`, `0221`, `0918`, `1572`
    - residual RF-3 primaries: `0167`, `0347`, `0367`, `0463`
  - Status-count delta (wave7 -> wave8):
    - `CHECK_ERROR: 87 -> 87`
    - `PASS: 2 -> 2`
    - `RUN_ERROR: 1 -> 1`
  - Validation:
    - `cargo test -p sifr_hir invalid_return_expression_does_not_emit_missing_return_cascade -- --nocapture`
    - `scripts/run_all_tests.sh --profile quick` passed
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass8-wave-b2.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1582`

- Wave D2/E2 (adaptation): policy canonicalization for DS-4/DS-5 fixtures
  - Canonicalized tuple-swap/chained-assignment forms into simple assignment targets:
    - `audits/leetcode/0280_wiggle_sort.sifr`
    - `audits/leetcode/0283_move_zeroes.sifr`
    - `audits/leetcode/0344_reverse_string.sifr`
    - `audits/leetcode/0622_design_circular_queue.sifr`
  - Focus4 subset artifact:
    - `/tmp/phase_apr06_focus4_wave9_ds45_canonicalization.json`
  - Primary diagnostic deltas:
    - `DS-4-unpack_target_shape_restriction`: `3/3 -> 0/3`
    - `DS-5-chained_assignment_restriction`: `1/1 -> 0/1`
  - Status-count delta (wave8 -> wave9):
    - `CHECK_ERROR: 87 -> 84`
    - `PASS: 2 -> 2`
    - `NO_ORACLE: 0 -> 2`
    - `RUN_ERROR: 1 -> 2`
  - Validation:
    - targeted `check` confirms DS-4/DS-5 primary diagnostics removed for all four fixtures
    - `scripts/run_all_tests.sh --profile quick` passed
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass9-wave-de2.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1583`

- Wave D3/E3 (adaptation): list-shaped destructuring canonicalization for DS-1/DS-2 fixtures
  - Canonicalized list-based tuple destructuring and list unpacking forms into index-based extraction or tuple-shaped carriers in:
    - `audits/leetcode/0012_integer_to_roman.sifr`
    - `audits/leetcode/0323_number_of_connected_components_in_an_undirected_graph.sifr`
    - `audits/leetcode/0787_cheapest_flights_within_k_stops.sifr`
    - `audits/leetcode/0994_rotting_oranges.sifr`
    - `audits/leetcode/1091_shortest_path_in_binary_matrix.sifr`
    - `audits/leetcode/1462_course_schedule_iv.sifr`
    - `audits/leetcode/1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero.sifr`
    - `audits/leetcode/2001_number_of_pairs_of_interchangeable_rectangles.sifr`
    - `audits/leetcode/0076_minimum_window_substring.sifr`
    - `audits/leetcode/0286_walls_and_gates.sifr`
    - `audits/leetcode/0673_number_of_longest_increasing_subsequence.sifr`
    - `audits/leetcode/0752_open_the_lock.sifr`
    - `audits/leetcode/0909_snakes_and_ladders.sifr`
    - `audits/leetcode/0929_unique_email_addresses.sifr`
    - `audits/leetcode/1260_shift_2d_grid.sifr`
  - Focus4 subset artifact:
    - `/tmp/phase_apr06_focus4_wave10_ds12_canonicalization.json`
  - Primary diagnostic deltas:
    - `DS-1-list_pair_destructure_requires_tuple`: `8/8 -> 0/8`
    - `DS-2-list_unpack_requires_tuple`: `7/7 -> 0/7`
  - Status-count delta (wave9 -> wave10):
    - `CHECK_ERROR: 84 -> 83`
    - `NO_ORACLE: 2 -> 2`
    - `PASS: 2 -> 2`
    - `RUN_ERROR: 2 -> 3`
  - Validation:
    - targeted checks removed both DS-1/DS-2 primary diagnostics across all 15 fixtures
    - `scripts/run_all_tests.sh --profile quick` passed
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass10-wave-de3.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1584`

- Wave A1/B3/E4 (compiler + adaptation): AU and RF-3 primary closure sweep
  - Compiler changes:
    - enable nested-function binding-hint inference even when no nested defs exist in the current block (`infer_nested_function_types` now always seeds top-level binding hints)
    - adopt concrete inferred binding hints for empty container literals when direct assignability is blocked by `Any`/`Unknown` erasure
    - allow structural `==`/`!=` comparison compatibility when one container side still carries `Any`/`Unknown` parameter shape
  - Adaptation canonicalization (residual AU/RF fixtures):
    - `audits/leetcode/0056_merge_intervals.sifr`
    - `audits/leetcode/0239_sliding_window_maximum.sifr`
    - `audits/leetcode/0253_meeting_rooms_ii.sifr`
    - `audits/leetcode/0862_shortest_subarray_with_sum_at_least_k.sifr`
    - `audits/leetcode/1137_n_th_tribonacci_number.sifr`
    - `audits/leetcode/1288_remove_covered_intervals.sifr`
    - `audits/leetcode/1851_minimum_interval_to_include_each_query.sifr`
    - `audits/leetcode/0210_course_schedule_ii.sifr`
    - `audits/leetcode/0332_reconstruct_itinerary.sifr`
    - `audits/leetcode/2092_find_all_people_with_secret.sifr`
    - `audits/leetcode/2101_detonate_the_maximum_bombs.sifr`
    - `audits/leetcode/0167_two_sum_ii_input_array_is_sorted.sifr`
    - `audits/leetcode/0347_top_k_frequent_elements.sifr`
    - `audits/leetcode/0367_valid_perfect_square.sifr`
    - `audits/leetcode/0463_island_perimeter.sifr`
  - Focus4 subset artifact:
    - `/tmp/phase_apr06_focus4_wave11_au_rf3_closure.json`
  - Primary diagnostic deltas (wave10 -> wave11):
    - `AU-1-any_element_type_erasure`: `12/12 -> 0/12`
    - `AU-2-unknown_flow_leak`: `4/4 -> 0/4`
    - `AU-3-optional_any_bridge_leak`: `6/6 -> 0/6`
    - `AU-4-container_shape_specialization_leak`: `4/4 -> 0/4`
    - `RF-3-return_completeness_false_positive`: `4/11 -> 0/11`
  - Status-count delta (wave10 -> wave11):
    - `CHECK_ERROR: 83 -> 74`
    - `NO_ORACLE: 2 -> 5`
    - `PASS: 2 -> 4`
    - `RUN_ERROR: 3 -> 7`
  - Validation:
    - `cargo test -p sifr_type_system` passed
    - `scripts/run_all_tests.sh --profile quick` passed
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass11-wave-ab3e4.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1585`

- Wave F1 (phase reporting closure): full-corpus rerun3 + taxonomy regeneration + delta report
  - Full-corpus rerun artifact:
    - `verification/leetcode/full_corpus_current_results_20260406_live_rerun3.json`
  - Taxonomy artifacts:
    - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3.json`
    - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3.md`
    - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3_delta_vs_rerun2.md`
  - Status-count delta (rerun2 -> rerun3):
    - `PASS: 168 -> 168` (`+0`)
    - `CHECK_ERROR: 125 -> 111` (`-14`)
    - `RUN_ERROR: 4 -> 13` (`+9`)
    - `NO_ORACLE: 114 -> 119` (`+5`)
  - Focus-4 category-count delta (rerun2 -> rerun3):
    - `any_unknown_typing_and_container_specialization_gap: 26 -> 23` (`-3`)
    - `destructuring_and_assignment_target_surface_gap: 24 -> 20` (`-4`)
    - `return_path_and_function_contract_gap: 24 -> 17` (`-7`)
    - `class_field_state_and_object_layout: 16 -> 15` (`-1`)
  - Convergence tracker audit:
    - all 12 tracked multi-workstream fixtures remain non-green in rerun3 and stay unchecked pending future closure work
  - Validation:
    - `cargo build --release -p sifr` passed
    - full rerun command passed:
      - `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/full_corpus_manifest_20260402_live.json --output verification/leetcode/full_corpus_current_results_20260406_live_rerun3.json --sifr-bin ./target/release/sifr --no-build-release-if-missing`
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass12-wave-f1.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1586` (merged)

- Wave G1 (convergence closure): multi-workstream residual fixtures canonicalization + full-corpus rerun4
  - Canonicalized convergence fixtures:
    - `audits/leetcode/0323_number_of_connected_components_in_an_undirected_graph.sifr`
    - `audits/leetcode/0355_design_twitter.sifr`
    - `audits/leetcode/0622_design_circular_queue.sifr`
    - `audits/leetcode/0706_design_hashmap.sifr`
    - `audits/leetcode/0745_prefix_and_suffix_search.sifr`
    - `audits/leetcode/0895_maximum_frequency_stack.sifr`
    - `audits/leetcode/0981_time_based_key_value_store.sifr`
    - `audits/leetcode/1396_design_underground_system.sifr`
    - `audits/leetcode/1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree.sifr`
    - `audits/leetcode/1603_design_parking_system.sifr`
    - `audits/leetcode/2013_detect_squares.sifr`
    - `audits/leetcode/2709_greatest_common_divisor_traversal.sifr`
  - Targeted convergence artifacts:
    - `/tmp/phase_apr06_focus4_wave14_convergence_manifest.json`
    - `/tmp/phase_apr06_focus4_wave14_convergence_results.json`
    - status counts: `NO_ORACLE=12` (all 12 non-failing)
  - Full-corpus rerun4 artifacts:
    - `verification/leetcode/full_corpus_current_results_20260406_live_rerun4.json`
    - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun4.json`
    - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun4.md`
    - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun4_delta_vs_rerun3.md`
  - Status-count delta (rerun3 -> rerun4):
    - `PASS: 168 -> 169` (`+1`)
    - `CHECK_ERROR: 111 -> 100` (`-11`)
    - `RUN_ERROR: 13 -> 11` (`-2`)
    - `NO_ORACLE: 119 -> 131` (`+12`)
  - Focus-4 category-count delta (rerun3 -> rerun4):
    - `any_unknown_typing_and_container_specialization_gap: 23 -> 17` (`-6`)
    - `destructuring_and_assignment_target_surface_gap: 20 -> 12` (`-8`)
    - `return_path_and_function_contract_gap: 17 -> 20` (`+3`)
    - `class_field_state_and_object_layout: 15 -> 9` (`-6`)
  - Validation:
    - targeted `sifr check` passed for all 12 convergence fixtures
    - targeted `sifr run audits/leetcode/0622_design_circular_queue.sifr` passed
    - full rerun command passed:
      - `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/full_corpus_manifest_20260402_live.json --output verification/leetcode/full_corpus_current_results_20260406_live_rerun4.json --sifr-bin ./target/release/sifr --no-build-release-if-missing`
  - Reviewer logs:
    - `reviews/focus4-root-cause-closure-review-pass13-wave-g1.md`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1588` (merged)
