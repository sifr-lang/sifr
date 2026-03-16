# Ad Hoc Phase: Test Strategy and Validation Lane Redesign

Status: open (documented 2026-03-16)
Context: ad hoc planning phase captured in `issues/` before any roadmap-phase promotion
Execution readiness: planning-ready; promote into an execution checklist issue before implementation begins
Execution tracking: create `issues/phaseXX-test-strategy-and-validation-lane-redesign-execution.md` when execution begins

## Objective
Redesign the compiler test platform so local validation is fast, thermally reasonable, and highly cache-efficient while preserving or improving correctness coverage across compiler internals, CLI contracts, e2e behavior, determinism, hardening, and performance.

The goal of this phase is not "make tests a bit faster."
The goal is to replace the current over-reliance on expensive end-to-end execution with a layered, deliberate validation architecture where each invariant is enforced at the cheapest layer that can prove it.

This phase is explicitly local-first.
CI/CD symmetry is not the design driver for this document.
The primary optimization target is the developer experience on a normal local machine.

## Closure Status
- Status: open
- Closure evidence issue: create `issues/phaseXX-test-strategy-and-validation-lane-redesign-execution.md` when execution begins

## Why This Needs Its Own Phase
The current suite is strong on breadth but inefficient in architecture.
The problem is no longer "we need more tests."
The problem is that too many invariants are currently enforced through the most expensive validation path.

Current local validation conflates:
- fast developer feedback
- merge-gate confidence
- determinism research
- no-cache equivalence
- hardening corpus execution
- e2e throughput and cache behavior

Those are distinct jobs with distinct cost profiles.
They should not all execute in the same default local loop.

## Source of Truth
Primary planning and implementation inputs for this phase:
- `scripts/run_all_tests.sh`
- `scripts/run_e2e_pass.sh`
- `scripts/run_verification_hardening.py`
- `scripts/check_e2e_report_determinism.sh`
- `scripts/check_e2e_sequential_parallel_equivalence.sh`
- `scripts/run_frontend_mode_parity_matrix.sh`
- `scripts/run_phase23_graph_isolation_matrix.sh`
- `scripts/run_phase24_hir_analysis_consolidation_matrix.sh`
- `scripts/run_phase25_cfg_flow_activation_matrix.sh`
- `crates/sifr/tests/e2e.rs`
- `crates/sifr/src/main.rs`
- `crates/sifr_driver/src/build/workspace.rs`
- `crates/sifr_driver/src/test_runner/execution.rs`

## Current Validated Shape
The current top-level local gate is a serial wrapper in `scripts/run_all_tests.sh` that executes:
- maintainability guardrails
- `cargo test -p sifr -- --skip test_e2e_pass`
- frontend parity matrix
- phase 23 graph/isolation matrix
- phase 24 HIR analysis consolidation matrix
- phase 25 CFG/flow activation matrix
- e2e pass suite
- verification hardening suites

Warm timing measurements on the current tree:
- HIR maintainability guardrails: `0.15s`
- sifr_driver maintainability guardrails: `0.11s`
- `cargo test -p sifr -- --skip test_e2e_pass`: `8.73s`
- frontend parity matrix: `14.07s`
- phase 23 graph/isolation matrix: `25.45s`
- phase 24 HIR analysis consolidation matrix: `19.09s`
- phase 25 CFG/flow activation matrix: `12.76s`
- subtotal before main e2e and hardening: `80.10s`

Observed structural facts:
- the matrix scripts alone execute `48` separate `cargo run -q -p sifr -- ...` calls
- `quick` e2e caching is enabled for the main e2e pass, but determinism/equivalence checks later rerun e2e work
- verification hardening still includes broad work in `quick`, including determinism-scale and fuzz-smoke
- the quick e2e cache currently grows into a multi-gigabyte cache tree under `crates/sifr/target/sifr_e2e_cache/quick`

## Root-Cause Findings

### 1. `quick` is not actually a small developer loop
The current `quick` profile is not a lightweight smoke lane.
It is already a broad validation matrix with significant e2e and hardening work.

In practice, `quick` currently pays for:
- standard Rust/unit/integration test coverage
- shell-driven CLI parity matrices
- the main e2e pass lane
- verification hardening
- determinism-scale work inside verification hardening

That makes the "quick" label misleading and produces poor local ergonomics.

