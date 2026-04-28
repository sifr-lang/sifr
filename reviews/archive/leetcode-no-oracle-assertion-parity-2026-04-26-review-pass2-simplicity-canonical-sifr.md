# LeetCode NO_ORACLE Assertion Parity — Review Pass 2 (2026-04-26)

## Lens

Pass 1 asked "do the assertions actually exercise the function?" — this pass asks something different. The promoted fixtures are now claiming `embedded_asserts` parity. Parity should mean: the Sifr fixture mirrors the *intent* and *simplicity* of the paired Python fixture while still being canonical, idiomatic Sifr — not "we made the assertion line up by adding workarounds, weakening contracts, or stubbing out the algorithm."

So the questions for each touched fixture are:

1. Did the Sifr port stay close in spirit to the Python pair (same algorithm, comparable shape)?
2. Where Sifr's type/ownership rules forced a different shape, is the divergence the *minimal* canonical-Sifr expression — or did it grow workaround scaffolding?
3. Did the assertion edit accidentally trade real coverage / canonical contracts for a passing run?

## Scope

- 14 modified `.sifr` fixtures and 1 modified `.py` fixture under `audits/leetcode/`.
- `verification/leetcode/full_corpus_manifest_20260402_live.json`: 203 entries flipped from `no_oracle` → `embedded_asserts`. `jq` confirms 411 cases, all unique slugs, all `embedded_asserts`. The diff is mode-only — `grep -v '"mode"'` on the diff body is empty (203 `+` mode lines, 203 `-` mode lines). All 15 source-edited slugs are present and now `embedded_asserts`.

I did not edit any files. Findings are listed by severity, with file/line references.

---

## Findings

### F1 — `0236_lowest_common_ancestor_of_a_binary_tree.sifr`: solver replaced by a `return None` stub to chase Python's bug — Severity: **Critical**

[audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:3-4](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:3):

```sifr
def lowestCommonAncestor(root: TreeNode | None, p: int, q: int) -> int | None:
    return None
```

The real recursive LCA implementation has been renamed to `lowestCommonAncestorValue` ([…sifr:6-25](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:6)) and is **never called from `main()`**. All three asserts compare to `None` and pass trivially.

Why this is the worst kind of "parity": the Python fixture ([0236…py:5-22](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.py:5)) is itself broken — it compares `root == p` between a `TreeNode` and an `int`, never matches, and falls through to `return None`. Rather than fix the Python pair to take `TreeNode` arguments (the canonical LC 236 signature), the Sifr was demoted to a stub so its asserts also yield `None`. The result:

- Sifr now has zero functional coverage of LCA logic for fixture 0236.
- The dead `lowestCommonAncestorValue` body is load-bearing only for `cargo clippy`/HIR coverage.
- PASS=411 conceals a stub.

Canonical-Sifr remediation: keep `lowestCommonAncestor` as the recursive algorithm and rewrite the Python pair to call it with `TreeNode` args (or to compare on values via `node.val`). Either way, the stub-and-rename is the wrong direction.

---

### F2 — `0021_merge_two_sorted_lists.sifr`: "merge two sorted lists" replaced with collect-concat-`sorted()` — Severity: **High**

[audits/leetcode/0021_merge_two_sorted_lists.sifr:6-31](audits/leetcode/0021_merge_two_sorted_lists.sifr:6) walks both inputs into `vals1`/`vals2`, concatenates, calls `sorted()`, then rebuilds a `ListNode` chain by popping from the tail. The Python pair ([0021…py:23-30](audits/leetcode/0021_merge_two_sorted_lists.py:23)) is the canonical 7-line recursive merge.

This isn't merge — it's a sort. The function under test is no longer the algorithm the file claims to demonstrate. Sifr can express the recursive (or iterative dummy-node) merge fine; the helpers in [helpers/list_node.sifr](audits/leetcode/helpers/list_node.sifr) (`nodeVal`, `nodeNext`, `hasNode`) plus `ListNode` constructor allow a faithful port. Replacing the algorithm with `sorted()` is the maximum loss of canonical intent.

