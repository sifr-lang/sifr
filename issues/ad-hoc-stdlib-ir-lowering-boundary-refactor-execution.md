# Ad Hoc Phase Execution Checklist: Stdlib, IR, and Lowering Boundary Refactor

Phase contract: [ad-hoc-stdlib-ir-lowering-boundary-refactor.md](./ad-hoc-stdlib-ir-lowering-boundary-refactor.md)

Status: in progress

## Checklist

- [x] `milestone_stdlib_boundary_1`: Create `sifr_stdlib` Contract Crate
- [x] `milestone_stdlib_boundary_2`: Centralize Stdlib Feature And Dependency Manifest
- [x] `milestone_ir_boundary_1`: Extract `sifr_ir` Data Crate
- [ ] `milestone_ir_boundary_2`: Rename Remainder To `sifr_lowering`
- [ ] `milestone_ir_boundary_3`: Dependency Direction Guardrails
- [ ] `milestone_ir_boundary_4`: Documentation And Phase Closeout

## Review Artifacts

Record planning and implementation reviews here.

- Initial planning review: `reviews/ad-hoc-stdlib-ir-lowering-boundary-refactor-review-pass-1.md` -> `CHANGES_REQUESTED`; addressed direct-vs-transitive lint dependency wording, added `sifr_stdlib` dependency validation, narrowed the driver stdlib-bootstrap exception, made binary-size validation unconditional, added `Cargo.lock` to stale-name sweeps, and added intrinsic signature/codegen/feature parity checks.
- Follow-up planning review: `reviews/ad-hoc-stdlib-ir-lowering-boundary-refactor-review-pass-2.md` -> `CHANGES_REQUESTED`; aligned `sifr_stdlib` locked dependency rules with the guardrail forbidden set and added a direct-lowering dependency guard for `sifr_analysis`.
- Final planning review: `reviews/ad-hoc-stdlib-ir-lowering-boundary-refactor-review-pass-3.md` -> `READY`; reviewer confirmed the contract is implementation-ready with precise crate ownership, acyclic dependency direction, enumerable exit gates, and milestone validation coverage.
- M1 implementation review: `reviews/ad-hoc-stdlib-boundary-m1-review-1.md` -> `READY`; reviewer confirmed the `sifr_stdlib` contract crate owns intrinsic signatures and embedded source inventory, driver still owns bootstrap compilation, no `sifr_hir` stdlib shim remains, and dependency direction is clean.
- M2 implementation review: `reviews/ad-hoc-stdlib-boundary-m2-review-4.md` -> `READY`; reviewer found no blockers and confirmed the stdlib feature/dependency manifest boundary is mergeable.
- M3 implementation review: `reviews/ad-hoc-stdlib-boundary-m3-review-2.md` -> `READY`; reviewer found no blockers and confirmed the `sifr_ir` data boundary, codegen/lint dependency direction, and retained HIR CFG/flow-graph construction match the M3 contract.

## Validation Ledger

Record local validation for each milestone before opening the corresponding PR.

- M1: local validation passed.
  - `cargo check -p sifr_stdlib` -> PASS.
  - `cargo test -p sifr_stdlib` -> PASS.
  - `cargo tree -p sifr_stdlib --depth 5` -> PASS; no lowering/frontend/codegen/driver/package/analysis/LSP/CLI dependency edge present.
  - `cargo test -p sifr -- stdlib` -> PASS.
  - `cargo test -p sifr_hir name_import_diagnostics_tests` -> PASS.
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/stdlib_test.sifr` -> PASS.
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/import_intrinsic.sifr` -> expected `SIFR-IMPORT-0001` failure.
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr` -> PASS.
  - `scripts/run_all_tests.sh --profile create-pr` -> PASS (`target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded).
- M2: local validation passed.
  - `cargo check -p sifr_stdlib` -> PASS.
  - `cargo check -p sifr_codegen` -> PASS.
  - `cargo check -p sifr_driver` -> PASS.
  - `cargo test -p sifr_stdlib` -> PASS.
  - `cargo test -p sifr_driver` -> PASS.
  - `cargo test -p sifr_codegen generate_project_emits -- --nocapture` -> PASS.
  - `cargo test -p sifr_codegen lowers_json_intrinsics_with_dependency_metadata -- --nocapture` -> PASS.
  - `cargo test -p sifr_codegen lowers_random_intrinsics_via_registry -- --nocapture` -> PASS.
  - `cargo test -p sifr --test e2e --no-run` -> PASS.
  - `python3 scripts/check_file_size_guardrails.py` -> PASS.
  - `scripts/run_all_tests.sh --profile create-pr` -> PASS (`target/validation_lane_reports/create-pr.latest.json`; wall time 77.93s, advisories: none).
  - `scripts/check_codegen_binary_size.sh origin/main HEAD` -> PASS (`baseline_size_bytes=522640`, `candidate_size_bytes=522640`, `delta_bytes=0`).
  - `cargo test -p sifr_codegen` -> FAILS with 54 pre-existing codegen/render expectation failures; verified representative failure on clean `origin/main` before M2 implementation. Focused M2 dependency/metadata tests above pass.
- M3: local validation passed.
  - `cargo check -p sifr_ir` -> PASS.
  - `cargo test -p sifr_ir` -> PASS.
  - `cargo check --workspace` -> PASS.
  - `cargo tree -p sifr_codegen --depth 2 --edges normal` -> PASS; normal dependency tree contains `sifr_ir` and no `sifr_hir`, parser, syntax, frontend, or lowering edge through IR.
  - `cargo tree -p sifr_lint --depth 1 --edges normal` -> PASS; direct dependency tree contains `sifr_ir` and no direct `sifr_hir` lowering dependency.
  - `rg "sifr_hir" crates/sifr_codegen/src --glob '!**/*tests.rs' --glob '!**/tests.rs' --glob '!**/lib_codegen_tests/**'` -> PASS; no production codegen imports from lowering crate.
  - `rg "sifr_hir" crates/sifr_lint/src -g '*.rs'` -> PASS; no lint source imports from lowering crate.
  - `cargo test -p sifr_lint` -> PASS.
  - `cargo test -p sifr_hir cfg` -> PASS.
  - `cargo test -p sifr_hir flow_graph` -> PASS.
  - `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
  - `python3 scripts/check_file_size_guardrails.py` -> PASS.
  - `cargo fmt --check` -> PASS.
  - `scripts/run_all_tests.sh --profile create-pr` -> PASS (`target/validation_lane_reports/create-pr.latest.json`; wall time 155.82s, advisory: warm wall-time budget exceeded).
  - `cargo test -p sifr_codegen` -> FAILS with the same 54 pre-existing codegen/render expectation failures documented for M2; M3 query tests passed within that run and no additional failure count was introduced.
- M4: pending.
- M5: pending.
- M6: pending.

## Merged PRs

Record merged PR links here as each milestone lands.

- M1: https://github.com/sifr-lang/sifr/pull/2284
- M2: https://github.com/sifr-lang/sifr/pull/2285
- M3: pending.
- M4: pending.
- M5: pending.
- M6: pending.
