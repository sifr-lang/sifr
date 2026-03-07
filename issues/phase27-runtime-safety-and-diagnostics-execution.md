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
- [ ] Open PR, review, and merge

### Part 4: milestone_27_4 Span and Diagnostic Schema Quality
- [ ] Introduce canonical structured diagnostic schema (`code`, `severity`, `message`, `url`, spans, children, help, suggestions)
- [ ] Implement `Severity = Error | Warning | Note | Help`
- [ ] Implement suggestion kinds: `DidYouMean | ReplaceText | InsertText | DeleteText`
- [ ] Add stable human/json renderers with json as lossless schema rendering
- [ ] Add demo: `demos/m27_4_span_and_diagnostic_schema_quality_demo`
- [ ] Add negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 5: milestone_27_5 Bounded Multi-Error Recovery
- [ ] Enforce caps: `50` top-level diagnostics, duplicate cap `5`, compact representative locations `5`
- [ ] Add deterministic ordering and duplicate-group summarization `... +N more similar diagnostics`
- [ ] Add recovery matrix regression coverage
- [ ] Add demo: `demos/m27_5_bounded_multi_error_recovery_demo`
- [ ] Add negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 6: milestone_27_6 Stability Contract Finalization
- [ ] Enforce exit code contract `0|1|2|3`
- [ ] Implement `--diagnostic-format human|json|compact` stability behavior and unknown-format exit `2`
- [ ] Lock compact renderer grouping/truncation invariants and snapshot stability
- [ ] Add checked-in panic inventory and convert remaining user-triggerable panic paths
- [ ] Add demo: `demos/m27_6_stability_contract_finalization_demo`
- [ ] Add negative case
- [ ] Run milestone demo + targeted tests + full local suite
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
status: done (2026-03-07, PR: pending)

- [x] Non-literal defaults (collection literals) preserved for function/class defaults
- [x] Unsupported default expressions produce deterministic diagnostics (no silent drop)
- [x] Driver codegen panic boundary converts panics into `CompilePhase::Codegen` diagnostics
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [x] Mark part progress in this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir` -> pass.
- Positive path: `cargo test -q -p sifr_driver run_codegen_with_boundary_reports` -> pass (2 tests).
- Positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/non_literal_default_args.sifr` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m27_3_defaults_and_panic_to_diagnostic_conversion_demo/main.sifr` -> prints `m27_3 defaults and panic-to-diagnostic conversion demo:` then `[1, 9]`, `[1, 9]`, `[1, 2]`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (403 pass tests completed).
- Negative path: `cargo run -q -p sifr -- run demos/m27_3_defaults_and_panic_to_diagnostic_conversion_demo/negative_cases/unsupported_default_call_expression/main.sifr` -> exits `1` with `type error: function 'pick': unsupported default argument expression for parameter 'x'`.
- Negative path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/fail/unsupported_default_expr_call.sifr` -> exits `1` with deterministic unsupported-default diagnostic.

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/897
- Part 2: https://github.com/yaseralnajjar/sifr/pull/898
- Part 3: pending