### 2. The same expensive e2e machinery is executed multiple times
The dominant cost is not one isolated slow command.
The dominant cost is repeated execution of the e2e runner family.

The current structure can trigger:
- one main `run_e2e_pass.sh` execution from `scripts/run_all_tests.sh`
- two additional `run_e2e_pass.sh` executions through deterministic report checking
- two more additional `run_e2e_pass.sh` executions through sequential-vs-parallel equivalence checking, with cache disabled

That means a single top-level validation loop can cause the e2e pass to run up to five times.
This is architecturally inappropriate for a default local lane.

### 3. Matrix suites overuse CLI/e2e execution for invariants that should be proven earlier
Many matrix checks validate:
- mode parity
- diagnostic parity
- repeat-run diagnostic stability
- project/test contract parity

Those invariants are valuable, but many are being enforced via repeated `cargo run -p sifr` executions on the same fixtures.
That means process startup, compiler startup, fixture reading, codegen, temporary workspace creation, and generated-Rust build work are paid repeatedly for assertions that are mostly contract-level rather than fully end-to-end semantic.

### 4. `run` and `test` cannot reuse generated-program build artifacts
The CLI `run` path allocates a fresh invocation workspace and deletes it afterward.
The test runner does the same for generated test projects.

Consequences:
- repeated `sifr run` on the same fixture does not materially improve
- repeated `sifr test` on the same fixture does not materially improve
- matrix scripts that invoke `check`, `build`, `run`, and `test` on the same project repeatedly are paying near-full rebuild cost for the `run`/`test` parts

This is a fundamental cache boundary problem, not just a shell-script problem.

### 5. The e2e cache only covers one expensive layer
The current e2e cache is real and persistent, but it is limited in scope.
It primarily caches built batch-group binaries.
It does not eliminate:
- fixture discovery
- Sifr compile/lowering planning work before batching
- all execution cost
- all determinism/no-cache lanes

So the cache helps, but it does not address the full cost stack.

### 6. Grouping imbalance likely creates long-tail critical paths
The observed quick-cache manifest shows heavily uneven group sizing, including one very large group.
Skewed batch sizes tend to create a bad tail:
- a few giant groups dominate total wall time
- workers go idle waiting for stragglers
- cache misses on oversized groups are disproportionately expensive

### 7. The current system is thermally aggressive by design
High CPU and laptop heat are expected outcomes of the current architecture:
- repeated Rust compilation
- repeated generated-program Cargo builds
- multiple concurrent e2e workers
- repeated runs of the same expensive validation family

This is not primarily a "parallelism tuning" problem.
It is a "wrong workload in the default lane" problem.

Local multi-core execution is acceptable and desirable in this phase as long as:
- it produces materially better wall time
- it stays within a thermally reasonable envelope for sustained development use
- it does not compensate for architectural waste by brute-force oversubscription

## What Must Be Preserved
Any redesign must keep the suite solid across all of these dimensions:
- parser correctness
- HIR lowering and type-system correctness
- ownership and borrow semantics
- control-flow and CFG correctness
- codegen correctness
- project graph and import-closure correctness
- CLI contract behavior
- diagnostics content, format, and stability
- project test-discovery behavior
- e2e runtime behavior
- determinism across runs
- equivalence across worker settings
- hardening against regressions and crashers
- cache invalidation correctness
- performance and throughput regression visibility

This phase is not allowed to narrow correctness scope.
It must preserve coverage while moving checks to cheaper and more appropriate layers.

## Design Principles

### 1. Prove each invariant at the cheapest valid layer
If an invariant can be proven with:
- a unit test
- a lowering snapshot
- a structured diagnostic contract test
- a generated-Rust snapshot

then it should not default to full e2e execution.

### 2. Keep e2e tests representative, not dominant
End-to-end execution is essential, but it should validate integration boundaries and representative semantics.
It should not be the default proof mechanism for every regression category.

### 3. Separate local signal, PR gates, nightly hardening, and release qualification
These are different lanes with different economics.
They should not all execute in the default local workflow.

For this phase, local lane design takes priority over CI/CD lane symmetry.
CI can be reconciled later after the local architecture is correct.

### 4. Make artifact reuse a first-class design constraint
Generated Rust and generated Cargo workspaces must be treated as reusable artifacts where fixture identity and environment identity permit reuse.

