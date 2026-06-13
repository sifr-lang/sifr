# LeetCode Divergence Decision Analysis — Adversarial Parity-Risk Review

Date: 2026-04-22
Target: `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`
Raw scan: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
Audit pairs: `audits/leetcode/`

Scope: try to falsify the classification. Only material, concrete corrections are listed.

## Summary of findings

- Four above-/near-threshold fixtures are misclassified as Category 1 "rewrite" when they are structurally Category 4 architecture boundaries under the stated no-aliasing / no-interior-mutability invariants: `0133`, `0138`, `0160`, plus the latent `0141` pattern.
- Four material below-cutoff parity-debt fixtures are missing from Category 1 entirely: `0146_lru_cache`, `0208_implement_trie_prefix_tree`, `0211_design_add_and_search_words_data_structure`, `0706_design_hashmap`.
- One below-cutoff fixture does not implement the algorithm at all and should be flagged as a distinct failure class: `0141_linked_list_cycle`.
- The Feature Ledger has no ID for non-owning node references / graph handles, yet Category 1 acceptance criteria for `0133`/`0138`/`0160` presuppose one. The rewrite table cannot be driven to closure as written.
- Category 3/5 is correctly narrow. No hidden stdlib debt there beyond the `0516` sentinel, which should be linked into `I2`.
- Category 4 is not over-used; `0673`/`0894` rationale holds. The gap is under-use.
- Boundaries section has no unsafe feature phrasing. `I1`/`I2`/`N3` are consistent with "no implicit nullable access"; they are proof-gated flow narrowings, not universal subscript retyping.

## 1. Category 1 → Category 4 reclassifications (aliasing/cycle shapes)

The analysis lists these as Category 1 with Cat 1 Acceptance criteria. All four require node-identity aliasing or cycles in the owned data model, which collides head-on with the boundaries:

- "Do not weaken ownership to emulate Python aliasing."
- "Do not introduce interior mutability (`Rc<RefCell<...>>`, `Cell`, etc.)."

So the "canonical Python problem shape" is not reachable under the stated design without a new language feature. That is definitionally a Category 4 boundary, not a rewrite target.

- `0133_clone_graph`: cloning an adjacency-graph of `Node` with cycles requires either `Rc`-style sharing in the clone, or an indexed-handle model. Neither is in the ledger. The Python pair relies on Python object identity inside a `dict[Node, Node]`; under single ownership, the visited-map would own each cloned node, which makes neighbor lists of other nodes un-expressible.
- `0138_copy_list_with_random_pointer`: identical issue — the `.random` pointer in the copy must point to *another copy* that is already owned by its own chain slot. Only an aliasing mechanism or index-based handle model permits this. Canonical rewrite target "random pointers target cloned nodes" is unreachable under current boundaries.
- `0160_intersection_of_two_linked_lists`: the canonical problem has two heads whose tails are the same node. That is an aliasing constraint in the *input*. The current fixture already betrays this: `headA = ListNode(4, ListNode(1, shared))` and `headB = ListNode(5, ListNode(6, ListNode(1, shared)))` consume `shared` twice; this compiles today only because `ListNode` is not treated as `own` in the constructor. A canonical rewrite that asserts *node identity* is structurally impossible under "each node owned exactly once at every program point".
- `0141_linked_list_cycle` (not currently listed anywhere — see §3): same shape, the input must contain a cycle.

Concrete edits:

- Move `0133_clone_graph`, `0138_copy_list_with_random_pointer`, `0160_intersection_of_two_linked_lists` from Category 1 into Category 4, paired with the same rationale already used for `0894_all_possible_full_binary_trees` ("Sifr's single-ownership model requires …"). Replace the Cat 1 entries with Cat 4 entries that pin the boundary as *node-identity aliasing / cycle in the owned model*.
- In the Rewrite Debt Execution Table, strike the three rows for `0133`, `0138`, `0160`. A fixture in Cat 4 does not owe a canonical rewrite; it owes a documented boundary plus (optionally) a Sifr-native alternate (a value-semantic variant of the problem, with its own assertions).
- Add an explicit subcategory header to Category 4: "4a: Nonlocal / closure-state architecture" (existing 0673 rationale) and "4b: Node aliasing / cycle shapes under single ownership" (new 0133/0138/0141/0160). Keep 0894 in 4b since its rationale is the same "shared subtree aliasing" pattern.

## 2. Category 1 misses — below-cutoff parity-debt

The analysis uses `changed_total_lines >= 80` with explicit exceptions below the cutoff where there is an asymptotic regression or a public-model flip. By that same rule, the following are also exceptions and are missing:

### 2.1 `0146_lru_cache` (total=79, below cutoff by one line)

- Canonical: doubly-linked list + hashmap, O(1) `get`/`put`. Sifr: parallel `keys`/`values` arrays with `findIndex` linear scan, `pop(idx)` linear shift, `pop(0)` linear shift. Asymptotic regression from O(1) to O(N) per op. Identical pattern to `0295_find_median_from_data_stream` which *is* listed as a below-cutoff exception.
- Files: [0146_lru_cache.sifr](audits/leetcode/0146_lru_cache.sifr:13), [0146_lru_cache.py](audits/leetcode/0146_lru_cache.py:9).

