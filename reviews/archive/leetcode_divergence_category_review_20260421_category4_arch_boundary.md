# Category 4 Review — Acceptable Divergence Because Of An Intentional Architecture Boundary

Date: 2026-04-21
Reviewer scope: Category 4 only, in [leetcode_divergence_decision_analysis_20260409.md](verification/leetcode/leetcode_divergence_decision_analysis_20260409.md).
Cross-checked against [leetcode_pair_diff_scan_20260409.json](verification/leetcode/leetcode_pair_diff_scan_20260409.json) and the source pairs under `audits/leetcode/`.

Guiding principle: mutable `nonlocal` capture is intentionally unsupported in Sifr. The purpose of this category is to surface divergences whose root cause is a *deliberate* Sifr design boundary (no mutable-nonlocal closures; single-ownership over aliased object graphs), not to create pressure to relax those boundaries.

## Fixtures listed

- `0673_number_of_longest_increasing_subsequence`
- `0894_all_possible_full_binary_trees`

Both fixtures do belong in Category 4. Classification is correct in both cases. Rationale for `0673` is slightly overstated and should be tightened; rationale for `0894` is accurate and can be minimally extended. Details below.

## 1. Is each listed fixture correctly classified?

### `0673_number_of_longest_increasing_subsequence` — correctly in Cat 4, with a caveat

The Python canonical reference is a recursive DP with `nonlocal lenLIS, res` (see [0673_number_of_longest_increasing_subsequence.py:23](audits/leetcode/0673_number_of_longest_increasing_subsequence.py:23)). The closure rebinds two ints across recursive calls while simultaneously mutating the `dp` cache. That specific pattern — *rebinding* outer-scope locals from inside a nested function — is exactly what Sifr intentionally does not support, and is *not* a narrowing/stdlib/ownership gap.

The Sifr version at [0673_number_of_longest_increasing_subsequence.sifr:11](audits/leetcode/0673_number_of_longest_increasing_subsequence.sifr:11) rewrites this into forward-iterative DP with a post-pass that reads `lengths` / `counts` and computes `longest` + `total`. That rewrite is idiomatic for a no-nonlocal language and gives the same result set, so the architecture-boundary framing is correct.

**Caveat that the rationale should acknowledge.** The fixture also carries a separable Cat 2b pressure that is independent of the nonlocal question: the `valueAt(values, index)` helper at [0673_number_of_longest_increasing_subsequence.sifr:3](audits/leetcode/0673_number_of_longest_increasing_subsequence.sifr:3) is a linear scan used to avoid `nums[i]` returning `int | None`. It is called as `valueAt(nums, i)` and `valueAt(nums, j)` inside the `O(n^2)` double loop at lines 25–49. A linear lookup inside a quadratic loop makes the realized complexity `O(n^3)`, not `O(n^2)`. The Cat 4 rationale as currently written — *"The iterative rewrite preserves the same O(n^2) asymptotic behavior"* — is only true of the *nonlocal → iterative* transformation in isolation. The fixture as it stands on disk is `O(n^3)` because of the `valueAt` workaround, which is a Cat 2b (collection/index/stdlib ergonomics) concern, not an architecture boundary.

This does not mean `0673` moves out of Cat 4 — the nonlocal boundary is still the primary, non-negotiable cause of divergence. But the rationale should not claim preserved asymptotic parity on a fixture whose current code is asymptotically worse, and the doc should note that Cat 4 and Cat 2b can co-occur on one fixture.

### `0894_all_possible_full_binary_trees` — correctly in Cat 4

The Python version at [0894_all_possible_full_binary_trees.py:42](audits/leetcode/0894_all_possible_full_binary_trees.py:42) memoizes `backtrack(n) -> list[TreeNode]` and *aliases* subtree instances: each generated parent tree stores the same `t1` and `t2` node references as every other parent tree at that level. The Sifr version at [0894_all_possible_full_binary_trees.sifr:59](audits/leetcode/0894_all_possible_full_binary_trees.sifr:59) cannot do that — reusing a subtree under a second parent would put two owners on the same heap allocation. The Sifr fixture clones via `cloneTree(t1)` / `cloneTree(t2)` at [0894_all_possible_full_binary_trees.sifr:73-75](audits/leetcode/0894_all_possible_full_binary_trees.sifr:73). This is precisely the design boundary this category is meant to cover.

One small addition worth folding into the rationale: the Sifr version also drops the `dp` memoization cache (it recomputes subtree lists on each recursive call). This is not a separate parity loss — memoizing `list[TreeNode]` under single ownership would require cloning the cached lists on each hit anyway, so the total node-copy cost is in the same family. Noting this explicitly prevents a future reader from flagging the missing `dp` as a separate bug.