### 5. Prefer one structured harness over many shell-process loops
A single declarative harness can:
- reuse state
- reuse artifact directories
- issue one report
- avoid process churn
- centralize timing and cache accounting

That harness should be free to use multiple local CPUs when beneficial, but it must do so with explicit worker controls and a thermally aware default rather than "max everything all the time."

### 6. Preserve deterministic confidence without taxing every local run
Determinism, no-cache equivalence, and sequential-vs-parallel equivalence remain required quality gates.
They should not all run on every default local invocation.

## Target Validation Architecture

### Layer 0: Static and Structural Guardrails
Purpose:
- formatting
- linting
- maintainability guardrails
- configuration/schema validation

Characteristics:
- very fast
- always-on
- should complete in seconds

### Layer 1: Compiler-Internal Unit Tests
Purpose:
- parser behavior
- type-system rules
- ownership/borrow analysis
- CFG and reachability analysis
- codegen helper invariants
- diagnostic rendering and normalization

Characteristics:
- should prove the majority of semantic correctness cheaply
- should be the default place for root-cause regressions

### Layer 2: Pipeline Contract Integration Tests
Purpose:
- parse -> lower -> type-check contracts
- project-mode contract checks
- frontend mode parity
- normalized diagnostic parity
- generated Rust shape/snapshot parity

Characteristics:
- fixture-driven
- cheaper than e2e
- should replace much of the current shell-matrix repetition

### Layer 3: Representative E2E Execution
Purpose:
- validate the integrated compiler pipeline plus Rust build plus runtime execution
- validate a curated set of representative positive and negative programs

Characteristics:
- intentionally small for PR/local gates
- broad enough to catch true integration failures

### Layer 4: Hardening and Determinism
Purpose:
- fixed bug corpus
- crash corpus
- property-style repeatability checks
- fuzz smoke
- OSS sample projects
- repeated-run determinism
- no-cache equivalence
- sequential-vs-parallel equivalence

Characteristics:
- broad
- expensive
- should default to nightly/release or explicitly requested lanes

### Layer 5: Throughput and Resource Regression
Purpose:
- runtime throughput budgets
- cache efficiency budgets
- group skew detection
- artifact-size growth monitoring
- lane wall-time regression alerts

Characteristics:
- performance-focused
- not a default local gate

## Recommended Lane Redesign

### `quick`
Purpose:
- fast local developer confidence

Target characteristics:
- ideally `2-5` minutes on a normal laptop
- thermally moderate even when using multiple cores
- no repeated e2e families

Recommended contents:
- Layer 0 guardrails
- Layer 1 unit/integration tests
- a small Layer 2 contract suite
- a very small representative Layer 3 e2e sample

Explicitly exclude from default `quick`:
- full verification hardening
- determinism-scale
- sequential-vs-parallel no-cache equivalence
- throughput benchmarking
- broad OSS and fuzz lanes

### `pr`
Purpose:
- authoritative merge gate

Recommended contents:
- all Layer 0 and Layer 1
- full Layer 2 contract coverage
- representative Layer 3 e2e lane
- a selected subset of Layer 4 hardening sufficient for merge confidence

### `nightly`
Purpose:
- breadth, drift detection, and regression discovery

Recommended contents:
- full hardening
- full determinism coverage
- sequential-vs-parallel equivalence
- no-cache checks
- broader OSS sample coverage
- fuzz smoke

### `release`
Purpose:
- highest-confidence qualification gate

Recommended contents:
- nightly plus stress-mode and comparison-mode validation
- throughput and cache-efficiency gates
- full end-to-end validation across the broadest supported corpus

Lane note:
- the exact CI/CD mapping for these lanes is not important in this phase
- what matters first is that the local lanes are architecturally correct, fast, and thermally sane

## Recommended Replacement For The Current Matrix Scripts
The current matrix scripts should be replaced by one declarative validation harness rather than maintained as many shell scripts.

The target harness should express, per fixture:
- fixture path
- command modes to validate
- expected exit code
- expected stdout/stderr or diagnostic fingerprints
- parity relationships across modes
- stability expectations across repeated runs
- whether full runtime execution is required or whether compile/check/emit is sufficient

Benefits:
- shared process and state reuse
- fewer shell processes
- one timing report
- one place to add new fixture metadata
- easier cache-aware execution planning
- easier lane partitioning

