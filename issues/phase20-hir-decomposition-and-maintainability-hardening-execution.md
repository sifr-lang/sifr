# Phase 20 Execution Checklist (HIR Decomposition and Maintainability Hardening)

Status: in_progress (2026-03-05)
Owner: phase_20 execution loop
Reference phase doc: `.cursor/plans/main/phases/20_hir_decomposition_and_maintainability_hardening.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 20 To-Do Plan

### Part 1: milestone_20_1 Split `lower.rs`
- [ ] Decompose `crates/sifr_hir/src/lower.rs` into focused lowering modules:
  - [ ] import handling
  - [ ] diagnostics/error-type helpers
  - [ ] class lowering
  - [ ] typing hooks + function signature/body lowering
  - [ ] statement lowering
  - [ ] expression lowering
- [ ] Keep public lowering API stable (`lower_module*`, `LoweringResult`, `ExternalDefs`, `LoweringError`)
- [ ] Add/confirm targeted regression tests for lowered behavior parity
- [ ] Add milestone 20.1 positive demo
- [ ] Add milestone 20.1 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 2: milestone_20_2 Split `stdlib.rs`
- [ ] Decompose intrinsic registry into focused modules:
  - [ ] shared intrinsic helpers/types
  - [ ] registry dispatch
  - [ ] grouped intrinsic module definitions (io/json/math/test/collections/bytes/time/sys/fs/crypto/regex/uuid/platform/toml/datetime/html/calendar/compress/logging)
- [ ] Preserve module names and type signatures exactly
- [ ] Add milestone 20.2 positive demo for stdlib/intrinsic stability
- [ ] Add milestone 20.2 negative regression case for unknown/unsupported intrinsic usage
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 3: milestone_20_3 Anti-Regrowth Guardrails
- [ ] Define and document max file-size/module-boundary conventions for HIR lowering and stdlib registry
- [ ] Add enforceable local/CI guard script(s) for file-size boundaries
- [ ] Add review checklist for adding new lowering logic and intrinsic definitions
- [ ] Add positive-path guardrail test
- [ ] Add negative-path guardrail test (intentional threshold violation fixture)
- [ ] Add milestone 20.3 demo (guardrails in action)
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_20_1 Split `lower.rs`
status: done (2026-03-05, PR #839)

- [x] Extract lowering concerns into coherent modules
- [x] Preserve current semantics and test outcomes
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m20_1_lower_decomposition_demo/main.sifr` -> prints `m20_1 lower decomposition demo:`, `21`, `3`.
- Negative path: `cargo run -q -p sifr -- run demos/m20_1_lower_decomposition_demo/negative_cases/return_type_mismatch.sifr` -> exits `1` with `type error: return type mismatch: expected 'int', got 'str'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 2: milestone_20_2 Split `stdlib.rs`
status: in_progress (2026-03-05, PR pending)

- [x] Partition stdlib metadata/registration logic into focused modules
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m20_2_stdlib_registry_split_demo/main.sifr` -> prints `m20_2 stdlib registry split demo:` and `\"ok\"`.
- Negative path: `cargo run -q -p sifr -- run demos/m20_2_stdlib_registry_split_demo/negative_cases/forbidden_intrinsic_import.sifr` -> exits `1` with `_sifr.* modules are internal compiler intrinsics` and `undefined function: 'sqrt'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 3: milestone_20_3 Anti-Regrowth Guardrails
status: pending

- [ ] Add file-size and module-boundary conventions
- [ ] Add review checklist items for new lowering additions
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Pending.

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/839
- Part 2: pending
- Part 3: pending

## Reviewer Follow-up
- External review pass 1 output: pending
- Remediation PR (pass 1): pending
- External review pass 2 output: pending
- Remediation PR (pass 2): pending
