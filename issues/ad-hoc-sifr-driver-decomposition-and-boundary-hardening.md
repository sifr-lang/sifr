# Ad Hoc Phase: sifr_driver Decomposition and Boundary Hardening

Status: open (documented 2026-03-10)
Context: ad hoc planning phase captured in `issues/` before any roadmap-phase promotion
Execution readiness: planning-ready; promote into an execution checklist issue before implementation begins
Execution tracking: `issues/phase31-sifr-driver-decomposition-and-boundary-hardening-execution.md`

## Objective
Break `crates/sifr_driver/src/lib.rs` into small, focused modules with explicit internal boundaries while preserving all current CLI-visible behavior, diagnostics, dependency behavior, and test contracts.

## Closure Status
- Status: open
- Closure evidence issue: create `issues/phaseXX-sifr-driver-decomposition-and-boundary-hardening-execution.md` when execution begins

## Depends on
- Phase 22 frontend mode parity hardening
- Phase 23 project graph and isolation correctness
- Phase 27 runtime safety and diagnostics stability
- Hard sequencing requirement: this phase does not begin until `issues/ad-hoc-entrypoint-compilation-unification-and-dependency-metadata-closure.md` reaches closure, because both efforts touch the same driver core and must not create competing internal abstractions

## Non-goals
- No CLI contract changes.
- No dependency metadata redesign in this phase.
- No manifest-generation behavior changes in this phase.
- No import/package model changes.
- No project/test discovery contract changes.
- No fallback/shim architecture left behind after extraction.

## Scope
This ad hoc phase owns:
- decomposition of `sifr_driver` into small internal modules
- stabilization of driver internal responsibilities and ownership boundaries
- preservation of the current public `sifr_driver` API
- extraction of inline tests into focused test modules
- maintainability guardrails for the driver crate

This ad hoc phase does not own:
- new compiler behavior
- frontend semantic changes
- build/test CLI semantics changes
- project discovery contract changes
- dependency-closure feature work
- broader driver architecture redesign beyond structural decomposition

## Current Validated Shape
- `crates/sifr_driver/src/lib.rs` is currently a single 3498-line file.
- The current file mixes these responsibility clusters:
  - stdlib embedding, intrinsic mapping, cache, and stdlib bootstrap
  - compile error and diagnostic types plus recovery limiting
  - single-file frontend compile/check plumbing
  - project lowering and cross-module export collection
  - dependency graph ordering and cycle diagnostics
  - project/test discovery and import-closure parsing
  - invocation-scoped workspace allocation and cleanup
  - single-file build and multi-file project build orchestration
  - test runner orchestration and Cargo manifest generation
  - a large embedded unit-test module

## Target Shape
Recommended end-state:

`crates/sifr_driver/src/`
- `lib.rs`
- `diagnostics.rs`
- `stdlib/`
- `frontend/`
- `project/`
- `build/`
- `test_runner/`
- `tests/`

Suggested boundary split:
- `diagnostics.rs`
  - `CompileError`, `CompilePhase`, diagnostic rendering/model types, recovery limiting
- `stdlib/`
  - embedded stdlib registry, intrinsic mapping, stdlib compilation cache, stdlib export/bootstrap logic
- `frontend/`
  - `parse_source`, `lower_source`, single-file frontend compile/check path, module export collection
- `project/`
  - dependency graph, cycle canonicalization, project/test discovery, import-closure parsing, workspace allocation helpers
- `build/`
  - single-file build, project build, Cargo project generation/write helpers
- `test_runner/`
  - test discovery orchestration, test lib composition, test-runner Cargo manifest helpers
- `tests/`
  - focused test modules grouped by concern rather than one giant `mod tests`

## Execution Model
- This remains an ad hoc issue-driven phase until promoted into `internal_docs/phases/`.
- Work executes one milestone at a time.
- `issues/ad-hoc-entrypoint-compilation-unification-and-dependency-metadata-closure.md` must be closed before execution starts for this phase.
- Public behavior preservation is a hard gate for every milestone, not only the last one.
- No milestone is complete if it merely redistributes complexity into another oversized file.
- No fallback, compatibility shim, or temporary parallel architecture is allowed to remain at phase end.
- The decomposition should proceed from lowest-risk seams to higher-churn orchestration seams so merge risk stays reviewable.

