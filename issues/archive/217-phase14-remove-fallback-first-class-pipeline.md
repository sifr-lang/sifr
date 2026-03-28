# Phase 14 Gap 1: Remove Fallback-First-Class Production Routing

Date: 2026-02-25  
Status: Done  
Parent: `issues/216-phase14-codegen-architecture-closeout-epic.md`
Merged PR: `#784`

---

## Problem

Production codegen still routes statement/expression emission to legacy string fallback as a normal control path.

Evidence:
- `crates/sifr_codegen/src/lib.rs:1162`
- `crates/sifr_codegen/src/lib.rs:1166`
- `crates/sifr_codegen/src/lib.rs:1179`
- `crates/sifr_codegen/src/lib.rs:1183`
- `crates/sifr_codegen/src/stmt_support_emitter.rs:6` (`emit_generator_init_stmt` calls `emit_expr`/`emit_stmt`)

This violates the intended Phase 14 architecture where structured lowering is primary and fallback is no longer a first-class path.

---

## Root Cause

`emit_stmt` and `emit_expr` currently treat `try_emit_structured_*` as opportunistic and always retain `emit_*_fallback` as the main escape hatch.  
This preserves legacy behavior but blocks the phase’s architectural end-state.

---

## Desired End State

1. Structured lowering/rendering is the production path for stmt/expr emission.
2. Legacy fallback emitters are not used as default production routing.
3. Any unsupported shape fails through structured `CodegenError` paths (or explicit narrow bridge paths that are intentionally temporary and tracked).

---

## Scope

### In scope
- `crates/sifr_codegen/src/lib.rs` (`emit_stmt`, `emit_expr`, structured dispatch plumbing)
- `crates/sifr_codegen/src/lower_stmt.rs`
- `crates/sifr_codegen/src/lower_expr.rs`
- `crates/sifr_codegen/src/stmt_support_emitter.rs` (`emit_generator_init_stmt` routing)
- `crates/sifr_codegen/src/function_emitter.rs` (generator init callsites)
- Any helper modules needed to make structured lowering complete for currently fallback-routed core cases.

### Out of scope
- Full module item assembly refactor (covered in issue 218)
- RawCode-zero in stdlib/preamble path (covered in issues 219/220)

---

## Implementation Plan

1. Add explicit structured-lowering result routing for stmt/expr:
   - `Ok(lowered)` -> render directly.
   - `Err(e)` -> surface controlled codegen failure or route through tightly scoped, audited adapter (not generic fallback emitter).

2. Remove direct calls from production wrappers to:
   - `emit_stmt_fallback`
   - `emit_expr_fallback`

3. Keep legacy emitters only if needed for:
   - tests
   - compatibility scaffolding behind explicit non-production gates
   - migration tooling

4. Add guard tests that fail if fallback routing is reintroduced in production wrappers.

5. Explicitly address generator-init path:
   - remove reliance on `emit_generator_init_stmt` string writes for structured flow, or
   - convert generator-init lowering to IR-first path used by function lowering,
   - ensure no hidden wrapper-level fallback dependency remains there.

---

## Acceptance Criteria

1. `crates/sifr_codegen/src/lib.rs` no longer routes `emit_stmt`/`emit_expr` to `emit_*_fallback` as a default path.
2. Production stmt/expr rendering path is structured-first and explicit.
3. New/updated tests assert wrapper-level fallback reintroduction is forbidden.
4. Behavior parity maintained on E2E pass corpus.
5. Generator initialization no longer depends on legacy wrapper fallback routing.

---

## Validation

1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `scripts/run_e2e_pass.sh`
4. `cargo test -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture`

---

## WS0 Coverage Inventory (Completed)

Corpus basis for reachability marker:
1. `crates/sifr/tests/e2e/pass/codegen_structured_ratio_gate.sifr`
2. `demos/milestone_codegen_stmt_expr_migration_demo.sifr`
3. Ratio gate verification: `stmt=8/9`, `expr=1/1` (`cargo test -q -p sifr --test e2e test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus -- --nocapture`)

### `HirExpr` variant inventory

