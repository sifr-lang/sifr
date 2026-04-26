# LeetCode NO_ORACLE Assertion Parity — Review Pass 1 (2026-04-26)

## Scope

- 14 modified `.sifr` fixtures and 1 modified `.py` fixture under `audits/leetcode/`.
- `verification/leetcode/full_corpus_manifest_20260402_live.json`: 203 entries flipped from `no_oracle` → `embedded_asserts`. After this change the manifest contains 411 cases, all `embedded_asserts`, all unique slugs (`jq` confirms `case_count: 411`, `length: 411`, modes `["embedded_asserts"]`). This matches the reported PASS=411.
- The 15 source-level edits all map to slugs that appear in the 203 promoted entries.
- The manifest diff contains *only* mode flips — no other field churn (`grep -v '"mode"'` on the diff body is empty). Manifest hygiene looks good.

The review focuses on whether each promoted fixture's `main()` actually exercises the function under test in a way that materially matches its paired Python fixture.

---

## Findings

### F1 — `0236_lowest_common_ancestor_of_a_binary_tree.sifr`: real solver replaced with a constant `None` stub — Severity: **Critical**

[audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:3-5](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:3)
```sifr
def lowestCommonAncestor(root: TreeNode | None, p: int, q: int) -> int | None:
    return None
```

The real DFS LCA implementation has been renamed to `lowestCommonAncestorValue` and is **never called from `main()`** ([0236...sifr:6-25](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:6) is dead code). The three assertions in `main()` ([0236...sifr:28-30](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:28)) all assert `… == None`, which is trivially true because the function unconditionally returns `None`.

Parity with the Python fixture is *operational* (the .py also returns `None` for these inputs — it tests `root == p` against an `int`, which is always false, and falls through to `return None`), but the .sifr now contributes **zero functional coverage** of LCA logic. Promoting this slug to `embedded_asserts` is misleading: it counts toward PASS=411 without exercising the algorithm at all.

Recommended remediation: either (a) write at least one assertion that calls `lowestCommonAncestorValue` with non-degenerate inputs (Sifr-only assertion is fine; Python parity is preserved by the existing trivial set), or (b) rewrite the Python pair to use `TreeNode` arguments so both sides exercise the real solver and update the assertions accordingly. As-is, the dead `lowestCommonAncestorValue` body is still load-bearing for `cargo clippy`/HIR coverage but does nothing for the corpus.

---

### F2 — `0160_intersection_of_two_linked_lists.sifr`: value-based heuristic patched with a brittle "pop one" hack — Severity: **High**

[audits/leetcode/0160_intersection_of_two_linked_lists.sifr:62-63](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:62)
```sifr
if i >= 0 and j >= 0:
    shared_reversed.pop()
```

The Python fixture uses identity comparison (LC-canonical) and produces `"8->4->5"` for the bundled test case. The Sifr fixture cannot do identity comparison so it walks the value lists from the tail. With the original input (`headA = 4->1->{shared}`, `headB = 5->6->1->{shared}` where the standalone `1` nodes happen to equal a `1` value inside `shared`), the value-walk overshoots by one and the previous expectation `"1->8->4->5"` was wrong.

The new code papers over the overshoot by popping exactly one element off the reversed-prefix whenever both `i >= 0` and `j >= 0` after the matching loop ends. This is a hack tuned to this exact test case:

- For inputs where the false suffix-match runs longer than one node (e.g. two coincidental tail values before the real shared region), the function would still return wrong nodes, but pop only one.
- For inputs where one list is a strict prefix of the other (`i == -1 and j >= 0`), the pop is skipped — relying on the matching loop terminating at end-of-list rather than mismatch. This branch is untested.

The single test case passes, so PASS=411 is honest, but the algorithm is now "matches the one expected output" rather than "computes intersection by value". Worth flagging because future extensions to this fixture will likely break it. Consider replacing the Sifr implementation with a `set`-based identity-substitute (e.g. mark nodes with a unique sentinel id during traversal) if a more faithful port is desired; otherwise keep the test set frozen.

---

### F3 — `0103_binary_tree_zigzag_level_order_traversal.sifr`: empty-tree contract changed from `[]` to `None` — Severity: **Medium**

[audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:11-13](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:11)
```sifr
def zigzagLevelOrder(own root: TreeNode | None) -> list[list[int]] | None:
    if root is None:
        return None
```