## Reviewer Gate
A milestone is not complete when the implementer believes the extraction is done.
A milestone is complete only when the reviewer explicitly confirms all of the following:
- the extracted boundary is coherent and simpler than the prior monolith
- ownership and responsibility boundaries are explicit
- public API and CLI-visible behavior are preserved
- no duplicate legacy path remains without justification
- tests remain understandable and correctly grouped around the extracted seams
- implementation quality is production-grade and deterministic

## Internal Boundary Contract
- `lib.rs` becomes a small crate entrypoint with module declarations and re-exports only.
- Each extracted module owns one coherent responsibility.
- Public `sifr_driver` entrypoints stay stable unless a separate behavior phase explicitly changes them.
- No new monolithic replacement files are allowed.
- Tests should live beside the concern they validate or in focused test modules, not as one giant `mod tests`.
- Shared helpers should be extracted only when a real boundary exists; this phase does not permit vague `utils.rs` dumping grounds.

## Stable Public API Contract
The following public crate surface is treated as stable for this phase and must continue to resolve from the crate root:
- public functions:
  - `compile`
  - `compile_with_metadata`
  - `check`
  - `parse_source`
  - `lower_source`
  - `type_check_source`
  - `build`
  - `build_project`
  - `check_project`
  - `run_tests`
  - `compile_errors_to_diagnostics`
  - `apply_diagnostic_recovery_limits`
- public result and diagnostic types:
  - `CompileResult`
  - `CompileResultFull`
  - `CompileError`
  - `CompilePhase`
  - `Severity`
  - `SuggestionKind`
  - `DiagnosticSpan`
  - `RelatedSpan`
  - `DiagnosticChild`
  - `DiagnosticSuggestion`
  - `CompilerDiagnostic`
- public re-exports currently depended on externally:
  - `LoweringStats`

The following are implementation details and may become internal or move freely during decomposition as long as behavior is preserved:
- stdlib bootstrap/cache helpers
- frontend/internal lowering helpers
- project graph/discovery/workspace helpers
- build/test-runner assembly helpers
- internal structs such as `StdlibCompiled`, `FrontendCompiled`, `ProjectLowering`, and related helper enums

## Driver-Codegen Boundary Contract
- Extracted `sifr_driver` modules may call `sifr_codegen` directly; this phase does not introduce a driver-local re-export layer for codegen internals.
- Ownership of codegen calls remains explicit:
  - stdlib bootstrap owns `generate_rust_with_stdlib`
  - single-file compile path owns `generate_rust_with_stdlib` and `generate_rust_with_metadata` usage where applicable
  - multi-module project build/test support owns `generate_rust_multi`
  - test-runner test-module lowering owns `generate_rust_test`
  - Cargo manifest generation remains owned by build/test-runner modules through `generate_project` and `generate_project_with_deps_and_crates`
- Discovery, graph, and diagnostics modules must not accumulate Cargo-manifest or codegen-assembly responsibilities.

## Test Decomposition Plan
The current embedded test block should be split into focused modules aligned with extracted responsibilities.

Recommended test-module boundaries:
- `tests/panic_boundary.rs`
  - panic boundary conversion and internal compiler failure handling
- `tests/diagnostics.rs`
  - diagnostic conversion, codes/URLs, recovery limiting, and renderer-adjacent invariants exposed by the driver
- `tests/single_file_frontend.rs`
  - parse/lower/type-check/compile behavior for single-file paths
- `tests/stdlib_bootstrap.rs`
  - stdlib cache reuse, stdlib export/bootstrap invariants
- `tests/project_graph.rs`
  - dependency graph ordering, deterministic cycle diagnostics, module export behavior
- `tests/discovery_and_workspace.rs`
  - import-closure discovery, project/test parity, invocation workspace isolation
- `tests/project_build_check.rs`
  - `build_project` / `check_project` parity and error-surface behavior
- `tests/test_runner.rs`
  - `run_tests`, composed test lib generation, test-runner Cargo manifest behavior

Coverage-preservation rule:
- each extracted implementation seam must move with its nearest regression coverage in the same milestone or an immediately adjacent milestone
- no milestone may delete or temporarily orphan an existing regression area while waiting for a later cleanup pass

## Maintainability Guardrail Contract
Milestone `milestone_driver_6` must add a checked-in guardrail system modeled after the existing HIR precedent.

Required enforcement artifacts:
- `scripts/check_sifr_driver_maintainability_guardrails.py`
- `internal_docs/sifr_driver_maintainability_guardrails.md`
- wiring in `scripts/run_all_tests.sh`

