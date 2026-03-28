## [Task] Implement Batch Crate Generation per Dependency Group

#### Current Situation
- Build orchestration is still effectively per fixture.
- Repeated crate setup/build overhead dominates runtime.

#### Desired Situation
- One generated Rust crate per dependency fingerprint group.
- Group crate can execute all its member fixtures.
- Symbol collisions and naming conflicts are eliminated.

#### Suggested Solution
- For each `BatchGroup`, generate:
  - `Cargo.toml` once
  - `src/main.rs` dispatcher
  - one module/function namespace per fixture
- Execute cases via `--case <fixture_name>` argument.

#### Implementation Checklist
- Implement stable sanitized module/function naming.
- Transform each fixture's generated `fn main()` into a namespaced callable (e.g., `fixture_<name>_main`) so batch dispatcher owns the crate-level `main`.
- Generate dispatcher that calls one fixture at a time.
- Generate group `Cargo.toml` from the **union** of all member fixtures' effective dependencies.
- Preserve old output comparison contract per fixture.
- Validate generated code compiles for multi-case groups.
- Document that this task implements generation logic in isolation; integration with parallel compile outputs from Task 210 is wired in Task 212.

#### Acceptance Criteria
- Group crate generation works for mixed fixture names.
- No symbol collisions across grouped fixtures.
- Per-fixture execution inside batch crate is correct.

#### Dependencies
- Depends on Task 209.

### Implemented

- Implemented per-group source generation in `crates/sifr/tests/e2e.rs`:
  - `build_group_sources(...)` builds one `BatchGroup` source payload per dependency cluster.
  - `build_rust_source_from_module(...)` rewrites each generated fixture `fn main(...)` into a namespaced callable.
  - Stable fixture/module naming via `fixture_module_name(...)` and `sanitize_identifier(...)`.
  - Group `main` dispatches fixtures by `--case <fixture_name>`.
- Added union crate manifest generation via `generate_cargo_toml(...)` from all group dependency inputs.
- Kept per-fixture output comparison by invoking each fixture entry and mapping stdout back to fixture identity in `run_batch_outcomes(...)`.
