# LeetCode WS6 Silent-Fallback Remediation

Status: closed
Owner: ad_hoc_leetcode_divergence_closure_followup
Source phase: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`
Source PR: `https://github.com/sifr-lang/sifr/pull/1632`
Merged remediation PR: `https://github.com/sifr-lang/sifr/pull/1635`
Created: 2026-04-25

## Progress

2026-04-25 code remediation:

- Removed silent tuple-projection fallback helpers from `0084_largest_rectangle_in_histogram`.
- Removed silent tuple-projection fallback helpers from `0739_daily_temperatures`.
- Restored `0838_push_dominoes` to canonical `tuple[int, str]` queue entries using `'L'` / `'R'`.
- Removed silent stack/tuple fallback helpers from `0332_reconstruct_itinerary`, `0735_asteroid_collision`, `0895_maximum_frequency_stack`, and `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`.
- Fixed HIR non-empty pop narrowing invalidation order so the `pop` result can use the pre-call non-empty proof while the sequence facts are still invalidated for later statements.
- Fixed codegen for constant-index projection of non-Copy fields from optional tuples by cloning the projected field when the tuple is accessed through `Option.as_ref()`.
- Added `crates/sifr/tests/e2e/pass/optional_tuple_string_projection.sifr`.
- Full corpus rerun artifact: `verification/leetcode/full_corpus_current_results_20260425_ws6_fallback_remediation.json` with `208 PASS`, `203 NO_ORACLE`, and no `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`.
- Merged as PR `https://github.com/sifr-lang/sifr/pull/1635`.

## Problem

The WS6 closure PR correctly restored a clean full-corpus compile/run result, but it did so for 12 legacy fixtures by adding fixture-local helpers that silently substitute sentinel values for impossible `None` paths.

Examples include helpers shaped like:

- `unwrapInt(value: int | None) -> int` returning `0` on `None`
- `unwrapStr(value: str | None) -> str` returning `""` on `None`
- tuple projection helpers returning `0` when a popped tuple or tuple field is `None`
- `nodeValue(node: TreeNode | None) -> int` returning `0` when the node or non-optional value field is treated as nullable

These helpers are not aligned with the project rule against fallback paths. They should be treated as temporary corpus debt introduced by closure pressure, not as an accepted fixture pattern.

## Affected Fixtures

- `audits/leetcode/0084_largest_rectangle_in_histogram.sifr`
- `audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr`
- `audits/leetcode/0232_implement_queue_using_stacks.sifr`
- `audits/leetcode/0332_reconstruct_itinerary.sifr`
- `audits/leetcode/0513_find_bottom_left_tree_value.sifr`
- `audits/leetcode/0735_asteroid_collision.sifr`
- `audits/leetcode/0739_daily_temperatures.sifr`
- `audits/leetcode/0838_push_dominoes.sifr`
- `audits/leetcode/0895_maximum_frequency_stack.sifr`
- `audits/leetcode/1046_last_stone_weight.sifr`
- `audits/leetcode/1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero.sifr`
- `audits/leetcode/1609_even_odd_tree.sifr`

## Root Cause

The closures exposed missing proof rules and ergonomic operations rather than true algorithmic failures:

- non-empty `pop` / `pop(0)` proofs should allow safe use of the popped element when the compiler can prove the collection is non-empty and unmutated between the proof and pop
- constant-index projection from fixed-shape tuples should preserve the tuple element type instead of forcing sentinel projection helpers
- recursive node value reads should not require `0` fallbacks when the node and field are proven present
- string tuple projection currently encourages shape changes such as encoding domino directions as `-1` / `1` instead of preserving the canonical `'L'` / `'R'` representation

## Required Approach

Do not replace these helpers with another silent fallback such as `return 0`, `return ""`, `continue`, or `return []` on an impossible path.

Use one of these approaches instead:

- implement the missing proof rule in the compiler with focused non-LeetCode regressions
- introduce an explicit stdlib/helper API whose return type makes absence impossible only after an explicit proof
- rewrite the fixture so it avoids the nullable operation without changing the canonical public model or algorithm shape
- if a fixture still cannot be expressed safely, move it to an explicit blocker note rather than hiding the gap with a sentinel

## Specific Follow-Ups

1. Add a proof-gated non-empty pop narrowing rule.
   - Covers stack/queue patterns in `0084`, `0232`, `0735`, `0739`, `0838`, `0895`, `1046`, and `1466`.
   - Must invalidate on intervening collection mutation, rebinding, or calls that can alias-mutate the collection.
   - Partially closed for direct non-empty guarded `pop` / `pop(0)` sites in `0084`, `0332`, `0735`, `0739`, `0838`, `0895`, and `1466`; `0232`, `1046`, `0103`, `0513`, and `1609` still need valid-operation precondition, post-pop length decrement, or snapshot-length queue proofs.

2. Add fixed tuple constant-index projection narrowing.
   - Covers tuple stack/queue patterns in `0084`, `0739`, `0838`, and `1466`.
   - Constant in-range tuple projection should produce the declared element type; dynamic tuple indexes can remain optional.
   - Partially closed for `0084`, `0332`, `0739`, `0838`, and `1466`.

3. Close recursive node field projection gaps.
   - Covers `0103`, `0513`, and `1609`.
   - Non-optional fields on a proven node should not need value sentinels.
   - Follow-up from helper extraction: reassess shared `helpers.list_node` accessors
     (`nodeVal`, `nodeNext`, `hasNode`) once cursor/narrowing support is strong enough
     for fixtures to read proven non-`None` nodes directly.

4. Restore `0838_push_dominoes` to a canonical direction representation if compiler support allows it.
   - Prefer `tuple[int, str]` with `'L'` / `'R'` over encoded `-1` / `1`.
   - Closed by the 2026-04-25 codegen and fixture remediation.

5. Reassess `0023_merge_k_sorted_lists`.
   - Current fixture restores the `ListNode` public model and heap ordering, but it heaps values and recursively appends the result chain.
   - A future canonical pass should use heap entries for list heads or an approved owned-node merge helper so the fixture is not conceptually drain/sort/rebuild.
   - Do not regress to a `list[list[int]]` public model.

## Non-Issues From The Same Review

- The LeetCode-local trie helper checking `_terminal[node] is not None` is required by the current list-subscript contract, which intentionally returns optional values.
- `lib/sifr/dsu.sifr` bounding `find` by `len(parent) + 1` is an invariant guard against corrupted parent chains; it should not be copied as generic corpus style, but it is not the same as substituting a user-visible sentinel result for an impossible `None`.

## Exit Criteria

- The 12 affected fixtures no longer contain silent sentinel helpers for impossible `None` paths.
- `0838_push_dominoes` either preserves canonical `'L'` / `'R'` queue state or has a specific compiler/codegen blocker recorded.
- `0023_merge_k_sorted_lists` has either a canonical owned-node merge implementation or a specific accepted blocker/design note.
- Targeted `check` and `run` pass for all affected fixtures.
- Full LeetCode corpus run remains free of `CHECK_ERROR`, `RUN_ERROR`, and `TIMEOUT`.
- `scripts/run_all_tests.sh --profile quick` passes.
