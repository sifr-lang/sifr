# Review: wave_psp_iter_fix_2 (Canonical Iterator HIR) - Pass 2

**Phase:** ad-hoc-canonical-iteration-model-and-lazy-parity-closure
**Wave:** wave_psp_iter_fix_2
**Date:** 2026-03-20
**Review:** Pass 2 (external review)

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
|-----------|----------|--------|
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

## Pass 2 Review: Production-Grade Readiness

### Correctness Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| Iterator semantics represented structurally in HIR | ✅ | `HirIteratorOp` enum with dedicated variants |
| `for` and builtin lazy operations share one lowering path | ✅ | All use `HirExpr::IteratorCall` |
| HIR snapshots can directly assert iterator semantics | ✅ | Unit tests verify `IteratorCall` nodes |
| All required iterator operations covered | ✅ | `iter`, `next`, `reversed`, `map`, `filter`, `zip`, `enumerate`, comprehensions, generators |
| Legacy generic builtin-call fallback removed | ✅ | No string function names in lowering |

### Maintainability Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| Code is well-organized | ✅ | Dedicated `HirIteratorOp` enum, clear lowering paths |
| No duplication | ✅ | Single lowering path for all iterator operations |
| Naming is clear | ✅ | `IteratorCall`, `HirIteratorOp` clearly named |
| Changes are localized | ✅ | Changes focused on iterator lowering |

### Regression Risk Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| Unit tests pass | ✅ | All iterator-related tests pass |
| E2E tests pass | ✅ | `phase_psp_iter_fix_2_canonical_iterator_hir.sifr` passes |
| Quick validation passes | ✅ | `scripts/run_all_tests.sh --profile quick` passes |
| No new clippy warnings introduced | ✅ | All clippy warnings are pre-existing |
| Changes are backward-compatible | ✅ | Internal HIR change, no user-facing API change |

### Known Issues

1. **Pre-existing clippy warnings**: There are several clippy warnings in the codebase (e.g., `explicit_iter_loop`, `unnecessary_wraps`, `single_match_else`, `uninlined_format_args`, `semicolon_if_nothing_returned`). These are NOT introduced by wave 2 and existed before the changes.

2. **Codegen path**: The `IteratorCall` is currently lowered to Rust function calls (e.g., `iter()`, `next()`) via the registry system. This is appropriate for wave 2 which focused on HIR representation. Wave 3 will implement concrete iterator codegen pipelines.

## Completeness Assessment

| Criterion | Status |
|-----------|--------|
| Iterator semantics represented structurally in HIR | ✅ |
| `for` and builtin lazy operations share one lowering path | ✅ |
| HIR snapshots can directly assert iterator semantics | ✅ |
| All required iterator operations covered | ✅ |
| Legacy generic builtin-call fallback removed | ✅ |
| Unit tests verify canonical lowering | ✅ |
| E2E tests pass | ✅ |
| Demo files work correctly | ✅ |

## Conclusion

The wave_psp_iter_fix_2 implementation is **production-grade ready** relative to its scoped responsibilities. All definition of done criteria are satisfied:

1. ✅ Iterator semantics are represented structurally in HIR via dedicated `HirIteratorOp` enum
2. ✅ `for` and builtin lazy operations share one lowering path through `HirExpr::IteratorCall`
3. ✅ HIR snapshots can directly assert iterator semantics (verified by unit tests)

The implementation correctly addresses the root cause identified in the phase description: builtin iterator operations are now lowered through dedicated iterator IR rather than ad hoc builtin-call paths.

**Recommendation:** ✅ **Approved for production use**

The wave achieves its objectives and provides the foundation for wave 3 (Concrete Iterator Codegen Pipelines) to build upon.