Initial hard requirements for the guardrail:
- `crates/sifr_driver/src/lib.rs` must be an entrypoint/re-export file and stay at or below 250 lines
- any `mod.rs` under `crates/sifr_driver/src/` must stay at or below 250 lines
- any non-test implementation file under `crates/sifr_driver/src/` must stay at or below 900 lines
- any file under `crates/sifr_driver/src/tests/` must stay at or below 700 lines
- banned monoliths at phase end:
  - `crates/sifr_driver/src/lib.rs` as an implementation monolith
  - `crates/sifr_driver/src/stdlib.rs`
  - `crates/sifr_driver/src/frontend.rs`
  - `crates/sifr_driver/src/project.rs`
  - `crates/sifr_driver/src/build.rs`
  - `crates/sifr_driver/src/test_runner.rs`

Required checklist/documentation coverage:
- guardrail doc must include a review checklist for placing new logic in the correct module
- guardrail doc must require local script execution before merge
- guardrail script must support a negative-path validation override similar to the existing HIR guardrail workflow

## Milestones

### milestone_driver_1: Public API Spine and Diagnostic Extraction
- Scope:
  - Reduce `lib.rs` to crate wiring plus public re-exports.
  - Extract diagnostics/error/public result types into a dedicated module.
  - Preserve the current public `sifr_driver` crate surface.
- Definition of done:
  - `lib.rs` is no longer the implementation home for diagnostics and public result types.
  - Public compile/check/build/test-facing entrypoints still resolve from the crate root.
  - No behavior changes are introduced in this milestone.

### milestone_driver_2: Stdlib Bootstrap Extraction
- Scope:
  - Extract embedded stdlib registry, intrinsic constant mapping, stdlib cache, and stdlib compilation/bootstrap flow.
  - Keep stdlib export and metadata behavior unchanged.
- Definition of done:
  - Stdlib bootstrap has one coherent home.
  - Cache and stdlib export behavior are preserved.
  - No driver-visible behavior changes are introduced in this milestone.

### milestone_driver_3: Frontend and Project-Graph Extraction
- Scope:
  - Extract frontend compile/check plumbing.
  - Extract module export collection.
  - Extract dependency graph construction, compile ordering, and cycle canonicalization.
  - Extract project frontend analysis helpers without changing semantics.
- Definition of done:
  - Frontend and project-graph logic are separated into coherent modules.
  - Phase 22 and Phase 23 behavior contracts remain intact.
  - No semantic or contract changes are introduced in this milestone.

### milestone_driver_4: Discovery, Workspace, and Build Orchestration Extraction
- Scope:
  - Extract project/test discovery, import-closure parsing, workspace allocation, file-write helpers, and build orchestration.
  - Preserve single-file and multi-file build behavior.
- Definition of done:
  - Discovery/workspace/build concerns are no longer mixed into frontend lowering logic.
  - Single-file and project build entrypoints remain behavior-preserving.
  - No CLI-visible changes are introduced in this milestone.

### milestone_driver_5: Test Runner Extraction
- Scope:
  - Extract `run_tests`, test-runner Cargo manifest generation, and combined test-lib assembly into a dedicated module.
  - Preserve project/test parity behavior and dependency handling.
- Definition of done:
  - Test-runner orchestration is isolated from normal build flow.
  - Existing test-runner behavior and diagnostics are preserved.
  - No CLI-visible changes are introduced in this milestone.

### milestone_driver_6: Test Suite Decomposition and Maintainability Guardrail
- Scope:
  - Split the embedded `mod tests` into focused test modules grouped by concern.
  - Add an enforceable maintainability guardrail for `sifr_driver` structure to prevent re-monolithization.
- Definition of done:
  - No giant in-file test block remains in `lib.rs`.
  - Driver test organization mirrors the extracted module boundaries.
  - The crate has an explicit maintainability guardrail script, documentation, and local-validation wiring that would fail if the driver collapses back into a monolith.

## Milestone Evidence Artifacts
- `milestone_driver_1`
  - crate-root spine with public re-exports
  - diagnostics module extraction evidence
- `milestone_driver_2`
  - stdlib module tree and preserved stdlib regression evidence
- `milestone_driver_3`
  - frontend/project module tree and preserved Phase 22/23 regression evidence
- `milestone_driver_4`
  - extracted discovery/workspace/build module tree and preserved build-path regression evidence
