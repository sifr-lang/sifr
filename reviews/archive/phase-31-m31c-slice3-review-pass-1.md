# Phase 31 m31_c Slice 3 Review

**Slice**: `m31_c_stdlib_module_parity` (defaultdict compatibility and len(deque))
**Review Date**: 2026-03-12
**Status**: PASS

---

## Summary

Slice 3 adds support for `collections.defaultdict(...)` with `list`, `set`, and `int` factories, plus `len(deque)` compatibility. The implementation correctly removes stdlib blockers without introducing unsafe fallback behavior.

---

## Implementation Overview

### 1. defaultdict Support

**Files Modified**:
- `crates/sifr_hir/src/lower/builtin_calls.rs` - HIR lowering for defaultdict constructors
- `crates/sifr_hir/src/lower/compat_imports.rs` - Import resolution for collections.defaultdict
- `crates/sifr_hir/src/lower/expressions.rs` - Expression lowering and refinement
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs` - Codegen for defaultdict operations
- `crates/sifr_codegen/src/lower_expr.rs` - Index expression lowering
- `crates/sifr_codegen/src/lower_stmt.rs` - Statement-level codegen
- `crates/sifr_codegen/src/intrinsics/collections.rs` - Runtime intrinsics

**Key Implementation Details**:

1. **Constructor Support**: Both `collections.defaultdict(list)` and bare `defaultdict(list)` forms work

2. **Factory Support**:
   - `defaultdict(int)` → returns 0 on missing key access
   - `defaultdict(list)` → returns empty Vec on missing key access
   - `defaultdict(set)` → returns empty HashSet on missing key access

3. **Type Inference**: Types are inferred from first usage (e.g., `groups["hit"].append(...)` refines the value type to `str`)

4. **Codegen Pattern**: Uses Rust's `HashMap::entry(...).or_insert(...)` for correct semantics:
   ```rust
   // For defaultdict[key] access
   map.entry(key).or_insert(default_value)
   ```

5. **Mutability**: Correctly emits mutable bindings since entry API requires mutability

### 2. len(deque) Support

**Files Modified**:
- `crates/sifr_hir/src/lower/builtin_calls.rs` - Extended len() to handle class types

**Implementation**: Extended `lower_len_call` to accept `Type::Class` instances that have a `len` method, covering `deque`.

---

## Correctness Analysis

### Positive Findings

1. **Demo runs successfully**:
   ```bash
   cargo run -q -p sifr -- run demos/phase31_defaultdict_compat_demo.sifr
   # Exit code: 0
   ```

2. **Unit test passes**:
   ```
   test test_defaultdict_list_call_resolves_without_import ... ok
   ```

3. **E2E test passes**:
   ```
   cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase31_defaultdict_len_deque_compat.sifr
   # Exit code: 0
   ```

4. **Type soundness**: Uses `Type::Alias` for tracking defaultdict types with proper inner type resolution

5. **Codegen soundness**: Proper method resolution, no unwrap/expect in user paths

### Regression Testing

- **Quick validation**: 395 pass tests, 0 failures
- **Verification hardening**: 64 variants, 0 failures, 0 blocking failures

---

## Safety Analysis

### No Unsafe Fallback Behavior

1. **defaultdict indexing**: Uses Rust's `entry(...).or_insert(...)` API which correctly handles missing keys without panics

2. **Type checking**: HIR-level validation ensures only supported factories (int, list, set) are accepted

3. **Error handling**: Proper error messages for unsupported factories or invalid arguments

4. **No unwrap/expect**: No data-dependent unwraps in generated runtime code

---

## Targeted Blockers Removed

From the execution report (`issues/phase31-m31c-defaultdict-len-compat-execution.md`):

| Case | Before | After |
|------|--------|-------|
| 0036 valid_sudoku | CHECK_ERROR (defaultdict) | **PASS** (checks and runs) |
| 0127 word_ladder | CHECK_ERROR (defaultdict/deque) | CHECK_ERROR (deeper issues: optional slicing) |
| 0149 max_points_on_a_line | CHECK_ERROR (defaultdict) | CHECK_ERROR (deeper issues: arithmetic/optional) |

The stdlib surface blockers are removed - cases now fail on deeper type system issues, not stdlib gaps.

---

## Test Coverage

- **Regression test**: `crates/sifr/tests/e2e/pass/phase31_defaultdict_len_deque_compat.sifr`
- **Unit test**: `test_defaultdict_list_call_resolves_without_import` in `crates/sifr_hir/src/lower/expressions.rs`
- **Demo**: `demos/phase31_defaultdict_compat_demo.sifr`

---

## Conclusion

**APPROVED** - The slice correctly implements defaultdict compatibility and len(deque) support:

1. **Correctness**: Demo, unit tests, and e2e tests all pass
2. **No regressions**: Quick validation (395 tests) and hardening suites pass
3. **Type soundness**: Proper type inference and alias handling
4. **Codegen soundness**: Uses correct Rust patterns (entry API)
5. **No unsafe fallbacks**: Proper error handling, no unwraps in user paths
6. **Blockers removed**: Cases now progress past stdlib gaps to deeper type issues

The implementation successfully removes the targeted stdlib blockers without compromising Sifr's safety guarantees.
