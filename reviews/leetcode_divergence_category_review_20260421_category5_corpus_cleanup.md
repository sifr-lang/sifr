# Category 5 Review — Needs Corpus Cleanup Before It Should Drive Design Priorities

Date: 2026-04-21
Source under review: `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md` (Section 5)
Cross-check source: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Audit pairs under: `audits/leetcode/<stem>.py` and `audits/leetcode/<stem>.sifr`

## Summary Verdict

The three fixtures explicitly labelled as Category 5 (`0104_maximum_depth_of_binary_tree`, `0200_number_of_islands`, `0516_longest_palindromic_subsequence`) are correctly treated as corpus-cleanup/noise rather than rewrite debt, recursive-cursor ergonomics, collection/stdlib ergonomics, or an intentional architecture boundary. They each present the same mechanical pattern: the Python fixture stacks multiple implementations (or unused kitchen-sink helpers) in a single file, while the Sifr fixture carries a single canonical algorithm with matching asymptotic behaviour. The category's framing as a *secondary* label layered onto Category 3 is consistent with the document's own preconditions and the stated Priority Order item 1 (corpus normalization).

There is one borderline case (`0130_surrounded_regions`) that could plausibly acquire a Category 5 secondary label, and one scope-widening suggestion to ensure Category 5 explicitly covers mirrored Sifr-side helper/kitchen-sink boilerplate (not only Python-side multi-implementation stacking). These are refinements, not structural errors.

## Per-Fixture Validation of the Three Listed Items

### `0104_maximum_depth_of_binary_tree`

- Raw scan: `changed_py_lines=74`, `changed_sifr_lines=8`, `similarity_ratio=0.317`.
- Python side ([audits/leetcode/0104_maximum_depth_of_binary_tree.py](audits/leetcode/0104_maximum_depth_of_binary_tree.py)) carries three stacked `maxDepth` definitions (recursive at line 43, iterative DFS at line 52, BFS at line 68), plus an unused `tree_to_string` helper and an unused kitchen-sink `Node` class (lines 24-41). All three `maxDepth` bodies are dead under Python function-name rebinding except the last.
- Sifr side ([audits/leetcode/0104_maximum_depth_of_binary_tree.sifr](audits/leetcode/0104_maximum_depth_of_binary_tree.sifr)) is a single 4-line canonical recursive `maxDepth` with `root: TreeNode | None`, an early `is None` return, and `1 + max(maxDepth(root.left), maxDepth(root.right))`. No cursor ergonomics pressure, no Optional/index ergonomics pressure, no rewrite-debt divergence, no ownership boundary.
- Placement is correct. The `changed_py_lines / changed_sifr_lines` ratio of 9.25 is the highest in the scope window and is entirely explained by Python-side noise.

### `0200_number_of_islands`

- Raw scan: `changed_py_lines=82`, `changed_sifr_lines=21`, `similarity_ratio=0.202`.
- Python side ([audits/leetcode/0200_number_of_islands.py](audits/leetcode/0200_number_of_islands.py)) carries three stacked `numIslands` definitions: a set-tracked recursive DFS (line 6), an in-place grid-mutation DFS (line 37), and a BFS using `deque` (line 56). The first two rebinds are dead.
- Sifr side ([audits/leetcode/0200_number_of_islands.sifr](audits/leetcode/0200_number_of_islands.sifr)) is a single canonical set-tracked DFS with `visited: set[tuple[int, int]]` and the standard four-direction fan-out. The Sifr version does contain a handful of language-level boilerplate lines (typed closure signature, explicit early-return conditions, `len(grid) == 0` empty-guard), but none of these are Category 2b collection/index/stdlib ergonomics pressure — they are just explicit restatements of the canonical algorithm.
- Placement is correct. There is no collection-ergonomics asymptotic regression and no recursive-cursor ergonomics deficit, so 2b would be the wrong escalation target.

### `0516_longest_palindromic_subsequence`

