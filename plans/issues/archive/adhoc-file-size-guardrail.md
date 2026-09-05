# Adhoc Phase: First-Party Source File-Size Guardrail

## Goal

Make Sifr's "no monolithic files" expectation enforceable with a strict, repository-wide guardrail: every hand-maintained first-party source file must stay at or below **900 lines**.

The end state must be simple:

- no legacy allowlist
- no grandfathered oversized files
- no compatibility mode
- one deterministic validation script wired into the existing local validation gate
- elegant responsibility-based refactors for every current violation

## Scope

Applies to hand-maintained first-party source files:

- Rust source: `crates/**/*.rs`
- Python tooling: `scripts/**/*.py`, `verification/**/*.py`
- Sifr fixtures and demos that are manually maintained: `demos/**/*.sifr`, `crates/sifr/tests/**/*.sifr`

Excludes generated or non-source artifacts:

- `third_party/**`
- `target/**`
- lockfiles
- generated snapshots
- generated emitted output such as `emitted.rs`
- machine-generated benchmark outputs and baselines
- issue, review, and long-form documentation files

Python tooling files under `verification/**/*.py` are in scope; the current baseline has zero `verification/**/*.py` violations above 900 lines. The unified guardrail must govern them automatically through the same include-pattern logic, with no separate Python-only rule.

## Current Baseline

Current first-party maintained source violations above 900 lines:

- 35 total maintained source files.
- 34 Rust files under `crates/`.
- 1 Python tooling file under `scripts/`.
- 32 production/tooling files after excluding test harnesses.
- 3 test harness/test files.
- 0 `.sifr` files under `demos/**` or `crates/sifr/tests/**`.

Known violations:

