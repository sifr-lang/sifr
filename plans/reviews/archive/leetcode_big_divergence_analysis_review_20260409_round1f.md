# Review — LeetCode Big-Divergence Analysis (round 1f)

Source under review: `verification/leetcode/leetcode_big_divergence_analysis_20260409.md`
Supporting data: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Scanner: `scripts/scan_leetcode_pair_diffs.py`
Date: 2026-04-09
Reviewer focus: misclassifications, missing constraints, overreach toward Python, underreach on parity, prioritization.

## Summary

The analysis is directionally correct and gets the single most important call right: the top raw-diff outliers are not an argument for Python dynamism, they are an argument for safe expression ergonomics and recursive-object handling. The four-category split is useful and the "do not do" list is well-scoped.

However the document is built on a metric whose caveats are mentioned once and then dropped, it conflates parity debt with parity aspiration, and it orders priorities in a way that would spend compiler effort against a still-poisoned baseline. The prioritization also omits the concrete dependency chain between stdlib parity and the parity-debt clearances it enables.

I verified the category-3 algorithm-substitution claims by reading the fixtures directly. They are accurate for `0148`, `0023`, `0295`, `0212`. That verification also surfaced a pattern the analysis misses — see "Misclassifications — 0295" below.

## 1. Misclassifications

### 1a. `0295_find_median_from_data_stream` — not purely algorithm substitution

The analysis places `0295` in Category 3 (material algorithm substitution) with the framing "Python uses heaps, Sifr keeps a sorted array". That is true but incomplete.

Reading `audits/leetcode/0295_find_median_from_data_stream.sifr:9-35`, the sorted-array variant is itself inflated by spurious `int | None` narrowing on `self.nums[i]` and `self.nums[n // 2]`, even though `self.nums: list[int]` and every index is bounds-checked at the source. The fixture contains dead guards like `if cur is not None:` on an `int` element and `if mid is not None:` on `self.nums[n // 2]`.

This means the fixture is simultaneously:
- a Category 3 case (heap missing), and
- a Category 1 case with a *specific, actionable* flavor: `list[int]` element access currently forces an Optional narrowing path that produces dead branches and no safety benefit.

This split matters because it generalizes. The 0295 pattern is evidence that even the workarounds the corpus uses to avoid missing primitives are themselves bloated by the same Optional-flow issue Category 1 names. Fixing Category 1 would shrink Category 3's fallback forms before the missing primitives are even shipped. The analysis should call this out explicitly and treat `0295` as a two-cause entry.

### 1b. `0148_sort_list` straddles three categories, not two

The per-problem table at line 164 credits `0148` as "algorithm substitution + recursive-object friction", which matches the prose split between Category 2 and Category 3. But both the Python and Sifr versions of `0148` carry a dead `class Node` (`0148_sort_list.py:20-37`, `0148_sort_list.sifr:45-70`) that is shared boilerplate unrelated to the problem.

Because the dead class is *mirrored* on both sides it cancels out in the raw diff, which means Category 4 ("reference-fixture noise") is real on this file but invisible to the scanner. The analysis's framing of Category 4 assumes noise only exists on the Python side. It does not — the noise is shared infrastructure and is currently hidden from the metric, not just inflating it.

Action: Category 4 should be renamed "shared / unilateral helper boilerplate" and the analysis should note that shared boilerplate *understates* divergence in the raw-diff scan rather than overstating it.

### 1c. `0200_number_of_islands` cited under Category 4 without substantiation

The analysis names `0200` as a Category 4 case but provides no evidence, and `0200` does not appear in the per-problem readout table. Either produce the specific helper-class / alternate-solution evidence or drop it from Category 4 until the corpus normalization step has run. Listing it without substantiation creates a spurious signal that the corpus is noisier on the Python side than it actually is.

## 2. Missing constraints on the underlying metric

The analysis mentions metric caveats in one paragraph at lines 15-20 and then treats the raw numbers as load-bearing for the rest of the document. The caveats are stronger than stated:

1. **The metric sums `changed_py_lines + changed_sifr_lines`** from `difflib.SequenceMatcher.get_opcodes()` (`scripts/scan_leetcode_pair_diffs.py:69-74`). No normalization: no whitespace trim, no comment strip, no signature/annotation strip, no import strip. A pure reformat of either side promotes a fixture into a higher bucket. The 120 / 100 / 80 thresholds at lines 11-13 are therefore not calibrated noise floors — they are raw bucket counts with an unknown amount of format drift mixed in.

