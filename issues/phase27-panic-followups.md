# Phase 27 Panic Follow-Ups

Date: 2026-03-07

## Open Items

1. Replace HIR CFG invariant panic with typed diagnostic flow
- Owner: phase_29_verification_hardening
- Tracking issue: [`phase27-runtime-safety-and-diagnostics-execution.md`](./phase27-runtime-safety-and-diagnostics-execution.md)
- Reference: `crates/sifr_hir/src/cfg.rs`
- Target: convert `panic!` invariant failure path to typed frontend diagnostic without panic boundary dependence.

2. Convert codegen invariant panics to typed error returns at source
- Owner: phase_34_generated_code_quality
- Tracking issue: [`phase27-runtime-safety-and-diagnostics-execution.md`](./phase27-runtime-safety-and-diagnostics-execution.md)
- Reference: `crates/sifr_codegen/src/lib.rs`, `function_emitter.rs`, `class_method_emitter.rs`, `expr_ref_emitter.rs`, `operator_protocol_emitters.rs`
- Target: replace `panic!` invariants with explicit `Result<_, CompileError>` flow and phase-accurate diagnostics.

3. Audit parser invariant unwrap/expect usage for parser-only diagnostics
- Owner: phase_29_verification_hardening
- Tracking issue: [`phase27-runtime-safety-and-diagnostics-execution.md`](./phase27-runtime-safety-and-diagnostics-execution.md)
- Reference: `crates/sifr_python_parser/src/lexer.rs`, `string.rs`
- Target: prove/guard unreachable assumptions and convert unexpected states to parser diagnostics where feasible.
