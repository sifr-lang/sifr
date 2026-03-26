# Phase 31 Ad Hoc Follow-up Milestones

Status: complete follow-up plan closure with production-grade review confirmation on 2026-03-26
Source inputs:

- `verification/leetcode/phase31_current_full_results_20260321.json`
- `verification/leetcode/phase31_failure_taxonomy.json`
- `verification/leetcode/phase31_remediation_backlog.json`
- `issues/full-leetcode-corpus-strategy-review.md`
- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`
- `issues/ad-hoc-full-nested-function-pipeline.md`

## Purpose

Convert the remaining Phase 31 LeetCode failures into a complete carry-forward plan that:

- fixes root causes rather than patching individual problems,
- keeps every in-scope LeetCode problem solvable in Sifr,
- separates raw-source incompatibilities from true algorithm support,
- treats cross-cutting language-feature work as explicit prerequisites when those features are broader than the Phase 31 LeetCode closure itself.

Phase 31 itself is complete. This document is the carry-forward plan for the remaining compatibility work it surfaced.

## Current Remaining Surface

- Seed corpus size: `50`
- Current passes: `50`
- Remaining failing raw fixtures: `0`
- Problems expected to be solvable in Sifr after this carry-forward: `50`
- Known raw-source divergence requiring a canonical Sifr rewrite: `0`

Historical snapshot regression relative to the warmed `2026-03-13` rerun (now resolved):

- prior warmed state: `PASS=15`, `CHECK_ERROR=35`, `RUN_ERROR=0`
- current snapshot: `PASS=13`, `CHECK_ERROR=36`, `RUN_ERROR=1`
- changed ids:
  - `0007`: `PASS -> CHECK_ERROR`
  - `0009`: `PASS -> CHECK_ERROR`
  - `0039`: `CHECK_ERROR -> PASS`
  - `0078`: `CHECK_ERROR -> RUN_ERROR`
  - `0151`: `PASS -> CHECK_ERROR`

Post-review-pass current state (`verification/leetcode/phase31_review_pass1_full_results_v2.json`):

- `PASS=50`, `CHECK_ERROR=0`, `RUN_ERROR=0`

## Planning Policy

- Fix root causes, not one-off fixtures.
- Every in-scope LeetCode problem must end up solvable in Sifr.
- If a required fix is already covered by a broader ad hoc language/compiler phase, treat that phase as a dependency and keep this Phase 31 plan focused on LeetCode closure after the dependency is available.
- A raw Python-shaped fixture may remain non-canonical only if it conflicts with an intentional Sifr language guarantee.
- If a raw fixture is non-canonical, add a canonical Sifr variant and count that as the pass target.
- Do not add fallback semantics that weaken ownership, type safety, or parse-safety guarantees.
- Each milestone must end with:
  - updated regression coverage,
  - regenerated compatibility artifacts where counts change,
  - demo evidence for the milestone scope,
  - `scripts/run_all_tests.sh --profile quick`,
  - `scripts/run_all_tests.sh`.

## Language Rot Guardrails

- Type-inference and narrowing milestones must implement general compositional rules, not corpus-driven pattern matches. If a general rule cannot be designed within the milestone scope, escalate the work into a broader feature phase rather than adding recognizers for seed-fixture shapes.
- Post-phase closure milestones may fix bugs in already-landed architecture, but they must not silently extend the feature surface. If a seed-corpus case needs a new supported shape rather than a bug fix, route it back to the owning ad hoc phase with a concrete gap report.
- Every type-inference or narrowing change must include at least one regression test on a shape that does not appear verbatim in the Phase 31 seed corpus. This guards against overfitting the compiler to the current LeetCode fixtures.
- Canonical Sifr fixture adaptation remains the preferred path whenever the raw fixture conflicts with an intentional language guarantee and the needed source form is already supported in Sifr.

## Canonical Sifr Fixture Policy

- Every in-scope LeetCode problem must be solvable in Sifr.
- If a scraped Python fixture conflicts with an intentional Sifr language guarantee, do not weaken the language to accept it verbatim.
- Instead, create a canonical Sifr variant that preserves the same algorithm and changes only the minimum syntax or semantics required by Sifr's contracts.
- Prefer the nearest already-supported Sifr form over broader rewrites.
- Preserve algorithm shape, asymptotic complexity, and test expectations.
- Track the original raw fixture as a `raw-source divergence`, not as an unsupported problem.
- Count the canonical Sifr variant as the pass target for milestone closure.

### Rewrite Rules

- Keep supported constructs if Sifr already supports them.
- Replace only the conflicting surface.
- Prefer explicit safety over implicit fallback behavior.
- Prefer local helper extraction over whole-function rewrites.

### Milestone Planning Rule

- When a failure is caused by a policy mismatch rather than a missing compiler capability, the milestone must target the canonical Sifr form of that problem, and the plan must record the raw fixture as a source divergence rather than treating the problem itself as unsupported.

## Cross-Phase Dependencies Already Landed

These broader feature phases already exist and should no longer be treated as future blockers in the current Phase 31 execution plan. Their remaining impact is closure work on the affected LeetCode cases.

### `dep_recursive_types`

- Source phase: `issues/ad-hoc-full-recursive-type-feature.md`
- Current state:
  - the broader recursive-type feature work is already landed
  - `m31_e` now owns the remaining recursive-tree LeetCode closure work on top of that landed feature
- Phase 31 responsibility now:
  - rerun the affected tree cases
  - add any remaining corpus-specific regression coverage
  - fix the residual closure gaps or send a concrete gap report back to the recursive-type phase if the landed feature is still incomplete for those cases

### `dep_own_mut`

- Source phase: `issues/ad-hoc-own-mut-parameter-convention.md`
- Current state:
  - `own mut` support is already landed
  - `m31_j` now owns only the canonical Sifr adaptation and closure work for `1299`
- Phase 31 responsibility now:
  - rewrite `1299` into canonical Sifr form
  - rerun the case and lock the corpus/demo coverage

### `dep_nested_function_pipeline`

- Source phase: `issues/ad-hoc-full-nested-function-pipeline.md`
- Current state:
  - the broader nested-function phase is already landed
  - `m31_d` now owns only the residual seed-corpus closure work and any explicit scope-expansion decision for unsupported recursive `nonlocal` mutation
- Phase 31 responsibility now:
  - rerun the seed nested-helper cases
  - add any remaining corpus-specific regression coverage
  - close `m31_d` only when the remaining Phase 31 cases are resolved or explicitly re-routed as a scoped feature-boundary decision

## Execution Log

- `2026-03-26`: external review pass 2 production-grade check completed.
  - Source review: `reviews/phase31-ad-hoc-followup-milestones-review-pass-2.md`
  - Execution report: `issues/phase31-followup-review-pass2-production-grade-execution.md`
  - Reviewer verdict: `PASS` (production-grade for phase scope)
  - Closure result:
    - no additional blocking fixes required after pass 1 hardening
- `2026-03-26`: external review pass 1 closure hardening completed.
  - Source review: `reviews/phase31-ad-hoc-followup-milestones-review-pass-1.md`
  - Execution report: `issues/phase31-followup-review-pass1-oracle-upgrade-execution.md`
  - Oracle-mode upgrade artifact: `verification/leetcode/phase31_review_pass1_oracle_upgrade_results.json` (`PASS=14`)
  - Regression-triplet closure artifact: `verification/leetcode/phase31_review_pass1_regression_triplet_results.json` (`PASS=3`)
  - Full-seed closure artifact: `verification/leetcode/phase31_review_pass1_full_results_v2.json` (`PASS=50`)
  - Closure result:
    - previously `NO_ORACLE` cases are now assertion-verified `PASS`
    - residual snapshot-regression cases `0007`, `0009`, and `0151` are closed
- `2026-03-26`: `m31_i_corpus_fixture_canonicalization_for_multi_solution_files` slice 1 completed canonical multi-solution fixture normalization.
  - Execution report: `issues/phase31-m31i-corpus-fixture-canonicalization-execution.md`
  - Demo: `demos/phase31_m31i_multi_solution_fixture_canonicalization_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31i_wave2_canonical_fixture_results.json`
  - Targeted two-case status: `NO_ORACLE=2`
  - Reclassification results:
    - `0215_kth_largest_element_in_an_array`: `CHECK_ERROR -> NO_ORACLE`
    - `1046_last_stone_weight`: `CHECK_ERROR -> NO_ORACLE`
  - Milestone closure:
    - `m31_i_corpus_fixture_canonicalization_for_multi_solution_files` owner scope is now closed
- `2026-03-26`: `m31_k_canonical_sifr_fixture_normalization` slice 1 completed canonical parse-safe fixture closure.
  - Execution report: `issues/phase31-m31k-canonical-sifr-fixture-normalization-execution.md`
  - Demo: `demos/phase31_m31k_canonical_fixture_normalization_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31k_wave3_canonical_fixture_results.json`
  - Targeted one-case status: `PASS=1`
  - Reclassification result:
    - `0043_multiply_strings`: `CHECK_ERROR -> PASS`
  - Milestone closure:
    - `m31_k_canonical_sifr_fixture_normalization` owner scope is now closed
- `2026-03-26`: `m31_j_own_mut_leetcode_closure` slice 1 completed canonical `own mut` closure.
  - Execution report: `issues/phase31-m31j-own-mut-closure-execution.md`
  - Demo: `demos/phase31_m31j_own_mut_closure_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31j_wave3_own_mut_closure_results.json`
  - Targeted one-case status: `PASS=1`
  - Reclassification result:
    - `1299_replace_elements_with_greatest_element_on_right_side`: `CHECK_ERROR -> PASS`
  - Milestone closure:
    - `m31_j_own_mut_leetcode_closure` owner scope is now closed
- `2026-03-26`: `m31_h_local_name_binding_and_shadowing` slice 1 completed canonical local-binding closure.
  - Execution report: `issues/phase31-m31h-local-binding-shadowing-closure-execution.md`
  - Demo: `demos/phase31_m31h_local_binding_shadowing_closure_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31h_wave7_local_name_shadowing_results.json`
  - Targeted two-case status: `PASS=2`
  - Reclassification results:
    - `0015_3sum`: `CHECK_ERROR -> PASS`
    - `0424_longest_repeating_character_replacement`: `CHECK_ERROR -> PASS`
  - Milestone closure:
    - `m31_h_local_name_binding_and_shadowing` owner scope is now closed
- `2026-03-26`: `m31_l_tree_local_state_follow_on_closure` slice 1 completed canonical tree local-state closure.
  - Execution report: `issues/phase31-m31l-tree-local-state-closure-execution.md`
  - Demo: `demos/phase31_m31l_tree_local_state_closure_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31l_wave2_tree_local_state_closure_results.json`
  - Targeted one-case status: `NO_ORACLE=1`
  - Reclassification result:
    - `0110_balanced_binary_tree`: `CHECK_ERROR -> NO_ORACLE`
  - Milestone closure:
    - `m31_l_tree_local_state_follow_on_closure` owner scope is now closed
- `2026-03-26`: `m31_e_recursive_tree_surface_leetcode_closure` slice 1 completed canonical recursive-tree surface closure.
  - Execution report: `issues/phase31-m31e-recursive-tree-canonical-closure-execution.md`
  - Demo: `demos/phase31_m31e_recursive_tree_closure_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31e_wave5_canonical_tree_surface_results.json`
  - Targeted three-case status: `NO_ORACLE=3`
  - Reclassification results:
    - `0100_same_tree`: `CHECK_ERROR -> NO_ORACLE`
    - `0102_binary_tree_level_order_traversal`: `CHECK_ERROR -> NO_ORACLE`
    - `0235_lowest_common_ancestor_of_a_binary_search_tree`: `CHECK_ERROR -> NO_ORACLE`
  - Milestone closure:
    - `m31_e_recursive_tree_surface_leetcode_closure` owner scope is now closed
- `2026-03-26`: `m31_d_nested_function_pipeline_completion` slice 1 completed canonical nested-helper closure across all owner cases.
  - Execution report: `issues/phase31-m31d-nested-helper-canonical-closure-execution.md`
  - Demo: `demos/phase31_m31d_nested_helper_canonical_closure_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31d_wave6_canonical_nested_helper_results.json`
  - Targeted eight-case status: `PASS=6`, `NO_ORACLE=2`
  - Reclassification results:
    - `0017_letter_combinations_of_a_phone_number`: `CHECK_ERROR -> PASS`
    - `0050_powx_n`: `CHECK_ERROR -> PASS`
    - `0052_n_queens_ii`: `CHECK_ERROR -> PASS` (canonical workaround route for recursive `nonlocal`)
    - `0078_subsets`: `RUN_ERROR -> PASS`
    - `0090_subsets_ii`: `CHECK_ERROR -> PASS`
    - `0207_course_schedule`: `CHECK_ERROR -> NO_ORACLE`
    - `0684_redundant_connection`: `CHECK_ERROR -> NO_ORACLE`
    - `0912_sort_an_array`: `CHECK_ERROR -> PASS`
  - Milestone closure:
    - `m31_d_nested_function_pipeline_completion` owner scope is now closed
- `2026-03-26`: `m31_b_destructuring_and_composite_lvalues` slice 2 completed recursive optional-field boxing closure.
  - Execution report: `issues/phase31-m31b-recursive-field-boxing-execution.md`
  - Demo: `demos/phase31_m31b_tuple_attribute_and_canonical_surface_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31b_wave4_recursive_field_boxing_results.json`
  - Targeted five-case status: `NO_ORACLE=3`, `PASS=2`
  - Reclassification result:
    - `0226_invert_binary_tree`: `RUN_ERROR -> NO_ORACLE`
  - Milestone closure:
    - `m31_b_destructuring_and_composite_lvalues` owner scope is now closed
- `2026-03-26`: `m31_b_destructuring_and_composite_lvalues` slice 1 completed tuple-attribute unpack lowering plus canonical closure for non-tree cases.
  - Execution report: `issues/phase31-m31b-tuple-attribute-and-canonical-closure-execution.md`
  - Demo: `demos/phase31_m31b_tuple_attribute_and_canonical_surface_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31b_wave3_tuple_and_canonical_results.json`
  - Targeted five-case status: `NO_ORACLE=2`, `PASS=2`, `RUN_ERROR=1`
  - Reclassification result:
    - `0295_find_median_from_data_stream`: `CHECK_ERROR -> NO_ORACLE`
    - `0703_kth_largest_element_in_a_stream`: `CHECK_ERROR -> NO_ORACLE`
    - `0997_find_the_town_judge`: `CHECK_ERROR -> PASS`
    - `1209_remove_all_adjacent_duplicates_in_string_ii`: `CHECK_ERROR -> PASS`
  - Residual blocker:
    - `0226_invert_binary_tree` now isolated to run-stage boxed optional-tree lowering
- `2026-03-26`: `m31_a_optional_flow_completion` slice 15 completed local validation for canonical encoded-heap closure on IPO and Network Delay Time.
  - Execution report: `issues/phase31-m31a-encoded-heap-closure-execution.md`
  - Demo: `demos/phase31_heap_encoded_priority_queue_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave15_encoded_heap_closure_results.json`
  - Targeted four-case status: `NO_ORACLE=3`, `PASS=1`
  - Reclassification result:
    - `0502_ipo` moved from `CHECK_ERROR` to `NO_ORACLE` (check + run green; oracle comparison not configured in current case mode)
    - `0743_network_delay_time` moved from `CHECK_ERROR` to `NO_ORACLE` (check + run green; oracle comparison not configured in current case mode)
- `2026-03-26`: `m31_a_optional_flow_completion` slice 14 completed local validation for canonical word-ladder queue + bucket normalization.
  - Execution report: `issues/phase31-m31a-canonical-word-ladder-execution.md`
  - Demo: `demos/phase31_word_ladder_canonical_queue_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave14_canonical_word_ladder_results.json`
  - Targeted four-case status: `PASS=1`, `NO_ORACLE=1`, `CHECK_ERROR=2`
  - Reclassification result:
    - `0127_word_ladder` moved from `CHECK_ERROR` to `NO_ORACLE` (check + run green; oracle comparison not configured in current case mode)
- `2026-03-26`: `m31_a_optional_flow_completion` slice 13 completed local validation for canonical coin-change bounded recurrence closure.
  - Execution report: `issues/phase31-m31a-canonical-coin-change-execution.md`
  - Demo: `demos/phase31_coin_change_canonical_bounded_recurrence_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave13_canonical_coin_change_results.json`
  - Targeted four-case status: `PASS=1`, `CHECK_ERROR=3`
  - Confirmed new pass: `0322_coin_change`
- `2026-03-26`: `m31_a_optional_flow_completion` slice 12 completed local validation for fixed-index len-guard closure and canonical source alignment.
  - Execution report: `issues/phase31-m31a-fixed-index-len-guard-execution.md`
  - Demo: `demos/phase31_fixed_index_len_guard_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave12_fixed_index_guard_results.json`
  - Targeted six-case status: `PASS=2`, `CHECK_ERROR=4`
  - Confirmed new passes: `0053_maximum_subarray`, `0746_min_cost_climbing_stairs`
- `2026-03-26`: `m31_a_optional_flow_completion` slice 11 completed local validation for guarded queue-pop narrowing (`pop(0)` and deque guarded pops).
  - Execution report: `issues/phase31-m31a-guarded-queue-pop-execution.md`
  - Demo: `demos/phase31_pop_guard_narrowing_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave11_guarded_queue_pop_results.json`
  - Targeted six-case status: `CHECK_ERROR=6` (count unchanged)
  - Confirmed reclassification signal:
    - `0127_word_ladder` moved further past optional-pop leakage:
      - `None | T` comparison/len errors are now `T` comparison/len follow-ons
- `2026-03-26`: `m31_a_optional_flow_completion` slice 10 completed local validation for guarded `pop`/`popleft` narrowing.
  - Execution report: `issues/phase31-m31a-pop-guard-narrowing-execution.md`
  - Demo: `demos/phase31_pop_guard_narrowing_demo.sifr`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave10_pop_guard_results.json`
  - Targeted six-case status: `CHECK_ERROR=6` (count unchanged)
  - Confirmed reclassification signal:
    - `0127_word_ladder` moved past optional pop leakage (`None | T`) into narrower generic-type and canonical mutability follow-ons (`T` + `mut` requirement), confirming this slice root cause is removed
  - Post-fix hardening:
    - constrained narrowing to zero-arg `list/deque pop/popleft` only
    - aligned codegen for narrowed pop paths (fixes prior `stdlib_configparser` Rust type mismatch)
