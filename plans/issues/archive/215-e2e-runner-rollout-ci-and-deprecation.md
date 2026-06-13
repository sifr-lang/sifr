## [Task] CI Rollout, Default Switch, and Legacy Deprecation Plan

#### Current Situation
- New architecture will not be safe to default without staged rollout.
- CI currently assumes existing runner behavior.

#### Desired Situation
- CI validates correctness and performance trend for new runner.
- New runner becomes default after evidence-based confidence.
- Legacy path remains temporarily as fallback then is removed.

#### Suggested Solution
- Add CI matrix modes during transition:
  - legacy mode
  - new mode
  - mandatory differential/equivalence mode
- Define explicit switch criteria and rollback playbook.
- After stability window, deprecate and remove legacy path.

#### Implementation Checklist
- Update CI jobs and docs for both modes.
- Record baseline and post-change performance with fixed protocol:
  - warm cache benchmark set: discard first run, then collect 7 runs
  - report p50 and p95
  - run on fixed CI runner class with pinned job env vars:
    - `runs-on: ubuntu-24.04` (GitHub-hosted x64 standard runner)
- Enforce cutover gates:
  - correctness: differential mode must be green (exact equivalence)
  - performance: p50 <= 90s and p95 <= 110s for full `test_e2e_pass`
  - stability: coefficient of variation <= 0.15 across the 7-run sample
- Define cutover checklist and rollback command.
- Remove legacy path only after acceptance window.

#### Acceptance Criteria
- CI covers both correctness and throughput checks for rollout.
- New runner default switch is documented and reproducible.
- Legacy deprecation/removal has a reversible plan until final removal.
- Cutover is blocked unless all correctness/performance gates pass.

#### Dependencies
- Depends on Task 214.

### Implemented

#### CI Changes

- Added workflow: `.github/workflows/e2e-throughput-rollout.yml`
  - matrix job: `legacy`, `new`, `compare` with `SIFR_E2E_RUNNER_MODE`
  - runner class pinned to `ubuntu-24.04`
  - worker tuning envs:
    - `SIFR_E2E_SIFR_JOBS`
    - `SIFR_E2E_RUST_JOBS`
    - `SIFR_E2E_RUN_JOBS`
- Added script `scripts/ci_e2e_throughput.sh` implementing warm+7-sample throughput protocol:
  - warmup run discarded
  - 7 timed `test_e2e_pass` runs
  - compute `p50`, `p95`, and coefficient of variation
  - fail on:
    - `p50 > 90000ms`
    - `p95 > 110000ms`
    - `cv > 0.15`

#### Rollout and Reversibility

- Default remains `legacy` in `runner_config()` until gates pass and explicit cutover decision is made.
- Diff mode (`SIFR_E2E_RUNNER_MODE=compare`) is required for correctness parity evidence.
- Reversal command remains available:
  - set `SIFR_E2E_RUNNER_MODE=legacy` (or `SIFR_E2E_LEGACY_RUNNER=1`).
- Legacy path still present in `test_e2e_pass` behind `RunnerMode::Legacy`, allowing staged rollout.