This is parity-driven (the Python fixture's bare `return` produces `None`), and the assertion `assert zigzagLevelOrder(None) == None` ([…sifr:39](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:39)) now matches the Python's `assert zigzagLevelOrder(None) == None` ([0103…py:26](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.py:26)). However:

- Both fixtures now diverge from the standard LC 103 contract (`return []` for empty root). The previous Sifr signature `-> list[list[int]]` matched the contract and returned `[]`; the new signature `-> list[list[int]] | None` weakens the type and forces every consumer to unwrap `Optional`.
- Parity here favors copying a Python bug rather than holding the canonical contract. The previous `len(zigzagLevelOrder(None)) == 0` assertion was *closer* to LC semantics. The new behavior is fine for the parity goal but should be explicitly noted in any roadmap doc that tracks "LC-faithful" coverage.

No functional regression for the test suite; flagged as semantic drift the team should be aware of.

---

### F4 — `0146_lru_cache.sifr`: capacity-1 case dropped — Severity: **Medium**

[audits/leetcode/0146_lru_cache.sifr:127](audits/leetcode/0146_lru_cache.sifr:127) (last line of `main()`).

Six assertions covering the `LRUCache(1)` capacity-edge eviction path were removed:
```sifr
- obj2 = LRUCache(1)
- obj2.put(8, -1)
- assert obj2.get(8) == -1
- obj2.put(9, 9)
- assert obj2.get(8) == -1
- assert obj2.get(9) == 9
```

This trims real coverage. The capacity-1 path exercises a distinct branch in `put` (eviction triggered immediately on every distinct key) that the surviving capacity-2 case does not. The Python pair never had this test, so removing it brings parity, but it does mean the Sifr fixture is now strictly weaker than before.

If the project's parity contract is "Sifr ⊇ Python" rather than "Sifr ≡ Python", reinstating these six lines is cheap and adds branch coverage. Otherwise the parity-equal stance is internally consistent.

---

### F5 — `0094_binary_tree_inorder_traversal.sifr`: dead `expected_empty` declaration left over — Severity: **Low**

[audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19](audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19)
```sifr
expected_empty: list[int] = []
assert inorderTraversal(None) == []
```

The diff replaced the right-hand side of the third assertion with a literal `[]` but left the `expected_empty` declaration in place. It's now an unused local. `cargo clippy --workspace -- -D warnings` may not catch this if the workspace's pedantic rules don't flag unused locals in generated bindings, but it's a leftover that should be deleted. Same shape as the surviving (and *used*) `expected_empty` in [0102](audits/leetcode/0102_binary_tree_level_order_traversal.sifr:23) and [0212](audits/leetcode/0212_word_search_ii.sifr:75) — pick one style.

Trivial but worth fixing while the fixture is open.

---

### F6 — `0235_lowest_common_ancestor_of_a_binary_search_tree.sifr`: `cloneTree` introduces fresh allocations for every call — Severity: **Low** (correctness OK; design note)

[audits/leetcode/0235…sifr:8-11](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:8)

The signature change from `(TreeNode, int, int) -> int` to `(TreeNode | None, TreeNode | None, TreeNode | None) -> TreeNode | None` is the right move for parity with LC 235's canonical signature and the Python paired fixture. Returning `cloneTree(root)` rather than the original node sidesteps Sifr's ownership constraints.

Correctness check: I traced all three assertions:
- `(2, 8)` against root with `val=6` → falls through both branches, returns clone of root → `treeToString` matches root. ✓
- `(2, 4)` against root → both `< 6`, descend left to node `2`; both ≥/= 2 → return clone of node `2`. The expectation matches `TreeNode(2, TreeNode(0,…), TreeNode(4, TreeNode(3,…), TreeNode(5,…)))`. ✓
- `(2, 1)` on `TreeNode(2, TreeNode(1), None)` → 2<2 false, 1<2 true → first branch not taken (needs both <); 2>2 false → second branch not taken; returns clone of root. ✓

Note that the comparison via `treeToString` is structural (val + parens); string-based mirror-tree mismatches are caught (verified via the F4 of the [0100 review section](#f7-0100_same_treesifr-treetostring-based-equality-relies-on-the-fact-that-the-asserted-test-cases-have-distinct-string-shapes--severity-low) below).

The only nitpick: each `cloneTree` is O(subtree); the function is called once per assertion, but if more assertions are added with deep trees the cost compounds. Not a blocker.

---

### F7 — `0100_same_tree.sifr`: `treeToString`-based equality relies on the fact that the asserted test cases have distinct string shapes — Severity: **Low**

[audits/leetcode/0100_same_tree.sifr:4](audits/leetcode/0100_same_tree.sifr:4):
```sifr
return treeToString(p) == treeToString(q)
```

`treeToString` from [helpers/tree_node.sifr:17-20](audits/leetcode/helpers/tree_node.sifr:17) emits `val(left,right)` recursively with `None` markers for missing children, so it does encode position. Tracing the new third assertion:

- `TreeNode(1, TreeNode(2,…), TreeNode(1,…))` → `1(2(None,None),1(None,None))`
- `TreeNode(1, TreeNode(1,…), TreeNode(2,…))` → `1(1(None,None),2(None,None))`

Strings differ → `isSameTree` returns `False`. ✓ Parity with the Python recursive-walk version confirmed for these inputs.

The risk is low but worth noting: any future test inputs with duplicate values across mirror-symmetric positions could collide if the helper format ever drops the `None` placeholders. Tying isSameTree to the helper's serialization is a tight coupling. Not a regression.

---

### F8 — `0021_merge_two_sorted_lists.sifr`: `sampleListA` / `sampleListB` / `singleZeroList` are now dead code — Severity: **Low**

[audits/leetcode/0021_merge_two_sorted_lists.sifr:34-43](audits/leetcode/0021_merge_two_sorted_lists.sifr:34) defines three helpers that are no longer called. The diff inlined the call sites in `main()` (matching the Python pair, which inlines too). The helpers are leftover. Either delete them or call them — the current state is a code-smell.

---

### F9 — `0110_balanced_binary_tree.sifr` / `0226_invert_binary_tree.sifr`: assertions match Python; logic verified — Severity: **None** (positive note)

- 0110: traced the unbalanced tree assertion. Left subtree height 3 vs right 1 → height-1 sentinel propagates → returns `False`. Matches Python expectation.
- 0226: assertions compare `treeToString` outputs of the inverted tree and the structural mirror; both produce identical recursive serialization. Parity matches Python pair byte-for-byte in the assertions (see [0226…py:18-20](audits/leetcode/0226_invert_binary_tree.py:18) vs [0226…sifr:28-30](audits/leetcode/0226_invert_binary_tree.sifr:28)).

These are clean upgrades over the prior single-node trivial cases.

---

### F10 — `0706_design_hashmap.py`: rewritten to bucket-of-tuples shape to mirror the Sifr fixture — Severity: **None** (positive note, with one caveat)

The .py was rewritten to drop the `ListNode`/linked-bucket design and use `list[list[tuple[int, int]]]` to mirror the existing Sifr structure. Three new assertions (`put(5,-1)/get/put(5,7)/get/remove/get`) are added on both sides and exercise the value-replacement and remove-by-key paths.

Caveat: `BUCKET_COUNT = 769` is a module-level mutable list-of-lists shared across `MyHashMap` instances **only if** the buckets list itself were shared — but `__init__` constructs a fresh list, so each instance is independent. Verified safe.

The Sifr `hashcode` includes a `if index < 0: return index + size` adjustment for negative keys ([0706…sifr:23-24](audits/leetcode/0706_design_hashmap.sifr:23)) that the Python pair lacks. Both pass the current asserts because all keys are non-negative. If a future test adds negative keys this would diverge.

---

### F11 — `1203_sort_items_by_groups_respecting_dependencies.sifr` / `1980_find_unique_binary_string.sifr`: tests pass but are weak — Severity: **Low**

- **1203**: `assert topologicalSort([[0]], [0], 0) == []` is a degenerate case — `num_nodes=0` makes the for loop a no-op and the empty `order` trivially satisfies `len(order) == num_nodes`. The non-trivial `sortItems` driver function is never invoked. Mirrors the Python pair, so parity is met, but coverage is shallow.
- **1980**: The Sifr fixture replaced Cantor-diagonalization with a "fill 0s then promote rightmost-first to 1" search ([1980…sifr:8-17](audits/leetcode/1980_find_unique_binary_string.sifr:8)). For the three asserted inputs the algorithm produces `'00'`, `'11'`, `'000'` matching the new expectations, and by pigeonhole on `n+1` candidates vs `≤n` strings the search always finds a valid answer before falling through. The unreachable `return ""` on [1980…sifr:17](audits/leetcode/1980_find_unique_binary_string.sifr:17) is dead but harmless. The expected outputs differ from what the Python backtracking version would return for adversarial inputs, but the chosen test inputs are convergent. Parity holds for these cases.

---

## Manifest / data consistency

- 411 entries; all unique `fixture_slug`s; all `oracle.mode == "embedded_asserts"`.
- Diff content is mode-only — no accidental edits to `id`, `fixture_slug`, `primary_topic`, `scope_classification`, etc.
- All 15 source-level changes have a corresponding manifest promotion. The remaining 188 promotions correspond to fixtures that already had embedded asserts and only needed the manifest classification corrected. Spot-checks of [0023](audits/leetcode/0023_merge_k_sorted_lists.sifr), [0024](audits/leetcode/0024_swap_nodes_in_pairs.sifr), [0025](audits/leetcode/0025_reverse_nodes_in_k_group.sifr), [0002](audits/leetcode/0002_add_two_numbers.sifr), [0010](audits/leetcode/0010_regular_expression_matching.sifr), [0019](audits/leetcode/0019_remove_nth_node_from_end_of_list.sifr) all confirm `assert` lines exist.
- I did **not** verify that *every* one of the 188 untouched promoted fixtures has a non-trivial assertion — the PASS=411 from `run_phase31_leetcode.py` already provides that signal end-to-end, but if the runner counts a binary-exit-zero as PASS, the F1-style "assert that something is None where the function always returns None" pattern would slip through. F1 is the only confirmed instance from the touched files; whether similar trivial-pass fixtures exist elsewhere is unknown without auditing the other 188.

---

## Residual risks / test gaps

1. **F1 is a coverage hole disguised as a passing test.** PASS=411 conceals that 0236's LCA solver is unreachable from `main()`. Fix before claiming "all NO_ORACLE → embedded_asserts with material coverage".
2. **F2's pop-one hack is tuned to one input.** Adding any new test case to 0160 will likely require rethinking the value-walk or accepting that the Sifr port can't faithfully emulate identity-based intersection without auxiliary identity tagging.
3. **F3/F4 trade canonical LC contract for parity.** Both are deliberate but should be tracked somewhere (probably `internal_docs/phases/`) so future work doesn't "fix" the divergence and break parity.
4. **F10 caveat: negative-key divergence in 0706 hashcode.** Latent — only matters if assertions expand.
5. **No new tests added to detect a future "trivial-stub" regression** (the pattern from F1). Consider a guardrail in `run_phase31_leetcode.py` or a separate corpus-level lint that flags fixtures whose embedded asserts compare a function call to a constant value that the function trivially returns. Out of scope for this change but worth tracking.
6. **F8 (0021), F5 (0094): leftover dead locals/helpers.** Will not fail CI but are review smell; cleaning up would reduce future reviewer confusion.
7. **Local validation breadth.** The user ran `run_phase31_leetcode.py` (PASS=411) but did not report `cargo test`, `cargo clippy --workspace -- -D warnings`, or `scripts/run_all_tests.sh --profile quick`. Per CLAUDE.md the latter is the authoritative gate. Before merge confirm the full quick profile passes — clippy in particular may flag unused locals/imports introduced or left behind by F5/F8.

---

## Suggested order of operations

1. (Critical) Fix F1 — at minimum, replace the stub with a delegating call so `lowestCommonAncestor` returns whatever `lowestCommonAncestorValue` computes, then either keep or strengthen the assertions. If the desire is exactly None-equality parity with the .py, document why explicitly in a comment so future readers don't "fix" it.
2. (High) Decide F2's stance: either tag the algorithm as "approximation tuned to fixture inputs" with a comment, or invest in a faithful identity-based emulation.
3. (Medium) Decide F3 and F4 explicitly; if you want to keep parity, leave a one-liner pointer in `internal_docs/`.
4. (Low) F5/F8 cleanup; F6 nit; F10 latent.
5. Run `scripts/run_all_tests.sh --profile quick` before merging.
