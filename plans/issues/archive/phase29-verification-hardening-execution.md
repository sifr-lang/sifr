# Phase 29 Execution Checklist (Verification Hardening)

Status: done (started 2026-03-08, completed 2026-03-08)
Owner: phase_29 execution loop
Reference phase docs:
- `internal_docs/phases/29_verification_hardening.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 29 To-Do Plan

### Part 1: milestone_29_1 Suite Taxonomy and Baseline Governance
- [x] Define canonical suite taxonomy and per-suite contracts
- [x] Add baseline-backed verification for diagnostics and project behavior
- [x] Define canonical checked-in artifacts and normalization rules
- [x] Define one canonical bless/accept workflow
- [x] Add demo: `demos/m29_1_suite_taxonomy_and_baseline_governance_demo/main.sifr`
- [x] Add positive and negative validations for diagnostics/project baseline checks
- [x] Run milestone demo + full local suite
- [x] Open PR, review, and merge

### Part 2: milestone_29_2 Fixedbugs and Crashes Corpus
- [x] Require resolved compiler bugs in scope to land in `fixedbugs`
- [x] Add issue-linked metadata and root-cause traceability contract
- [x] Define and enforce `crashes` sentinel policy and promotion rules
- [x] Add demo: `demos/m29_2_fixedbugs_and_crashes_corpus_demo/main.sifr`
- [x] Add positive and negative validations for corpus metadata/policy checks
- [x] Run milestone demo + full local suite
- [x] Open PR, review, and merge

### Part 3: milestone_29_3 Fuzz and Property Operationalization
- [x] Define fuzz targets/property suites and seed corpora manifests
- [x] Implement deterministic local smoke gate for fuzz/property suites
- [x] Define triage/minimization/reproducibility workflow and artifacts
- [x] Add demo: `demos/m29_3_fuzz_and_property_operationalization_demo/main.sifr`
- [x] Add positive and negative validations for deterministic smoke gates
- [x] Run milestone demo + full local suite
- [x] Open PR, review, and merge

### Part 4: milestone_29_4 Curated OSS Gate and Broader Ecosystem Lane
- [x] Add pinned curated OSS gate manifest with owners/rationale/commands/timeouts
- [x] Implement blocking curated gate execution path
- [x] Define broader non-blocking ecosystem lane and structured result classification
- [x] Add demo: `demos/m29_4_curated_oss_gate_and_ecosystem_lane_demo/main.sifr`
- [x] Add positive and negative validations for curated/broader lanes
- [x] Run milestone demo + full local suite
- [x] Open PR, review, and merge

### Part 5: milestone_29_5 Deterministic Scale, Flake Control, and Structured Evidence
- [x] Define deterministic sharding strategy and per-suite runtime expectations
- [x] Add rerun policy, quarantine contract, and re-enable criteria
- [x] Emit machine-readable artifacts from hardening gates
- [x] Make suggestion/autofix boundary explicit and enforce chosen contract in verification
- [x] Add demo: `demos/m29_5_deterministic_scale_flake_control_and_structured_evidence_demo/main.sifr`
- [x] Add positive and negative validations for shard/rerun/flake and artifact contracts
- [x] Run milestone demo + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_29_1 Suite Taxonomy and Baseline Governance
status: done (2026-03-08, PR #920)

- [x] Canonical suite taxonomy and contract docs added
- [x] Baseline-backed diagnostics and project suites added
- [x] Canonical bless/accept workflow implemented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m29_1_suite_taxonomy_and_baseline_governance_demo/main.sifr` -> prints milestone demo contract lines and exits `0`.
- Positive path: `python3 scripts/run_verification_hardening.py --profile full` -> `verification ok: variants=7, failures=0, blocking_failures=0`.
- Positive path: `python3 scripts/run_verification_hardening.py --profile full --bless` -> baselines updated deterministically.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: project case `missing_import_reports_error` in `verification/suites/manifest.json` is expected-fail and baseline-locked across `human|json|compact` diagnostics with exit code `1`.

