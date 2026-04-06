# Execution Ledger: Optional/Any Root-Cause Closure (2026-04-06)

## Baseline

- source run: `verification/leetcode/full_corpus_current_results_20260406_live_rerun1.json`
- source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun1.json`
- focused map: `verification/leetcode/phase_apr06_on_au_root_cause_map.json`

## Workstream Checklist

- [ ] `W1-ON-arithmetic-and-operator-narrowing`
  - progress: focused residual reduced `15 -> 4` in latest cold-cache waves
  - closed in this lane: `0134`, `0338`, `2482`, `0153`, `0658`, `0918`, `0410`, `1011`, `0076`, `1288`, `1475`
- [ ] `W2-ON-container-boundary-and-element-refinement`
- [ ] `W3-AU-flow-stabilization-and-operator-safety`
- [ ] `W4-AU-compat-container-contract-typing`
- [ ] `W5-ON-contract-return-closure`
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
- completed: local validation gate -> `scripts/run_all_tests.sh --profile quick` (pass)
- pending: full-corpus rerun
- pending: taxonomy refresh