- `2026-03-26`: `m31_a_optional_flow_completion` slice 9 completed local validation for append-growth sized-list shape propagation under alias-backed index guards.
  - Execution report: `issues/phase31-m31a-append-growth-shape-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave9_append_growth_results.json`
  - Targeted seven-case status: `PASS=1`, `CHECK_ERROR=6`
  - Confirmed new pass: `0238_product_of_array_except_self`
  - Reclassification summary:
    - `0238` moved fully past optional-index arithmetic blockers
    - remaining `m31_a` blockers are now concentrated in `0053`, `0322`, and mutability/canonicalization or non-optional-flow follow-ons (`0127`, `0502`, `0743`, `0746`)
- `2026-03-26`: `m31_a_optional_flow_completion` slice 8 completed local validation for end-pointer while-guard narrowing through `len(...)` aliases.
  - Execution report: `issues/phase31-m31a-end-pointer-len-alias-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave8_end_pointer_alias_results.json`
  - Targeted seven-case status: `CHECK_ERROR=7` (count unchanged), with further narrowing in `0238`
  - Confirmed reclassification signal:
    - `0238_product_of_array_except_self` reduced optional arithmetic failures from 2 to 1 after `i = n - 1` / `while i >= 0` now narrows `nums[i]` via `n = len(nums)` alias-backed end-pointer flow
    - remaining `0238` optional failure is isolated to sized-local `result[i]` growth proof
