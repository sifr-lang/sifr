# Ad Hoc Phase Execution Checklist (Entrypoint Compilation Unification and Dependency Metadata Closure)

Status: complete (started 2026-03-10; external review passes completed on 2026-03-10)
Owner: ad_hoc_entrypoint_compilation_unification execution loop
Reference planning doc:
- `issues/ad-hoc-entrypoint-compilation-unification-and-dependency-metadata-closure.md`

Loop per part: Plan -> Implement -> Validate -> Demo -> PR -> Review -> Merge -> Update docs -> Next part

## Global Gates
- [x] Scope remains constrained to the active milestone part
- [x] Root cause addressed without fallback or compatibility shims
- [x] CLI command semantics contract remains unchanged
- [x] Positive-path and negative-path validation recorded for the active milestone
- [x] Milestone demo runs successfully before the PR is opened
- [x] Local validation is run before the PR is opened
- [x] PR is opened, reviewed, and merged before the next milestone starts
- [x] Docs/checklists/PR links are updated before moving on

## Full Phase To-Do Plan
1. [x] `milestone_adhoc_1`: introduce one canonical rooted-entrypoint compilation plan and shared build materialization for single-file and project builds
2. [x] `milestone_adhoc_2`: aggregate multi-module `used_stdlib_modules` and `required_crates` deterministically from compiler/codegen outputs
3. [x] `milestone_adhoc_3`: route single-file and multi-file manifest generation through one canonical dependency-driven path
4. [x] `milestone_adhoc_4`: harden CLI contract preservation regressions around mode boundaries and unchanged `check`/`emit` semantics
5. [x] `milestone_adhoc_5`: add dependency-closure regression matrix coverage for imported and transitive dependency sources

## Baseline Revalidation
- 2026-03-10: full local validation started with `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Completed before interruption: unit tests, frontend mode parity matrix, phase 23 matrix, phase 24 matrix, phase 25 matrix, and e2e pass suite all passed
- Remaining work: rerun the full suite to capture completed verification-hardening evidence without interruption

## Milestone Progress

### milestone_adhoc_1: Canonical Rooted Entrypoint Compilation Plan
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1082
- Implementation target:
  - add one rooted-entrypoint driver plan abstraction
  - route both single-file and project build orchestration through it
  - share generated-project materialization/build execution
- Demo target:
  - `cargo run -q -p sifr -- run demos/m_adhoc_1_rooted_entrypoint_compilation_demo/main.sifr`
- Validation target:
  - `cargo test -p sifr_driver --lib rooted_entrypoint`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -p sifr_driver --lib rooted_entrypoint` -> passed (`3 passed, 0 failed`)
  - positive path: `cargo run -q -p sifr -- run demos/m_adhoc_1_rooted_entrypoint_compilation_demo/main.sifr` -> printed `adhoc milestone 1 rooted entrypoint demo: pass`
  - negative path: `rooted_entrypoint::tests::test_project_entrypoint_plan_reports_reachable_frontend_errors` asserts reachable helper-module type errors still fail at rooted-plan construction with `[helper]` diagnostics
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)

### milestone_adhoc_2: Multi-Module Dependency Metadata Aggregation
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1083
- Implementation target:
  - return aggregate `used_stdlib_modules` and `required_crates` from multi-module codegen
  - thread aggregated metadata through rooted project build planning and test support-module codegen
  - ensure reachable-module metadata participates while unreachable sibling modules stay excluded
- Demo target:
  - `cargo test -p sifr_driver test_project_entrypoint_plan_aggregates_reachable_dependency_metadata -- --nocapture`
- Validation target:
  - `cargo test -p sifr_codegen generate_rust_multi_with_metadata`
  - `cargo test -p sifr_driver rooted_entrypoint`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -p sifr_codegen generate_rust_multi_with_metadata` -> passed (`1 passed, 0 failed`)
  - positive path: `cargo test -p sifr_driver test_project_entrypoint_plan_aggregates_reachable_dependency_metadata -- --nocapture` -> passed (`1 passed, 0 failed`)
  - negative path: `rooted_entrypoint::tests::test_project_entrypoint_plan_ignores_unreachable_dependency_metadata` proves unreachable sibling modules do not leak `used_stdlib_modules` or `required_crates` into rooted project metadata
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)

### milestone_adhoc_3: Canonical Manifest Generation Path
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1084
- Implementation target:
  - move Cargo.toml generation into the one shared rooted-entrypoint materialization path
  - drive both single-file and project manifests from aggregated compiler metadata
  - remove the zero-dependency project manifest path
- Demo target:
  - `cargo run -q -p sifr -- run demos/m_adhoc_3_manifest_unification_demo/main.sifr`
- Validation target:
  - `cargo test -p sifr_driver build_project_manifest -- --nocapture`
  - `cargo test -p sifr_driver rooted_entrypoint`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -p sifr_driver build_project_manifest -- --nocapture` -> passed (support-module-required crate manifests and unreachable sibling exclusion both covered)
  - positive path: `cargo run -q -p sifr -- run demos/m_adhoc_3_manifest_unification_demo/main.sifr` -> printed `adhoc milestone 3 manifest unification demo: pass`
  - negative path: `rooted_entrypoint::tests::test_build_project_manifest_ignores_unreachable_required_crates` proves unreachable bigint-only siblings do not contaminate rooted project manifests
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)

