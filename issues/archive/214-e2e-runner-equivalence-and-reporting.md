## [Task] Legacy Equivalence and Failure Reporting Compatibility

#### Current Situation
- Throughput work can accidentally change user-visible failure behavior.
- Legacy/new runner equivalence is not yet formally enforced.

#### Desired Situation
- New runner produces equivalent pass/fail outcomes versus legacy.
- Failure output remains actionable and stable.
- Developers can switch between runners via toggle during rollout.

#### Suggested Solution
- Add explicit runner-mode contract:
  - Primary: `SIFR_E2E_RUNNER_MODE=legacy|new|compare`
  - Back-compat mapping:
    - `SIFR_E2E_NEW_RUNNER=1` => `new`
    - `SIFR_E2E_LEGACY_RUNNER=1` => `legacy`
  - Conflict rule: if both legacy/new booleans are set, fail fast with clear error.
  - Default rule during transition: `legacy` unless CI job explicitly sets mode.
- Add equivalence test command that compares outcome sets.
- Add report-shape assertions for key failure scenarios.

#### Implementation Checklist
- Keep legacy path available while new path stabilizes.
- Add differential comparison harness in CI/local command.
- Ensure report includes fixture name, mismatch details, and artifact context.
- Verify aggregate failure count semantics match legacy.
- Add mode-resolution unit tests for:
  - explicit mode values
  - back-compat env mapping
  - conflict behavior

#### Acceptance Criteria
- Legacy/new modes are both runnable.
- Equivalence checks pass on full corpus.
- Failure report format remains at least as informative as legacy.
- Mode resolution behavior is deterministic and documented.

#### Dependencies
- Depends on Task 213.

### Implemented

- Added explicit mode contract in `crates/sifr/tests/e2e.rs`:
  - `SIFR_E2E_RUNNER_MODE=legacy|new|compare`
  - Back-compat booleans (`SIFR_E2E_NEW_RUNNER`, `SIFR_E2E_LEGACY_RUNNER`) with conflict detection.
- Added full runner dispatcher in `test_e2e_pass` for legacy/new/compare.
- Added compare mode assertion in `compare_pass_reports(...)` to enforce exact pass/fail set equality.
- Preserved legacy failure and reporting semantics while adding grouped-run diagnostics:
  - per-fixture failures and statuses
  - group context in grouped build failures
  - deterministic sorted reporting.
- Added contract tests in `test_runner_mode_resolution`.
