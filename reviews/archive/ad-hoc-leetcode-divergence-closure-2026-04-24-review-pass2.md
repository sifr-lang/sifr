# Pass-2 Review: Ad-hoc LeetCode Divergence Closure Phase (2026-04-24)

Reviewer date: 2026-04-24
Review angle: blocker re-check after pass-1 feedback
Phase under review: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`
Cross-checked against:
- `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`
- `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
- `audits/leetcode/` (fixture content spot-checks)
- `reviews/ad-hoc-leetcode-divergence-closure-2026-04-24-review-pass1.md`

## Summary

All five pass-1 blocking edits are resolved. Ten of the eleven non-blocking edits are fully incorporated; N2 (testable vs review-only acceptance in WS4) is partially addressed (only the two rows with design prereqs carry `behavioral:` / `structural:` labels) but pass-1 marked it non-blocking, so it does not gate readiness. Two new material ambiguities were introduced by the fixes — both minor, neither rises to a blocker, but both are worth cleaning up before execution because they sit directly on the Ready-To-Implement First PRs list.

## Readiness Verdict

**Ready.** No remaining blockers. Two concrete implementation-readiness gaps (R1, R2 below) are recommended edits; a developer can start with the first three PRs in parallel without hitting either.

## Pass-1 Blocker Re-check

### B1. `O1` scope enumeration — RESOLVED

- WS2 feature line now enumerates `drain`, `take_at`, `split_first`, `iter_mut_indexed` at lines 140 and 158-162 (`O1 minimum deliverables` subsection).
- WS4 table now names specific `O1` helpers per fixture:
  - `0146_lru_cache`: `take_at` only if array-backed transition remains temporary (line 229)
  - `0706_design_hashmap`: `take_at` or `split_first` only if chosen design needs them (line 237)
- Matches analysis `O1` definition at `leetcode_divergence_decision_analysis_20260409.md:133`.

### B2. `0146` / `0706` phantom design prereqs — RESOLVED

- New "Design prerequisites" subsection at WS4 (lines 240-243) defines `0146_recency_structure_design` and `0706_hashmap_storage_design` as concrete deliverables.
- Ready-To-Implement First PRs #6 and #7 (lines 358-364) produce these artifacts.
- Design-note location is "rewrite PR or a preceding PR" — aligns with pass-1 recommendation B2 option A.
- Minor note: pass-1 suggested filename `0706_bucket_design.md`; phase renamed to `0706_hashmap_storage_design` to cover bucket-chaining or open-addressing. Rename is an improvement, not a regression.

### B3. HIR maintainability guardrail — RESOLVED

- `python3 scripts/check_hir_maintainability_guardrails.py` added to "Required Validation Per PR" at line 321 with scope note "for compiler/HIR changes".

### B4. WS0 scan-regeneration command — RESOLVED

- WS0 step 4 (line 86) now names the canonical invocation: `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json`.
- Verified against `scripts/scan_leetcode_pair_diffs.py:34-42`: `--output` flag exists and accepts a path. Command is correct.

### B5. `WS3_B1` pilot fixture naming — RESOLVED (but see R1)

- Ready-To-Implement First PRs #5 (lines 353-356) now names `0206_reverse_linked_list` as the pilot and scopes it to "excludes new cursor features not yet landed".
- The name is present per pass-1 ask. See R1 below for a follow-on concern about pilot viability.

## Pass-1 Non-blocking Re-check

| ID | Ask | Status |
| --- | --- | --- |
| N1 | One-line out-of-scope note for 16 `_v2` sifr-only fixtures | Resolved — Non-goals line 62 |
| N2 | Split WS4 acceptance into behavioral vs structural | Partial — only 0146 and 0706 rows carry the `behavioral:`/`structural:` prefix; other 11 rows still mix them |
| N3 | Annotate `0516` and `0673` as layered 2b | Resolved — "Layered-pressure note" lines 50-52 |
| N4 | Non-LeetCode regression as WS1 exit bullet | Resolved — line 131 |
| N5 | Cat 4a pattern-continuity fixtures under WS5 | Resolved — "Pattern continuity" lines 267-272 |
| N6 | `S1` and `S2` parallelizable | Resolved — line 153 |
| N7 | Annotate WS3 cursor ledger delta vs analysis | Resolved — "Intentional ledger delta" lines 209-212 |
| N8 | Roadmap/architecture/phases in Required Artifacts | Resolved — line 333 |
| N9 | Baseline snapshot layered-category footnote | Resolved via N3 |
| N10 | WS0 `changed_sifr_lines` scope | Resolved — exit criterion line 92 scopes Sifr-side cleanup out |
| N11 | `WS2_Sx` candidate crate list | Resolved — PRs #3 and #4 list `crates/sifr_hir`, `crates/sifr_codegen`, `crates/sifr_driver`, runtime shims |

## New Ambiguities Introduced By The Fixes

### R1. `WS3_B1` pilot on `0206_reverse_linked_list` has nothing to migrate