### milestone_adhoc_4: CLI Contract Preservation and Regression Hardening
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1085
- Implementation target:
  - add explicit CLI tests for single-file isolation after the rooted-entrypoint refactor
  - prove non-main entrypoints still bypass project mode
  - prove `emit` remains single-file even when `check` resolves project-mode imports
- Demo target:
  - `cargo test -p sifr emit_entrypoint_preserves_single_file_boundary_for_project_like_main -- --nocapture`
- Validation target:
  - `cargo test -p sifr entrypoint_`
  - `cargo test -p sifr emit_entrypoint_preserves_single_file_boundary_for_project_like_main -- --nocapture`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -p sifr entrypoint_` -> passed (`9 passed, 0 failed`)
  - positive path: `cargo test -p sifr emit_entrypoint_preserves_single_file_boundary_for_project_like_main -- --nocapture` -> passed (`1 passed, 0 failed`)
  - negative path: `tests::test_emit_entrypoint_preserves_single_file_boundary_for_project_like_main` proves `check` resolves a project-like `main.sifr` successfully while `emit` still fails on the same local import as a single-file compilation boundary
  - local gate: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)

### milestone_adhoc_5: Dependency Closure Regression Matrix
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1086
- Implementation target:
  - add regression coverage for reachable support-module stdlib dependencies and unreachable sibling exclusion
  - prove non-main intrinsic-required crates remain included only through reachable closure
  - prove transitive reachable dependency chains contribute manifest crates while unreachable chains stay excluded
- Demo target:
  - `cargo run -q -p sifr -- run demos/m_adhoc_5_dependency_closure_demo/main.sifr`
- Validation target:
  - `cargo test -p sifr_codegen generate_rust_multi_with_metadata -- --nocapture`
  - `cargo test -p sifr_driver support_module_stdlib -- --nocapture`
  - `cargo test -p sifr_driver rooted_entrypoint -- --nocapture`
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -p sifr_codegen generate_rust_multi_with_metadata -- --nocapture` -> passed (`2 passed, 0 failed`), keeping aggregate multi-module dependency metadata stable while directly guarding the trait-impl visibility regression in publicized support modules
  - positive path: `cargo test -p sifr_driver support_module_stdlib -- --nocapture` -> passed (`2 passed, 0 failed`), proving a reachable helper using `sifr.tomllib.loads` pulls `toml = "0.8"` into the rooted project manifest while an unreachable sibling does not
  - positive path: `cargo test -p sifr_driver rooted_entrypoint -- --nocapture` -> passed (`11 passed, 0 failed`), covering reachable intrinsic-required crates outside `main`, reachable transitive dependency closure, and the corresponding unreachable negative paths in one rooted-entrypoint regression slice
  - positive path: `cargo run -q -p sifr -- run demos/m_adhoc_5_dependency_closure_demo/main.sifr` -> printed `adhoc milestone 5 dependency closure demo: pass`
  - negative path: `rooted_entrypoint::tests::test_build_project_manifest_ignores_unreachable_support_module_stdlib_crates` proves unreachable `sifr.tomllib` support modules do not contaminate the manifest
  - negative path: `rooted_entrypoint::tests::test_build_project_manifest_ignores_unreachable_required_crates` proves unreachable bigint-only siblings do not leak intrinsic-required crates into the manifest
  - negative path: `rooted_entrypoint::tests::test_build_project_manifest_ignores_unreachable_transitive_dependency_chain` proves unreachable transitive chains stay outside the rooted dependency closure
  - local gate: `scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
  - merge evidence: PR #1086 merged into `main` on 2026-03-10

## External Review Passes

### review_pass_1
- Reviewer artifact: `/Users/yaseralnajjar/work/sifr/codebase/reviews/adhoc-entrypoint-review-2.md`
- Status: complete
- Review summary:
  - phase approved overall
  - remove dead-store `project_name` plumbing in rooted-entrypoint helpers
  - document `RootedEntrypointPlan` in the architecture guide
- Validation evidence:
  - positive path: `cargo test -p sifr_driver rooted_entrypoint -- --nocapture` -> passed (`11 passed, 0 failed`) after removing the dead-store rooted-entrypoint helper plumbing
  - local gate: `scripts/run_all_tests.sh --profile quick` -> passed (`verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`)
- Follow-up PR:
  - https://github.com/sifr-lang/sifr/pull/1087 (merged 2026-03-10)

### review_pass_2
- Reviewer artifact: `/Users/yaseralnajjar/work/sifr/codebase/reviews/adhoc-entrypoint-review-3.md`
- Status: complete
- Review summary:
  - production-grade verdict approved
  - all milestone PRs remain accepted on `main`
  - no additional code changes required after review pass 1
- Validation evidence:
  - reviewer explicitly approved the current `main` state as production-grade for the phase
- Follow-up PR:
  - https://github.com/sifr-lang/sifr/pull/1088