## Part 2: milestone_29_2 Fixedbugs and Crashes Corpus
status: done (2026-03-08, PR #921)

- [x] Require resolved compiler bugs in scope to land in `fixedbugs`
- [x] Add issue-linked metadata and root-cause traceability
- [x] Define `crashes` sentinel policy and promotion rules
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m29_2_fixedbugs_and_crashes_corpus_demo/main.sifr` -> prints fixedbugs/crashes corpus contract lines and exits `0`.
- Positive path: `python3 scripts/run_verification_hardening.py --profile full` -> `verification ok: variants=12, failures=0, blocking_failures=0` (includes `fixedbugs` and `crashes` suites).
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass with `fixedbugs` and `crashes` enforced via `scripts/run_verification_hardening.py`.
- Negative path: fixedbugs regression `FB-0002` executes `crates/sifr/tests/e2e/fail/unsupported_default_expr_call.sifr` with expected `check` exit code `1` and fails the hardening gate if it regresses to success.
- Negative path: crash sentinels require valid metadata and existing `source_reference`; missing fields/path would fail `crashes` suite metadata validation.

## Part 3: milestone_29_3 Fuzz and Property Operationalization
status: done (2026-03-08, PR #922)

- [x] Define fuzz targets, property suites, and seed corpora manifests
- [x] Define reproducibility, deduplication, minimization, and triage rules
- [x] Separate local smoke fuzz/property gates from sustained non-blocking lane
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m29_3_fuzz_and_property_operationalization_demo/main.sifr` -> prints property/fuzz operationalization contract lines and exits `0`.
- Positive path: the historical phase-29 full-profile command passed the then-current `property` and fuzz suite. The current suite names are `cargo-smoke`, `mutation-smoke`, and `sustained-fuzz`.
- Positive path: `bash scripts/run_smoke_fuzz_property.sh` -> pass for legacy smoke tests plus the phase-29 property and fuzz suite runner invocation.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass with `property` and the current fuzz suites active in the canonical hardening gate.
- Negative path: property entry `PROP-0001` enforces deterministic failing diagnostics for invalid import seed (`expect_exit_code=1`) and would fail on drift (`exit-code|stdout|stderr`).
- Negative path: the current fuzz suites fail on panic signals or non-allowed exit codes and enforce the configured corpus and finding contracts.

## Part 4: milestone_29_4 Curated OSS Gate and Broader Ecosystem Lane
status: done (2026-03-08, PR #923)

- [x] Build pinned curated OSS gate with owner/rationale/commands/timeout metadata
- [x] Separate broader non-blocking ecosystem lane with explicit classification
- [x] Enforce structured reproducible execution through canonical verification runner
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m29_4_curated_oss_gate_and_ecosystem_lane_demo/main.sifr` -> prints curated-vs-broader lane contract lines and exits `0`.
- Positive path: `python3 scripts/run_verification_hardening.py --profile full --suite oss-curated --suite ecosystem-broader` -> `verification ok: variants=7, failures=0, blocking_failures=0, non_blocking_failures=0`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass with `oss-curated` and `ecosystem-broader` suites active in canonical hardening gate.
- Negative path: curated project `OSS-CURATED-0002` initially failed (`unexpected-exit`) due list-borrow codegen mismatch; fixture was corrected (`own` parameter + safe option narrowing) and gate revalidated.
- Negative path: `ecosystem-broader` suite is marked `blocking=false`; mismatches remain signal-only and are reported in machine-readable artifacts without merge blocking.

## Part 5: milestone_29_5 Deterministic Scale, Flake Control, and Structured Evidence
status: done (2026-03-08, PR #924)

- [x] Define deterministic suite sharding strategy and CLI controls (`--shard-total`, `--shard-index`)
- [x] Add rerun tracking contract (`--rerun-failures`) and quarantine metadata validation
- [x] Add determinism-scale suite (repeat-run + sequential-vs-parallel equivalence checks)
- [x] Emit structured machine-readable artifacts with shard/rerun/quarantine metadata
- [x] Keep suggestion/autofix boundary explicit (`suggestions` baseline-validated, autofix application deferred)
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m29_5_deterministic_scale_flake_control_and_structured_evidence_demo/main.sifr` -> prints deterministic-scale/flake contract lines and exits `0`.
- Positive path: `bash scripts/check_e2e_sequential_parallel_equivalence.sh --profile quick` -> confirms identical report signature between sequential and parallel e2e runs.
- Positive path: `python3 scripts/run_verification_hardening.py --profile full --suite determinism-scale` -> `verification ok: variants=2, failures=0, blocking_failures=0, non_blocking_failures=0`.
- Positive path: sharding check `python3 scripts/run_verification_hardening.py --profile full --suite diagnostics --suite project --shard-total 2 --shard-index 0` and `--shard-index 1` -> deterministic suite partitioning with stable outcomes.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass with determinism-scale included in canonical hardening gate.
- Negative path: determinism/flake runner paths mark suite failures as blocking and execute rerun attempts; fail->pass transitions are recorded in `flake_events` instead of being silently treated as clean.
- Negative path: quarantine metadata file `verification/flake/quarantine.json` is schema-validated; malformed entries or unknown suites fail the hardening gate.

## PR Log
- Part 1: merged https://github.com/sifr-lang/sifr/pull/920
- Part 2: merged https://github.com/sifr-lang/sifr/pull/921
- Part 3: merged https://github.com/sifr-lang/sifr/pull/922
- Part 4: merged https://github.com/sifr-lang/sifr/pull/923
- Part 5: merged https://github.com/sifr-lang/sifr/pull/924
- External review pass 1 remediation: merged https://github.com/sifr-lang/sifr/pull/926
- External review pass 2 remediation: merged https://github.com/sifr-lang/sifr/pull/927

## External Review Passes
- Reviewer pass 1 request: `reviews/phase-29-review.md` (requested via talk-to-claude external app)
- Reviewer pass 1 remediation: done (2026-03-08, PR #926)
  - fixed determinism script execute permission for DET-0002
  - required and validated crash `reproducer_fixture` metadata and added minimized fixtures
  - operationalized quarantine metadata format with concrete template entry
- Reviewer pass 2 request: `reviews/phase-29-production-grade-review.md` (requested via talk-to-claude external app)
- Reviewer pass 2 remediation: done (2026-03-08, PR #927)
  - added pinned-revision validation (`local-main@<sha>`) against latest commit touching `project_root`
  - refreshed OSS manifest pinned revisions to current project revision (`local-main@f6ababa5`)
  - expanded fuzz seed corpus and deterministic mutation operator coverage
- Reviewer pass 3 request: `reviews/phase-29-production-grade-review-2.md` (requested via talk-to-claude external app)
- Reviewer pass 3 status: done (2026-03-08, no additional remediation required)
  - reviewer assessment: production-ready; previous findings remain closed
  - validation evidence: `python3 scripts/run_verification_hardening.py --profile quick` -> `verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`