- `milestone_driver_5`
  - extracted test-runner module and preserved project/test parity evidence
- `milestone_driver_6`
  - focused test-module tree
  - `scripts/check_sifr_driver_maintainability_guardrails.py`
  - `internal_docs/sifr_driver_maintainability_guardrails.md`
  - maintainability guardrail enforcement evidence

## Quality Contract

### Entry criteria
- Phase 22 exit gate is satisfied.
- Phase 23 exit gate is satisfied.
- Phase 27 non-regression baseline is green at phase start.
- `issues/ad-hoc-entrypoint-compilation-unification-and-dependency-metadata-closure.md` is closed before this phase begins.

### Baseline evidence
- Phase 22 completion evidence exists in `issues/phase22-frontend-mode-parity-hardening-execution.md`.
- Phase 23 completion evidence exists in `issues/phase23-project-graph-and-isolation-correctness-execution.md`.
- Phase 27 completion and full-suite green evidence exist in `issues/phase27-runtime-safety-and-diagnostics-execution.md`.
- When this ad hoc phase is promoted into active execution, the current full-suite result must be recorded again at phase start as the working non-regression baseline for this effort.

### Phase-wide invariants
- No user-triggerable panic paths.
- No data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths.
- Stable diagnostic contract remains intact.
- Canonical and lossless `json` diagnostics remain authoritative.
- Recovery ordering remains deterministic.
- Exit-code and CLI contract remain stable.
- Public `sifr_driver` entrypoints remain behavior-preserving during decomposition.
- The stable public API listed in this document remains available from the crate root throughout the phase.
- No new driver replacement monolith is allowed to emerge in another file or directory.

### Milestone quality checks
- No fallback, migration, or legacy compatibility code is allowed; implement the canonical structure directly with clean code only.
- No lazy or partial extraction is allowed; each milestone must resolve the owned structural boundary completely.
- All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and clear ownership boundaries.
- Every milestone must include at least one positive-path and one negative-path validation case.
- Validation evidence must be recorded in the execution checklist issue before merge.
- No milestone is complete if it leaves a replacement monolith behind.

### Validation planning goals
- `milestone_driver_1`:
  - validation goals cover public API preservation and diagnostic behavior preservation.
- `milestone_driver_2`:
  - validation goals cover stdlib bootstrap/cache behavior preservation.
- `milestone_driver_3`:
  - validation goals cover frontend parity, project graph ordering, and cycle-diagnostic behavior preservation.
- `milestone_driver_4`:
  - validation goals cover discovery, workspace isolation, and build-path behavior preservation.
- `milestone_driver_5`:
  - validation goals cover test-runner parity and dependency-handling preservation.
- `milestone_driver_6`:
  - validation goals cover test decomposition and maintainability guardrail enforcement.
- Exit-gate evidence explicitly demonstrates:
  - `lib.rs` is a thin entrypoint rather than the implementation
  - driver responsibilities are decomposed into coherent modules
  - external behavior is unchanged
  - the crate cannot easily regress back into a monolith

## Local Validation Commands
- Baseline revalidation at execution start:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Driver-specific smoke checks at execution start:
  - `cargo test -q -p sifr_driver -- --test-threads=1`
  - `cargo test -q -p sifr_driver test_check_project_error_messages_match_build_project`
  - `cargo test -q -p sifr_driver test_run_tests_resolves_local_imports_and_constants`
  - `cargo test -q -p sifr_driver test_compute_module_compile_order_is_deterministic_across_hashmap_insertion_order`
  - `cargo test -q -p sifr test_frontend_error_messages_match_across_check_build_and_run_paths`
  - `cargo test -q -p sifr test_runner_mode_resolution`
- Quick local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Targeted driver tests:
  - `cargo test -p sifr_driver -- <test_name>`
- Targeted CLI tests:
  - `cargo test -p sifr -- <test_name>`
- Milestone demos:
  - `cargo run -q -p sifr -- run demos/<milestone_demo>.sifr`

## Exit Gate
- `crates/sifr_driver/src/lib.rs` is a small entrypoint, not the implementation.
- Driver responsibilities are decomposed into coherent modules with clear ownership.
- Existing CLI-visible behavior is unchanged.
- Phase 22, Phase 23, and Phase 27 contracts remain green.
- A maintainability guardrail exists so the driver crate does not collapse back into a monolith.
