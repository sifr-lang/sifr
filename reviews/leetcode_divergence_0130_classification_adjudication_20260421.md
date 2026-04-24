# 0130 `surrounded_regions` — Classification Adjudication

Date: 2026-04-21
Scope: independent adjudication of the category placement of `0130_surrounded_regions` in [verification/leetcode/leetcode_divergence_decision_analysis_20260409.md](../verification/leetcode/leetcode_divergence_decision_analysis_20260409.md).

Inputs:

- Python fixture: [audits/leetcode/0130_surrounded_regions.py](../audits/leetcode/0130_surrounded_regions.py) — 71 lines
- Sifr fixture: [audits/leetcode/0130_surrounded_regions.sifr](../audits/leetcode/0130_surrounded_regions.sifr) — 42 lines
- Raw scan entry in [verification/leetcode/leetcode_pair_diff_scan_20260409.json](../verification/leetcode/leetcode_pair_diff_scan_20260409.json) (lines 643-654):
  `changed_py_lines=55, changed_sifr_lines=26, changed_total_lines=81, length_delta=29, similarity_ratio=0.283`.

Current placement in the analysis: Category 2b (collection / index / stdlib ergonomics), line 86.

## TL;DR

0130 does not exhibit Category 2b pressure. The raw diff is driven almost entirely by a Python-side triple-quoted alternate implementation (lines 27-58, ~32 lines). The Sifr fixture is a clean, canonical DFS-with-set solution. Placement should be **Category 3 (okay as-is / corpus noise), with Category 5 as the secondary cleanup label** — identical to `0200_number_of_islands`, which this fixture structurally parallels.

## Evidence

### 1. Does the Sifr fixture exhibit Category 2b pressure?

Walking the Category 2b "what should improve" checklist ([leetcode_divergence_decision_analysis_20260409.md:109-120](../verification/leetcode/leetcode_divergence_decision_analysis_20260409.md)) against [0130_surrounded_regions.sifr](../audits/leetcode/0130_surrounded_regions.sifr):

| Category 2b symptom | Present in 0130.sifr? |
| --- | --- |
| Dead Optional guards on proven `list[T]` indexing | **No.** `board[r][c]` is read and written directly with no `if x is not None`, `.unwrap()`, or copy-to-local ceremony at lines 14, 27, 32-33. |
| Dict-entry narrowing after `.get` / contains-key / insert | **No.** No dict is used. Only a `set[tuple[int, int]]` with `.add` and `in`, which works cleanly. |
| Ownership/cloning ceremony on owned collections | **No.** Only a `mut board` parameter marker (line 3). No `.clone()`, no copy-out-to-local, no helper functions. |
| Missing `deque` / `heap` / DSU / trie / parse / `isdigit`/`isalpha` | **No.** The algorithm is recursive DFS over a grid with an ordinary `set`. None of these primitives are needed or worked around. |

The only substantive delta versus the Python primary solution is:

- Sifr uses numeric bounds (`r < 0 or c < 0 or r >= rows or c >= cols`, [0130_surrounded_regions.sifr:12](../audits/leetcode/0130_surrounded_regions.sifr)) where the Python primary uses `r in range(rows) and c in range(cols)` ([0130_surrounded_regions.py:10](../audits/leetcode/0130_surrounded_regions.py)). That is an idiom/style difference, not an Optional-flow issue — the Python *alternate* implementation at [0130_surrounded_regions.py:33](../audits/leetcode/0130_surrounded_regions.py) also uses numeric bounds (`r < 0 or c < 0 or r == ROWS or c == COLS`), so this is not a Sifr-induced ceremony.
- Sifr splits three conditions into three `if` statements ([lines 12-17](../audits/leetcode/0130_surrounded_regions.sifr)) where Python uses one compound condition. This is style, not a narrowing workaround — no `is not None` pattern, no local rebinding.
- Sifr adds an explicit empty-board guard ([lines 4-5](../audits/leetcode/0130_surrounded_regions.sifr)) that Python omits. Defensive, small, unrelated to Category 2b.
- Sifr annotates the closure signature as `def dfs(r: int, c: int) -> None:`. Required by Sifr's typing rules; benign.

**Conclusion:** no Category 2b pressure. The Sifr solution is canonical and unceremonious.

### 2. How much of the raw diff is Python-side noise?

The Python file contains two stacked implementations of `solve`:

