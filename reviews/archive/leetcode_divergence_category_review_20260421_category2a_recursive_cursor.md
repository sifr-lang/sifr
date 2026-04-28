# Category 2a Review — Recursive Node / Cursor Ergonomics

Date: 2026-04-21
Source: `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`
Scan data: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Scope: only Category 2a ("Recursive Node / Cursor Ergonomics").

## Summary

The category is directionally correct: the bulk of these fixtures diverge from canonical Python primarily because cursor-style traversal and recursive child access through `T | None` are expensive to express safely in Sifr today. Evidence: every linked-list fixture in this bucket carries the same four hand-rolled helpers — `nodeVal`, `nodeNext`, `hasNode`, `unwrapInt` — and every tree fixture carries a matching `treeVal`/`treeLeft`/`treeRight`/`hasTreeNode` set or inlined `is None` guards that serve the same purpose. That repetition is the signature of the ergonomics gap and it belongs in 2a.

However, several fixtures carry a primary divergence driver that is **not** recursive-node narrowing. Three sub-problems are getting folded in:

1. Collection/index Optional-flow on `list.pop(0)` / string indexing (should be 2b).
2. Stdlib parity gaps (notably: no `int(str)` parser).
3. Shared-ownership / aliasing on recursive node structures (distinct from narrowing; closer to an architecture boundary).

The "What should improve" bullets also need sharpening — one is confusingly worded and one big pattern is missing.

## Classification check — per fixture

Metrics below are from [leetcode_pair_diff_scan_20260409.json](verification/leetcode/leetcode_pair_diff_scan_20260409.json). `delta` is `length_delta`; `sim` is `similarity_ratio`.

