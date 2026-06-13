# Optional/None Closure: Majority Root-Cause Findings

Date: 2026-03-30  
Phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`  
Source artifact: `verification/leetcode/full_corpus_current_results_20260329_live_after_optional_wave9e.json`

## Scope and Method

- Considered only remaining failures in the phase bucket (`CHECK_ERROR` diagnostics containing Optional/union-None signatures such as `int | None`, `str | None`, `Any | None`).
- Remaining phase failures at this checkpoint: `61`.
- Grouped each failing fixture into an exclusive root-cause cluster by first actionable diagnostic shape and full stderr context.

## Majority Finding

Largest single cluster:

- `optional_arithmetic_and_reduction`: `30 / 61` (`49.2%`)

Root cause statement:

- Arithmetic/reduction paths still consume Optional-contaminated values (`T | None`) from index/get/boundary flows without explicit local proof or canonical defaults, causing operator/reduction type failures.

Representative diagnostics:

- `unsupported operand type(s) for +: 'int | None' and 'int'`
- `unsupported operand type(s) for -: 'int' and 'int | None'`
- `ord() argument must be 'str', got 'str | None'`
- `min() takes 1 or 2 arguments` (in Optional-contaminated reduction flows)

Representative fixtures:

- `0064_minimum_path_sum`
- `0105_construct_binary_tree_from_preorder_and_inorder_traversal`
- `0134_gas_station`
- `0150_evaluate_reverse_polish_notation`
- `0221_maximal_square`
- `0516_longest_palindromic_subsequence`

## Secondary High-Impact Cluster

- `mutability_boundary_missing_mut`: `13 / 61` (`21.3%`)

Root cause statement:

- In-place fixture mutations still cross function boundaries without explicit `mut` parameter contracts, generating immutable-parameter mutation diagnostics and blocking downstream typing.

Representative fixtures:

- `0016_3sum_closest`
- `1498_number_of_subsequences_that_satisfy_the_given_sum_condition`
- `1838_frequency_of_the_most_frequent_element`
- `1984_minimum_difference_between_highest_and_lowest_of_k_scores`
- `2616_minimize_the_maximum_difference_of_pairs`

## Distribution Snapshot

- `optional_arithmetic_and_reduction`: `30`
- `mutability_boundary_missing_mut`: `13`
- `optional_argument_or_return_boundary`: `5`
- `optional_comparison_and_membership`: `5`
- `other_optional`: `7`
- `optional_container_write_mismatch`: `1`

## Conclusion

- The majority root cause is **Optional-contaminated arithmetic/reduction over container/index/get flows**.
- A wave strategy that prioritizes this cluster first should remove the largest block of remaining phase failures.
