# Review: INT-2B — Tracker update for reserved-width shadowing policy slice (post PR #1810)

Reviewer: Claude Opus 4.7
Date: 2026-05-06
Branch: `int-2b-shadowing-policy-tracker`
Phase: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), milestone INT-2B
Slice this tracks: PR #1810, [Document reserved width shadowing policy](https://github.com/sifr-lang/sifr/pull/1810) (merged 2026-05-06)
Review artifact for that slice: [reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md](reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md)
Local validation reference: `scripts/run_all_tests.sh --profile quick`, `report_signature=e1bf653aaa770517`, `wall_time=55.23s`

## Verdict: SATISFIED — ready to merge

The uncommitted edit to `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` is a docs-only tracker update that records the merged shadowing-policy slice. The review-history entry, the new ticked checklist sub-item, the PR number, the review artifact path, and the trimmed remaining follow-up bullet are all internally consistent, consistent with the merged review artifact, and consistent with the pattern used by neighboring INT-2B sub-items. No blockers.

---

## What changed

`git diff issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` shows two hunks, both inside the INT-2B section:

1. **Review history**, after line 414, inserts:
   ```
   - [x] INT-2B reserved-width shadowing policy documentation review satisfied: `reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md`.
   ```
2. **Implementation Checklist**, splits the previously combined "Carry remaining follow-ups" bullet into:
   - a new ticked sub-item recording the merged slice and PR #1810, and
   - a trimmed open bullet that retains only the still-pending fixed-width diagnostic formatting/fallback follow-up.

`git status` confirms only this one file is modified. No code, fixture, registry, schema, generated-doc, design-doc, or unrelated tracker edits in this slice.

---

## Correctness of each tracker field

### 1. Review-history entry ([line 415](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:415))

> `- [x] INT-2B reserved-width shadowing policy documentation review satisfied: reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md.`

- The path resolves: `ls reviews/ | grep shadowing` returns the file, and the artifact's stated verdict is "SATISFIED — ready to merge" at [line 10](reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md:10), so "review satisfied" is faithful.
- Placement is chronologically correct: the four immediately preceding INT-2B history entries reference PRs #1804, #1806, #1808, and the new entry sits before any future #1810+ history. The pattern of "INT-2B <topic> review satisfied: reviews/..." matches the surrounding entries (e.g. lines 412–414).
- Topic phrasing ("reserved-width shadowing policy documentation") matches the title and scope of the merged review artifact.

### 2. New checklist sub-item ([line 440](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:440))

> `- [x] Reserved int128/uint128 diagnostics are documented as applying after ordinary annotation name resolution, preserving user-defined type variable, alias, and class shadowing until a future language-wide reserved-identifier policy; review is satisfied and quick validation is passing: PR #1810.`

- Tense and structure mirror neighbors (e.g. lines 437, 438, 439): present-tense statement of what the slice landed, then "review is satisfied and quick validation is passing: PR #####." This is the project pattern.
- Substantive accuracy:
  - "applying after ordinary annotation name resolution" matches the precedence story documented in the integer model and verified by the merged review artifact ([review §"Correctness against current implementation"](reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md:30)) — the type-var, alias, and class branches in `resolve_annotation_expr` all run before the reserved-width diagnostic.
  - "type variable, alias, and class" enumerates exactly the three shadowing categories the doc paragraph names.
  - "until a future language-wide reserved-identifier policy" is the same forward-looking caveat the doc paragraph uses; it does not commit the project to a specific design and matches the policy text.
- PR reference: `gh pr view 1810 --json` confirms PR #1810 with title "Document reserved width shadowing policy" is **merged** at 2026-05-06T12:40:39Z, so "PR #1810" is correct and accurately attributed.
- "Quick validation is passing" matches the supplied local-validation reference (`report_signature=e1bf653aaa770517`, `wall_time=55.23s`, profile `quick`). Consistent with the merged review's validation-gate reasoning that a paragraph-only doc edit lands outside the workspace surfaces but is still gated by `scripts/run_all_tests.sh --profile quick` per `AGENTS.md`.

### 3. Trimmed remaining-follow-ups bullet ([line 441](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:441))

Previous wording (one line, two follow-ups):

> Carry remaining follow-ups from INT-2A/INT-2B reviews: decide reserved-name shadowing policy during `bigint` cleanup and clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable.

New wording (one follow-up):

> Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable.

- The removed clause ("decide reserved-name shadowing policy during `bigint` cleanup") is exactly the obligation that the new ticked sub-item at line 440 closes. Removing it from the open bullet is correct.
- The retained clause ("clean up fixed-width diagnostic formatting/fallback paths as those code paths become reachable") is genuinely still open — it was not the subject of PR #1810 and is unrelated to the shadowing-policy decision. Keeping it on the open bullet is correct.
- This split exactly implements option (b) from the merged review's non-blocking observation O1 ("split line 439 into two sub-items so this slice can mark its own done"). The merged review explicitly flagged this as the cleaner of the two acceptable approaches.