| Stem | Total | Delta | Sim | Verdict |
|---|---|---|---|---|
| 0002_add_two_numbers | 124 | 32 | 0.43 | ✅ Keep. Textbook dummy-head cursor pattern replaced with list-accumulate-then-rebuild. |
| 0019_remove_nth_node_from_end_of_list | 107 | 33 | 0.34 | ✅ Keep. Two-pointer cursor + delete. |
| 0021_merge_two_sorted_lists | 121 | 31 | 0.32 | ✅ Keep. Dummy-head cursor merge. |
| 0025_reverse_nodes_in_k_group | 130 | 32 | 0.29 | ✅ Keep. Sub-chain reverse with rewiring across Optional cursors. |
| 0061_rotate_list | 124 | 36 | 0.31 | ✅ Keep. Tail-seek + rewire. |
| 0083_remove_duplicates_from_sorted_list | 98 | 40 | 0.34 | ✅ Keep. In-place `.next` skip. |
| 0086_partition_list | 105 | 29 | 0.33 | ✅ Keep. Two dummy-head cursors. |
| 0092_reverse_linked_list_ii | 123 | 35 | 0.34 | ✅ Keep. Sub-range in-place reverse. |
| 0143_reorder_list | 126 | 34 | 0.31 | ⚠️ Keep, but this is a shared-ownership case as well — canonical is slow/fast → reverse back-half → interleave, which needs either splitting ownership mid-chain or reading a tail view you don't own. Narrowing help alone will not close the gap. |
| 0160_intersection_of_two_linked_lists | 113 | 51 | 0.36 | ❌ **Move to Category 1 (rewrite debt).** Sifr solves a different problem: it finds the longest common value-suffix, so `ListNode(4)→ListNode(1)→shared` and `...→ListNode(1)→shared` are treated as intersecting at the `1`, which is algorithmically wrong per LeetCode 160. The Sifr test assertion (`"1->8->4->5"`) diverges from the canonical (`"8->4->5"`). The two-pointer pivot-on-tail algorithm needs shared references to the other list's head, which is an aliasing issue, not a narrowing issue. |
| 0203_remove_linked_list_elements | 100 | 30 | 0.35 | ✅ Keep. Classic dummy-head filter-in-place. |
| 0234_palindrome_linked_list | 105 | 17 | 0.35 | ⚠️ Keep, but note the asymptotic trade-off: Sifr flattens to a `list[int]` (O(n) space) while canonical Python does slow/fast → reverse-in-place → compare (O(1) space). The flatten approach is still a valid canonical shape, so 2a placement is defensible, but the in-place variant will remain blocked on cursor-mutation ergonomics even after narrowing is fixed. |
| 0297_serialize_and_deserialize_binary_tree | 123 | 63 | 0.42 | ❌ **Move to 2b (with a stdlib-parity note).** The dominant divergence is *not* recursive narrowing; it's a hand-rolled `digitValue`/`parseIntToken` replacing `int(str)` (stdlib parity) plus string indexing returning `str | None` (`first = token[0]`) and `vals.pop(0)` returning `str | None` (collection/index Optional-flow). The recursive `dfs` is actually fine. |
| 0450_delete_node_in_a_bst | 83 | 15 | 0.48 | ✅ Keep. Pure narrowing-after-`is None` on `root.left`/`root.right`, with recursive re-assignment already working. One of the cleanest 2a exemplars. |
| 0513_find_bottom_left_tree_value | 99 | 5 | 0.41 | ⚠️ Primary driver is 2b, not 2a. The divergence is driven by `q.pop(0)` on `list[TreeNode]` surfacing as Optional → dead `if node0 is None: continue` guard, and by the Python file having **two** implementations (one using `nonlocal`) which inflates the raw diff (Category 3/5 pattern). Consider moving to 2b or annotating with both. |
| 0662_maximum_width_of_binary_tree | 80 | 24 | 0.51 | ✅ Keep, mildly 2a/2b mixed. Child access needs narrowing, but `q.pop(0)` Optional also applies. |
| 0669_trim_a_binary_search_tree | 87 | 31 | 0.45 | ✅ Keep. Uses the `treeVal/treeLeft/treeRight/hasTreeNode` helper quartet that embodies exactly the 2a pain. |
| 0876_middle_of_the_linked_list | 98 | 38 | 0.35 | ⚠️ Shared-ownership more than narrowing. Canonical Python returns the middle node as a *sub-chain view* of the input; Sifr cannot return an un-owned tail, so it flattens and rebuilds. This belongs in a "shared ownership for recursive nodes" bullet under 2a, or arguably in Category 4. |
| 0894_all_possible_full_binary_trees | 88 | 12 | 0.48 | ❌ **Move to Category 4 (intentional architecture boundary)** or carve out a dedicated "shared recursive-node ownership" bullet. The divergence is `cloneTree` calls forced by Sifr's single-ownership when the same sub-tree is spliced into multiple output trees. `is not None` narrowing does nothing here; only shared ownership (Rc/Arc) or structural sharing would close the gap, which is a distinct language decision. |
| 1609_even_odd_tree | 85 | 21 | 0.49 | ⚠️ Keep, but primary driver is 2b. `q.pop(0)` Optional guard is the main noise; child narrowing is secondary. |
| 1669_merge_in_between_linked_lists | 120 | 50 | 0.33 | ✅ Keep. Splice-in rewiring through `.next` chains. |
| 1721_swapping_nodes_in_a_linked_list | 109 | 39 | 0.32 | ✅ Keep. Index-walk + value swap across cursors. |
| 2130_maximum_twin_sum_of_a_linked_list | 104 | 32 | 0.35 | ✅ Keep. Flatten + index pair-sum; mirrors the palindrome shape. |

### Moves and adjustments

