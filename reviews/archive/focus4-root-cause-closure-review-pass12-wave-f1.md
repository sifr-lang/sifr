# Focus4 Root-Cause Closure Review Pass 12 (Wave F1)

Date: 2026-04-06
Scope: Phase reporting closure (full-corpus rerun + taxonomy regeneration + delta reporting)

## Reviewed Changes

- Added deterministic taxonomy/delta generator:
  - `scripts/build_full_corpus_failure_taxonomy.py`
- Generated rerun3 full-corpus artifacts:
  - `verification/leetcode/full_corpus_current_results_20260406_live_rerun3.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3.md`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3_delta_vs_rerun2.md`
- Updated phase docs/checklists:
  - `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06.md`
  - `issues/ad-hoc-phase-focus4-root-cause-closure-2026-04-06-execution.md`

## Validation Evidence

- Release compiler build:
  - `cargo build --release -p sifr` passed
- Full-corpus rerun command:
  - `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/full_corpus_manifest_20260402_live.json --output verification/leetcode/full_corpus_current_results_20260406_live_rerun3.json --sifr-bin ./target/release/sifr --no-build-release-if-missing`
  - summary: `PASS=168`, `CHECK_ERROR=111`, `RUN_ERROR=13`, `NO_ORACLE=119`
- Taxonomy/delta regeneration command:
  - `python3 scripts/build_full_corpus_failure_taxonomy.py --results verification/leetcode/full_corpus_current_results_20260406_live_rerun3.json --output-json verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3.json --baseline-taxonomy verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2.json --delta-md verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun3_delta_vs_rerun2.md --generated-on 2026-04-06`
  - produced taxonomy markdown and delta markdown artifacts

## Reviewer Notes

- Remaining checklist items under "Validation and Reporting" are now complete in the execution ledger.
- Multi-workstream convergence fixtures remain unresolved and are explicitly kept unchecked; focus4 primary root-cause closure status remains unchanged (`0/x` primary presence for all mapped `AU/DS/RF/CF` lanes).
