# Phase 16: Local-First Test Platform Foundation

## Objective
Make local parallel testing the authoritative quality gate and ensure CI mirrors it exactly.

## Depends on
- Phase 15

## Milestones

### milestone_16_1: Parallel Test Profiles
status: done (2026-03-03, PR #806)
- Scope:
  - Define local profiles: `create-pr`, `merge`, `release`.
  - Make profile execution parallel-safe and reproducible.
- Definition of done:
  - Profiles run reliably on developer machines.
  - Profile purpose and runtime envelope are documented.
- Evidence:
  - Profile contracts and runtime envelopes are implemented in `scripts/run_e2e_pass.sh`.
  - Local-first profile entrypoint is implemented in `scripts/run_all_tests.sh --profile <create-pr|merge|release>`.
  - Profile-specific cache roots are enforced through `SIFR_E2E_CACHE_DIR` wiring in `crates/sifr/tests/e2e.rs`.
  - Milestone demo: `cargo run -q -p sifr -- run demos/m16_1_parallel_test_profiles_demo.sifr`.

### milestone_16_2: Deterministic Reporting
status: done (2026-03-03, PR #807)
- Scope:
  - Stabilize output ordering, summary format, and failure grouping.
  - Ensure reruns produce equivalent reports.
- Definition of done:
  - Identical inputs produce deterministic pass/fail summaries.
  - Failure reports are actionable and not order-noisy.
- Evidence:
  - Deterministic tie-break ordering for slowest-group reporting is enforced in `crates/sifr/tests/e2e.rs`.
  - Failure summaries are grouped by stage (`compile`, `planning`, `build`, `run`, `other`) with deterministic ordering.
  - Stable `report_signature` output is emitted per e2e pass run (`[sifr-e2e] report_signature=...`).
  - Rerun-equivalence command is provided by `scripts/check_e2e_report_determinism.sh`.
  - Milestone demo: `cargo run -q -p sifr -- run demos/m16_2_deterministic_reporting_demo.sifr`.

### milestone_16_3: CI-Parity and Smoke Hardening
status: done (2026-03-03, PR #808)
- Scope:
  - Wire CI to run exact local scripts and flags.
  - Add always-on smoke fuzz/property jobs.
- Definition of done:
  - CI and local commands are 1:1.
  - Smoke fuzz/property checks run in default validation flow.
- Evidence:
  - CI workflow parity is implemented in `.github/workflows/local-first-validation.yml`, invoking `scripts/run_all_tests.sh` profiles directly.
  - Always-on smoke jobs are implemented via `scripts/run_smoke_fuzz_property.sh` and `scripts/check_e2e_report_determinism.sh`.
  - Smoke fuzz/property checks are embedded into default local validation through `cargo test -p sifr -- --skip test_e2e_pass` in `scripts/run_all_tests.sh`.
  - Milestone demo: `cargo run -q -p sifr -- run demos/m16_3_ci_parity_smoke_hardening_demo.sifr`.

## Quality Contract
- Entry criteria: Phase 15 is completed and canonical backlog/contracts are finalized.
- Exit criteria: Local parallel validation is trusted as primary, with CI parity confirmed.
- Milestone quality checks:
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_16_1` (Parallel Test Profiles): validation goals cover: Define local profiles: `create-pr`, `merge`, `release`; Make profile execution parallel-safe and reproducible. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_16_2` (Deterministic Reporting): validation goals cover: Stabilize output ordering, summary format, and failure grouping; Ensure reruns produce equivalent reports. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_16_3` (CI-Parity and Smoke Hardening): validation goals cover: Wire CI to run exact local scripts and flags; Add always-on smoke fuzz/property jobs. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Local parallel validation is trusted as primary, with CI parity confirmed.

## Exit Gate
- Local parallel validation is trusted as primary, with CI parity confirmed.
