## [Task] Parallelize Sifr Compile Stage

#### Current Situation
- Sifr compile phase is currently serialized across many fixtures.
- Profiling shows this phase is a major share of total time.

#### Desired Situation
- Fixture compile through `sifr_driver::compile_with_metadata` runs in parallel.
- Worker count is bounded and configurable.
- Failure aggregation semantics remain unchanged.

#### Suggested Solution
- Implement a bounded parallel worker model (Rayon or scoped thread pool).
- Add env-configurable knob `SIFR_E2E_SIFR_JOBS`.
- Preserve deterministic output ordering by sorting post-collection.

#### Implementation Checklist
- Add parallel compile executor over discovered fixtures.
- Collect successes and failures without early exit.
- Re-sort final compiled case list by fixture name/path for stable reports.
- Add timing instrumentation for this phase.

#### Acceptance Criteria
- Compile stage runs in parallel with configurable worker count.
- Output ordering remains deterministic.
- Correctness equivalent to pre-change behavior.

#### Dependencies
- Depends on Task 209.

### Implemented

- Added bounded parallel compile stage in `crates/sifr/tests/e2e.rs`:
  - `compile_suite_parallel(...)` calls `run_in_parallel(...)` with `SIFR_E2E_SIFR_JOBS`.
  - Worker defaults to available logical CPU count and clamps to at least 1.
- Kept failure aggregation semantics by collecting compile results as `(FixtureCase, Result<CompiledCase, String>)`.
- Preserved deterministic result ordering by sorting outputs by fixture index and by name where needed.
- Timed and logged compile phase duration in new suite pipeline.
