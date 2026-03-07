# Phase 29 Execution Checklist (Verification Hardening)

Status: in_progress (started 2026-03-08)
Owner: phase_29 execution loop
Reference phase docs:
- `.cursor/plans/main/phases/29_verification_hardening.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 29 To-Do Plan

### Part 1: milestone_29_1 Suite Taxonomy and Baseline Governance
- [ ] Define canonical suite taxonomy and per-suite contracts
- [ ] Add baseline-backed verification for diagnostics and project behavior
- [ ] Define canonical checked-in artifacts and normalization rules
- [ ] Define one canonical bless/accept workflow
- [ ] Add demo: `demos/m29_1_suite_taxonomy_and_baseline_governance_demo/main.sifr`
- [ ] Add positive and negative validations for diagnostics/project baseline checks
- [ ] Run milestone demo + full local suite
- [ ] Open PR, review, and merge

### Part 2: milestone_29_2 Fixedbugs and Crashes Corpus
- [ ] Require resolved compiler bugs in scope to land in `fixedbugs`
- [ ] Add issue-linked metadata and root-cause traceability contract
- [ ] Define and enforce `crashes` sentinel policy and promotion rules
- [ ] Add demo: `demos/m29_2_fixedbugs_and_crashes_corpus_demo/main.sifr`
- [ ] Add positive and negative validations for corpus metadata/policy checks
- [ ] Run milestone demo + full local suite
- [ ] Open PR, review, and merge

### Part 3: milestone_29_3 Fuzz and Property Operationalization
- [ ] Define fuzz targets/property suites and seed corpora manifests
- [ ] Implement deterministic local smoke gate for fuzz/property suites
- [ ] Define triage/minimization/reproducibility workflow and artifacts
- [ ] Add demo: `demos/m29_3_fuzz_and_property_operationalization_demo/main.sifr`
- [ ] Add positive and negative validations for deterministic smoke gates
- [ ] Run milestone demo + full local suite
- [ ] Open PR, review, and merge

### Part 4: milestone_29_4 Curated OSS Gate and Broader Ecosystem Lane
- [ ] Add pinned curated OSS gate manifest with owners/rationale/commands/timeouts
- [ ] Implement blocking curated gate execution path
- [ ] Define broader non-blocking ecosystem lane and structured result classification
- [ ] Add demo: `demos/m29_4_curated_oss_gate_and_ecosystem_lane_demo/main.sifr`
- [ ] Add positive and negative validations for curated/broader lanes
- [ ] Run milestone demo + full local suite
- [ ] Open PR, review, and merge

### Part 5: milestone_29_5 Deterministic Scale, Flake Control, and Structured Evidence
- [ ] Define deterministic sharding strategy and per-suite runtime expectations
- [ ] Add rerun policy, quarantine contract, and re-enable criteria
- [ ] Emit machine-readable artifacts from hardening gates
- [ ] Make suggestion/autofix boundary explicit and enforce chosen contract in verification
- [ ] Add demo: `demos/m29_5_deterministic_scale_flake_control_and_structured_evidence_demo/main.sifr`
- [ ] Add positive and negative validations for shard/rerun/flake and artifact contracts
- [ ] Run milestone demo + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_29_1 Suite Taxonomy and Baseline Governance
status: done (2026-03-08, pending PR link)

- [x] Canonical suite taxonomy and contract docs added
- [x] Baseline-backed diagnostics and project suites added
- [x] Canonical bless/accept workflow implemented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m29_1_suite_taxonomy_and_baseline_governance_demo/main.sifr` -> prints milestone demo contract lines and exits `0`.
- Positive path: `python3 scripts/run_verification_hardening.py --profile full` -> `verification ok: variants=7, failures=0, blocking_failures=0`.
- Positive path: `python3 scripts/run_verification_hardening.py --profile full --bless` -> baselines updated deterministically.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: project case `missing_import_reports_error` in `verification/suites/manifest.json` is expected-fail and baseline-locked across `human|json|compact` diagnostics with exit code `1`.

## PR Log
- Pending

## External Review Passes
- Pending
