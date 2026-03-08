# Ad Hoc Phase: Entrypoint Compilation Unification and Dependency Metadata Closure

Status: open (documented 2026-03-08)
Context: ad hoc planning phase captured in `issues/` before any roadmap-phase promotion

## Objective
Unify the compiler's build internals around one rooted-entrypoint compilation model, ensure dependency metadata is complete for both single-file and multi-file builds, and remove the current project-build manifest gap while preserving the existing CLI semantics contract.

## Depends on
- Phase 23 project graph and isolation correctness
- Phase 22 frontend mode parity hardening
- Phase 14 codegen metadata architecture

## Non-goals
- Changing the documented CLI mode-resolution contract.
- Treating every `.sifr` file as a project entrypoint by default.
- Changing `check` or `emit` semantics in this phase.
- Adding package-layout or multi-directory module semantics beyond the current local-project contract.
- Introducing fallback manifest inference from emitted Rust text when canonical metadata should come from compiler/codegen outputs.

## Scope
This ad hoc phase owns:
- one canonical internal build model for rooted entrypoint compilation
- unification of single-file and project build plumbing where behavior is meant to be identical
- multi-module aggregation of `used_stdlib_modules`
- multi-module aggregation of `required_crates`
- canonical Cargo manifest generation for both single-file and multi-file builds
- regression coverage for dependency-correct multi-file manifests
- explicit preservation of current CLI-visible mode behavior

This ad hoc phase does not own:
- CLI contract changes for mode selection
- forcing project mode for non-`main.sifr` entrypoints
- `check`/`emit` contract redesign
- package/import model expansion
- broader build-system or workspace feature work

## Execution Model
- This remains an ad hoc issue-driven phase until promoted into `.cursor/plans/main/phases/`.
- Work executes one milestone at a time.
- Internal architecture unification must land before manifest-generation cleanup is considered complete.
- CLI contract preservation is a hard gate for every milestone, not only the last one.
- No temporary dual-path dependency logic is allowed to remain at the end of this work.
- No string-scanning or post-hoc manifest inference is allowed where canonical compiler metadata should exist.

## Reviewer Gate
A milestone is not complete when the implementer believes the refactor is done.
A milestone is complete only when the reviewer explicitly confirms all of the following:
- the internal unification model is clear and simpler than the previous split
- dependency metadata is canonical and compiler-derived
- current CLI-visible semantics are preserved
- no unresolved manifest/dependency gap remains in multi-file builds
- no duplicate legacy path remains without justification
- implementation quality is production-grade and deterministic

## Internal Model Contract
- Single-file build is treated as the one-module case of the same rooted-entrypoint compilation architecture.
- Project build is treated as the reachable local import-closure case of that same architecture.
- CLI mode selection remains the boundary that chooses which rooted compilation shape to use.
- Dependency metadata must be aggregated from compiler/codegen outputs, not inferred from emitted Rust source.
- Cargo manifest generation must be driven by one canonical dependency-generation path.

## Milestones

### milestone_adhoc_1: Canonical Rooted Entrypoint Compilation Plan
- Scope:
  - Define and introduce one internal build-plan abstraction for rooted entrypoint compilation.
  - Express both current single-file and project builds through that abstraction.
  - Keep CLI-visible mode behavior unchanged.
- Definition of done:
  - Single-file and project build paths share one internal compilation-plan model.
  - The remaining distinction is input shape, not duplicated build architecture.

### milestone_adhoc_2: Multi-Module Dependency Metadata Aggregation
- Scope:
  - Extend multi-module codegen/build plumbing to return aggregate `used_stdlib_modules` and `required_crates`.
  - Ensure deterministic and closure-complete aggregation across reachable modules.
- Definition of done:
  - Multi-file builds produce compiler-derived dependency metadata equivalent in quality to single-file builds.

### milestone_adhoc_3: Canonical Manifest Generation Path
- Scope:
  - Route both single-file and multi-file builds through the same Cargo manifest generation logic.
  - Eliminate the current zero-dependency project-build manifest path.
- Definition of done:
  - `Cargo.toml` generation is dependency-correct and canonical for both build modes.

### milestone_adhoc_4: CLI Contract Preservation and Regression Hardening
- Scope:
  - Add regression coverage proving internal unification does not alter mode resolution or current user-visible semantics.
  - Cover non-`main` single-file behavior, unrelated sibling isolation, reachable-module errors, and current `check`/`emit` boundaries.
- Definition of done:
  - The refactor is externally behavior-preserving against the documented CLI contract.

### milestone_adhoc_5: Dependency Closure Regression Matrix
- Scope:
  - Add regression suites for multi-file dependency closure across:
    - stdlib dependencies introduced in imported support modules
    - intrinsic-required crates introduced outside `main`
    - transitive dependency closure across reachable modules
  - Ensure dependency regressions fail in local validation.
- Definition of done:
  - Multi-file Cargo dependency correctness is locally enforced and reproducible.

## Quality Contract

### Entry criteria
- Phase 23 exit gate is satisfied.
- Phase 22 mode-parity contract remains in force.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract; canonical/lossless `json` diagnostics; deterministic recovery ordering; and stable CLI exit-code behavior.

### Phase-wide invariants
- The documented single-file vs project CLI contract does not change in this phase.
- Dependency metadata is compiler-derived, not text-inferred.
- Manifest generation is deterministic.
- No user-visible dependency omission remains in multi-file builds.
- No fallback or compatibility shim remains at phase end.

### Milestone quality checks
- No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
- No lazy or partial fixes are allowed; each milestone must resolve root causes completely.
- All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards.
- Every milestone must include at least one positive-path and one negative-path validation case.
- Validation evidence must be recorded in the execution checklist issue before merge.
- No milestone is complete if it preserves duplicated architecture that the milestone claims to eliminate.

### Validation planning goals
- `milestone_adhoc_1`:
  - validation goals cover one canonical rooted-entrypoint build model and preservation of current build-mode semantics.
- `milestone_adhoc_2`:
  - validation goals cover aggregate multi-module `used_stdlib_modules` and `required_crates` collection with deterministic closure behavior.
- `milestone_adhoc_3`:
  - validation goals cover canonical manifest generation for both single-file and multi-file builds and removal of the zero-dependency project manifest path.
- `milestone_adhoc_4`:
  - validation goals cover preservation of current CLI mode resolution, unrelated sibling isolation, reachable-import failures, and `check`/`emit` boundaries.
- `milestone_adhoc_5`:
  - validation goals cover dependency closure regressions introduced from imported and transitive modules, with local matrix enforcement.
- Exit-gate evidence explicitly demonstrates:
  - one internal build architecture
  - dependency-correct manifests in multi-file builds
  - unchanged external CLI contract

## Local Validation Commands
- Full local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Quick local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Targeted driver tests:
  - `cargo test -p sifr_driver -- <test_name>`
- Targeted CLI tests:
  - `cargo test -p sifr -- <test_name>`
- Milestone demos:
  - `cargo run -q -p sifr -- run demos/<milestone_demo>.sifr`

## Exit Gate
- Single-file and project builds share one canonical internal rooted-entrypoint architecture.
- Multi-file builds generate correct Cargo dependencies from compiler-derived metadata.
- The documented CLI single-file vs project contract remains unchanged.
- Phase 27 non-regression contract remains green.
