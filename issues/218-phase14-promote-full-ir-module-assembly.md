# Phase 14 Gap 2: Promote Full IR Module Assembly to Production Path

Date: 2026-02-25  
Status: Open  
Parent: `issues/216-phase14-codegen-architecture-closeout-epic.md`

---

## Problem

Module codegen is still string-emitter centric for classes/functions and then drained into `RustItem::RawCode`.

Evidence:
- `crates/sifr_codegen/src/lib.rs:946`
- `crates/sifr_codegen/src/lib.rs:954`
- `crates/sifr_codegen/src/lib.rs:955`
- `crates/sifr_codegen/src/function_emitter.rs:8`
- `crates/sifr_codegen/src/class_emitter.rs:9`
- `crates/sifr_codegen/src/module_body.rs:39`
- `crates/sifr_codegen/src/stmt_support_emitter.rs:6`
- `crates/sifr_codegen/src/function_emitter.rs:209`

This is incompatible with the target architecture of building full module output as IR items and rendering once.

---

## Root Cause

`emit_module` orchestrates legacy emitters (`emit_class`, `emit_function`) that write to a string buffer.  
`module_body` then captures string chunks and wraps them in `RustItem::RawCode`, preserving legacy architecture under an IR wrapper.

---

## Desired End State

1. Module assembly is item-first (`RustItem`) for user code, not string-first.
2. Classes/functions are lowered into structured IR items.
3. No drain-and-wrap pattern for module body generation.
4. Single render sink from assembled `RustFile`.

---

## Scope

### In scope
- `crates/sifr_codegen/src/lib.rs` (`emit_module` orchestration)
- `crates/sifr_codegen/src/module_body.rs`
- `crates/sifr_codegen/src/function_emitter.rs`
- `crates/sifr_codegen/src/stmt_support_emitter.rs`
- `crates/sifr_codegen/src/class_emitter.rs`
- `crates/sifr_codegen/src/lower_item.rs`
- `crates/sifr_codegen/src/entrypoints.rs` and multi-module flow (`generate_rust_multi`)

### Out of scope
- RawCode elimination from stdlib preamble and module constants fallback (covered by issue 219)

---

## Implementation Plan

1. Introduce item-lowering entrypoints:
   - `lower_function_item(...) -> RustItem`
   - `lower_class_items(...) -> Vec<RustItem>`
   - and equivalent for other top-level constructs as needed.

2. Refactor `emit_module` to:
   - collect all user module items as IR
   - avoid intermediary string emission for classes/functions
   - assemble directly into `RustFile.items`

3. Remove `module_body::drain_emitted_output_item` usage for class/function generation.

3.1. Remove generator-init string-emission dependency from class/function generation path:
   - replace `emit_generator_init_stmt` string writes with IR lowering in top-level function item generation.
   - keep generator semantics unchanged.

4. Align entrypoints (`generate_rust_test`, `generate_rust_multi`) to the same module IR assembly contract.

5. Add tests proving:
   - no module body class/function output is wrapped via `RustItem::RawCode`
   - module assembly paths remain consistent across single-module/test/multi-module entrypoints.

---

## Acceptance Criteria

1. User class/function module body is produced as structured IR items, not `RawCode` wrappers.
2. `emit_module` is IR assembly orchestration, not legacy-string orchestration.
3. `generate_rust`, `generate_rust_test`, and `generate_rust_multi` share the same item-first assembly model.
4. Existing behavior parity is preserved (compile + run parity).
5. Generator function initialization statements are represented in structured IR item bodies (no string bridge).

---

## Validation

1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `scripts/run_e2e_pass.sh`
4. `cargo run -q -p sifr -- run demos/milestone_codegen_stmt_expr_migration_demo.sifr`
5. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`

---

## Suggested PR Slices

1. Slice A: Add structured top-level item-lowering functions in `lower_item.rs`.
2. Slice B: Refactor `emit_module` + `module_body` to item-first assembly.
3. Slice C: Align `entrypoints.rs` and `generate_rust_multi` contract + tests.