**Output correctness.** Both fixtures produce the expected test outputs; asserts in `main()` match on each side. Category 4 here is about *how* the result is produced, not *what* it produces.

## 2. Is any fixture missing from Category 4?

I searched the raw scan and the audit sources for every Python fixture using `nonlocal`. The full population is eight fixtures:

| stem | changed_total_lines | similarity_ratio | current category | nonlocal rewrite shape in Sifr |
|---|---|---|---|---|
| `0673_number_of_longest_increasing_subsequence` | 113 | 0.150 | **Cat 4** | iterative DP + post-pass |
| `0261_graph_valid_tree` | 117 | 0.204 | Cat 2b | inlined union body (no closure) |
| `0513_find_bottom_left_tree_value` | 99 | 0.414 | Cat 2b | BFS (iterative) |
| `0894_all_possible_full_binary_trees` | 88 | 0.476 | **Cat 4** | clone subtrees per parent |
| `0783_minimum_distance_between_bst_nodes` | 69 | 0.543 | unlisted | closure-append list, then iterate |
| `0543_diameter_of_binary_tree` | 63 | 0.526 | unlisted | tuple-returning recursion |
| `1466_reorder_routes...` | 59 | 0.289 | unlisted | iterative DFS on an explicit stack |
| `0052_n_queens_ii` | 29 | 0.580 | unlisted | accumulator-returning recursion with explicit set params |

None of the six non-Cat-4 fixtures need to move into Cat 4, but the reasons are worth stating so the boundary stays calibrated:

- **`0261_graph_valid_tree`** — the Python DSU variant at [0261_graph_valid_tree.py:39](audits/leetcode/0261_graph_valid_tree.py:39) uses `nonlocal components`, but the Sifr version at [0261_graph_valid_tree.sifr:1](audits/leetcode/0261_graph_valid_tree.sifr:1) inlines the `union` body directly into the edge loop, so the nonlocal issue is not what drives the 117-line diff. The diff is dominated by Optional-flow ceremony on `parents[node]` / `ranks[root]` list access — that is correctly Cat 2b.
- **`0513_find_bottom_left_tree_value`** — Python stacks a BFS definition *then* a second recursive definition that uses `nonlocal max_height, res`; only the last definition is bound in Python. Sifr mirrors the *first* (BFS) variant. The divergence is therefore not nonlocal-driven — it is Cat 2b flow on `q.pop(0)` plus corpus-noise from the shadowed second Python definition. Staying in Cat 2b is correct.
- **`0543_diameter_of_binary_tree`**, **`0783_minimum_distance_between_bst_nodes`**, **`1466_reorder_routes...`**, **`0052_n_queens_ii`** — all are clean examples of the nonlocal → pure-return or explicit-stack rewrite. All four sit below the document's stated scope cutoff of `changed_total_lines >= 80` (and below the 70–90 "manual judgment" band for `0543`, `0783`, `1466`). Adding them to Cat 4 would expand scope past what the analysis document defines. Optionally, the Cat 4 reasoning section could cite `0543` or `0052` as one-sentence precedent examples ("the same nonlocal → pure-return rewrite appears below scope in 0052, 0543, 0783, 1466"), which both validates the pattern and documents that it is sub-threshold rather than overlooked. Not a must-fix.

`1466` has an additional wrinkle worth flagging separately: the Sifr signature takes `list[tuple[int, int]]` instead of Python's `list[list[int]]` (see [1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero.sifr:3](audits/leetcode/1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero.sifr:3)). That is a public-surface change of the same flavor flagged in Cat 1 (e.g. `0023`, `0138`). It is independent of the nonlocal question and should not be used to argue `1466` belongs in Cat 4 — if anything, the signature change is a small Cat 1 concern to track separately.

I also searched for other single-ownership/aliasing patterns analogous to `0894` (Python uses of `deepcopy`, sifr uses of `cloneTree`, etc.): only `0894` exhibits this intra-algorithm aliasing pattern outside the already-known public-surface rewrites (`0133`, `0138`) which live in Cat 1. So nothing is missing on the ownership-boundary side either.

## 3. Should any fixture move out of Category 4?

No. Both listed fixtures stay. The underlying divergence for each is caused by a Sifr boundary that is intentional and documented:

- `0673` — no mutable nonlocal closures.
- `0894` — no aliased ownership of shared subtree objects.

Neither is a rewrite-debt case (Cat 1) because the public input/output surface is preserved and the algorithmic family matches the canonical problem (`O(n^2)` LIS DP; Catalan-family subtree enumeration). Neither is a recursive-cursor case (Cat 2a) because there is no `.next`/`.left`/`.right` narrowing gap at the core. Neither is purely collection/index ergonomics (Cat 2b), though `0673` has Cat 2b pressure layered on top (see Section 1). And neither is corpus noise (Cat 3/5).