- `crates/sifr/src/main.rs` (3613 lines)
- `crates/sifr/tests/e2e.rs` (4012 lines)
- `crates/sifr_analysis/src/host.rs` (1226 lines)
- `crates/sifr_codegen/src/expr_render_helpers.rs` (2457 lines)
- `crates/sifr_codegen/src/function_emitter.rs` (2277 lines)
- `crates/sifr_codegen/src/helpers.rs` (1306 lines)
- `crates/sifr_codegen/src/hir_analysis/queries.rs` (1168 lines)
- `crates/sifr_codegen/src/hir_analysis/traversal.rs` (940 lines)
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs` (4308 lines)
- `crates/sifr_codegen/src/intrinsics/collections.rs` (1073 lines)
- `crates/sifr_codegen/src/intrinsics/json.rs` (1424 lines)
- `crates/sifr_codegen/src/intrinsics/math.rs` (2527 lines)
- `crates/sifr_codegen/src/intrinsics/mod.rs` (1312 lines)
- `crates/sifr_codegen/src/ir_optimize.rs` (1211 lines)
- `crates/sifr_codegen/src/lib.rs` (2681 lines)
- `crates/sifr_codegen/src/lib_codegen_tests.rs` (4700 lines)
- `crates/sifr_codegen/src/lower_expr.rs` (4691 lines)
- `crates/sifr_codegen/src/lower_item.rs` (1034 lines)
- `crates/sifr_codegen/src/lower_stmt.rs` (9631 lines)
- `crates/sifr_codegen/src/preamble.rs` (3073 lines)
- `crates/sifr_codegen/src/render.rs` (1765 lines)
- `crates/sifr_codegen/src/stdlib_filter.rs` (969 lines)
- `crates/sifr_codegen/src/stmt_support_emitter.rs` (10521 lines)
- `crates/sifr_diagnostics/src/codes.rs` (2834 lines)
- `crates/sifr_frontend/src/lib.rs` (1493 lines)
- `crates/sifr_hir/src/lower/builtin_calls.rs` (1169 lines)
- `crates/sifr_hir/src/lower/classes.rs` (1277 lines)
- `crates/sifr_hir/src/lower/expressions.rs` (3765 lines)
- `crates/sifr_hir/src/lower/expressions_tests.rs` (6343 lines)
- `crates/sifr_hir/src/lower/mod.rs` (1194 lines)
- `crates/sifr_hir/src/lower/nested_function_inference.rs` (1721 lines)
- `crates/sifr_hir/src/lower/statements.rs` (2128 lines)
- `crates/sifr_hir/src/lower/typing_and_functions.rs` (1400 lines)
- `crates/sifr_type_system/src/types.rs` (2096 lines)
- `scripts/run_verification_hardening.py` (1962 lines)

## Implementation Plan

### milestone_adhoc_file_size_1: Codegen statement emitter decomposition

Status: completed

Purpose:

- Decompose the largest codegen statement modules by emitted-language responsibility, not by line count.
- Preserve generated Rust output and public codegen behavior exactly.

Target files:

- `crates/sifr_codegen/src/stmt_support_emitter.rs`
- `crates/sifr_codegen/src/lower_stmt.rs`
- closely coupled statement-support modules discovered during refactor

Expected shape:

- statement lowering orchestration remains thin
- loop/control-flow emission is isolated
- assignment/storage emission is isolated
- pattern/match/branch emission is isolated
- task/async/defer/error statement support is isolated where applicable
- helper modules have concrete names tied to emitted behavior

Validation:

- targeted `sifr_codegen` unit tests
- `bash verification/generated_code_quality/generated_code_quality_determinism.sh`
- `bash verification/generated_code_quality/generated_code_quality_rustfmt.sh`
- `cargo fmt --check`
- `cargo clippy -p sifr_codegen -- -D warnings`

Completion notes:

- Statement lowering and statement-support emitter files are below the unified 900-line cap.
- Quick validation passed after targeted fixes for structured statement expression lowering of class receivers and Decimal/BigDecimal division operands.

### milestone_adhoc_file_size_2: Codegen expression, preamble, and intrinsic decomposition

Status: completed

Purpose:

- Decompose oversized codegen expression and intrinsic modules into cohesive emitters.
- Keep Rust preamble generation deterministic and auditable.
- All files listed in this milestone exceed 900 lines in the current baseline. Files may be removed from this milestone if they become compliant during earlier work or are addressed as a byproduct of another decomposition; the authoritative current violation list is always the file-size guardrail scan.

Target files:

- `crates/sifr_codegen/src/lower_expr.rs`
- `crates/sifr_codegen/src/expr_render_helpers.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- `crates/sifr_codegen/src/preamble.rs`
- `crates/sifr_codegen/src/intrinsics/math.rs`
- `crates/sifr_codegen/src/intrinsics/json.rs`
- `crates/sifr_codegen/src/intrinsics/mod.rs`
- `crates/sifr_codegen/src/function_emitter.rs`
- `crates/sifr_codegen/src/render.rs`
- `crates/sifr_codegen/src/helpers.rs`
- `crates/sifr_codegen/src/ir_optimize.rs`
- `crates/sifr_codegen/src/hir_analysis/queries.rs`
- `crates/sifr_codegen/src/hir_analysis/traversal.rs`
- `crates/sifr_codegen/src/intrinsics/collections.rs`
- `crates/sifr_codegen/src/lower_item.rs`
- `crates/sifr_codegen/src/stdlib_filter.rs`

Expected shape:

- intrinsic groups are split by Python/Sifr surface area
- expression rendering helpers are grouped by expression family
- preamble sections are built from explicit, testable fragments
- crate roots and module roots are wiring and re-export files only

Validation:

- targeted `sifr_codegen` unit tests
- `bash verification/generated_code_quality/generated_code_quality_corpus.sh`
- `bash verification/generated_code_quality/generated_code_quality_panic_scan.sh`
- `bash verification/generated_code_quality/generated_code_quality_determinism.sh`
- `cargo fmt --check`
- `cargo clippy -p sifr_codegen -- -D warnings`

Completion notes:

- Oversized codegen expression, preamble, intrinsic, render, helper, and analysis files are decomposed below the unified cap.
- Diagnostic and split-brain guardrails were adjusted where split `include!` files changed source discovery shape.

### milestone_adhoc_file_size_3: HIR, type-system, diagnostics, frontend, and CLI decomposition

Status: completed

Purpose:

- Finish all non-codegen production/tooling source violations with architecture-preserving splits.
- Prepare older per-domain maintainability checks to defer source file-size enforcement to the unified rule after the unified rule exists.

Target files:

- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/statements.rs`
- `crates/sifr_hir/src/lower/nested_function_inference.rs`
- `crates/sifr_hir/src/lower/typing_and_functions.rs`
- `crates/sifr_hir/src/lower/classes.rs`
- `crates/sifr_hir/src/lower/mod.rs`
- `crates/sifr_hir/src/lower/builtin_calls.rs`
- `crates/sifr_type_system/src/types.rs`
- `crates/sifr_diagnostics/src/codes.rs`
- `crates/sifr_frontend/src/lib.rs`
- `crates/sifr_analysis/src/host.rs`
- `crates/sifr/src/main.rs`
- `scripts/run_verification_hardening.py`

Expected shape:

- HIR lowering stays split by language construct and semantic responsibility
- type-system data definitions are grouped around coherent type families and operations
- diagnostics code registry data is split without weakening coverage checks
- CLI command parsing and command execution are separated from crate entry wiring
- verification hardening script is split into command, fixture, mutation, runner, and reporting modules
- Python tooling remains in scope for this phase; `scripts/run_verification_hardening.py` is the only current Python source violation.

Validation:

- targeted HIR/type/diagnostic/frontend/CLI tests for each touched area
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_sifr_driver_maintainability_guardrails.py`
- `python3 scripts/check_package_manager_guardrails.py`
- `python3 scripts/run_verification_hardening.py --self-test`
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`

Completion notes:

- HIR, type-system, diagnostics, frontend, analysis host, CLI, and verification hardening files are below 900 lines.
- HIR and driver maintainability guardrails now retain domain architecture checks while delegating overlapping line-size enforcement to the unified guardrail.

### milestone_adhoc_file_size_4: Test harness decomposition

Status: completed

Purpose:

- Split oversized tests and harnesses without losing fixture discovery order, snapshot stability, or validation semantics.

Target files:

- `crates/sifr_hir/src/lower/expressions_tests.rs`
- `crates/sifr_codegen/src/lib_codegen_tests.rs`
- `crates/sifr/tests/e2e.rs`

Expected shape:

- tests move into focused modules that mirror production ownership
- e2e harness responsibilities are separated into discovery, execution, expectation handling, and reporting
- fixture order and snapshot names remain deterministic
- create `scripts/validate_fixture_order.py` as part of this milestone; it reads the e2e fixture discovery paths, sorts them lexicographically, and asserts the sorted list matches a committed baseline captured before the harness split

Validation:

- targeted test filters for moved tests
- `python3 scripts/validate_fixture_order.py`
- `cargo test -p sifr_hir`
- `cargo test -p sifr_codegen`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `scripts/run_e2e_pass.sh`

Completion notes:

- Oversized HIR, codegen, and e2e test harness files are split below 900 lines while preserving deterministic fixture discovery and quick e2e pass results.

### milestone_adhoc_file_size_5: Strict unified guardrail

Status: completed

Purpose:

- Add the final repository-wide file-size guardrail after all known violations are removed.
- Wire it into authoritative local validation with no allowlist and no legacy mode.
- Retire older HIR and driver file-size checks in the same milestone as the unified guardrail goes live. If the coverage assertion finds gaps in the unified guardrail patterns, fix the pattern logic before retiring the old checks; do not re-add per-file budgets or compatibility allowlists.

Expected implementation:

- Add `scripts/check_file_size_guardrails.py`.
- Count physical lines using UTF-8 text reads.
- Enforce a hard `900` line cap for included first-party maintained source files.
- Encode include/exclude policy directly and transparently in the script.
- Provide actionable failures: path, current line count, limit, and category.
- Add a self-test mode that creates temporary fixture trees and proves:
  - included files with 900 lines pass
  - included files with 901 lines fail
  - generated output, lockfiles, snapshots, baselines, `target/**`, and `third_party/**` never fail even when over 900 lines
  - failure output includes path, current line count, limit, and category
  - `demos/**/*.sifr` and `crates/sifr/tests/**/*.sifr` are included and pass, matching the baseline scan that found zero `.sifr` fixture violations
- Wire the script into `scripts/run_all_tests.sh --profile quick` and the full validation path.
- Replace or retire narrower file-size checks in older maintainability scripts only after proving every path previously governed by those checks is included in the unified guardrail.
- Migration step: before retiring `check_hir_maintainability_guardrails.py` and `check_sifr_driver_maintainability_guardrails.py` file-size logic, verify that every file previously governed by `MAX_LINES_BY_FILE` or per-domain implementation limits matches the new path-pattern logic. Implement this as either a `--verify-includes <ref-file>` mode that reads the old per-file map or as self-test fixtures that add the old file paths as included cases.
- Remove `MAX_LINES_BY_FILE` and per-file-budget entries after the unified coverage assertion passes. Keep non-file-size checks from existing maintainability scripts when they still enforce domain-specific architecture.
- Update or retire `internal_docs/hir_maintainability_guardrails.md` and `internal_docs/sifr_driver_maintainability_guardrails.md` so their checklists reference the unified file-size guardrail instead of obsolete per-file budgets.

Validation:

- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py --self-test`
- existing maintainability guardrails
- a coverage assertion, in the guardrail self-test or a dedicated test, that every current `MAX_LINES_BY_FILE` / per-domain source path from the older maintainability scripts is included by the unified guardrail patterns
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