## Cache Strategy Redesign

### Current Limitation
The existing e2e cache is useful but only covers batch Rust-build reuse.
That is too narrow to solve the overall problem.

### Target Cache Layers

#### A. Frontend Compilation Cache
Keyed by:
- fixture source hash
- import/dependency closure fingerprint
- compiler semantic version or schema version

Stores:
- parsed/lowered/typed intermediate result or a canonical compiled fixture representation

Purpose:
- avoid recompiling the same source through the Sifr frontend in repeated validation flows

#### B. Generated Rust Cache
Keyed by:
- frontend compilation result fingerprint
- codegen configuration
- relevant stdlib/dependency metadata

Stores:
- normalized generated Rust source
- metadata describing required crates and stdlib surfaces

Purpose:
- avoid regenerating identical Rust for repeated checks across modes/lane executions

#### C. Generated Program Build Cache
Keyed by:
- generated Rust fingerprint
- dependency set
- Rust toolchain/environment signature

Stores:
- reusable Cargo workspace/build output
- executable path
- build logs when needed

Purpose:
- eliminate repeated generated-program rebuilds for `run` and `test`

#### D. Optional Execution Cache
Use only for fixtures explicitly marked pure and deterministic.

Purpose:
- skip repeated execution when the contract is already fully represented by known deterministic output

Constraint:
- this must be opt-in and carefully scoped; do not hide runtime regressions behind aggressive output memoization

### Workspace Reuse Requirement
Invocation-scoped temp workspaces are appropriate for isolation, but they are the wrong default for repeated validation of unchanged fixtures.

The end-state should support:
- content-addressed reusable workspaces for stable fixture builds
- temporary isolated workspaces only where required by the contract under test

## Coverage Strategy By Invariant Type

### Diagnostics
Prefer:
- normalized structured diagnostic snapshots
- contract tests for parity across output formats

Avoid:
- repeated `check`/`build`/`run` full CLI loops when the invariant is only diagnostic consistency

### Lowering and Analysis
Prefer:
- unit and integration tests over HIR/analysis outputs
- snapshots for canonical graph/flow shapes

Avoid:
- proving CFG or HIR invariants only through fully executing generated programs

### Codegen
Prefer:
- generated Rust snapshots
- focused compile-only validation
- representative runtime execution for integration confidence

Avoid:
- using runtime execution as the only proof of codegen shape correctness

### CLI and Project Mode Contracts
Keep:
- a representative number of true CLI tests

Move most parity checks to:
- declarative contract fixtures
- integration harness assertions over normalized outputs and exit codes

### Determinism
Keep:
- repeated-run checks
- sequential-vs-parallel checks
- no-cache checks

But run them in:
- `nightly`
- targeted `pr` subsets when a touched area justifies it

### Hardening
Keep:
- fixedbugs
- crashes
- fuzz smoke
- OSS sample projects

But treat them as:
- broad safety discovery lanes
- not default local-loop work

## Metrics That Must Become First-Class
This phase should make the following metrics visible and tracked:
- wall time per lane
- wall time per suite
- CPU time per lane
- e2e compile/build/run breakdown
- cache hit rate per cache layer
- count of rebuilt artifacts per run
- slowest fixtures and slowest groups
- group-size skew and tail behavior
- total cache size and file count
- flake rate and rerun rate
- no-cache versus warm-cache delta
- thermal proxy metrics where practical

Without these measurements, future slowness will only be rediscovered anecdotally.

## Non-goals
- no correctness regression in exchange for speed
- no removal of determinism or hardening coverage
- no fallback/shim architecture where old and new validation systems permanently coexist without a planned transition
- no silent weakening of developer guarantees
- no broad architectural changes to compiler semantics in this phase
- no optimization plan that depends on CI/CD-specific behavior to feel acceptable locally

## Risks

### Risk: Accidentally reducing coverage while speeding up the loop
Mitigation:
- make invariant ownership explicit by layer
- require traceability from removed expensive checks to new cheaper proofs

### Risk: Cache complexity introduces stale or invalid reuse
Mitigation:
- strict cache keys
- schema-versioned invalidation
- explicit no-cache lanes remain mandatory

### Risk: One new harness becomes a new monolith
Mitigation:
- modular harness design
- manifest-driven execution model
- maintainability guardrails for test infrastructure