- `2026-03-26`: `m31_a_optional_flow_completion` slice 7 completed local validation for `len(...)` alias range guard narrowing.
  - Execution report: `issues/phase31-m31a-len-alias-range-guard-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave7_len_alias_results.json`
  - Targeted seven-case status: `CHECK_ERROR=7` (count unchanged), with narrowed error-surface improvement in `0238`
  - Confirmed reclassification signal:
    - `0238_product_of_array_except_self` dropped one optional arithmetic failure after `range(n)` where `n = len(nums)` now proves `nums[i]` is definite
    - remaining `0238` optional failures are now isolated to sized-local `result[i]` flow, not len-alias range inference
- `2026-03-26`: `m31_a_optional_flow_completion` slice 6 completed local validation for dict-membership guarded index narrowing.
  - Execution report: `issues/phase31-m31a-dict-membership-guard-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave6_dict_membership_results.json`
  - Targeted three-case status: `PASS=2`, `RUN_ERROR=1`
  - Confirmed passes: `0523_continuous_subarray_sum`, `0560_subarray_sum_equals_k`
  - Remaining reclassified follow-on: `0001_two_sum` now fails only on raw fixture missing guaranteed return path (canonicalization/closure follow-on), not dict-membership optional narrowing.
- `2026-03-26`: `m31_g_container_literal_specialization_and_state_tracking` completed local validation and targeted corpus rerun.
  - Execution report: `issues/phase31-ad-hoc-followup-milestones-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31g_wave1_results.json`
  - Targeted five-case status: `CHECK_ERROR=4`, `RUN_ERROR=1` (no remaining `dict[Any, Any]` / `Any` arithmetic failures in this slice)
  - Reclassification summary:
    - `0001` moved past the prior `dict[Any, Any]` check failure into run-stage optional/index follow-on closure
    - `0242` moved past `Any` arithmetic into dict comparability / optional-key follow-on closure
    - `0424` moved past dict/`Any` blockers into local-name follow-on closure
    - `0523` and `0560` moved past dict/`Any` blockers into optional-flow follow-on closure
