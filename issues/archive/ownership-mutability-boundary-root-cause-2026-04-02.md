# Ownership/Mutability Boundary Root-Cause Analysis

Status: execution complete (ownership boundary adaptations landed on 2026-04-02)
Execution evidence:
- `verification/leetcode/ownership_mutability_boundary_targeted_results_20260402_post_fix.md`
- `verification/leetcode/ownership_mutability_boundary_targeted_results_20260402_post_fix.json`
- `verification/leetcode/full_corpus_current_results_20260402_live_after_ownership_boundary_closure.json`
- `verification/leetcode/ownership_mutability_boundary_breakdown_20260402_live_after_closure.json`
- `verification/leetcode/ownership_mutability_boundary_breakdown_20260402_live_after_closure.md`
- `scripts/run_all_tests.sh --profile quick` (pass)
- `scripts/run_all_tests.sh` (pass)

Date: 2026-04-02
Source run: `verification/leetcode/full_corpus_current_results_20260402_live.json`
Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260402_live.json`
Breakdown artifact: `verification/leetcode/ownership_mutability_boundary_breakdown_20260402_live.json`

## Scope

Current bucket from this rerun:

- `47` fixtures in `ownership_and_mutability_boundary`

Note: this rerun result is `47` (not the previously discussed `48`). This report is grounded in the fresh full-corpus execution on 2026-04-02.

## Current Decomposition

1. `30` `immutable_parameter_mutation`
   - diagnostic shape: `cannot mutate through immutable parameter ... add mut`
2. `11` `immutable_parameter_reassignment`
   - diagnostic shape: `cannot reassign immutable parameter ... add mut`
3. `4` `borrowed_parameter_escape_store`
   - diagnostic shape: `cannot store borrowed parameter ... add own or clone`
4. `2` `borrowed_parameter_escape_return`
   - diagnostic shape: `cannot return borrowed parameter ... add own or clone`

Overlap note: sub-bucket assignment is based on first emitted diagnostic. Compound boundary requirements are:

- diagnostic-confirmed (`2`): `0669_trim_a_binary_search_tree`, `0701_insert_into_a_binary_search_tree` (both emit mutability + borrow-escape return diagnostics)
- inferred (`1+`): `0075_sort_colors` is likely `own mut` semantically, but current diagnostics show borrow-escape-return first and do not explicitly emit a mutability diagnostic in this run

High-frequency parameter names in this bucket:

- `nums` (`11`)
- `node` (`4`)
- `s`, `matrix`, `nums1`, `n`, `flowerbed`, `root`, `grid` (`2` each)

## Architectural Ground Truth

This bucket aligns directly with documented language rules:

- `internal_docs/architecture.md:149`
  - parameter reassignment/mutation is explicit only (`mut` / `own mut`)
- `internal_docs/architecture.md:308-324`
  - borrow-by-default parameter model; ownership and mutability are explicit axes
- `internal_docs/architecture.md:338`
  - borrowed move-type params cannot escape by return/store unless explicitly owned or cloned

## Root Cause

The dominant root cause is not compiler unsoundness. It is a surface mismatch between Python-style LeetCode code and explicit Sifr ownership/mutability contracts.

### root_cause_a_explicit_mutability_not_declared (`30 + 11 = 41`)

The source mutates or rebinds parameters without `mut`.

Typical patterns:

- in-place list/matrix mutation on input params
- `sort()`/write-through operations on input collections
- scalar traversal using parameter rebinding (`n`, etc.)

Why it happens:

- LeetCode Python baselines treat parameter rebinding/mutation as implicit
- Sifr intentionally requires explicit mutability at the boundary

Decision:

- adapt fixtures/source to explicit `mut`/`own mut` or local-copy style
- do not loosen language semantics

### root_cause_b_borrowed_escape_requires_ownership (`4 + 2 = 6`)

The source tries to store or return borrowed move-type parameters.

Typical patterns:

- storing node parameters in state/containers
- returning borrowed parameter values directly

Why it happens:

- Python references alias freely; Sifr enforces borrow/ownership boundaries

Decision:

- adapt fixtures to `own`/`own mut` or explicit `.clone()` based on caller contract:
  - use `own`/`own mut` when caller relinquishes value ownership (expected default for LeetCode-style function boundaries)
  - use `.clone()` when caller must retain independent access after the call
- do not add implicit cloning or hidden ownership transfer

## Secondary Error Inventory (From This Rerun)

`33/47` ownership fixtures already emit at least one non-ownership secondary diagnostic in the same run. Representative first secondary diagnostics per fixture:

- `0002_add_two_numbers`: return-type mismatch (`ListNode` vs `None | ListNode`)
- `0005_longest_palindromic_substring`: moved value (`s1`)
- `0016_3sum_closest`: return-type mismatch (`int` vs `float`)
- `0046_permutations`: undefined variable (`n`)
- `0066_plus_one`: non-bool while condition
- `0067_add_binary`: non-bool if condition
- `0075_sort_colors`: return-type mismatch (`None` vs `list[int]`)
- `0086_partition_list`: chained assignment target shape
- `0106_construct_binary_tree_from_inorder_and_postorder_traversal`: helper return type inference failure
- `0141_linked_list_cycle`: unsupported `and` operand types
- `0160_intersection_of_two_linked_lists`: missing `self` annotation
- `0168_excel_sheet_column_title`: unsupported operand (`str` + `Result[str, ValueError]`)
- `0179_largest_number`: undefined function (`cmp_to_key`)
- `0189_rotate_array`: tuple unpacking target shape
- `0191_number_of_1_bits`: non-bool while condition
- `0234_palindrome_linked_list`: unsupported `and` operand types
- `0312_burst_balloons`: undefined variable (`coins`)
- `0435_non_overlapping_intervals`: for-loop tuple target shape
- `0452_minimum_number_of_arrows_to_burst_balloons`: optional-aware min() violation
- `0881_boats_to_save_people`: optional arithmetic mismatch
- `1020_number_of_enclaves`: missing return on all paths
- `1383_maximum_performance_of_a_team`: for-loop tuple target shape
- `1498_number_of_subsequences_that_satisfy_the_given_sum_condition`: optional arithmetic mismatch
- `1700_number_of_students_unable_to_eat_lunch`: undefined variable (`curr_student`)
- `1838_frequency_of_the_most_frequent_element`: optional arithmetic mismatch
- `1888_minimum_number_of_flips_to_make_the_binary_string_alternating`: min() arity mismatch
- `1958_check_if_move_is_legal`: unpack non-tuple
- `1984_minimum_difference_between_highest_and_lowest_of_k_scores`: return-type mismatch (`int` vs `float`)
- `2215_find_the_difference_of_two_arrays`: duplicate function definition
- `2300_successful_pairs_of_spells_and_potions`: optional arithmetic mismatch
- `2402_meeting_rooms_iii`: for-loop tuple target shape
- `2616_minimize_the_maximum_difference_of_pairs`: missing return on all paths
- `2971_find_polygon_with_the_largest_perimeter`: duplicate function definition

Secondary family totals across ownership fixtures: `operator/truthiness` (`20`), `undefined variable` (`19`), `destructuring/assignment target` (`11`), `return contract` (`8`), plus smaller signature/builtin/dup-definition groups.
These family totals are aggregate counts across all secondary diagnostics (not first-secondary only).
The remaining `14/47` ownership fixtures in this run emit only ownership-category diagnostics.

## Language-Level Judgment

For this bucket, the right policy is:

1. Keep explicit ownership/mutability semantics unchanged.
2. Treat current failures in this bucket as adaptation-required by design.
3. Improve diagnostics and migration ergonomics, not semantics.

This bucket should not drive language weakening.

## Compiler Work That Is Still Justified

Quality improvements (not semantic relaxations):

1. Better primary diagnostics
   - when a parameter needs `own mut` (not just `mut`) due to both mutation and escape, suggest the precise convention
2. Better fixer-oriented guidance
   - suggest local-copy rewrite for copy-type scalar rebinding when cleaner than mutating parameter contracts
3. Reduced cascades
   - after emitting a boundary mutability/ownership error, reduce secondary noise from the same root cause

## Execution-Ready Remediation Strategy

1. Adapt this bucket by subcategory workstreams:
   - mutation/reassignment (`41`) split into:
     - `mut`-only adaptation (`39`)
     - compound `own mut` adaptation (`2` confirmed: `0669`, `0701`; plus `1+` inferred candidates such as `0075`)
   - escape-by-store/return (`6`)
2. For each fixture, choose boundary annotation by contract:
   - use `mut` for in-place mutation with no ownership escape
   - use `own mut` when both mutation and ownership escape/return are required
   - use `.clone()` only when preserving caller-side availability is required
3. Apply concrete mutability decision rule:
   - copy-type scalar-rebinding fixtures in this bucket (`left`, `columnNumber`, `k`, `n` x2, `speed`) should prefer `let mut local = param`
   - move-type rebinding fixtures in this bucket (`a`, `s`, `head`, `nums`, `nums1`) should prefer explicit parameter mutability (`mut` / `own mut`) to preserve boundary clarity
   - collection/object in-place edits (`nums`, `matrix`, `intervals`, tree roots) should prefer explicit parameter `mut` (or `own mut` when escaping)
4. Rerun full corpus and reclassify residuals as secondary defects.
   - expectation: at least `6` node/root-style fixtures may newly unmask secondary categories after ownership/mutability adaptation.
   - current baseline already shows `33` ownership fixtures with explicit non-ownership secondary diagnostics, so residual migration should be treated as expected.
5. Escape (`6`) and mutation/reassignment (`41`) streams are independent and can run in parallel.
6. Apply identical adaptation policy to fixture variants when diagnostics are identical (for example `0605_can_place_flowers` and `0605_can_place_flowers_v2`).

## Bottom Line

`47/47` in this bucket are consistent with Sifr core principles for this rerun.
Primary action is fixture/source adaptation plus diagnostic-quality improvements.
No language-semantics broadening is warranted.

## Execution Closure (2026-04-02)

### Completed Work

- Adapted all `47` ownership/mutability boundary fixtures listed in `verification/leetcode/ownership_mutability_boundary_breakdown_20260402_live.json`.
- Applied explicit parameter contracts at the fixture boundary:
  - `mut` for in-place collection/object mutation.
  - `own` for borrowed escape-by-return/store helper boundaries.
  - `own mut` for compound mutate+escape tree-root cases (`0669`, `0701`).
  - local-copy rewrites for copy-type scalar rebinding cases (`left/right`, `columnNumber`, `k`, `n`).
- Fixed the residual four ownership diagnostics found in the first post-fix pass (`0067`, `0088`, `1020`, `1700`).

### Verification Outcome

- Targeted ownership bucket rerun (`47` fixtures):
  - check pass: `12/47`
  - run pass: `7/47`
  - remaining ownership-category diagnostics: `0`
- Full corpus rerun (`411` fixtures):
  - output: `verification/leetcode/full_corpus_current_results_20260402_live_after_ownership_boundary_closure.json`
  - status counts: `PASS=134`, `NO_ORACLE=42`, `RUN_ERROR=13`, `CHECK_ERROR=222`
  - post-closure ownership breakdown (`first emitted diagnostic`): `0`
  - breakdown artifact: `verification/leetcode/ownership_mutability_boundary_breakdown_20260402_live_after_closure.json`
- Interpretation:
  - ownership/mutability boundary failures have been fully reclassified out of this bucket.
  - residual failures are non-ownership secondary categories already predicted by this analysis.

### Local Validation Gates

- `scripts/run_all_tests.sh --profile quick`: pass
- `scripts/run_all_tests.sh`: pass

## Secondary Wave 1 Progress (2026-04-02)

- Scope: non-ownership secondary check failures in the same 47-fixture cohort.
- Artifacts:
  - `verification/leetcode/ownership_mutability_boundary_check_results_20260402_wave1.json`
  - `verification/leetcode/ownership_mutability_boundary_check_results_20260402_wave1.md`
- Wave 1 check-only result:
  - check pass improved from `12/47` to `24/47` (`+12`).
  - ownership-boundary diagnostics remain `0`.
- Wave 1 fixtures that now check-pass:
  - `0075_sort_colors`
  - `0189_rotate_array`
  - `0086_partition_list`
  - `0067_add_binary`
  - `0066_plus_one`
  - `0191_number_of_1_bits`
  - `1020_number_of_enclaves`
  - `1700_number_of_students_unable_to_eat_lunch`
  - `0435_non_overlapping_intervals`
  - `1383_maximum_performance_of_a_team`
  - `2402_meeting_rooms_iii`
  - `1958_check_if_move_is_legal`
- Remaining notable hard cases in this wave include:
  - run-stage failures (`0006`, `0048`, `0669`, `0701`, `1968`)

## Secondary Wave 2 Progress (2026-04-02)

- Scope: remaining non-ownership secondary check failures in the same 47-fixture cohort after wave 1.
- Artifacts:
  - `verification/leetcode/ownership_mutability_boundary_check_results_20260402_wave2.json`
  - `verification/leetcode/ownership_mutability_boundary_check_results_20260402_wave2.md`
- Wave 2 check-only result:
  - check pass improved from `24/47` to `47/47` (`+23`).
  - ownership-boundary diagnostics remain `0`.
  - remaining check failures in this cohort: `0`.
- Wave 2 fixtures that now check-pass:
  - `0002_add_two_numbers`
  - `0005_longest_palindromic_substring`
  - `0016_3sum_closest`
  - `0046_permutations`
  - `0088_merge_sorted_array`
  - `0106_construct_binary_tree_from_inorder_and_postorder_traversal`
  - `0141_linked_list_cycle`
  - `0160_intersection_of_two_linked_lists`
  - `0168_excel_sheet_column_title`
  - `0179_largest_number`
  - `0234_palindrome_linked_list`
  - `0312_burst_balloons`
  - `0452_minimum_number_of_arrows_to_burst_balloons`
  - `0881_boats_to_save_people`
  - `0948_bag_of_tokens`
  - `1498_number_of_subsequences_that_satisfy_the_given_sum_condition`
  - `1838_frequency_of_the_most_frequent_element`
  - `1888_minimum_number_of_flips_to_make_the_binary_string_alternating`
  - `1984_minimum_difference_between_highest_and_lowest_of_k_scores`
  - `2215_find_the_difference_of_two_arrays`
  - `2300_successful_pairs_of_spells_and_potions`
  - `2616_minimize_the_maximum_difference_of_pairs`
  - `2971_find_polygon_with_the_largest_perimeter`