2. **The buckets have no baseline.** The document does not say what the median `changed_total_lines` is across the 395 paired fixtures, so a reader cannot tell whether 120 is 4× median or 1.5× median. Without that, "16 pairs at ≥120" is not interpretable as "outliers" vs "the top of a long tail". Add median, p75, p90 alongside the bucket counts.

3. **The metric is direction-agnostic.** A case where Sifr shrinks the file (e.g., `0023` collapsing the linked-list interface to `list[list[int]]`, 79 Python lines vs 18 Sifr lines — `length_delta: 61` approximately) looks identical in `changed_total_lines` to a case where Sifr grows the file. These are very different design signals. Report `length_delta` sign separately in the category tables, or at minimum split the outlier set into "Sifr expanded" vs "Sifr contracted" buckets.

4. **Mirrored noise cancels.** As shown in the `0148` case above, shared dead helpers (`class Node`, serialization helpers, etc.) exist on both sides and never appear in the diff. The "reference-fixture noise" category as written only describes *unilateral* noise. Mirrored noise is invisible to the metric and inflates both files without being flagged for cleanup.

5. **No complexity-regression check is cross-indexed with the diff rank.** 0295's sorted-array variant is O(n) per insert; it passes the corpus asserts but would TLE against LeetCode's real grader. The analysis acknowledges complexity loss on 0295 in prose, but the readout table ("Should language/stdlib change? yes") flattens the difference between "verbose but asymptotically correct" (most Category 1 cases) and "shape-wrong and asymptotically wrong" (some Category 3 cases). A parity-debt item should be flagged as a separate column in the table.

6. **`similarity_ratio` is not used** to distinguish "many small edits" from "mostly-rewritten". The JSON has it (e.g. `1397 ratio 0.089` vs `1203 ratio 0.126` vs `1631 ratio 0.102`) and it is a much better proxy for "rewritten" than raw sum. The analysis should use `similarity_ratio < 0.15` as a secondary filter — cases with high raw diff *and* low ratio are the ones least likely to be explained by formatting or helper drift.

## 3. Overreach toward Python — mostly avoided, one risk area

The "What this says about Sifr" and "What we should not do" sections are tight and I endorse them. One item in the Category 1 / Category 2 language-direction bullets needs a sharper constraint before it can be acted on safely:

- "**stronger flow-sensitive narrowing after `is not None`**" (Category 1 and Category 2). This is the right direction for single-access sites. It becomes unsafe if it is allowed to persist across a *reassignment* of the narrowed binding — e.g., `node = node.next` in a cursor loop, which is exactly the 0002/0021/0143 pattern. Python's model silently permits this because the access would raise at runtime. Sifr's narrowing should explicitly **re-narrow after rebinding**, and the analysis should state that as a hard constraint on the design so a future reader does not interpret "narrowing" as "permissive access once narrowed".

- "**more direct field and collection access once local safety is established**" (Category 1, line 54). "Local safety" needs a concrete definition, otherwise the rule is interpretive. Propose: "access at a site that is strictly dominated by a bounds check or `is not None` check on the same unaliased binding, with no intervening reassignment or call that could invalidate the check". Without pinning this down, the rule can slide toward Python-style implicit access by a future contributor in good faith.

- Category 1's "stdlib parity for core algorithmic tools" is correctly separated from "Python decorator parity" (line 57). Good — that is the subtlest trap in this analysis space and the doc avoids it.

Outside those two clarifications I did not find overreach toward Python. The `@cache` framing ("evidence that memoization should be ergonomic, not evidence that Sifr must copy Python's decorator model") is the cleanest part of the document.

## 4. Underreach on parity

This is where the document is weakest.

### 4a. `0023_merge_k_sorted_lists` is not a parity sample

`audits/leetcode/0023_merge_k_sorted_lists.sifr:3` changes the function signature from `list[ListNode] -> ListNode` to `list[list[int]] -> list[int]`. That is a breaking public-surface change. The Sifr file even labels itself "residual Sifr-safe canonical form" in its header comment, which is an honest admission that it is not the LeetCode 23 problem.

The analysis tucks this under "temporary algorithm substitution" (line 110) and recommends it "move closer to the Python shape over time" (line 131). That framing is too soft. Concretely:

