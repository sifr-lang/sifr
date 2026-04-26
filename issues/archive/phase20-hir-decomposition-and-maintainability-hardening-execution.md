# Phase 20 Execution Checklist (HIR Decomposition and Maintainability Hardening)

Status: completed (2026-03-05)
Owner: phase_20 execution loop
Reference phase doc: `internal_docs/phases/20_hir_decomposition_and_maintainability_hardening.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [x] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [x] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [x] Milestone demo runs successfully before opening each part PR
- [x] PR opened, reviewed, and merged before starting next part
- [x] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 20 To-Do Plan

### Part 1: milestone_20_1 Split `lower.rs`
- [x] Decompose `crates/sifr_hir/src/lower.rs` into focused lowering modules:
  - [x] import handling
  - [x] diagnostics/error-type helpers
  - [x] class lowering
  - [x] typing hooks + function signature/body lowering
  - [x] statement lowering
  - [x] expression lowering
- [x] Keep public lowering API stable (`lower_module*`, `LoweringResult`, `ExternalDefs`, `LoweringError`)
- [x] Add/confirm targeted regression tests for lowered behavior parity
- [x] Add milestone 20.1 positive demo
- [x] Add milestone 20.1 negative regression case
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 2: milestone_20_2 Split `stdlib.rs`
- [x] Decompose intrinsic registry into focused modules:
  - [x] shared intrinsic helpers/types
  - [x] registry dispatch
  - [x] grouped intrinsic module definitions (io/json/math/test/collections/bytes/time/sys/fs/crypto/regex/uuid/platform/toml/datetime/html/calendar/compress/logging)
- [x] Preserve module names and type signatures exactly
- [x] Add milestone 20.2 positive demo for stdlib/intrinsic stability
- [x] Add milestone 20.2 negative regression case for unknown/unsupported intrinsic usage
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

### Part 3: milestone_20_3 Anti-Regrowth Guardrails
- [x] Define and document max file-size/module-boundary conventions for HIR lowering and stdlib registry
- [x] Add enforceable local/CI guard script(s) for file-size boundaries
- [x] Add review checklist for adding new lowering logic and intrinsic definitions
- [x] Add positive-path guardrail test
- [x] Add negative-path guardrail test (intentional threshold violation fixture)
- [x] Add milestone 20.3 demo (guardrails in action)
- [x] Run milestone demo + targeted tests + full local suite
- [x] Open PR, review, and merge

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
status: done (2026-03-05, PR #840)

- [x] Partition stdlib metadata/registration logic into focused modules
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_hir` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m20_2_stdlib_registry_split_demo/main.sifr` -> prints `m20_2 stdlib registry split demo:` and `\"ok\"`.
- Negative path: `cargo run -q -p sifr -- run demos/m20_2_stdlib_registry_split_demo/negative_cases/forbidden_intrinsic_import.sifr` -> exits `1` with `_sifr.* modules are internal compiler intrinsics` and `undefined function: 'sqrt'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 3: milestone_20_3 Anti-Regrowth Guardrails
status: done (2026-03-05, PR #841)

- [x] Add file-size and module-boundary conventions
- [x] Add review checklist items for new lowering additions
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `python3 scripts/check_hir_maintainability_guardrails.py` -> `HIR maintainability guardrails: PASS`.
- Positive path: `cargo run -q -p sifr -- run demos/m20_3_guardrails_demo.sifr` -> prints `m20_3 guardrails demo:` and `20`.
- Negative path: `SIFR_HIR_GUARD_MAX_OVERRIDE=100 python3 scripts/check_hir_maintainability_guardrails.py` -> exits `1` with file-size violations.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass (includes guardrail gate execution).

## PR Log
- Part 1: https://github.com/sifr-lang/sifr/pull/839
- Part 2: https://github.com/sifr-lang/sifr/pull/840
- Part 3: https://github.com/sifr-lang/sifr/pull/841

## Reviewer Follow-up
- External review pass 1 output: `reviews/phase20-review.md`
- Pass-1 triage outcome:
  - Reviewer confirmed Phase 20 implementation quality and contract adherence with no blocking defects.
  - Verified CI/local enforcement path: `.github/workflows/local-first-validation.yml` runs `scripts/run_all_tests.sh`, which now executes `scripts/check_hir_maintainability_guardrails.py`.
- Remediation PR (pass 1): https://github.com/sifr-lang/sifr/pull/842
- External review pass 2 output: `reviews/phase20-production-grade-review.md`
- Pass-2 triage outcome:
  - Reviewer confirmed phase-20 implementation is production-grade with no blocking defects.
  - No additional compiler-code changes were required after validating reviewer observations.
- Remediation PR (pass 2): https://github.com/sifr-lang/sifr/pull/843
