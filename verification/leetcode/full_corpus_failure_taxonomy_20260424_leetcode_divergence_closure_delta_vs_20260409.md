# Full Corpus Taxonomy Delta Report

- Baseline results: `verification/leetcode/full_corpus_current_results_20260409_live_rerun1.json`
- Baseline taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260409_live_rerun1.json`
- Current results: `verification/leetcode/full_corpus_current_results_20260424_leetcode_divergence_closure.json`
- Current taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure.json`
- Generated on: `2026-04-24`

## Status Delta

The April 9 baseline artifact records all 411 cases as `PASS`. The April 24 runner distinguishes embedded-oracle successes from successful fixtures without embedded assertions:

- `PASS`: `411 -> 208`
- `NO_ORACLE`: `0 -> 203`
- `CHECK_ERROR`: `0 -> 0`
- `RUN_ERROR`: `0 -> 0`
- `TIMEOUT`: `0 -> 0`

## Failure Taxonomy Delta

- Failing cases: `0 -> 0`
- Category count: `0 -> 0`

No compile or runtime failures remain after WS6 closure remediation.