Edit: add `0146_lru_cache` to the below-cutoff parity-debt exception list in Category 1. Rewrite table row:

| Fixture | Canonical Target | Key Prereqs | Acceptance |
| --- | --- | --- | --- |
| `0146_lru_cache` | doubly-linked-list + hashmap, O(1) get/put | `C1`, `C2`, `I2`, `B1` (and the same 4b boundary resolution that unblocks `0133`/`0138`) | no linear scans in `get`/`put`; node removal is pointer-relink, not array shift |

Note: if `0146` really needs a doubly-linked list of *owned* nodes with prev/next back-pointers, it has the same aliasing-requirement issue as 4b. So this may also be Category 4b. Flagging both options: the analysis should decide explicitly.

### 2.2 `0208_implement_trie_prefix_tree` (total=79)

- Canonical: 26-child trie, O(L) per op. Sifr: `words: list[str]` with linear scan per op ([0208_implement_trie_prefix_tree.sifr:13](audits/leetcode/0208_implement_trie_prefix_tree.sifr:13)). Asymptotic regression from O(L) to O(N·L).
- This is the canonical trie *design* problem. Leaving it below the parity-debt bar is inconsistent with listing `S6` ("trie decision and API") and with flagging `0212_word_search_ii` as Cat 1.

Edit: add `0208_implement_trie_prefix_tree` to below-cutoff Cat 1 exceptions. Add to `S6` exit signal. Rewrite table row:

| `0208_implement_trie_prefix_tree` | canonical trie with per-character child links | `S6` | no word-list scan; lookup per character |

### 2.3 `0211_design_add_and_search_words_data_structure` (total=71)

- Canonical: trie + DFS on `'.'` branches, O(26^dots · L). Sifr: list of strings with per-word character loop ([0211_design_add_and_search_words_data_structure.sifr:12](audits/leetcode/0211_design_add_and_search_words_data_structure.sifr:12)). Same regression as 0208 plus the wildcard branching is also absent.
- Depends on the same `S6` as 0212.

Edit: add to below-cutoff Cat 1 exceptions and `S6` exit signal.

### 2.4 `0706_design_hashmap` (total=62)

- Canonical (purpose of the problem): implement a hashmap without using a built-in hashmap. Sifr: `map: dict[int, int]` and delegates `get`/`put`/`remove` to the built-in dict ([0706_design_hashmap.sifr:4](audits/leetcode/0706_design_hashmap.sifr:4)). This is the exact data structure the problem forbids.
- Separately, `remove(key)` stores `-1` as a sentinel instead of deleting. That happens to satisfy the fixture assertions only because `get` returns `-1` on absence. Any caller that stores `-1` as a real value would expose the bug.

Edit: add `0706_design_hashmap` to below-cutoff Cat 1 exceptions with rewrite-table row:

| `0706_design_hashmap` | buckets-of-`ListNode` hashmap (chaining) | `C1`, `C2`, `B1`, 4b boundary | no delegation to built-in `dict`; `remove` is real deletion, not `-1` sentinel |

Note same 4b caveat: a chained hashmap needs owned-node chain manipulation. If 4b is not unblocked, 0706's canonical target degrades to "open-addressed array" which is a valid alternate target — document which is chosen.

## 3. `0141_linked_list_cycle` — absent-algorithm failure class

[0141_linked_list_cycle.sifr:72](audits/leetcode/0141_linked_list_cycle.sifr:72):

```sifr
def hasCycle(own head: ListNode) -> bool:
    return False
```

The test ([0141_linked_list_cycle.sifr:76](audits/leetcode/0141_linked_list_cycle.sifr:76)) only exercises `ListNode(0, None)` — a no-cycle input. So the "implementation" passes by always returning `False`.

This is not divergence. It is an absent algorithm masked by a cycle-input that cannot be constructed in Sifr's owned model (same 4b boundary as 0160). The analysis scope section says "'Failure' means a Sifr fixture does not compile or run correctly" — 0141 compiles and runs, but its post-condition (correctness on cycle inputs) cannot be tested because the test cannot construct one.

Edit: add a new bullet under "Execution Scope":

> - Some fixtures whose canonical input requires node aliasing or cycles pass their test suites only because the test cases avoid the shape the problem is about. `0141_linked_list_cycle` is the current clear case (`hasCycle` unconditionally returns `False`; no cycle-input test). Classify these under Category 4b alongside `0160`; add a note that the fixture's test cases need a value-semantic alternate or are moot under current boundaries.

Also list `0141` explicitly in the new Category 4b.

## 4. Feature Ledger gap — no handle / non-owning reference feature

The Rewrite Debt Execution Table acceptance for `0133`/`0138`/`0160` is unreachable given the current ledger. If §1 is adopted these rows move to Cat 4; if not, the ledger is materially incomplete and must add an ID, e.g.:

