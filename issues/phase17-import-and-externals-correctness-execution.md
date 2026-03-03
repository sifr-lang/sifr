# Phase 17 Execution Checklist (Import and Externals Correctness)

Status: in progress (2026-03-04)
Owner: phase_17 execution loop
Reference phase doc: `.cursor/plans/main/phases/17_import_and_externals_correctness.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `./scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_17_1 Frontend-Only Check Path
status: done (2026-03-04, PR #813)

- [x] Ensure `check` stops after frontend/type phases
- [x] Remove codegen/runtime coupling from check flow
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [x] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- check demos/m17_1_frontend_only_check_path_demo.sifr` -> `no errors found`.
- Positive path: `cargo test -q -p sifr_driver` -> pass (includes `test_check_only_reports_frontend_phases`).
- Negative path: `cargo run -q -p sifr -- check <temp_type_error_file.sifr>` -> exits `1` with `type mismatch`.
- Milestone demo: `cargo run -q -p sifr -- run demos/m17_1_frontend_only_check_path_demo.sifr` -> prints `m17_1 frontend-only check path demo:` and `17`.

## Part 2: milestone_17_2 Non-Main Externals Resolution
status: pending

- [ ] Resolve stdlib/local externals in non-main modules
- [ ] Ensure multi-file projects type-check consistently
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

## Part 3: milestone_17_3 Test and Constant Import Parity
status: pending

- [ ] Align `sifr test` import behavior with regular compilation
- [ ] Support local-module constant imports in externals model
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/813 (open)
- Part 2: pending
- Part 3: pending
