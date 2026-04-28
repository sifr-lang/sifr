# Category 1 Review: Should Have Parity, Rewrite Mainly

Reviewed against source files in `audits/leetcode/` and the divergence analysis
(`leetcode_divergence_decision_analysis_20260409.md`).

## Verdict: All Six Fixtures Belong Here — But One Reason Is Slightly Off

All six fixtures are correctly placed. The stated reasons for five of them are
accurate. One needs a correction.

---

## Fixture-by-Fixture Assessment

### `0023_merge_k_sorted_lists`

- **Sifr version**: Takes `list[list[int]]`, flattens all rows, sorts, returns `list[int]`. No linked-list structure at all.
- **Python version**: Takes `list[ListNode]`, performs k-way merge using pairwise heap-style reduction.
- **Reason stated**: "changes the public input model from linked lists to `list[list[int]]`"
- **Verdict**: ACCURATE. The Sifr port abandoned the linked-list input model entirely and replaced the k-way merge with a flatten-sort. This is a clear rewrite case.

### `0133_clone_graph`

- **Sifr version**: `cloneGraph(adjacency: list[list[int]]) -> list[list[int]]` — iterates rows, copies and sorts each row's neighbors. Not a graph clone at all; no node-level DFS, no reference mapping.
- **Python version**: `cloneGraph(node: Node) -> Node` — DFS-based deep copy with `oldToNew` reference map, proper graph structure.
- **Reason stated**: "changes the public model from object-graph cloning to adjacency-list copying"
- **Verdict**: ACCURATE. The Sifr version is a pure adjacency-list copy, not a graph clone. No node-identity mapping, no recursive traversal.

### `0148_sort_list`

- **Sifr version**: `sortList` — flattens the linked list into a `list[int]`, calls Python's `sorted()`, then rebuilds a new linked list by iterating in reverse order. Includes dead helper functions (`nodeVal`, `nodeNext`, `hasNode`, `unwrapInt`) that gate access on `is not None` even though the loop already uses `hasNode()`.
- **Python version**: Classic linked-list merge sort — `get_mid()` uses the slow/fast pointer technique to find the midpoint in O(n), `sortList()` recurses on both halves, `merge_two_sorted()` merges in order.
- **Reason stated**: "replaces linked-list merge sort with flatten/sort/rebuild"
- **Verdict**: ACCURATE. The Sifr version uses O(n) flatten + Python's `sorted()` (likely O(n log n) under the hood) + rebuild, completely abandoning the in-place O(n log n) merge sort algorithm. The helper-gate boilerplate also confirms this is a workaround, not a principled port.

### `0212_word_search_ii`

- **Sifr version**: `findWords` — for each word in `words`, calls `_word_exists()` which does a plain backtracking DFS over the board with no trie, no prefix pruning, no shared state.
- **Python version**: `findWords` — builds a `TrieNode` prefix tree, uses ref-counting for word removal during DFS to avoid revisits, coordinates across all words to share the trie structure.
- **Reason stated**: "replaces trie/prefix-pruning with per-word board search"
- **Verdict**: ACCURATE. The Sifr version removes the trie entirely and searches each word independently with no cross-word sharing or pruning. Correctly a rewrite case.

### `0295_find_median_from_data_stream`

- **Sifr version**: `MedianFinder` — stores a flat `list[int]`, inserts via linear-scan linear-time insertion (`while i < len(self.nums)` followed by list concatenation `left + [num] + right`).
- **Python version**: `MedianFinder` — uses a max-heap for the lower half and a min-heap for the upper half, both via `heapq`. O(log n) insertion.
- **Reason stated**: "replaces heap-based updates with sorted-array insertion and changes asymptotic behavior"
- **Verdict**: ACCURATE. The Sifr version uses linear-time array insertion. The asymptotic change from O(log n) to O(n) per `addNum` is real and material.

### `0707_design_linked_list`

- **Sifr version**: `MyLinkedList` — backed by `list[int]`. `get()` does a linear scan. `addAtHead()` does `[val] + values` (O(n) cons). `addAtIndex()` rebuilds two sublists and concatenates. `deleteAtIndex()` rebuilds the list without the element.
- **Python version**: `MyLinkedList` — backed by real `ListNode` doubly-linked sentinel nodes. All operations are O(1) pointer manipulations.
- **Reason stated**: "replaces a linked-list data-structure design with array-backed storage and loses the intended operation-cost profile"
- **Verdict**: ACCURATE. The Sifr version uses an array list as the backing store, which converts all the intended O(1) operations (add at head, add at tail, add at index, delete at index) into O(n) operations. The operation-cost profile is entirely lost.

---

## Issue: One Reason Is Slightly Imprecise

### `0148_sort_list` reason

The stated reason says "replaces linked-list merge sort with flatten/sort/rebuild". This is correct but the Sifr file also includes dead helper boilerplate (`nodeVal`, `nodeNext`, `hasNode`, `unwrapInt`) that signals the author had to work around the lack of `is not None` narrowing to extract and pass node values safely. This boilerplate is noise on top of the divergence — it confirms the algorithmic rewrite but also flags it as a case where Sifr's current null-guard ergonomics are actively painful. Worth noting in the reason, since the fix for this particular fixture involves both a rewrite **and** the ergonomics work in Category 2a.

---

## MissingFixtures Check

Scanning the full audit results for fixtures with high divergence that **are not** in Category 1 but **should** be rewrite cases based on algorithm/data-structure abandonment:

The Category 2a (recursive/cursor ergonomics) fixtures include `0023_merge_k_sorted_lists` and `0148_sort_list` but they are correctly in Category 1 since they go beyond ergonomics — they abandon the core data structure. All other Category 2a fixtures (linked-list traversals, tree rewiring) preserve the data structure even if the code is more verbose due to ergonomics gaps.

No high-divergence fixtures appear to be missing from Category 1.

---

## Summary

| Fixture | In Category 1 Correctly? | Reason Accurate? |
|---|---|---|
| `0023_merge_k_sorted_lists` | Yes | Yes |
| `0133_clone_graph` | Yes | Yes |
| `0148_sort_list` | Yes | Yes (with note about dead boilerplate) |
| `0212_word_search_ii` | Yes | Yes |
| `0295_find_median_from_data_stream` | Yes | Yes |
| `0707_design_linked_list` | Yes | Yes |

**No removals. No additions. All reasons accurate.**