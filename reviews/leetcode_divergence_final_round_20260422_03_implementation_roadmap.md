# LeetCode Divergence Decision Analysis — Final Round Review

Date: 2026-04-22
Angle: Implementation roadmap and actionability
Under review: `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`
Supporting data: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`, `audits/leetcode/*`

## Summary Verdict

The analysis is a solid *classification* document but it is **not yet a plan**. The categories name the problem shape but stop short of what an implementation owner needs to open issues, sequence work, and know when to stop: there is no DAG of prerequisites, no per-feature exit criterion, no per-fixture acceptance delta, and no distinction between "compile-failing" and "passing-but-divergent" fixtures. The Practical Priority Order is directionally right but several of its items are multi-feature umbrellas that must be split before they can be scheduled.

Top three gaps to fix before this becomes actionable:

1. The scan goal is literally "LeetCode fixture failures / divergence", but the document never splits those two. A fixture that is both failing-to-compile *and* divergent needs a different plan than one that passes but is rewritten poorly.
2. Categories 2a and 2b are written as capability *lists*, not as *features* with acceptance tests mapped to specific fixtures. Ten bullets become twenty-plus issues that cannot be scheduled until they are named and sized.
3. Most Category 1 rewrites depend on at least one item from Category 2a, 2b, or 4 of the priority order — yet the ordering presents rewrites as a *last* step. In practice, individual rewrites unlock as their ergonomic prereqs land, not as a phase.

---

## 1. Practical Priority Order — Correctness Assessment

### What the order gets right

- Corpus normalization first is correct: without it the "done" signal is unreliable, and every subsequent decision is noisy.
- Putting collection/index Optional flow before recursive/cursor ergonomics is correct — 2b unblocks more fixtures and has lower ownership-model risk than 2a.
- Putting explicit rewrites last is correct in aggregate: rewriting before the ergonomics land re-creates the same friction in canonical form.
- The closing sentence ("work sequence, not a severity ranking") is the right framing but is undersold — it should be a section header, not an aside.

### Where the order is ambiguous or wrong

1. **"Zero failures/divergence" conflates two populations.** The priority order is framed as a work sequence toward a goal that is not defined. Based on the scan, there are at least three populations:
   - fixtures that fail to compile/run (not directly addressed in this doc)
   - fixtures that pass but have public-model drift (Category 1)
   - fixtures that pass but have ergonomic inflation (Categories 2a/2b)
   - a `sifr_only` set of sixteen `*_v2` files ([lines 12–29](verification/leetcode/leetcode_pair_diff_scan_20260409.json)) with no Python pair, completely unaddressed
   The priority order only serves population 2 and 3. It should either narrow its scope explicitly or add steps for populations 1 and 4.

2. **Priority 4 is a stdlib grab-bag, not an ordered wave.** `heap`, `deque`, DSU, and trie are listed bare. They have very different unblock power:
   - DSU + `heap` alone unlock the bulk of graph-shape fixtures in 2b (`0261`, `0721`, `0743`, `0778`, `1584`, `1631`, `1489`, `2092`, `2709`, plus the `0295`/`0355` rewrites). These should be wave A.
   - `deque` is narrower (BFS pattern; `0286`, `0417`, `0513`, `0752`, `0200`'s alternative form). Wave B.
   - Trie serves only `0212` and, partially, `0269` depending on the rewrite approach. Wave C — and coupled to the `0212` rewrite, not independent.
   The doc should state this wave ordering and the fixture-count justification.

3. **Rewrites are not a single step.** As soon as 2b wave A lands, `0295_find_median_from_data_stream` is unblocked and should ship. Holding all twelve rewrites behind a single terminal phase will stall both adoption testing (do the new ergonomics actually produce canonical code?) and reviewer attention. Restructure as: each rewrite is gated on its specific prereq, not on "all ergonomics work".

4. **Ownership/borrow work is implicit.** 2a's "own-annotated chain" and 2b's "owned collection helpers" both depend on the current maturity of the borrow checker and the state of Phase 10/10.1 ("borrow by default"). This prerequisite is never named. If those phases are incomplete, 2a and parts of 2b cannot start.

### Does it risk breaking Sifr principles?

No — the Boundaries To Preserve section is tight, and each of the ergonomics asks is compatible with them. One latent risk worth calling out: the "narrowing preserved across rebinding when the RHS is itself provably non-`None`" rule ([line 90](verification/leetcode/leetcode_divergence_decision_analysis_20260409.md:90)) is subtle. It can be sound, but it is the kind of rule that accretes edge cases. Recommend it be split out as a standalone design note with enumerated admissible and inadmissible cases *before* it becomes an issue.

---

## 2. Category Actionability Assessment

### Category 1 — "Rewrite mainly"

**Actionable signal:** above-cutoff vs. below-cutoff split is good; "why" list per fixture is good.

**Missing for execution:**

- No public-model old/new signature pair per fixture. Each rewrite changes the type of the public entry point — the spec should state it explicitly, e.g. for `0023`:
  `mergeKLists(list[list[int]]) -> list[int]` → `mergeKLists(list[ListNode | None]) -> ListNode | None`.
- No feature-prereq list per rewrite. Without it, scheduling the twelve rewrites is guesswork.
- No acceptance delta. The existing `main()` checks the wrong invariants (value equality on lists, not node identity on lists of lists of `ListNode`). Rewrites will need new `main()` bodies and the doc should flag this.
- `0147`, `0148`, `0206`, `0024`, `0160`, `0133`, `0138` all need a shared canonical `ListNode`/`TreeNode` helper story. Today each fixture re-defines kitchen-sink `Node`/`nodeVal`/`hasNode`/`unwrapInt` ([0002_add_two_numbers.sifr:12–70](audits/leetcode/0002_add_two_numbers.sifr)). A rewrite pass that does not address this will either duplicate the boilerplate or diverge in yet a new way. This is a Category 1 cross-cutting concern and should be lifted to its own top-level sub-item.

### Category 2a — Recursive node / cursor ergonomics

**Actionable signal:** the fixture list is the best enumeration of cursor-flavored work in the repo.

**Missing for execution:**

- Each of the six bullets under "What should improve" is a distinct language/compiler feature. Read as a package they are a multi-milestone phase. Read as a to-do list they are unorderable. Split:
  - **F-N1** narrowing after `is not None` on local bindings (no write interval).
  - **F-N2** narrowing through recursive-node field projections (`node.next` after `node.next is not None`).
  - **F-N3** narrowing preserved across rebinding of a non-`None` RHS.
  - **F-N4** narrowing preserved across repeated checks (no copy-to-local).
  - **F-C1** cursor through own-annotated chain (trailing dummy-head pattern).
  - **F-C2** in-place `.next` skip under double narrowing.
  - **F-C3** sub-range rewire/reverse on owned chains.
  - **F-R1** structural recursion over owned chains/trees with read-only reborrow.
- The final bullet ([line 93](verification/leetcode/leetcode_divergence_decision_analysis_20260409.md:93)) correctly warns that ergonomics alone do not convert the drain/rebuild fixtures. But this means each 2a fixture carries a rewrite tail that belongs in Category 1 scheduling, not 2a scheduling. The overlap should be made explicit with a separate "2a-rewrite tail" list.
- No mapping of which fixture flips when which feature lands. A minimal expectation is that `0021_merge_two_sorted_lists` (the simplest cursor pattern) flips on F-N1 + F-C1 alone. If that is not true, the features are wrong or still too large.

### Category 2b — Collection / index / stdlib ergonomics

**Actionable signal:** the fixture list is good; the stdlib sub-bullets are reasonably concrete.

**Missing for execution:**

- Three different axes are fused into one category: index/key narrowing (compiler), owned-collection helpers (language + stdlib), stdlib parity (stdlib). They have different owners and different risk profiles. Split into 2b-compiler, 2b-stdlib-core, and 2b-stdlib-algorithms.
- The four "owned collection helpers" (`drain`, `take_at`, `split_first`, `iter_mut_indexed`) are not all equally needed. Which fixture requires each? If you cannot name one per helper, some of them belong in "nice to have" and should be deferred.
- `character-class predicates such as isdigit / isalpha` and `whole-token integer parsing returning Result` are two materially independent stdlib items and should be split. They also unblock already-shipped fixtures that embed a 10-branch `digitValue` ladder (see [0297_serialize_and_deserialize_binary_tree.sifr:52–73](audits/leetcode/0297_serialize_and_deserialize_binary_tree.sifr), [0394_decode_string.sifr:17–60](audits/leetcode/0394_decode_string.sifr)) — fixing them retroactively shrinks the corpus *and* pays down real ergonomic debt. This is a high-leverage early win that the priority order buries under priority 4.
- `trie-friendly APIs` is phrased in a way that dodges the actual decision. It should either be "we ship a `Trie` type" or "we ship nested-dict construction helpers". The doc should pick one; the `0212` rewrite cost differs significantly between them.

### Category 3 / 5 — Okay as-is / corpus cleanup

**Actionable signal:** these two categories are the most ready to execute. Four fixtures, each with a specific Python-side edit named.

**Missing for execution:**

- No ordering for the cleanup passes. They should all happen in one PR so the scan re-run produces one clean delta. Recommend stating that explicitly.
- The guardrail ("do not delete alternative implementations that document intentional algorithmic variety") is not stated. Without it a well-meaning sweep could remove instructive multi-solution Python fixtures beyond the four named here.
- Cleanup should be followed by a re-scan-and-compare step. The doc does not say so, but without it Category 3 does not produce evidence that the classification was right.

### Category 4 — Architecture boundary

**Actionable signal:** clear, explicit, unambiguous.

**Missing for execution:**

- The four below-cutoff continuation fixtures ([line 166](verification/leetcode/leetcode_divergence_decision_analysis_20260409.md:166)) are correctly flagged "do not escalate", but the doc needs one more line: what action, if any, is ever appropriate for Category 4 items. If the answer is "none, document and move on", say so. Otherwise every future scan will re-debate their classification.
- `0673`'s layered Category 2b pressure ([line 161](verification/leetcode/leetcode_divergence_decision_analysis_20260409.md:161)) means the fixture appears twice in roadmap thinking but only once in the category list. Recommend a small "mixed classification" sub-list so `0673` and any siblings do not get orphaned.

---

## 3. Hidden Dependencies That Should Be Explicit

These dependencies are implied by the content but never drawn. Making them explicit changes the actual work order.

1. **Borrow-checker maturity → 2a and half of 2b.** "own-annotated chain" and "owned collection helpers" assume the borrow-by-default work is landed. If Phases 10/10.1 are not complete at the moment this plan starts, they are a hard prerequisite.
2. **Narrowing-invalidation semantics → both 2a and 2b.** Both categories depend on "narrowing invalidated by alias-mutating calls / intervening writes". That invalidation rule is one decision; if it is made twice, 2a and 2b will disagree. Lift this rule to a top-level design item *before* either category begins.
3. **Stdlib heap + DSU → four Category 1 rewrites.** `0295` (heap), `0023` (heap, if keeping the canonical k-way merge), `0212` (trie), `0707` (list-of-nodes / cursor — not stdlib but ownership work). These are not "rewrite last"; they are "rewrite as the prereq lands".
4. **Cleaned-up Python corpus → re-run the scan before starting Category 1.** Category 5 mathematically changes the diff rankings. If Category 1 starts before Category 5 completes, the priority of which rewrite to do next is wrong.
5. **Shared Sifr-side boilerplate story → all linked-list and tree rewrites.** Every linked-list fixture embeds a near-identical `ListNode` + kitchen-sink `Node` block ([0002_add_two_numbers.sifr:3–70](audits/leetcode/0002_add_two_numbers.sifr) and siblings). Until there is a convention on shared helpers (a fixtures-prelude, a module import, or an explicit "each fixture is self-contained, duplication is accepted"), each rewrite risks inventing its own.
6. **Diagnostics quality → adoption.** Flow-sensitive narrowing fails silently if the diagnostic message does not point to the invalidating write / call. This is Phase 27-scope work and must be co-sequenced with each narrowing feature, or the ergonomics win will not materialize in practice.

---

## 4. Features That Are Too Large

| Listed as one item | Should be split into |
| --- | --- |
| Flow-sensitive narrowing (2a bullets 1–4) | Four features: F-N1..F-N4 as named above, each with its own acceptance fixture |
| Cursor-style mutation patterns (2a bullet 5) | Three features: dummy-head cursor, in-place skip, sub-range rewire/reverse |
| Owned collection helpers (2b bullet 3) | Four features: `drain`, `take_at`, `split_first`, `iter_mut_indexed`, each with an example fixture |
| Collection/index Optional-flow narrowing (2b bullets 1–2) | Two features: list-index (in-bounds proof), dict-key (contains-key proof) — different proofs, different invalidation, different diagnostics |
| `isdigit`/`isalpha` + whole-token int parsing (2b bullet 4) | Two features: character-class predicates; `int` parse returning `Result` |
| Trie ergonomics (2b bullet 4) | One decision (Trie type vs. nested-dict helpers), then one feature |
| Stdlib parity (priority 4) | Three ordered waves (heap+DSU, deque, trie) |
| Category 1 "rewrites" (priority 5) | Twelve individually scheduled rewrites, each with its own prereq list |

---

## 5. Rewrite List Completeness

The rewrite list names the twelve fixtures and gives one-line "why". For direct follow-up work each entry needs more. A minimal spec template:

```
Fixture: 0023_merge_k_sorted_lists
Canonical algorithm: min-heap of list heads, O(N log k)
Public-model change:
  old: mergeKLists(list[list[int]]) -> list[int]
  new: mergeKLists(list[ListNode | None]) -> ListNode | None
Feature prereqs:
  - heap stdlib (wave A)
  - owned ListNode cursor ergonomics (F-C1)
  - F-N2 field-projection narrowing on .next
Acceptance:
  - updated main() exercises node-identity chains
  - listNodeToString(result) == "1->1->2->3->4->4->5->6"
Notes:
  - shared ListNode / helper boilerplate story must be resolved first
```

Gaps in the current list:

- **Public-model deltas missing.** Seven of the twelve rewrites change the entry-point signature. None are written down.
- **Fallback algorithm choices missing.** `0023` has two canonical solutions (heap; divide-and-conquer pairwise merge). `0295` has two (two-heap; order-statistic tree). The doc should pick one per fixture or state explicit acceptability criteria.
- **`main()` update required, not flagged.** Every rewrite changes what the asserts prove. This is not cosmetic — it is how the fixture certifies correctness after rewrite.
- **No "who verifies it" step.** Rewrites should re-run through the pair scan and any upstream failure taxonomy (Phase 31 artifacts) to confirm the fixture moved from diverged to matched.
- **`sifr_only` `*_v2` files not addressed.** Sixteen `_v2` fixtures ([scan lines 12–29](verification/leetcode/leetcode_pair_diff_scan_20260409.json:12)) have no Python pair. Are these deliberate Sifr-native alternates, orphans awaiting Python pairs, or candidates for deletion? The plan is silent. At minimum: flag them for triage in Category 5.

---

## 6. Concrete Edits to Make The Report Useful As A Plan

These are the specific edits that would convert this document from a classification into an executable plan.

1. **Add a one-paragraph Scope clarification** at the top distinguishing *failure* (does not compile/run) from *divergence* (runs but differs from canonical). State which of the two this plan addresses, and where the other is tracked.
2. **Add a Feature Ledger** section between the Categories and the Priority Order. Each feature gets an ID (F-N1, F-C1, S-heap, S-dsu, …), a one-line scope, prerequisites, at least one fixture whose divergence it is expected to remove, and a boundary citation.
3. **Add a Fixture → Feature matrix** as an appendix. One row per fixture, one column per feature, cells showing "required" vs. "opportunistic". Without this matrix the plan cannot be read as a DAG.
4. **Restructure the Priority Order** as waves with explicit exit criteria:
   - Wave 0: Corpus normalization (exit: re-scan delta matches expected LOC reductions).
   - Wave 1: Narrowing rule design + diagnostics co-design (exit: written spec).
   - Wave 2: F-N1..F-N4 + list-index narrowing + dict-key narrowing (exit: named fixtures flip).
   - Wave 3: stdlib wave A (heap + DSU) + unblocked Category 1 rewrites (`0295`).
   - Wave 4: F-C1..F-C3 + F-R1 + shared-boilerplate convention.
   - Wave 5: stdlib wave B (deque).
   - Wave 6: Remaining Category 1 rewrites, each gated on prereqs.
   - Wave 7: stdlib wave C (trie) + `0212` rewrite together.
5. **Replace "Priority 5 — Explicit parity-debt rewrites" with a per-fixture table** using the spec template above. The flat bullet list is not plannable.
6. **Split compound bullets in 2a and 2b** into numbered sub-bullets, each with "example fixture that flips".
7. **Promote "borrow-by-default maturity" and "narrowing-invalidation rule"** to named prerequisites at the top of the Priority Order. They are currently invisible.
8. **Mark the four Category 3/5 fixtures as a single PR** with explicit DoD: Python-side edits, re-scan run, and updated scan numbers committed together.
9. **Add a "Not in this plan" section**: runtime failures outside divergence, `sifr_only` `*_v2` triage, fixtures below 80 lines that were not escalated. A short explicit list beats silent omission.
10. **Capture the shared-boilerplate decision** as its own top-level sub-item under Category 1 cross-cutting, with three options (fixtures-prelude import, codegen-side helper set, accept duplication). Any of the three is fine; unstated means each rewrite re-litigates it.
11. **Link each rewrite and each feature** to the issues/phase milestone it will live under. Phase 31 is complete per [internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md:3](internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md), so this work likely needs a new phase or a clearly-scoped 31.x follow-up. Say which.
12. **Record a "done" metric**: target `changed_total_lines >= 80` outliers trending to zero *and* `similarity_ratio` distribution shifting, measured against the 2026-04-09 baseline. Without a target the plan cannot be closed.

---

## Closing Note

The classification work in this document is careful and the boundaries are right. The gap is between classification and execution: no feature IDs, no DAG, no per-rewrite spec, no exit criteria, no scope statement separating failure from divergence. Landing the twelve edits above — especially the Feature Ledger, the Fixture → Feature matrix, and the wave-based Priority Order — converts this from a decision document into something an implementation owner can turn directly into a milestone backlog.
