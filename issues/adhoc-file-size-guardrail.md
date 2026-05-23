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

Status: planned

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

### milestone_adhoc_file_size_2: Codegen expression, preamble, and intrinsic decomposition

Status: planned

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

### milestone_adhoc_file_size_3: HIR, type-system, diagnostics, frontend, and CLI decomposition

Status: planned

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

### milestone_adhoc_file_size_4: Test harness decomposition

Status: planned

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

### milestone_adhoc_file_size_5: Strict unified guardrail

Status: planned

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
