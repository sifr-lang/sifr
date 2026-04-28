# LeetCode NO_ORACLE Assertion Parity — Review Pass 4 (Final, 2026-04-26)

## Lens

Pass 4 re-checks the residuals from pass 3 — most importantly the Medium F7 (0160 fixture-tuned identity hack) and the missing ownership-workaround comments — and sweeps the remaining touched files for any new regression. No files were edited.

## Scope

- 14 modified `.sifr` and 5 modified `.py` fixtures under `audits/leetcode/` (set unchanged from pass 3).
- `verification/leetcode/full_corpus_manifest_20260402_live.json`: `case_count == 411`, `cases | length == 411`, all 411 `fixture_slug`s unique, `oracle.mode` is single-valued `"embedded_asserts"`. Diff body is mode-flip-only (203 `+ "mode": "embedded_asserts"` / 203 `- "mode": "no_oracle"`, no other field churn).
- `/tmp/sifr_full_corpus_after_review_fixes_v2_20260426.json`: `summary.status_counts == {"PASS": 411}`, `summary.scope_counts.in_scope == 411`, no `blocked_feature` / `out_of_scope_external_dep`.

---

## Findings (re-check of pass-3 residuals)

### F1 — 0160 fixture-tuned `pop()` replaced with stable-id identity model — Severity: **None** (resolved)

Pass 3 F7 (Medium): `getIntersectionNode` walked both lists into value lists, suffix-matched, and `pop()`-ed exactly one element — a hack that only worked for "false-suffix-of-length-1" inputs.

Current state ([audits/leetcode/0160_intersection_of_two_linked_lists.sifr:48-66](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:48)):
- `ListNode` gained a `node_id: int` field ([…sifr:5-13](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:5)) and a `nodeId` accessor ([…sifr:25-28](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:25)).
- The algorithm is now: collect `idsA: set[int]` from list A, walk list B and return `cloneList(curB)` at the first `nodeId(curB) in idsA`. O(|A|+|B|) time, O(|A|) space, no `pop`/suffix arithmetic.
- The boundary-fixture comment at [audits/leetcode/0160_intersection_of_two_linked_lists.sifr:49-50](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:49) honestly states: "Sifr cannot share the same owned tail … so the fixture uses stable node ids to model the Python identity-based contract."
- Asserts traced:
  - Test 1 ([…sifr:69-73](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:69)): headA = ids `[1,2,3,4,5]`, headB = ids `[8,7,6,3,4,5]`. `idsA = {1,2,3,4,5}`. Walk B until id 3 hits → return `cloneList` of B's `8→4→5` chain → `"8->4->5"`. ✓
  - Test 2 ([…sifr:75-77](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:75)): disjoint ids `{10,11,12}` vs `{13,14}` → returns `None` → `"None"`. ✓
- `cloneList` ([…sifr:43-46](audits/leetcode/0160_intersection_of_two_linked_lists.sifr:43)) handles the `own headA, own headB` ownership constraint cleanly — the returned chain is independent of either input.

This fully addresses pass 3 F7. The id-set approach is canonical Sifr and the algorithm is sound for any well-formed input, not just the asserted shapes.

Residual (Low, latent): correctness still depends on fixture-construction discipline — nodes that conceptually represent "the same node" must be assigned matching `node_id`s, and unrelated nodes must have unique ids. Nothing in the file enforces this. A future fixture author who re-uses an id between unrelated lists would silently produce a false intersection. Worth a one-line note at the top of `main` if anyone adds a third test, but not blocking.

### F2 — 0146 ownership-workaround comment added — Severity: **None** (resolved)

Pass 3 residual #2 (Low): the 5-dict-of-ints LRU design was undocumented; a "simplification" PR could plausibly try to revert it to an aliased `Node` shape and rediscover the ownership constraint the hard way.

[audits/leetcode/0146_lru_cache.sifr:3-4](audits/leetcode/0146_lru_cache.sifr:3) now reads: "Use integer node ids instead of a cyclic Node graph; this keeps the fixture ownership-safe while preserving the doubly-linked-list LRU behavior." Concise and load-bearing — names the constraint and the behavior preserved. No code changes; capacity-1 coverage from pass 3 is intact ([…sifr:129-134](audits/leetcode/0146_lru_cache.sifr:129)).

### F3 — 0235 / 0236 ownership-workaround comments added — Severity: **None** (resolved)

Pass 3 residual #3 (Low): `cloneTree(root)` returned at each match in 0235 (and equivalent `cloneNodeByValue` in 0236) was undocumented as an ownership trade.

Both files now carry an explicit comment at the entry to `lowestCommonAncestor`:
- [audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:14](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:14): "Return a cloned subtree because borrowed input nodes cannot escape."
- [audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:25](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:25): same wording.

That's the right place and the right wording — a future reader sees it before they try to "optimize" the function down to returning `root` directly. Implementation is otherwise unchanged from pass 3 (canonical recursive descent; verified PASS on the embedded asserts in pass 3, unchanged here).

---

## Other carryover items (unchanged since pass 3, all Low / Latent)