- **0297_serialize_and_deserialize_binary_tree** → move to 2b, and add it to the "stdlib parity" list under "What should improve" as evidence that a string→int parser is missing.
- **0513_find_bottom_left_tree_value** → move to 2b (or dual-tag 2a+2b). Also add a Category 5 note: Python file has two implementations including a `nonlocal`-based one; raw diff is inflated by corpus noise.
- **0160_intersection_of_two_linked_lists** → move to Category 1 (rewrite debt). The Sifr version is not equivalent — it solves a value-suffix problem, not a node-identity intersection problem, and the test assertion reflects that mismatch. Flag as parity debt and, separately, mark the test value `"1->8->4->5"` as wrong relative to LeetCode 160 semantics.
- **0894_all_possible_full_binary_trees** → move to Category 4, or expand 2a with a dedicated "shared-ownership for recursive nodes" sub-bullet and keep it under 2a but tagged.
- **0876**, **0143**, **0234** → keep under 2a but annotate that the remaining gap after narrowing is an aliasing/shared-ownership question, not a narrowing question.

### Missing from 2a?

I looked at all linked-list and tree fixtures under `audits/leetcode/` and cross-checked against this category plus 1 and 3.

No additional fixtures should be pulled *into* 2a:

- `0138_copy_list_with_random_pointer` has been rewritten to `list[tuple[int, int]]` and no longer uses a linked list at all — that's parity debt (belongs in Category 1, and is currently unlisted there; worth surfacing separately).
- `0141_linked_list_cycle` is a trivial `return False` stub — corpus cleanup / Category 5 or rewrite debt, not a narrowing exemplar. Also under the 80-line cutoff.
- `0206_reverse_linked_list` was rewritten to operate on `list[int]` and is out of the linked-list bucket — parity debt, not 2a.
- Other tree fixtures (`0094`, `0098`, `0100`, `0101`, `0104`, `0226`, `0543`, `0572`, `0617`, …) sit at similarity ≥ 0.44 with small deltas, i.e., they are already close to the canonical Python shape and do not materially demonstrate 2a pain.

So the set of *additions* is empty; the fixes are all moves/annotations.

## Faithfulness of the "What should improve" bullets

The four bullets:

1. narrowing after `is not None`
2. compiler-preserved narrowing within a proven scope, including across rebinding when the new value is provably the same type; no user-side re-narrowing required
3. easier safe field access on recursive nodes
4. clearer cursor-style mutation patterns without weakening ownership

Assessment:

- **(1) and (3) are precise and faithful.** These are textbook flow-sensitive typing extensions and compose cleanly with Sifr's ownership model (narrowing is a read-side refinement; it does not grant additional aliasing or mutation rights).

- **(2) is imprecise.** The motivating case is `cur = cur.next` where `cur: ListNode | None`. After narrowing `cur` to `ListNode`, the RHS `cur.next` has static type `ListNode | None`, so the phrase "when the new value is provably the same type" does not apply to the canonical cursor walk. What the bullet seems to want is: "when a narrowed binding is re-assigned from an expression that is *itself* provably non-`None` at that point, the narrowed type survives" — or, separately, "repeated `is not None` checks after rebinding should not require ceremonial re-narrowing to local variables." I recommend splitting (2) into two bullets with concrete examples pulled from `0002` (dummy-head `cur = cur.next`) and `0083` (`cur.next = cur.next.next` when both halves are separately narrowed).

- **(4) is vague.** "Cursor-style mutation patterns" covers at least three distinct sub-patterns in these fixtures, and they have different ownership implications:
  - **Dummy-head trailing cursor** (e.g., `0002`, `0021`, `0086`): mutate `cur.next` through a cursor that does not own the chain. Today this forces list-accumulate-then-rebuild.
  - **In-place `.next` skip** (e.g., `0083`, `0203`): `cur.next = cur.next.next` under the invariant that both are non-`None`.
  - **Sub-range reverse / rewire** (e.g., `0025`, `0092`, `0206`): temporary unwiring and re-wiring across Optional boundaries.
  Spelling these out would make (4) actionable and let someone designing the ergonomics fix see which cases are closed by narrowing alone vs. which need a borrow-style cursor primitive.

