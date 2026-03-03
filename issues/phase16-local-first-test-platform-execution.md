# Phase 16 Execution Checklist (Local-First Test Platform Foundation)

Status: in progress (started 2026-03-03)
Owner: phase_16 execution loop
Reference phase doc: `.cursor/plans/main/phases/16_local_first_test_platform_foundation.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `./scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_16_1 Parallel Test Profiles
status: in review (PR pending merge)

- [x] Define local profiles: `quick`, `full`, `stress`
- [x] Make profile execution parallel-safe and reproducible
- [x] Document profile intent and runtime envelope
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `bash scripts/run_all_tests.sh --profile quick` -> pass.
- Positive path: `bash scripts/run_all_tests.sh --profile full` -> pass.
- Negative path: `bash scripts/run_all_tests.sh --profile invalid` -> exits `2` immediately with profile validation error.
- Milestone demo: `cargo run -q -p sifr -- run demos/m16_1_parallel_test_profiles_demo.sifr` -> `m16_1 profile demo: ok`.

## Part 2: milestone_16_2 Deterministic Reporting
status: pending

- [ ] Stabilize output ordering
- [ ] Stabilize summary format
- [ ] Stabilize failure grouping
- [ ] Ensure reruns produce equivalent reports
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

## Part 3: milestone_16_3 CI-Parity and Smoke Hardening
status: pending

- [ ] Wire CI to run exact local scripts and flags
- [ ] Add always-on smoke fuzz/property jobs
- [ ] Confirm CI/local command parity is documented
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc, checklist, and roadmap

## PR Log
- Part 1: pending
- Part 2: pending
- Part 3: pending

## Reviewer Follow-up
- External review pass 1 output: pending
- Remediation PR (pass 1): pending
- External review pass 2 output: pending
- Remediation PR (pass 2): pending
