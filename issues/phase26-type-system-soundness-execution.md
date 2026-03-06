# Phase 26 Execution Checklist (Type-System Soundness)

Status: in_progress (started 2026-03-06)
Owner: phase_26 execution loop
Reference phase doc: `.cursor/plans/main/phases/26_type_system_soundness.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 26 To-Do Plan

### Part 1: milestone_26_1 TypeVar Constraint Enforcement
- [ ] Remove permissive TypeVar assignability shortcuts
- [ ] Capture and enforce TypeVar bounds/constraints for generic calls (PEP 695 + `TypeVar(...)`)
- [ ] Add strict negative diagnostics for unknown/unsatisfied bounds
- [ ] Add part 26.1 positive demo
- [ ] Add part 26.1 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 2: milestone_26_2 Inheritance and Variance Corrections
- [ ] Implement transitive inheritance assignability (multi-level)
- [ ] Remove inheritance special-case hacks
- [ ] Enforce invariance for mutable collections (`list`, `set`, `dict`)
- [ ] Add part 26.2 positive demo
- [ ] Add part 26.2 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 3: milestone_26_3 Optional Arithmetic Soundness
- [ ] Remove implicit optional arithmetic acceptance (`T | None` auto-unwrap)
- [ ] Keep explicit narrowing as the only safe path for optional arithmetic
- [ ] Add part 26.3 positive demo
- [ ] Add part 26.3 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 4: milestone_26_4 Protocol-Bound Strictness Closure
- [ ] Remove protocol-bound default-allow shortcuts
- [ ] Enforce explicit protocol conformance checks for all generic bound validations
- [ ] Add strict regressions for unknown and non-conforming bounds
- [ ] Add part 26.4 positive demo
- [ ] Add part 26.4 negative case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_26_1 TypeVar Constraint Enforcement
status: pending

- [ ] Canonical TypeVar bound/constraint validation implemented
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: pending
- Negative path: pending

## Part 2: milestone_26_2 Inheritance and Variance Corrections
status: pending

- [ ] Multi-level inheritance assignability implemented
- [ ] Invariance on mutable collections implemented
- [ ] Inheritance hacks removed
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: pending
- Negative path: pending

## Part 3: milestone_26_3 Optional Arithmetic Soundness
status: pending

- [ ] Optional arithmetic no longer auto-accepted
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: pending
- Negative path: pending

## Part 4: milestone_26_4 Protocol-Bound Strictness Closure
status: pending

- [ ] Protocol-bound validation is strict and explicit
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: pending
- Negative path: pending

## PR Log
- Part 1: pending
- Part 2: pending
- Part 3: pending
- Part 4: pending
