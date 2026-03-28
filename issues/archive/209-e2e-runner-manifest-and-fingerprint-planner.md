## [Task] E2E Manifest and Dependency Fingerprint Planner

#### Current Situation
- Fixture handling is implicit inside test logic.
- Dependency grouping is not a first-class model.
- No planner exists to batch compatible fixtures together.

#### Desired Situation
- A clear data model exists for fixture discovery and compiled metadata.
- Dependency fingerprinting is deterministic and test-covered.
- Group planning can be consumed by later batch build stages.

#### Suggested Solution
- Introduce explicit structs:
  - `FixtureCase`
  - `CompiledCase`
  - `DependencyFingerprint`
  - `BatchGroup`
- Implement deterministic normalization and fingerprint hashing.
- Compute fingerprint from **effective dependencies**, not only compiler-reported metadata.
- Add unit tests for grouping determinism and edge cases.

#### Implementation Checklist
- Define structs and helper functions in `crates/sifr/tests/e2e.rs` or dedicated helper module.
- Normalize crate/dependency sets (sort + dedup).
- Replicate or unify implicit dependency detection currently done in `build_and_run_capture_with_deps` source-scanning logic (e.g. bigint/regex/rand/sha crates), and include those inferred deps in the fingerprint input.
- Include toolchain/version salt in fingerprint key.
- Write unit tests for:
  - same deps in different order produce same fingerprint
  - different deps produce different fingerprint
  - compiler-reported deps + inferred deps produce stable effective fingerprint
  - deterministic group ordering

#### Acceptance Criteria
- Group planner deterministically maps fixtures to groups.
- Fingerprint behavior is unit-tested.
- Planner output can be printed/debugged for inspection.

#### Dependencies
- Depends on Task 208.

### Implemented

- Added the full data model and helper types in `crates/sifr/tests/e2e.rs`:
  - `FixtureCase`, `CompiledCase`, `DependencyFingerprint`, `BatchGroup`
  - deterministic `dependency fingerprint` creation and hashing
- Added normalized dependency handling via `normalize_dependency_set` (`BTreeSet`) and `DependencyFingerprint::signature()`.
- Added inferred dependency enrichment in `infer_dependencies(...)` for:
  - bigint (`num_bigint`/`num-traits`)
  - regex/regex-like modules
  - rand/rand_distr
  - sha family crates
- Planner now groups by fingerprint in `plan_batches(...)` and sorts group output deterministically.
- Added tests in `test_dependency_fingerprint_and_cache_key_determinism`.
