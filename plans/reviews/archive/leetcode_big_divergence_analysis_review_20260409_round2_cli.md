

# Round-2 Review — LeetCode Big-Divergence Analysis

## Source

`verification/leetcode/leetcode_big_divergence_analysis_20260409.md`
Round-1 feedback (round1f): treated as accepted in principle.

---

## 1. Revised Ordering — Sound, but Incomplete Acceptance of Round-1 Premises

The document retains its original four-priority structure (recursive-object ergonomics → safe collection/Optional ergonomics → stdlib parity → corpus cleanup). The round-1 review called for two structural changes to that order:

- **Corpus normalization to priority 0** (rename `_v2` non-parity samples, normalize intra-file dead helpers before computing diff)
- **Category 1 split into 1a (list/index Optional-flow) then 1b (recursive-field narrowing)**

Neither change appears in the document. The gap matters because the round-1 review gave a concrete mechanism for priority 0: rename `0023_merge_k_sorted_lists.sifr` to a `_v2` suffix (the convention already exists for 16 other `sifr_only` files in the scan JSON), add a normalization pass that strips shared intra-file boilerplate (`class Node`, `list_node_to_string`) before computing the diff, and emit median/p75/p90 so the bucket thresholds are interpretable. None of those sub-steps appear.

The ordering is therefore still the round-1 ordering, not a round-2 ordering. The direction is defensible; the incomplete acceptance of round-1 premises leaves it on uncertain footing.

---

## 2. Overstated or Under-Evidenced Claims

### 2a. Bucket thresholds stated as calibrated (lines 11-13)

The document describes "16 pairs at ≥ 120 changed lines", "33 pairs at ≥ 100", and "53 pairs at ≥ 80" as triage buckets, but the surrounding text acknowledges that no normalization has been applied. The round-1 review showed that `changed_py_lines + changed_sifr_lines` from `difflib.SequenceMatcher.get_opcodes()` with no whitespace trim, no comment strip, no import strip, and no intra-file helper stripping means these counts contain unknown quantities of format drift and mirrored dead code. Calling them "calibrated" in the table header is stronger than the underlying metric supports. They should be presented as raw bucket counts with that caveat repeated at the table.

### 2b. Category 4 framing still assumes unilateral noise

The document's Category 4 description (line 140) describes noise as residing entirely on the Python side. The round-1 review showed that `0148` carries a mirrored dead `class Node` on both sides that cancels out of the diff entirely — meaning Category 4 noise is invisible to the metric, not absent from the files. This matters for how the document uses Category 4 as evidence: it currently argues "do not change the language for this" partly because the metric may be overstating divergence. But the metric may also be *understating* divergence where shared dead helpers mask real structural differences. The category framing should acknowledge both directions.

### 2c. `0295` still classified as Category 3 only

The round-1 review demonstrated that `0295`'s Sifr fixture contains dead `int | None` narrowing on `self.nums[n // 2]` even though `self.nums: list[int]`. This makes `0295` simultaneously a Category 3 case (missing heap) and a Category 1 case (spurious Optional-flow scaffolding). The document lists it in Category 3 with no cross-reference to Category 1. Given that `0295` was the primary evidence for the "1a list/index Optional-flow is the highest-leverage single change" argument, omitting it from Category 1 undercuts the revised priority ordering's first step.

---

## 3. Single Most Important Remaining Blind Spot

**The stdlib dependency table was identified but not integrated.**

The round-1 review (table at lines 88-98) showed that six of the seven highest raw-diff cases have a direct stdlib dependency that blocks their parity clearance:

| Blocking primitive | Cases it unblocks |
|---|---|
| binary heap / `heapq` | 0295, 1631, 0778 |
| union-find helper | 1489, 0721 |
| `deque`-based topological sort | 1203 |
| trie ergonomics | 0212 |

The document's priority 3 ("stdlib parity for algorithmic primitives") lists heap, deque, map/list helpers, and trie without linking any of them to specific parity-debt cases. A reader cannot audit whether "priority 3" is the right third step or whether heap should precede trie by two items. The round-1 dependency table is the document's own analytical contribution — leaving it hanging after citing it in the round-1 review means the prioritization still lacks the internal logic needed to be acted on reliably. This is the most consequential unresolved gap.
