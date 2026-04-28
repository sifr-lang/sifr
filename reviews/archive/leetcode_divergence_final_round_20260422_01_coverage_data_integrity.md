# Review: LeetCode Divergence Decision Analysis — Coverage and Data Integrity

Date: 2026-04-22
Subject: `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`
Raw scan: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Source pairs: `audits/leetcode/`
Angle: coverage and data integrity (independent whole-analysis pass).

---

## Summary

Coverage at the declared cutoff is complete and mutually exclusive. Cited line counts and structural claims match the raw scan and source files. One clear omission and several smaller hygiene gaps are flagged below.

---

## 1. Does the analysis categorize every paired fixture with `changed_total_lines >= 80`?

**Yes, completely, for the declared scope.**

The scan has 395 paired fixtures; 53 are at or above the `changed_total_lines >= 80` cutoff. I cross-listed every above-cutoff stem against Categories 1, 2a, 2b, 3, and 4:

| Category | Listed | Of which `>=80` |
|----------|--------|-----------------|
| 1 (rewrite) | 10 | 7 |
| 2a (recursive/cursor) | 19 | 19 |
| 2b (collection/stdlib) | 21 | 21 |
| 3 (okay as-is) | 4 | 4 |
| 4 (arch boundary) | 2 | 2 |
| **Total primary** | **56** | **53** |

- `over_cutoff ∖ categorized = ∅` — every `>=80` fixture has a primary category.
- `categorized ∖ over_cutoff = {0138_copy_list_with_random_pointer, 0206_reverse_linked_list, 0295_find_median_from_data_stream}` — three below-cutoff Category 1 items (see §2).
- `over_cutoff = 53` matches the sum of above-cutoff cells in the table exactly.

---

## 2. Are below-cutoff manual exceptions explicitly justified and not arbitrary?

**Partially.** Justification is per-item substantive, but the scope bar is not visibly applied at the list level.

Below-cutoff exceptions actually present in the analysis:

| Stem | Category | total | py | sifr | similarity | Flagged as below-cutoff? |
|------|----------|-------|----|------|-----------:|--------------------------|
| 0138_copy_list_with_random_pointer | 1 | 65 | 56 | 9 | 0.085 | **No** |
| 0206_reverse_linked_list | 1 | 71 | 54 | 17 | 0.078 | **No** |
| 0295_find_median_from_data_stream | 1 | 56 | 26 | 30 | 0.317 | **No** |
| 0052_n_queens_ii | 4 (pattern) | 29 | 21 | 8 | 0.580 | Yes |
| 0543_diameter_of_binary_tree | 4 (pattern) | 63 | 31 | 32 | 0.526 | Yes |
| 0783_minimum_distance_between_bst_nodes | 4 (pattern) | 69 | 25 | 44 | 0.543 | Yes |
| 1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero | 4 (pattern) | 59 | 27 | 32 | 0.289 | Yes |

Category 4 handles this well (line 154): the below-cutoff items are explicitly demarcated as pattern notes, not escalations. The "Why" paragraph makes the scope-bar status visible.

Category 1 does not. The 10-item bullet list at lines 26–35 mixes 7 above-cutoff and 3 below-cutoff items with no visual distinction, and the opening scope rule (line 5) is the only pointer to "manual exceptions below the cutoff." A reader auditing against the raw scan would have to discover the below-cutoff status themselves. The per-fixture "Why" bullets (lines 39–48) describe the public-model change substantively, so each inclusion has a defensible reason — just not one tied to the scope bar.

Concrete edit: mirror the Category 4 demarcation in Category 1. For example, after line 35 add a line like "Below the `changed_total_lines >= 80` scope, included as explicit parity-debt because the Sifr fixture implements a different public model than LeetCode 138 / 206 / 295:" and move `0138`, `0206`, `0295` under it. That makes the exceptions auditable at a glance and preserves the per-item rationale already in the "Why" section.

---

## 3. Are any fixtures listed in multiple primary categories accidentally?

**No.** Set intersections of Categories 1, 2a, 2b, 3, and 4 are all empty. Category 5 is explicitly documented (lines 157–162) as a secondary label applied only to Category 3 fixtures, not a new category — that's correctly framed and not a data-integrity issue.

---

## 4. Do `changed_py_lines` / `changed_sifr_lines` numbers and rationale match the raw scan and source files?