- **0102 still imports `treeToString` unused** ([audits/leetcode/0102_binary_tree_level_order_traversal.sifr:2](audits/leetcode/0102_binary_tree_level_order_traversal.sifr:2)). Same dead-import pattern that pass 3 cleaned up on 0094 / 0103. Low — clippy/lints would catch it; doesn't affect correctness.
- **0102 still uses recursive level-merge** instead of BFS, while sibling 0103 demonstrates BFS (carried from pass 2 F10). Stylistic inconsistency, not a correctness issue.
- **`expected_empty` vs literal `[]` mix is unchanged** (0094 literal; 0102 / 0103 / 0212 named local). Low, cross-fixture style.
- **0706 latent: Sifr `hashcode` corrects negatives ([audits/leetcode/0706_design_hashmap.sifr:23-24](audits/leetcode/0706_design_hashmap.sifr:23)), Python does not.** No tested negative keys — no current divergence, but it's a real semantic gap if a future test ever uses one. Note also the Sifr `put`/`remove` rebuild buckets into a fresh `next_bucket` and re-store `self.buckets = buckets` after `buckets[index] = next_bucket` ([…sifr:45-47](audits/leetcode/0706_design_hashmap.sifr:45), [..sifr:70-72](audits/leetcode/0706_design_hashmap.sifr:70)) — ceremonial unless Sifr's mutation rules require it; no comment explains either way. Carryover from pass 3 residual #5.
- **1203 keeps the degenerate `topologicalSort([[0]], [0], 0) == []` assert** ([audits/leetcode/1203_sort_items_by_groups_respecting_dependencies.sifr:99](audits/leetcode/1203_sort_items_by_groups_respecting_dependencies.sifr:99)). `num_nodes=0` short-circuits the for-loop; `sortItems` is never called. Mirrors the Python pair, so parity-equal, but the fixture provides zero direct coverage of the 50-line `sortItems` body. Carried.
- **0235/0236 `nodeVal(None) -> 0` sentinel collision** ([0235:1-4](audits/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr:1), [0236:4-7](audits/leetcode/0236_lowest_common_ancestor_of_a_binary_tree.sifr:4)). Asserts don't trigger; latent.

---

## Manifest / data consistency

- 411 cases, 411 unique `fixture_slug`s, single-valued `oracle.mode == "embedded_asserts"`.
- Mode-flip-only diff (203 `+` / 203 `-` `"mode"` lines, no other field churn).
- `/tmp/sifr_full_corpus_after_review_fixes_v2_20260426.json`: `summary.status_counts == {"PASS": 411}`. End-to-end run is honest with respect to the manifest.

---

## Residual risks / test gaps

1. **0160 id-discipline is unenforced.** The id-set approach is correct for well-formed inputs, but the fixture relies on the author assigning matching ids to "shared" nodes and unique ids elsewhere. A future test that breaks that discipline silently misclassifies. Consider one of: (a) a one-line comment at the top of `main` stating the discipline; (b) an `id_counter` helper that auto-generates sequential ids and is reused for the shared chain. Not blocking.
2. **0102 unused import.** Trivial dead import; clippy / `cargo fmt --check` likely won't catch it but a Sifr-side unused-import lint would. Carry into a follow-up cleanup.
3. **0706 negative-key divergence is latent.** Worth tracking — adding even one negative-key assert to the parity pair would either close the gap or expose a real divergence.
4. **1203 has zero direct `sortItems` coverage.** Parity-equal to the Python pair, but a meaningful corpus-level coverage gap.
5. **No corpus-level lint catches trivial-stub regressions** (carried from pass 3 residual #7). The class of regression that 0236 had (`return None` body with asserts comparing to `None`) could re-emerge in any of the 188 untouched promoted fixtures. Out of scope here; worth tracking in `internal_docs/`.
6. **Local-validation breadth still unverified.** Per [AGENTS.md](AGENTS.md) the authoritative gate is `scripts/run_all_tests.sh --profile quick`. The user reported PASS=411 from `run_phase31_leetcode`; the working tree carries no evidence that `cargo test`, `cargo clippy --workspace -- -D warnings`, or the `--profile quick` aggregate ran. Confirm before merge — clippy is the most likely place an unused-import or dead-helper would surface (e.g. the dead `treeToString` import in 0102).

---

## Summary

| # | Pass-3 residual                                       | Pass-4 status |
|---|-------------------------------------------------------|---------------|
| 1 | F7 0160 fixture-tuned `pop()` identity hack           | Resolved      |
| 2 | 0146 ownership-workaround undocumented                | Resolved      |
| 3 | 0235 / 0236 ownership-workaround undocumented         | Resolved      |
| 4 | 0102 unused `treeToString` import / recursive level   | Carried (Low) |
| 5 | `expected_empty` vs literal `[]` style mix            | Carried (Low) |
| 6 | 0706 negative-key hashcode divergence                 | Carried (Low / latent) |
| 7 | 1203 degenerate-input-only assert                     | Carried (Low) |
| 8 | 0235 / 0236 `nodeVal(None)→0` sentinel collision      | Carried (Latent) |
| 9 | No corpus-level trivial-stub lint                     | Out of scope  |
|10 | Local-validation breadth (clippy / `--profile quick`) | Unverified    |

**Verdict: no blocking correctness or canonicality issues remain.** The previously-Medium F7 (0160) is resolved by switching to stable-id identity modeling with an honest fixture comment; the previously-Low ownership-workaround documentation gap is resolved on 0146 / 0235 / 0236. Manifest invariants hold (411 unique slugs, all `embedded_asserts`, mode-flip-only diff) and the run summary reports PASS=411 in scope. The remaining residuals are style, documentation, or corpus-level coverage concerns that are safe to address in follow-ups; the only non-test action recommended before merge is running `scripts/run_all_tests.sh --profile quick` (or at minimum `cargo clippy --workspace -- -D warnings`) to back-stop the unused-import note on 0102.
