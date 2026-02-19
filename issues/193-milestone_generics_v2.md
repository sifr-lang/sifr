# PRDS: milestone_generics_v2 — User-Facing Generics Completion

## 📄 Product Requirements & Solution Design

---

### 🧭 1. Product Requirements

#### Title
milestone_generics_v2: Complete Generic Classes, Type Parameter Substitution, and None as Standalone Type

---

#### Objective / Problem Statement

Generic functions work in Sifr, but generic classes have incomplete type parameter substitution. When `Stack[int]` is instantiated, the `T` in field types and method signatures is not substituted with `int`. This means generic classes are unusable in practice. Additionally, `None` as a standalone type and value has edge cases. This milestone completes the generics story.

---

#### Scope

##### ✅ Features In
- Generic class field and method type parameter substitution at instantiation sites
- Type parameter inference at class instantiation from constructor arguments
- Generic class auto-init interaction (type parameters in generated constructor)
- `None` as a standalone value and type in all positions
- Protocol bounds on type parameters (`def f[T: Comparable](x: T)`)

##### ❌ Features Out
- Multiple type parameter bounds with `&` syntax (complex, deferred)
- Generic type aliases (`type Pair[T] = tuple[T, T]`)
- Full Rust generic monomorphization (Rust handles this automatically)

---

#### Acceptance Criteria

1. `class Stack[T]: items: list[T]` — `Stack[int]([1,2,3])` works, `stack.items` has type `list[int]`
2. `Stack([1,2,3])` infers `T = int` from the argument
3. `x: None = None` is valid
4. `def f() -> None` is valid and equivalent to returning nothing
5. All existing E2E tests still pass

---

### 🔧 2. Solution Design

#### Generic Class Substitution (sifr_hir + sifr_type_system)

When a generic class is instantiated with concrete type arguments:
1. Create substitution map `{T: int}` from the type arguments
2. Apply substitution to all field types and method signatures
3. Field access resolves through the substitution
4. Method calls substitute type parameters before type-checking

#### None as Standalone Type

- `x: None = None` — `Type::None` in type system, `()` in Rust
- `def f() -> None` — already works, verify no regressions
- `None` in dict keys, set members — `()` implements Hash, Eq, Clone

#### Testing Strategy

New E2E pass tests:
- `generic_class_basic`: `class Stack[T]` with field access
- `generic_class_field_access`: field type substitution
- `generic_class_method`: method type substitution
- `generic_class_inference`: type inference from constructor args
- `none_standalone_value`: `x: None = None`