Completion notes:

- Added `scripts/check_file_size_guardrails.py` with the unified include/exclude policy and self-test coverage for prior HIR/driver line-budget paths.
- Wired the unified guardrail into `scripts/run_all_tests.sh`, so quick and default/full validation run the same check.
- Removed mechanical `_1` / `_2` / `_part_*` split names from the phase stack; helper and test modules now use responsibility-based names.
- Kept the work reviewable as focused commits: non-codegen decomposition, codegen decomposition, and unified guardrail wiring.
- Quick validation passed end to end: `scripts/run_all_tests.sh --profile quick` completed successfully with `file-size guardrails: PASS (1910 files, limit 900 lines)`, `67 pass tests completed`, and `0` failures.
- Full validation passed end to end: `scripts/run_all_tests.sh` completed successfully with `file-size guardrails: PASS (1910 files, limit 900 lines)`, generated-code corpus/panic-scan/rustfmt/clippy/determinism checks passing, `73 pass tests completed`, and `0` hardening failures.
- Clean-stack review passed with `Verdict: SATISFIED` in `reviews/adhoc-file-size-guardrail-clean-stack-review-pass-2.md`.
- Final readiness review passed with `Verdict: SATISFIED` in `reviews/adhoc-file-size-guardrail-final-review-pass-3.md`.

## Phase Closeout

Status: completed

Merged PRs:

- https://github.com/sifr-lang/sifr/pull/2161 - non-codegen decomposition
- https://github.com/sifr-lang/sifr/pull/2162 - codegen decomposition
- https://github.com/sifr-lang/sifr/pull/2163 - unified guardrail and closeout

Post-closeout cleanup PRs:

- https://github.com/sifr-lang/sifr/pull/2165 - cleanup of refactor smells from the file-size phase

Final validation:

- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py --self-test`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_sifr_driver_maintainability_guardrails.py`
- `python3 scripts/check_diagnostic_code_coverage.py`
- `python3 verification/performance/check_split_brain_guardrail.py`
- `python3 verification/performance/check_split_brain_guardrail.py --self-test`
- `python3 verification/performance/check_budgets.py --self-test`
- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_codegen lib_codegen_tests::test_production_codegen_source_has_no_non_ir_tokens -- --nocapture`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

Validation notes:

- Quick and full validation both passed locally. Both lane reports include warm wall-time and e2e grouping advisories, but the authoritative scripts exited successfully with no blocking failures.
- The phase-specific codegen source scanner test passed. The broad `cargo test -p sifr_codegen` package suite is not part of the authoritative validation gate for this phase and still contains legacy expectation failures unrelated to the file-size guardrail work.
- No first-party maintained source file exceeds the unified 900-line cap.
- No generated, vendored, lockfile, snapshot, baseline, issue, review, or long-form documentation artifact is governed by the source-file cap.
- Post-closeout smell cleanup removed the remaining `#[rustfmt::skip]` refactor artifacts, replaced the verification hardening `exec` loader with package imports, and added a regression guardrail for Rust `rustfmt::skip` attributes.
- Post-closeout cleanup validation passed with `cargo fmt --check`, `cargo test -p sifr_diagnostics`, `python3 scripts/check_file_size_guardrails.py`, `scripts/run_all_tests.sh --profile quick`, and `scripts/run_all_tests.sh`.
- Post-closeout reviewer passes both returned `Verdict: SATISFIED` in `reviews/clean-refactor-smells-review-1.md` and `reviews/clean-refactor-smells-review-2.md`.
- Final module-layout cleanup replaced the remaining first-party hand-written Rust `include!` / `#[path]` split shape with conventional Rust modules and responsibility-based file locations.
- Final module-layout cleanup validation passed with `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `git diff --check`, focused first-party scans for `include!`, `#[path]`, `rustfmt::skip`, `allow(unused_imports)`, and numeric split filenames, `scripts/run_all_tests.sh --profile quick`, and `scripts/run_all_tests.sh`.
- Final module-layout agent review returned `Satisfied` with no blocking findings in `reviews/include-module-refactor-review-round1.md`.
- E2E runner cleanup removed the retired runner compatibility path from `crates/sifr/tests/e2e.rs` and its support modules, including compare-mode dispatch, legacy runner mode environment variables, and validation-lane mode metadata.
- E2E runner cleanup validation passed with focused stale-runner scans, `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `git diff --check`, shell syntax checks, `python3 -m py_compile scripts/validation_lane.py`, `cargo test -p sifr --test e2e -- --skip test_e2e_pass`, `scripts/run_e2e_pass.sh --profile quick`, `scripts/run_all_tests.sh --profile quick`, and `scripts/run_all_tests.sh`.
- E2E runner cleanup agent review reached `Verdict: Satisfied` in `reviews/e2e-runner-cleanup-review-round2.md`.
- Codegen refactor polish replaced the remaining broad root re-exports in the touched statement-lowering roots with explicit surfaces, split near-cap statement-support and lower-statement modules by responsibility, and kept test functions private while retaining explicit helper visibility only for sibling test modules.
- Codegen refactor polish validation passed with `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, focused scans for `include!` and wildcard re-exports in the touched codegen roots/modules, and `scripts/run_all_tests.sh --profile quick`.
- Codegen refactor polish agent review returned `SATISFIED` in `reviews/codegen-refactor-polish-review-round1.md`.
- Expr render cleanup removed stale `legacy_i64` terminology from the SifrInt promotion helpers and adjacent state comments, replacing it with current plain-`i64` storage naming.
- Expr render cleanup validation passed with `cargo fmt --check`, `cargo check -p sifr_codegen`, focused legacy/backcompat scans, and `cargo test -p sifr_codegen expr_render_helpers::tests`.
- Expr render cleanup agent review returned `SATISFIED` in `reviews/expr-render-legacy-cleanup-review-round1.md`. The attempted quick validation progressed through the earlier guardrail and package lanes but was stopped after reproducing a pre-existing `cargo test -p sifr_lsp --doc` hang unrelated to this cleanup.

## Done Criteria

- No first-party maintained source file exceeds 900 lines.
- The unified guardrail passes without any allowlist.
- Quick and full local validation include the unified guardrail.
- Existing HIR, driver, and package maintainability guardrails either remain stricter for their domains or have their overlapping file-size logic cleanly retired.
- HIR and driver maintainability checklist docs reference the unified source file-size guardrail.
- No generated, vendored, lockfile, snapshot, baseline, issue, review, or long-form documentation artifact is governed by the source-file cap.
- Refactors preserve public behavior and generated output unless a milestone explicitly documents a behavior change, which this phase does not require.
- Review artifacts confirm implementation readiness before work begins.

## AGENTS.md Addition

Add only the concise policy note:

```md
## File-size guardrail

Hand-maintained first-party source files must stay under **900 lines**. Generated files, lockfiles, snapshots, baselines, `target/**`, and `third_party/**` are excluded.

Run the file-size guardrail before considering work complete. If a touched file exceeds the cap, refactor it by responsibility rather than adding more code to an oversized module.

Use the existing HIR lowering and package-manager module layouts as examples of responsibility-based decomposition: split by compiler concern and ownership boundary, not by alphabetical order or line-count chunks.
```
