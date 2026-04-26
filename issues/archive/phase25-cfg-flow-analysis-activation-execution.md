# Phase 25 Execution Checklist (CFG/Flow Analysis Activation)

Status: completed (started 2026-03-06, completed 2026-03-06)
Owner: phase_25 execution loop
Reference phase doc: `internal_docs/phases/25_cfg_flow_analysis_activation.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 25 To-Do Plan

### Part 1: milestone_25_1 CFG Integration Contract
- [ ] Replace unused CFG side module with canonical CFG construction/query entrypoints in `sifr_hir`
- [ ] Define CFG construction ownership boundary (`sifr_hir` owns CFG truth, consumers query via API)
- [ ] Integrate CFG query consumption into active analysis path for selected flow query
- [ ] Add part 25.1 positive demo
- [ ] Add part 25.1 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 2: milestone_25_2 CFG Validity Invariants
- [ ] Add explicit CFG invariants (block identity/order, edge validity, terminator completeness)
- [ ] Enforce fail-fast internal CFG validation
- [ ] Add deterministic-repeat checks for CFG shape construction
- [ ] Add part 25.2 positive demo
- [ ] Add part 25.2 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 3: milestone_25_3 Canonical Flow Truth Queries
- [ ] Implement CFG-backed canonical queries for reachability and always-exits
- [ ] Replace tree-walk fallback logic for correctness-critical flow queries
- [ ] Migrate affected query consumers to canonical CFG flow-truth path
- [ ] Add part 25.3 positive demo
- [ ] Add part 25.3 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 4: milestone_25_4 Diagnostics and Consumer Integration
- [ ] Integrate CFG flow facts into lowering/codegen decision points
- [ ] Ensure control-flow diagnostics consume CFG-derived facts deterministically
- [ ] Add integration regressions for deterministic diagnostics behavior
- [ ] Add part 25.4 positive demo
- [ ] Add part 25.4 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 5: milestone_25_5 Regression and Determinism Matrix
- [ ] Add focused CFG/flow regressions for nested branching, loop exits, early return/raise, unreachable blocks
- [ ] Add deterministic repeat-run checks for CFG shape and query results over the phase corpus
- [ ] Wire phase 25 matrix into local validation
- [ ] Add part 25.5 positive demo
- [ ] Add part 25.5 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_25_1 CFG Integration Contract
status: done (2026-03-06, PR #883)

- [x] Canonical CFG entrypoints implemented in `sifr_hir`
- [x] Selected active analysis query consumes CFG truth
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir cfg::tests::` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m25_1_cfg_integration_contract_demo/main.sifr` -> prints `m25_1 cfg integration contract demo:` then `41` and `0`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m25_1_cfg_integration_contract_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'None | int'`.

## Part 2: milestone_25_2 CFG Validity Invariants
status: done (2026-03-06, PR #884)

- [x] CFG invariants implemented and fail-fast enforced
- [x] Deterministic repeat-run coverage added
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir cfg::tests::control_flow_graph_` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m25_2_cfg_validity_invariants_demo/main.sifr` -> prints `m25_2 cfg validity invariants demo:` then `4`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m25_2_cfg_validity_invariants_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'str'`.

## Part 3: milestone_25_3 Canonical Flow Truth Queries
status: done (2026-03-06, PR #885)

- [x] Reachability and always-exits queries are CFG-backed and canonical
- [x] Tree-walk fallback logic removed for correctness-critical flow facts
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_codegen hir_analysis::queries::tests::` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m25_3_canonical_flow_truth_queries_demo/main.sifr` -> prints `m25_3 canonical flow truth queries demo:` then `5` and `77`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m25_3_canonical_flow_truth_queries_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'str'`.

## Part 4: milestone_25_4 Diagnostics and Consumer Integration
status: done (2026-03-06, PR #886)

- [x] Lowering/codegen decision points consume CFG flow facts
- [x] Control-flow diagnostics consume CFG-derived reachability facts
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m25_4_diagnostics_and_consumer_integration_demo/main.sifr` -> prints `m25_4 diagnostics and consumer integration demo:` then `2` and `3` (with deterministic unreachable warning).
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m25_4_diagnostics_and_consumer_integration_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'str'`.

## Part 5: milestone_25_5 Regression and Determinism Matrix
status: done (2026-03-06, PR #887)

- [x] Focused regression matrix added (nested branching, loop exits, early return/raise, unreachable blocks)
- [x] Deterministic repeat-run checks for CFG graph shape and query results added
- [x] Matrix wired into local validation gate
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `bash scripts/run_phase25_cfg_flow_activation_matrix.sh` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m25_5_cfg_flow_activation_regression_matrix_demo/main.sifr` -> prints `m25_5 cfg flow activation regression matrix demo:` then `8`, `42`, `9`.
- Positive path: `cargo test -q -p sifr_hir cfg::tests::cfg_repeat_run_matrix_is_deterministic -- --nocapture` -> pass.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes phase-25 matrix gate).
- Negative path: `cargo run -q -p sifr -- run demos/m25_5_cfg_flow_activation_regression_matrix_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'str'`.
- Negative path: matrix row `negative_reachable_type_error_parity` asserts identical diagnostics across `check/build/run`.
- Negative path: matrix row `negative_diagnostic_stability` asserts repeated diagnostics are byte-identical.

## PR Log
- Part 1: https://github.com/sifr-lang/sifr/pull/883
- Part 2: https://github.com/sifr-lang/sifr/pull/884
- Part 3: https://github.com/sifr-lang/sifr/pull/885
- Part 4: https://github.com/sifr-lang/sifr/pull/886
- Part 5: https://github.com/sifr-lang/sifr/pull/887
- Review pass 1 remediation: https://github.com/sifr-lang/sifr/pull/889

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase25-review.md` (2026-03-06, APPROVED with notes)
- Pass 1 remediation scope:
  - document synthetic CFG blocks explicitly in `CfgBlockLabel::Synthetic` comments for architecture clarity.
- Pass 1 remediation validation evidence:
  - `cargo test -q -p sifr_hir cfg::tests::` -> pass.
- External review pass 2 output: `reviews/phase25-production-grade-review-2.md` (2026-03-06, APPROVED FOR PRODUCTION)
- Pass 2 reviewer note validation:
  - reviewed all listed notes; all were non-blocking and already satisfied by the current implementation/test matrix.
  - no additional correctness or architecture fixes were required in this pass.