1. While `0023.sifr` carries the current signature, it is not a valid parity sample for the corpus and **it should be excluded from the raw-diff scan** (e.g., renamed to `0023_merge_k_sorted_lists_v2.sifr` so the pair scanner stops matching it). There are already sixteen `*_v2.sifr` files flagged as `sifr_only` in the JSON (`verification/leetcode/leetcode_pair_diff_scan_20260409.json:12-29`), which means the convention for "Sifr-only placeholder variants" already exists. Use it.

2. The parity-debt entries (`0023`, `0148`, `0212`, `0295`) should be surfaced as tracked work items, not as aspirational "over time" prose. At minimum they need an `issues/` entry each with the blocking language/stdlib dependency.

### 4b. Parity aspiration without dependency chains

The recommended priority order at lines 175-194 places stdlib parity third. But the Category 3 parity-debt items depend on exactly those stdlib primitives:

| Parity-debt case | Blocking stdlib primitive |
| --- | --- |
| `0295_find_median_from_data_stream` | binary heap / `heapq` |
| `0212_word_search_ii` | trie-friendly nested-map ergonomics |
| `0023_merge_k_sorted_lists` | binary heap + recursive-object ergonomics |
| `1631_path_with_minimum_effort` | binary heap (Dijkstra) |
| `0778_swim_in_rising_water` | binary heap (Dijkstra) |
| `1489_find_critical_and_pseudo_critical_edges...` | union-find helper or canonical DSU |
| `0721_accounts_merge` | union-find helper or canonical DSU |
| `1203_sort_items_by_groups_respecting_dependencies` | `deque`-based topological sort |

This dependency chain is invisible in the current priority ordering. Without it, "priority 3: stdlib parity" reads as an open-ended wishlist. It should be rewritten as "priority 3: stdlib primitives that unblock these specific parity-debt cases".

### 4c. "May still pass" softens the finding

Line 116: "The fixture may still pass, but the representation, asymptotics, or public surface no longer mirrors the Python source closely". The three failure modes listed — representation, asymptotics, public surface — are not interchangeable. A changed *representation* that produces the same asymptotics and the same public surface is a stylistic divergence. A changed *public surface* (`0023`) is a correctness regression for parity purposes. A changed *asymptotic class* (`0295`) is an acceptance-criteria regression. The analysis should separate these three and raise the urgency on the latter two.

## 5. Prioritization — proposed adjustment

The document's order (lines 175-194):

1. Recursive node and cursor ergonomics
2. Safe collection and Optional expression ergonomics
3. Stdlib parity for algorithmic primitives
4. Corpus cleanup for comparison quality

Proposed adjustment:

**Step 0 — corpus normalization (currently listed as priority 4).** This needs to move first. Three concrete sub-tasks:

- Rename `0023_merge_k_sorted_lists.sifr` (and any similar cases where the Sifr version changes the public signature) to a `_v2` suffix so the pair scanner stops treating them as canonical comparison points. These are not parity samples.
- Add a normalization pass to `scripts/scan_leetcode_pair_diffs.py` that strips shared dead helpers (at minimum: `class Node` bodies unrelated to the problem, repeated `list_node_to_string` / `build_list_node`) on both sides before computing the diff. Otherwise mirrored boilerplate continues to be invisible and unilateral boilerplate continues to be overweighted.
- Emit median, p75, p90, p95 of `changed_total_lines` across the 395 pairs so bucket thresholds are interpretable.

Without step 0, any compiler work driven by this metric is optimizing against a signal that still contains parity-debt samples, mirrored dead code, and unnormalized formatting drift.

**Step 1 — Category 1 sub-split, hit the cheap wins first.** Category 1 as written is two distinct compiler tasks. They should be ordered separately:

1a. Kill spurious `int | None` narrowing on `list[int]` element access when the site is dominated by a bounds check on the same binding. This alone shrinks `0295` and likely all of the grid / DP fixtures in the representative list. This is the highest-leverage single change because it affects every fixture that touches a `list[int]`.

1b. Flow-sensitive re-narrowing on recursive field access (`node.next`, `node.val`) with explicit re-narrowing after reassignment. This is what unblocks Category 2.

In the current document these are fused. Splitting them makes the compiler scope and the expected fixture-level impact easier to track.

