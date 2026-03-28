# Phase 14 Remaining Work Loop Plan (Re-baselined 2026-02-23)

> Status note (2026-02-24): This document is superseded by
> `internal_docs/phases/14_codegen_architecture_finish_checklist.md`
> for strict criterion-by-criterion completion tracking.
> Use the strict checklist as the active source of truth.

This document replaces stale closeout checkboxes with code-verified remaining work.

Source-of-truth inputs:

- `internal_docs/phases/14_codegen_architecture.md`
- `internal_docs/phases/14_codegen_architecture_execution.md`
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

- `crates/sifr_codegen/src/lib.rs`: `1345` lines
- Direct string emission calls in `lib.rs`: `2`
- Largest write-heavy files: `intrinsic_method_emitters.rs` (`523`), `expr_emitter.rs` (`394`), `stmt_emitter.rs` (`295`)
- Legacy emitter files: none (`legacy_expr_emitter.rs` and `legacy_stmt_emitter.rs` removed)
- `lower_stmt` production coverage: `27/27` `HirStmt` variants
  - Missing: none
- `lower_expr` production coverage: `35/35` `HirExpr` variants
  - Missing: none
- Remaining `RawCode` bridge loci in core paths: none
- Active `sifr_codegen` clippy suppressions in `lib.rs`: none
- Execution checklist drift: none (reconciled with merged PR reality).

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
  - [x] `StarUnpack` (fully lowered via structured IR `Let`/`Index`/`Range`/`MethodCall`)
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

status: done

Root cause: `emit_expr` in `lib.rs` still owns many core expression families.

- [x] Add lowering paths for missing expr variants:
  - [x] `FieldAccess` (non-`self` conservative path), `ContainsOp`
  - [x] `SuperCall`, `WalrusExpr`
  - [x] `QuestionMark`, `OkWrap`, `ErrWrap`
  - [x] Added conservative structured lowering for safe `FString` and `Lambda` subshapes
  - [x] Remaining complex families explicitly routed through conservative legacy-bridge raw-lowering path in structured mode:
    `Call`, `MethodCall`, `ConstructorCall`, `Index`, `Slice`, `DictLiteral`, `SetLiteral`,
    `ListComp`, `DictComp`, `SetComp`, `GeneratorExpr` plus unsupported `Lambda`/`FString` subshapes
- [x] Keep fallback semantics only for explicitly-documented complex residue
- [x] Add regression/unit tests for newly-lowered and bridge paths
- [x] Coverage snapshot: `35/35` expr variants explicitly covered in `lower_expr` production path
- [x] Validation:
  - [x] `cargo test -p sifr_codegen`
  - [x] targeted e2e parity tests for changed expr semantics
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part D: `lib.rs` Decomposition and Orchestration-Only End State

status: done

Root cause: core codegen logic remains concentrated in a monolithic string-emitter file.

- [x] Move remaining statement/expression emission internals out of `lib.rs`
- [x] Keep `lib.rs` focused on orchestration and entrypoint wiring
- [x] Reduce direct write-call concentration in `lib.rs` materially (`804 -> 2`)
- [x] Add guard checks to prevent regression into monolithic emission patterns
  - [x] `test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs` validates wrapper-only `emit_stmt`/`emit_expr`, line-count cap, write-call cap, and migrated legacy-module size floors
- [x] Validation:
  - [x] `cargo test -p sifr_codegen`
  - [x] targeted e2e parity tests on moved paths
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part E: Clippy Suppression Burn-down (Root-Cause)

status: done

Root cause: lint suppressions mask conversion and state-structure debt in legacy paths.

- [x] Remove `#![allow(clippy::cast_sign_loss)]`
- [x] Remove `#![allow(clippy::cast_possible_truncation)]`
- [x] Remove `#![allow(clippy::cast_possible_wrap)]`
- [x] Remove `#![allow(clippy::struct_excessive_bools)]`
- [x] Replace suppressions with checked/typed conversions and state refactors
  - [x] Replace lossy tuple/slice index casts with checked conversions in legacy expression emission paths
  - [x] Refactor bool-heavy state structs (`RustEmitter`, `IrImportNeeds`, `SharedPreludeNeeds`) into grouped state structs with <=3 bools each
- [x] Validation:
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
  - [x] `cargo test -p sifr_codegen`
  - [x] targeted e2e parity gates (`test_codegen_differential_old_vs_new_corpus_parity`, `test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus`)
- [x] Demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part F: Phase-Document Reconciliation and Final Closeout

status: done

Root cause: phase docs can claim done before code-level gates are actually complete.

- [x] Reconcile `14_codegen_architecture.md` DoD status with implemented code
- [x] Reconcile `14_codegen_architecture_execution.md` checklist with merged PR reality
- [x] Close remaining unchecked items only when code gates are actually met
- [x] Final validation:
  - [x] `cargo test -p sifr_codegen`
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
  - [x] targeted `cargo test -p sifr --test e2e` (parity + ratio gate)
- [x] Final demo check:
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`
- [x] PR loop complete (open -> review -> merge)

---

## Part G: RawCode-Zero Gate Closeout (Re-opened)

status: done

Root cause: structural-passes DoD requires zero `RawCode` in core paths, but bridge capture still exists for complex stmt/expr families.

- [x] Remove `RawCode` bridge from `StarUnpack` lowering (`lower_stmt`)
- [x] Remove `RawCode` bridge from top-level exiting `if x is None` narrowing path (`lower_stmt`)
- [x] Add conservative structured lowering for safe `FString` and `Lambda` subshapes (`lower_expr`)
- [x] Eliminate stmt fallback raw-capture for `TryExcept` by introducing structured IR lowering path
- [x] Eliminate stmt fallback raw-capture for `NestedFunction` by introducing structured IR lowering path
- [x] Eliminate expr fallback raw-capture by adding structured lowering for remaining residue:
  - [x] `Call`
  - [x] `MethodCall`
  - [x] `ConstructorCall`
  - [x] `Index`
  - [x] `Slice`
  - [x] `DictLiteral`
  - [x] `SetLiteral`
  - [x] `ListComp`
  - [x] `DictComp`
  - [x] `SetComp`
  - [x] `GeneratorExpr`
- [x] Delete `try_capture_fallback_expr_as_raw` and `try_capture_fallback_stmt_as_raw` once no callsites remain
- [x] Validation for each slice:
  - [x] `cargo test -p sifr_codegen`
  - [x] `cargo clippy -p sifr_codegen -- -D warnings`
  - [x] `cargo run -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
  - [x] `cargo test -p sifr --test e2e test_e2e_pass`
- [x] PR loop complete (open -> review -> merge)