- `2026-03-11`: `m31_c_stdlib_module_parity` slice 1 completed local validation and targeted corpus rerun.
  - Execution report: `issues/phase31-m31c-stdlib-module-parity-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave1_results.json`
  - Targeted six-case status: `PASS=1`, `CHECK_ERROR=5`, `RUN_ERROR=0`
  - Confirmed pass: `0007_reverse_integer`
  - Confirmed reclassification signal: `0502_ipo` moved past missing-`heapq` failure into deeper typing/destructuring blockers
- `2026-03-11`: `m31_c_stdlib_module_parity` slice 2 completed local validation and targeted constructor-surface rerun.
  - Execution report: `issues/phase31-m31c-constructor-compatibility-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave2_results.json`
  - Targeted three-case status: `PASS=1`, `RUN_ERROR=1`, `CHECK_ERROR=1`
  - Confirmed pass: `0217_contains_duplicate`
  - Confirmed reclassification signal: `0127_word_ladder` moved past missing bare `deque(...)` into remaining `defaultdict` / `len(deque)` blockers
  - Confirmed deeper follow-on blocker: `0003_longest_substring_without_repeating_characters` moved past missing `set(...)` into a downstream codegen panic
- `2026-03-11`: `m31_c_stdlib_module_parity` slice 3 completed local validation for `defaultdict(...)` compatibility and `len(deque)`.
  - Execution report: `issues/phase31-m31c-defaultdict-len-compat-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave3_results.json`
  - Primary seeded-case status: `0127` remains `CHECK_ERROR`, but the prior stdlib blockers are removed
  - Confirmed parity pass: `0036_valid_sudoku` now checks and runs with `defaultdict(set)`
  - Confirmed reclassification signal: `0149_max_points_on_a_line` moved past `defaultdict(int)` surface failure into deeper optional/arithmetic typing gaps
