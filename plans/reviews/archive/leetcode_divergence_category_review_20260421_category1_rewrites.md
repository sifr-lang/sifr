# Category 1 Review — "Should Have Parity, Rewrite Mainly"

Date: 2026-04-21
Source: `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`, Category 1
Evidence: paired fixtures read under `audits/leetcode/`

## Scope

Category 1 lists six fixtures asserted to be parity-debt rewrites rather than verbose Sifr ports:

- `0023_merge_k_sorted_lists`
- `0133_clone_graph`
- `0148_sort_list`
- `0212_word_search_ii`
- `0295_find_median_from_data_stream`
- `0707_design_linked_list`

This review reads each paired fixture and answers: are the six correctly classified, is anything missing, does anything belong in another category, and are the stated reasons precise enough to drive implementation work?

## Per-fixture verification

### `0023_merge_k_sorted_lists` — CORRECT (Category 1)

- Python ([audits/leetcode/0023_merge_k_sorted_lists.py:48](audits/leetcode/0023_merge_k_sorted_lists.py:48)): `mergeKLists(lists: list[ListNode]) -> ListNode`, pairwise-merge linked lists, `O(N log k)`.
- Sifr ([audits/leetcode/0023_merge_k_sorted_lists.sifr:3](audits/leetcode/0023_merge_k_sorted_lists.sifr:3)): `mergeKLists(lists: list[list[int]]) -> list[int]`, flatten + `sort()`, `O(N log N)`.
- Classification is right: the **public input/output type changed** and the pairwise-merge invariant over `k` sorted streams is gone. Stated reason is precise.

### `0133_clone_graph` — CORRECT (Category 1)

- Python ([audits/leetcode/0133_clone_graph.py:48](audits/leetcode/0133_clone_graph.py:48)): `cloneGraph(node: Node) -> Node`, DFS over neighbor references with an old→new identity map.
- Sifr ([audits/leetcode/0133_clone_graph.sifr:3](audits/leetcode/0133_clone_graph.sifr:3)): `cloneGraph(adjacency: list[list[int]]) -> list[list[int]]`, copies rows and sorts each row.
- Classification is right: **public surface changes** from object-graph to adjacency matrix and the identity/aliasing invariant is eliminated. Reason is precise.
- Minor precision edit: note that sorting each row also **mutates the observable output shape** vs. Python's order-preserving neighbor list; a canonical rewrite should drop the extra `row.sort()`.

### `0148_sort_list` — CORRECT (Category 1), with a refined reason

- Python ([audits/leetcode/0148_sort_list.py:39](audits/leetcode/0148_sort_list.py:39)): top-down merge sort on the linked list, `O(n log n)` time and `O(log n)` recursion space with **in-place node relinking**.
- Sifr ([audits/leetcode/0148_sort_list.sifr:72](audits/leetcode/0148_sort_list.sifr:72)): drains the list into `list[int]`, calls `sorted()`, then rebuilds a fresh linked list.
- Public signature is preserved (`ListNode | None -> ListNode | None`), but the algorithm is fully abandoned, including node reuse. The extra `unwrapInt` plus the dead `if True:` guard at [audits/leetcode/0148_sort_list.sifr:84](audits/leetcode/0148_sort_list.sifr:84) are symptoms of Optional-narrowing pressure, but the divergence itself is algorithmic, not cosmetic.
- Keep in Category 1. Refine the reason to: "replaces in-place linked-list merge sort with drain/`sorted()`/rebuild, losing node reuse and the canonical algorithm — even though the public signature still returns a `ListNode | None`."

### `0212_word_search_ii` — CORRECT (Category 1), stronger reason available

- Python ([audits/leetcode/0212_word_search_ii.py:24](audits/leetcode/0212_word_search_ii.py:24)): single DFS over the board driven by a shared Trie with `refs` pruning and `removeWord` on hit; roughly `O(m·n·4^L)` amortized across all words.
- Sifr ([audits/leetcode/0212_word_search_ii.sifr:42](audits/leetcode/0212_word_search_ii.sifr:42)): naive per-word board DFS, `O(W·m·n·4^L)` with no prefix pruning.
- Classification is right. Make the reason sharper: "abandons the Trie/refs/`removeWord` pruning central to the LeetCode canonical solution and reverts to per-word independent search — an asymptotic regression in `W` and a loss of the intended data-structure."

### `0295_find_median_from_data_stream` — CORRECT (Category 1)

- Python ([audits/leetcode/0295_find_median_from_data_stream.py:6](audits/leetcode/0295_find_median_from_data_stream.py:6)): two-heap invariant; `addNum` `O(log n)`, `findMedian` `O(1)`.
- Sifr ([audits/leetcode/0295_find_median_from_data_stream.sifr:9](audits/leetcode/0295_find_median_from_data_stream.sifr:9)): scans to an insertion point, then splices via `left + [num] + right`; `addNum` `O(n)` time and `O(n)` allocation per call, `findMedian` `O(1)`.
- Classification is right and the reason is precise. This one unblocks with a `heap`/`heapq` stdlib primitive (already flagged in Practical Priority Order §4).

### `0707_design_linked_list` — CORRECT (Category 1)

