# milestone_classes: Basic Classes

## Product Requirements

### Objective

Add basic class support to Sifr -- enough to define data types and error types. Classes compile to Rust `struct` + `impl` blocks. This milestone is a prerequisite for `milestone_error_handling` because typed error hierarchies (`class ValueError(Error)`) require classes.

### Scope

#### Features In

1. `class` definitions compile to Rust `struct` + `impl`
2. `__init__` constructor maps to `new()` associated function
3. Instance methods with `self` parameter (receiver inference: `&self` / `&mut self`)
4. Field access (`obj.field`) compiles to Rust field access
5. Auto-derived traits (`Debug`, `Clone`, `PartialEq`; conditional `Eq`/`Hash`)
6. `isinstance` narrowing for class types (extends type narrowing engine)
7. Class instances as union type members (`Circle | Square` -> Rust enum)
8. `hash(x)` built-in for hashable types
9. `str(obj)` using auto-derived `Debug` (Display deferred to milestone_protocols)

#### Features Out

| Feature | Reason |
|---------|--------|
| Inheritance (`class Child(Parent)`) | Deferred to milestone_inheritance |
| `@staticmethod` / `@classmethod` | Deferred to milestone_inheritance |
| Operator overloading (`__add__`, `__eq__`) | Deferred to milestone_protocols |
| Protocols / traits | Deferred to milestone_protocols |
| `__str__` / `__repr__` dunder methods | Deferred to milestone_protocols |
| Generic classes (`class Box[T]`) | Deferred to milestone_generics |
| `*args` / `**kwargs` in `__init__` | Deferred to milestone_decorators |

## Solution Design

### Architecture

All changes span four crates in the pipeline:

```
sifr_type_system  (new Type::Class variant, method type resolution)
       ↓
sifr_hir          (new HIR nodes for classes, lowering logic)
       ↓
sifr_codegen      (Rust struct + impl emission, constructor calls)
       ↓
sifr (tests)      (E2E pass/fail tests)
```

### Type System Changes

- Add `Type::Class { name: String, fields: Vec<(String, Type)>, methods: Vec<(String, FunctionType)> }` variant
- Method resolution: `obj.method(args)` resolves against the class's method list
- Field access type checking: `obj.field` resolves against the class's field list
- Constructor type: `ClassName(args)` resolves to `Type::Class` via the `__init__` signature
- `isinstance(obj, ClassName)` narrows union types to the specific class

### HIR Changes

- Add `HirClass` struct: `name`, `fields`, `methods`, `derive_traits`
- Add `HirModule.classes: Vec<HirClass>` alongside existing `functions`
- Add `HirExpr::FieldAccess { object, field, ty }` for `obj.field`
- Add `HirExpr::ConstructorCall { class_name, args, ty }` for `Point(1.0, 2.0)`
- Extend `HirExpr::MethodCall` to handle class method dispatch
- Add `HirStmt::FieldAssign { object, field, value }` for `self.x = value` in `__init__`

### Lowering Logic

- First pass: collect class definitions (fields from annotations, methods from `def` in body)
- Extract `__init__` parameters to build constructor signature
- Method receiver: analyze method body for `self.field = ...` (mutating -> `&mut self`) vs read-only (`&self`)
- Register class type in scope so other functions can use it as a type annotation
- Lower `ClassName(args)` calls to `HirExpr::ConstructorCall`
- Lower `obj.field` to `HirExpr::FieldAccess`
- Lower `self.field = value` inside methods to `HirStmt::FieldAssign`

### Codegen

- Emit `#[derive(Debug, Clone, PartialEq)]` on all structs (add `Eq, Hash` when all fields support it)
- Emit `struct ClassName { field: Type, ... }`
- Emit `impl ClassName { fn new(...) -> Self { Self { ... } } ... }`
- Method receiver: `&self` for read-only, `&mut self` for mutating methods
- Constructor call: `ClassName::new(args)`
- Field access: `obj.field`
- Class instances in unions: generate Rust enum with one variant per class

### Task Breakdown

**Task 1: Type System & HIR Nodes**
- Add `Type::Class` variant to type system
- Add `HirClass`, `HirExpr::FieldAccess`, `HirExpr::ConstructorCall`, `HirStmt::FieldAssign`
- Add `HirModule.classes`

**Task 2: Lowering -- Class Definitions & Constructors**
- First pass: collect class definitions (fields, methods, constructor)
- Register class types in scope
- Lower `__init__` to constructor
- Lower method definitions
- Method receiver inference (`&self` vs `&mut self`)

**Task 3: Lowering -- Field Access, Method Calls, isinstance**
- Lower `obj.field` to `HirExpr::FieldAccess`
- Lower `self.field = value` to `HirStmt::FieldAssign`
- Extend method call resolution for class methods
- Extend `isinstance` narrowing for class types
- Class instances as union members

**Task 4: Codegen -- Struct, Impl, Derives**
- Emit `struct` with `#[derive(...)]`
- Emit `impl` block with `new()` and methods
- Emit constructor calls as `ClassName::new(args)`
- Emit field access
- Emit class-based union enums
- `hash(x)` built-in codegen

**Task 5: E2E Tests & Demo**
- 6 pass tests: class_basic, class_methods, class_field_access, class_isinstance, class_union, hash_builtin
- 3 fail tests: missing_field, use_after_move_self, unhashable_dict_key
- Regression tests for all prior milestones
- Milestone demo

### Testing Strategy

| Test | Layer | Check |
|------|-------|-------|
| class_basic | E2E pass | Class with fields, __init__, print |
| class_methods | E2E pass | Methods with &self, &mut self |
| class_field_access | E2E pass | obj.field read and write |
| class_isinstance | E2E pass | isinstance narrowing for class types |
| class_union | E2E pass | Class instances as union members |
| hash_builtin | E2E pass | hash(x) for hashable types |
| missing_field | E2E fail | Access undefined field -> error |
| use_after_move_self | E2E fail | Use self after move -> error |
| unhashable_dict_key | E2E fail | Non-hashable type as dict key -> error |
