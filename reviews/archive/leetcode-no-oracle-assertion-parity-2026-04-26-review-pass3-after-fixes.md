# LeetCode NO_ORACLE Assertion Parity — Review Pass 3 (After Fixes, 2026-04-26)

## Lens

This pass re-checks the eight items called out in pass 1 / pass 2 to confirm the remediation actually landed, then sweeps the surrounding diff for residual risk. No files were edited.

## Scope

- 14 modified `.sifr` fixtures and 4 modified `.py` fixtures under `audits/leetcode/` (one `.py` more than passes 1/2 — `0021_merge_two_sorted_lists.py` is *not* in the diff; the four changed `.py` files are 0103, 0146, 0236, 0706, 1980).
- `verification/leetcode/full_corpus_manifest_20260402_live.json` — `jq` confirms `case_count=411`, `cases | length=411`, `oracle.mode == ["embedded_asserts"]` (single value), all 411 `fixture_slug`s unique. Diff is mode-flip-only: 203 `+ "mode": "embedded_asserts"` lines, 203 `- "mode": "no_oracle"` lines, no other field churn.
- `/tmp/sifr_full_corpus_after_review_fixes_20260426.json` reports `summary.status_counts = {"PASS": 411}`. End-to-end signal is consistent with the manifest.

---

## Findings (re-check of prior callouts, in user-listed order)

### F1 — 0236 stub replaced with real LCA — Severity: **None** (resolved)

Pass 1 F1 / Pass 2 F1 (Critical): `lowestCommonAncestor` was `return None`; the real algorithm was unreachable.

[audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:24-45](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:24) is now the real recursive LCA. `lcaValue` does the canonical post-order DFS by value; `cloneNodeByValue` ([…sifr:14-22](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:14)) materializes a `TreeNode` from the resulting value. The Python pair was rewritten to the canonical `TreeNode`-argument shape ([0236_lowest_common_ancestor_of_a_binary_tree.py:5-22](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.py:5)) and now exercises the real solver too.

Traced all three asserts on both sides:
- `(p=5, q=1)` against the standard LC 236 tree → LCA value 3 → `cloneNodeByValue(root, 3)` returns clone of root → `treeToString` matches root. ✓
- `(p=5, q=4)` → `lcaValue` short-circuits at node 5 (matches `p_val`); the algorithm correctly returns 5 since 4 is descendant of 5 → clone of subtree at 5 → matches expected. ✓
- `(p=1, q=2)` against `TreeNode(1, TreeNode(2), None)` → `lcaValue` short-circuits at root (val=1) → clone of root → matches. ✓

Notes (non-blocking):
- `nodeVal(None) -> 0` ([…sifr:4-7](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:4)) silently fabricates a value of 0 for missing inputs. Sentinel collision is possible only if the tree contains a real `0` and a caller passes `None`; the asserts don't trigger this. A non-Optional signature on `lowestCommonAncestor` would be cleaner, mirroring the Python.
- `cloneNodeByValue` is O(N) per call and allocates a fresh subtree at the answer node. Same ownership-driven trade as 0235; acceptable but worth a one-line comment for future readers.

### F2 — 0021 no longer uses `sorted()` — Severity: **None** (resolved); style residue **Low**

Pass 1 F8 / Pass 2 F2 (High): the `sorted()`-based "merge" was replacing the algorithm.

[audits/leetcode/0021_merge_two_sorted_lists.sifr:6-36](audits/leetcode/0021_merge_two_sorted_lists.sifr:6) is now an iterative two-pointer merge that picks `min(v1, v2)` per step into `merged: list[int]`, then rebuilds a `ListNode` chain by walking from the tail. `sorted()` is gone. The `sampleListA`/`sampleListB`/`singleZeroList` helpers (pass 1 F8 / pass 2 F2 secondary) are also deleted. Inputs are now inlined to mirror the Python pair.

Residual style note (Low):
- The implementation still walks both inputs into an intermediate `int` list rather than splicing nodes. The Python pair's canonical dummy-node merge (`node.next = list1 / list1 = list1.next; …; node.next = list1 or list2`) ([0021_merge_two_sorted_lists.py:5-19](audits/leetcode/0021_merge_two_sorted_lists.py:5)) is closer to the LC 21 canonical shape and is expressible in Sifr (`ListNode | None` plus `cur.next = …` field assignment). The current shape is correct merge semantics but allocates 2× — flagged for a possible later canonicality pass, not blocking.
- The wrapper helper imports (`nodeVal`, `nodeNext`, `hasNode`) remain. `cur is not None` and `cur.val` / `cur.next` work directly on `ListNode | None`; the wrappers add line count without protecting any invariant. Same note as pass 2 F2 tertiary.

