# LeetCode 0148 Owned Merge Sort Blocker

Status: open
Owner: ad_hoc_leetcode_divergence_closure
Source phase: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`
Tracked from: WS4 canonical rewrite debt

## Fixture

- `audits/leetcode/0148_sort_list.sifr`

## Required Canonical Shape

`0148_sort_list` should be a linked-list merge sort over `ListNode` chains:

- preserve the `ListNode | None -> ListNode | None` public model,
- split the owned chain into two subchains,
- recursively sort both halves,
- merge sorted chains by moving/relinking owned nodes,
- avoid draining to `list[int]`, calling `sorted`, or rebuilding from a sorted array.

## Current Blocker

The Sifr checker currently rejects the two-list owned merge helper needed by the canonical rewrite. A minimized shape is:

```python
def mergeNodes(own mut left: ListNode, own mut right: ListNode) -> ListNode:
    if left.val <= right.val:
        left_next: ListNode | None = left.next
        if left_next is None:
            left.next = right
            return left
        left.next = mergeNodes(left_next, right)
        return left

    right_next: ListNode | None = right.next
    if right_next is None:
        right.next = left
        return right
    right.next = mergeNodes(left, right_next)
    return right
```

`cargo run -q -p sifr -- check audits/leetcode/0148_sort_list.sifr` reports moved-value errors for `left` / `right` when both owned nodes can be moved in sibling branches. Rewriting the helper to use optional parameters, early returns, or branch-local value reads still leaves the same ownership false positives.

## Closure Rule

Do not replace this with another drain/sort/rebuild workaround. Close this issue only when either:

- owned two-list merge can be expressed and compiled safely, or
- an approved helper/stdlib abstraction provides the same ownership transfer semantics without interior mutability or shared mutable aliases.

## Validation Needed When Fixed

- `python3 audits/leetcode/0148_sort_list.py`
- `cargo run -q -p sifr -- check audits/leetcode/0148_sort_list.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0148_sort_list.sifr`
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json --top 80`
- `scripts/run_all_tests.sh --profile quick`
