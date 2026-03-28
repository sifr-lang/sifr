## 📄 Product Requirements & Solution Design Template

---

### 🧭 1. Product Requirements

#### **Title**

M2: Control Flow and Data Structures

---

#### **Objective / Problem Statement**

The Sifr compiler (M1) can compile simple programs with functions, if/else, and primitives to native binaries. However, it cannot express loops or work with compound data types (lists, dicts, tuples). Without these, Sifr programs cannot process collections, iterate over data, or implement real algorithms. M2 adds the control flow and data structure primitives needed to write meaningful programs.

---

#### **Constraints**

| Constraint | Rationale |
| --- | --- |
| Must emit valid Rust source code | Continues M1's Rust emission strategy |
| New types must integrate with existing ownership model | Move-by-default for compound types (list, dict, tuple), copy for primitives |
| Generic type parameters for collections (list[T], dict[K,V]) | Type safety for collection elements |
| Backward compatible with M1 programs | Existing .sifr programs must continue to compile |
| Built entirely by AI agents | Comprehensive test coverage at every layer |

---

#### **Business Goals & Success Criteria (KPIs)**

| Metric | Baseline (Today) | Target (Post-launch) |
| --- | --- | --- |
| Loop constructs | 0 (no while/for) | while and for loops compile and run correctly |
| Data structure types | 0 (only primitives) | list[T], dict[K,V], tuple[T,...] all work |
| String operations | Only concatenation | len(), upper(), lower(), split(), strip(), f-strings |
| Collection operations | None | Indexing, slicing, in operator, append, len |
| Test coverage | M1 tests passing | M2 tests added at all 4 layers |

---

#### **Scope**

##### ✅ Features In

1. `while` loop with break/continue
2. `for` loop over ranges and iterables (lists)
3. `range()` built-in function
4. `list[T]` type with literal syntax, append, len, indexing
5. `dict[K, V]` type with literal syntax, key access, len
6. `tuple[T, ...]` type with literal syntax, indexing
7. String methods: `.len()`, `.upper()`, `.lower()`, `.split()`, `.strip()`
8. f-string formatting (`f"Hello {name}"`)
9. `in` operator for membership testing (lists, dicts, strings)
10. Indexing and slicing (`my_list[0]`, `my_list[1:3]`)
11. Tuple unpacking / multiple assignment (`a, b = 1, 2`)
12. `len()` built-in function for all collection types

##### ❌ Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| List/dict comprehensions | Deferred to M6 (closures needed) |
| Nested generics (list[list[int]]) | Deferred to M6 |
| Custom iterators (__iter__/__next__) | Deferred to M6 |
| Set type | Deferred to M7 (stdlib) |
| enumerate(), zip(), map(), filter() | Deferred to M6 |

---

#### **Users / Stakeholders, Use-Cases & Dependencies**

| Persona | Use-Case / Benefit | Dependencies | **AC-ID** |
| --- | --- | --- | --- |
| Sifr developer | Write programs with loops to process data | Loop lowering + codegen | AC-1 |
| Sifr developer | Use lists, dicts, tuples to model data | Collection type system + codegen | AC-2 |
| Sifr developer | Format strings with f-strings | String operation codegen | AC-3 |
| Sifr developer | Index and slice collections | Indexing/slicing lowering + codegen | AC-4 |
| AI agent | Run `cargo test` to verify M2 correctness | Test suite extended | AC-5 |

---

### **Acceptance Criteria**

| **AC-ID** | Persona | Criterion *(Given / When / Then)* |
| --- | --- | --- |
| AC-1 | Developer | **Given** a `.sifr` file with while and for loops **When** running `sifr run` **Then** the program executes loops correctly and produces expected output |
| AC-2 | Developer | **Given** a `.sifr` file using list[int], dict[str,int], tuple[int,str] **When** running `sifr build` **Then** a native binary is produced that correctly creates, modifies, and reads collections |
| AC-3 | Developer | **Given** a `.sifr` file with f-strings and string methods **When** running `sifr run` **Then** strings are formatted and manipulated correctly |
| AC-4 | Developer | **Given** a `.sifr` file with indexing (list[0]), slicing (list[1:3]), and `in` operator **When** running `sifr run` **Then** correct values are returned |
| AC-5 | AI Agent | **Given** the full codebase **When** running `cargo test` **Then** all M1 + M2 tests pass |

---

## 🧠 2. Solution Design

### **2.1 Functional Requirements**

* Parse `while` and `for` loop syntax from Python AST
* Parse `list[T]`, `dict[K,V]`, `tuple[T,...]` type annotations
* Parse list/dict/tuple literal expressions
* Parse f-string expressions
* Parse indexing (`x[0]`) and slicing (`x[1:3]`) expressions
* Parse `in` operator in boolean context
* Parse `break` and `continue` statements
* Parse tuple unpacking assignments (`a, b = 1, 2`)
* Type-check collection operations (element types, key types)
* Type-check loop variables (infer from iterable element type)
* Generate Rust code for all new constructs
* Extend ownership model: list, dict, tuple are Move types

---

### **2.2 Non-Functional Requirements**

| ID | Requirement |
| --- | --- |
| NFR-1 | Programs with loops (< 200 lines) compile in under 5 seconds |
| NFR-2 | All M1 tests continue to pass (backward compatibility) |
| NFR-3 | New tests run in under 60 seconds via `cargo test` |
| NFR-4 | No new compiler warnings on stable Rust |

---

### **2.3 High-Level Architecture**

No new crates needed. M2 extends existing crates:

```
sifr_type_system  -- Add List(T), Dict(K,V), Tuple(T...) types
        ↓
sifr_hir          -- Add While, For, Break, Continue HIR nodes
        ↓                Add collection literal, indexing, slicing, method call HIR nodes
sifr_codegen      -- Add Rust codegen for loops, collections, f-strings
        ↓
sifr_driver       -- No changes needed (pipeline unchanged)
sifr (CLI)        -- No changes needed
```

