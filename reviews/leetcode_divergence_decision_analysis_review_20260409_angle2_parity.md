Now I have enough context. Here is the review body:

---

## Review: Parity-Debt and Rewrite Angle

### 1. Is the rewrite list (Category 1) missing any cases or including any cases that should stay divergent?

**Likely missing: `0707_design_linked_list` (108 diff lines)**

The Sifr version substitutes an actual `list[int]` dynamic array for the Python doubly-linked list with sentinel nodes. The Python version guarantees O(1) for `get`, `addAtHead`, `addAtTail`, and `addAtIndex`. The Sifr version has O(n) for `addAtHead` and `addAtIndex` due to array concatenation/slicing. This is not cursor-ergonomics noise — it is a material asymptotic substitution on a data-structure-design problem. It belongs in Category 1 alongside `0212_word_search_ii` and `0295_find_median_from_data_stream`, not in the recursive-node ergonomics bucket.

**Likely correctly excluded: `0269_alien_dictionary` (118 diff lines) and `0261_graph_valid_tree` (117 diff lines)**

The alien dictionary Sifr version uses the same topological-sort algorithm as Python (BFS-based, same O(N+C) asymptotics) but with an explicit 26×26 boolean matrix and `_nz_*` null-guards. The algorithm is preserved; the verbosity is ergonomics, not substitution. Similarly, `0261_graph_valid_tree` uses DSU in both versions — the Python diff is inflated because it contains two implementations (DFS and DSU) while the Sifr version keeps only the DSU one. Both stay in their current buckets correctly.

**`0023_merge_k_sorted_lists` is misclassified in Category 5**

It appears in both Category 1 (rewrite mainly) and Category 5 (corpus cleanup), which is internally contradictory. The Python source uses a heap queue over `ListNode` objects — the canonical approach. The Sifr version switches to `list[list[int]]` as the input model. This IS a material substitution: it changes what the public-facing input representation is, not just how it is traversed. It should be treated as genuine rewrite debt, not corpus noise, and its Category 5 co-classification should be removed. The fact that the Python fixture happens to include dead helper baggage does not alter that the Sifr version made an intentional structural choice to avoid expressing linked-list traversal.

**`0148_sort_list` should be removed from Category 5 entirely**

It appears in both Category 1 and Category 5. The Category 5 note ("do not use as language design signal until corpus is normalized") makes sense for some co-listed items (e.g., `0200_number_of_islands` which has three implementations in one Python file), but `0148_sort_list`'s divergence is not driven by Python-side noise — it is driven by Sifr genuinely substituting linked-list merge sort with flatten/sort/rebuild. It belongs firmly in Category 1 and should be removed from Category 5.

### 2. Do any ergonomics-bucket items belong in explicit rewrite debt?

**`0707_design_linked_list` should move from 2a to Category 1**

As analyzed above: array-based storage with O(n) insertions versus a real doubly-linked list with O(1) operations throughout. This is not recursive-node ergonomics. It is data-structure substitution. The fix requires a structural rewrite, not narrowing improvements.

**`0208_implement_trie_prefix_tree` and `0146_lru_cache` are borderline but correctly placed**

Both are data-structure-design problems where the interface may shift between Python dict-of-dicts and a Sifr struct-based approach. However, the algorithmic core (trie traversal, LRU bookkeeping) is preserved. They belong in ergonomics pending trie and collection ergonomics work, not in the rewrite list.

### 3. Is the rewrite list prioritized correctly given public-surface changes and asymptotic regressions?

The Practical Priority Order lists rewrite items as step 5, after ergonomics work. This underweights the public-surface regressions. A better ordering would front-load the cases where the public input/output model changed:

**Suggested reorder within Category 1:**

1. `0148_sort_list` — material algorithm substitution (linked-list merge sort → array sort) and asymptotic difference
2. `0295_find_median_from_data_stream` — heap vs. sorted-array insertion changes asymptotic insert cost; included despite only 56 diff lines (correct)
3. `0023_merge_k_sorted_lists` — public input model change (linked-list inputs → `list[list[int]]`); needs explicit rewrite toward heap-over-linked-nodes
4. `0707_design_linked_list` — data-structure substitution with O(1) guarantees violated; needs redesign toward actual linked representation
5. `0133_clone_graph` — public model change (object-graph → adjacency list), same pattern as `0023`
6. `0212_word_search_ii` — trie vs. board-search substitution, but priority can remain lower since trie ergonomics work would naturally unblock the rewrite

The ergonomics-first ordering in the doc is reasonable as a work-planning constraint (the ergonomics work makes rewrites easier), but as a **classification of debt severity**, the public-surface changes (`0148`, `0295`, `0023`, `0707`) should be labeled as higher-risk than the stdlib-unblock rewrites (`0133`, `0212`).

### 4. Strongest correction to the rewrite classification

**Move `0707_design_linked_list` from 2a into Category 1 as the sixth rewrite item.**

The defining test for "rewrite mainly" versus "ergonomics" is: does the Sifr version change what the problem guarantees about the solution, not just how verbose the solution is? For `0707`, the Sifr version demonstrably does — it replaces guaranteed O(1) operations with O(n) for most mutations. No amount of narrowing improvement on `node.next` makes `[val] + self.values` become O(1). This is the clearest misclassification in the ergonomics buckets: it is listed under "Recursive Node / Cursor Ergonomics" but the issue is not cursor friction, it is an intentional data-structure substitution to avoid expressing a real linked list.

The second strongest correction is **removing `0023_merge_k_sorted_lists` and `0148_sort_list` from Category 5** — their presence there undermines the credibility of Category 1 as a serious rewrite list. If they are rewrite debt, they are rewrite debt. Categorizing them as corpus-noise signals mixed treatment of two genuinely different problems.
