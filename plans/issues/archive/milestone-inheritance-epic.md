# milestone_inheritance: Inheritance and Class Utilities

## Product Requirements

### Objective

Add single inheritance, `super()`, class-level methods, and properties. These are important for OOP but not blocking for error handling or protocols.

### Scope

#### Features In

1. Single inheritance via trait delegation (child class inherits parent fields and methods)
2. `super()` calls parent class method in inheritance chains
3. `@classmethod` class-level methods (associated functions)
4. `@staticmethod` methods (no self/cls parameter)
5. `@property` getter/setter

#### Features Out

| Feature | Reason |
|---------|--------|
| Multiple inheritance | Not supported in Rust, not planned |
| Generic classes | Deferred to milestone_generics |
| Metaclasses | Not planned |

## Solution Design

### Architecture

```
sifr_hir          (parent_class field on HirClass, super() handling)
       ↓
sifr_codegen      (struct embedding, method delegation, associated functions)
       ↓
sifr (tests)      (E2E pass/fail tests)
```

### HIR Changes

- Add `HirClass.parent_class: Option<String>` for single inheritance
- Add `HirFunction.is_classmethod: bool` and `is_staticmethod: bool` flags
- Handle `super().__init__()` calls in lowering

### Codegen Changes

- Child struct embeds parent struct as a field
- Delegate parent methods to the embedded field
- `@classmethod` -> associated function (no self parameter)
- `@staticmethod` -> free function in impl block
- `super().method()` -> direct call to parent impl

### Testing Strategy

- E2E pass tests: inheritance_basic, super_call, classmethod_basic, staticmethod_basic, property_getter_setter
- E2E fail tests: multiple_inheritance_rejected, super_no_parent
- Milestone demo in `./demos/milestone_inheritance_demo.sifr`