**All four explicitly cited pairs match exactly**, and the structural claims verify against the source:

| Cite | Scan | File verification |
|------|------|-------------------|
| `0104 ... sifr=8 against py=74` | sifr=8, py=74 ✓ | [0104 Py](audits/leetcode/0104_maximum_depth_of_binary_tree.py) has three `maxDepth` defs (recursive, iterative-DFS, BFS), unused `Node` kitchen-sink class, and `tree_to_string` helper ✓ |
| `0130 ... sifr=26 against py=55` | sifr=26, py=55 ✓ | [0130 Py](audits/leetcode/0130_surrounded_regions.py) has primary DFS-with-set `solve` plus triple-quoted 3-pass alternate (lines 27–58) ✓ |
| `0200 ... sifr=21 against py=82` | sifr=21, py=82 ✓ | [0200 Py](audits/leetcode/0200_number_of_islands.py) stacks set-tracked DFS, in-place grid-mutation DFS, and BFS-via-deque ✓ |
| `0516 ... sifr=22 against py=61` | sifr=22, py=61 ✓ | Claim about `memo.get((i, j), -1)` sentinel in Sifr verified at [0516 Sifr:11](audits/leetcode/0516_longest_palindromic_subsequence.sifr) ✓ |

Other spot-checks:
- "0673 uses mutable `nonlocal` closure state in Python" — [0673 Py:23](audits/leetcode/0673_number_of_longest_increasing_subsequence.py) confirms `nonlocal lenLIS, res` ✓.
- "0673 ... `valueAt` helper linearly scans to avoid `int | None` from `nums[i]`" — verified at [0673 Sifr:3](audits/leetcode/0673_number_of_longest_increasing_subsequence.sifr) ✓.
- "0894 ... Python aliases shared subtree nodes ... Sifr clones each subtree per parent" and "drops Python's memoized `dp` cache because cached `list[TreeNode]` values would still require clone-out on each hit" — rationale is consistent with the stated Sifr ownership model; no contradicting evidence in the fixture.

One minor imprecision in the 0516 wording (line 135): "stacks a bottom-up 2D DP, an unreachable memoization block, **and a separate LCS helper inside one function**." In the actual file, the DP and the unreachable memoization block live inside the first `longestPalindromeSubseq`, but `longestCommonSubsequence` is a separate top-level function, reached via a second `longestPalindromeSubseq` redefinition at line 48. Suggested rewrite:

> `0516_longest_palindromic_subsequence.py` stacks a bottom-up 2D DP with an unreachable memoization block inside the first `longestPalindromeSubseq`, plus a second `longestPalindromeSubseq` redefinition that delegates to a separate `longestCommonSubsequence` helper; the Sifr version is a top-down memoized LCS on `s` and `s[::-1]`, with `changed_sifr_lines=22` against `changed_py_lines=61`.

Also worth noting (but not a defect in the cited numbers): [0673 Py](audits/leetcode/0673_number_of_longest_increasing_subsequence.py) carries a second, unreachable "O(n^2) Dynamic Programming" block after `return res` on line 33. This inflates `changed_py_lines=56` by roughly the same Python-side multi-implementation pattern Category 5 flags for 0104/0130/0200/0516. The Category 4 primary classification is still correct — the `nonlocal` boundary is real — but the Python side also carries Category 5-style noise, which the analysis does not acknowledge.

---

## 5. Are any category names, counts, or scope claims misleading?

Category names are clear and internally consistent. Counts are not written explicitly but are recoverable; they match the bullet lists (10 / 19 / 21 / 4 / 2). The one scope claim that could mislead a reader is the Category 1 list treating above-cutoff and below-cutoff items as indistinguishable — addressed in §2.

The Practical Priority Order (lines 164–202) re-lists the 10 Category 1 items under "Explicit parity-debt rewrites." That sub-list is consistent with Category 1's bullet list.

The "Preconditions" section (lines 7–18) is measured and self-aware — in particular the callout that "Some Python fixtures contain multiple full implementations, which inflates divergence artificially" is exactly what the scan data shows for the Category 3 fixtures, and is consistent with the 0673 observation above.

---

## 6. Concrete edits

### 6.1 Coverage gap: add `0024_swap_nodes_in_pairs`

