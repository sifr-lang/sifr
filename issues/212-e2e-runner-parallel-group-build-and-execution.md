## [Task] Parallel Group Build and Execution Pipeline

#### Current Situation
- Even with grouping, a serial execution pipeline may remain too slow.
- Rust build and execution stages need careful concurrency control.

#### Desired Situation
- Groups build in parallel with bounded concurrency.
- Group execution also runs with safe parallel policy.
- System avoids rustc oversubscription and unstable performance.

#### Suggested Solution
- Add bounded worker pools for build and run stages:
  - `SIFR_E2E_RUST_JOBS`
  - `SIFR_E2E_RUN_JOBS`
- Add conservative defaults and optional tuning.
- Keep deterministic merged reporting order.

#### Implementation Checklist
- Parallelize group build stage with bounded workers.
- Parallelize group execution stage with bounded workers.
- Capture stage timing and top slow groups.
- Merge results in sorted deterministic order.
- Define grouped-build failure attribution contract:
  - mark all fixtures in failed group as failed
  - include group fingerprint, crate path, build log path
  - print deterministic fixture list for the failed group

#### Acceptance Criteria
- Group build and run are parallelized with worker caps.
- No correctness regressions under parallel mode.
- Deterministic report ordering is preserved.
- Group build failures are attributable to member fixtures with actionable diagnostics.

#### Dependencies
- Depends on Task 210 and Task 211.

### Implemented

- Added parallel bounded build + run stages in `crates/sifr/tests/e2e.rs`:
  - `SIFR_E2E_RUST_JOBS` drives parallel group builds in `build_batch_suite(...)`.
  - `SIFR_E2E_RUN_JOBS` drives parallel group execution in `run_batch_suite(...)`.
- Introduced deterministic merge + sorting of fixture-level outcomes in `run_batch_suite(...)`.
- Added run/build timing instrumentation:
  - suite-level timing in `run_new_pass_suite(...)`
  - top slow build/run group reporting for triage.
- Group build failures now emit actionable diagnostics in `run_batch_outcomes(...)`:
  - group fingerprint
  - group id / cache key context
  - crate path
  - build log path
  - fixture list for the failed group.
