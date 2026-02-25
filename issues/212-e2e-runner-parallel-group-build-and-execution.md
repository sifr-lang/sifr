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