- `2026-03-12`: `m31_c_stdlib_module_parity` slice 4 completed local validation for private `heapq` max-heap compatibility.
  - Execution report: `issues/phase31-m31c-private-heapq-max-compat-execution.md`
  - PR: `#1112`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave4_results.json`
  - Targeted six-case status: `PASS=2`, `CHECK_ERROR=3`, `RUN_ERROR=1`
  - Confirmed reclassification signal: `1046_last_stone_weight` moved past missing private `heapq` symbols into deeper annotation / `Any` typing failures
  - Confirmed broader parity probe: `2971_find_polygon_with_the_largest_perimeter` now resolves private `heapq` helpers and fails only on downstream optional arithmetic
- `2026-03-12`: `m31_c_stdlib_module_parity` milestone closed.
  - Closure report: `issues/phase31-m31c-milestone-closure.md`
  - Closure PR: `#1112`
  - Closure basis: all remaining watched-case failures are now downstream codegen/type-system work rather than `stdlib.python_module_surface`
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 1 completed local validation for guarded sequence index narrowing.
  - Execution report: `issues/phase31-m31a-guarded-sequence-index-narrowing-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave1_results.json`
  - Targeted 10-case status: `PASS=3`, `CHECK_ERROR=7`, `RUN_ERROR=0`
  - Confirmed passes: `0014_longest_common_prefix`, `0198_house_robber`, `1768_merge_strings_alternately`
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 2 completed local validation for same-sequence two-pointer `while` guard narrowing.
  - Execution report: `issues/phase31-m31a-two-pointer-while-guard-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave2_results.json`
  - Targeted 10-case status: `PASS=4`, `CHECK_ERROR=6`, `RUN_ERROR=0`
  - Confirmed new pass: `0042_trapping_rain_water`
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 3 completed local validation for canonical sliding-window left-pointer narrowing.
  - Execution report: `issues/phase31-m31a-sliding-window-left-pointer-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave3_results.json`
  - Targeted three-case status: `PASS=2`, `CHECK_ERROR=1`, `RUN_ERROR=0`
  - Confirmed new passes: `0003_longest_substring_without_repeating_characters`, `1456_maximum_number_of_vowels_in_a_substring_of_given_length`
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 4 completed local validation for sentinel-domain normalization on canonical infinity accumulators.
  - Execution report: `issues/phase31-m31a-sentinel-domain-normalization-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave4_results.json`
  - Targeted seeded-case status: `PASS=1`, `CHECK_ERROR=0`, `RUN_ERROR=0`
  - Confirmed new pass: `0209_minimum_size_subarray_sum`