- **Missing bullet — shared ownership for recursive nodes.** Several fixtures in the category (0143, 0160, 0234, 0876, 0894, and arguably 0019 at the edge) remain divergent even with perfect narrowing because the canonical solution needs two cursors/views into the *same* chain, or splices the same sub-tree into multiple result trees. Narrowing is orthogonal to this. Either (a) add a bullet acknowledging shared/structural sharing as a separate ergonomics question, or (b) move these fixtures out of 2a as indicated above. Silence here risks driving a narrowing fix that leaves half the bucket unchanged and then looking like the category was mis-sized.

- **Missing bullet — dead guard boilerplate on infallible field projection.** Every linked-list fixture in this bucket reimplements `nodeVal/nodeNext/hasNode/unwrapInt`. `unwrapInt` in particular (`value: int | None → int`) is called on `values[i]` where `values: list[int]` — i.e. the guard only exists because index access surfaces as Optional. That is strictly a 2b concern, but its *ubiquity across 2a fixtures* is what makes them look noisier than they are. Call this out explicitly so the 2a/2b interaction is clear.

## Sifr-side corpus noise (orthogonal observation)

Every fixture I read in this bucket ships the same ~25 lines of dead shared scaffolding: an unused `Node` class (the kitchen-sink variant with `next`/`random`/`left`/`right`/`neighbors`/`key`) and, for tree fixtures, an unused `treeToString` alternative. This is Sifr-side corpus noise and mirrors the Python-side noise already called out in Category 5. The Preconditions section of the analysis only names the Python side ("Some Python fixtures contain multiple full implementations"). Recommend adding a parallel bullet noting that Sifr fixtures also carry shared-helper boilerplate that inflates `changed_sifr_lines` without reflecting language divergence, and folding Sifr-side cleanup into the Practical Priority Order §1 ("Corpus normalization").

## Concrete edits

1. In the Category 2a fixture list, **remove** `0160_intersection_of_two_linked_lists`, `0297_serialize_and_deserialize_binary_tree`, `0513_find_bottom_left_tree_value`, and `0894_all_possible_full_binary_trees`. Re-home per the table above.

2. In Category 1, **add** `0160_intersection_of_two_linked_lists` with a one-line note that the Sifr test assertion itself drifts from LeetCode 160 semantics and should be reviewed during the rewrite. Consider also adding `0138_copy_list_with_random_pointer`, which is already a full rewrite but is not currently listed anywhere.

3. In Category 2b, **add** `0297_serialize_and_deserialize_binary_tree` and `0513_find_bottom_left_tree_value`, noting `0297` additionally depends on stdlib parity for `int(str)`.

4. In Category 4, **add** `0894_all_possible_full_binary_trees` with the rationale: single-ownership prevents aliasing shared sub-trees across multiple results; the `cloneTree` call is the architecture-boundary cost.

5. Replace the four 2a "What should improve" bullets with:

   - narrowing after `is not None` on local bindings and on `.field` projections of recursive nodes
   - narrowing preserved across rebinding when the RHS is *itself* provably non-`None` at the point of assignment (not merely when the static type is the same)
   - narrowing preserved across a re-check of the same binding without requiring copy-to-local ceremony
   - cursor-style mutation patterns, separated into: (a) trailing dummy-head cursor mutating `.next`, (b) in-place `.next` skip under a double-narrow, (c) sub-range rewire/reverse across Optional boundaries — each without weakening ownership
   - shared/structural recursion over owned chains and trees, treated as a distinct ergonomics question from narrowing (gates 0143, 0234, 0876 and is the ceiling for any narrowing-only improvement)

6. In the Preconditions section, add: "Sifr fixtures also carry shared-helper boilerplate (unused kitchen-sink `Node` class, repeated `nodeVal`/`nodeNext`/`hasNode`/`unwrapInt` or `treeVal`/`treeLeft`/`treeRight`/`hasTreeNode` helpers) that inflates `changed_sifr_lines` independently of real language divergence; treat it the same way as Python-side multi-implementation noise."

7. Under the retained Category 2a entries that still have shared-ownership residual after narrowing (`0143`, `0234`, `0876`), add a one-line annotation noting the residual so the priority order in §3 does not over-promise what narrowing alone will close.