- Primary DFS-with-set approach ([0130_surrounded_regions.py:5-25](../audits/leetcode/0130_surrounded_regions.py)) — structurally identical to the Sifr version.
- A 32-line **triple-quoted alternate implementation** using the 3-pass `O → T → X → O` marker pattern ([0130_surrounded_regions.py:27-58](../audits/leetcode/0130_surrounded_regions.py)), which is never executed. This block carries its own definition, comments, and signature.

Counting the sources of `changed_py_lines=55` (of 71 total):

- Triple-quoted alternate block (lines 27-58): 32 lines.
- Inline comments the Sifr version drops (`# Python version`, `# traverse through the board`, `# set all of the 'X's to 'O's`, blank separators): ~5 lines.
- Python's tuple-return DFS chain `return (dfs(r + 1, c), dfs(r - 1, c), dfs(r, c + 1), dfs(r, c - 1))` on one line vs Sifr's four statement lines: formatting expansion, not semantic divergence.

≥32 of 55 changed Python lines — roughly 60% — are attributable to the triple-quoted alternate alone. Together with comments and formatting, effectively the entire diff is explained by Python-side corpus noise, not by any Sifr-side divergence.

### 3. Parallel to existing Category 3 fixtures

Category 3 currently lists `0104_maximum_depth_of_binary_tree`, `0200_number_of_islands`, and `0516_longest_palindromic_subsequence` ([leetcode_divergence_decision_analysis_20260409.md:126-128](../verification/leetcode/leetcode_divergence_decision_analysis_20260409.md)). The justifications (lines 132-134) all describe the same pattern: Python stacks multiple implementations; the Sifr version is a single canonical solution.

`0200_number_of_islands` is a direct structural neighbor of 0130 — both are grid DFS problems — and its Python file "stacks three `numIslands` definitions (set-tracked DFS, in-place grid-mutation DFS, and BFS via `deque`)". 0130's Python file stacks two implementations (primary set-DFS plus a commented-out 3-pass marker variant). The pattern is identical in kind, smaller only in count.

### 4. Correct placement

**Primary: Category 3.** Corpus noise driven by a Python-side alternate implementation, no Sifr-side ergonomic debt.

**Secondary: Category 5.** Per the analysis's own rule at [line 155-161](../verification/leetcode/leetcode_divergence_decision_analysis_20260409.md), Category 5 is the cleanup follow-up applied to the Category 3 set. 0130 should be listed in the same breath as 0104/0200/0516.

## Concrete edits to the analysis file

File: [verification/leetcode/leetcode_divergence_decision_analysis_20260409.md](../verification/leetcode/leetcode_divergence_decision_analysis_20260409.md)

### Edit 1 — remove from Category 2b (line 86)

Delete the bullet:

```
- `0130_surrounded_regions`
```

at line 86, so the 2b list begins with `0150_evaluate_reverse_polish_notation`.

### Edit 2 — add to Category 3 list (after line 127)

Insert `- \`0130_surrounded_regions\`` so the list becomes:

```
- `0104_maximum_depth_of_binary_tree`
- `0130_surrounded_regions`
- `0200_number_of_islands`
- `0516_longest_palindromic_subsequence`
```

### Edit 3 — add a "Why" bullet for 0130 (in the block at lines 131-136)

Insert, in fixture-sorted order between the 0104 and 0200 bullets:

```
- `0130_surrounded_regions.py` pairs a primary DFS-with-set `solve` against a ~32-line triple-quoted alternate 3-pass `O → T → X → O` implementation; the Sifr version is the canonical DFS-with-set with `changed_sifr_lines=26` against `changed_py_lines=55`, and exhibits none of the Category 2b symptoms (no dead Optional guards on `board[r][c]`, no dict narrowing, no missing stdlib primitive, no cloning ceremony beyond a single `mut` parameter).
```

### Edit 4 — update Category 5 scope (line 161)

Replace:

```
- applies to the Category 3 fixtures (`0104_maximum_depth_of_binary_tree`, `0200_number_of_islands`, `0516_longest_palindromic_subsequence`), whose raw diffs are inflated by Python-side multi-implementation or helper baggage rather than Sifr-side divergence
```

with:

```
- applies to the Category 3 fixtures (`0104_maximum_depth_of_binary_tree`, `0130_surrounded_regions`, `0200_number_of_islands`, `0516_longest_palindromic_subsequence`), whose raw diffs are inflated by Python-side multi-implementation or helper baggage rather than Sifr-side divergence
```

## Recommendation

Apply all four edits. 0130 should not be cited as Category 2b pressure in any downstream prioritization; doing so inflates the apparent demand for collection/index/stdlib work when the actual signal is Python-side corpus noise of a kind already acknowledged elsewhere in the analysis.
