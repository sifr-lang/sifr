# Phase 14 Remaining Work Loop Plan

This document tracks the remaining implementation work from:

- `.cursor/plans/main/phases/14_codegen_architecture.md`
- `.cursor/plans/main/phases/14_codegen_architecture_execution.md`

Execution loop per part:

1. Implement root-cause fix
2. Validate (tests + clippy + targeted e2e)
3. Run milestone demo
4. Open PR
5. Review PR
6. Merge PR
7. Update phase checklist/roadmap status

---

## Part 1: Differential Parity Harness (root cause: no enforced old-vs-new parity gate)

- [x] Add explicit codegen mode support (`StructuredPreferred` vs `LegacyOnly`)
- [x] Wire mode through driver compile API for test harness access
- [x] Add differential harness test utility (compile + run both modes)
- [x] Add corpus parity test set focused on stmt/expr migration surfaces
- [x] Ensure both modes produce identical runtime behavior on selected corpus
- [x] Validation:
  - [x] `cargo test -p sifr --test e2e test_codegen_differential_*`
  - [x] `cargo test -p sifr_codegen`
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part 2: Stmt/Expr Migration Closeout Gates (root cause: incomplete measurable closeout)

- [x] Add measurable structured-lowering gate for stmt/expr paths
- [x] Verify semantic transform coverage (`elif`, `for/else`, `while/else`) with explicit tests
- [x] Verify `emit_* -> lower_* -> IR -> render` dual path in active entrypoints
- [x] Validate `>=80%` structured lowering threshold for tracked corpus
- [x] Validation:
  - [x] `cargo test -p sifr_codegen`
  - [x] targeted e2e tests for control-flow and loop-else paths
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part 3: Milestone and Phase Closeout (root cause: checklist/roadmap drift)

- [x] Update `.cursor/plans/main/phases/14_codegen_architecture_execution.md`:
  - [x] mark completed Part 1/2 checklist entries
  - [x] set `milestone_codegen_stmt_expr_migration` to `done`
  - [x] mark global guards that were executed for this closeout loop
- [x] Update `.cursor/plans/main/phases/14_codegen_architecture.md` status markers if needed
- [x] Update `.cursor/plans/main/roadmap.md`:
  - [x] set Phase 14 status to `completed`
- [x] Validation:
  - [x] final `cargo test -p sifr_codegen`
  - [x] final `cargo clippy -p sifr_codegen -- -D warnings`
  - [x] final targeted `cargo test -p sifr --test e2e`
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
- [x] PR loop complete (open -> review -> merge)
