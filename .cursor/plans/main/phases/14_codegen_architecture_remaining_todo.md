# Phase 14 Remaining Work Loop Plan (Re-baselined 2026-02-23)

This document replaces stale closeout checkboxes with code-verified remaining work.

Source-of-truth inputs:

- `.cursor/plans/main/phases/14_codegen_architecture.md`
- `.cursor/plans/main/phases/14_codegen_architecture_execution.md`
- `crates/sifr_codegen/src/lib.rs`
- `crates/sifr_codegen/src/lower_stmt.rs`
- `crates/sifr_codegen/src/lower_expr.rs`

Execution loop per part (mandatory):

1. Implement root-cause fix
2. Validate (`cargo test`, targeted e2e, `cargo clippy -D warnings`)
3. Run milestone demo
4. Open PR
5. Review PR
6. Merge PR
7. Update phase docs/checklists

---

## Current Baseline (code-verified)

- `crates/sifr_codegen/src/lib.rs`: `4134` lines
- Direct string emission calls in `lib.rs`: `804`
- Largest write-heavy files: `lib.rs` (`804`), `intrinsic_method_emitters.rs` (`523`)
- `lower_stmt` production coverage: `21/27` `HirStmt` variants
  - Missing: `Match`, `NestedFunction`, `StarUnpack`, `TryExcept`, `With`, `Yield`
- `lower_expr` production coverage: `15/35` `HirExpr` variants
  - Missing: `Call`, `ConstructorCall`, `ContainsOp`, `DictComp`, `DictLiteral`, `ErrWrap`, `FString`, `FieldAccess`, `GeneratorExpr`, `Index`, `Lambda`, `ListComp`, `MethodCall`, `OkWrap`, `QuestionMark`, `SetComp`, `SetLiteral`, `Slice`, `SuperCall`, `WalrusExpr`
- Active `sifr_codegen` clippy suppressions in `lib.rs`: `cast_sign_loss`, `cast_possible_truncation`, `cast_possible_wrap`, `struct_excessive_bools`
- Execution checklist drift: `.cursor/plans/main/phases/14_codegen_architecture_execution.md` still has one unchecked item (`Remove at least 5 clippy suppressions...`) despite later sections claiming broader suppression removals.

---

## Part A: Re-baseline and Planning Sync

status: done

Root cause: remaining work tracking drifted from real code state.

- [x] Replace stale "all done" narrative with code-verified backlog (this file)
- [x] Add explicit variant-gap inventory for stmt/expr lowering
- [x] Add explicit clippy suppression inventory for `sifr_codegen`
- [x] Define measurable completion gates for each remaining part
- [x] Validation:
  - [x] `cargo test -p sifr_codegen`
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part B: Statement Lowering Coverage Closeout (`lower_stmt`)

status: done

Root cause: `emit_stmt` in `lib.rs` still owns unsupported statement semantics.

- [x] Add explicit lowering paths for missing stmt variants:
  - [x] `Match` (simple/lowerable pattern+guard+body forms)
  - [x] `NestedFunction` (legacy bridge via captured `RustStmt::RawCode` in structured mode)
  - [x] `StarUnpack` (lowered as constrained IR `RawCode` bridge)
  - [x] `TryExcept` (legacy bridge via captured `RustStmt::RawCode` in structured mode)
  - [x] `With` (non-context-manager protocol path)
  - [x] `Yield`
- [x] Prefer structured IR for safe subshapes; use constrained fallback only where unavoidable
- [x] Add regression tests for every newly-lowered stmt variant path
- [x] Update lowering coverage metrics and gates to reflect new coverage (`lower_stmt` variant coverage: `27/27`)
- [x] Validation:
  - [x] `cargo test -p sifr_codegen`
  - [x] targeted e2e parity tests for changed stmt semantics
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part C: Expression Lowering Coverage Closeout (`lower_expr`)

status: pending

Root cause: `emit_expr` in `lib.rs` still owns many core expression families.

- [ ] Add lowering paths for missing expr variants:
  - [ ] `Call`, `MethodCall`, `ConstructorCall`
  - [ ] `FieldAccess`, `Index`, `Slice`, `ContainsOp`
  - [ ] `DictLiteral`, `SetLiteral`
  - [ ] `ListComp`, `DictComp`, `SetComp`, `GeneratorExpr`, `Lambda`
  - [ ] `FString`, `SuperCall`, `WalrusExpr`
  - [ ] `QuestionMark`, `OkWrap`, `ErrWrap`
- [ ] Keep fallback semantics only for explicitly-documented complex residue
- [ ] Add regression/unit tests for each new lowering family
- [ ] Validation:
  - [ ] `cargo test -p sifr_codegen`
  - [ ] targeted e2e parity tests for changed expr semantics
  - [ ] `cargo clippy -p sifr_codegen -- -D warnings`
- [ ] Demo check:
  - [ ] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] PR loop complete (open -> review -> merge)

---

## Part D: `lib.rs` Decomposition and Orchestration-Only End State

status: pending

Root cause: core codegen logic remains concentrated in a monolithic string-emitter file.

- [ ] Move remaining statement/expression emission internals out of `lib.rs`
- [ ] Keep `lib.rs` focused on orchestration and entrypoint wiring
- [ ] Reduce direct write-call concentration in `lib.rs` materially
- [ ] Add guard checks to prevent regression into monolithic emission patterns
- [ ] Validation:
  - [ ] `cargo test -p sifr_codegen`
  - [ ] targeted e2e parity tests on moved paths
  - [ ] `cargo clippy -p sifr_codegen -- -D warnings`
- [ ] Demo check:
  - [ ] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] PR loop complete (open -> review -> merge)

---

## Part E: Clippy Suppression Burn-down (Root-Cause)

status: pending

Root cause: lint suppressions mask conversion and state-structure debt in legacy paths.

- [ ] Remove `#![allow(clippy::cast_sign_loss)]`
- [ ] Remove `#![allow(clippy::cast_possible_truncation)]`
- [ ] Remove `#![allow(clippy::cast_possible_wrap)]`
- [ ] Remove `#![allow(clippy::struct_excessive_bools)]`
- [ ] Replace suppressions with checked/typed conversions and state refactors
- [ ] Validation:
  - [ ] `cargo clippy -p sifr_codegen -- -D warnings`
  - [ ] `cargo test -p sifr_codegen`
- [ ] Demo check:
  - [ ] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] PR loop complete (open -> review -> merge)

---

## Part F: Phase-Document Reconciliation and Final Closeout

status: pending

Root cause: phase docs can claim done before code-level gates are actually complete.

- [ ] Reconcile `14_codegen_architecture.md` DoD status with implemented code
- [ ] Reconcile `14_codegen_architecture_execution.md` checklist with merged PR reality
- [ ] Close remaining unchecked items only when code gates are actually met
- [ ] Final validation:
  - [ ] `cargo test -p sifr_codegen`
  - [ ] `cargo clippy -p sifr_codegen -- -D warnings`
  - [ ] targeted `cargo test -p sifr --test e2e`
- [ ] Final demo check:
  - [ ] `cargo run -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
- [ ] PR loop complete (open -> review -> merge)
