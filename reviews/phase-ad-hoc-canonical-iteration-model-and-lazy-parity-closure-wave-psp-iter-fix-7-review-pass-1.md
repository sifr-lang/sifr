# wave_psp_iter_fix_7 Review Pass 1 (Completion-Gap)

**Phase:** `ad-hoc-canonical-iteration-model-and-lazy-parity-closure`
**Wave:** `wave_psp_iter_fix_7` (user-defined iterable protocol participation)
**Review Type:** completion-gap
**Date:** 2026-03-20

## Scope

This wave enables user-defined classes to participate in canonical iterable semantics through protocol-shaped methods:
- `__iter__`
- `__next__`
- `__reversed__`

Implementation spans:
- Type-system iterable/iterator/reversible inference from user-class protocol methods
- HIR `next(...)` typing support for user-defined iterator classes
- Codegen iterable lowering over user classes through protocol methods
- Protocol-conformance diagnostics for malformed user-defined iteration methods

## Review Findings

### ✅ Implementation Completeness

**Type System** (`crates/sifr_type_system/src/types.rs`):
- Added `class_iter_element_type()`, `class_next_element_type()`, `class_reversed_element_type()` methods
- Extended `iterable_element_type()` to handle `Type::Class` with `__iter__` or `__next__`
- Added new `iterator_element_type()` method for protocol-based iterator inference
- Extended `iteration_metadata()` to compute capabilities from class methods
- Extended `is_assignable_to()` to handle class-to-Iterator/Iterable assignability
- Added unit tests covering iterable, iterator, and reversible class scenarios

**HIR Lowering** (`crates/sifr_hir/src/lower/classes.rs`):
- Added `validate_iteration_protocol_methods()` function that checks:
  - `__iter__` returns `Iterator[T]` or `Iterable[T]`
  - `__next__` returns `T | None`
  - `__reversed__` returns `Iterator[T]` or `Iterable[T]`
  - No extra parameters (beyond self)
  - Protocol mismatch detection between `__iter__`/`__next__` and `__iter__`/`__reversed__`

**HIR Expressions** (`crates/sifr_hir/src/lower/expressions.rs`):
- Extended `next()` builtin to accept user-defined iterator protocol classes

**Codegen** (`crates/sifr_codegen/src/intrinsic_method_emitters.rs`):
- Added `registry_iter_from_next_method_expr()` for converting `__next__`-based iterators
- Extended `registry_iterable_to_owned_iter_expr()` for class-based iteration
- Added special handling in `next()` intrinsic for user-defined iterator classes
- Added special handling in `reversed()` intrinsic for user-defined `__reversed__` method

**Codegen For-Loops** (`crates/sifr_codegen/src/stmt_support_emitter.rs`, `lower_stmt.rs`):
- Added class-aware iterator lowering in `try_lower_for_iter_source()`
- Added `class_next_iter_expr()` helper using `std::iter::from_fn`
- Extended `should_force_mutable_binding()` to handle classes with `__next__` protocol

**HIR Analysis** (`crates/sifr_codegen/src/hir_analysis/queries.rs`):
- Extended `collect_mutated_vars()` to mark `next(name)` as mutation for mutable-binding inference
- Added unit test for this behavior

### ⚠️ Issue Found: Duplicate Diagnostic Emission

**Severity:** Medium

**Location:** `crates/sifr_hir/src/lower/classes.rs` and `crates/sifr_hir/src/lower/mod.rs`

**Problem:** Protocol validation errors are emitted twice. When checking fail fixtures:

```
$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_iter_signature.sifr
type error: class 'BadIter.__iter__' must return 'Iterator[T]' or 'Iterable[T]'
type error: class 'BadIter.__iter__' must return 'Iterator[T]' or 'Iterable[T]'
```

**Root Cause:** `collect_class_type()` is called twice in `mod.rs`:
- First pass (line 596): before alias resolution
- Second pass (line 606): after alias resolution

The `validate_iteration_protocol_methods()` function is called inside `collect_class_type()`, causing validation to run twice. The validation should only happen once, likely after the second pass when all types are fully resolved.

**Recommendation:** Move `validate_iteration_protocol_methods()` call to occur only in the second pass, or add a flag to prevent duplicate validation.

### ✅ Test Coverage

**Pass Fixtures:**
- `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_7_user_defined_iterable_protocol.sifr` - Tests:
  - User-defined iterable class (`Countdown`) with `__iter__` and `__reversed__`
  - User-defined iterator class (`CountdownIter`) with `__iter__` and `__next__`
  - `for` loop iteration over user-defined iterable
  - `list()` materialization of user-defined iterable
  - `reversed()` on user-defined reversible class
  - `next()` on user-defined iterator class

**Fail Fixtures:**
- `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_iter_signature.sifr` - Invalid `__iter__` return type
- `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_next_signature.sifr` - Invalid `__next__` return type
- `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_reversed_signature.sifr` - Invalid `__reversed__` return type

**Demo:**
- `demos/ad_hoc_iter_fix_wave7_user_defined_iterable_protocol_demo.sifr` - Full end-to-end demonstration

**HIR Tests:**
- `test_user_defined_iterable_class_participates_in_builtin_iteration_surface`
- `test_next_accepts_user_defined_iterator_class`
- `test_user_defined_iterable_protocol_rejects_invalid_iter_signature`
- `test_user_defined_iterable_protocol_rejects_invalid_next_signature`

### ✅ CPython Traceability

Traceability document (`verification/stdlib/wave_psp_iter_fix_7_cpython_traceability.md`) maps to:
- `Lib/test/test_iter.py` - User-defined iterable/iterator tests
- `Lib/test/test_generators.py` - Iterator next semantics

### ✅ Demo Execution

```
$ cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave7_user_defined_iterable_protocol_demo.sifr
[4, 3, 2, 1]
[1, 2, 3, 4]
10
2
1
None
```

All expected outputs match CPython behavior.

## Summary

| Category | Status |
|----------|--------|
| User-defined iterable protocol (`__iter__`) | ✅ Implemented |
| User-defined iterator protocol (`__next__`) | ✅ Implemented |
| User-defined reversible protocol (`__reversed__`) | ✅ Implemented |
| Protocol conformance diagnostics | ✅ Implemented |
| For-loop codegen with user classes | ✅ Implemented |
| `next()` builtin support | ✅ Implemented |
| `reversed()` builtin support | ✅ Implemented |
| Test/demo/traceability completeness | ✅ Complete |
| **Diagnostic precision** | ⚠️ **Duplicate error emission** |

## Recommendation

**Approved with remediation required.** The duplicate diagnostic emission issue must be fixed before production-grade review. The fix is straightforward: ensure protocol validation only runs once (after the second `collect_class_type` pass).

## Action Items

- [ ] Fix duplicate diagnostic emission by moving validation to only run after second class collection pass
- [ ] Re-validate fail fixtures produce single diagnostic
- [ ] Run full test suite to confirm no regressions
