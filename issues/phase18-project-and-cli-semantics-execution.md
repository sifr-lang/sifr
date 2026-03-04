# Phase 18 Execution Checklist (Project and CLI Semantics Correctness)

Status: in progress (2026-03-04)
Owner: phase_18 execution loop
Reference phase doc: `.cursor/plans/main/phases/18_project_and_cli_semantics_correctness.md`

Loop per part: Work -> Validate -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [x] Scope remains constrained to the current part definition-of-done
- [x] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone quality-contract checks include at least one positive-path and one negative-path validation
- [ ] Full local suite passes: `./scripts/run_all_tests.sh`
- [ ] Milestone demo runs successfully before opening each part PR
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_18_1 Run/Build Semantics Alignment
status: done (2026-03-04, PR #TBD)

- [x] Align project detection and compilation scope between `run` and `build`
- [x] Positive-path validation recorded
- [x] Negative-path validation recorded
- [x] Run milestone demo
- [x] Run full local suite
- [ ] Open PR, review, and merge
- [x] Mark part complete in phase doc and this checklist

Validation evidence:
- Positive path: `cargo run -q -p sifr -- run demos/m18_1_run_build_semantics_alignment_demo/main.sifr` -> prints `m18_1 run/build alignment demo:` and `aligned`.
- Positive path: `cargo run -q -p sifr -- build demos/m18_1_run_build_semantics_alignment_demo/main.sifr -o <tmp_output_dir>` -> succeeds and emits compiled binary path.
- Positive path: `cargo test -q -p sifr` -> pass (includes CLI mode resolver tests).
- Negative path: `cargo run -q -p sifr -- run <tmp_project_with_missing_module>/main.sifr` -> exits `1` with `[main] unknown module 'missing_mod'`.
- Negative path: `cargo run -q -p sifr -- build <tmp_project_with_missing_module>/main.sifr -o <tmp_output_dir>` -> exits `1` with `[main] unknown module 'missing_mod'`.
- Full suite: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` -> pass.

## Part 2: milestone_18_2 Auto-Detection Rule Tightening
status: pending

- [ ] Replace over-aggressive auto project mode with explicit, documented rules
- [ ] Ensure nearby scratch files do not unexpectedly break single-file runs
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

## Part 3: milestone_18_3 CLI Contract and Regression Suite
status: pending

- [ ] Document stable CLI semantics and edge cases
- [ ] Add regression tests for command-mode behavior
- [ ] Positive-path validation recorded
- [ ] Negative-path validation recorded
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

## PR Log
- Part 1: pending
- Part 2: pending
- Part 3: pending
