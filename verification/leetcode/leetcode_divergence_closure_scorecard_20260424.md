# LeetCode Divergence Closure Scorecard

Date: `2026-04-24`
Phase: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`

## Corpus Run

- Full run artifact: `verification/leetcode/full_corpus_current_results_20260424_leetcode_divergence_closure.json`
- Manifest: `verification/leetcode/full_corpus_manifest_20260402_live.json`
- Cases: `411`
- `PASS`: `208`
- `NO_ORACLE`: `203`
- `CHECK_ERROR`: `0`
- `RUN_ERROR`: `0`
- `TIMEOUT`: `0`

## Failure Taxonomy

- Taxonomy JSON: `verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure.json`
- Taxonomy Markdown: `verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure.md`
- Delta: `verification/leetcode/full_corpus_failure_taxonomy_20260424_leetcode_divergence_closure_delta_vs_20260409.md`
- Failing cases: `0`
- Failure categories: `0`

## Pair Divergence

- Pair scan artifact: `verification/leetcode/leetcode_pair_diff_scan_20260424.json`
- Paired cases: `395`
- Python-only cases: `1`
- Sifr-only cases: `16`

The remaining high-diff outliers are not silent rewrite debt:

- Closed structurally in this phase despite raw diff growth from explicit Sifr safety/helper code: `0004`, `0023`, `0024`, `0146`, `0147`, `0206`, `0208`, `0211`, `0212`, `0295`, `0706`, `0707`.
- Separately tracked blocker: `0148_sort_list` in `issues/leetcode-0148-owned-merge-sort-blocker-2026-04-24.md`.
- Architecture-boundary classifications: `verification/leetcode/leetcode_architecture_boundary_classification_20260424.md`.
- Remaining Category 2 ergonomics pressure is follow-up language/stdlib work, not hidden fixture rewrite debt.

## WS6 Remediation

The first WS6 full run exposed 12 legacy fixtures that no longer compiled or ran under the stricter Optional/index proof surface. WS6 made explicit, panic-free fixture-local handling for optional `pop`, subscript, tuple-index, and node-field access in:

- `0084_largest_rectangle_in_histogram`
- `0103_binary_tree_zigzag_level_order_traversal`
- `0232_implement_queue_using_stacks`
- `0332_reconstruct_itinerary`
- `0513_find_bottom_left_tree_value`
- `0735_asteroid_collision`
- `0739_daily_temperatures`
- `0838_push_dominoes`
- `0895_maximum_frequency_stack`
- `1046_last_stone_weight`
- `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`
- `1609_even_odd_tree`

Targeted `check` and `run` passed for all 12 before the final full corpus rerun.
