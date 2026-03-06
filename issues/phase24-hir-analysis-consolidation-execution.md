# Phase 24 Execution Checklist (HIR Analysis Consolidation)

Status: in progress (started 2026-03-06)
Owner: phase_24 execution loop
Reference phase doc: `.cursor/plans/main/phases/24_hir_analysis_consolidation.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 24 To-Do Plan

### Part 1: milestone_24_1 Canonical Traversal Layer Contract
- [x] Establish one canonical traversal layer as the only recursive descent over `HirStmt`/`HirExpr` for analysis use-cases
- [x] Document traversal invariants and HIR-variant extension rules
- [x] Migrate analysis recursion entrypoints to canonical traversal APIs
- [x] Add traversal-layer regression tests (positive + negative)
- [x] Add milestone 24.1 positive demo
- [x] Add milestone 24.1 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 2: milestone_24_2 Semantic Query Layer Standardization
- [x] Build reusable semantic query APIs on top of traversal (return/yield/function-call/defined-variable/references/mutation)
- [x] Migrate emitter/lowering consumers to query APIs instead of local recursive matching
- [x] Add query-layer regression tests (positive + negative)
- [x] Add milestone 24.2 positive demo
- [x] Add milestone 24.2 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 3: milestone_24_3 Control-Flow Effect Query Unification
- [x] Introduce canonical control-flow effect model/query API
- [x] Replace remaining ad-hoc `body_always_exits`-style logic with shared query API
- [x] Ensure all affected call sites consume the canonical effect query
- [x] Add control-flow effect regressions (positive + negative)
- [x] Add milestone 24.3 positive demo
- [x] Add milestone 24.3 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 4: milestone_24_4 Analysis/Emission Boundary Hardening
- [ ] Enforce strict analysis/emission boundaries: analysis computes facts, emitters consume facts
- [ ] Remove emitter-local analysis branching where canonical queries exist
- [ ] Add boundary-hardening regressions (positive + negative)
- [ ] Add milestone 24.4 positive demo
- [ ] Add milestone 24.4 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 5: milestone_24_5 Consolidation Regression Matrix
- [ ] Add consolidation regression matrix for nested conditionals, loop exits, early returns/raises, and mixed blocks
- [ ] Add parity checks proving consolidated query semantics remain correct
- [ ] Wire matrix into local validation so regressions fail pre-merge
- [ ] Add milestone 24.5 positive demo
- [ ] Add milestone 24.5 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_24_1 Canonical Traversal Layer Contract
status: done (2026-03-06, PR #875)

- [x] Canonical traversal contract implemented
- [x] Invariants and extension rules documented
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_codegen walk_stmts_` -> pass (`walk_stmts_covers_try_handlers_loop_else_and_match_patterns`, `walk_stmts_respects_nested_function_scope_boundary`).
- Positive path: `cargo test -q -p sifr_codegen` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m24_1_canonical_traversal_layer_contract_demo/main.sifr` -> prints `m24_1 canonical traversal layer contract demo:` and `6`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m24_1_canonical_traversal_layer_contract_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'str'`.

## Part 2: milestone_24_2 Semantic Query Layer Standardization
status: done (2026-03-06, PR #877)

- [x] Query layer implemented and adopted by consumers
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_codegen hir_analysis::queries::tests::` -> pass.
- Positive path: `cargo test -q -p sifr_codegen` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m24_2_semantic_query_layer_standardization_demo/main.sifr` -> prints `m24_2 semantic query layer standardization demo:` and `0`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m24_2_semantic_query_layer_standardization_demo/negative_cases/recursive_call_typo/main.sifr` -> exits `1` with `type error: undefined function: 'reccurse'`.

## Part 3: milestone_24_3 Control-Flow Effect Query Unification
status: done (2026-03-06, PR #878)

- [x] Control-flow effect model/query implemented
- [x] Ad-hoc exit analysis removed from emitters/lowering
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_codegen block_control_flow_effect_` -> pass.
- Positive path: `cargo test -q -p sifr_codegen` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m24_3_control_flow_effect_query_unification_demo/main.sifr` -> prints `m24_3 control-flow effect query unification demo:` then `7` and `99`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m24_3_control_flow_effect_query_unification_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'str'`.

## Part 4: milestone_24_4 Analysis/Emission Boundary Hardening
status: pending

- [ ] Boundary hardening complete across affected modules
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Pending.

## Part 5: milestone_24_5 Consolidation Regression Matrix
status: pending

- [ ] Regression matrix added and wired into local validation
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Pending.

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/875
- Part 2: https://github.com/yaseralnajjar/sifr/pull/877
- Part 3: https://github.com/yaseralnajjar/sifr/pull/878
- Part 4: pending
- Part 5: pending

## Reviewer Follow-up
- External review pass 1 output: pending
- Remediation PR (pass 1): pending
- External review pass 2 output: pending
- Remediation PR (pass 2): pending
