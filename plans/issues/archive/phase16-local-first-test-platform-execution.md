# Phase 16 Execution Checklist (Local-First Test Platform Foundation)

Status: completed (2026-03-03)
Owner: phase_16 execution loop
Reference phase doc: `internal_docs/phases/16_local_first_test_platform_foundation.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `./scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_16_1 Parallel Test Profiles
status: done (2026-03-03, PR #806)

- [x] Define local profiles: `quick`, `full`, `stress`
- [x] Make profile execution parallel-safe and reproducible
- [x] Document profile intent and runtime envelope
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `bash scripts/run_all_tests.sh --profile quick` -> pass.
- Positive path: `bash scripts/run_all_tests.sh --profile full` -> pass.
- Negative path: `bash scripts/run_all_tests.sh --profile invalid` -> exits `2` immediately with profile validation error.
- Milestone demo: `cargo run -q -p sifr -- run demos/m16_1_parallel_test_profiles_demo.sifr` -> `m16_1 profile demo: ok`.

## Part 2: milestone_16_2 Deterministic Reporting
status: done (2026-03-03, PR #807)

- [x] Stabilize output ordering
- [x] Stabilize summary format
- [x] Stabilize failure grouping
- [x] Ensure reruns produce equivalent reports
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -p sifr --test e2e test_failure_summary_is_grouped_and_order_stable -- --nocapture` -> pass.
- Positive path: `cargo test -p sifr --test e2e test_report_signature_is_order_invariant -- --nocapture` -> pass.
- Positive path: `bash scripts/check_e2e_report_determinism.sh --profile quick` -> pass (`deterministic report signature confirmed`).
- Negative path: `cargo test -p sifr --test e2e test_report_signature_changes_on_failure_delta -- --nocapture` -> pass (asserts signature changes on report delta).
- Negative path: `bash scripts/check_e2e_report_determinism.sh --profile invalid` -> exits `2` with profile validation error.
- Milestone demo: `cargo run -q -p sifr -- run demos/m16_2_deterministic_reporting_demo.sifr` -> `m16_2 deterministic reporting demo: ok`.

## Part 3: milestone_16_3 CI-Parity and Smoke Hardening
status: done (2026-03-03, PR #808)

- [x] Wire CI to run exact local scripts and flags
- [x] Add always-on smoke fuzz/property jobs
- [x] Confirm CI/local command parity is documented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc, checklist, and roadmap

Validation evidence:
- Positive path: `bash scripts/run_smoke_fuzz_property.sh` -> pass.
- Positive path: `bash scripts/run_all_tests.sh --profile full` -> pass.
- Negative path: `bash scripts/run_smoke_fuzz_property.sh --bad` -> exits `2` with usage error.
- Milestone demo: `cargo run -q -p sifr -- run demos/m16_3_ci_parity_smoke_hardening_demo.sifr` -> `m16_3 ci parity + smoke demo: ok`.

## PR Log
- Part 1: https://github.com/sifr-lang/sifr/pull/806 (merged)
- Part 2: https://github.com/sifr-lang/sifr/pull/807 (merged)
- Part 3: https://github.com/sifr-lang/sifr/pull/808 (merged)

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase16-review.md`
- Remediation PR (pass 1): https://github.com/sifr-lang/sifr/pull/810 (merged)
- External review pass 2 output: `reviews/phase16-production-grade-review.md`
- Remediation PR (pass 2): https://github.com/sifr-lang/sifr/pull/811 (merged)