- Python ([audits/leetcode/0707_design_linked_list.py:29](audits/leetcode/0707_design_linked_list.py:29)): doubly-linked list with sentinels, all ops pointer-manipulation.
- Sifr ([audits/leetcode/0707_design_linked_list.sifr:3](audits/leetcode/0707_design_linked_list.sifr:3)): `list[int]`-backed; `addAtHead` is `[val] + self.values` (O(n)), `addAtIndex` splits into two new lists and reconcatenates.
- Classification is unambiguously right — the problem's entire point is the linked-list design. Reason is precise; consider adding: "every op is currently `O(n)` because of whole-list rebuilds (see `addAtHead`, `addAtIndex`, `deleteAtIndex`), not just `O(n)` indexing — a stronger regression than `list`-backed naive array semantics would suggest."

## Missing from Category 1

### `0147_insertion_sort_list` — should MOVE FROM 2a → Category 1

- Python ([audits/leetcode/0147_insertion_sort_list.py:39](audits/leetcode/0147_insertion_sort_list.py:39)): genuine insertion sort via linked-list splicing.
- Sifr ([audits/leetcode/0147_insertion_sort_list.sifr:72](audits/leetcode/0147_insertion_sort_list.sifr:72)): **byte-for-byte the same workaround as `0148`** — drains to a `list[int]`, calls `sorted()`, rebuilds. It does not perform insertion sort at all.
- This is the exact same parity-debt pattern as `0148_sort_list`: public linked-list signature preserved, canonical algorithm abandoned. Its inclusion in Category 2a (recursive node / cursor ergonomics) is misleading — better cursor ergonomics alone will not cause the Sifr version to suddenly become insertion sort; it needs a rewrite to the canonical shape.
- **Concrete edit**: add `0147_insertion_sort_list` to the Category 1 bullet list and to "Practical Priority Order §5 — Explicit parity-debt rewrites". Remove it from Category 2a.

Proposed line in Category 1:

```
- `0147_insertion_sort_list`
```

Proposed Why-bullet:

```
- `0147_insertion_sort_list` replaces linked-list insertion sort with drain/`sorted()`/rebuild; the canonical algorithm is absent despite the preserved `ListNode` signature.
```

## Candidates that should NOT move into Category 1

I checked the most obvious adjacent fixtures; these are correctly kept outside Category 1 even though they are algorithmically divergent:

- **`0297_serialize_and_deserialize_binary_tree`** (currently 2a): Sifr version preserves the DFS serialize/deserialize algorithm. The extra code is a hand-rolled `parseIntToken` (stdlib parity — `int()` from string) plus Optional-narrowing boilerplate around `str` indexing and `list.pop`. Same asymptotics, same shape. Stays in 2a/2b.
- **`0894_all_possible_full_binary_trees`** (currently 2a): Python memoizes subtree results in a dict and **aliases the same child tree across many parents**. Sifr refuses to alias and `cloneTree`s each subtree per combination, changing the asymptotic cost. This is an intentional Sifr ownership boundary (no safe aliased mutable trees), so it belongs in Category 4, not Category 1. Worth flagging as a separate edit to the analysis, but not a Category 1 addition.

## Should anything move OUT of Category 1?

No. All six listed fixtures have either a changed public surface (`0023`, `0133`, `0707`) or a wholesale algorithm/asymptotic replacement (`0148`, `0212`, `0295`). None are saveable via Category 2 ergonomics alone, and none are corpus noise (Category 3/5) — the canonical Python versions are clean and singular. Keep all six.

## Precision of stated reasons

Good enough to plan against for five of six. Suggested refinements:

| Fixture | Current reason | Suggested refinement |
|---|---|---|
| `0023` | "changes the public input model from linked lists to `list[list[int]]`" | Add: and drops the `O(N log k)` pairwise-merge structure |
| `0133` | "adjacency-list copying" | Add: sorts each neighbor row, so even output ordering diverges |
| `0148` | "replaces linked-list merge sort with flatten/sort/rebuild" | Note public signature is preserved; divergence is algorithmic, not surface |
| `0212` | "replaces trie/prefix-pruning with per-word board search" | Add: this is an `O(W)` asymptotic regression, not a style change |
| `0295` | "replaces heap-based updates with sorted-array insertion and changes asymptotic behavior" | Fine — consider spelling out `addNum` `O(log n) → O(n)` |
| `0707` | "replaces linked-list design with array-backed storage and loses the intended operation-cost profile" | Add: current impl rebuilds the whole list on most ops, so even indexed ops are `O(n)` |

None of these refinements change the category assignment; they make the planning scope explicit.

## Concrete edits to the analysis file

1. In Category 1, add `0147_insertion_sort_list` to the fixture list and the "Why" block (same pattern as `0148`).
2. In Category 2a, remove `0147_insertion_sort_list` from the bullet list.
3. In "Practical Priority Order §5 — Explicit parity-debt rewrites", add `0147_insertion_sort_list` alongside `0148_sort_list`.
4. (Optional, out of Category 1 scope but adjacent): consider moving `0894_all_possible_full_binary_trees` from Category 2a to Category 4 — its divergence is driven by Sifr's no-aliasing ownership model, not by cursor ergonomics.
5. Apply the reason refinements in the table above.

## Summary

- All six fixtures in Category 1 are correctly classified.
- One fixture is **missing**: `0147_insertion_sort_list` uses the identical drain/`sorted()`/rebuild rewrite as `0148_sort_list` and should be reclassified from 2a into Category 1.
- No fixture should move out of Category 1.
- Stated reasons are planning-adequate; small precision upgrades (asymptotics, surface-vs-algorithm framing) would make implementation scoping crisper.