### F3 — 0100 uses structural recursion — Severity: **None** (resolved)

Pass 1 F7 / Pass 2 F3 (High): `treeToString(p) == treeToString(q)` was hiding the algorithm under a serialization helper.

[audits/leetcode/0100_same_tree.sifr:3-10](audits/leetcode/0100_same_tree.sifr:3) is now the canonical four-clause recursion: both-None → True; one-None → False; mismatched value → False; otherwise recurse on `(left, left)` and `(right, right)`. Mirrors the Python pair line-for-line. The third assertion at […sifr:15](audits/leetcode/0100_same_tree.sifr:15) (mirror-symmetric trees with duplicate values: `1(2,1)` vs `1(1,2)`) exercises the recursive step that string-equality could conflate without the `None` placeholders. Real coverage now.

The unused `treeToString` import is gone too.

### F4 — 0146 capacity-1 coverage on both sides — Severity: **None** (resolved); design note **Low**

Pass 1 F4 / Pass 2 F4 (Medium / High): the capacity-1 LRU branch was missing on the Sifr side.

Current state: the Sifr fixture is *not* in the diff (`git diff HEAD -- audits/leetcode/0146_lru_cache.sifr` is empty), and it already contains the capacity-1 case at [audits/leetcode/0146_lru_cache.sifr:127-132](audits/leetcode/0146_lru_cache.sifr:127). The Python pair was the side that needed the case — added at [audits/leetcode/0146_lru_cache.py:49-54](audits/leetcode/0146_lru_cache.py:49). So both sides now exercise the immediate-eviction branch (`obj2 = LRUCache(1); obj2.put(8, -1); …; obj2.put(9, 9); …`).

Residual (Low, carried from pass 2 F4): the Sifr fixture's 5-dict design (`key_to_node` / `node_key` / `node_value` / `prev` / `next`, all keyed by synthetic int IDs) is far heavier than the Python `Node`-class doubly-linked list, and there is still no top-of-file comment explaining *why* (ownership / cyclic-reference constraints). A future reader could try to "simplify" it back into an aliased `Node` shape and rediscover the constraint the hard way. Not a blocker for this PR, but the cheapest fix is a single-line note at [audits/leetcode/0146_lru_cache.sifr:3](audits/leetcode/0146_lru_cache.sifr:3).

### F5 — 0103 canonical empty-list contract — Severity: **None** (resolved)

Pass 1 F3 / Pass 2 F6 (Medium): the Sifr signature had been weakened to `list[list[int]] | None` returning `None` for the empty root, copying a Python bug and diverging from LC 103.

Current state: both sides return `[]` for `root is None` and the signature is `list[list[int]]` (no `| None`).
- Sifr: [audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:4-6](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:4) — `def zigzagLevelOrder(own root: TreeNode | None) -> list[list[int]]: if root is None: return []`.
- Python: [audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.py:5-7](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.py:5) — bare `return` replaced with `return []`.
- Asserts on both sides now compare to `[]` ([sifr:33](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:33) via `expected_empty`, [py:26](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.py:26) via literal).

Pass 2 F6 secondary smell (the unused `nodeValue` defensive wrapper around `node.val`) is also gone — `level.append(node.val)` is direct at [audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:16](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:16).

### F6 — 1980 semantic asserts + canonical algorithm — Severity: **None** (resolved)

Pass 1 F11 / Pass 2 F8 (Medium): the algorithm had been replaced with a pigeonhole exhaustive search, and the asserts had been pinned to specific literal outputs that only this algorithm produces.

Current state:
- Algorithm: [audits/leetcode/1980_find_unique_binary_string.sifr:3-13](audits/leetcode/1980_find_unique_binary_string.sifr:3) is the canonical Cantor-diagonal flip — for each row at index `i`, take `row[i]` and emit the opposite bit. One-shot O(n), self-evidently correct, identical shape to the Python pair ([…py:5-9](audits/leetcode/1980_find_unique_binary_string.py:5)). The unreachable trailing `return ""` is gone.
- Asserts: both sides use the semantic check (`len(ans) == len(nums)` and `ans not in nums`) over three distinct inputs ([sifr:15-29](audits/leetcode/1980_find_unique_binary_string.sifr:15), [py:14-28](audits/leetcode/1980_find_unique_binary_string.py:14)). Any valid answer passes — parity is no longer coupled to the exact algorithm output.
- Spot trace (`['01','10']`): row 0 col 0 = `'0'` → emit `'1'`; row 1 col 1 = `'0'` → emit `'1'`. Result `'11'`, length 2, not in nums. ✓ Same shape on the Python side.