Secondary smell: `sampleListA`, `sampleListB`, `singleZeroList` ([…sifr:34-43](audits/leetcode/0021_merge_two_sorted_lists.sifr:34)) are now dead. The `main()` diff inlined every call site to mirror the Python literal-construction style, but the helpers were left in. Either delete them or use them.

Tertiary smell: the file imports `nodeVal`, `nodeNext`, `hasNode` for what could be `cur is not None` and `cur.val`/`cur.next`. Wrapper-heavy style is unnecessary indirection here — inside this file the `ListNode | None` type is in scope, so direct field access is canonical Sifr.

---

### F3 — `0100_same_tree.sifr`: structural recursion replaced by `treeToString(p) == treeToString(q)` — Severity: **High**

[audits/leetcode/0100_same_tree.sifr:3-4](audits/leetcode/0100_same_tree.sifr:3):

```sifr
def isSameTree(p: TreeNode | None, q: TreeNode | None) -> bool:
    return treeToString(p) == treeToString(q)
```

The Python pair ([0100…py:5-11](audits/leetcode/0100_same_tree.py:5)) is the canonical 4-line recursive walk:

```python
if not p and not q: return True
if p and q and p.val == q.val:
    return isSameTree(p.left, q.left) and isSameTree(p.right, q.right)
return False
```

That shape ports directly to Sifr — `TreeNode | None` plus `is None` checks plus `node.val`/`.left`/`.right`. The string-based shortcut:

- hides the algorithm under a serialization helper,
- couples correctness to [helpers/tree_node.sifr:17-20](audits/leetcode/helpers/tree_node.sifr:17)'s `treeToString` format (drop the `None` markers and equality silently breaks for some mirror-symmetric trees),
- allocates a string of size O(N) per side per call.

The helper is fine for *assertions* (the rest of the corpus uses `treeToString(...)` to compare expected vs actual structurally). Using it as the *implementation* of `isSameTree` is the wrong abstraction. Recommend rewriting the body as the canonical recursion; the asserts already exercise the third "mirror values, different shape" case so the recursive port would pass without any test changes.

---

### F4 — `0146_lru_cache.sifr`: capacity-1 case dropped while the implementation is already a heavy ownership workaround — Severity: **High**

[audits/leetcode/0146_lru_cache.sifr:127](audits/leetcode/0146_lru_cache.sifr:127). The diff removes six lines covering the `LRUCache(1)` capacity-edge eviction path. The surviving capacity-2 case never exercises the "evict on every put" branch in [audits/leetcode/0146_lru_cache.sifr:88-114](audits/leetcode/0146_lru_cache.sifr:88).

Pass-1 F4 noted this as a coverage trim. The simplicity-lens issue is sharper:

- Python ([0146…py:5-36](audits/leetcode/0146_lru_cache.py:5)) is ~30 lines: a tiny `Node` class + dict-of-nodes + sentinel `left`/`right` doubly-linked list with direct `.prev`/`.next` mutation.
- Sifr ([…sifr:3-114](audits/leetcode/0146_lru_cache.sifr:3)) is ~115 lines: 5 separate dicts (`key_to_node`, `node_key`, `node_value`, `prev`, `next`) keyed on synthetic node IDs, with explicit re-store-of-self-fields (`self.prev = prev`, etc.) after every mutation.

The 5-dict design is a workaround for whatever ownership/cyclic-reference constraint blocks the natural `Node` class. That is fine as a one-time cost — but trimming the capacity-1 test on top of that means the Sifr fixture is now both *more complex than* the Python pair *and* tests less than it. The trim narrows the gap between the verbose Sifr and a regression that fakes-passes a put-evict-immediately bug.

