# Phase 14 Gap 2: Promote Full IR Module Assembly to Production Path

Date: 2026-02-25  
Status: Done  
Parent: `issues/216-phase14-codegen-architecture-closeout-epic.md`
Merged PR: `#785`

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

## WS0 Emitter Ownership Matrix (Completed)

Matrix scope: emitter modules listed in the Phase 14 execution plan, mapped to IR-first lowering/assembly entrypoints now used in production module generation.

| Emitter file | IR-first entrypoints (owner path) | Production ownership summary |
|---|---|---|
| `crates/sifr_codegen/src/stmt_emitter.rs` | `try_emit_structured_stmt` -> `try_lower_simple_stmt_with_scope_result` (`lib.rs`, `lower_stmt.rs`) | Structured-first stmt emission with legacy bridge as non-default sink |
| `crates/sifr_codegen/src/expr_emitter.rs` | `try_emit_structured_expr` -> `try_lower_leaf_expr_result` (`lib.rs`, `lower_expr.rs`) | Structured-first expr emission with registry + lowering path |
| `crates/sifr_codegen/src/class_emitter.rs` | `emit_module_body`/`emit_module` assembled item flow (`module_body.rs`, `lib.rs`) + `push_syn_items_from_source` boundary | Class emission routed through assembled module-item pipeline |
| `crates/sifr_codegen/src/function_emitter.rs` | `emit_module_body`/`emit_module` assembled item flow (`module_body.rs`, `lib.rs`) | Function emission routed through assembled module-item pipeline |
| `crates/sifr_codegen/src/class_method_emitter.rs` | Class/module assembly via `emit_class` + module item collection | Class method generation participates in assembled item flow |
| `crates/sifr_codegen/src/match_emitter.rs` | `try_lower_simple_match_stmt` (`lower_stmt.rs`) + structured render path | Match statements have structural lowering path for simple shapes |
| `crates/sifr_codegen/src/method_call_emitter.rs` | `try_emit_method_via_registry` (`intrinsic_method_emitters.rs`) + `try_emit_structured_expr` (`lib.rs`) | Method-call emission is registry-first before bridge fallback |
| `crates/sifr_codegen/src/operator_protocol_emitters.rs` | `emit_operator_impls`/`emit_protocol_impls` via assembled module-item flow | Operator/protocol impl generation wired into module assembly |
| `crates/sifr_codegen/src/slice_emitter.rs` | `try_lower_simple_slice_expr` (`lower_expr.rs`) + renderer path | Slice lowering available in structured expr lowering path |
| `crates/sifr_codegen/src/intrinsic_method_emitters.rs` | `try_emit_intrinsic_via_registry`, `try_emit_method_via_registry`, strict registry arg lowering | Intrinsic/method lowering registry-first on IR expressions |
| `crates/sifr_codegen/src/expr_ref_emitter.rs` | `emit_parenthesized_expr` + structured expr entry in `try_emit_structured_expr` | Ref/compare helpers consume structured expr output where available |
| `crates/sifr_codegen/src/expr_render_helpers.rs` | `try_lower_registry_expr_result` + `render_expr`/rewrites | Helper rendering uses explicit result-based lowering contracts |
| `crates/sifr_codegen/src/type_emitters.rs` | Protocol/enum/newtype emitters composed into module item assembly | Type item generation integrated with module assembly |
| `crates/sifr_codegen/src/stmt_support_emitter.rs` | `emit_lowered_stmts` sink for lowered `RustStmt` vectors | Structured stmt vectors render through common stmt sink |

Assembly and pass orchestration owners:
1. `crates/sifr_codegen/src/module_body.rs` (`emit_module_body`, `push_syn_items_from_source`)
2. `crates/sifr_codegen/src/lower_item.rs` (`try_lower_simple_module_constant_item_result`)
3. `crates/sifr_codegen/src/entrypoints.rs` (`generate_rust*` item assembly + validation + render)
4. `crates/sifr_codegen/src/lib.rs` (`generate_rust_with_stdlib`, `generate_rust_multi`, `emit_module`, structured wrappers)

---

## Suggested PR Slices

1. Slice A: Add structured top-level item-lowering functions in `lower_item.rs`.
2. Slice B: Refactor `emit_module` + `module_body` to item-first assembly.
3. Slice C: Align `entrypoints.rs` and `generate_rust_multi` contract + tests.
