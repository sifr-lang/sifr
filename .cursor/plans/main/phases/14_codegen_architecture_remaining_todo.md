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

- [ ] Add explicit codegen mode support (`StructuredPreferred` vs `LegacyOnly`)
- [ ] Wire mode through driver compile API for test harness access
- [ ] Add differential harness test utility (compile + run both modes)
- [ ] Add corpus parity test set focused on stmt/expr migration surfaces
- [ ] Ensure both modes produce identical runtime behavior on selected corpus
- [ ] Validation:
  - [ ] `cargo test -p sifr --test e2e test_codegen_differential_*`
  - [ ] `cargo test -p sifr_codegen`
  - [ ] `cargo clippy -p sifr_codegen -- -D warnings`
- [ ] Demo check:
  - [ ] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] PR loop complete (open -> review -> merge)

---

## Part 2: Stmt/Expr Migration Closeout Gates (root cause: incomplete measurable closeout)

- [ ] Add measurable structured-lowering gate for stmt/expr paths
- [ ] Verify semantic transform coverage (`elif`, `for/else`, `while/else`) with explicit tests
- [ ] Verify `emit_* -> lower_* -> IR -> render` dual path in active entrypoints
- [ ] Validate `>=80%` structured lowering threshold for tracked corpus
- [ ] Validation:
  - [ ] `cargo test -p sifr_codegen`
  - [ ] targeted e2e tests for control-flow and loop-else paths
  - [ ] `cargo clippy -p sifr_codegen -- -D warnings`
- [ ] Demo check:
  - [ ] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] PR loop complete (open -> review -> merge)

---

## Part 3: Milestone and Phase Closeout (root cause: checklist/roadmap drift)

- [ ] Update `.cursor/plans/main/phases/14_codegen_architecture_execution.md`:
  - [ ] mark completed Part 1/2 checklist entries
  - [ ] set `milestone_codegen_stmt_expr_migration` to `done`
  - [ ] mark global guards that were executed for this closeout loop
- [ ] Update `.cursor/plans/main/phases/14_codegen_architecture.md` status markers if needed
- [ ] Update `.cursor/plans/main/roadmap.md`:
  - [ ] set Phase 14 status to `completed`
- [ ] Validation:
  - [ ] final `cargo test -p sifr_codegen`
  - [ ] final `cargo clippy -p sifr_codegen -- -D warnings`
  - [ ] final targeted `cargo test -p sifr --test e2e`
- [ ] Demo check:
  - [ ] `cargo run -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
- [ ] PR loop complete (open -> review -> merge)

