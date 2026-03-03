# Phase 15 Execution Checklist (Baseline Reconciliation)

Status: In Progress
Owner: phase_15 execution loop
Reference phase doc: `.cursor/plans/main/phases/15_baseline_reconciliation.md`

Loop per part: Work -> Validate -> Demo -> PR -> Review -> Merge -> Mark Done

## Global Gates (apply to every part)
- [ ] Scope remains constrained to the current part definition-of-done
- [ ] Root cause addressed (no superficial workaround/fallback)
- [ ] Milestone demo runs with `cargo run -q -p sifr -- run demos/<milestone_demo>.sifr` (waivable for docs-only parts)
- [ ] Full local suite passes: `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh` (waivable for docs-only parts)
- [ ] PR opened, reviewed, and merged before starting next part
- [ ] Roadmap/phase/issues docs updated with latest status and merged PR links

## Part 1: milestone_15_1 Canonical Backlog Reconciliation
status: done (2026-03-03, PR #793)

- [x] Build one canonical backlog file with deduplicated IDs
- [x] Normalize severity to `P0`-`P3`
- [x] Tag each finding with owning future phase
- [x] Add demo file `demos/milestone_15_1_canonical_backlog_demo.sifr`
- [x] Run milestone demo
- [x] Run full local suite (waived by user for docs-only phase scope)
- [x] Open PR, review, and merge (https://github.com/yaseralnajjar/sifr/pull/793)
- [x] Mark part complete in phase doc and this checklist

## Part 2: milestone_15_2 Phase Contract Definition
status: in_progress

- [x] Define entry criteria for every phase (`15`-`36`)
- [x] Define exit criteria for every phase (`15`-`36`)
- [x] Define mandatory local validation steps for every phase (`15`-`36`)
- [x] Ensure every phase gate maps to at least one concrete validation command
- [x] Add demo file `demos/milestone_15_2_phase_contracts_demo.sifr`
- [x] Run milestone demo
- [x] Run full local suite (waived by user for docs-only phase scope)
- [ ] Open PR, review, and merge
- [ ] Mark part complete in phase doc and this checklist

## Part 3: milestone_15_3 Stakeholder Sign-off Snapshot
status: pending

- [ ] Record sign-off decision over reconciled backlog + phase contracts
- [ ] Capture deferred risks with links to backlog issues
- [ ] Update roadmap and phase status to reflect Phase 15 completion
- [ ] Add demo file `demos/milestone_15_3_signoff_snapshot_demo.sifr`
- [ ] Run milestone demo
- [ ] Run full local suite
- [ ] Open PR, review, and merge
- [ ] Mark phase complete in all planning docs

## PR Log
- Part 1: https://github.com/yaseralnajjar/sifr/pull/793 (merged)
- Part 2: pending
- Part 3: pending