**Step 2 — Recursive node and cursor ergonomics** (currently priority 1). After step 1b lands, revisit the Category 2 cluster to see which remaining pain is actually ownership/mutability friction vs what was in fact Optional-flow friction in disguise. The analysis's claim that Category 2 is the highest-leverage cluster may survive or may shrink significantly — we will know after step 1b.

**Step 3 — stdlib primitives, in the order dictated by the dependency table in 4b.** Heap first (unblocks the largest number of debt items), then DSU, then deque, then trie ergonomics. Each primitive lands with an accompanying parity-debt clearance commit that converts the `_v2` placeholder back to the canonical form.

**Step 4 — parity-debt clearance backlog.** Remaining Category 3 items that need language work beyond the stdlib primitives (e.g., `0148` canonical linked-list merge sort depends on step 1b + step 2, not on stdlib).

The key inversion against the document's current order: **corpus normalization has to come first**, and **the Category 1 split should precede Category 2 compiler work** because a large fraction of what looks like Category 2 friction in the fixtures may be re-narrowing friction dressed up as ownership friction.

## 6. Smaller issues

- Line 82: "This is the highest-leverage category because many top raw-diff outliers cluster here." — the analysis does not quantify "many". Of the 15 rows in the readout table, 8 are recursive-object friction (Category 2) and 6 are safety scaffolding (Category 1). That is "most" for Category 2 — quote the split explicitly so readers can audit the claim.

- Lines 110-112: the descriptions of how `0148` / `0023` / `0212` / `0295` differ from their Python counterparts are correct, but `0295`'s description omits that the Sifr variant is O(n) per `addNum` (via slice-and-concatenate at lines 18-20 of the Sifr file). Call out asymptotic loss — it is the most consequential bit.

- Line 45: "stdlib parity for core algorithmic tools such as queues, heaps, and common map/list helpers" and line 185 "Priority queue / heap, queue / deque, tighter map/list helper surface" — these lists should be a single authoritative list with case dependencies (see 4b). Restating the list twice in different words in the same document makes the prioritization harder to audit.

- Line 199: "We should not add Python-style implicit nullable access." — concretize with the counter-example: `node.next` being readable after a prior `node is not None` guard is safe narrowing; `node.next.next` in a single expression without a guard on `node.next` is not. Without a concrete example the rule is interpretive.

- The scanner (`scripts/scan_leetcode_pair_diffs.py`) skips `run_audit.py` and `convert_all.py` as helper baggage (line 13) but does not skip the embedded dead `class Node` / `ListNode` helpers inside individual problem files. The normalization in step 0 should extend the helper-skip set into intra-file stripping.

## 7. What I think is right and should stay

- The four-category structure is the right shape. The fix is better separation between categories, not collapsing them.
- The "language direction" per category is correct at the direction level. It needs the two sharpened constraints in §3 above.
- The refusal to copy Python decorator semantics for `@cache` is the best judgment call in the document. Keep it explicit.
- The identification of Category 2 (recursive-object ergonomics) as a top-priority cluster is correct even after my proposed re-ordering — I am only asking that step 1b land first so we can measure how much of the cluster survives the fix.
- The "do not do" list at lines 197-201 is the clearest boundary statement in the document. Keep it verbatim.

## Prioritized fixes for the analysis document itself

In order of how much they change the interpretation:

1. Move corpus normalization from priority 4 to priority 0, and rename/relabel `0023_merge_k_sorted_lists.sifr` as a `_v2` non-parity sample. Without this the rest of the prioritization is built on a still-poisoned baseline.
2. Split Category 1 into (1a) list/index Optional-flow and (1b) recursive-field Optional-flow, and make (1a) the first compiler step because of its breadth across the corpus (0295 evidence).
3. Add the stdlib → parity-debt dependency table and reorder priority 3 by case-unblock count.
4. Distinguish representation / asymptotic / public-surface regressions in Category 3, and treat asymptotic and public-surface regressions as tracked parity debt rather than aspirations.
5. Reclassify `0148` and `0295` as multi-category entries (2+3+4 and 1+3 respectively); drop `0200` from Category 4 until it is substantiated.
6. Tighten the "narrowing after `is not None`" language with explicit re-narrowing-after-reassignment semantics.
7. Add median / p75 / p90 to the bucket thresholds so "16 pairs at ≥120" is interpretable.
