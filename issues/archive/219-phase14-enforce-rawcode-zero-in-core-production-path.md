# Phase 14 Gap 3: Enforce RawCode-Zero in Core Production Paths

Date: 2026-02-25  
Status: Done  
Parent: `issues/216-phase14-codegen-architecture-closeout-epic.md`
Merged PR: `#786`

---

## Problem

Core production codegen still emits `RawCode` nodes for user/module flow and stdlib preamble insertion.

Evidence:
- `crates/sifr_codegen/src/module_body.rs:39` (`RustItem::RawCode(emitted)`)
- `crates/sifr_codegen/src/module_constants.rs:19` (`RustItem::RawCode(fallback_item)`)
- `crates/sifr_codegen/src/lib.rs:411` (`RustItem::RawCode(stdlib_preamble.clone())`)
- `crates/sifr_codegen/src/stmt_support_emitter.rs:34` (`emit_lowered_stmts` has `RustStmt::RawCode` branch)
- `crates/sifr_codegen/src/expr_render_helpers.rs:224` (`RustExpr::RawCode` passthrough rewrite branch)
- `crates/sifr_codegen/src/ir_imports.rs:274` (`RustType::RawCode` import-need path)

Support evidence in strict checklist:
- `internal_docs/phases/14_codegen_architecture_finish_checklist.md:162`

---

## Root Cause

The migration preserved bridge paths that convert string output to `RawCode` instead of lowering into typed IR.  
Stdlib preamble collection also remains string-based and inserted as one raw block.

---

## Desired End State

1. Core production output for user code and preamble is fully structured IR.
2. `RawCode` is not used in production assembly paths.
3. Any remaining `RawCode` support exists only as non-production/testing escape hatch.

---

## Scope

### In scope
- `crates/sifr_codegen/src/module_body.rs`
- `crates/sifr_codegen/src/module_constants.rs`
- `crates/sifr_codegen/src/lib.rs` (stdlib preamble insertion path)
- `crates/sifr_codegen/src/lower_expr.rs`, `crates/sifr_codegen/src/lower_stmt.rs`, `crates/sifr_codegen/src/lower_item.rs` (`*_raw` helpers and bridge usage)
- `crates/sifr_codegen/src/stmt_support_emitter.rs` (`emit_lowered_stmts` raw branch)
- `crates/sifr_codegen/src/expr_render_helpers.rs` (raw passthrough rewrite branch)
- `crates/sifr_codegen/src/preamble.rs` and `crates/sifr_codegen/src/lib.rs` type-mapping callsites as needed
- `crates/sifr_codegen/src/stdlib_filter.rs` if required to output structured items instead of text

### Out of scope
- Structural-pass hard fail policy when raw appears (covered in issue 220)

---

## Implementation Plan

1. Eliminate user/module raw wrapping:
   - remove/replace `RustItem::RawCode(emitted)` body drain flow
   - remove/replace module-constant fallback raw insertion with structured item lowering

2. Replace stdlib preamble raw insertion:
   - stop appending preamble as a raw string block
   - convert preamble assembly to structured `RustItem` list before final render

3. Remove or isolate raw helper APIs from production:
   - `lower_expr_raw`
   - `lower_stmt_raw`
   - `lower_item_raw`
   Keep only under `#[cfg(test)]` if still needed for unit tests.

4. Add hard assertions in production assembly that no `RawCode` reaches final `RustFile`.

5. Add explicit type-level raw audit:
   - ensure production `sifr_type_to_rust_type` mappings never emit `RustType::RawCode`,
   - add regression tests around alias/complex type mapping to prove no `RustType::RawCode` leakage.

6. Remove dead raw passthrough branches in structured helpers once production raw is eliminated:
   - `emit_lowered_stmts` `RustStmt::RawCode` handling (production path)
   - `rewrite_stdlib_constant_idents_in_expr` raw passthrough branch

---

## Acceptance Criteria

1. No `RustItem::RawCode` injected by module body/constants/preamble production paths.
2. Final assembled production `RustFile` contains no raw item/stmt/expr nodes.
3. `lower_*_raw` helpers are not available in production flow.
4. `emit_lowered_stmts` production path has no `RustStmt::RawCode` branch usage.
5. `rewrite_stdlib_constant_idents_in_expr` no longer needs raw passthrough in production path.
6. `sifr_type_to_rust_type` cannot produce `RustType::RawCode` for production-reachable types.
7. Phase 14 strict `RawCode`-zero core-path requirement is satisfied.

---

## Validation

1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `scripts/run_e2e_pass.sh`
4. `cargo test -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture`
5. `cargo run -q -p sifr -- run demos/milestone_codegen_structural_passes_demo.sifr`

---

## Suggested PR Slices

1. Slice A: Remove module body/constants `RawCode` insertion.
2. Slice B: Convert stdlib preamble insertion to structured items.
3. Slice C: Remove/guard raw helper APIs and add no-raw assertions.
