# Review: wave_psp_iter_fix_3 (Concrete Iterator Codegen Pipelines)

**Phase:** ad-hoc-canonical-iteration-model-and-lazy-parity-closure
**Wave:** wave_psp_iter_fix_3
**Date:** 2026-03-20
**Review:** Pass 2 (external production-grade readiness review)

## Scope

From `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure-execution.md`:

> **wave_psp_iter_fix_3: Concrete Iterator Codegen Pipelines**
>
> Scope:
> - emit concrete Rust iterator chains
> - centralize collection-to-iterator lowering
> - remove clone-based fake re-iteration of true iterators
>
> Definition of done:
> - `iter(...)` emits concrete `.into_iter()` path for both collections and iterators
> - iterator consumers (`any`, `all`, `sum`, `min`, `max`, `sorted`) use centralized iterable-to-iterator conversion
> - `filter(pred, iter(xs))` emits concrete Rust `.filter()` closure instead of unresolved builtin fallback

## Implementation Summary

The wave_psp_iter_fix_3 implementation added:

1. **Registry lowering for `iter(...)`** (`intrinsic_method_emitters.rs`)
   - Iterator inputs: pass through unchanged
   - Collection inputs: use `.clone().into_iter()`

2. **Centralized `registry_iterable_to_owned_iter_expr`** function
   - Single source of truth for iterable-to-iterator conversion
   - Handles: List, Set, Iterable, Bytes, Iterator, Range, Str, Dict, homogeneous tuples

3. **Iterator consumer rewiring**
   - `any`, `all`, `sum`, `min`, `max`, `sorted` now use centralized conversion
   - No more ad-hoc `.iter().cloned()` patterns

4. **`filter(...)` codegen**
   - Emits Rust's native `.filter()` method
   - Accepts both iterator and collection inputs

5. **`sorted(...)` generalized element type**
   - Uses `.iterable_element_type()` helper for generalized input handling

## Verification Results

### Local Validation

| Validation | Result | Notes |
|------------|--------|-------|
| Unit tests (`cargo test -p sifr -- --skip test_e2e_pass`) | ✅ PASS | 25 tests passed |
| E2E fixture (`phase_psp_iter_fix_3_concrete_iterator_codegen.sifr`) | ✅ PASS | Output matches expected |
| Demo (`ad_hoc_iter_fix_wave3_codegen_demo.sifr`) | ✅ PASS | Correct output: `true`, `[5, 3, 4]`, `[1, 3, 4, 5]` |
| HIR maintainability guardrails | ✅ PASS | `scripts/check_hir_maintainability_guardrails.py` |
| Architecture doc updated | ✅ PASS | `internal_docs/architecture.md` line 10 |
| Traceability matrix | ✅ PASS | `verification/stdlib/wave_psp_iter_fix_3_cpython_traceability.md` |

### Manual Feature Verification

| Feature | Test | Result |
|---------|------|--------|
| `any(iter(xs))` | Flags list | ✅ `true` |
| `filter(pred, iter(xs))` | Filter evens | ✅ `[5, 3, 4]` |
| `sorted(iter(xs))` | Sort iterator | ✅ `[1, 3, 4, 5]` |
| `map(fn, iter(xs))` | Double values | ✅ `[2, 4, 6]` |
| `zip(iter(xs), iter(ys))` | Zip iterators | ✅ `[(1, "a"), (2, "b"), (3, "c")]` |
| `enumerate(iter(xs))` | Enumerate | ✅ `[(0, 10), (1, 20), (2, 30)]` |
| `reversed(iter(xs))` (negative) | Non-reversible iterator | ✅ Rejected with error |

### Generated Code Verification

For input:
```sifr
def greater_than_two(x: int) -> bool:
    return x > 2

def main():
    nums: list[int] = [5, 1, 3, 4]
    print(list(filter(greater_than_two, iter(nums))))
```

Generated Rust:
```rust
println!("{:?}", (Box::new((Box::new((nums).clone().into_iter())).into_iter()
    .filter(|__filter_item| {
        let __filter_value = __filter_item.clone();
        return greater_than_two(__filter_value);
    }))).into_iter().collect::<Vec<_>>());
```

Key observations:
- `iter(nums)` emits `.clone().into_iter()` (not `.iter().cloned()`)
- `filter(...)` uses Rust's native `.filter()` with a concrete closure
- No unresolved builtin symbol fallback

## Issues Found

### 1. Regression: `filter()` with Iterator Variables

**Severity:** High

**Issue:** `filter(pred, iterator_variable)` fails to compile while `filter(pred, iter(collection))` works.

**Failing test case:**
```sifr
def main():
    nums: list[int] = [1, 2, 3, 4]
    it: Iterator[int] = iter(nums)
    result: Iterator[int] = filter(is_even, it)  # FAILS
```

