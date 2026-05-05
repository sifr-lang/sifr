# Semantic Diagnostics Phase × LeetCode Corpus Relevance — Review Pass 1

Date: 2026-05-05
Scope: Verify the conclusion that the LeetCode `audits/leetcode` corpus is unaffected by, and not validated under, the semantic diagnostic code taxonomy phase (phase 31.7).

## Verdict

**No blocking findings.** The four-point conclusion is sound. The LeetCode corpus is structurally and procedurally separate from the semantic diagnostics phase, the recently-added issue artifacts are historical phase-31 corpus-adaptation docs, and the diagnostics-phase validation gate does not exercise the LeetCode corpus. Three minor evidence corrections noted below; none invalidate the conclusion.

## Evidence reviewed

### 1. Issue artifacts added in PR #1786 (commit `fb45ddb1`)

`git show --stat fb45ddb1` shows the commit added **only** five markdown files and touched no code, no fixtures, and no diagnostics surfaces:

- [issues/ad-hoc-signature-invalid-fixture-adaptation-checklist-2026-03-31.md](issues/ad-hoc-signature-invalid-fixture-adaptation-checklist-2026-03-31.md)
- [issues/ad-hoc-signature-invalid-fixture-adaptation-recategorization-2026-03-31.md](issues/ad-hoc-signature-invalid-fixture-adaptation-recategorization-2026-03-31.md)
- [issues/ownership-mutability-boundary-root-cause-2026-04-02.md](issues/ownership-mutability-boundary-root-cause-2026-04-02.md)
- `reviews/ownership-mutability-boundary-root-cause-review-pass1.md`
- `reviews/ownership-mutability-boundary-root-cause-review-pass2.md`

Their content is dated 2026-03-31 / 2026-04-02 and explicitly cites:

- `verification/leetcode/full_corpus_current_results_20260331_live_after_signature_adaptation.json`
- `verification/leetcode/full_corpus_current_results_20260402_live.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260402_live.json`
- `verification/leetcode/ownership_mutability_boundary_breakdown_20260402_live.json`

These are LeetCode corpus adaptation artifacts (signature adaptation batches, ownership/mutability boundary classification). They predate the diagnostics phase closure (2026-05-03 per the tracker preamble) by a month and discuss fixture-side adaptation, not diagnostic taxonomy work. **Conclusion claim 1 holds.**

### 2. Diagnostics phase tracker has no LeetCode coupling

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) was greppedfor LeetCode references — the four matches are:

- Line 11, 17, 50: references to roadmap **"phase 31.7"** (the diagnostics sub-phase), not the LeetCode Phase 31.
- Line 116: "fail corpus" (the e2e fail corpus), not the LeetCode corpus.
- Line 361: same e2e pass corpus advisory.

There are zero references to `audits/leetcode`, `phase 31` (LeetCode), or LeetCode corpus runs. The phase-closure reviews (`reviews/semantic-diagnostic-code-taxonomy-phase-closure-review-pass-{1,2,3}.md`) likewise contain no LeetCode coupling. **Conclusion claim 2 holds.**

### 3. Diagnostics phase validation gate excludes LeetCode

`scripts/run_all_tests.sh` contains no `leetcode` / `audits` references. The hardening commit `934973cd` added one line — `cargo test -p sifr_hir -- --skip test_e2e_pass` — to the gate, not a LeetCode invocation. The LeetCode driver is `audits/leetcode/run_audit.py`, which lives only in the nested submodule and is not wired into the main validation gate. **Conclusion claim 3 holds.**

### 4. Diag-11 hardening overlap with LeetCode dict.get(default=...) is purely diagnostic-quality

