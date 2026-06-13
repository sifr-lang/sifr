# PRDS: milestone_auto_init — Auto-Generated Constructors

## 📄 Product Requirements & Solution Design

---

### 🧭 1. Product Requirements

#### Title
milestone_auto_init: Auto-Generated Constructors, `__eq__`, and `__str__`

---

#### Objective / Problem Statement

Every Sifr class with typed fields currently requires a boilerplate `__init__` that does nothing but assign parameters to `self`. This pattern is repeated hundreds of times in the stdlib, demos, and user code. The compiler already knows the field names and types — it should generate the constructor automatically. This milestone eliminates that boilerplate and also auto-generates `__eq__` and `__str__` for eligible classes.

---

#### Scope

##### ✅ Features In
- Auto-generated `__init__` when a class has typed fields but no explicit `__init__`
- Default field values become default parameters in the auto-generated constructor
- Auto-generated `__eq__` (field-by-field equality) for eligible classes
- Auto-generated `__str__` (format as `ClassName(field=value, ...)`) for eligible classes
- Explicit `__init__`, `__eq__`, `__str__` always take precedence
- Inheritance diagnostic when child has fields but no `__init__` and extends a parent
- Stdlib migration: remove boilerplate `__init__` from eligible classes

##### ❌ Features Out
- `super().__init__()` auto-call (user must define explicit `__init__` for this)
- `__repr__` auto-generation (out of scope)
- `__hash__` auto-generation (out of scope)
- Dataclass-style frozen/immutable semantics

---

#### Acceptance Criteria

1. `class Point: x: int; y: int` compiles and `Point(1, 2)` works without explicit `__init__`
2. `class Config: debug: bool = False` compiles and `Config()` and `Config(True)` both work
3. `p1 == p2` works for auto-init classes (field-by-field equality)
4. `str(p)` returns `"Point(x=1, y=2)"` for auto-init classes
5. Explicit `__init__` overrides auto-generation
6. Required fields after defaulted fields produce a compile error
7. All existing E2E tests still pass

---

### 🔧 2. Solution Design

#### Architecture

##### HIR Lowering (`sifr_hir/src/lower.rs`)

During class lowering, after collecting all methods:
1. Check if the class has typed field declarations
2. Check if the class has an explicit `__init__` method
3. If fields exist and no `__init__`: generate a synthetic `__init__` HIR method
4. Validate field ordering: required fields before defaulted fields
5. Similarly auto-generate `__eq__` and `__str__` if not explicitly defined

##### Synthetic `__init__` generation

```
fields: [(name, type, default?), ...]
→ HirMethod {
    name: "__init__",
    params: [self] + [(name, type, default?) for each field],
    body: [HirStmt::Assign(self.name, HirExpr::Var(name)) for each field],
    return_type: Type::None,
}
```

##### Synthetic `__eq__` generation

```
→ HirMethod {
    name: "__eq__",
    params: [self, other: Self],
    body: return self.f1 == other.f1 and self.f2 == other.f2 ...,
    return_type: Type::Bool,
}
```

##### Synthetic `__str__` generation

```
→ HirMethod {
    name: "__str__",
    params: [self],
    body: return "ClassName(f1=" + str(self.f1) + ", f2=" + str(self.f2) + ")",
    return_type: Type::Str,
}
```

#### Codegen (`sifr_codegen/src/lib.rs`)

- Auto-generated methods are indistinguishable from hand-written ones in codegen
- `__eq__` maps to `impl PartialEq` (already handled by `#[derive(PartialEq)]` if all fields are PartialEq)
- `__str__` maps to the existing `Display` impl codegen path

#### Testing Strategy

New E2E pass tests:
- `auto_init_basic`: basic class with fields, no `__init__`
- `auto_init_defaults`: class with default field values
- `auto_init_eq`: `==` operator on auto-init class
- `auto_init_str`: `str()` on auto-init class
- `auto_init_explicit_override`: explicit `__init__` takes precedence
- `auto_init_inheritance_warning`: child with fields + parent (diagnostic)

New E2E fail tests:
- `auto_init_required_after_default`: required field after defaulted field
