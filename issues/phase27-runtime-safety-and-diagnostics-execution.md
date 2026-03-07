# Phase 27 Execution Checklist (Runtime Safety and Diagnostics Contract)

Status: in_progress (started 2026-03-07)
Owner: phase_27 execution loop
Reference phase docs:
- `.cursor/plans/main/phases/27_runtime_safe_codegen_semantics.md`
- `.cursor/plans/main/phases/27_diagnostics_error_recovery_and_stability_contract.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 27 To-Do Plan

### Part 1: milestone_27_1 Remove Data-Dependent `unwrap/expect`
- [x] Remove generated data-dependent unwrap/expect on index-related paths and optional-len lowering
- [x] Replace internal non-optional index fallback unwraps with explicit codegen errors
- [x] Add demo: `demos/m27_1_remove_data_dependent_unwrap_expect_demo`
- [x] Add negative case for unsafe optional method usage
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 2: milestone_27_2 Indexing and Semantics Parity Fixes
- [x] Fix negative indexing parity across read/mutation/index-derived flows
- [x] Add strict negative-index regressions that fail if values silently degrade to `None`
- [x] Add demo: `demos/m27_2_indexing_and_semantics_parity_fixes_demo`
- [x] Add negative case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 3: milestone_27_3 Defaults and Panic-to-Diagnostic Conversion
- [x] Lock non-literal default argument behavior with positive/negative regressions
- [x] Add panic boundary and convert user-triggerable panic surfaces to diagnostics
- [x] Add demo: `demos/m27_3_defaults_and_panic_to_diagnostic_conversion_demo`
- [x] Add negative case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 4: milestone_27_4 Span and Diagnostic Schema Quality
- [x] Introduce canonical structured diagnostic schema (`code`, `severity`, `message`, `url`, spans, children, help, suggestions)
- [x] Implement `Severity = Error | Warning | Note | Help`
- [x] Implement suggestion kinds: `DidYouMean | ReplaceText | InsertText | DeleteText`
- [x] Add stable human/json renderers with json as lossless schema rendering
- [x] Add demo: `demos/m27_4_span_and_diagnostic_schema_quality_demo`
- [x] Add negative case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 5: milestone_27_5 Bounded Multi-Error Recovery
- [x] Enforce caps: `50` top-level diagnostics, duplicate cap `5`, compact representative locations `5`
- [x] Add deterministic ordering and duplicate-group summarization `... +N more similar diagnostics`
- [x] Add recovery matrix regression coverage
- [x] Add demo: `demos/m27_5_bounded_multi_error_recovery_demo`
- [x] Add negative case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 6: milestone_27_6 Stability Contract Finalization
- [x] Enforce exit code contract `0|1|2|3`
- [x] Implement `--diagnostic-format human|json|compact` stability behavior and unknown-format exit `2`
- [x] Lock compact renderer grouping/truncation invariants and snapshot stability
- [x] Add checked-in panic inventory and convert remaining user-triggerable panic paths
- [x] Add demo: `demos/m27_6_stability_contract_finalization_demo`
- [x] Add negative case
- [x] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_27_1 Remove Data-Dependent `unwrap/expect`
status: done (2026-03-07, PR #897)

- [x] Generated data-dependent unwrap/expect removed from option-len and non-optional index fallback paths
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_codegen` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_compile_indexing_path_does_not_emit_unwrap_in_main_body` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m27_1_remove_data_dependent_unwrap_expect_demo/main.sifr` -> prints `m27_1 remove data-dependent unwrap/expect demo:` then `20`.
- Positive path: `cargo run -q -p sifr -- emit demos/m27_1_remove_data_dependent_unwrap_expect_demo/main.sifr | rg -n "\\.unwrap\\(|\\.expect\\("` -> no matches.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (401 pass tests completed).
- Negative path: `cargo run -q -p sifr -- run demos/m27_1_remove_data_dependent_unwrap_expect_demo/negative_cases/option_method_without_narrowing/main.sifr` -> exits `1` with `type error: type 'None | list[int]' has no method 'len'`.