`git show 934973cd -- crates/sifr_hir/src/lower/method_call_args.rs` confirms the hardening change adds **primary diagnostic ranges** for the `default=` keyword on `dict.get/pop/setdefault` and the `reverse=` keyword on `list.sort`. The associated tests in `crates/sifr_hir/src/lower/expressions_tests.rs` (e.g. `test_dict_get_default_keyword_type_mismatch_has_type_code_and_range`) target *type-mismatched* defaults (`default="bad"` against `dict[int, int]`), not well-typed defaults. Existing valid uses in the LeetCode corpus (e.g. `prev_totals.get(col, default=0)` in `0120_triangle.sifr` against `dict[int, int]`) remain valid by construction — there is no fixture surface change required.

The targeted overlap check the user performed (`check` + `run` on every fixture matching `dict.get(..., default=...)`) is the right scoped validation for this change, and the listed nine fixtures are the complete set under `audits/leetcode/src/`. **Conclusion claim 4 holds.**

### 5. Working tree state

- `git status` reports `nothing to commit, working tree clean` on `main`, ahead of `origin/main` by zero commits.
- The nested `audits/leetcode` repo log ends at `5d5fbf7 Simplify LeetCode README`, with no in-progress diagnostics-phase edits.

No pending LeetCode work is associated with the diagnostics phase.

## Minor evidence corrections (non-blocking)

These do not change the verdict but should be tightened for accuracy.

### C1 — PR list overstates by one

The conclusion lists "PRs #1785, #1787, #1788". The diagnostics-phase tracker references **#1785 (closure)** and **#1787 (hardening)**; I found no reference to #1788 in `issues/`, `reviews/`, or `internal_docs/`. The post-hardening commit `b05e6118 "Document semantic diagnostics hardening PR"` only updates docs to point to #1787 — it is not a separate PR. Replace "#1785, #1787, #1788" with "#1785, #1787" unless #1788 exists out-of-tree and was inspected.

### C2 — "208 PASS / 203 NO_ORACLE / 2026-04-25" is one closure stale

The cited "208 PASS, 203 NO_ORACLE, 0 CHECK_ERROR/RUN_ERROR/TIMEOUT" snapshot is from [reviews/archive/ad-hoc-leetcode-divergence-closure-2026-04-24-review-pass3.md:13](reviews/archive/ad-hoc-leetcode-divergence-closure-2026-04-24-review-pass3.md) (date 2026-04-24, not 2026-04-25). A **later** closure artifact, [reviews/archive/leetcode-no-oracle-assertion-parity-2026-04-26-review-pass4-final.md:11](reviews/archive/leetcode-no-oracle-assertion-parity-2026-04-26-review-pass4-final.md) (2026-04-26), reports `summary.status_counts == {"PASS": 411}` and `summary.scope_counts.in_scope == 411` after the 203 `no_oracle` entries were promoted to `embedded_asserts`. The substantive point — 0 check / 0 run / 0 timeout failures in the latest archived corpus signal — still holds, and is in fact stronger under the 04-26 snapshot (PASS=411, all `embedded_asserts`). The conclusion should cite the 04-26 artifact as the latest signal.

### C3 — Sharpen the diag-11 overlap framing

It is worth explicitly stating *why* the targeted overlap check is sufficient: the hardening change only widens the **diagnostic primary range** on a *type-mismatched* `default=` keyword; it does not narrow the set of accepted programs. So well-typed `dict.get(key, default=<same-value-type>)` uses cannot regress as a class. The targeted run is therefore confirmatory rather than risk-driven. (Optional clarification — does not affect the verdict.)

## Bottom line

The four-point conclusion is correct:

1. The LeetCode issue/adaptation artifacts in PR #1786 are historical Phase-31 corpus-adaptation docs — not diagnostic-phase fixes.
2. The diagnostics phase did not edit, depend on, or leave pending work in the LeetCode corpus.
3. The LeetCode corpus is not part of the diagnostics-phase validation gate. The latest archived full-corpus signal shows zero check/run/timeout failures (correct cite is the 2026-04-26 PASS=411 closure, not the 2026-04-24 208/203 closure).
4. Unless the user separately requests a Phase 31 corpus rerun, no diagnostics-phase action is warranted.

Recommended next step: tighten the two cite-level inaccuracies (C1, C2) in any user-visible summary, then close.