| Variant | Status | Reachability marker (phase14 corpus) |
|---|---|---|
| `IntLiteral` | `structured-ready` | `observed` |
| `FloatLiteral` | `structured-ready` | `not-observed` |
| `StringLiteral` | `structured-ready` | `observed` |
| `BoolLiteral` | `structured-ready` | `observed` |
| `NoneLiteral` | `structured-ready` | `not-observed` |
| `Name` | `structured-ready` | `observed` |
| `BinOp` | `structured-ready` | `observed` |
| `UnaryOp` | `structured-ready` | `not-observed` |
| `Compare` | `structured-ready` | `observed` |
| `BoolOp` | `structured-ready` | `not-observed` |
| `Call` | `structured-ready` | `observed` |
| `IfExpr` | `structured-ready` | `observed` |
| `RangeLiteral` | `structured-ready` | `observed` |
| `ListLiteral` | `structured-ready` | `observed` |
| `SetLiteral` | `structured-ready` | `not-observed` |
| `DictLiteral` | `structured-ready` | `not-observed` |
| `TupleLiteral` | `structured-ready` | `not-observed` |
| `Index` | `structured-ready` | `not-observed` |
| `MethodCall` | `structured-ready` | `not-observed` |
| `ContainsOp` | `structured-ready` | `not-observed` |
| `FString` | `structured-ready` | `observed` |
| `Slice` | `structured-ready` | `not-observed` |
| `WalrusExpr` | `structured-ready` | `not-observed` |
| `FieldAccess` | `legacy-dependent` | `not-observed` |
| `ConstructorCall` | `structured-ready` | `not-observed` |
| `QuestionMark` | `structured-ready` | `not-observed` |
| `OkWrap` | `structured-ready` | `not-observed` |
| `ErrWrap` | `structured-ready` | `not-observed` |
| `SuperCall` | `structured-ready` | `not-observed` |
| `Lambda` | `structured-ready` | `not-observed` |
| `ListComp` | `structured-ready` | `not-observed` |
| `DictComp` | `structured-ready` | `not-observed` |
| `SetComp` | `structured-ready` | `not-observed` |
| `GeneratorExpr` | `structured-ready` | `not-observed` |
| `EnumVariant` | `structured-ready` | `not-observed` |

### `HirStmt` variant inventory

| Variant | Status | Reachability marker (phase14 corpus) |
|---|---|---|
| `Let` | `structured-ready` | `observed` |
| `Assign` | `structured-ready` | `not-observed` |
| `AugAssign` | `structured-ready` | `observed` |
| `Return` | `structured-ready` | `not-observed` |
| `Expr` | `structured-ready` | `observed` |
| `If` | `structured-ready` | `observed` |
| `While` | `structured-ready` | `observed` |
| `For` | `structured-ready` | `observed` |
| `Break` | `structured-ready` | `not-observed` |
| `Continue` | `structured-ready` | `not-observed` |
| `TupleUnpack` | `structured-ready` | `not-observed` |
| `StarUnpack` | `structured-ready` | `not-observed` |
| `Pass` | `structured-ready` | `not-observed` |
| `Assert` | `structured-ready` | `observed` |
| `Raise` | `structured-ready` | `not-observed` |
| `TryExcept` | `legacy-dependent` | `not-observed` |
| `FieldAssign` | `structured-ready` | `not-observed` |
| `SubscriptAssign` | `structured-ready` | `not-observed` |
| `NestedSubscriptAssign` | `structured-ready` | `not-observed` |
| `SubscriptAugAssign` | `structured-ready` | `not-observed` |
| `AttributeAugAssign` | `structured-ready` | `not-observed` |
| `AttributeSubscriptAssign` | `structured-ready` | `not-observed` |
| `Delete` | `structured-ready` | `not-observed` |
| `Yield` | `structured-ready` | `not-observed` |
| `With` | `legacy-dependent` | `not-observed` |
| `NestedFunction` | `legacy-dependent` | `not-observed` |
| `Match` | `structured-ready` | `not-observed` |

---

## Suggested PR Slices

1. Slice A: Wrapper routing refactor in `lib.rs` (remove generic fallback default).
2. Slice B: Fill structured lowering holes in `lower_stmt.rs`/`lower_expr.rs`.
3. Slice C: Add regression guards + ratio/parity assertions.