### Risk: Developers continue using the wrong lane
Mitigation:
- redefine lane semantics clearly
- make `quick` genuinely quick
- reserve broad hardening for `nightly` and explicit opt-in commands

## Execution Model
This remains an ad hoc planning phase until promoted into an execution issue or internal phase document.

Work should proceed in this order:
1. define lane semantics and target contents
2. replace shell-matrix repetition with one declarative harness
3. move invariants downward into unit and integration layers where appropriate
4. introduce reusable generated-program artifact caching
5. move determinism/no-cache equivalence out of the default `quick` lane
6. add measurement and regression reporting
7. rebalance e2e grouping and worker defaults based on observed tail behavior

No execution milestone is complete if it only improves wall time by weakening validation.

## Recommended Milestones

### milestone_test_1: Lane Taxonomy and Policy Redesign
Scope:
- redefine `quick`, `pr`, `nightly`, and `release`
- classify every current validation step into the correct lane
- remove hardening/determinism overreach from the default local lane
- define explicit local worker and thermal policy for each lane

Definition of done:
- each suite has one intended lane
- `quick` has an explicit time-budget target
- expensive repeated e2e work is no longer part of the default local loop unless intentionally justified
- local multi-core defaults are intentional and bounded rather than incidental

### milestone_test_2: Declarative Validation Harness
Scope:
- replace the current shell-matrix scripts with one manifest-driven harness
- unify mode parity, diagnostic parity, repeatability, and fixture expectations in one system

Definition of done:
- shell-driven repetition is retired or reduced to minimal wrappers
- fixture contracts are defined declaratively
- one unified timing report is emitted

### milestone_test_3: Invariant Downshifting
Scope:
- audit current expensive checks
- move eligible invariants from e2e/CLI execution into unit or integration coverage
- add snapshots/contract tests where needed

Definition of done:
- expensive runtime execution is no longer the default proof for diagnostic and lowering invariants
- every removed expensive check has equivalent or stronger cheaper coverage

### milestone_test_4: Artifact Reuse and Cache Boundary Redesign
Scope:
- introduce reusable generated-program workspaces and artifact caching
- reuse stable outputs for repeated `run` and `test` validation
- extend cache accounting and invalidation reporting

Definition of done:
- repeated validation of unchanged fixtures materially improves
- `run`/`test` no longer always pay full generated-program rebuild cost
- cache-hit metrics are visible

### milestone_test_5: Hardening Lane Refactor
Scope:
- keep hardening broad but partition it into nightly/release lanes
- preserve determinism and no-cache equivalence checks outside the default local loop

Definition of done:
- broad hardening remains covered
- `quick` no longer re-executes e2e families multiple times through nested hardening flows

### milestone_test_6: Throughput and Resource Governance
Scope:
- add lane budgets
- track cache size, tail groups, slow fixtures, and no-cache deltas
- add regression alarms for throughput and cache health
- add local worker/thermal guidance and measure the wall-time tradeoff of different worker defaults

Definition of done:
- performance regressions are detectable from local reports
- cache growth and group skew are no longer invisible
- worker defaults are chosen for local wall-time improvement without unacceptable sustained heat

## Reviewer Gate
A milestone is complete only when the reviewer explicitly confirms:
- coverage did not narrow
- invariants are assigned to the correct validation layer
- the new lane boundaries are technically coherent
- cache keys and invalidation rules are defensible
- the resulting developer workflow is materially faster and clearer
- no duplicate legacy validation path remains without an explicit deprecation plan

## Closure Criteria
This ad hoc phase is complete only when all of the following are true:
- `quick` is a genuinely fast local lane with a bounded target runtime
- broad hardening and determinism remain covered in non-default lanes
- matrix-style contract validation is harness-driven rather than shell-loop-driven
- repeated `run`/`test` validation can reuse stable generated-program artifacts
- cache hit rates and resource costs are measurable and reported
- no critical correctness domain has lost coverage
- developer guidance clearly distinguishes local, PR, nightly, and release validation

## Expected Outcome
If executed correctly, this phase should deliver:
- materially faster local validation
- lower CPU and thermal load during normal development
- better cache leverage
- clearer lane semantics
- stronger confidence in where each invariant is actually proven
- a test platform that scales with the compiler rather than fighting it
