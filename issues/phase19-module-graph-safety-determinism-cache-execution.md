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
status: pending

- [ ] Introduce topological ordering for module compilation
- [ ] Add cycle diagnostics with actionable context
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

## Part 2: milestone_19_2 Deterministic Assembly
status: pending

- [ ] Remove nondeterministic HashMap-order behavior from module assembly/output
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
- Part 1: pending
- Part 2: pending
- Part 3: pending

## Reviewer Follow-up
- External review pass 1 output: pending
- Remediation PR (pass 1): pending
- External review pass 2 output: pending
- Remediation PR (pass 2): pending