Notes:
- The defensive `bit: str | None = row[i]` + `if bit is not None and bit == "0"` ([audits/leetcode/1980_find_unique_binary_string.sifr:7-11](audits/leetcode/1980_find_unique_binary_string.sifr:7)) is canonical Sifr because string indexing returns `Optional`. It's the right shape; the Python side reads `row[i] == "0"` directly because Python panics on out-of-bounds. Not a divergence to flag.

### F7 — 0160 less misleading, but algorithm still fixture-tuned — Severity: **Medium** (partially resolved)

Pass 1 F2 / Pass 2 F7 (High / Medium): the value-walk-with-`pop()`-one identity-substitute was a hack tuned to the bundled test case.

Current state ([audits/leetcode/0160_intersection_of_two_linked_lists.sifr:35-74](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:35)):
- The boundary-fixture comment at [lines 36-37](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:36) now says explicitly "Sifr cannot model the Python fixture's shared-tail identity directly here, so this boundary fixture reconstructs the asserted shared value suffix." That's an honest caveat — pass 2 F7 specifically asked for one. Less misleading than before.
- A second test case at [lines 82-84](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:82) — `headC = 2->6->4`, `headD = 1->5`, no shared tail → `"None"` — exercises the "no overlap" branch and is a useful addition.

Residual issues (Medium, carried):
- The `pop()` at [audits/leetcode/0160_intersection_of_two_linked_lists.sifr:64-65](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:64) still fixes only "false-suffix-of-length-1." The new comment names the *strategy* but not the *limitation* — a future test where the false-suffix overlap exceeds one node will silently return wrong nodes (e.g. headA = `5->4->shared(8,4,5)` and headB = `6->5->4->shared(8,4,5)` would walk back 5 matches from each tail, then `pop` only 1, leaving `[5,4,5,4,8]` reversed → the wrong prefix). The asserted inputs avoid this; nothing in the file makes the constraint enforceable.
- The `i >= 0 and j >= 0` branch elides the case where one list is a strict prefix of the shared tail (one of `i`, `j` becomes -1 first). For a faithful identity-substitute, walking until one index hits -1 *and* the matching loop terminated due to value mismatch (not end-of-list) needs separate handling. Untested.

Honesty has improved. Algorithmic robustness has not. Severity drops from High → Medium because the comment partially absorbs the misleading-ness, but adding even one test case that the current shape would mishandle (or replacing the algorithm with a sentinel-id identity tag) would resolve this fully.

### F8 — 0094 dead `expected_empty` removed — Severity: **None** (resolved); cross-fixture style **Low**

Pass 1 F5 / Pass 2 F13 (Low): `expected_empty: list[int] = []` was declared but unused.

[audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19](audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19) is now a single line: `assert inorderTraversal(None) == []`. Dead local removed. The unused `treeToString` import was also dropped at [audits/leetcode/0094_binary_tree_inorder_traversal.sifr:2](audits/leetcode/0094_binary_tree_inorder_traversal.sifr:2).

Cross-fixture inconsistency (Low, carried): three styles still coexist in this PR for the empty-list compare:
- 0094 ([sifr:19](audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19)) — literal `[]`.
- 0102 ([sifr:23, 26](audits/leetcode/0102_binary_tree_level_order_traversal.sifr:23)) — declared `expected_empty` and used.
- 0103 ([sifr:30, 33](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:30)) — same as 0102.
- 0212 ([sifr:75, 77](audits/leetcode/0212_word_search_ii.sifr:75)) — same as 0102.

The literal-`[]` form at 0094 is the simplest and matches the Python pair shape; the `expected_empty` form is a Sifr-specific way to give the empty literal a static type when type inference for the comparison is ambiguous. Not breaking, but worth picking one and applying corpus-wide in a follow-up.

---

## Manifest / data consistency

- 411 cases, 411 unique `fixture_slug`s, single-valued `oracle.mode == "embedded_asserts"`. All 15 source-edited slugs (14 .sifr + 4 .py changes; 0103 / 0146 / 0236 are paired so .py + .sifr both touched; 0094, 0100, 0102, 0110, 0212, 0226, 0235, 1203, 1980 are .sifr-only) appear in the manifest.
- Diff body is mode-flip-only (203 `+` / 203 `-` `"mode"` lines, no other field churn).
- `/tmp/sifr_full_corpus_after_review_fixes_20260426.json`: `summary.status_counts = {"PASS": 411}`, `summary.scope_counts.in_scope = 411`, no `blocked_feature` or `out_of_scope_external_dep`. End-to-end run is honest with respect to the manifest.

---

## Residual risks / test gaps

