# Execution Ledger: Optional/Any Root-Cause Closure (2026-04-06)

## Closure Summary

- phase state: `DONE`
- focused residual status: `CHECK_ERROR=0` (wave26)
- local validation: `scripts/run_all_tests.sh --profile quick` passed
- full validation: `scripts/run_all_tests.sh` passed
- full-corpus rerun/taxonomy refresh completed (rerun2 artifacts)

## Baseline

- source run: `verification/leetcode/full_corpus_current_results_20260406_live_rerun1.json`
- source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun1.json`
- focused map: `verification/leetcode/phase_apr06_on_au_root_cause_map.json`

## Workstream Checklist

- [x] `W1-ON-arithmetic-and-operator-narrowing`
  - progress: focused residual reduced `15 -> 0` across wave13..wave18
  - closed in this lane: `0134`, `0338`, `2482`, `0153`, `0658`, `0918`, `0410`, `1011`, `0076`, `1288`, `1475`, `0149`, `0286`, `1074`, `2001`
- [x] `W2-ON-container-boundary-and-element-refinement`
  - progress: focused residual reduced `6 -> 0` across wave19..wave20
  - closed in this lane: `0347`, `0210`, `0785`, `0787`, `2092`, `2101`, `0253`, `0280`, `0456`
- [x] `W3-AU-flow-stabilization-and-operator-safety`
  - progress: AU-3 residual reduced `5 -> 0` across wave22..wave25
  - closed in this lane: `1642`, `0155`, `0232`, `0303`, `0535`
- [x] `W4-AU-compat-container-contract-typing`
  - progress: AU-1 residual reduced `4 -> 0` across wave23..wave24
  - closed in this lane: `0621`, `0767`, `1481`, `1985`
- [x] `W5-ON-contract-return-closure`
  - progress: compiler-owned ON contract/return residuals are fully removed from focused `CHECK_ERROR` and remaining residuals were closed under `A2` policy lane
- [x] `A1-signature-annotation-required`
- [x] `A2-explicit-guard-canonicalization`
  - closed residual fixtures: `0208`, `0332`, `0752`, `0929`, `1466`, `1845`

## Suggested Execution Order

- Tier 1 (parallel): `W1`, `W2`, `A1`
- Tier 2 (after `W1` + `W2`): `W3`
- Tier 3 (after `W3`): `W4`
- Tier 4 (after `W1` + `W2`): `W5`
- Tier 5 (after `W5`): `A2`

## Review Log

- completed: Claude pass1b -> `reviews/optional-any-root-cause-phase-review-pass1b.md`
- completed: Claude pass2 -> `reviews/optional-any-root-cause-phase-review-pass2.md`
- completed: Claude pass3 final audit (READY) -> `reviews/optional-any-root-cause-phase-review-pass3.md`

## Rerun Log

- completed: focused 58-fixture rerun -> `/tmp/phase_apr06_on_au_wave7b_a1_adaptation_1472_coldcache.json`
- completed: focused compiler follow-up rerun -> `/tmp/phase_apr06_on_au_wave8_subscript_guard_defaultdict_coldcache.json`
- completed: focused A2 residual batch rerun -> `/tmp/phase_apr06_on_au_wave9_a2_residual_batch_coldcache.json`
- completed: focused A2 closure + stability rerun -> `/tmp/phase_apr06_on_au_wave11_a2_full6_plus_0350_stability_coldcache.json`
- completed: focused ON-1 adaptation subset rerun -> `/tmp/phase_apr06_on_au_wave13_on1_adaptation_subset3_coldcache.json`
- completed: focused ON-1 follow-up rerun -> `/tmp/phase_apr06_on_au_wave14_on1_adaptation_subset3b_coldcache.json`
- completed: focused ON-1 expansion rerun (transient run error on `0076`) -> `/tmp/phase_apr06_on_au_wave15_on1_adaptation_subset6_coldcache.json`
- completed: focused ON-1 expansion rerun + `0076` run fix -> `/tmp/phase_apr06_on_au_wave16_on1_adaptation_subset6_plus_0076_runfix_coldcache.json`
- completed: focused ON-1 pair closure rerun (`1288`, `1475`) -> `/tmp/phase_apr06_on_au_wave17_on1_adaptation_1288_1475_coldcache.json`
- completed: focused ON-1 full closure rerun (`0149`, `0286`, `1074`, `2001`) -> `/tmp/phase_apr06_on_au_wave18_on1_full_closure_coldcache.json`
- completed: focused ON-2 single closure rerun (`0347`) -> `/tmp/phase_apr06_on_au_wave19_on2_single_0347_coldcache.json`
- completed: focused ON-2 full closure rerun (`0210`, `0785`, `0787`, `2092`, `2101`) -> `/tmp/phase_apr06_on_au_wave20_on2_full_closure_coldcache.json`
- completed: focused ON-3 full closure rerun (`0253`, `0280`, `0456`) -> `/tmp/phase_apr06_on_au_wave21_on3_full_closure_coldcache.json`
- completed: focused AU-3 single closure rerun (`1642`) -> `/tmp/phase_apr06_on_au_wave22_au3_single_1642_coldcache.json`
- completed: focused AU-1 single closure rerun (`0621`) -> `/tmp/phase_apr06_on_au_wave23_au1_single_0621_coldcache.json`
- completed: focused AU-1 full closure rerun (`0767`, `1481`, `1985`) -> `/tmp/phase_apr06_on_au_wave24_au1_full_closure_coldcache.json`
- completed: focused AU-3 full closure rerun (`0155`, `0232`, `0303`, `0535`) -> `/tmp/phase_apr06_on_au_wave25_au3_full_closure_coldcache.json`
- completed: focused AU-2 full closure rerun (`0079`, `0118`, `0225`, `0269`, `0496`, `0901`, `0909`, `0953`, `1049`, `1462`, `1572`, `2306`) -> `/tmp/phase_apr06_on_au_wave26_au2_full_closure_coldcache.json`
- completed: local validation gate -> `scripts/run_all_tests.sh --profile quick` (pass)
- completed: full validation gate -> `scripts/run_all_tests.sh` (pass; profile `pr`)
- completed: full-corpus rerun -> `verification/leetcode/full_corpus_current_results_20260406_live_rerun2.json`
- completed: taxonomy refresh + delta report ->
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2.md`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2_delta.md`
- investigated: non-targeted stability drift (`0115_distinct_subsequences`: `RUN_ERROR -> PASS`) confirmed as improvement, not regression
