# Phase 31 Follow-up: External Review Pass 1 Closure Hardening

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Source review: `reviews/phase31-ad-hoc-followup-milestones-review-pass-1.md`

## Goal

Address external review pass 1 findings around weak `NO_ORACLE` closures and unresolved regression ownership.

## Validated Findings and Actions

1. `NO_ORACLE` manifest mismatch was valid:
   - all 14 `NO_ORACLE` seed entries already had embedded assertions
   - action: upgraded all 14 to `oracle.mode = "embedded_asserts"` in `verification/leetcode/phase31_seed_corpus.json`
2. Regression triplet ownership gap (`0007`, `0009`, `0151`) was valid:
   - action: canonical mutability adaptation landed by adding explicit `mut` parameter markers in:
     - `audits/leetcode/0007_reverse_integer.sifr`
     - `audits/leetcode/0009_palindrome_number.sifr`
     - `audits/leetcode/0151_reverse_words_in_a_string.sifr`
3. Residual non-pass seed cases after oracle upgrade were revalidated:
   - `0001` run-stage closure:
     - added explicit typed fallback return path
     - file: `audits/leetcode/0001_two_sum.sifr`
   - `0242` closure:
     - canonicalized to sorting-based anagram check to avoid dict key move/run-stage borrow regression
     - file: `audits/leetcode/0242_valid_anagram.sifr`
4. Policy-doc mismatch was valid:
   - action: corrected oracle-mode wording in:
     - `internal_docs/verification/phase31_leetcode_corpus_policy.md`

## Targeted Validation Artifacts

- Oracle-upgrade 14-case rerun:
  - `verification/leetcode/phase31_review_pass1_oracle_upgrade_results.json`
  - status: `PASS=14`
- Regression triplet rerun:
  - baseline: `verification/leetcode/phase31_review_pass1_regression_triplet_baseline.json` (`CHECK_ERROR=3`)
  - post-fix: `verification/leetcode/phase31_review_pass1_regression_triplet_results.json` (`PASS=3`)
- Residual pair rerun:
  - `verification/leetcode/phase31_review_pass1_residual_pair_results.json` (`PASS=2`)
- Full seed rerun:
  - `verification/leetcode/phase31_review_pass1_full_results_v2.json`
  - status: `PASS=50`, `CHECK_ERROR=0`, `RUN_ERROR=0`

## Closure Decision

External review pass 1 is resolved with root-cause fixes and verified corpus evidence. Phase 31 seed corpus is fully green in current manifest mode (`PASS=50`).

