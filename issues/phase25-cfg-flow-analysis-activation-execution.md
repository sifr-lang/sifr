# Phase 25 Execution Checklist (CFG/Flow Analysis Activation)

Status: in_progress (started 2026-03-06)
Owner: phase_25 execution loop
Reference phase doc: `.cursor/plans/main/phases/25_cfg_flow_analysis_activation.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

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
status: in_progress

- [ ] Canonical CFG entrypoints implemented in `sifr_hir`
- [ ] Selected active analysis query consumes CFG truth
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir cfg::tests::` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m25_1_cfg_integration_contract_demo/main.sifr` -> prints `m25_1 cfg integration contract demo:` then `41` and `0`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m25_1_cfg_integration_contract_demo/negative_cases/reachable_type_error/main.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'None | int'`.

## PR Log
- Part 1: pending
