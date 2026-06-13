# Phase 27 Panic Inventory (User-Input Reachable Paths)

Date: 2026-03-07  
Owner: phase_27 execution loop

## Scope
- parser: `crates/sifr_python_parser`
- lowering/type-check: `crates/sifr_hir`
- codegen: `crates/sifr_codegen`
- orchestration/CLI: `crates/sifr_driver`, `crates/sifr`

## Active panic boundaries
- `crates/sifr_driver/src/lib.rs`
  - `run_codegen_with_boundary(...)` converts codegen panics into `CompileError` diagnostics.
- `crates/sifr/src/main.rs`
  - `run_with_panic_boundary(...)` wraps `build`, `run`, `check`, `emit`, and `test` command execution and converts panics into diagnostics.

## Inventory Summary
1. **Codegen internal invariant panics**
   - Representative sites:
     - `crates/sifr_codegen/src/lib.rs` (`panic!` on structured-lowering invariant violations)
     - `crates/sifr_codegen/src/function_emitter.rs` / `class_method_emitter.rs` / `expr_ref_emitter.rs`
   - User impact status:
     - no uncaught panic in CLI flows; routed to diagnostics by `run_codegen_with_boundary(...)`.
   - Follow-up:
     - tracked in [`phase27-panic-followups.md`](./phase27-panic-followups.md).

2. **HIR/CFG invariant panic**
   - Site:
     - `crates/sifr_hir/src/cfg.rs` (`panic!` on invalid CFG invariant)
   - User impact status:
     - no uncaught panic in CLI flows; routed to diagnostics by `run_with_panic_boundary(...)`.
   - Follow-up:
     - tracked in [`phase27-panic-followups.md`](./phase27-panic-followups.md).

3. **Parser internal unwrap/expect invariants**
   - Representative sites:
     - `crates/sifr_python_parser/src/lexer.rs`
     - `crates/sifr_python_parser/src/string.rs`
   - User impact status:
     - parser error paths remain diagnostics-first; unexpected invariant panic is caught at CLI boundary and converted to an internal diagnostic.
   - Follow-up:
     - tracked in [`phase27-panic-followups.md`](./phase27-panic-followups.md).

## Contract result for Phase 27
- User-triggerable panic crashes are eliminated from CLI/compiler entrypoints.
- Internal invariant panics are converted to deterministic diagnostics with stable exit code `3`.