- Raw scan: `changed_py_lines=61`, `changed_sifr_lines=22`, `similarity_ratio=0.210`.
- Python side ([audits/leetcode/0516_longest_palindromic_subsequence.py](audits/leetcode/0516_longest_palindromic_subsequence.py)) contains a bottom-up 2D DP `longestPalindromeSubseq` (line 5) *plus an unreachable memoization block inside the same function* (lines 24-44 are dead after the `return res` at line 21), followed by a second top-level rebind that dispatches to a third `longestCommonSubsequence` helper (lines 48-63). This is a triple-stacked impl plus dead code, even heavier than 0104 / 0200 per line but visually compressed.
- Sifr side ([audits/leetcode/0516_longest_palindromic_subsequence.sifr](audits/leetcode/0516_longest_palindromic_subsequence.sifr)) is a single top-down memoized LCS on `s` and `s[::-1]`. The asymptotic behaviour matches the canonical LCS-via-reverse solution.
- The Category 5 entry honestly acknowledges the residual Category 2b-adjacent `memo.get((i, j), -1)` sentinel at line 11 of the Sifr file. This is the correct call: the asymptotic complexity and algorithmic shape match canonical LCS, so the fixture is primarily Category 3 / 5; the dict-Optional workaround is a minor ergonomics note, not a reclassification trigger.
- Placement is correct.

## Missing / Borderline Candidates

### `0130_surrounded_regions` — borderline candidate for a Category 5 secondary label

Currently placed in Category 2b. Raw scan: `changed_py_lines=55`, `changed_sifr_lines=26`, `similarity_ratio=0.283`, ratio 2.12 (fifth highest in scope).

- The Python fixture ([audits/leetcode/0130_surrounded_regions.py](audits/leetcode/0130_surrounded_regions.py)) contains a live `solve` at line 5 (22 lines) and a ~29-line triple-quoted second `solve` between lines 27-58. The triple-quoted block is a textual second implementation that inflates the raw diff the same way a rebinding stack does, even though Python semantically discards it.
- The Sifr fixture ([audits/leetcode/0130_surrounded_regions.sifr](audits/leetcode/0130_surrounded_regions.sifr)) is algorithmically canonical: set-tracked DFS seeded from the border, then flip unflagged `O` cells to `X`. The only Sifr-side overhead over Python is language-level — `mut board`, explicit `len(board) == 0 or len(board[0]) == 0` guard, `set[tuple[int, int]]` annotation, typed closure signature — none of which fit Category 2b's stated pressure axes ("preserve proven non-Optional collection/index values", "dict-entry non-Optional facts after insertions", "safer owned collection helpers with minimal cloning", "stdlib parity for heap/deque/DSU/trie").
- Recommendation: either add `0130_surrounded_regions` as a *secondary* Category 5 label alongside its current 2b placement (acknowledging that the triple-quoted second `solve` inflates raw diff), or, if the analysis intends 2b to strictly cover Sifr-side ergonomics pressure, justify the 2b placement explicitly with the specific Sifr-side 2b friction it exhibits. Reading the current Sifr file I cannot identify concrete 2b friction; the fixture looks like a Category 3+5 case more than a 2b case.

No other fixture in the `changed_total_lines >= 80` scope is a clean candidate for a Category 5 *primary* label.

### Fixtures with high `changed_py_lines / changed_sifr_lines` ratio that do *not* belong in Category 5

For completeness, the next-highest ratio candidates after the three Cat 5 items and 0130 are `0023_merge_k_sorted_lists`, `0133_clone_graph`, `0212_word_search_ii`, `0707_design_linked_list`. Each of these is already correctly routed to Category 1 because the Sifr fixture changes the public surface or drops the canonical algorithm, which is genuine rewrite debt rather than corpus noise — the inflation here comes from real divergence plus Python helper stacking, not from noise alone. They should not move to Category 5.

`0513_find_bottom_left_tree_value` (ratio 1.11) has two stacked `findBottomLeftValue` definitions in the Python file (BFS at line 43, recursive `nonlocal` at line 63) alongside a mirrored Sifr-side BFS that carries real Optional ceremony around `q.pop(0)`. Because the Sifr-side divergence is genuine 2b pressure, the primary 2b placement is correct. A *secondary* Cat 5 label would be defensible to account for the Python-side stacked impl on top of the mirrored `Node` / `treeToString` kitchen-sink; flagging this in the Cat 5 body is optional and lower-value than the 0130 case above.

