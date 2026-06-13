

## M2d Review Findings

### `0212_word_search_ii.sifr`

**Change (lines 67–69):** Replaced output loop from iterating over `words` (input) to iterating over `found.keys()`.

**Duplicate handling — satisfied.** `Trie.insert` stores terminal state per character-sequence node — inserting `"oath"` twice sets `terminal[node] = True` on the same node, no duplicate entry. `collectWords` (`collectWords:36`) sets `found[next_prefix] = True` by string key, so even path-level deduplication is covered. `found` thus contains at most one entry per unique word regardless of how many times it appears in `words` or how many board paths reach it. The harness validates count/checksum, which both improve with deduplication.

**Scope creep — satisfied.** Single focused change. No compiler semantics touched.

---

### `0269_alien_dictionary.sifr`

**Change:** Replaced Kahn topological sort with DFS postorder over `present_order` (first-seen character order as tiebreak).

**DFS cycle behavior — satisfied.** `_dfs_visit:50–54` implements the standard3-state cycle detection: state 0 = unvisited, state 1 = in-progress (recursion stack), state 2 = done. On state 1 (back edge), returns `True` → `alienOrder:102` returns `""`. State 2 returns `False` (no cycle, continue). Correct.

**DFS neighbor iteration order — satisfied.** `_dfs_visit:59–62` iterates neighbors via `order` (the `present_order` parameter), so when multiple valid orderings exist, DFS explores in first-seen character order — matching the Python oracle's expected string shape.

**Prefix invalid-order — satisfied.** `alienOrder:95` returns `""` when `not found_diff and len(w1) > len(w2) and _has_prefix(w1, w2, min_len)`. `_has_prefix:35–41` confirms character-by-character match over `min_len`. Traced: `"abc"`, `"ab"` → min_len=2, prefix matches → returns `""`. Correct.

**Expected-order mismatch risk — low.** The DFS produces a valid topological order; if the Python oracle also produces a valid topological order, they will match because both use first-seen character order as tiebreak. The harness validates exact string, so any mismatch would surface in the correctness run.

**Scope creep — satisfied.** All original building blocks (edge building, cycle detection, order reversal) preserved. Only the traversal strategy changed from Kahn BFS to DFS postorder.

---

### No blockers found.

**Satisfied for M2d.**
