# milestone_protocols: Protocols, Operators, and Discriminated Unions

## Product Requirements

### Objective

Add the advanced OOP features that make the type system expressive: protocols (traits), operator overloading, discriminated unions, and pattern matching on classes. Builds on milestone_classes's basic class support and milestone_type_system's narrowing engine.

### Scope

#### Features In

1. `Protocol` classes compile to Rust `trait` definitions
2. Operator overloading: `__add__`, `__eq__`, `__lt__`, `__str__` map to Rust trait impls (`Add`, `PartialEq`, `PartialOrd`, `Display`)
3. Discriminated unions: classes with shared literal-typed tag field, narrowed via attribute equality
4. Pattern matching on classes: field destructuring in `match` arms
5. Nested patterns and `@` bindings in match arms
6. Property existence narrowing (`in` operator on objects)
7. Newtype pattern: `class Port(int)` compiles to Rust newtype struct
8. Struct update/spread: `User(email="new@example.com", **old_user)` clones non-overridden fields

#### Features Out

| Feature | Reason |
|---------|--------|
| Protocol-as-generic-bound (`T: Protocol`) | Deferred to milestone_generics |
| Full single inheritance | Deferred to milestone_inheritance |
| Generic classes | Deferred to milestone_generics |
| Async protocols | Deferred to milestone_async |

## Solution Design

### Architecture

All changes span four crates in the pipeline:

```
sifr_type_system  (Protocol type variant, operator trait resolution)
       ↓
sifr_hir          (new HIR nodes for protocols, operator overloads, discriminated unions)
       ↓
sifr_codegen      (Rust trait emission, impl blocks, enum generation for discriminated unions)
       ↓
sifr (tests)      (E2E pass/fail tests)
```

### Type System Changes

- Add `Type::Protocol { name, methods }` variant for protocol definitions
- Protocol assignability: a `Type::Class` satisfies a `Type::Protocol` if it has all required methods with compatible signatures (structural matching)
- Operator method resolution: `__add__` -> `Add` trait, `__eq__` -> `PartialEq`, etc.
- Discriminated union: union of classes with shared tag field generates Rust enum
- Newtype: `class Port(int)` generates `Type::Class` with a single wrapped field

### HIR Changes

- Add `HirClass.is_protocol` flag for protocol definitions
- Add `HirClass.operator_impls` for operator overloading methods
- Add `HirClass.parent_type` for newtype declarations (e.g., `int` for `class Port(int)`)
- Extend `HirStmt::Match` for class pattern destructuring (field extraction)

### Codegen Changes

- Emit `trait` definitions for protocols
- Emit `impl Trait for Struct` for protocol satisfaction
- Emit `impl Add<...>`, `impl PartialEq`, etc. for operator overloading
- Emit Rust `enum` with variants for discriminated unions (tag-based)
- Emit newtype structs (`struct Port(i64)`) for newtype pattern
- Emit struct update syntax with `.clone()` for spread operator

### Testing Strategy

- E2E pass tests: protocol_dispatch, discriminated_union, operator_overload, pattern_destructure, nested_pattern, at_binding, property_narrowing, newtype_basic, struct_update
- E2E fail tests: protocol_not_satisfied, non_exhaustive_match, newtype_validation_error
- Milestone demo in `./demos/milestone_protocols_demo.sifr`
