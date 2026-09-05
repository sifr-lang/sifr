# wave_psp_ext_1 Review Pass 2a (Production-Grade)

**Phase**: ad-hoc-python-source-parity-extension-waiver-reduction
**Wave**: wave_psp_ext_1 (Builtin Iterator Re-Closure)
**Review Type**: Production-Grade Review
**Reviewer**: agent
**Date**: 2026-03-18

---

## Executive Summary

This production-grade review validates that wave_psp_ext_1 correctly implements the builtin iterator re-closure for `reversed`, `enumerate`, `zip`, and `map`, ensuring:

1. **Correctness**: All four builtins now return `Iterator[T]` matching CPython semantics
2. **Deterministic Behavior**: Iterator protocol follows Rust stdlib semantics consistently
3. **No-Panic Guarantees**: All production code paths use proper error handling with `?`
4. **Governance Accuracy**: Parity ledgers correctly reflect the iterator-returning behavior

**Verdict**: APPROVED for production

---

## 1. Scope Verification

### 1.1 Wave Definition (from phase doc)

- Port predecessor builtin-iterator architecture into the legacy parity ledgers ✅
- Convert `reversed`, `enumerate`, `zip`, and `map` to true iterator-returning semantics ✅
- Revalidate `list(...)`, `tuple(...)`, `set(...)`, and `dict(...)` as the canonical materialization boundary ✅

### 1.2 Implementation Delivered

| Target | Status | Evidence |
|--------|--------|----------|
| `reversed(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator(Box::new(elem_ty))` in expressions.rs; Codegen: `registry_box_iterator_expr()` in intrinsic_method_emitters.rs |
| `enumerate(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator(Box::new(tuple_ty))` in expressions.rs; Codegen: `registry_box_iterator_expr()` |
| `zip(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator` for zip results; Codegen: `registry_box_iterator_expr()` |
| `map(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator(Box::new(result_elem_ty))`; Codegen: `registry_box_iterator_expr()` |

---

## 2. Correctness Validation

### 2.1 Type System Implementation (HIR)

The implementation correctly returns `Iterator[T]` types:

- **reversed**: Returns `Type::Iterator(Box::new(elem_ty))` - correct single-type iterator
- **enumerate**: Returns `Type::Iterator(Box::new(tuple_ty))` - correct tuple-type iterator with index-value pairs
- **zip**: Returns `Type::Iterator` with tuple-type elements from multiple iterables
- **map**: Returns `Type::Iterator(Box::new(result_elem_ty))` - correctly derives result type from callable return type

**Finding**: ✅ Correct - Type inference correctly computes iterator element types

### 2.2 Type Error Enforcement

The implementation correctly rejects iterator-to-list assignments without explicit materialization:

```rust
#[test]
fn test_map_rejects_plain_list_annotation_without_materialization() {
    // This should fail: assigning Iterator[int] to list[int]
    let result = lower_source(
        "def add(x: int, y: int) -> int:\n    return x + y\n\ndef main():\n    values: list[int] = map(add, [1, 2], [3, 4])\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("expected 'list[int]', got 'Iterator[int]'")));
}
```

**Verified**: ✅ PASS - Test confirms compile-time type enforcement

---

## 3. Deterministic Behavior Verification

### 3.1 Iterator Protocol Consistency

All four builtins follow standard Rust iterator protocol:

| Builtin | Iterator Implementation | Verified |
|---------|------------------------|----------|
| `reversed` | Uses `.rev()` on the iter | ✅ |
| `enumerate` | Uses `.enumerate()` + `.map()` for start offset | ✅ |
| `zip` | Uses `std::iter::zip()` | ✅ |
| `map` | Uses `.map()` transformation | ✅ |

### 3.2 Demo Validation

The demo file `demos/ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` demonstrates:

- Iterator exhaustion: `next(rev_it)` consumes first element, remaining elements correctly exhausted
- Materialization: `list(rev_it)` correctly materializes remaining elements
- Custom start: `enumerate(..., start=3)` correctly handles start parameter
- Multi-iterable: `map(add, [1, 2, 3], [4, 5, 6])` correctly iterates multiple inputs

**Verified**: ✅ `cargo run -q -p sifr -- check demos/ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` → `no errors found`

---

## 4. No-Panic Guarantees

### 4.1 Production Code Path Analysis