## 4. Is the rationale precise enough to preserve Sifr principles while still identifying real parity risk?

Mostly yes, with the sharpening below. The current Cat 4 text already states the correct non-negotiable: *"It should not be used as pressure to add mutable nonlocal support."* That line is load-bearing and should stay exactly as it is.

What needs tightening:

- **0673's asymptotic claim.** The sentence *"The iterative rewrite preserves the same O(n^2) asymptotic behavior"* is true of the nonlocal → iterative transformation on paper, but false of the fixture as it exists on disk, because `valueAt` makes every element access `O(n)`. A reader who takes the asymptotic claim at face value will not see the hidden `O(n^3)` regression. The cleanest fix is to separate the two claims: (a) the nonlocal→iterative structural rewrite is asymptotically neutral; (b) the fixture additionally carries Cat 2b Optional-flow pressure on `nums[i]`, and the `valueAt` workaround used today is what's degrading the realized complexity. Once Cat 2b ergonomics for list indexing land, the fixture should return to `O(n^2)` without any change in its Cat 4 status.

- **Cat 4 / Cat 2b co-occurrence.** The category intro does not acknowledge that a single fixture can sit in Cat 4 (structural boundary) while *also* being pressured by Cat 2b (collection ergonomics). `0673` is the clearest example. A one-sentence note — "a fixture may carry Cat 2b pressure on top of a Cat 4 boundary; the Cat 4 label is about the structural boundary, not a claim that the rest of the fixture is ergonomically clean" — would make the classification honest without weakening the boundary.

- **0894's memoization loss.** Adding one sentence that ownership also drops the `dp` cache (because cached `list[TreeNode]` would itself need cloning on each hit) prevents future confusion that the missing `dp` is an independent bug.

What must not be recommended (explicit non-asks, restated for durability):

- Do not introduce mutable `nonlocal` capture. The correct rewrite path for `nonlocal`-using fixtures in Sifr is one of: accumulator-returning recursion (`0052`, `0543`), tuple-returning recursion (`0543`), iterative DP with a post-pass accumulator (`0673`), or iterative stack/BFS (`0513`, `1466`). All of these are already in use in the corpus.
- Do not introduce shared-ownership primitives (Rc / Arc / interior mutability) purely to emulate Python's aliased subtree graph in `0894`. Cloning at the ownership boundary is the right answer for this problem shape.
- Do not use Cat 4 as general cover for "Sifr needed a different algorithm." It is specifically for *structural* boundaries. `0673`'s iterative DP is on-boundary; a hypothetical rewrite that, say, traded LIS-DP for an entirely different algorithm to dodge a narrowing gap would not be.

## 5. Concrete edits

### Edit A — Sharpen `0673`'s rationale (required)

Replace the current 0673-specific bullets in Cat 4 with something closer to:

> - `0673_number_of_longest_increasing_subsequence` — Python uses `nonlocal lenLIS, res` in the recursive DP. Sifr intentionally does not support mutable `nonlocal` capture, and the Sifr fixture rewrites the same algorithm as forward-iterative DP with a post-pass accumulator, preserving the canonical `O(n^2)` shape. Note that the fixture *as currently written* carries layered Cat 2b pressure: the `valueAt` helper is a linear scan used to avoid `int | None` from `nums[i]`, which degrades realized complexity to `O(n^3)` until Cat 2b list-index ergonomics land. The Cat 4 boundary here is the nonlocal rewrite; the asymptotic recovery is a Cat 2b follow-up.

### Edit B — Tighten `0894`'s rationale (recommended)

Extend the existing bullet with one clause:

> - `0894_all_possible_full_binary_trees` — Python aliases shared subtree nodes across multiple generated parent trees (and memoizes via `dp`). Sifr's single-ownership model cannot alias, so the Sifr fixture clones each subtree per parent. The `dp` cache is also dropped because memoized `list[TreeNode]` would itself require clone-out on each hit; that is a consequence of the same ownership boundary, not a separate divergence.

### Edit C — Add co-occurrence note to the Cat 4 intro (recommended)

At the end of the Cat 4 intro paragraph, add:

> A fixture may carry Cat 2b (collection/index/stdlib) pressure on top of a Cat 4 boundary — `0673` is the clearest example. The Cat 4 label identifies the structural boundary, not a claim that the rest of the fixture is ergonomically clean.

### Edit D — Optional cross-reference to sub-threshold nonlocal rewrites

In Cat 4's reasoning section, add a single sentence:

> The same nonlocal → pure-return or explicit-stack rewrite pattern also appears below the `changed_total_lines >= 80` scope in `0052_n_queens_ii`, `0543_diameter_of_binary_tree`, `0783_minimum_distance_between_bst_nodes`, and `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`. These are recorded for pattern continuity, not escalation.

No fixtures need to move in or out of Cat 4.
