## M3: Advanced Type System

---

### 1. Product Requirements

#### **Title**

M3: Advanced Type System -- Union Types, Literal Types, and Type Narrowing

---

#### **Objective / Problem Statement**

The sifr compiler currently supports only primitive types (int, float, bool, str, None), collections (list, dict, tuple), and functions. Real-world programs need to express "a value can be one of several types" (union types), use specific values as types (literal types), and have the compiler automatically narrow types based on control flow (type narrowing).

Without these features, sifr cannot implement clean error handling (M4's Result/Option), discriminated unions (M5), or generic type bounds (M7). Every subsequent milestone depends on M3's type system foundations.

---

#### **Constraints**

| Constraint | Rationale |
| --- | --- |
| Python-first syntax | Union types use `int \| str` (PEP 604), not custom syntax |
| TypeScript-style literal types | Values as types: `"GET" \| "POST"` instead of `Literal["GET"]` |
| No redundant sugar | One way to express optionals: `str \| None`, no `T?` shorthand |
| Intersection types are internal only | Used by narrowing engine, not exposed as user syntax |

---

#### **Business Goals & Success Criteria (KPIs)**

| Metric | Baseline (Today) | Target (Post-launch) |
| --- | --- | --- |
| Type expressiveness | Primitives + collections only | Union, literal, optional, unknown, type aliases |
| Type safety | No narrowing; manual type assertions | Full control-flow-based narrowing |
| E2E test coverage | 23 pass / 5 fail tests | +10 pass / +5 fail tests for M3 features |

---

#### **Scope**

##### Features In

1. Union types (`int | str`, `A | B | C`) with flattening and deduplication
2. Literal types (`"GET" | "POST"`, `200 | 404`, `True | False`)
3. Type aliases (`type HttpMethod = "GET" | "POST"`)
4. Optional types (`str | None`)
5. Unknown type (safe top type, must be narrowed before use)
6. Type narrowing via control flow analysis:
   - Truthiness checks (`if x:`)
   - `isinstance()` checks
   - Equality checks (`if x == "GET":`)
   - `is None` / `is not None` checks
   - `not` negation (else branches get complement type)
7. Type predicates (`TypeGuard[T]`)
8. `reveal_type()` built-in
9. `never` exhaustiveness checking
10. Control flow graph for narrowing

##### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| Discriminated unions (struct tag narrowing) | Deferred to M5 when classes exist |
| User-facing intersection types (`A & B`) | Internal only until protocols in M5 |
| Generics with union bounds | Deferred to M7 |
| Contextual typing for lambdas | Deferred to M7 |

---

#### **Users / Stakeholders, Use-Cases & Dependencies**

| Persona | Use-Case / Benefit | Dependencies | AC-ID |
| --- | --- | --- | --- |
| sifr developer | Express "value is one of several types" | M2 complete | AC-1 |
| sifr developer | Use specific values as types for APIs | M2 complete | AC-2 |
| sifr developer | Compiler narrows types in if/else branches | M2 complete | AC-3 |
| sifr developer | Handle optional values safely | M2 complete | AC-4 |
| sifr developer | Use Unknown for safe dynamic data | M2 complete | AC-5 |
| Future milestones | M4 Result/Option, M5 discriminated unions | M3 complete | AC-6 |

---

### **Acceptance Criteria**

| AC-ID | Persona | Criterion |
| --- | --- | --- |
| AC-1 | Developer | **Given** a function parameter typed `int \| str`, **When** the program passes an int or str value, **Then** it compiles and runs correctly, generating a Rust enum |
| AC-2 | Developer | **Given** a type alias `type Shape = "circle" \| "square"`, **When** a variable is typed as `Shape`, **Then** only literal values "circle" or "square" are accepted |
| AC-3 | Developer | **Given** a variable `x: int \| str` and `if isinstance(x, int):`, **When** inside the if-branch, **Then** `x` is narrowed to `int` and int operations work; in else-branch `x` is `str` |
| AC-4 | Developer | **Given** a variable `x: str \| None` and `if x is not None:`, **When** inside the if-branch, **Then** `x` is narrowed to `str` |
| AC-5 | Developer | **Given** a parameter typed `Unknown`, **When** used without narrowing, **Then** the compiler reports an error; after `isinstance` narrowing, operations work |
| AC-6 | Future | **Given** M3 is complete, **When** M4 implements `Result[T, E]`, **Then** it can be built as `T \| E` union type |

---

## 2. Solution Design

### **2.1 Functional Requirements**

* Extend the `Type` enum with `Union`, `Intersection`, `LiteralInt`, `LiteralStr`, `LiteralBool`, `Optional`, `Alias`, and `Unknown` variants
* Build union normalization (flatten, deduplicate, sort) and literal widening
* Build a control flow graph (CFG) during HIR lowering
* Build a narrowing engine that narrows types based on conditions
* Update HIR lowering to use CFG and narrowing in if/else branches
* Update codegen to emit Rust enums for union types and match expressions for narrowing
* Add `reveal_type()` built-in and `never` exhaustiveness checking

---

### **2.2 Non-Functional Requirements**

| ID | Requirement |
| --- | --- |
| NFR-1 | Compilation time: adding union types should not increase compile time by more than 2x for existing programs |
| NFR-2 | Error messages: narrowing errors must show the current narrowed type and what operations are available |
| NFR-3 | Generated Rust code must compile without warnings |

---

### **2.3 High-Level Architecture**

```
Source (.sifr)
    |
    v
Parser (existing)
    |
    v
AST (existing)
    |
    v
HIR Lowering (updated: builds CFG, applies narrowing)
    |
    v
Type Checker (updated: union subtyping, literal checking)
    |
    v
HIR (updated: narrowed types on nodes)
    |
    v
Codegen (updated: union -> Rust enum, narrowing -> match)
    |
    v
Rust Source -> rustc -> Native Binary
```

---

### **2.4 Detailed Component Design**

**Type System (`sifr_type_system`)**

New files:
- `union.rs`: Union construction, normalization (flatten, dedup, sort), subtyping
- `literal.rs`: Literal type handling, widening to base type
- `narrow.rs`: Narrowing engine with `NarrowingCondition` enum

Updated files:
- `types.rs`: Extended `Type` enum with new variants
- `check.rs`: Updated type checking for union operands
- `infer.rs`: Updated inference for literal preservation

**HIR (`sifr_hir`)**

New files:
- `cfg.rs`: Control flow graph with `FlowNode` types

Updated files:
- `hir_nodes.rs`: New HIR nodes for type alias, isinstance, is-None checks
- `lower.rs`: Two-pass lowering updated to build CFG and apply narrowing
- `scope.rs`: Track `narrowed_type` per variable

**Codegen (`sifr_codegen`)**

Updated files:
- `lib.rs`: Union type -> Rust enum generation, narrowing -> match expression generation

**Driver (`sifr_driver`)**

Updated files:
- `lib.rs`: Pipeline unchanged but passes through new type information

---

### **2.5 Data Model**

Type enum extensions:
```rust
enum Type {
    // ... existing ...
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    LiteralInt(i64),
    LiteralStr(String),
    LiteralBool(bool),
    Optional(Box<Type>),
    Alias(String, Box<Type>),
    Unknown,
}
```

Control flow graph:
```rust
enum FlowNode {
    Start,
    Assignment { var: String, ty: Type, antecedent: FlowNodeId },
    Condition { expr: HirExprId, true_branch: FlowNodeId, false_branch: FlowNodeId },
    Label { antecedents: Vec<FlowNodeId> },
    Unreachable,
}
```

---

### **2.6 API Integration**

N/A -- this is a compiler-internal change.

---

### **2.7 Error Handling & Monitoring**

New error types:
- "Cannot use operator X on union type `int | str` without narrowing"
- "Unknown type must be narrowed via isinstance before use"
- "Non-exhaustive match: type `never` is not reachable (missing variant X)"
- "Type `X` is not assignable to `Y | Z`"

---

### **2.8 Deployment Plan**

N/A -- compiler feature, released with next version.

---

### **2.9 Trade-offs & Alternatives**

| Option Considered | Pros | Cons | Rationale for Final Choice |
| --- | --- | --- | --- |
| Tagged enum approach for unions | Simple, maps directly to Rust enums | Requires enum name generation | Chosen: clean Rust codegen |
| Box<dyn Any> for unions | Flexible | No compile-time safety, runtime overhead | Rejected: defeats purpose of static typing |
| Full intersection types for users | More expressive | Complex, rarely needed without protocols | Deferred: internal only until M5 |

---

### **2.10 Testing Strategy**

| AC-ID | Test Layer | Happy-Path Check | Non-Happy / Edge Check | Tooling | Pass/Fail Gate |
| --- | --- | --- | --- | --- | --- |
| AC-1 | E2E pass | `union_basic.sifr`: int\|str param works | `union_mismatch.sifr`: wrong type rejected | cargo test | Must pass |
| AC-2 | E2E pass | `literal_types.sifr`: literal type alias works | `literal_mismatch.sifr`: wrong literal rejected | cargo test | Must pass |
| AC-3 | E2E pass | `isinstance_narrowing.sifr`: narrowing works | `no_narrowing_access.sifr`: error without narrowing | cargo test | Must pass |
| AC-4 | E2E pass | `optional_narrowing.sifr`: None check works | `optional_no_check.sifr`: error without check | cargo test | Must pass |
| AC-5 | E2E pass | `unknown_type.sifr`: narrowed Unknown works | `unknown_no_narrow.sifr`: error without narrowing | cargo test | Must pass |
| AC-6 | Unit | Union normalization, literal widening | Edge: empty union, single-element union | cargo test | Must pass |