## Part 2: milestone_27_2 Indexing and Semantics Parity Fixes
status: done (2026-03-07, PR #898)

- [x] Negative indexing parity fixed across list mutation/delete and nested mutation paths
- [x] Strong read/mutation negative-index regressions added
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_codegen` -> pass.
- Positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/negative_index_list.sifr` -> pass (now asserts non-`None` for `[-1]` and `[-2]`).
- Positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/negative_index_string.sifr` -> pass (now asserts non-`None` for `[-1]` and `[-2]`).
- Positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/negative_index_mutations.sifr` -> pass (`[-1]` assignment/augassign/delete semantics validated).
- Positive path: `cargo run -q -p sifr -- run demos/m27_2_indexing_and_semantics_parity_fixes_demo/main.sifr` -> prints `m27_2 indexing and semantics parity fixes demo:` then `[1, 7]`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (402 pass tests completed).
- Negative path: `cargo run -q -p sifr -- run demos/m27_2_indexing_and_semantics_parity_fixes_demo/negative_cases/invalid_index_type/main.sifr` -> exits `1` with `type error: cannot index type 'list[int]' with 'str'`.

## Part 3: milestone_27_3 Defaults and Panic-to-Diagnostic Conversion
status: done (2026-03-07, PR #899)

- [x] Non-literal defaults (collection literals) preserved for function/class defaults
- [x] Unsupported default expressions produce deterministic diagnostics (no silent drop)
- [x] Driver codegen panic boundary converts panics into `CompilePhase::Codegen` diagnostics
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir` -> pass.
- Positive path: `cargo test -q -p sifr_driver run_codegen_with_boundary_reports` -> pass (2 tests).
- Positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/non_literal_default_args.sifr` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m27_3_defaults_and_panic_to_diagnostic_conversion_demo/main.sifr` -> prints `m27_3 defaults and panic-to-diagnostic conversion demo:` then `[1, 9]`, `[1, 9]`, `[1, 2]`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (403 pass tests completed).
- Negative path: `cargo run -q -p sifr -- run demos/m27_3_defaults_and_panic_to_diagnostic_conversion_demo/negative_cases/unsupported_default_call_expression/main.sifr` -> exits `1` with `type error: function 'pick': unsupported default argument expression for parameter 'x'`.
- Negative path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/fail/unsupported_default_expr_call.sifr` -> exits `1` with deterministic unsupported-default diagnostic.

## Part 4: milestone_27_4 Span and Diagnostic Schema Quality
status: done (2026-03-07, PR #900)

- [x] Canonical structured diagnostic schema introduced for CLI-facing diagnostics with stable fields (`code`, `severity`, `message`, `url`, `primary_span`, `related_spans`, `children`, `help`, `suggestions`)
- [x] Severity enum implemented exactly as `Error | Warning | Note | Help`
- [x] Structured suggestion kinds implemented exactly as `DidYouMean | ReplaceText | InsertText | DeleteText`
- [x] Stable `human` and `json` diagnostic renderers implemented (`json` emits the canonical schema)
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver compile_error_to_diagnostic` -> pass.
- Positive path: `cargo test -q -p sifr_driver compile_errors_to_diagnostics_preserves_order` -> pass.
- Positive path: `cargo test -q -p sifr` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m27_4_span_and_diagnostic_schema_quality_demo/main.sifr` -> prints `m27_4 diagnostic schema quality demo`.
- Positive path: `cargo run -q -p sifr -- --diagnostic-format json check /tmp/.../ok.sifr` -> prints `[]`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (403 pass tests completed).
- Negative path: `cargo run -q -p sifr -- --diagnostic-format json check demos/m27_4_span_and_diagnostic_schema_quality_demo/negative_cases/type_error_json_diagnostic/main.sifr` -> exits `1` and emits canonical json diagnostic with stable `code`, `severity`, and `url`.
- Negative path: `cargo run -q -p sifr -- --diagnostic-format json check /tmp/.../parse_error.sifr` -> exits `1` and emits canonical parse diagnostic with stable parser code/url.

## Part 5: milestone_27_5 Bounded Multi-Error Recovery
status: done (2026-03-07, PR #901)

- [x] Recovery hard limits enforced (`50` top-level diagnostics, `5` per duplicate group, summary tail)
- [x] Deterministic grouping/ordering implemented via canonical key ordering
- [x] Duplicate summarization uses exact suffix `... +N more similar diagnostics`
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver apply_diagnostic_recovery_limits` -> pass (2 tests).
- Positive path: `cargo run -q -p sifr -- check demos/m27_5_bounded_multi_error_recovery_demo/main.sifr` -> emits multiple deterministic diagnostics in one invocation.
- Positive path: `cargo run -q -p sifr -- --diagnostic-format json check demos/m27_5_bounded_multi_error_recovery_demo/negative_cases/repeated_type_errors/main.sifr` -> emits bounded json diagnostics with grouped summary tail.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (403 pass tests completed).
- Negative path: `cargo run -q -p sifr -- check demos/m27_5_bounded_multi_error_recovery_demo/negative_cases/repeated_type_errors/main.sifr` -> exits `1` and includes `type error: ... +3 more similar diagnostics`.
- Negative path: repeated diagnostics above cap are truncated to 5 representatives per group with summary record.

## Part 6: milestone_27_6 Stability Contract Finalization
status: done (2026-03-07, PR: pending)

- [x] Exit code contract `0|1|2|3` enforced by CLI command flow and panic/error boundaries
- [x] Stable CLI format contract implemented: `--diagnostic-format human|json|compact`
- [x] Unknown diagnostic-format values fail with exit code `2` before semantic work
- [x] Compact renderer invariants implemented:
  - first line severity summary
  - grouping by `(severity, code, canonical message)`
  - max 5 representative locations per group with `... +N more` truncation
  - max one help line and one docs URL line per group
- [x] Compact output is renderer-only over canonical diagnostics (no semantic ownership changes)
- [x] Checked-in panic inventory and explicit follow-up owners/issues for remaining invariant panics
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr` -> pass (includes new stability contract tests).
- Positive path: `cargo test -q -p sifr test_compile_error_exit_code_contract_user_vs_internal` -> pass.
- Positive path: `cargo test -q -p sifr test_run_with_panic_boundary_converts_panic_to_internal_compile_error` -> pass.
- Positive path: `cargo test -q -p sifr test_compact_renderer_invariants_summary_grouping_and_bounds` -> pass.
- Positive path: `cargo run -q -p sifr -- --diagnostic-format human check demos/m27_6_stability_contract_finalization_demo/main.sifr` -> exits `0`.
- Positive path: `cargo run -q -p sifr -- --diagnostic-format json check demos/m27_6_stability_contract_finalization_demo/main.sifr` -> prints `[]`, exits `0`.
- Positive path: `cargo run -q -p sifr -- --diagnostic-format compact check demos/m27_6_stability_contract_finalization_demo/main.sifr` -> prints zero-summary line, exits `0`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (403 pass tests completed).
- Negative path: `cargo run -q -p sifr -- --diagnostic-format unknown check demos/m27_6_stability_contract_finalization_demo/negative_cases/unknown_diagnostic_format/main.sifr` -> exits `2` with clap format validation error.
- Negative path: `cargo run -q -p sifr -- --diagnostic-format compact check demos/m27_6_stability_contract_finalization_demo/negative_cases/compact_grouping_contract/main.sifr` -> exits `1`; compact output emits grouped diagnostics with truncation (`... +3 more`).
- Negative path: `cargo run -q -p sifr -- --diagnostic-format json check demos/m27_6_stability_contract_finalization_demo/negative_cases/compact_grouping_contract/main.sifr` -> exits `1`; canonical bounded diagnostic list is emitted in json.
- Inventory: [`phase27-panic-inventory.md`](./phase27-panic-inventory.md)
- Follow-ups: [`phase27-panic-followups.md`](./phase27-panic-followups.md)

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/897
- Part 2: https://github.com/yaseralnajjar/sifr/pull/898
- Part 3: https://github.com/yaseralnajjar/sifr/pull/899
- Part 4: https://github.com/yaseralnajjar/sifr/pull/900
- Part 5: https://github.com/yaseralnajjar/sifr/pull/901
- Part 6: pending
