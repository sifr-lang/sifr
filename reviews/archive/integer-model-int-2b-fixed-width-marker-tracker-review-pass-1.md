# INT-2B fixed-width fail fixture marker tracker — review pass 1

Branch: `int-2b-fixed-width-marker-tracker`
Scope: docs-only tracker update in
[`issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
recording the merge of PR #1812 (the canonical `expect-error` marker cleanup
for `crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr`).

## Change under review

`git status --short` shows exactly one path modified:

```
 M issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md
```

`git diff` reports two added lines and zero removals — one in the **Review
History** section and one in the **Implementation Checklist** under the
"INT-2B HIR, type system, and const fitting" parent. No other tracker bullets,
parent boxes, or section ordering shift, so the change is strictly additive
and matches a docs-only slice.

## Review-history entry (line 416)

```
- [x] INT-2B fixed-width fail fixture marker cleanup review satisfied:
  `reviews/integer-model-int-2b-fixed-width-fail-fixture-markers-review-pass-1.md`.
```

Verified:

- The artifact exists on disk at the cited path.
- The artifact's heading ("INT-2B fixed-width fail fixture markers — review
  pass 1") and stated branch (`int-2b-fixed-width-fail-fixture-markers`)
  match the slice that became PR #1812.
- The bullet is checked (`[x]`) and follows the established phrasing template
  used by the immediately prior entries (e.g. "INT-2B reserved-width
  shadowing policy documentation review satisfied: …" at line 415).
- Insertion position is chronologically correct: appended at the end of the
  history list, immediately after the prior PR #1810 review entry.

No discrepancy.

## Implementation-checklist entry (line 442)

```
- [x] Fixed-width const-expression fail fixture markers are canonical
  top-level `expect-error` entries, so the e2e fail harness now enforces
  `SIFR-INT-0001` and `SIFR-INT-0004` columns; review is satisfied and quick
  validation is passing: PR #1812.
```

Verified:

- **PR number**: `gh pr view 1812 --json title,number,url,mergedAt` returns
  title "Enforce fixed width fail fixture markers", merged at
  `2026-05-06T12:53:16Z`, matching today's date and the merged commit
  `1541b5b8 Enforce fixed width fail fixture markers` on the current branch.
- **Slice description accuracy**: the review artifact (lines 11–20) records
  the diff that promotes the two indented `# expect-error[col=…]` markers to
  canonical top-level form, and explains (lines 25–38) that
  `parse_expect_error_line` strips only the literal `# expect-error[` /
  `# expect-error:` prefixes — so the indented form silently bound zero
  expectations and the harness was only asserting "compilation fails" rather
  than enforcing the specific `SIFR-INT-0001` and `SIFR-INT-0004` columns.
  The tracker bullet's wording ("the e2e fail harness now enforces
  `SIFR-INT-0001` and `SIFR-INT-0004` columns") faithfully describes that
  behavior change and the diagnostic codes involved.
- **Validation wording**: "review is satisfied and quick validation is
  passing" matches the standard template used for sibling INT-2B entries
  (lines 430–441). The local validation evidence supplied for PR #1812
  (`cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` covering 265
  fail tests, plus `scripts/run_all_tests.sh --profile quick` with
  `report_signature=e1bf653aaa770517` at `wall_time=55.70s`) corresponds to
  the AGENTS.md gate, so the wording is supportable.
- **Insertion position**: the new bullet sits between PR #1810 (line 441) and
  the still-open broader follow-up (line 443), preserving slice ordering.
- **Marker state**: `[x]`, consistent with the merged status.

No discrepancy.

## Open follow-up preserved (line 443)

```
- [ ] Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up
  fixed-width diagnostic formatting/fallback paths as those code paths
  become reachable.
```

This bullet is unchanged by the diff and remains unchecked. That is the
correct outcome: PR #1812 only repositions two `expect-error` markers in a
single fail fixture; it does not touch the fixed-width diagnostic formatting
or fallback code paths in `sifr_hir`/`sifr_codegen`. The broader cleanup
remains genuinely open. The parent "INT-2B HIR, type system, and const
fitting" box (line 429) is also still `[ ]`, consistent with that
outstanding follow-up.

## Scope discipline

- Only the tracker file is modified; no source, fixture, snapshot, or
  doc-elsewhere churn slipped into this branch.
- No unrelated entries in the review history or checklist were edited or
  reordered.
- No state was flipped on parent boxes that should remain open.

## Verdict

SATISFIED — the tracker update accurately records PR #1812: the review-
history entry points at the existing artifact, the checklist bullet captures
the slice's behavioral effect (harness now enforces the `SIFR-INT-0001` and
`SIFR-INT-0004` columns) with the correct PR number and standard validation
wording, and the broader fixed-width diagnostic/fallback cleanup remains
correctly open. No blockers.