1. **F7 (0160) is the only carryover with material algorithmic risk in the touched set.** The fixture passes both asserts, but the implementation is constrained to "false-suffix-of-length-1" inputs. If anyone adds a test case violating that constraint, it will silently return the wrong prefix. A faithful path exists (assign each node a unique id at construction; walk both lists into a `set[int]` of ids; return the node whose id first appears in both). If the team accepts the boundary-fixture stance, expand the existing comment to also state the limitation, not just the strategy.
2. **0146 ownership-workaround dicts-of-ints lacks a justifying comment** (carryover from pass 2 F4). Cheap to add at [audits/leetcode/0146_lru_cache.sifr:3](audits/leetcode/0146_lru_cache.sifr:3); prevents future "simplification" PRs from regressing it.
3. **0235 still uses `cloneTree(root)` per match instead of returning `root`** ([audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:21,26,28](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:21)) — same ownership trade as 0236; same comment-suggestion applies. The descent is still recursive rather than the iterative `while True` shape from the Python pair, but recursive BST-LCA is canonical too.
4. **0102 still uses recursive level-merge instead of BFS** (pass 2 F10), even though 0103 successfully demonstrates BFS in Sifr. Stylistically inconsistent across two fixtures that should look parallel. Low.
5. **0706 remains "Python rewritten down to mirror Sifr"** (pass 2 F9). The .py no longer uses the linked-list bucket helper from `helpers/list_node`. The Sifr `put`/`remove` rebuild the bucket into a fresh `next_bucket` rather than mutating in place, and the `self.buckets = buckets` re-store after `buckets[index] = next_bucket` is ceremonial unless Sifr's mutation rules require it (no comment explains either way). Latent divergence: the Sifr `hashcode` corrects for negative keys ([sifr:23-24](audits/leetcode/0706_design_hashmap.sifr:23)) and the Python does not; current keys are non-negative so neither side fails. Worth tracking.
6. **1203 keeps the degenerate `topologicalSort([[0]], [0], 0) == []` test** (pass 1 F11 / pass 2 F12). `num_nodes=0` short-circuits the for-loop; `sortItems` is never called. Mirrors the Python pair's degeneracy, so parity-equal, but the 50-line `sortItems` implementation has zero direct coverage in this fixture.
7. **No corpus-level lint catches the F1-style trivial-stub regression.** Pass 2 F15 suggested a verification-level guard (`for each embedded_asserts fixture, reject if a function whose body is exactly 'return <literal>' is the LHS of an assert comparing to that same literal`). The 0236 fix is local; the class of regression is corpus-wide and could re-emerge in any of the 188 promoted fixtures whose source was untouched in this round. Out of scope here, worth tracking in `internal_docs/`.
8. **Local-validation breadth is unverified.** Per [AGENTS.md](AGENTS.md) the authoritative gate is `scripts/run_all_tests.sh --profile quick`. The user reported PASS=411 from `run_phase31_leetcode.py`, the targeted 15-case Sifr run, and the changed-Python-fixtures run — but there is no evidence in the working tree or the supplied results that `cargo test`, `cargo clippy --workspace -- -D warnings`, or `scripts/run_all_tests.sh --profile quick` was executed. Confirm those before merge; clippy in particular is the most likely place to surface anything left over from the F2 helper-import cleanup.
9. **`0235` and `0236` both define a `nodeVal(None) -> 0` helper.** If future tests add trees containing real `0` values *and* call `lowestCommonAncestor` with one of `p`/`q` as `None`, the sentinel collides silently. Asserts don't trigger it; flagged as latent.

---

## Summary

All eight prior callouts the user explicitly flagged are addressed in substance:

| # | Item                                       | Pass-3 status         |
|---|--------------------------------------------|-----------------------|
| 1 | 0236 stub                                  | Resolved              |
| 2 | 0021 `sorted()` merge                      | Resolved              |
| 3 | 0100 `treeToString` impl                   | Resolved              |
| 4 | 0146 capacity-1 on both sides              | Resolved              |
| 5 | 0103 canonical empty-list contract         | Resolved              |
| 6 | 1980 semantic asserts + canonical algo     | Resolved              |
| 7 | 0160 misleading                            | Partially resolved (Medium) |
| 8 | 0094 dead `expected_empty`                 | Resolved              |

PASS=411 with all-`embedded_asserts` manifest is now backed by real algorithmic coverage on the previously-stubbed slug (0236), real merge logic on 0021, structural recursion on 0100, the canonical Cantor diagonal on 1980, and the canonical LC 103 empty-list contract on both sides. The single residual algorithmic concern (F7 / 0160) is honestly documented now; the remaining items are style or "explain the workaround inline" notes that are safe to land in a follow-up.
