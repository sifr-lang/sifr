# Review: wave_psp_iter_fix_2 (Canonical Iterator HIR)

**Phase:** ad-hoc-canonical-iteration-model-and-lazy-parity-closure
**Wave:** wave_psp_iter_fix_2
**Date:** 2026-03-20
**Review:** Pass 1 (self-review)

## Scope

From `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`:

> **wave_psp_iter_fix_2: Canonical Iterator HIR**
>
> Scope:
> - add dedicated iterator HIR for protocol entry, adapters, and explicit collection
> - lower `for`, `iter`, `next`, `reversed`, `map`, `filter`, `zip`, `enumerate`, generator expressions, and comprehension sources through the same iterator IR family
> - remove generic builtin-call lowering for iterator operations where canonical HIR exists
>
> Definition of done:
> - iterator semantics are represented structurally in HIR,
> - `for` and builtin lazy operations share one lowering path,
> - and HIR snapshots can directly assert iterator semantics.

## Implementation Summary

### 1. Dedicated Iterator HIR Nodes

**Status:** ✅ Complete

**Changes in `crates/sifr_hir/src/hir_nodes.rs`:**

```rust
pub enum HirIteratorOp {
    Iter,       // Protocol entry / explicit iterator creation
    Next,       // Get next element from iterator
    Reversed,   // Reverse iteration
    Map,        // Map adapter
    Filter,     // Filter adapter
    Zip,        // Zip adapter
    Enumerate,  // Enumerate adapter
}

pub enum HirExpr {
    // ... existing variants ...
    IteratorCall {
        op: HirIteratorOp,
        args: Vec<HirExpr>,
        ty: Type,
    },
}
```

### 2. Canonical Lowering Path

**Status:** ✅ Complete

All iterator operations now lower through `HirExpr::IteratorCall`:

| Operation | Location | Status |
|----------|----------|--------|
| `for` loop | `lower/statements.rs:2075` | ✅ Uses `IteratorCall::Iter` |
| `iter()` | `lower/expressions.rs:663` | ✅ Uses `IteratorCall::Iter` |
| `next()` | `lower/expressions.rs:694` | ✅ Uses `IteratorCall::Next` |
| `reversed()` | `lower/expressions.rs:1344` | ✅ Uses `IteratorCall::Reversed` |
| `enumerate()` | `lower/expressions.rs:1418` | ✅ Uses `IteratorCall::Enumerate` |
| `zip()` | `lower/expressions.rs:1447` | ✅ Uses `IteratorCall::Zip` |
| `map()` | `lower/expressions.rs:1522` | ✅ Uses `IteratorCall::Map` |
| `filter()` | `lower/expressions.rs:1544` | ✅ Uses `IteratorCall::Filter` |
| List comprehension | `lower/expressions.rs:3560` | ✅ Uses `IteratorCall::Iter` |
| Set comprehension | `lower/expressions.rs:3612` | ✅ Uses `IteratorCall::Iter` |
| Dict comprehension | `lower/expressions.rs:3693` | ✅ Uses `IteratorCall::Iter` |
| Generator expression | `lower/expressions.rs:3764` | ✅ Uses `IteratorCall::Iter` |

### 3. Legacy Fallback Removal

**Status:** ✅ Complete

Iterator operations no longer fall back to generic `HirExpr::Call` with string function names. All explicit checks in `lower/expressions.rs` return `HirExpr::IteratorCall` directly.

Test verification in `lower/expressions_tests.rs`:
- `test_for_loop_lowers_through_iter_protocol_call` - verifies for loop uses `IteratorCall`
- `test_iterator_builtins_lower_to_canonical_iterator_call_nodes` - verifies no legacy builtin string names remain in HIR

### 4. Codegen Integration

**Status:** ✅ Complete

**Files updated:**
- `sifr_codegen/src/lower_expr.rs` - Leaf expression handling for `IteratorCall`
- `sifr_codegen/src/intrinsic_method_emitters.rs` - Registry lowering for iterator ops
- `sifr_codegen/src/hir_analysis/traversal.rs` - Expression traversal
- `sifr_codegen/src/error_refs.rs` - Error reference tracking
- `sifr_codegen/src/stmt_support_emitter.rs` - For loop statement support
- `sifr_codegen/src/lower_stmt.rs` - Statement lowering

### 5. Verification

**E2E Test:**
- `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_2_canonical_iterator_hir.sifr` ✅

**Demo:**
- `demos/ad_hoc_iter_fix_wave2_canonical_hir_demo.sifr` ✅

**Unit Tests:**
- `test_for_loop_lowers_through_iter_protocol_call` ✅
- `test_iterator_builtins_lower_to_canonical_iterator_call_nodes` ✅

## Completeness Assessment

| Criterion | Status |
|-----------|--------|
| Iterator semantics represented structurally in HIR | ✅ |
| `for` and builtin lazy operations share one lowering path | ✅ |
| HIR snapshots can directly assert iterator semantics | ✅ |
| All required iterator operations covered | ✅ |
| Legacy generic builtin-call fallback removed | ✅ |

## Known Limitations / Notes

1. **Codegen path:** The `IteratorCall` is currently lowered to Rust function calls (e.g., `iter()`, `next()`) via the registry system. This is appropriate for wave 2 which focused on HIR representation. Wave 3 will implement concrete iterator codegen pipelines.

2. **Protocol entry:** The `Iter` operation serves as the canonical protocol entry point for iteration, which is used by for loops and comprehensions.

3. **Pre-existing clippy warnings:** There are some clippy pedantic warnings in the codebase (e.g., in `lower_bytes_literal`), but none related to the wave 2 iterator HIR changes.

## Conclusion

The wave_psp_iter_fix_2 implementation is **complete** relative to its scope. All required iterator operations are represented as dedicated `HirIteratorOp` variants, for loops and builtin operations share the canonical lowering path through `IteratorCall`, and the legacy generic builtin-call fallback has been removed. The definition of done criteria are satisfied.

**Recommendation:** Ready to merge / merged (PR #1345).
