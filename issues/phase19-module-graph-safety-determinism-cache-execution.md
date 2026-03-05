# Phase 19 Execution Checklist (Module Graph Safety, Determinism, and Cache)

Status: in progress (2026-03-05)
Owner: phase_19 execution loop
Reference phase doc: `.cursor/plans/main/phases/19_module_graph_safety_determinism_and_cache.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_19_1 Dependency-Safe Module Ordering
status: done (2026-03-05, PR #834)

- [x] Introduce topological ordering for module compilation
- [x] Add cycle diagnostics with actionable context
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m19_1_dependency_safe_module_ordering_demo/main.sifr` -> prints `m19_1 dependency-safe module ordering demo:` and `19`.
- Positive path: `cargo test -q -p sifr_driver` -> pass (includes `test_compute_module_compile_order_is_dependency_safe`).
- Negative path: `cargo run -q -p sifr -- run demos/m19_1_dependency_safe_module_ordering_demo/negative_cases/main.sifr` -> exits `1` with cycle diagnostic containing `module dependency cycle detected: a -> b -> a`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 2: milestone_19_2 Deterministic Assembly
status: done (2026-03-05, PR #835)

- [x] Remove nondeterministic HashMap-order behavior from module assembly/output
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m19_2_deterministic_assembly_demo/main.sifr` -> prints `m19_2 deterministic assembly demo:` and `A-Z`.
- Positive path: `cargo test -q -p sifr_driver` -> pass (includes deterministic assembly regression coverage).
- Negative path: `cargo test -q -p sifr_driver test_assemble_project_main_rs_is_deterministic_against_hashmap_order` -> pass (guards against HashMap insertion-order drift).
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 3: milestone_19_3 Stdlib Cache for Local Loops
status: pending

- [ ] Cache stdlib compilation artifacts for repeated check/test cycles
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: pending
- Negative path: pending
- Full suite: pending

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/834
- Part 2: https://github.com/yaseralnajjar/sifr/pull/835
- Part 3: pending

## Reviewer Follow-up
- External review pass 1 output: pending
- Remediation PR (pass 1): pending
- External review pass 2 output: pending
- Remediation PR (pass 2): pending
