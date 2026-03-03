# Phase 16: Local-First Test Platform Foundation

## Objective
Make local parallel testing the authoritative quality gate and ensure CI mirrors it exactly.

## Depends on
- Phase 15

## Phase Demo Path
- `demos/phase16_exit_demo.sifr`

## Milestones

### milestone_16_1: Parallel Test Profiles
- Scope:
  - Define local profiles: `quick`, `full`, `stress`.
  - Make profile execution parallel-safe and reproducible.
- Definition of done:
  - Profiles run reliably on developer machines.
  - Profile purpose and runtime envelope are documented.

### milestone_16_2: Deterministic Reporting
- Scope:
  - Stabilize output ordering, summary format, and failure grouping.
  - Ensure reruns produce equivalent reports.
- Definition of done:
  - Identical inputs produce deterministic pass/fail summaries.
  - Failure reports are actionable and not order-noisy.

### milestone_16_3: CI-Parity and Smoke Hardening
- Scope:
  - Wire CI to run exact local scripts and flags.
  - Add always-on smoke fuzz/property jobs.
- Definition of done:
  - CI and local commands are 1:1.
  - Smoke fuzz/property checks run in default validation flow.

## Exit Gate
- Local parallel validation is trusted as primary, with CI parity confirmed.