### 4. Internal consistency between the three edits

- Review-history line 415 names "reserved-width shadowing policy documentation review"; checklist line 440 says "Reserved int128/uint128 diagnostics are documented…"; both describe the same docs-only paragraph in `internal_docs/integer_model.md`. Topics align without overstating scope (no claim of new code, tests, or registry coverage).
- The umbrella checkbox at line 428 ("INT-2B HIR, type system, and const fitting") is correctly still unticked because line 441 still has an open child. This matches the project's "umbrella stays unticked while any child is open" pattern.
- The sub-item ordering (#1804 → #1806 → #1808 → #1810) preserves PR-number ascending order under [line 428](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:428), consistent with how prior INT-2B sub-items were appended.

---

## Cross-checks with the merged slice

- The merged review's non-blocking observation **O1** anticipated this exact tracker shape and recommended either (a) ticking the umbrella line only when both umbrella sub-items land or (b) splitting it. Option (b) was chosen and executed cleanly.
- The merged review's non-blocking observation **O3** (no regression test for the shadow case) is intentionally **not** addressed in this tracker update — that is correct, because adding a regression test belongs to a code-side slice, not to this docs-only slice's tracker.
- The merged review's non-blocking observations **O4** ("no anchor for the future language-wide reserved-identifier policy") and **O5** ("`bigint` parallel not asserted") are also intentionally not surfaced here. Neither is in scope for this tracker update; both are for a future consolidation slice.
- PR #1810's title ("Document reserved width shadowing policy") matches the policy decision recorded in line 440, and the merged-at timestamp (2026-05-06) is on or before today (2026-05-06), so the past-tense framing in the review-history entry is justified.

---

## Scope discipline

- One file modified, two hunks, no collateral edits.
- No retroactive edits to prior INT-2A/INT-2B history rows or sub-items, which keeps the historical record stable.
- No edits to `internal_docs/integer_model.md`, `internal_docs/architecture.md`, or `internal_docs/roadmap.md` — appropriate for a tracker-only follow-up after a docs-only slice already landed.
- The umbrella line that PR #1810's review (in O1) flagged as needing eventual splitting is split here, but the split is conservative: only the resolved clause moves to a ticked sub-item, the remaining clause keeps its original wording.

---

## Validation gates

This is a tracker-only edit inside `issues/`. The path is not scanned by the Rust workspace, the diagnostic-doc generator, the schema/registry coverage scripts, or the HIR maintainability guardrail (which targets `crates/sifr_hir/src/lower/`). The supplied `report_signature=e1bf653aaa770517` from `scripts/run_all_tests.sh --profile quick` already exercised the merge state of PR #1810 at the code level; the tracker update itself does not change any code-reachable surface, so a re-run is not load-bearing for correctness. Per `AGENTS.md`, the user should still run `scripts/run_all_tests.sh --profile quick` before opening the PR for this slice.

---

## Non-blocking observations (not required for merge)

These are flagged for awareness; none warrant blocking.

- **O1 — Plurality of "Carry remaining follow-ups…".** With only one follow-up remaining, the prefix "Carry remaining follow-ups from INT-2A/INT-2B reviews:" is technically still accurate (it describes the *origin* of the follow-up, not a count) but reads as plural while listing a single item. A purely cosmetic tweak would be to rephrase as "Carry remaining INT-2A/INT-2B follow-up:" (singular). Not in scope; the current wording does not introduce any incorrect claim.
- **O2 — `bigint` parallel still implicit.** The new sub-item is scoped to `int128`/`uint128`, but the same precedence rule in `resolve_annotation_expr` applies to `bigint` (the merged review's O5 noted this). When a future slice consolidates the language-wide reserved-identifier policy, the tracker will likely want a parallel sub-item for `bigint`. Not in scope here.
- **O3 — No anchor in the tracker for the future "language-wide reserved-identifier policy."** The trailing clause references a future policy without an INT-3+/issue-link anchor. This mirrors the same forward-looking phrasing in the design-doc paragraph that PR #1810 added, so propagating the same level of indirection is consistent. A separate tracking line could be added once such a slice is opened. Not blocking.

---

## Final verdict

**SATISFIED — ready to merge.** The tracker update accurately reflects the merged PR #1810 slice. The review-history entry resolves to the correct review artifact, the new checklist sub-item faithfully summarizes the recorded policy and cites the right PR number, and the trimmed remaining-follow-up bullet correctly excises only the resolved clause while preserving the unrelated open clause. The split implements option (b) from the merged review's O1 cleanly. Non-blocking observations O1–O3 are noted for future slices and do not block this docs-only tracker edit.