[0024_swap_nodes_in_pairs](audits/leetcode/0024_swap_nodes_in_pairs.sifr) is not in any category and, by the criteria the analysis applies elsewhere, is the most obvious Category 1 candidate missing from the write-up.

Scan data:

```
0024_swap_nodes_in_pairs: total=79, py=63, sifr=16, sim=0.092
```

Evidence it belongs in Category 1 as a below-cutoff parity-debt rewrite, on the same basis as `0138`, `0206`, and `0023`:

- The Sifr file header reads "LeetCode 24: Swap Nodes In Pairs (residual Sifr-safe canonical form)" — the same self-describing marker used by [0023](audits/leetcode/0023_merge_k_sorted_lists.sifr), [0133](audits/leetcode/0133_clone_graph.sifr), and [0206](audits/leetcode/0206_reverse_linked_list.sifr), all of which the analysis already places in Category 1.
- [0024 Py](audits/leetcode/0024_swap_nodes_in_pairs.py) is a canonical `ListNode`-based dummy-cursor pairwise swap with the LeetCode-24 public signature `swapPairs(head: ListNode) -> ListNode`.
- [0024 Sifr](audits/leetcode/0024_swap_nodes_in_pairs.sifr) has the public signature `swapPairs(values: list[int]) -> list[int]` and performs an in-place pairwise swap on a copied array — the same public-model change as `0206` (linked-list reversal → list-value reversal).
- `similarity_ratio=0.092` is well below the three currently-listed below-cutoff Cat 1 items (0.085 / 0.078 / 0.317) and `changed_total_lines=79` is one line under the cutoff.

Suggested edit: add `0024_swap_nodes_in_pairs` to the Category 1 list (lines 26–35) and, in the "Why" block after line 35, add:

> - `0024_swap_nodes_in_pairs` changes the public model from `ListNode` pairwise swap to `list[int]` pairwise value swap; the canonical dummy-cursor implementation is absent.

And add it to the Practical Priority Order "Explicit parity-debt rewrites" list at lines 190–200.

### 6.2 Consider evaluating `0004_median_of_two_sorted_arrays`

Not a firm miss — flagging for explicit decision. Scan data: `total=63, py=32, sifr=31, sim=0.241`. [0004 Sifr](audits/leetcode/0004_median_of_two_sorted_arrays.sifr) carries the same "residual Sifr-safe canonical form" marker and implements O(m+n) merge rather than LeetCode-4's canonical O(log(min(m,n))) binary-search partition — an asymptotic regression analogous to `0295_find_median_from_data_stream` (total=56), which is already in Category 1. Either include it, or add one sentence to the analysis explaining why asymptotic regressions below cutoff are escalated for some problems (295) but not others (004).

### 6.3 Make below-cutoff Cat 1 exceptions visible

In Category 1 (line 35), add a trailing note analogous to line 154:

> Below the `changed_total_lines >= 80` scope but included as explicit parity-debt rewrites: `0138_copy_list_with_random_pointer`, `0206_reverse_linked_list`, `0295_find_median_from_data_stream` (and `0024_swap_nodes_in_pairs` if accepted from §6.1).

### 6.4 Tighten 0516 wording

Replace "a separate LCS helper inside one function" (line 135) with language that matches the actual file structure — see §4 for suggested text.

### 6.5 Acknowledge 0673 Python-side noise (optional)

In the Category 4 "Why" block after line 150, add:

> `0673_number_of_longest_increasing_subsequence.py` also trails an unreachable "O(n^2) Dynamic Programming" block after `return res`, inflating `changed_py_lines=56`; the Category 4 classification is still the primary one, but the raw diff carries Category 5-style Python-side noise on top of the architecture boundary.

This is optional — it doesn't change the decision, just makes the inflation source auditable.

### 6.6 Optional: add per-category counts

Adding counts next to each category heading (e.g. "### 1. Should Have Parity, Rewrite Mainly (10 items; 7 above cutoff, 3 below)") would make future coverage audits trivial without changing any content.

---

## Bottom line

The analysis is internally consistent and its cited numbers survive independent cross-checking against the raw scan and the source fixtures. The main substantive coverage issue is the omission of `0024_swap_nodes_in_pairs` from Category 1; the main hygiene issue is that Category 1's below-cutoff exceptions are silently mixed into the primary list instead of being demarcated the way Category 4's below-cutoff pattern notes are.