## Framing Review

Category 5's framing as "a secondary label for the same fixtures enumerated in Category 3" is correct and matches:

- The preconditions bullet: *"Some Python fixtures contain multiple full implementations, which inflates divergence artificially."*
- Practical Priority item 1: *"Corpus normalization — mark explicit non-canonical parity-debt fixtures clearly; normalize helper-boilerplate noise in comparison scripts; stop treating raw diff buckets as calibrated severity."*

This avoids the trap of promoting 0104 / 0200 / 0516 into design pressure (they would otherwise rank near the top of the raw scan by `changed_total_lines`). The separation from Category 3 — algorithmic parity vs. diff-inflation — is clean: Category 3 is the *judgment* (okay as-is), Category 5 is the *follow-up action* (normalize the corpus so the next scan stops surfacing these).

One framing gap worth closing: the Category 5 body names only "Python-side multi-implementation or helper baggage" as the inflation mechanism. The preconditions section additionally calls out mirrored Sifr-side kitchen-sink helpers as a distinct noise source (`Node` classes with unused fields, `nodeVal` / `nodeNext` / `hasNode` / `unwrapInt`, tree helper sets) that *cancels in the diff*. That second mechanism currently has no category placement — it affects fixtures like 0513 and several linked-list items where both sides carry parallel boilerplate. Extending Category 5's scope statement to cover both "Python-side stacking" and "mirrored helper/kitchen-sink baggage" would make Category 5 a more complete triage bucket and would give reviewers a place to note that a Cat 2a or Cat 2b fixture *also* needs corpus cleanup independent of the ergonomics work.

## Concrete Edits (Optional)

1. In the Category 5 body, widen the scope statement from:

   > "applies to the Category 3 fixtures ... whose raw diffs are inflated by Python-side multi-implementation or helper baggage rather than Sifr-side divergence"

   to something like:

   > applies to fixtures whose raw diffs are inflated by (a) Python-side stacked implementations, (b) unused helper or kitchen-sink classes on either side, or (c) mirrored Sifr-side boilerplate that cancels in the diff; currently this covers the Category 3 fixtures `0104_maximum_depth_of_binary_tree`, `0200_number_of_islands`, `0516_longest_palindromic_subsequence`, and optionally `0130_surrounded_regions` (triple-quoted second `solve`).

2. Consider adding `0130_surrounded_regions` as a Category 5 secondary label, or alternatively add a one-line justification of its 2b placement naming the specific Sifr-side 2b friction it is meant to illustrate.

3. Optionally, append a per-fixture cleanup checklist to Category 5 so the "corpus normalization" priority item can be executed directly:
   - `0104_maximum_depth_of_binary_tree.py`: keep one canonical `maxDepth`, drop the unused `Node` kitchen-sink class and the unused `tree_to_string` helper.
   - `0200_number_of_islands.py`: keep one canonical `numIslands`, drop the other two rebinds and unused imports.
   - `0516_longest_palindromic_subsequence.py`: keep one canonical solver, drop the unreachable memoization block and the separate LCS helper (or separate into clearly-marked alternative files).
   - `0130_surrounded_regions.py` (if added): remove the triple-quoted second `solve` block.

None of these edits change the structural verdict of Category 5; they tighten the scope statement, close one borderline placement question, and make the follow-up action concrete.

## Conclusion

Category 5 is structurally sound and correctly framed as a secondary cleanup follow-up rather than a competing primary classification. The three enumerated fixtures are the right set for the primary Cat 5 call. The only recommended changes are (a) consider adding `0130_surrounded_regions` as a secondary Cat 5 label or strengthen its 2b justification, (b) optionally widen the Cat 5 scope statement to explicitly cover mirrored Sifr-side helper baggage, and (c) optionally record a concrete cleanup checklist per fixture so the Practical Priority item 1 is directly actionable.