- `2026-03-13`: `m31_a_optional_narrowing_core` slice 5 completed local validation for reverse-range recurrence narrowing over sized local constructions.
  - Execution report: `issues/phase31-m31a-reverse-range-recurrence-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave5_results.json`
  - Targeted four-case status: `PASS=1`, `CHECK_ERROR=3`, `RUN_ERROR=0`
  - Confirmed new pass: `1143_longest_common_subsequence`
  - Stable warmed full-corpus rerun: `verification/leetcode/phase31_current_full_results_after_m31a_wave5_rerun.json`
  - Full-corpus state after slice 5: `PASS=15`, `CHECK_ERROR=35`, `RUN_ERROR=0`

## Recommended Execution Order

1. `m31_g_container_literal_specialization_and_state_tracking`
2. `m31_a_optional_flow_completion`
3. `m31_b_destructuring_and_composite_lvalues`
4. `m31_d_nested_function_pipeline_completion`
5. `m31_e_recursive_tree_surface_leetcode_closure`
6. `m31_l_tree_local_state_follow_on_closure`
7. `m31_h_local_name_binding_and_shadowing`
8. `m31_j_own_mut_leetcode_closure`
9. `m31_k_canonical_sifr_fixture_normalization`
10. `m31_i_corpus_fixture_canonicalization_for_multi_solution_files`

This order assumes the broader dependency phases are already landed and keeps the remaining work focused on current seed-corpus closure rather than re-owning those broader features.

## Milestones

### `m31_g_container_literal_specialization_and_state_tracking`

- Scope:
  - specialize empty container literals from later typed writes and reads
  - remove `Any` leakage through dictionary growth, `.get`, membership, and equality
- Classification:
  - this is a targeted compiler/type-inference feature inside the Phase 31 carry-forward, not a trivial one-off closure patch
- Implementation notes:
  - implement first-write specialization as a general forward-propagation rule in the type system, not as a recognizer for specific dict-usage idioms
  - propagate the specialized key/value shape through subsequent reads, `.get(...)`, membership checks, equality, and other normal container consumers rather than only the exact operations observed in the current seed corpus
  - reject conflicting later writes with deterministic "empty literal type conflict" diagnostics
  - do not add LeetCode-specific or fixture-specific branches to the type checker; the implementation must compose with the existing container type machinery
- Affected ids:
  - `0001`, `0242`, `0424`, `0523`, `0560`
- Definition of done:
  - these five cases move past `dict[Any, Any]` / `Any` arithmetic failures
  - regression coverage locks empty-literal specialization and conflicting-write diagnostics
  - at least one regression case proves the specialization rule on a non-seed shape

### `m31_a_optional_flow_completion`

- Current execution status (`2026-03-26`):
  - guarded sequence indexing, two-pointer `while`, sliding-window left-pointer narrowing, sentinel normalization, and reverse-range recurrence narrowing are already landed
  - dict membership guarded narrowing for keyed dict reads (`key in dict`, `key in dict.keys()`, and `if key not in dict: return`) is landed in slice 6
  - `len(...)` alias flow into `range(...)` bounds is landed in slice 7 (`n = len(seq)` now composes with forward/reverse range guarded indexing)
  - alias-backed end-pointer while guards are landed in slice 8 (`i = n - 1`, `while i >= 0`, `nums[i]`)
  - append-growth shape facts are landed in slice 9 (`for i in range(n): out.append(...)` establishes `len(out) >= n` for guarded indexed reads)
  - guarded `pop`/`popleft` reads now narrow under non-empty flow guards (`while seq:` / truthiness guards), removing optional pop leakage
  - guarded queue-pop narrowing now includes safe `pop(0)` plus deque guarded pop/popleft shapes (slice 11)
  - fixed-index len-guard closure is landed for `len(...) < / <=` false-exit proofs, and canonical fixture alignment landed `0053`/`0746` (slice 12)
  - canonical bounded-recurrence closure is landed for `0322` (slice 13)
  - canonical word-ladder queue/bucket normalization is landed for `0127` (slice 14)
  - canonical encoded-heap closure is landed for `0502` and `0743` (slice 15)
  - owner scope is now closed for this milestone
