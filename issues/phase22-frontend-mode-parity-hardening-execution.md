# Phase 22 Execution Checklist (Frontend Mode Parity Hardening)

Status: in_progress (2026-03-06)
Owner: phase_22 execution loop
Reference phase doc: `.cursor/plans/main/phases/22_frontend_mode_parity_hardening.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Full Phase 22 To-Do Plan

### Part 1: milestone_22_1 Canonical Frontend Entry Path
- [ ] Define one shared frontend module-orchestration path in `sifr_driver` consumed by `check`, `build`, `run`, and `test`
- [ ] Remove mode-specific lowering/resolution forks; allow only explicit mode flags for documented diagnostic surface differences
- [ ] Add regression tests to lock shared frontend behavior across single-module and project-module lowering entrypoints
- [ ] Add milestone 22.1 positive demo
- [ ] Add milestone 22.1 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 2: milestone_22_2 Project-Aware `check` Parity
- [ ] Make `sifr check` path-aware for project entries and local module resolution parity with `build`/`run`
- [ ] Ensure stdlib external resolution parity in `check` for multi-file projects
- [ ] Add tests covering known gap closure for valid local imports in `check`
- [ ] Add milestone 22.2 positive demo
- [ ] Add milestone 22.2 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 3: milestone_22_3 Cross-Mode Diagnostic and Exit Contract
- [ ] Define and document explicit parity rules for frontend diagnostics, exit codes, and ordering guarantees across `check`/`build`/`run`/`test`
- [ ] Add regression tests for equivalent frontend failures surfacing equivalent diagnostics across modes
- [ ] Add tests for deterministic ordering of frontend errors/diagnostics
- [ ] Add milestone 22.3 positive demo
- [ ] Add milestone 22.3 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

### Part 4: milestone_22_4 Parity Regression Matrix
- [ ] Add an explicit parity matrix that runs representative positive/negative fixtures through `check`, `build`, `run`, and `test` frontend paths
- [ ] Include fixtures for local imports, stdlib externals, and multi-module resolution
- [ ] Wire matrix into local validation workflow so mode drift fails before merge
- [ ] Add milestone 22.4 positive demo
- [ ] Add milestone 22.4 negative regression case
- [ ] Run milestone demo + targeted tests + full local suite
- [ ] Open PR, review, and merge

## Part 1: milestone_22_1 Canonical Frontend Entry Path
status: in_progress

- [x] Shared frontend orchestration path implemented and reused across modes
- [x] Explicit mode flag behavior documented in code/tests
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo test -q -p sifr_driver test_compile_frontend_modules_uses_explicit_diagnostic_style` -> pass.
- Positive path: `cargo test -q -p sifr_driver test_check_and_project_lowering_share_typecheck_contract` -> pass.
- Positive path: `cargo run -q -p sifr -- run demos/m22_1_canonical_frontend_entry_path_demo/main.sifr` -> prints `m22_1 canonical frontend entry path demo:` and `7`.
- Positive path: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.
- Negative path: `cargo run -q -p sifr -- run demos/m22_1_canonical_frontend_entry_path_demo/negative_cases/type_error_dependency/main.sifr` -> exits `1` with `type error: [helper] return type mismatch: expected 'int', got 'str'`.

## Part 2: milestone_22_2 Project-Aware `check` Parity
status: pending

- [ ] Project-aware `check` implementation complete
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Pending.

## Part 3: milestone_22_3 Cross-Mode Diagnostic and Exit Contract
status: pending

- [ ] Diagnostic/exit/ordering contract implemented and documented
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Pending.

## Part 4: milestone_22_4 Parity Regression Matrix
status: pending

- [ ] Parity matrix implemented and wired into local validation
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

Validation evidence:
- Pending.

## PR Log
- Part 1: pending
- Part 2: pending
- Part 3: pending
- Part 4: pending

## Reviewer Follow-up
- External review pass 1 output: pending
- Remediation PR (pass 1): pending
- External review pass 2 output: pending
- Remediation PR (pass 2): pending