**Generated broken code:**
```rust
let mut result: Box<dyn Iterator<Item = i64>> = Vec::from_iter(it.clone().into_iter().filter(is_even));
//                                                             ^^^^^^
// Error: dyn Iterator is not Clone
```

**Working case:**
```sifr
result: Iterator[int] = filter(is_even, iter(nums))  # WORKS
```

**Root cause:** The filter codegen at `intrinsic_method_emitters.rs:2070-2080` checks if `args[1]` is an `Iterator` type but this check returns `false` for iterator variable references, causing it to take the wrong branch (Vec materialization + invalid `.clone()`).

**Impact:** Users cannot pass existing iterator variables to `filter()`. This breaks the expected pattern of creating an iterator once and passing it to multiple operations.

**Verification performed:**
- `filter(is_even, iter(nums))` → Works ✅
- `filter(is_even, it)` where `it: Iterator[int] = iter(nums)` → Fails ❌

### 2. Formatting Issues in Wave3 Files

**Severity:** Low

The following files have formatting issues that fail `cargo fmt --check`:

- `crates/sifr_codegen/src/intrinsic_method_emitters.rs` (line 1699)
- `crates/sifr_codegen/src/stmt_support_emitter.rs` (lines 3316, 3340)

**Fix:** Run `cargo fmt`

### 3. Pre-existing Clippy Errors (Not from Wave3)

**Severity:** Informational

`cargo clippy --workspace -- -D warnings` fails with 6 errors in `sifr_hir`:
- `uninlined_format_args` in `function_flow.rs`
- `semicolon_if_nothing_returned` in `lower/mod.rs`

These errors existed before wave3 changes (confirmed by git history). Not a blocker for wave3.

### 4. Pre-existing E2E Test Failure (UUID)

**Severity:** Informational

E2E test `stdlib_uuid_consolidated` fails with:
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `uuid`
```

This is a pre-existing infrastructure issue (uuid crate not being linked). Not related to wave3.

## Production-Grade Readiness Assessment

### Completeness

| Definition of Done Criterion | Status |
|------------------------------|--------|
| `iter(...)` emits concrete `.into_iter()` for collections | ✅ |
| `iter(...)` passes through iterator inputs unchanged | ✅ |
| Iterator consumers use centralized conversion | ✅ |
| `filter(pred, iter(xs))` emits Rust `.filter()` closure | ✅ |
| `sorted(iter(xs))` works with iterator-typed inputs | ✅ |
| Capability guard (`reversed(iter(xs))`) still enforced | ✅ |
| No unresolved-symbol fallback in generated Rust | ✅ |
| Legacy `.iter().cloned()` patterns removed | ✅ |

### Code Quality

| Criterion | Status |
|-----------|--------|
| No clippy warnings in wave3 changes | ⚠️ Pre-existing issues in sifr_hir |
| Formatting | ❌ Needs `cargo fmt` |
| HIR maintainability guardrails | ✅ Pass |
| Single source of truth (centralized conversion) | ✅ |

### Risk Assessment

| Risk | Level | Notes |
|------|-------|-------|
| **filter() regression with iterator variables** | **High** | **Requires fix before production** |
| Backward compatibility | Low | Iterator inputs pass through unchanged (except filter) |
| Performance regression | Low | Collections must be cloned to produce owned iterators |
| Missing type coverage | Medium | `.iterable_element_type()` should be extended as new types are added |
| Formatting regression | Low | Minor fix required |

## Recommendation

**Status:** Requires regression fix before production readiness

The wave_psp_iter_fix_3 implementation has a **confirmed regression bug** that blocks production readiness:

- ❌ `filter(pred, iterator_variable)` fails to compile (tries to clone a `dyn Iterator`)
- ✅ `filter(pred, iter(collection))` works correctly

### Required Remediation

1. **Fix the filter regression**: The filter codegen needs to correctly identify iterator variable types at codegen time and emit the correct branch (boxed iterator) instead of trying to clone and materialize to Vec.

2. **Run formatting**: `cargo fmt`

### Optional (Pre-existing Issues)

- Fix pre-existing clippy errors in `sifr_hir` (not blocking)
- Fix pre-existing uuid test infrastructure (not blocking)

## Conclusion

wave_psp_iter_fix_3 addresses most definition-of-done criteria, but has a **critical regression** that blocks production readiness:

- ✅ `iter(...)` emits concrete `.into_iter()` for collections
- ✅ Iterator consumers (`any`, `all`, `sum`, `min`, `max`, `sorted`) use centralized conversion
- ✅ `filter(pred, iter(collection))` emits Rust `.filter()` closure
- ✅ `sorted(iter(xs))` works with iterator-typed inputs
- ✅ Capability guard (`reversed(iter(xs))`) still enforced
- ❌ **REGRESSION:** `filter(pred, iterator_variable)` fails - needs fix
