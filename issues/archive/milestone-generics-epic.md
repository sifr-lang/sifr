# milestone_generics — Generics, Lambdas & Iterators

## 1. Product Requirements

### Objective
Enable generic programming, closures, and higher-order functions in Sifr. This milestone adds parameterized types, lambda expressions, and iterator-based collection operations that compile to idiomatic Rust.

### Scope

**In Scope (Phase 1 — Core):**
- Generic functions: `def first[T](items: list[T]) -> T`
- Generic classes: `class Stack[T]:`
- Lambda expressions: `lambda x: x + 1` → Rust closures
- Higher-order functions: `map`, `filter` on lists
- List comprehensions: `[x * 2 for x in items]` → `.iter().map().collect()`
- Filtered comprehensions: `[x for x in items if x > 0]` → `.iter().filter().map().collect()`
- Generic built-ins: `min`, `max`, `sum`, `sorted`, `enumerate`, `zip`, `any`, `all`

**Out of Scope (deferred):**
- Type bounds (`T: Protocol`) — deferred to future enhancement
- Dict/set comprehensions — deferred
- Nested comprehensions — deferred
- Utility types (Partial, Readonly, etc.) — deferred
- Closure capture inference (borrow vs move) — use clone for now
- Iterator protocol (__iter__/__next__) — deferred to milestone_generators

### Acceptance Criteria
- AC-1: `def first[T](items: list[T]) -> T` compiles with monomorphization
- AC-2: `class Stack[T]:` with push/pop compiles correctly
- AC-3: `lambda x: x + 1` compiles to Rust closure `|x| x + 1`
- AC-4: `list(map(lambda x: x * 2, items))` works
- AC-5: `list(filter(lambda x: x > 0, items))` works
- AC-6: `[x * 2 for x in items]` compiles to iterator chain
- AC-7: `min()`, `max()`, `sum()`, `sorted()` work on lists
- AC-8: `enumerate()`, `zip()` work in for loops
- AC-9: `any()`, `all()` work with lists of booleans

## 2. Solution Design

### 2.1 Type System Changes
- Add `Type::TypeVar { name: String }` for generic type parameters
- Add `Type::Generic { name: String, type_params: Vec<Type> }` for instantiated generics
- Monomorphization: at call sites, substitute `TypeVar` with concrete types

### 2.2 HIR Changes
- Add `HirExpr::Lambda { params, body, ty }` for lambda expressions
- Add `HirExpr::ListComp { expr, var, iter, filter, ty }` for list comprehensions
- Generic functions/classes carry type parameters in their definitions

### 2.3 Lowering Changes
- Parse `[T]` type parameter syntax on functions and classes
- Infer generic type arguments at call sites
- Lower lambda expressions to `HirExpr::Lambda`
- Lower list comprehensions to `HirExpr::ListComp`
- Register built-in generic functions (min, max, sum, sorted, enumerate, zip, any, all)

### 2.4 Codegen Changes
- Emit Rust generics: `fn first<T>(items: &Vec<T>) -> T`
- Emit Rust closures: `|x| x + 1`
- Emit iterator chains for comprehensions: `.iter().map(|x| x * 2).collect::<Vec<_>>()`
- Emit built-in function implementations using Rust's iterator methods

### 2.5 Testing Strategy
**E2E pass tests:**
- `lambda_basic.sifr` — basic lambda and higher-order usage
- `list_comprehension.sifr` — list comprehension with filter
- `builtin_min_max.sifr` — min, max, sum on lists
- `builtin_sorted.sifr` — sorted() on lists
- `builtin_enumerate_zip.sifr` — enumerate and zip in for loops
- `builtin_any_all.sifr` — any() and all() on boolean lists

**Demo:** `milestone_generics_demo.sifr`