- Remaining root-cause scope:
  - none inside `m31_a` owner cases
- Implementation notes:
  - implement a general forward-propagation rule for definite in-bounds access; do not add narrow recognizers for individual access patterns
  - derive narrowing from compositional proofs such as prior guards, known bounds, and arithmetic constraints rather than syntax-specific matches on seed-fixture code
  - track range/loop bounds, arithmetic offsets such as `i + 1` and `i + 2`, and first-element access after non-empty proofs
  - keep the existing no-implicit-unwrap rule outside proven-safe flow
- Affected ids:
  - `0502`, `0743` (closed in slice 15)
- Definition of done:
  - remaining owner cases move past `int | None`, `None | str`, and `None | tuple[...]` failures
  - regression coverage exists for guarded queue/heap pops and guarded recurrence indexing
  - at least one regression case proves the narrowing rule on a non-seed shape

### `m31_b_destructuring_and_composite_lvalues`

- Current execution status (`2026-03-26`):
  - tuple-assignment lowering now supports attribute targets (`obj.a, obj.b = ...`) in HIR/codegen
  - canonical closure landed for `0295`, `0703`, `0997`, `1209`
  - recursive optional-field assignment boxing closure landed for `0226`
  - owner scope is now closed
- Scope:
  - support fixed-shape destructuring into locals and attributes
  - support loop destructuring from known two-element items
  - support fixed-shape heterogeneous mutable cells used with subscript mutation
- Affected ids:
  - `0226`, `0295`, `0703`, `0997`, `1209` (closed across slices 1-2)
- Definition of done:
  - these five cases move past destructuring/composite-lvalue failures
  - regression coverage exists for attribute destructuring, loop tuple targets, and fixed-shape subscript augassign

### `m31_d_nested_function_pipeline_completion`

- Current execution status (`2026-03-26`):
  - canonical nested-helper closure landed across all eight owner cases
  - `0052` was closed through the documented canonical workaround route (recursive `nonlocal` avoidance) rather than a scope-expansion of nested recursive `nonlocal`
  - owner scope is now closed
- Scope:
  - builds on the already-landed nested-function phase
  - finish lowering for remaining nested function shapes, including `nonlocal`
  - infer nested helper params/returns for the supported corpus patterns
  - eliminate generic `Any` fallback leakage from nested helper bodies
- Implementation notes:
  - prefer usage-driven inference from nested helper call sites and captured-state operations rather than requiring manual annotations
  - flow argument and return expectations across recursive helpers, backtracking helpers, and captured mutable locals
  - keep this milestone focused on residual bugs in the already-landed nested-function architecture; do not add new supported shapes locally if they were outside the landed phase contract
  - if a remaining seed case needs a genuinely new nested-function shape, send it back to the nested-function phase as a concrete gap report instead of patching it here
  - `0052` is not routine cleanup under the currently landed phase contract; closure for that case requires either a scoped nested-function feature extension for recursive `nonlocal` mutation or a canonical Sifr workaround decision