---

### **2.4 Detailed Component Design**

**📦 sifr_type_system extensions**

New type variants:
- `Type::List(Box<Type>)` — `list[T]` → `Vec<T>`
- `Type::Dict(Box<Type>, Box<Type>)` — `dict[K,V]` → `HashMap<K,V>`
- `Type::Tuple(Vec<Type>)` — `tuple[A,B,C]` → `(A,B,C)`

New type checking:
- Indexing: `list[int]` indexed by `int` returns `T`
- Dict access: `dict[K,V]` accessed by `K` returns `V`
- Tuple indexing: `tuple[A,B]` indexed by literal int returns the positional type
- `in` operator: returns `bool` for list/dict/str membership
- `len()`: returns `int` for list/dict/tuple/str
- Method calls: `.append(T)` on list, `.upper()` on str, etc.

Ownership:
- `List`, `Dict`, `Tuple` are all `Move` types
- Iterating over a list borrows it (for loop)

**⚙️ sifr_hir extensions**

New HIR statements:
- `HirStmt::While { condition, body }` — while loop
- `HirStmt::For { target, iter, body }` — for loop
- `HirStmt::Break` — break statement
- `HirStmt::Continue` — continue statement

New HIR expressions:
- `HirExpr::ListLiteral { elements, ty }` — `[1, 2, 3]`
- `HirExpr::DictLiteral { keys, values, ty }` — `{"a": 1, "b": 2}`
- `HirExpr::TupleLiteral { elements, ty }` — `(1, "hello")`
- `HirExpr::Index { object, index, ty }` — `x[0]`
- `HirExpr::Slice { object, lower, upper, ty }` — `x[1:3]`
- `HirExpr::MethodCall { object, method, args, ty }` — `x.append(1)`
- `HirExpr::FStringLiteral { parts, ty }` — `f"Hello {name}"`
- `HirExpr::RangeLiteral { start, end, ty }` — `range(n)` / `range(a, b)`
- `HirExpr::ContainsOp { left, right, ty }` — `x in collection`

New lowering:
- `lower_while` — Lower while loop with break/continue
- `lower_for` — Lower for loop, infer loop variable type from iterable
- `lower_list_literal` — Lower list display, infer element type
- `lower_dict_literal` — Lower dict display, infer key/value types
- `lower_tuple_literal` — Lower tuple display
- `lower_subscript` — Lower indexing and slicing
- `lower_attribute` — Lower method calls on types
- `lower_fstring` — Lower f-string parts
- `lower_tuple_unpack` — Lower tuple unpacking assignments

**🗄️ sifr_codegen extensions**

New Rust generation:
- `while condition { body }` — direct mapping
- `for target in iter { body }` — direct mapping with `.iter()` for borrowed iteration
- `break;` / `continue;` — direct mapping
- `vec![1, 2, 3]` — list literal
- `HashMap::from([("a", 1)])` — dict literal (with `use std::collections::HashMap;`)
- `(1, "hello".to_string())` — tuple literal
- `x[0]` / `x.get("key")` — indexing
- `x[1..3].to_vec()` — slicing
- `x.push(val)` — append
- `x.len()` — len
- `format!("Hello {}", name)` — f-string
- `0..n` / `a..b` — range
- `x.contains(&val)` / `x.contains_key(&val)` — in operator

---

### **2.5 Data Model**

Not applicable (compiler, no database).

---

### **2.6 API Integration**

Not applicable for M2.

---

### **2.7 Error Handling & Monitoring**

* Type mismatch on collection element types (e.g., `list[int]` with `str` element)
* Index out of bounds is a runtime error (Rust panics)
* Wrong key type for dict access
* Method not found on type
* Break/continue outside of loop
* Tuple unpacking length mismatch

---

### **2.8 Deployment Plan**

* Same as M1 -- distributed as `sifr` binary via `cargo install`

---

### **2.9 Trade-offs & Alternatives**

| Option Considered | Pros | Cons | Rationale for Final Choice |
| --- | --- | --- | --- |
| Implement for-loop as while-loop desugar | Simpler HIR | Loses semantic info, harder codegen | Keep for-loop as first-class HIR node |
| Use Vec<Box<dyn Any>> for untyped lists | Simpler type system | No type safety | Use Vec<T> with generic type tracking |
| Implement slicing as method call | Consistent with methods | Less Pythonic | Keep subscript syntax, map to Rust slice |

---

### **2.10 Testing Strategy** *(mapped to ACs)*

| **AC-ID** | Test Layer | Happy-Path Check | Non-Happy / Edge Check | Tooling & Automation | Pass/Fail Gate |
| --- | --- | --- | --- | --- | --- |
| AC-1 | E2E (Layer 3) | while_loop.sifr, for_loop.sifr compile and run | break/continue in nested loops, infinite loop guard | cargo test --test e2e | All pass |
| AC-2 | E2E + Snapshot | list_ops.sifr, dict_ops.sifr, tuple_ops.sifr | Empty collections, type mismatch errors | insta + e2e | All pass |
| AC-3 | E2E + Snapshot | fstring.sifr, string_methods.sifr | Empty string, special chars | insta + e2e | All pass |
| AC-4 | E2E + Snapshot | indexing.sifr, slicing.sifr, in_operator.sifr | Negative index, out of bounds | insta + e2e | All pass |
| AC-5 | All layers | cargo test passes | No regressions in M1 tests | cargo test | Exit code 0 |

**Additional NFR Tests**

* Backward compatibility: all M1 E2E tests still pass
* Performance: compilation of 200-line program with loops < 5s