Reviewed code paths in:
- `crates/sifr_hir/src/lower/expressions.rs` (map/enumerate/zip/reversed lowering)
- `crates/sifr_codegen/src/lower_expr.rs` (`try_lower_simple_map_call_expr`)
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs` (registry intrinsics)

**Finding**: ✅ Correct - Production code paths use `?` for error handling. No `.unwrap()` or `.expect()` calls in user-facing code paths.

Note: `.expect()` exists in test code only, which is appropriate for test assertions.

### 4.2 Error Handling Patterns

The implementation correctly propagates errors:
- Callable argument count validation uses `?`
- Type computation errors use `?`
- Lowering failures use `?`

---

## 5. Governance Accuracy

### 5.1 Parity Ledger Updates

**File**: `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`

Updated entries:
- Line 28: `reversed(...)` → "Iterator-returning contract is closed; eager materialization is explicit (`list(...)`)"
- Line 29: `enumerate(...)` → Positional/keyword start forms closed (implicit iterator-returning)
- Line 30: `zip(...)` / `map(...)` → "Base variadic forms close on iterator-returning behavior"

**Finding**: ✅ Correct - Governance inventory accurately reflects iterator-returning behavior

### 5.2 Traceability Documentation

**File**: `verification/stdlib/wave_psp_a1_cpython_traceability.md`

Updated to reflect iterator behavior for the builtin surfaces.

**Finding**: ✅ Correct - Traceability notes updated

---

## 6. Test Coverage

### 6.1 Unit Tests

| Test | Command | Result |
|------|---------|--------|
| `test_reversed_enumerate_zip_are_typed_as_iterators` | `cargo test -p sifr_hir -- test_reversed_enumerate_zip_are_typed_as_iterators` | ✅ PASS |
| `test_map_is_typed_as_iterator` | `cargo test -p sifr_hir -- test_map_is_typed_as_iterator` | ✅ PASS |
| `test_map_rejects_plain_list_annotation_without_materialization` | `cargo test -p sifr_hir -- test_map_rejects_plain_list_annotation_without_materialization` | ✅ PASS |

### 6.2 Demo Validation

| Demo | Command | Result |
|------|---------|--------|
| `ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` | `cargo run -q -p sifr -- check demos/ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` | ✅ PASS |

---

## 7. Phase Execution Alignment

### 7.1 Entry Baseline

From `issues/ad-hoc-python-source-parity-extension-waiver-reduction-execution.md`:
- Entry baseline validated: ✅
- Baseline tests: All pass before wave implementation

### 7.2 Wave Progress

- Status: merged ✅
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1254 ✅
- Validation evidence recorded ✅

### 7.3 Review Chain

- review_pass_1 (completion-gap): ✅ Completed
- review_pass_2a (production-grade): This review

---

## 8. Findings Summary

### Strengths

1. **Correct Iterator Semantics**: All four builtins now return `Iterator[T]` matching CPython behavior
2. **Type Safety**: Compile-time errors for incorrect iterator-to-collection assignments
3. **No Panics**: All production code paths use proper error handling with `?`
4. **Deterministic**: Iterator protocol follows Rust stdlib semantics consistently
5. **Materialization Boundaries**: Explicit `list(...)`/`tuple(...)` required - no silent eager behavior
6. **Governance Accuracy**: Parity ledgers correctly reflect the new behavior

### Minor Observations (Non-blocking)

1. The `try_lower_simple_map_call_expr` function in `lower_expr.rs:570` uses a default case for handling unknown iterable types - this is appropriate defensive coding
2. Test file updates (wrapping `map()` calls with `list()`) demonstrate the expected usage pattern clearly

---

## 9. Verdict

**APPROVED** - The wave_psp_ext_1 implementation correctly:

1. ✅ Converts `reversed`, `enumerate`, `zip`, and `map` to iterator-returning semantics
2. ✅ Maintains type safety with compile-time errors for incorrect usage
3. ✅ Ensures no user-triggerable panics in the implementation
4. ✅ Provides deterministic, CPython-compatible behavior
5. ✅ Enforces explicit materialization boundaries
6. ✅ Governance ledgers accurately reflect the changes

The implementation is ready for wave closure and progression to wave_psp_ext_2.

---

## 10. Next Steps

1. Mark review_pass_2a as completed in execution ledger
2. Proceed to wave_psp_ext_2 (itertools lazy surface closure)
3. Continue phase execution as planned
