# Phase 20 Cleanup Review: HIR Lowering Legacy Fallback Removal

**Commit**: `2a6a7819` - "hir: remove legacy generic-class fallback and enforce strict tuple-for typing (#845)"

## Summary

This review covers the fallback-cleanup implementation that removes legacy/backward-compat behavior in HIR lowering, focusing on three areas:
1. Strict generic class type-parameter handling
2. Removal of fallback-to-Any behavior in tuple-target for-loops
3. ExternalDefs class type-parameter metadata plumbing

---

## 1. Strict Generic Class Type-Parameter Handling

### Changes

- **Removed**: Legacy fallback that scanned class fields and methods to infer type parameters when `class_declared_type_params` was empty
- **Added**: Strict enforcement requiring generic classes to declare type parameters using PEP 695 syntax (`class Foo[T]:`)
- **Added**: Error messages for:
  - Classes without declared type parameters that are used with type arguments (e.g., `LegacyBox[int]`)
  - Arity mismatch between declared type params and provided type args

### Code Review

**Location**: `crates/sifr_hir/src/lower/typing_and_functions.rs:478-537`

```rust
let class_type_params = ctx
    .class_declared_type_params
    .get(&base_name)
    .cloned()
    .unwrap_or_default();
if !type_args.is_empty() {
    if class_type_params.is_empty() {
        ctx.error(format!(
            "class '{base_name}' does not declare type parameters; use `class {base_name}[T]: ...`"
        ));
        return Type::Any;
    }
    if class_type_params.len() != type_args.len() {
        ctx.error(format!(
            "generic class '{base_name}' expects {} type argument(s), got {}",
            class_type_params.len(),
            type_args.len()
        ));
        return Type::Any;
    }
    // ... substitution logic
}
```

**Verdict**: CORRECT

- The implementation properly checks for declared type params before allowing subscript notation
- Error messages are clear and actionable
- The fallback inference logic has been completely removed

---

## 2. Tuple-Target For-Loop Fallback Removal

### Changes

- **Removed**: Legacy fallback that defaulted loop variables to `Type::Any` when the iterable didn't yield a tuple
- **Added**: Strict enforcement requiring iterables to yield tuple types with matching element count
- **Added**: Error messages for:
  - Wrong number of tuple elements (e.g., `for a, b in iterable` but iterable yields 3-tuples)
  - Iterable doesn't yield tuple type (e.g., `for a, b in [1, 2, 3]`)

### Code Review

**Location**: `crates/sifr_hir/src/lower/statements.rs:1837-1861`

```rust
if target_name.contains(',') {
    // Tuple unpacking: define each variable with its type from the tuple
    let names: Vec<&str> = target_name.split(',').collect();
    if let Type::Tuple(elem_types) = &elem_ty {
        if elem_types.len() != names.len() {
            ctx.error(format!(
                "for loop tuple target expects {} element(s), iterable yields {}",
                names.len(),
                elem_types.len()
            ));
            ctx.scope.pop();
            return None;
        }
        for (i, name) in names.iter().enumerate() {
            let ty = elem_types[i].clone();  // No longer using .get(i).unwrap_or(Type::Any)
            ctx.scope.define((*name).to_string(), ty);
        }
    } else {
        ctx.error(format!(
            "for loop tuple target expects iterable elements of tuple type, got '{}'",
            elem_ty.display_name()
        ));
        ctx.scope.pop();
        return None;
    }
}
```

**Verdict**: CORRECT

- The `.unwrap_or(Type::Any)` fallback has been removed
- Proper error handling for both element count mismatch and non-tuple iterables
- The old behavior of silently falling back to `Type::Any` is eliminated

---

## 3. ExternalDefs Class Type-Parameter Metadata Plumbing

### Changes

- **Added**: New field `class_type_params` to `ExternalDefs` struct
- **Added**: Metadata flow from class definitions to external exports
- **Added**: Metadata consumption during import resolution and annotation lowering

### Implementation Details

**ExternalDefs Structure** (`crates/sifr_hir/src/lower/mod.rs:286-288`):
```rust
pub struct ExternalDefs {
    // ...
    /// Map of `module_name` -> (`class_name` -> `type_param_names`)
    pub class_type_params:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
    // ...
}
```

**Metadata Flow Points**:

1. **Stdlib Compilation** (`crates/sifr_driver/src/lib.rs:362-364`):
   ```rust
   if !class.type_params.is_empty() {
       class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
   }
   ```

2. **Project Module Exports** (`crates/sifr_driver/src/lib.rs:773-775`):
   ```rust
   if !class.type_params.is_empty() {
       class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
   }
   ```

3. **Import Resolution** (`crates/sifr_hir/src/lower/imports.rs:48-55`):
   ```rust
   if let Some(module_class_type_params) = externals.class_type_params.get(&module_key) {
       if let Some(type_params) = module_class_type_params.get(name) {
           ctx.class_declared_type_params.insert(local.clone(), type_params.clone());
       }
   }
   ```

4. **Module Lowering** (`crates/sifr_hir/src/lower/mod.rs:638-646` and `745-749`):
   - Handles both stdlib and project module imports

**Verdict**: CORRECT

- The metadata plumbing is complete and consistent across all code paths
- Proper use of `Option` and `if let Some` for safe access
- Type params flow from class definition → export → import → context

---

## Test Coverage

### Tests Added (`crates/sifr_hir/src/lower/expressions.rs`)

1. **`test_for_tuple_target_requires_tuple_elements`** - Verifies error when iterating non-tuple with tuple target
2. **`test_generic_class_subscript_requires_declared_type_params`** - Verifies error when using subscript on non-generic class
3. **`test_generic_class_subscript_arity_mismatch_errors`** - Verifies error on wrong number of type arguments

All tests pass:
```
cargo test --package sifr_hir  # 34 passed
cargo test --package sifr_driver  # 22 passed
```

---

## Breaking Changes

This implementation is a **breaking change** for code that relied on:

1. **Legacy generic class inference**: Code like `T = TypeVar("T")` with fields using `T` now requires explicit `class Foo[T]:` declaration
2. **Tuple for-loop fallback**: Code like `for a, b in [1, 2, 3]` (iterating list with tuple target) now produces an error instead of silently typing variables as `Any`

These breaking changes are intentional as indicated by the commit message ("enforce strict tuple-for typing", "remove legacy generic-class fallback").

---

## Potential Edge Cases

### Checked and Handled

1. **Class with no type params used with subscript** → Error: "does not declare type parameters"
2. **Generic class with wrong number of type args** → Error: "expects X type argument(s), got Y"
3. **Non-tuple iterable with tuple target** → Error: "expects iterable elements of tuple type"
4. **Tuple element count mismatch** → Error: "expects X element(s), iterable yields Y"

### Appear Correct

1. **Local generic class definitions** - Properly populates `ctx.class_declared_type_params` in `collect_class_type`
2. **External generic class imports** - Properly copies from `externals.class_type_params`
3. **Stdlib generic classes** - Properly exports during `compile_stdlib_uncached`
4. **Project module generic classes** - Properly exports during `collect_module_exports`

---

## Conclusion

The implementation is **correct and complete**. All three focus areas are properly implemented:

1. ✅ Strict generic class type-parameter handling enforced
2. ✅ Tuple-target for-loop fallback-to-Any removed
3. ✅ ExternalDefs class type-parameter metadata properly plumbed

The code changes are well-structured, properly tested, and represent a clean removal of legacy compatibility behavior in favor of strict typing.
