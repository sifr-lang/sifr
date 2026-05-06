# Review: INT-1 SifrInt Multilevel Forced Captures Tracker Pass 1

PR: [sifr#1850](https://github.com/sifr-lang/sifr/pull/1850)
Branch: `int-1-sifrint-multilevel-forced-captures-tracker`
Commit: `4451d6e2`

## Verdict

**Satisfied.** Doc-only tracker update; three lines added/modified in [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). Each edit is accurate against the underlying review artifact, the merged implementation PR, and the review's "Carry-forward open INT-1 items" list. No blocking findings.

## Findings

No blocking findings.

### 1. Review-history line 415 is correctly placed and accurately worded

The new entry:

> - [x] INT-1 multi-level forced-local capture review satisfied with non-blocking chained-forcing follow-up: `reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md`.

- **Filename** matches the on-disk artifact ([reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md)). ✓
- **Verdict** matches the source review: line 9 of the review reads `**Satisfied.**` ✓
- **Follow-up framing** ("non-blocking chained-forcing follow-up") matches the review's N1 finding at [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:179-219](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:179) — `register_sifr_int_forced_local_bindings` calling unseeded `collect_sifr_int_forced_locals`, with the recommendation to route through a seeded variant. ✓
- **Sequencing** (line 415, after the multi-level shadowing review at line 414) follows the pre-existing chronological pattern of INT-1 review entries (immediate → single-level nested → multi-level shadow → multi-level forced capture). ✓

### 2. Implementation checklist line 462 accurately summarizes PR #1849

The new entry:

> - [x] Multi-level nested helpers now propagate outer locals already forced to `SifrInt` transitively through helper-inside-helper return analysis, closure body lowering, and recursive hidden capture parameters, preserving both non-recursive and recursive local-source forced capture shapes; review is satisfied and quick validation is passing: PR #1849.

Cross-checked against the review and the merged PR:

| Claim | Source of truth | Status |
|-------|-----------------|--------|
| "helper-inside-helper return analysis" | `collect_sifr_int_captured_forced_locals` routed through `collect_captured_outer_names_transitively` ([review §1, lines 21-23](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md)) | ✓ |
| "closure body lowering" | Second extend at [function_emitter.rs:387-392](crates/sifr_codegen/src/function_emitter.rs:387) re-seeding `sifr_int_forced_local_bindings` for the next nested level ([review §1, lines 25-41](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md)) | ✓ |
| "recursive hidden capture parameters" | [Review §7, lines 154-163](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md) — `recursive_capture_lowers_to_sifr_int` checks `is_registered_sifr_int_local("big")`, lowering to `fn inner(remaining: i64, big: SifrInt) -> SifrInt` | ✓ |
| "non-recursive and recursive local-source forced capture shapes" | New e2e fixtures `returned_big_from_local_multilevel_nested_helper` and `returned_big_from_local_multilevel_recursive_nested_helper` ([review §3, probe matrix](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md)) | ✓ |
| "review is satisfied" | Review verdict line 9 | ✓ |
| "quick validation is passing" | Review N4 + this PR's `report_signature=e1bf653aaa770517` matches the prior chain | ✓ |
| "PR #1849" | `gh pr view 1849`: state=`MERGED`, mergedAt=`2026-05-06T22:54:06Z`, title=`Propagate multilevel SifrInt forced captures` | ✓ |

The wording mirrors the prior multi-level shadowing entry (line 461) exactly in structure — same "across A, B, and C, preserving X and Y shapes" template — which is the right shape for tracker symmetry given that this PR is the forced-capture analog of the shadowing PR.

### 3. Narrowed residual at line 463 maps precisely to the review's open items

The diff replaces:

> Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: multi-level nested helper capture of outer locals already forced to `SifrInt` still needs transitive capture propagation, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support.

with:

> Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: multi-level nested helpers with locals derived from captured forced `SifrInt` parents still need seeded chained-forcing in codegen, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support.

Cross-checked clause-by-clause:

- **Removed clause** ("multi-level nested helper capture of outer locals already forced to `SifrInt` still needs transitive capture propagation") — closed by PR #1849 per [review §1, §3, §6, §7](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md). Removal is justified.
- **Added clause** ("multi-level nested helpers with locals derived from captured forced `SifrInt` parents still need seeded chained-forcing in codegen") — maps to review N1 at [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:179-219](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md) and the carry-forward bullet 1 at [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:262](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md). The shape — `derived: int = big + 1` inside an inner where `big` is captured-forced — is exactly "locals derived from captured forced `SifrInt` parents". The fix shape — make `register_sifr_int_forced_local_bindings` seed `collect_sifr_int_forced_locals` with the existing forced set — is exactly "seeded chained-forcing in codegen". ✓
- **Preserved clause** ("unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support") — verbatim, matches the review's carry-forward bullet 2 at [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:263](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md) and the prior tracker wording. ✓

### 4. Top-level INT-1 checkbox correctly remains open

Line 443 (`- [ ] INT-1 runtime SifrInt and ownership semantics`) stays unchecked. Correct — the residual sub-bullet at line 463 is still open, and the review's carry-forward list explicitly notes "INT-1 closure remains very close after this slice" but not closed.

### 5. PR description aligns with diff

PR #1850's body claims three changes:
1. Record the satisfied review for PR #1849.
2. Mark the implementation complete.
3. Narrow remaining residuals.

The diff does exactly those three things — no more, no less. No surprise edits, no rewording of unrelated checklist lines, no metadata churn elsewhere in the file.

## Notes

(Non-blocking observations only.)

### N1 — `def`-in-conditional gap is intentionally outside the residual

Review N2 at [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:221-227](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md) flags that `collect_captured_outer_names_transitively` does not recurse into `def`s nested inside `if`/`while`/`for`/`match`. The tracker line 463 does not mention this. That's the right call for an INT-1-scoped residual list — the review explicitly says "this PR is not introducing a new gap" and the gap is consistent with the pre-existing nested-return analysis pattern, so it's a more general capture-analysis concern that crosses milestones rather than a Type::Int codegen-migration tail. Worth confirming with the phase owner whether it belongs on a separate `internal_docs/phases/` follow-up doc; currently it lives only in the review history transcript.

### N2 — Sub-bullet voice consistency

The new line 462 uses present-tense "now propagate ... preserving ..." — matches the surrounding completed bullets verbatim. The new residual line 463 uses "still need seeded chained-forcing" — matches the prior residual's "still need transitive capture propagation". Voice and tense consistency are preserved. ✓

### N3 — Review filename normalization

The on-disk filename `integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md` follows the same `integer-model-int-1-sifrint-<topic>-review-pass-N.md` pattern as the prior 17 INT-1 review entries. The added line 415 reference uses the relative path with no `.md` substitution drift. Spot-checked against `ls reviews/ | grep multilevel-forced` — file exists. ✓

### N4 — Validation report signature

`report_signature=e1bf653aaa770517` matches the same signature reported by every recent INT-1 implementation/tracker PR (#1817 onward, per review N4). That's expected for a doc-only change — no test deltas — and confirms the local quick gate ran clean. The 69.05s wall time is in the same ballpark as other recent quick runs.

## Probe matrix

| Probe | Result |
|-------|--------|
| Diff scope (`git show 4451d6e2 --stat`) | 1 file, +3 -1 — matches PR description |
| Review file exists at the path referenced on line 415 | ✓ |
| Review file verdict line 9 = "Satisfied." | ✓ |
| Review N1 (chained-forcing follow-up) maps to line 463 added clause | ✓ |
| Review carry-forward bullets (1 chained-forcing, 2 augassign//`%`) match line 463 in order | ✓ |
| `gh pr view 1849` shows MERGED with title matching the line 462 narrative | ✓ |
| Top-level INT-1 checkbox at line 443 still `[ ]` | ✓ |
| Sub-bullet voice/tense matches surrounding entries | ✓ |
| `git diff --check` (per PR validation) | clean |
| `scripts/run_all_tests.sh --profile quick` (per PR validation) | report_signature=e1bf653aaa770517, 69.05s |

## Carry-forward open INT-1 items

Unchanged from the source review — restated for tracker continuity:

1. Multi-level chained-forcing in nested helpers: route `register_sifr_int_forced_local_bindings` through a seeded variant of `collect_sifr_int_forced_locals`, paired with a fixture exercising `derived: int = big + 1` inside a captured-forced inner. This is now the only Type::Int-codegen-migration item under INT-1.
2. Unsupported augmented assignment / fallible `//` and `%` exact-int runtime/codegen support.

After those land, the INT-1 milestone-closure review can proceed.