- Affected ids:
  - `0017`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912` (closed in slice 1)
- Definition of done:
  - these eight cases move past nested-function and generic frontend failures
  - no Phase 31 seed case fails with a generic nested-function frontend error caused by a residual bug in the landed architecture
  - every fix contributing to that result is a closure bug fix rather than a new special-case lowering path

### `m31_e_recursive_tree_surface_leetcode_closure`

- Current execution status (`2026-03-26`):
  - canonical recursive-tree closure landed across all three owner cases
  - owner scope is now closed
- Scope:
  - builds on the already-landed recursive-type phase
  - verify that the landed recursive-type feature fully unblocks the tree LeetCode cases for this corpus
  - add any remaining corpus-specific regression coverage and demos needed for closure
- Affected ids:
  - `0100`, `0102`, `0235` (closed in slice 1)
- Definition of done:
  - these three recursive-tree cases pass in the Phase 31 corpus on top of the landed recursive-type feature
  - any residual tree-case failure is either fixed as a narrow LeetCode closure bug or sent back to the recursive-type phase with a concrete gap report
  - regression coverage exists for the corpus-facing recursive-node behavior exercised by these problems

### `m31_l_tree_local_state_follow_on_closure`

- Current execution status (`2026-03-26`):
  - canonical tree local-state closure landed for `0110`
  - owner scope is now closed
- Scope:
  - close tree-adjacent cases that are no longer primarily blocked on recursive-type support
  - keep local-state, bool-flow, and helper-binding cleanup separate from the recursive-type milestone
- Affected ids:
  - `0110` (closed in slice 1)
- Definition of done:
  - `0110` passes without being treated as evidence of a recursive-type gap
  - regression coverage locks the bool/local-state behavior that blocked the case

### `m31_h_local_name_binding_and_shadowing`

- Current execution status (`2026-03-26`):
  - canonical local binding/shadowing closure landed for `0015` and `0424`
  - owner scope is now closed
- Scope:
  - make local assignment shadow the enclosing function symbol immediately and consistently
  - audit same-block reads/comparisons so they resolve to the local binding
- Follow-on note:
  - `0015` is the primary owner case
  - `0424` should be rechecked here if its current `undefined variable: 'r'` failure remains after `m31_g` removes the `dict[Any, Any]` blocker
- Affected ids:
  - `0015` (closed in slice 1)
- Definition of done:
  - `0015` moves past the `function` vs `int` comparison failure
  - regression coverage locks same-name local shadowing behavior

### `m31_j_own_mut_leetcode_closure`

- Current execution status (`2026-03-26`):
  - canonical `own mut` closure landed for `1299`
  - owner scope is now closed
- Scope:
  - builds on the already-landed `own mut` feature
  - rewrite `1299` into canonical Sifr form using `own mut`
  - verify corpus/demo/regression closure for the LeetCode problem on top of the landed feature
- Affected ids:
  - `1299` (closed in slice 1)
- Definition of done:
  - `1299` is no longer treated as a permanent divergence in the Phase 31 corpus
  - canonical `1299` Sifr source using `own mut` checks, emits, and runs successfully
  - any residual failure is either fixed as a narrow LeetCode closure bug or sent back to the `own mut` phase with a concrete gap report

### `m31_k_canonical_sifr_fixture_normalization`

- Current execution status (`2026-03-26`):
  - canonical parse-safe fixture closure landed for `0043`
  - owner scope is now closed
- Scope:
  - define the corpus rule for raw-source policy mismatches
  - keep the problem in scope while replacing the pass target with a canonical Sifr fixture
  - do not weaken core Sifr guarantees just to accept the raw Python-shaped syntax verbatim
- Initial affected ids:
  - `0043`
- Definition of done:
  - canonical Sifr version of `0043` exists and is counted as the pass target
  - the canonical rewrite changes only the parse-safety-conflicting surface and leaves the algorithm shape intact
  - any remaining compiler/runtime failure after canonicalization is explicitly reclassified into the normal closure milestone that owns it
  - current expectation: the post-canonicalization follow-on for `0043` is `m31_a_optional_flow_completion`
  - corpus docs clearly separate “problem supported” from “raw fixture source-compatible”

### `m31_i_corpus_fixture_canonicalization_for_multi_solution_files`

- Current execution status (`2026-03-26`):
  - canonical multi-solution fixture normalization landed for `0215` and `1046`
  - owner scope is now closed
- Scope:
  - normalize scraped fixtures that contain multiple alternative top-level solutions
  - prefer one canonical typed / lowest-dependency solution
  - do not treat duplicate top-level solution blocks as a language feature requirement
- Affected ids:
  - `0215`, `1046`
- Definition of done:
  - each file is reduced to one canonical solution
  - the milestone owns fixture canonicalization only; it is not the final functional owner of making the case pass
  - any remaining failure after canonicalization is reclassified into the normal closure milestone that owns it
  - current expectation: `0215` and `1046` both fall into `m31_a_optional_flow_completion` after canonicalization

### `m31_c_stdlib_module_parity`

- Status:
  - complete
  - leave closed unless later milestones expose a real new stdlib blocker rather than a corpus artifact or deeper compiler failure

## Raw-Source Divergence List

These are not unsupported LeetCode problems. They are raw source shapes we do not plan to support verbatim if doing so would weaken intentional Sifr guarantees.

### `0043_multiply_strings` (resolved in `m31_k` slice 1)

- Why it is a raw-source divergence:
  - the scraped Python solution relies on unchecked `int(str)` conversion
  - Sifr intentionally keeps parse safety: `int(str)` is `Result[int, ParseError]`
  - weakening that behavior would change the language’s error model, not just fix a compiler bug
- Carry-forward policy:
  - keep the problem in scope
  - add a canonical Sifr rewrite and count that as the pass target (completed)
  - rewrite only the conflicting parse-safety surface and keep the rest of the algorithm as close as possible to the existing supported form
  - document the raw-source incompatibility as a corpus divergence

## Exit Conditions For The Carry-forward Plan

- Every remaining failing problem is assigned to exactly one milestone or to the raw-source divergence list.
- Every in-scope LeetCode problem ends up solvable in Sifr, even if the raw scraped Python source is non-canonical.
- The recursive-type phase and `own mut` phase land before their dependent Phase 31 closure milestones begin.
- `1299` is closed in the corpus after the `own mut` prerequisite, rather than left as a permanent unsupported case.
- Dependency-bearing milestones are sequenced before their dependents.
- Each milestone can be executed as its own PR loop: plan -> implement -> validate -> demo -> PR -> review -> merge.
