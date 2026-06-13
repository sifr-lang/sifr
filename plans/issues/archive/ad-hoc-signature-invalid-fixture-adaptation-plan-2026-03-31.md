# Ad Hoc Signature-Invalid Fixture Adaptation Plan

Date: 2026-03-31
Owning analysis: `issues/any-unknown-container-specialization-root-cause-2026-03-31.md`
Source run: `verification/leetcode/full_corpus_current_results_20260331_live.json`

## Execution Update (2026-03-31)

Status: in progress, signature adaptation pass executed for all scoped fixtures.

Completed artifacts:

- fixture checklist: `issues/ad-hoc-signature-invalid-fixture-adaptation-checklist-2026-03-31.md`
- targeted validation (36 fixtures): `verification/leetcode/signature_adaptation_targeted_results_20260331.md`
- full corpus rerun (411 fixtures): `verification/leetcode/full_corpus_current_results_20260331_live_after_signature_adaptation.json`
- residual recategorization: `issues/ad-hoc-signature-invalid-fixture-adaptation-recategorization-2026-03-31.md`
- reviewer follow-up update: `issues/ad-hoc-signature-invalid-fixture-adaptation-review-fixes-2026-04-02.md`
- post-review targeted validation (36 fixtures): `verification/leetcode/signature_adaptation_targeted_results_20260402_after_review_fixes.md`
- latest targeted loop rerun (36 fixtures): `verification/leetcode/signature_adaptation_targeted_results_20260402_loop.md`

Current result snapshot:

- fixtures in scope: `36`
- cleared by current fixture adaptation work (`check` + `run` pass): `21`
- residual failures requiring further fixture/compiler follow-up: `15`

## Goal

Adapt all LeetCode fixtures that do not have clear explicit function input/output types before treating residual `Any/Unknown` failures as compiler defects.

This plan follows the language decision:

- every function must have explicit input and output types
- this includes nested/local helpers
- this includes class methods and constructors
- no function-signature inference will be added for this bucket

## Scope

First-step adaptation set: `36` fixtures

- `21` fixtures with top-level untyped signatures
- `3` fixtures whose remaining signature defects are class methods/constructors and must still follow explicit class-boundary policy
- `12` fixtures whose remaining signature defects are true nested/local helpers

Important scope note:

- these counts are a **primary-root-cause** split, not a proof that each fixture has only one issue
- a fixture in the top-level/class-boundary set may also contain untyped nested helpers; those helpers must be adapted in the same file edit
- after signature adaptation, some fixtures may expose a second real defect and must be reclassified rather than treated as completed by assumption

## Signature Contract

For this adaptation phase, every named function must have explicit inputs and outputs:

1. top-level functions: explicit parameter types and explicit return type
2. nested/local helpers: explicit parameter types and explicit return type
3. class methods: explicit parameter types and explicit return type
4. constructors: explicit parameter types; `None` returns are inferred by compiler policy
5. `main()` helpers in fixtures: explicit parameter list; `None` return is inferred

Out of scope:

- no signature inference for named functions
- no reliance on contextual typing for named helper signatures
- lambda/callback contextual typing is unchanged and not part of this phase

## Acceptance Criteria

1. Every function in the `36` fixtures has explicit input and output types.
2. Missing-signature failures are removed from those fixtures.
3. A full corpus rerun is performed after the adaptation batch lands.
4. Residual failures from these fixtures are reclassified before any compiler work is planned from them.

## Non-Goals

- no compiler support for inferring missing function signatures
- no broad language expansion
- no mixing signature adaptation with unrelated canonicalization unless required for type clarity

## Work Batches

### batch_a_top_level_and_class_boundaries

Policy:

- add explicit input/output types only
- preserve algorithm shape unless a tiny local rewrite is required to make the signature meaningful
- class methods/constructors remain explicit boundaries, not inference candidates
- if a file in this batch also contains untyped nested helpers, adapt those in the same file edit rather than deferring them

Fixtures:

- `0018_4sum`
- `0025_reverse_nodes_in_k_group`
- `0034_find_first_and_last_position_of_element_in_sorted_array`
- `0044_wildcard_matching`
- `0131_palindrome_partitioning`
- `0202_happy_number`
- `0213_house_robber_ii`
- `0252_meeting_rooms`
- `0253_meeting_rooms`
- `0271_encode_and_decode_strings`
- `0647_palindromic_substrings`
- `0665_non_decreasing_array`
- `0680_valid_palindrome_ii`
- `0698_partition_to_k_equal_sum_subsets`
- `0740_delete_and_earn`
- `0946_validate_stack_sequences`
- `2002_maximum_product_of_the_length_of_two_palindromic_subsequences`
- `2017_grid_game`
- `2306_naming_a_company`
- `2348_number_of_zero_filled_subarrays`
- `2390_removing_stars_from_a_string`

Class-boundary exceptions that were structurally nested in the earlier inventory and must still follow explicit class-boundary policy:

- `0706_design_hashmap`
- `0721_accounts_merge`
- `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`

### batch_b_nested_and_local_helpers

Policy:

- add explicit helper signatures
- do not rely on contextual or recursive signature inference
- if helper signatures expose a second real defect, record that and leave it for post-rerun classification
- this batch owns only files whose remaining signature defects are true nested/local helpers rather than class-boundary methods

Fixtures:

- `0077_combinations`
- `0098_validate_binary_search_tree`
- `0210_course_schedule_ii`
- `0286_walls_and_gates`
- `0332_reconstruct_itinerary`
- `0417_pacific_atlantic_water_flow`
- `0752_open_the_lock`
- `0909_snakes_and_ladders`
- `1239_maximum_length_of_a_concatenated_string_with_unique_characters`
- `1448_count_good_nodes_in_binary_tree`
- `2092_find_all_people_with_secret`
- `2101_detonate_the_maximum_bombs`

## Execution Order

1. Adapt `batch_a_top_level_and_class_boundaries`
2. Adapt `batch_b_nested_and_local_helpers`
3. Rerun the full LeetCode corpus
4. Reclassify the residual failures from these `36` fixtures
5. Only then open compiler work for the remaining typed-boundary `Any/Unknown` defects

Implementation detail:

- after each fixture edit, run targeted `check` and `run`
- if the fixture still fails after signature adaptation, record the new primary failure and continue
- do not mix unrelated canonicalization into this phase unless the signature itself is impossible to express without a tiny structural rewrite

## Validation

Minimum validation for the adaptation phase:

1. targeted `check`/`run` on each changed fixture
2. full corpus rerun against `audits/leetcode`
3. updated count of:
   - fixtures cleared by signature adaptation alone
   - fixtures that now expose a second root cause

Required output artifacts:

1. a concrete fixture checklist artifact for the `36` files with completion state
2. a fresh full-corpus rerun artifact after the adaptation phase
3. a recategorization note listing:
   - fixtures fixed by signature adaptation alone
   - fixtures still failing, with their new primary category

## Expected Output

After this phase, the `Any/Unknown` bucket should be split into:

1. signature-invalid fixtures now adapted away
2. typed-boundary residual compiler defects, primarily:
   - deterministic local container specialization
   - read-site type recovery from specialized containers
   - avoidable `Any` fallback after concrete evidence exists
