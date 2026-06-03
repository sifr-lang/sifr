Now I have a complete picture. Let me write the review.

---

# Review Pass 2: `issues/ad-hoc-leetcode-benchmark-slowness-root-cause-analysis.md`

## Count Reconciliation: Precise and Defensible ✓

Verified against the raw document:
- Main table: **75 rows, 75 unique problem IDs** — no duplicates
- Appendix: **53 rows, 53 unique problem IDs**
- `0234_palindrome_linked_list` appears in **both** main table (has 2 complete pairs) and appendix (1 pair missing) — correctly documented
- Count arithmetic is internally consistent: 75 slower + 52 true failures + 1 partial = 128 covered problems

The count reconciliation section is clear, ground-truth-backed, and defensible.

## 75-Row Table Covers All Measured-Slower Problems Exactly Once ✓

Every row has exactly one problem ID. No duplicate entries. All problem IDs in the table are unique. The only dual-appearance case (`0234`) is intentional and explained.

## 53-Row Incomplete/Failed Appendix: Useful, Not Misleading ✓

Every entry has:
- Problem ID
- Failure mode category (type error / partial / correctness / build error / timeout)
- Representative error excerpt

The `0234` entry correctly notes partial completeness ("complete pairs exist for some sizes"). The mix of type errors (move semantics, index access), build errors, correctness failures, and timeouts is appropriate and actionable for separate work tracks.

## Compiler-vs-LeetCode-Code Split: Sound ✓

All 75 problems in the main table are classified with a primary owner. The four tracks (C1–C4 + L1–L3) are internally consistent with the root cause descriptions. Cross-category overlaps (e.g., `0535` in C1+C2, `0706` in C1+C2) are intentional and justified by the evidence.

## Emitted Rust Snippets and Metadata Plan: Sufficient ✓

Four concrete snippets are present:
- `1985` → `chars().count()` in comparator
- `0535` → `self.encodeMap.clone().contains_key(...)`
- `0208`/`0211` → full `_children`/`_terminal` vector clones
- Object-op runner → `Vec<String>` parsing overhead

The metadata plan specifies concrete fields (`parity_status`, `primary_slowness_owner`, `slowness_tags`) with machine-readable paths. This is the right abstraction level for driving future benchmark tooling.

---

## Findings

### Required Fixes

None. All pass 1 required findings are resolved.

### Optional Polish (Low Priority)

**P1: `0703_kth_largest_element_in_a_stream` is assigned to C1 (String Indexing) but the evidence points to L1 (Heap Parity)**

In the C1 problem families (line 254):
> `0003`, `0014`, `0058`, `0067`, `0125`, `0187`, `0205`, `0392`, `0402`, `0424`, `0567`, `0647`, `0680`, `0763`, `0929`, `1189`, `1456`, `1461`, `1768`, `1888`, `1930`, `2405`

`0703` is listed here but does not appear in the 22-problem enumeration. Let me verify.

Actually, re-reading: the C1 list contains 22 IDs and `0703` is not among them. The root cause note in the table for `0703` says "Python commonly uses heap discipline; Sifr stream implementation uses list manipulation and pays extra object-state clones." The primary issue is L1 (heap parity), not string indexing. This is correctly handled — `0703` is in L1, not C1. **No action needed.**

**P2: `0015_3sum` — C category gap**

The root cause note says "Sifr uses brute-force triple loops plus deduplication" (algorithmic) but also "string/list operations allocate more" (compiler overhead). The L1/L2 tracks cover algorithmic parity, but the compiler overhead from `clone()` calls in the brute-force implementation is not mapped to a C-track family. The problem is absent from C1–C4.

Minor: the algorithmic gap is the dominant factor (correctly attributed to LeetCode Sifr code), so this doesn't misdirect work. But for completeness, the compiler-side overhead from this implementation would fall under C2 (dict/list operations with clones).

**P3: `0149_max_points_on_a_line` in C4 may be misclassified**

C4 problem families list `0149` under "graph/DP matrix cases." But `0149_max_points_on_a_line` is a computational geometry problem with no matrix or grid structure — it computes slopes between all point pairs. The root cause in the table is "dict key/value cloning and tuple/string formatting overhead" — this maps to C2, not C4. `0149` is absent from C1 and C2 in the problem family lists.

Recommend moving `0149` from C4 to C2 in the problem families section (line 320), or clarifying that C4 covers "dict/set heavy graph-adjacent problems" rather than specifically matrix cases.

### No Action Required — Already Correct

- `0239_sliding_window_maximum` is correctly classified as LeetCode Sifr code with high confidence (no verification qualifier).
- `1834_single_threaded_cpu` is correctly classified with O(n log n) vs O(n²) complexity note.
- `1046_last_stone_weight` correctly notes repeated sort vs heap.
- `0015_3sum` is correctly classified as LeetCode Sifr code (brute force vs two-pointer).

---

## Summary

| Check | Status |
|---|---|
| Count reconciliation precise | ✓ |
| 75 rows = 75 unique IDs | ✓ |
| 53-row appendix useful and accurate | ✓ |
| Compiler/LeetCode split sound | ✓ |
| Emitted Rust evidence present | ✓ |
| Metadata plan actionable | ✓ |
| Pass 1 required fixes resolved | ✓ |

**Recommendation**: The document is elegant and actionable. The three optional polish items (P1–P3) are minor and do not affect the primary conclusions or work tracks. The phase can proceed to M1 milestone work without further revision passes.