| ID | Scope | Example Fixtures | Exit Signal |
| --- | --- | --- | --- |
| `G1` | non-owning node handles (e.g., indexed graph nodes, arena-backed references) with borrow-safe access | `0133`, `0138`, `0160`, `0141`, `0894` alt | shared/cyclic graph shapes expressible without `Rc<RefCell<...>>` and without weakening ownership |

Without `G1` (or equivalent), Category 1 cannot drive the linked-/graph-aliasing rewrites. Leaving the ledger as-is silently papers over the gap. Pick one of:

1. Move the aliasing fixtures to Cat 4b and drop their rewrite rows. Preferred.
2. Add `G1` to the ledger and make it a prereq on the rewrite rows.

The analysis must pick explicitly — the current state is ambiguous.

## 5. Category 3/5 parity check

The four Cat 3 fixtures (`0104`, `0130`, `0200`, `0516`) are correctly "okay as-is". One correction:

- `0516_longest_palindromic_subsequence` uses `memo.get((i, j), -1)` as a sentinel for absent key. The analysis acknowledges this but does not link it into `I2`. It should: `I2`'s exit-signal list currently excludes `0516`, so the `memo.get(_, sentinel)` pattern won't be treated as an I2 acceptance test, which means corpus cleanup can "pass" I2 while leaving the sentinel intact.

Edit: add `0516` to `I2` example fixtures, or mirror the language from the `I1` row into an explicit `I2.b` covering sentinel-return `.get(k, s)` patterns. Minor.

## 6. Category 4 is not over-used

- `0673` rationale is sound. The layered 2b pressure note (`valueAt` linear scan) is correct and properly quarantined. Suggest adding `0673` to `I1` example fixtures so asymptotic recovery has a named exit signal — currently `I1` lists `0673` already; good.
- `0894` rationale holds. The memoization-drop note is correct. One small addition: the acceptance that "the fixture still produces all FBTs for n up to K" should be pinned explicitly — without memoization the algorithm re-solves subproblems per parent, so the fixture may be borderline for larger n. If n=7 is the test, fine; if someone raises to n=15, the fixture becomes a CPU-burn risk. Optional hardening, not material today.

No Cat 4 over-reach found.

## 7. Boundary phrasing audit

No unsafe phrasing survived into the Boundaries section. Specifically:

- "Do not change the abstract return type of `list` / `dict` subscripts" is not weakened by `I1`/`I2`. Both rows explicitly describe *local flow-sensitive* narrowing with "no mutation intervenes" invalidation — consistent.
- "Do not introduce interior mutability" is respected by `O1` ("helpers compile without `Rc<RefCell<...>>` or interior mutability") and `C1`–`C3`.
- "No auto-insert-on-read" is correctly mirrored in `S6`.
- The only potential tension is with `0133`/`0138`/`0160`/`0141`/`0706` (chained hashmap) / `0146` (doubly-linked LRU) — those tensions are *structural*, not phrasing issues, and are the topic of §1 and §2.

## 8. Priority-order impact

With §1 adopted:

- Wave 0 gains clarification: move the four aliasing fixtures to 4b and record that none owes a rewrite. This also retires three rows from the Wave 7 "Final rewrite sweep".
- Wave 4 ("Owned-chain cursor ergonomics") remains the right home for `0021`/`0024`/`0025`/`0092`/`1669`/`1721`, which are single-chain not aliased-graph.
- Wave 6 ("Trie decision") gains `0208` and `0211` as explicit exit-signal fixtures.

With §1 rejected (choosing `G1` instead):

- Add `G1` to the ledger with exit signal and a Wave slot between 3 and 4 (graph-handle model must predate any graph-rewrite).
- Add `G1` as a prereq to the `0133`/`0138`/`0160`/`0146` (chaining variant) rows.

## 9. Concrete edit list

Minimal set of edits to make the analysis adversarially consistent:

1. Relabel `0133`, `0138`, `0160` as Cat 4b; delete their rows from the Rewrite Debt Execution Table.
2. Add Cat 4b subheader with `0141`, `0133`, `0138`, `0160`, `0894` (and note `0673`/`0894` sit in 4a/4b respectively if splitting).
3. Add `0141_linked_list_cycle` to Cat 4b with a note about the passing-by-vacuous-test pattern. Add a corresponding bullet to "Execution Scope".
4. Add to Cat 1 below-cutoff exceptions: `0146_lru_cache`, `0208_implement_trie_prefix_tree`, `0211_design_add_and_search_words_data_structure`, `0706_design_hashmap`. Add their rewrite-table rows with the prereqs shown in §2. Decide explicitly whether the chained variants push into 4b.
5. Link `0516` into `I2` example fixtures (for the `.get(k, sentinel)` pattern).
6. Either add `G1` (non-owning node handles) to the Feature Ledger or commit to the Cat 4b path; the current state leaves the ledger unable to drive closure of the aliasing rewrites.
7. Optional: add asymptotic acceptance criteria to the Rewrite Debt Execution Table rows where the classification hinges on asymptotics (`0004`, `0146`, `0208`, `0211`, `0295`, `0706`, `0707`), so "canonical" is measurable, not just shape-matched.

No changes needed elsewhere — Cat 2a/2b membership, Cat 3 choices, and the boundary list are internally consistent once §1–§4 land.
