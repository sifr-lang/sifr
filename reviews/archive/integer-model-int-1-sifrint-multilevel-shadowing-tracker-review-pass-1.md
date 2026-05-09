---
name: INT-1 SifrInt multilevel shadowing tracker review pass 1
description: Tracker-only review of PR #1848 — confirms it truthfully records the PR #1847 multi-level nested shadowing artifact, marks multi-level nested helper lexical shadowing complete, and narrows the residual INT-1 gap to multi-level forced-local capture propagation plus exact-int `//` and `%` support.
type: review
---

# Review: INT-1 SifrInt Multilevel Shadowing Tracker Pass 1

PR: [sifr#1848](https://github.com/sifr-lang/sifr/pull/1848)
Branch: `int-1-sifrint-multilevel-shadowing-tracker`
Commit: `5abf55d3`

## Verdict

**Satisfied.**

The diff against `main` is a 4-line edit confined to [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). It adds a Validation entry for the PR #1847 review artifact, ticks the implementation checklist item attributed to PR #1847 with an accurate one-line summary of the three covered helper shapes (non-recursive multi-level local, recursive multi-level local, parameter-shadow + multi-level nested), and narrows the still-open residual to "multi-level nested helper capture of outer locals already forced to `SifrInt` … and unsupported augmented assignment/fallible `//` and `%`." Each claim is supported by the underlying artifact and the merged PR #1847.

## Findings

No blocking findings.

### 1. Recorded review artifact exists and matches the tracker claim

Tracker line 414 in [ad-hoc-integer-model-and-fixed-width-numeric-contract.md:414](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:414) points at [`reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md`](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md). That file exists with verdict "Satisfied" at [line 9](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:9) and a single non-blocking N1 note about forced-locals capture asymmetry at [line 127](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:127). The Validation-section phrasing "satisfied with non-blocking forced-local capture follow-up" is an accurate summary of that artifact — N1's title is verbatim "Asymmetry: forced-locals capture is still non-transitive", and its recommendation at [line 147](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:147) is to make `collect_sifr_int_captured_forced_locals` transitive in a follow-up. ✓

### 2. Multi-level nested helper completion claim is correct

Checklist line 460 in [ad-hoc-integer-model-and-fixed-width-numeric-contract.md:460](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:460) credits PR #1847 with preserving "helper-inside-helper local, recursive, and parameter-shadow shapes." This maps cleanly onto the artifact's verified probe matrix at [integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:171-173](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:171):

- "Multi-level non-recursive nested local shadow" → "helper-inside-helper local"
- "Multi-level recursive nested local shadow" → "recursive"
- "Multi-level non-recursive nested param shadow" → "parameter-shadow"

PR #1847 is merged on `main` (`a9aabaca0 Respect multilevel shadows for SifrInt module constants`), so attributing the slice to that PR is correct. The phrase "across nested return analysis, closure body rewriting, and recursive hidden capture parameters" matches the artifact's mechanism description at [lines 38-41](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:38), where both the analysis pipeline (`collect_nested_sifr_int_function_returns`) and the codegen pipeline (`try_lower_structured_nested_function_stmt`) are routed through the new transitive helper. The closing clause "while unshadowed module constants still lower through `SifrInt`" mirrors the artifact's control-case verification at [lines 86-99](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:86). ✓

### 3. Residual gap is accurate and narrowed correctly

The pre-PR residual at line 459 of the prior file read "multi-level nested helper lexical shadowing for outer locals that shadow exact-int module constants still needs scope-safe coverage, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support." This PR rewrites it (now line 461) to "multi-level nested helper capture of outer locals already forced to `SifrInt` still needs transitive capture propagation, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support."

The first clause is meaningfully narrower:

- The pre-PR clause referred to *module constants* shadowed by outer scopes. PR #1847 closed that case.
- The post-PR clause refers to *outer locals already forced to `SifrInt`* — a different shape that is still open. This is the exact gap the artifact's N1 calls out at [lines 131-149](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:131), with a constructed counterexample that traces the failure mode through `collect_sifr_int_captured_forced_locals` (non-transitive) versus the now-transitive `collect_sifr_int_captured_shadowed_module_bindings`. The artifact's carry-forward at [line 185](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:185) phrases it as "Multi-level forced-local capture propagation — make `collect_sifr_int_captured_forced_locals` transitive"; the tracker's "transitive capture propagation" is a faithful one-line condensation. ✓

The preserved second clause (augmented assignment / fallible `//` and `%`) is unchanged and still consistent with the artifact's carry-forward second item at [line 186](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:186). ✓

### 4. Validation notes are adequate for a tracker-only PR

The diff is doc-only (1 file, 3 insertions, 1 deletion, no code paths touched). `git diff --check` plus `scripts/run_all_tests.sh --profile quick` with `report_signature=e1bf653aaa770517` is the right amount of evidence. That signature matches the one reported across PRs #1817–#1845 per the underlying artifact's N4 at [line 162](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:162), confirming no test deltas — neither new failures nor new flakes introduced by this tracker slice. The lower wall time (72.18s vs the prior tracker's 112.87s) is expected; signatures track outcomes, not wall time. ✓

### 5. Diff scope is clean

`git diff main...HEAD --stat` shows `1 file changed, 3 insertions(+), 1 deletion(-)`, all inside [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). No unrelated edits, no review-file additions (the underlying review artifact `reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md` was already merged separately and is present on `main`), no code, no fixtures. The single commit on the branch is `5abf55d3 Track SifrInt multilevel shadowing` with no extraneous history. ✓

## Notes

(Non-blocking observations only.)

### N1 — Tracker preserves both follow-up clauses on a single bullet

The remaining checkbox at line 461 keeps both follow-up clauses (multi-level forced-local capture propagation + augmented assignment/fallible `//`/`%`) on a single bullet. That mirrors the artifact's "Carry-forward open INT-1 items" framing at [lines 183-186](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:183), so the next slice can pick either one without re-splitting the line. No action required.

### N2 — Validation-section line is consistent with prior shadowing entries

The newly added line 414 follows the exact phrasing pattern used by lines 412 (immediate lexical → "non-blocking nested-scope shadowing follow-up") and 413 (single-level nested → "non-blocking multi-level nesting follow-up"). The progression "immediate → single-level nested → multi-level nested" stays easy to skim, and the "non-blocking … follow-up" suffix consistently flags that each prior slice deferred a strictly downstream concern. Stylistically consistent with the rest of the Validation section.

### N3 — Checklist line is consistent with the prior single-level nested entry

Checklist line 460 ("Multi-level nested helpers now preserve outer locals and parameters …") mirrors the structure of line 459 ("Single-level nested helpers now preserve outer locals and parameters …") almost word-for-word, swapping the helper-shape qualifiers ("non-recursive, recursive, and parameter-shadow" → "helper-inside-helper local, recursive, and parameter-shadow"). The continuity makes the trajectory of the slice obvious to a future reader. ✓

### N4 — Review-file naming follows the established tracker-review pattern

This review file is named `integer-model-int-1-sifrint-multilevel-shadowing-tracker-review-pass-1.md` to match the prior tracker reviews (`integer-model-int-1-sifrint-lexical-shadowing-tracker-review-pass-1.md`, `integer-model-int-1-sifrint-nested-shadowing-tracker-review-pass-1b.md`). The underlying implementation review (`...-multilevel-shadowing-review-pass-1.md`) is distinguished from this one by the inserted `tracker-` segment, which is the convention used elsewhere in the directory. The tracker artifact itself does not need to be cross-listed in the Validation section of the issue file (prior tracker reviews are similarly not listed there).

### N5 — INT-1 closure trajectory

With this tracker entry, the open INT-1 surface narrows to two items on a single bullet: (1) multi-level forced-local capture propagation (the artifact's N1 follow-up — a one-line analysis swap symmetric to PR #1847's, plus a `returned_big_from_local_multilevel_nested_helper` fixture), and (2) the long-standing unsupported augmented assignment / fallible `//` and `%` runtime/codegen support. Per the artifact's closing observation at [line 188](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:188), "INT-1 closure remains very close after this slice." The tracker now reflects that state truthfully.

## Probe matrix

| Probe | Result |
|-------|--------|
| Validation-section line 414 points at an existing review artifact with verdict "Satisfied" | ✓ |
| Phrase "non-blocking forced-local capture follow-up" matches artifact N1 (`Asymmetry: forced-locals capture is still non-transitive`) | ✓ |
| Checklist line 460 attributes the slice to PR #1847; PR #1847 is merged on `main` (`a9aabaca0`) | ✓ |
| Three covered shapes ("helper-inside-helper local, recursive, parameter-shadow") map onto artifact probe matrix rows 171–173 | ✓ |
| Updated residual at line 461 narrows from "multi-level nested helper lexical shadowing" → "multi-level nested helper capture of outer locals already forced to `SifrInt`" | ✓ |
| Updated residual matches artifact N1 + carry-forward open item 1 | ✓ |
| Augmented assignment / `//` / `%` clause preserved verbatim | ✓ |
| Diff is doc-only, scoped to one file, 3 insertions / 1 deletion | ✓ |
| `report_signature=e1bf653aaa770517` matches the established quick-profile signature for this branch lineage | ✓ |
| Single branch commit `5abf55d3 Track SifrInt multilevel shadowing` with no unrelated history | ✓ |
