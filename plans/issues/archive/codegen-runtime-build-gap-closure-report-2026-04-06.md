# Codegen Runtime Build Gap Closure Report (2026-04-06)

Status: CLOSED
Owning phase: `issues/ad-hoc-codegen-runtime-build-gap-closure-phase-2026-04-05.md`
Execution log: `issues/ad-hoc-codegen-runtime-build-gap-closure-phase-2026-04-05-execution.md`

## Exit criteria
- Targeted scoped bucket (`58` fixtures) reached `RUN_ERROR=0`.
- Fresh full-corpus rerun and taxonomy artifacts were produced.
- Local authoritative validation gate passed.

## Final artifacts
- Scoped closure runner:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260406_wave16_post_patch18_ws2closure_runner.json`
  - summary: `case_count=58`, `NO_ORACLE=48`, `PASS=10`, `RUN_ERROR=0`
- Full corpus rerun:
  - `verification/leetcode/full_corpus_current_results_20260406_live_rerun2.json`
  - summary: `case_count=411`, `CHECK_ERROR=125`, `RUN_ERROR=4`, `PASS=168`, `NO_ORACLE=114`
- Refreshed full-corpus taxonomy:
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2.md`

## Closure statement
`codegen_runtime_build_gap` was reduced from baseline `58` to `0` and no longer appears in the refreshed full-corpus taxonomy category counts.

## Validation
- `scripts/run_all_tests.sh --profile quick` -> PASS

## PR
- Merged: https://github.com/sifr-lang/sifr/pull/1575
