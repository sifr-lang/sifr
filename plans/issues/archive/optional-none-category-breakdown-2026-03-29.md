# Optional/None Category Breakdown

Date: 2026-03-29

Context:
- Fresh full-corpus rerun: `PASS=97`, `CHECK_ERROR=290`, `RUN_ERROR=24`
- Largest previously identified category: `Optional/None flow and narrowing gap` (`62` fixtures)
- Architecture decision remains unchanged: Python `nonlocal` mutable capture is unsupported by design

Supporting artifacts:
- Live corpus results: `verification/leetcode/full_corpus_current_results_20260329_live.json`
- Claude pass 1 transcript: `reviews/optional-none-direct-pass1.md`

## Bottom Line

This bucket is not one monolithic bug, but it is also not a flat set of unrelated issues.

The right model is:
- one dominant workstream: path-sensitive Optional/None narrowing
- three additional first-class workstreams: inference cleanup, container element refinement, and recursive/graph/tree optional-boundary typing
- fixture rewrites should be a small residual lane, not a primary strategy

## First-Class Workstreams

### 1. CFG / Path-Sensitive Optional Narrowing

This is the biggest subproblem.

Representative failures:
- `0004_median_of_two_sorted_arrays`: branch merge still sees `int | None`
- `0013_roman_to_integer`: arithmetic still sees `int | None`
- `0287_find_the_duplicate_number`: index expression still sees `int | None`
- `0802_find_eventual_safe_states`: iteration source still sees `list[int] | None`

Symptoms it explains:
- arithmetic/comparison with `T | None`
- index/call/iterate-on-optional failures
- return positions that still carry `T | None` after guards

Expected impact:
- roughly `25-30` fixtures directly
- additional downstream improvement in inference-heavy cases

### 2. Inference Cleanup and Unknown Stabilization

This is separate from narrowing, even though narrowing failures amplify it.

Representative failures:
- `0010_regular_expression_matching`: `Unknown | None`
- `0309_best_time_to_buy_and_sell_stock_with_cooldown`: `Unknown | None`
- `0494_target_sum`, `0518_coin_change_ii`: same pattern

Core issue:
- incomplete inference joins `Unknown` with `None` and never resolves the result back to a concrete type

Expected impact:
- roughly `10-15` fixtures

### 3. Container Element Refinement

This is not just ordinary variable narrowing. The compiler needs element-level refinement and better empty-literal stabilization.

Representative failures:
- `0023_merge_k_sorted_lists`: `list[None | ListNode]` where `list[ListNode]` is required
- `0115_distinct_subsequences`: cache value inference contaminated by `int | None`

Core issue:
- filtering, building, and reusing containers does not refine element unions strongly enough

Expected impact:
- roughly `8-12` fixtures

### 4. Recursive / Graph / Tree Optional-Boundary Typing

This must be treated as its own lane, not as a mere symptom of general narrowing.

Representative failures:
- `0024_swap_nodes_in_pairs`: expected `ListNode`, got `None`
- `0104_maximum_depth_of_binary_tree`: expected `TreeNode`, got `None | TreeNode`
- `0133_clone_graph`: expected `Node`, got `None | Node`
- `0206_reverse_linked_list`: expected `ListNode`, got `None`

Core issue:
- recursive node APIs, base cases, and optional recursive fields are not being typed consistently at function boundaries

Expected impact:
- roughly `8-12` fixtures

### 5. Residual Fixture Canonicalization

This should be a small residual lane only.

Use it only when the raw fixture truly relies on semantics Sifr intentionally rejects. It should not be the default explanation for this bucket.

Expected impact:
- likely `<5` fixtures inside this category

## Correct Fix Order

1. CFG / path-sensitive Optional narrowing
2. Inference cleanup for `Unknown | None` and unstable joins
3. Container element refinement
4. Recursive / graph / tree optional-boundary typing
5. Residual fixture canonicalization

Parallelism:
- `1` and `2` are tightly coupled, but `1` should lead
- `3` can begin once the core narrowing facts are trustworthy
- `4` can proceed in parallel after the recursive-surface owner confirms the intended typing contract

## What Not To Do

- Do not add implicit `Option[T] -> T` coercion
- Do not auto-unwrap `None` at call, return, index, or iteration sites
- Do not adopt Python truthiness as hidden narrowing
- Do not weaken comparison rules just to let `Any | None` flow through
- Do not recommend `nonlocal` or hidden mutable closure state as an escape hatch
- Do not classify this bucket as mostly fixture bugs

## Final Judgment

The earlier claim that this bucket is "mostly one root cause" was directionally useful but too aggressive.

The accurate planning model is:
- dominant root cause: Optional/None narrowing
- plus three independent first-class compiler workstreams:
  - inference cleanup
  - container element refinement
  - recursive optional-boundary typing

That is the decomposition to use if the goal is zero LeetCode failures without weakening Sifr's core principles.
