# Phase 20 Cleanup Review - Fallback Removal in HIR Lowering

**Review Date**: 2026-03-05
**Commit**: 2a6a7819 ("hir: remove legacy generic-class fallback and enforce strict tuple-for typing")
**Reviewer**: Claude Code

---

## Executive Summary

The fallback-cleanup implementation successfully removes legacy backward-compatibility behavior in HIR lowering and enforces stricter typing guarantees. The implementation is well-structured with clear error messages and comprehensive test coverage. Three minor observations are noted, but no critical regressions were identified.

---

## 1. Generic Class Type-Parameter Handling

### Implementation Review

**Location**: `crates/sifr_hir/src/lower/typing_and_functions.rs:475-538`

The implementation correctly enforces strict PEP 695 compliance:

- **Type parameter declaration required**: Classes using type parameters in annotations must declare them with `class C[T]:` syntax
- **Arity checking**: Correctly validates that the number of type arguments matches declared type parameters
- **Type substitution**: Properly substitutes type variables in both fields and methods using `substitute_type_vars()`

### Error Messages

| Scenario | Error Message |
|----------|---------------|
| Undeclared type params | `"class 'X' does not declare type parameters; use 'class X[T]: ...'"` |
| Arity mismatch | `"generic class 'X' expects N type argument(s), got M"` |

### Edge Case: Generic Class Without Type Arguments

When using a generic class without type arguments (e.g., `Box` instead of `Box[int]`), the code correctly returns the class type as-is (lines 483-539):

```rust
if !type_args.is_empty() {
    // ... substitution logic
}
return class_ty;  // Returns unsubstituted class when no type args
```

### Edge Case: Nested Generics

Nested generics like `Box[List[int]]` are correctly handled through recursive `resolve_annotation_expr` calls in the type argument extraction (lines 463-470).

**Status**: ✅ Correct

---

## 2. Tuple-Target For-Loop Fallback Removal

### Implementation Review

**Location**: `crates/sifr_hir/src/lower/statements.rs:1837-1861`

The implementation correctly removes the fallback-to-Any behavior:

- **Requires tuple type**: Only allows tuple unpacking when the iterable is `Type::Tuple`
- **Element count validation**: Correctly reports mismatch between target names and tuple elements
- **Error handling**: Properly pops the scope and returns None on error

### Error Messages

| Scenario | Error Message |
|----------|---------------|
| Non-tuple iterable | `"for loop tuple target expects iterable elements of tuple type, got 'list[int]'"` |
| Count mismatch | `"for loop tuple target expects 2 element(s), iterable yields 3"` |

### Edge Case: Empty Tuples

The code handles empty tuples correctly:
- `Type::Tuple(vec![])` matches the `if let Type::Tuple(elem_types)` pattern
- For `for a, b in ()`, the target name contains `,`, so `names.len() = 2` but `elem_types.len() = 0`
- The count mismatch error is correctly produced

**Status**: ✅ Correct

---

## 3. ExternalDefs Class Type-Parameter Metadata Plumbing

### Implementation Review

The metadata plumbing is correctly implemented across three components:

#### A. ExternalDefs Structure

**Location**: `crates/sifr_hir/src/lower/mod.rs:286-288`

```rust
pub struct ExternalDefs {
    // ... other fields
    pub class_type_params:
        HashMap<String, HashMap<String, Vec<String>>>,  // module -> class -> [type_param_names]
}
```

#### B. Driver Population

**Location**: `crates/sifr_driver/src/lib.rs:249, 363, 501-503`

The driver correctly extracts and stores type parameters when collecting exports:
- For stdlib modules (line 363): `class_type_param_exports.insert(class.name.clone(), class.type_params.clone());`
- For local modules (line 774): Same pattern
- Only inserts non-empty maps (lines 499-502, 791-794)

#### C. Import Resolution

Type parameters are correctly propagated in three import resolution locations:

1. **Early imports** (`imports.rs:48-55`): Uses `module_key` for lookup
2. **Stdlib resolution** (`mod.rs:638-646`): Uses `stdlib_module_key` for lookup
3. **Local module resolution** (`mod.rs:745-751`): Uses `module_name` for lookup

All three paths correctly insert into `ctx.class_declared_type_params`.

### Data Flow

```
AST (class.type_params)
    → collect_class_type() [classes.rs:12-34]
    → ctx.class_declared_type_params
    → driver class_type_param_exports [lib.rs:363]
    → ExternalDefs.class_type_params
    → import resolution [imports.rs:48-55, mod.rs:638-646, 745-751]
    → ctx.class_declared_type_params (in downstream module)
    → type substitution [typing_and_functions.rs:478-482]
```

**Status**: ✅ Correct and complete

---

## 4. Test Coverage

### New Tests Added

Three new tests in `crates/sifr_hir/src/lower/expressions.rs`:

| Test | Purpose | Status |
|------|---------|--------|
| `test_for_tuple_target_requires_tuple_elements` | Verifies error when using tuple unpacking on non-tuple | ✅ Pass |
| `test_generic_class_subscript_requires_declared_type_params` | Verifies error when class doesn't declare `[T]` | ✅ Pass |
| `test_generic_class_subscript_arity_mismatch_errors` | Verifies error on wrong type arg count | ✅ Pass |

### Existing Tests

All 34 existing HIR tests continue to pass:
```
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured
```

---

## 5. Observations

### Observation 1: Match Pattern Fallback (Low Priority)

In `statements.rs:812-816`, match patterns still fall back to `Type::Any` for non-tuple subjects:

```rust
let elem_types: Vec<Type> = if let Type::Tuple(ref elems) = *subject_ty {
    elems.clone()
} else {
    vec![Type::Any; seq_pat.patterns.len()]
};
```

This is inconsistent with the tuple-for-loop strictness but may be intentional for match patterns (different semantics). **Not a regression** - this behavior predates the cleanup.

### Observation 2: Clear Error Messages

The error messages are user-friendly and actionable, showing:
- What was expected vs. what was received
- Suggestion to use PEP 695 syntax for generic classes

### Observation 3: Backward Compatibility

The changes are breaking for code that:
- Used type parameters without declaring them (`LegacyBox[int]` without `class LegacyBox[T]:`)
- Used tuple unpacking on non-tuple iterables (`for a, b in [1, 2]:`)

These are intentional breaking changes to enforce stricter typing.

---

## 6. Conclusion

The fallback-cleanup implementation is **correct and well-tested**. No regressions were identified. The implementation successfully:

1. ✅ Enforces strict generic class type-parameter declaration
2. ✅ Removes fallback-to-Any in tuple-target for-loops
3. ✅ Correctly plumbs ExternalDefs class type-parameter metadata
4. ✅ Provides clear, actionable error messages
5. ✅ Maintains all existing test pass

### Recommendation

**Approve for merge/production use.** The implementation is ready.
