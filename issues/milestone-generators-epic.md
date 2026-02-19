# milestone_generators — Generators and Context Managers

## 1. Product Requirements

### Objective
Add generator functions (`yield`), generator expressions, and context managers (`with`) to Sifr. Generators compile to eager collection (Vec), and `with` compiles to scoped blocks.

### Scope

**In Scope:**
- Generator functions with `yield` -> compile to functions returning `Vec<T>`
- Generator expressions: `(x * 2 for x in items)` -> lazy iterator
- `with` statement for scoped resource management
- `next()` built-in for iterators (on lists)

**Out of Scope (deferred):**
- True state-machine generators (lazy yield)
- `yield from` delegation
- Async generators
- ContextManager protocol enforcement
- File I/O (no stdlib yet)

### Acceptance Criteria
- AC-1: `yield` in a function body collects values into a Vec
- AC-2: Generator expressions compile to lazy iterators
- AC-3: `with` statement compiles to scoped blocks
- AC-4: For loops over generator expressions work

## 2. Solution Design

### 2.1 HIR Changes
- Add `HirStmt::Yield { value }` for yield statements
- Add `HirExpr::GeneratorExpr { expr, var, iter, filter, ty }` for generator expressions

### 2.2 Lowering Changes
- Detect `yield` in function bodies and mark function as generator
- Lower generator expressions similarly to list comprehensions but lazy
- Lower `with` statements to scoped blocks

### 2.3 Codegen Changes
- Generator functions: collect yields into a Vec and return it
- Generator expressions: emit as iterator chain without `.collect()`
- `with` statements: emit as scoped blocks with variable binding

### 2.4 Testing Strategy
**E2E pass tests:**
- `generator_basic.sifr` — simple yield function
- `generator_expr.sifr` — generator expression in for loop
- `with_basic.sifr` — with statement for scoped resources

**Demo:** `milestone_generators_demo.sifr`