Concrete finding: the current Sifr fixture at `audits/leetcode/0206_reverse_linked_list.sifr` uses `def reverseList(values: list[int]) -> list[int]` and contains no `ListNode` type, no `ListNode` helpers, and no tree helpers. A `B1` helper-convention pilot exists to move shared `ListNode` / `TreeNode` helpers under the chosen convention (fixture prelude, generated helper module, or templated duplication). If the pilot fixture has no such helpers to begin with, the pilot either does no work or silently expands into the WS4 `0206` canonical rewrite — which requires `C1` and `N2`, neither landed when `WS3_B1` runs.

This contradicts the explicit scope at line 356: "the pilot excludes new cursor features not yet landed".

Impact: the `WS3_B1` PR either (a) ships a no-op pilot that doesn't validate the convention, (b) introduces `ListNode` into `0206` for the first time (i.e., begins the WS4 rewrite) which is explicitly excluded, or (c) stalls on scope clarification.

Recommended edit (pick one):

- **Option A — switch the pilot to a fixture that already carries `ListNode` helpers today.** `0021_merge_two_sorted_lists` is a natural choice (pass-1 named it as an alternative), because the existing Sifr version uses the node model and has helpers to migrate. Update line 355 to `Pilot fixture: 0021_merge_two_sorted_lists`.
- **Option B — keep `0206` but scope the pilot explicitly.** Replace line 356 with: "the pilot introduces `ListNode` and the chosen helper pattern only; it does not perform the canonical in-place reversal, which remains in WS4 until `C1` and `N2` land." Under this option, WS4's `0206` row must be updated to note `B1` has already landed its helper scaffolding and the WS4 PR only completes the cursor rewrite.

### R2. `0146_lru_cache` `take_at` prerequisite is self-contradicting

The WS4 row at line 229 lists `take_at only if array-backed transition remains temporary` as a prereq, but the same row's structural acceptance (line 229) reads `no linear scans or array shifts in final get / put; eviction is O(1)`. No canonical `O(1)` LRU design uses `take_at`; the canonical design is a hashmap plus an intrusive doubly-linked recency list where every mutation is `Θ(1)` node relink. `take_at` on an array-backed structure is itself `O(n)`.

Impact: a developer reading the prereq could reasonably conclude that shipping an interim array-backed LRU with `take_at` is acceptable, which directly contradicts the acceptance criterion. Worse, if `0146_recency_structure_design` picks the canonical doubly-linked structure, `take_at` should never have been listed.

Recommended edit: drop `take_at` from the `0146` prereq column. The new row reads:

```
| `0146_lru_cache` | `O(1)` LRU cache using hashmap plus explicit recency structure | `I2`, `0146_recency_structure_design` | behavioral: LeetCode sample sequence passes; structural: no linear scans or array shifts in final `get` / `put`; eviction is `O(1)` |
```

`0706_design_hashmap`'s analogous conditional ("`take_at` or `split_first` only if chosen design needs them") is genuinely conditional on bucket-chaining vs open-addressing choice and does **not** need the same fix.

## Validation Gates And First PRs

Given R1 and R2 are not blockers:

- Validation gates (lines 315-323) are sufficient for closure. The addition of HIR guardrails and the hard-coded scan command close the two pass-1 gaps.
- Ready-To-Implement First PRs #1 (`WS0_corpus_noise_normalization`), #2 (`WS1_D0_narrowing_invalidation_design`), #3 (`WS2_S1_heap_stdlib`), #4 (`WS2_S2_dsu_helper`), #6 (`WS4_0146_recency_structure_design`), and #7 (`WS4_0706_hashmap_storage_design`) are specific enough for a developer to start cold.
- PR #5 (`WS3_B1_fixture_helper_convention`) is the only entry that carries a remaining specificity gap (R1). It should be edited or deferred until R1 is resolved; the other six first PRs can land in parallel independent of R1.

## Fixture Classification And Safety Spot-Check

No regressions vs pass-1 spot-check:
- Category totals still reconcile: `13 + 19 + 21 + 4 + 6 = 63` (lines 27-32).
- Below-cutoff inclusions unchanged; all ten entries (lines 38-47) still match the analysis's explicit parity-debt list plus the two Cat 4b fixtures scanning below 80 lines.
- Non-goals (lines 56-62) still mirror every boundary from analysis "Boundaries To Preserve" including the new non-goal against triaging `_v2` fixtures in this phase.
- `0206`, `0707`, `0148`, and the `0894` boundary-adjacent reads delta now explicitly annotated at lines 209-212.

## Summary Of Recommended Edits

Concrete edits for remaining ambiguity (both minor, neither blocking):

1. **R1** — either switch `WS3_B1` pilot to `0021_merge_two_sorted_lists` at line 355, or tighten the `0206` pilot scope at line 356 to explicitly state "introduce `ListNode` and chosen helper pattern only; cursor rewrite deferred to WS4".
2. **R2** — remove `take_at only if array-backed transition remains temporary` from the `0146_lru_cache` prereq column at line 229. The design note and acceptance criteria already gate the canonical design correctly; the `take_at` note only creates a false interim path.

No other edits needed. The phase is implementation-ready.