Decisions to make explicitly:
- If 0146 is going to keep the dicts-of-ints design, the file deserves at least a one-line comment at [audits/leetcode/0146_lru_cache.sifr:3](audits/leetcode/0146_lru_cache.sifr:3) explaining *why* (ownership-driven, no `Node` cycles), so future readers don't try to "simplify" it back to the natural shape and find out the hard way.
- Reinstate the capacity-1 case (it's six lines; mirrors no Python expectation but adds a real branch) **or** add a one-liner in `internal_docs/` declaring "Sifr ≡ Python" parity stance.

---

### F5 — `0235_lowest_common_ancestor_of_a_binary_search_tree.sifr`: `cloneTree` per call is an ownership workaround that masks the canonical shape — Severity: **Medium**

[audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:8-28](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:8). Three things drift from the Python pair's simple iterative descent ([0235…py:5-14](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.py:5)):

1. The Sifr is recursive instead of iterative-`while True`. Sifr supports `while True` (1980 uses it) and can use BST short-circuit identical to Python.
2. Every "found" branch returns `cloneTree(root)` instead of `root` ([…sifr:21,26,28](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:21)). This is a workaround for Sifr's ownership rules — returning the same node would alias an in-tree node out of the input. The cost: each LCA call now allocates O(subtree) at the answer node.
3. `nodeVal(p: TreeNode | None) -> int` ([…sifr:3-6](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:3)) returns 0 for `None`. The Python takes non-Optional `TreeNode` arguments. The Sifr's defensive `if root is None: return None` plus `nodeVal(p)`/`nodeVal(q)` invent semantics for `lowestCommonAncestor(None, …)` and `lowestCommonAncestor(root, None, None)` that the LC contract doesn't define. Either lift the signature to non-Optional `TreeNode` (matches Python and LC) or comment why the defensive shape is required.

The fixture is correct for the three asserted inputs (verified: `(2,8)→root`, `(2,4)→subtree at 2`, `(2,1)→TreeNode(2,1)`). The complaint is shape, not correctness — the canonical Sifr port is "iterative descent, return `root` directly, signature is `TreeNode → TreeNode`" and the cloning is residual fight-with-ownership that should be either justified inline or eliminated by storing the answer's value (LC 235 actually only needs to identify the LCA — node identity is a Python convenience).

---

### F6 — `0103_binary_tree_zigzag_level_order_traversal.sifr`: return-type weakened to mirror Python's bare `return` — Severity: **Medium**

[audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:11-13](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:11):

```sifr
def zigzagLevelOrder(own root: TreeNode | None) -> list[list[int]] | None:
    if root is None:
        return None
```

The Python pair's bare `return` ([0103…py:6-7](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.py:6)) yields `None` — a Python-specific quirk that drifts from LC 103's contract (`[]` for empty root). The Sifr previously returned `[]` and the contract-correct `len(zigzagLevelOrder(None)) == 0` assertion. To match the Python None-equality assertion, the Sifr signature was widened to `list[list[int]] | None`.

This is parity bought by:

- copying a Python bug,
- weakening the Sifr type so every consumer of `zigzagLevelOrder` must unwrap `Optional`,
- diverging from LC 103's published contract.

Canonical Sifr would be `-> list[list[int]]`, `return []`, and assert `zigzagLevelOrder(None) == []`. Update the Python pair to match (it's a one-line `return []` change). Saves the `| None` variance everywhere downstream.

Secondary smell, [audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:4-10](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:4): `nodeValue` checks `value is None` after `node.val`, but `TreeNode.val` is typed `int` ([helpers/tree_node.sifr:2](audits/leetcode/helpers/tree_node.sifr:2)), so that branch is unreachable. Defensive code with no purpose; either remove the helper and inline `node.val` after the existing `node is None` guard at [audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:22](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:22), or shrink it to one ternary line.

---

### F7 — `0160_intersection_of_two_linked_lists.sifr`: identity-substitute by tail-value walk plus a single-input `pop()` patch — Severity: **Medium**

[audits/leetcode/0160_intersection_of_two_linked_lists.sifr:35-72](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:35). Pass 1 F2 covered the algorithmic risk. The simplicity/canonical lens adds:

- The `pop()` at [audits/leetcode/0160_intersection_of_two_linked_lists.sifr:62-63](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:62) is a one-input-tuned correction; nothing in the file documents that it only handles "false-suffix-of-length-1." A reader cannot derive the invariant from the code.
- The Python pair ([0160…py:17-24](audits/leetcode/0160_intersection_of_two_linked_lists.py:17)) is the canonical 5-line two-pointer trick, relying on identity. Sifr can't do that, granted — but the comment at [audits/leetcode/0160_intersection_of_two_linked_lists.sifr:3-4](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:3) ("Boundary fixture: canonical intersection depends on shared tail identity, so this fixture keeps its local ListNode shape inline") explains *why the class is local* but does not warn that *the algorithm itself* is an approximation tuned to this fixture.

If the team accepts this as a "boundary fixture," at minimum the algorithm body should carry a one-line comment explaining the value-walk-with-pop-one is an identity-substitute and is not generally correct (e.g. fails for lists where the false-suffix overlap exceeds 1 node). Otherwise a future contributor will read `getIntersectionNode` as a reference implementation.

A faithful identity-substitute exists: assign each node a unique `id` field on construction, walk both lists into `set[int]` of ids, return the node whose id first appears in both. More code, but generally correct. The current shape trades correctness for line-count.

---

### F8 — `1980_find_unique_binary_string.sifr`: canonical Cantor-diagonal swapped for an incremental-flip search — Severity: **Medium**

[audits/leetcode/1980_find_unique_binary_string.sifr:3-17](audits/leetcode/1980_find_unique_binary_string.sifr:3). The previous Sifr port was the canonical LC 1980 trick: walk the diagonal of `nums`, flip each bit, return the result. That solution is 4-5 lines, one-shot O(n), guaranteed unique, and obvious from the LC problem statement.

The new algorithm:

```sifr
candidate = ["0"] * len(nums)
while True:
    result = "".join(candidate)
    if result not in nums: return result
    if index < 0: return result
    candidate[index] = "1"
    index -= 1
```

This is a special-shape exhaustive search: it tries `00...0`, `00...01`-flipped-from-the-right (i.e. `00...01` becomes `00...10`?, actually it flips the *rightmost* bit toward `1` and walks left). What it actually enumerates is `n+1` strings: `00...0`, `00...01`, `00...011`, `00...0111`, …, `11...1` — each step turns one more rightmost-zero into a `1`. By pigeonhole on `n+1` candidates vs `≤n` strings in `nums`, one must be missing.

Problems:

1. **Non-canonical for LC 1980.** Cantor diagonalization is the textbook reason this problem exists. Replacing it loses the pedagogical purpose of the fixture.
2. **Assertions are now algorithm-deterministic, not LC-correct.** The Python pair was changed from `assert ans not in nums` (semantic check — any valid answer passes) to `assert findDifferentBinaryString(['01','10']) == '00'` (literal — only this exact algorithm output passes). That coupling makes both fixtures break if anyone "fixes" the algorithm to the canonical diagonal. The previous semantic assertion was *more parity-friendly*, not less.
3. **Unreachable trailing `return ""`** at [audits/leetcode/1980_find_unique_binary_string.sifr:17](audits/leetcode/1980_find_unique_binary_string.sifr:17). Pigeonhole guarantees the `result not in nums` check fires before `index < 0`. Dead line.
4. **Subtle correctness reasoning hidden.** A reader has to derive the pigeonhole argument to convince themselves the function terminates with a unique string. The diagonal version was self-evidently correct.

The only reason I can find for this rewrite is that the previous Sifr code did `bit: str | None = row[i]` and `if bit is not None and bit == "0"` — which is exactly the Sifr-canonical way to handle string-indexing's `Optional` return. Both that and the test `result not in nums` (used in the new code) are valid Sifr. The new code's shift is a regression in clarity, not a forced choice.

Recommended: restore the diagonal form, keep the Python pair's literal `'01'/'10' → '00'` assertions (they happen to coincide with what the diagonal produces for these inputs: row 0 column 0 is `'0'` → flip to `'1'`? — actually no, diagonal produces `'1'` then `'1'` → `'11'` for `['01','10']`, so the literals would need to be `'11'`, `'10'`, `'010'` or similar). Or restore the pre-edit semantic asserts (`len(ans) == len(nums) and ans not in nums`) on both sides — that's the most parity-friendly form and the one that actually mirrors LC's "any valid answer."

---

### F9 — `0706_design_hashmap.py` and `.sifr`: rewriting Python down to mirror the verbose Sifr instead of the other direction — Severity: **Medium**

[audits/leetcode/0706_design_hashmap.py](audits/leetcode/0706_design_hashmap.py) was rewritten from the project-canonical "linked-list-of-buckets" using `helpers.list_node` to a `list[list[tuple[int, int]]]` shape that mirrors the Sifr ([audits/leetcode/0706_design_hashmap.sifr](audits/leetcode/0706_design_hashmap.sifr)).

The new Python:

- drops the `from helpers.list_node import …` line, so 0706 is now the only fixture in the corpus that uses neither the linked-list helper nor the canonical chained-bucket pattern;
- exists primarily to mirror the Sifr's chosen data layout, not to be a clean Python reference.

Meanwhile the Sifr has its own simplicity hits ([…sifr:27-47](audits/leetcode/0706_design_hashmap.sifr:27)):

- `put` always rebuilds the entire bucket into a `next_bucket` list instead of mutating in place. The "found and replaced" case rebuilds; the "not found, append" case rebuilds; both then re-store `self.buckets`. A direct loop with `bucket[index] = (key, value); return` is canonical Python *and* canonical Sifr — the rewrite-everything pattern reads as "fight ownership."
- `remove` does the same rebuild ([…sifr:65-72](audits/leetcode/0706_design_hashmap.sifr:65)).
- `self.buckets = buckets` after `buckets[index] = next_bucket` ([…sifr:45-47, 70-72](audits/leetcode/0706_design_hashmap.sifr:45)) is ceremonial — `self.buckets` is the same reference. Either Sifr's mutation rules require it (then a comment is owed) or it's leftover defensiveness.

Pass 1 F10 noted the negative-key handling at [audits/leetcode/0706_design_hashmap.sifr:23-24](audits/leetcode/0706_design_hashmap.sifr:23) is missing on the Python side — that gap remains and is now a *latent divergence* introduced by adopting the Sifr's defensive `index < 0` branch only on one side.

The lopsided direction-of-rewrite is the issue: parity should normally pull *Sifr* toward the simpler Python, not pull *Python* toward verbose Sifr.

---

### F10 — `0102_binary_tree_level_order_traversal.sifr`: recursive level-merge instead of BFS, despite 0103 successfully using BFS — Severity: **Low**

[audits/leetcode/0102_binary_tree_level_order_traversal.sifr:3-20](audits/leetcode/0102_binary_tree_level_order_traversal.sifr:3) implements level-order by recursing into left and right subtrees and zipping their level lists. The Python pair ([0102…py:7-24](audits/leetcode/0102_binary_tree_level_order_traversal.py:7)) is BFS with `collections.deque`.

[audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:14-34](audits/leetcode/0103_binary_tree_zigzag_level_order_traversal.sifr:14) uses BFS with `q.pop(0)` — proving BFS is expressible in Sifr. So 0102's recursive merge is a stylistic choice, not a forced one. It costs O(N·H) work vs. BFS's O(N), and creates inconsistency between two fixtures that should have parallel shapes.

Lower severity because correct and the test inputs are tiny, but worth aligning for future readability.

Secondary nit: [audits/leetcode/0102_binary_tree_level_order_traversal.sifr:23-26](audits/leetcode/0102_binary_tree_level_order_traversal.sifr:23) declares `expected_empty: list[list[int]] = []` and uses it on line 26, while [audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19-20](audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19) declares the same binding but compares to a literal `[]` instead — leaving the binding unused (also flagged in pass 1 F5). Pick one style and apply it consistently across the corpus.

---

### F11 — `0226_invert_binary_tree.sifr`: split into `invertTree` + `invertNode` for ownership reasons — Severity: **Low**

[audits/leetcode/0226_invert_binary_tree.sifr:3-25](audits/leetcode/0226_invert_binary_tree.sifr:3) defines two functions where the Python pair ([0226…py:5-15](audits/leetcode/0226_invert_binary_tree.py:5)) defines one. The Sifr `invertNode` takes `own mut node: TreeNode` (non-Optional) and the outer `invertTree` handles the `None` case by short-circuiting. Functionally equivalent to a single function that branches on `is None`, which would more directly mirror Python's:

```python
if not root: return None
root.left, root.right = root.right, root.left
invertTree(root.left)
invertTree(root.right)
return root
```

The Sifr version does the swap by reassigning fields (`node.right = invertNode(left)`, `node.left = invertNode(right)`) — which is the right move because Sifr probably can't do tuple-swap on object fields in a single line. Splitting into two functions to avoid threading `None` through the body is a small workaround; could be a single function with `if node is None: return None` at the top. Not blocking — just a slight loss of one-to-one mirroring with the canonical Python.

---

### F12 — `1203_sort_items_by_groups_respecting_dependencies.sifr`: helper-wrapper boilerplate around list indexing — Severity: **Low**

[audits/leetcode/1203_sort_items_by_groups_respecting_dependencies.sifr:3-23](audits/leetcode/1203_sort_items_by_groups_respecting_dependencies.sifr:3) defines `unwrapInt`, `getIntAt`, `setIntAt`, `getBucket`, `appendEdge` to wrap basic list reads/writes that return `T | None` in Sifr. Python's plain `list[i] += 1` becomes 3 lines (`new_degree = getIntAt(...) - 1; setIntAt(...)`).

This is a fundamental stdlib-shape difference between the languages, not a fixture-author choice — flagging it because *if* Sifr ever exposes a `[]` operator that returns `T` (panicking on out-of-bounds, which the project disallows) or a sugar like `list[i] ?? 0`, this fixture should be the first to adopt it. Until then, the verbosity is canonical Sifr, just not canonical to the LC problem.

The single assertion (pass 1 F11) — `assert topologicalSort([[0]], [0], 0) == []` with `num_nodes=0` — is degenerate (the for-loop never runs, the empty `order` trivially satisfies the post-condition). Mirrors the Python pair's equally degenerate test. The whole `sortItems` function — 50 lines of careful work — is never invoked. If the parity contract permits "Sifr ⊇ Python," adding one non-degenerate `sortItems(n=…, m=…, group=…, beforeItems=…)` assertion would convert this from "passes" to "actually tested."

---

### F13 — `0094_binary_tree_inorder_traversal.sifr`: leftover unused `expected_empty` — Severity: **Low**

[audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19-20](audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19):

```sifr
expected_empty: list[int] = []
assert inorderTraversal(None) == []
```

The diff replaced `== expected_empty` with `== []` but left the binding declaration. Dead local. Trivial; flagged only because other fixtures in the same series are inconsistent ([0102](audits/leetcode/0102_binary_tree_level_order_traversal.sifr:23) keeps both declaration *and* use; [0212](audits/leetcode/0212_word_search_ii.sifr:75) keeps both; [0094](audits/leetcode/0094_binary_tree_inorder_traversal.sifr:19) keeps only the dead declaration). Pick one style across the corpus.

---

### F14 — `0212_word_search_ii.sifr`: heavy threading of `found` and `visit` through recursion — Severity: **Low** (informational)

[audits/leetcode/0212_word_search_ii.sifr:10-45](audits/leetcode/0212_word_search_ii.sifr:10). `collectWords` takes 10 parameters, threads `found: dict[str, bool]` as a return value through every recursive call, and uses a separate `Trie` arena helper from [helpers/trie.sifr](audits/leetcode/helpers/trie.sifr).

The Python ([0212…py:12-34](audits/leetcode/0212_word_search_ii.py:12)) is a closure over `res`, `visit`, `root`, `board` — short and obvious because it captures by reference. Sifr can't do that the same way; the parameter-threading + arena-Trie style is the canonical Sifr equivalent and is internally consistent with the rest of the corpus. Flagged only as the reason this fixture is ~80 lines vs Python's ~30 — fundamental, not a workaround introduced by this PR.

The `expected_empty: list[str] = []` at [audits/leetcode/0212_word_search_ii.sifr:75](audits/leetcode/0212_word_search_ii.sifr:75) is the third style variant in this PR — keep one, drop the others.

---

### F15 — Manifest hygiene — Severity: **None** (positive)

411 cases, all unique `fixture_slug`s, all `oracle.mode == "embedded_asserts"`. Diff body is mode-flip-only (203 `+ "mode": "embedded_asserts"` lines, 203 `- "mode": "no_oracle"` lines, no other field churn). All 15 source-edited slugs are present and now `embedded_asserts`. No accidental edits.

PASS=411 from `run_phase31_leetcode.py` (per pass-1 context) is consistent with the manifest count, but the pattern in F1 (stub returning a constant that the asserts check for) means PASS-count alone does not verify "every fixture exercises its solver." A corpus-level lint that flags `assert f(...) == K` where `f` has `return K` as its only top-level body would catch the F1 shape.

---

## Residual risks / test gaps (simplicity + canonicality lens)

1. **F1 is a stub-driven false-pass.** The fixture promotes to `embedded_asserts` while the algorithm is unreachable from `main()`. Highest-priority fix.
2. **F2/F3/F8 are canonical-algorithm regressions.** 0021 became sort-and-rebuild, 0100 became string-equality, 1980 became enumerate-with-pigeonhole. None of those changes were forced by Sifr's type system; they're stylistic detours that lose the algorithm's identity. PRs from contributors learning Sifr will copy these patterns elsewhere unless rolled back.
3. **F4/F5/F9 are workarounds without comments.** The verbosity is real (ownership/cyclic-reference constraints), but the *reason* is invisible. The fix is cheap: a 1-2 line comment at the top of each fixture explaining "this shape is forced by X" prevents future readers from "simplifying" them back into broken canonical shape.
4. **F6 trades canonical LC 103 contract for parity with a Python bug.** Should be tracked somewhere durable (e.g. `internal_docs/phases/`) so a future "fix the contract" PR doesn't unknowingly break parity.
5. **F7 fixture-tuned algorithm.** The boundary-fixture comment at [audits/leetcode/0160_intersection_of_two_linked_lists.sifr:3-4](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:3) explains the local `ListNode` but not the value-walk-plus-pop. Add a one-line caveat.
6. **F9 reverse-pull.** Rewriting Python down to match Sifr's verbosity (rather than tightening Sifr) is a directional anti-pattern. If 0706 is a precedent, future fixtures will follow it. Worth deciding stance explicitly.
7. **F10/F11/F12/F13/F14 are style inconsistencies.** Three different `expected_empty` patterns; two BFS shapes (recursive 0102 vs queue 0103); two LCA shapes (recursive 0235 vs Python's iterative); two inversion shapes (split 0226 vs single-function Python). Each individually is a Low. Together they make the `audits/leetcode/` corpus harder to read because the same Python pattern is rendered three ways across neighboring files.
8. **No corpus-level lint catches F1's shape.** Recommend a `verification/` script: for each `embedded_asserts` fixture, parse the function body of every assert's left-hand-side call, reject if its body is exactly `return <literal>` and any assert compares to that same literal. ~50 lines of Python, prevents this class of regression.
9. **Pass-1 carryovers still relevant.** F4 (capacity-1 drop), F5 (cloneTree allocations), F11 (degenerate `topologicalSort([[0]], [0], 0)`), and the dead `sampleListA`/`B`/`singleZeroList` helpers in 0021 remain unaddressed in this round.
10. **Local-validation breadth unverified.** No evidence in the working tree that `scripts/run_all_tests.sh --profile quick` was run. Per [AGENTS.md](AGENTS.md), that's the authoritative gate. Clippy in particular may flag F13's dead local under workspace pedantic lints.

---

## Suggested order of operations

1. (Critical) F1: restore `lowestCommonAncestor` to the real recursive solver in 0236; repair the Python pair to call it with proper `TreeNode` arguments.
2. (High) F2, F3, F8: revert to canonical shapes — recursive merge for 0021, recursive walk for 0100, diagonal flip for 1980. Re-derive expected literals from the canonical algorithm if necessary.
3. (High) F4: either reinstate the capacity-1 LRU case or add an explicit "Sifr ≡ Python" stance note + a comment explaining the dicts-of-ints workaround.
4. (Medium) F5, F6, F7, F9: add inline comments explaining the workaround in each (ownership for 0235, parity-with-Python-bug for 0103, value-walk-tuned-to-input for 0160, mirror-Sifr-shape-on-Python-side for 0706). Decide F6 contract stance and document.
5. (Low) F10–F14: pick consistent styles for `expected_empty`, BFS-vs-recursive level traversal, and `invertTree` shape; clean up dead helpers in 0021 and the dead local in 0094.
6. Run `scripts/run_all_tests.sh --profile quick` before merge.
