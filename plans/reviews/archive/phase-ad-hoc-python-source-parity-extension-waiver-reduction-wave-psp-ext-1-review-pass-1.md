# wave_psp_ext_1 Review Pass 1

**Phase**: ad-hoc-python-source-parity-extension-waiver-reduction
**Wave**: wave_psp_ext_1 (Builtin Iterator Re-Closure)
**Review Type**: Completion Gap Review
**Reviewer**: agent
**Date**: 2026-03-18

---

## Executive Summary

The wave_psp_ext_1 implementation successfully converts `reversed`, `enumerate`, `zip`, and `map` builtins from eager list-returning behavior to lazy iterator-returning behavior. This aligns Sifr with CPython's iterator protocol semantics and retires the stale eager adaptation waivers.

**Verdict**: APPROVED with no blocking issues

---

## Scope Review

### Wave Definition (from phase doc)

- Port predecessor builtin-iterator architecture into the legacy parity ledgers
- Convert `reversed`, `enumerate`, `zip`, and `map` to true iterator-returning semantics where they are still eager
- Revalidate `list(...)`, `tuple(...)`, `set(...)`, and `dict(...)` as the canonical materialization boundary

### Implementation Delivered

| Target | Status | Evidence |
|--------|--------|----------|
| `reversed(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator(Box::new(elem_ty))` in `expressions.rs:1313`; Codegen: `registry_box_iterator_expr()` in `intrinsic_method_emitters.rs:1611` |
| `enumerate(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator(Box::new(tuple_ty))` in `expressions.rs:1385`; Codegen: `registry_box_iterator_expr()` in `intrinsic_method_emitters.rs:1868` |
| `zip(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator` for zip results; Codegen: `registry_box_iterator_expr()` in `intrinsic_method_emitters.rs:1621-1651` |
| `map(...)` returns `Iterator[T]` | ✅ Complete | HIR: `Type::Iterator(Box::new(result_elem_ty))` in `expressions.rs:1494`; Codegen: `registry_box_iterator_expr()` in `intrinsic_method_emitters.rs:1910-1970` and `Box::new()` in `lower_expr.rs:570-630` |

---

## Detailed Review

### 1. Builtin Iterator Re-Closure for reversed/enumerate/zip/map

#### 1.1 Type System Changes (HIR)

**File**: `crates/sifr_hir/src/lower/expressions.rs`

- **reversed** (line ~1304-1317): Changed return type from `Type::List` to `Type::Iterator`
- **enumerate** (line ~1376-1385): Changed return type from `Type::List` to `Type::Iterator`
- **zip** (line ~1395+): Changed return type from `Type::List` to `Type::Iterator`
- **map** (line ~1491-1494): Changed return type from `Type::List` to `Type::Iterator`

**Finding**: ✅ Correct - All four builtins now correctly return `Iterator[T]` types matching CPython semantics.

#### 1.2 Codegen Implementation

**File**: `crates/sifr_codegen/src/intrinsic_method_emitters.rs`

All four builtins now use the pattern:
```rust
Some(registry_box_iterator_expr(RustExpr::MethodCall { ... }))
```

Instead of the previous:
```rust
Some(RustExpr::MethodCall {
    receiver: ...,
    method: "collect::<Vec<_>>".to_string(),
    ...
})
```

**Finding**: ✅ Correct - The `.collect::<Vec<_>>()` calls have been removed and replaced with iterator boxing.

---

### 2. Map Iterator Typing/Lowering Correctness

#### 2.1 HIR Typing

**File**: `crates/sifr_hir/src/lower/expressions.rs:1471-1494`

The map implementation now correctly:
- Validates callable argument count matches iterable count
- Computes result element type from callable return type
- Returns `Type::Iterator(Box::new(result_elem_ty))`

#### 2.2 Codegen Lowering

**File**: `crates/sifr_codegen/src/lower_expr.rs:570-630`

The `try_lower_simple_map_call_expr` function correctly handles different input types:
- `Type::Iterator` / `Type::Range`: Uses `.into_iter()` directly
- `Type::Str`: Uses `.chars().map(...)` with `.to_string()` for each char
- `Type::Dict`: Uses `.keys().cloned()`
- Default (List, etc.): Uses `.clone().into_iter()`

**Finding**: ✅ Correct - Map lowering handles all major iterable types correctly.

---

### 3. Explicit Materialization Boundaries

#### 3.1 Materialization Requires Explicit Calls

The implementation correctly enforces that:
- `map(...)` returns `Iterator[T]`
- Users must write `list(map(...))`, `tuple(map(...))`, etc. to materialize

#### 3.2 Type Error for Missing Materialization

**Test evidence** (`crates/sifr_hir/src/lower/expressions_tests.rs:325-347`):

```rust
#[test]
fn test_map_rejects_plain_list_annotation_without_materialization() {
    let result = lower_source(
        "def add(x: int, y: int) -> int:\n    return x + y\n\ndef main():\n    values: list[int] = map(add, [1, 2], [3, 4])\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("expected 'list[int]', got 'Iterator[int]'")));
}
```

**Finding**: ✅ Correct - Type system correctly rejects iterator-to-list assignment without explicit materialization.

---

### 4. Deterministic Behavior

#### 4.1 Iterator Protocol Consistency

All four builtins now follow the standard Rust iterator protocol:
- `reversed`: Uses `.rev()` on the iter
- `enumerate`: Uses `.enumerate()` + `.map()` for start offset
- `zip`: Uses `std::iter::zip()` or multi-iterator chaining
- `map`: Uses `.map()` transformation

**Finding**: ✅ Correct - Deterministic behavior matching CPython semantics.

---

### 5. No-Panic Guarantees

#### 5.1 Error Handling via `?` Operator

Reviewed code paths in:
- `crates/sifr_hir/src/lower/expressions.rs` (map/enumerate/zip/reversed lowering)
- `crates/sifr_codegen/src/lower_expr.rs` (`try_lower_simple_map_call_expr`)
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs` (registry intrinsics)

**Finding**: ✅ Correct - Production code paths use `?` for error handling. No `.unwrap()` or `.expect()` calls in user-facing code paths.

Note: `.expect()` exists in test code only (line ~1501-1618 in `lower_expr.rs`) which is appropriate for test assertions.

---

## Validation Evidence

### Unit Tests

| Test | Command | Result |
|------|---------|--------|
| `test_reversed_enumerate_zip_are_typed_as_iterators` | `cargo test -p sifr_hir -- test_reversed_enumerate_zip_are_typed_as_iterators` | ✅ PASS |
| `test_map_is_typed_as_iterator` | `cargo test -p sifr_hir -- test_map_is_typed_as_iterator` | ✅ PASS |
| `test_map_rejects_plain_list_annotation_without_materialization` | `cargo test -p sifr_hir -- test_map_rejects_plain_list_annotation_without_materialization` | ✅ PASS |

### Demo Validation

| Demo | Command | Result |
|------|---------|--------|
| `ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` | `cargo run -q -p sifr -- run demos/ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` | ✅ PASS |
| `cpython_builtins_subset.sifr` | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_builtins_subset.sifr` | ✅ PASS |
| `phase_psp_a1_builtin_callable_surface.sifr` | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr` | ✅ PASS |

### Quick Validation Suite

```
Validation lane report
  profile=quick
  wall_time=195.04s cpu=26.45s
  e2e=compile=400ms plan=1ms build=1ms run=43ms cache_hits=6/6
  e2e pass suite: 24 fixtures, 24 passed, 0 failed
```

---

## Documentation Updates

The following files were updated to reflect the new iterator-returning behavior:

1. `verification/stdlib/milestone_psp_7_parity_governance_inventory.md` - Updated wording from "Eager list materialization" to "Iterator-returning contract"
2. `verification/stdlib/wave_psp_a1_cpython_traceability.md` - Updated traceability notes to reflect iterator behavior
3. `issues/ad-hoc-python-source-parity-extension-waiver-reduction-execution.md` - Created execution tracking document

---

## Findings Summary

### Strengths

1. **Correct Iterator Semantics**: All four builtins now return `Iterator[T]` matching CPython behavior
2. **Type Safety**: Compile-time errors for incorrect iterator-to-collection assignments
3. **No Panics**: All production code paths use proper error handling with `?`
4. **Deterministic**: Iterator protocol follows Rust stdlib semantics consistently
5. **Materialization Boundaries**: Explicit `list(...)`/`tuple(...)` required - no silent eager behavior

### Minor Observations (Non-blocking)

1. The `try_lower_simple_map_call_expr` function in `lower_expr.rs:570` uses a default case for handling unknown iterable types - this is appropriate defensive coding but could be documented
2. The test file updates (wrapping `map()` calls with `list()`) demonstrate the expected usage pattern clearly

---

## Verdict

**APPROVED** - The wave_psp_ext_1 implementation correctly:

1. Converts `reversed`, `enumerate`, `zip`, and `map` to iterator-returning semantics
2. Maintains type safety with compile-time errors for incorrect usage
3. Ensures no user-triggerable panics in the implementation
4. Provides deterministic, CPython-compatible behavior
5. Enforces explicit materialization boundaries

The implementation is ready for the completion review phase.

---

## Next Steps

1. Run full validation suite before merge
2. Open implementation PR
3. Complete external completion review
4. Address any findings
5. Merge and proceed to wave_psp_ext_2
