---
name: Sifr Compiler Architecture
overview: Build "sifr", a compiled programming language with Python syntax and enforced typing that emits Rust source code, compiled via rustc into native binaries. The compiler is built in Rust, forking ruff's parser/AST crates and adding type checking, IR, and Rust codegen phases. TypeScript-inspired type system features (union/intersection types, literal types, full control-flow-based type narrowing) are first-class citizens. The end goal is a language capable of building web applications and general-purpose programs.
todos:
  - id: fork-parser
    content: "M1: Fork ruff parser/AST crates (python_ast, python_parser) into crates/ with sifr_ prefix. Use git deps for infrastructure crates (text_size, source_file, python_trivia, python_literal). Set up Cargo workspace."
    status: completed
  - id: strip-ast
    content: "M1: Strip the forked AST to only the nodes needed for M1 (function def, if/elif/else, assign, ann_assign, return, expr, basic expressions, literals). Remove IPython, match, async, with, try, import, etc."
    status: completed
  - id: type-system
    content: "M1: Build sifr_type_system crate -- Type enum (Int, Float, Bool, Str, None, Function, Any, Never), type inference from initializers, type checking (binary ops, comparisons, function calls), subtyping rules."
    status: completed
  - id: hir
    content: "M1: Build sifr_hir crate -- Typed IR with resolved names and types on every node. Name resolution (scopes). Ownership tracking (move vs copy)."
    status: completed
  - id: codegen
    content: "M1: Build sifr_codegen crate -- Walk HIR and emit Rust source code. Type mapping (int->i64, str->String, etc.). Generate Cargo.toml + main.rs. Handle print() as println! macro."
    status: completed
  - id: driver
    content: "M1: Build sifr_driver crate -- Orchestrate parse -> type-check -> HIR -> codegen pipeline. Error reporting with source spans and nice diagnostics (use miette or ariadne)."
    status: completed
  - id: cli
    content: "M1: Build sifr CLI binary -- sifr build/run/check/emit commands using clap. Invoke cargo build on generated Rust project."
    status: completed
  - id: test-e2e
    content: "M1: End-to-end test -- Write sample .sifr programs (hello world, factorial, fibonacci, basic arithmetic) and verify they compile and run correctly."
    status: completed
  - id: m2-loops
    content: "M2: While/for loops, break/continue, range() support."
    status: completed
  - id: m2-collections
    content: "M2: List, dict, tuple types with collection operations."
    status: completed
  - id: m2-strings
    content: "M2: String operations, f-strings, and tuple unpacking."
    status: completed
  - id: m3-type-enum
    content: "M3: Extend Type enum with Union, Intersection, LiteralInt, LiteralStr, LiteralBool, Optional, Alias variants. Add union normalization, literal widening, and subtyping rules."
    status: pending
  - id: m3-narrowing-engine
    content: "M3: Build the narrowing engine (narrow.rs) with NarrowingCondition enum and narrow_type function. Support truthiness, isinstance, equality, is None, type predicates, and negation."
    status: pending
  - id: m3-cfg
    content: "M3: Build control flow graph (cfg.rs) during HIR lowering. FlowNode types for assignments, conditions, labels, unreachable. Wire into scope for narrowed type tracking."
    status: pending
  - id: m3-hir-narrowing
    content: "M3: Update HIR lowering to use CFG and narrowing. If/else branches narrow types, isinstance calls trigger narrowing, equality checks narrow literals."
    status: pending
  - id: m3-codegen-unions
    content: "M3: Update codegen to emit Rust enums for union types, match expressions for narrowing, and handle literal type -> value mapping."
    status: pending
  - id: m3-tests
    content: "M3: Add comprehensive tests -- unit tests for union/literal/narrowing, E2E pass tests (union_basic, optional_narrowing, isinstance_narrowing, etc.), E2E fail tests (non-exhaustive, no-narrowing access)."
    status: pending
  - id: m3-demo
    content: "M3: Create milestone demo in ./tmp/m3_demo.sifr showcasing union types, literal types, type narrowing, and optional handling."
    status: pending
isProject: false
---

# Sifr Compiler -- Architecture and Implementation Plan

## Vision

Sifr is a compiled programming language that uses Python syntax with enforced static typing. It compiles Python-like source code to Rust source code, which is then compiled by `rustc` into native binaries. Ownership semantics follow Rust's move-by-default model. Types are strict with an opt-in `Any` escape hatch (like TypeScript's strict mode).

The type system draws heavily from TypeScript's design: union and intersection types, literal types, and full control-flow-based type narrowing are first-class citizens. Unlike TypeScript (which erases types at runtime), sifr uses types to generate efficient Rust code -- union types become Rust enums, narrowing becomes `match` expressions, and literal types enable compile-time value checking.

The end goal is a language capable of building web applications and general-purpose programs -- anywhere Python is used today, but with native performance and compile-time safety.

## Compiler Pipeline

```mermaid
flowchart LR
    Source["Source (.sifr)"] --> Lexer
    Lexer --> Parser
    Parser --> AST["Sifr AST"]
    AST --> Binder["Binder / Name Resolution"]
    Binder --> Checker["Type Checker"]
    Checker --> HIR["Sifr HIR"]
    HIR --> RustCodegen["Rust Codegen"]
    RustCodegen --> RustSource[".rs files"]
    RustSource --> Rustc["rustc"]
    Rustc --> Binary["Native Binary"]
```



## Milestone Roadmap

```mermaid
flowchart TD
    M1["M1: Core Language (DONE)\nVariables, functions, if/else,\nprimitives, print, CLI"] --> M2
    M2["M2: Control Flow + Data (DONE)\nLoops, list, dict, tuple,\nstring ops, indexing"] --> M3
    M3["M3: Advanced Type System\nUnion/intersection types,\nliteral types, type narrowing,\ncontrol flow analysis"] --> M4
    M4["M4: Error Handling\nResult/Option via unions,\ntry/except as match,\n? operator, assert"] --> M5
    M5["M5: Structs and Methods\nclass -> struct+impl,\nprotocols/traits,\ndiscriminated unions"] --> M6
    M6["M6: Module System\nimport/from -> mod/use,\nmulti-file, sifr.toml"] --> M7
    M7["M7: Generics\nType params, bounds,\nclosures/lambdas,\niterators, HOFs"] --> M8
    M8["M8: Standard Library\nFile I/O, JSON, env,\nmath, collections,\ntime, regex"] --> M9
    M9["M9: Async + Networking\nasync/await -> tokio,\nHTTP, web framework"] --> M10
    M10["M10: Metaprogramming\nDecorators, dataclass,\ncompile-time eval"] --> M11
    M11["M11: Production Readiness\nLSP, formatter, linter,\npackage registry, FFI"]
```



**Rationale for milestone order:** Union types, literal types, and type narrowing are placed in M3 (before error handling) because they are foundational -- M4's `Result[T, E]` and `Option[T]` are union-based, M5's discriminated unions need narrowing, and M7's generics need type bounds with unions. Every milestone after M3 benefits from the advanced type system.

---

## Crate Structure (Rust Workspace)

**Hybrid dependency approach:** Infrastructure crates are referenced as git dependencies from ruff v0.4.10 (unmodified). Parser and AST crates are vendored forks that may diverge from Python syntax in future milestones.

```
sifr/
  Cargo.toml                (workspace root)
  crates/
    sifr_python_ast/        (vendored fork of ruff_python_ast -- may diverge for sifr syntax)
    sifr_python_parser/     (vendored fork of ruff_python_parser -- may diverge for sifr syntax)
    sifr_hir/               (High-level IR: typed AST after name resolution + type checking)
    sifr_type_system/       (type definitions, inference, checking, subtyping)
    sifr_codegen/           (Rust source code generation from HIR)
    sifr_driver/            (orchestrates the pipeline, error reporting)
    sifr/                   (CLI binary: sifr build, sifr check, sifr run)

  # Git dependencies from ruff v0.4.10 (not vendored):
  #   ruff_text_size          -- text span/range utilities
  #   ruff_source_file        -- source file representation, line indexing
  #   ruff_python_trivia      -- whitespace/comment handling
  #   ruff_python_literal     -- literal parsing (string escapes, number formats)
```

New crates added per milestone as needed (e.g. `sifr_std`, `sifr_lsp`, `sifr_fmt`).

---

## M1: Core Language (First Working Compiler)

**Goal:** Compile a simple program with variables, functions, basic types, and branching to a native binary.

### Language Features

- **Types:** `int`, `float`, `bool`, `str`, `None`
- **Literals:** integer, float, string, boolean, None
- **Variables:** typed declarations (`x: int = 5`), inferred declarations (`x = 5`)
- **Functions:** typed parameters and return types, recursion
- **Expressions:** arithmetic (`+`, `-`, `*`, `/`, `//`, `%`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`), boolean (`and`, `or`, `not`), string concatenation
- **Statements:** assignment, return, expression statements, `if`/`elif`/`else`
- **Built-in:** `print()` function
- **Entry point:** `main()` function as program entry
- **Move semantics:** move on assignment for `str`, copy for primitives (`int`, `float`, `bool`)
- **CLI:** `sifr build`, `sifr run`, `sifr check`, `sifr emit`

### Implementation Steps

1. Fork ruff parser/AST crates into `crates/` with `sifr_` prefix; use git deps for infrastructure crates
2. Strip the AST to M1-relevant nodes only
3. Build `sifr_type_system` -- Type enum, inference from initializers, checking binary ops / function calls
4. Build `sifr_hir` -- Typed IR with name resolution and ownership tracking
5. Build `sifr_codegen` -- Emit Rust source code, generate Cargo.toml + main.rs
6. Build `sifr_driver` -- Orchestrate the pipeline with nice error diagnostics
7. Build `sifr` CLI binary with clap
8. End-to-end tests (hello world, factorial, fibonacci)

### Example Program

```python
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    x: int = factorial(5)
    print(x)
```

### Type Mapping (M1)

- `int` -> `i64`
- `float` -> `f64`
- `bool` -> `bool`
- `str` -> `String`
- `None` -> `()`

---

## M2: Control Flow and Data Structures

**Goal:** Support loops and compound data types so programs can process collections of data.

### Language Features

- **Loops:** `while` loop, `for` loop over ranges and iterables
- **Data types:** `list[T]`, `dict[K, V]`, `tuple[T, ...]`
- **Indexing:** `my_list[0]`, `my_dict["key"]`
- **Slicing:** `my_list[1:3]`
- **String operations:** `.len()`, `.upper()`, `.lower()`, `.split()`, `.strip()`, f-strings
- **Type inference:** infer collection element types from usage
- `**in` operator:** membership testing
- `**range()` built-in**
- **Multiple assignment:** `a, b = 1, 2` (tuple unpacking)

### Example Program

```python
def sum_list(numbers: list[int]) -> int:
    total: int = 0
    for n in numbers:
        total = total + n
    return total

def main():
    nums: list[int] = [1, 2, 3, 4, 5]
    result: int = sum_list(nums)
    print(f"Sum: {result}")
```

### Type Mapping (New)

- `list[T]` -> `Vec<T>`
- `dict[K, V]` -> `std::collections::HashMap<K, V>`
- `tuple[A, B, C]` -> `(A, B, C)`
- `range(n)` -> `0..n`

---

## M3: Advanced Type System

**Goal:** Add union types, intersection types, literal types, and full control-flow-based type narrowing to the sifr compiler. This makes sifr's type system as expressive as TypeScript's while compiling to Rust.

### Why M3 (before Error Handling)

Union types, literal types, and type narrowing are **prerequisites** for clean error handling and later milestones:

- M4's `Result[T, E]` and `Option[T]` are union-based types
- M5's discriminated unions (e.g., `Shape` with a `.tag` field) need narrowing
- M7's generics need type bounds with unions
- Every milestone after M3 benefits from the advanced type system

### Syntax Design Principles

Sifr reuses familiar syntax from Python, TypeScript, and Rust rather than inventing new constructs:

- **Python-first:** if Python has syntax for it, use that (`isinstance`, `is None`, `type` statement)
- **TypeScript for types:** where Python's typing module is verbose, borrow TypeScript's cleaner syntax (values as types: `"GET" | "POST"` instead of `Literal["GET"] | Literal["POST"]`)
- **No redundant sugar:** one way to do things. `str | None` for optionals, no `T?` shorthand
- **No user-facing syntax for internal features:** intersection types are internal to the narrowing engine, not exposed as `A & B` syntax

### Language Features

- **Union types:** `int | str`, `A | B | C` -- a value can be one of several types (Python 3.10+ syntax)
- **Literal types:** values used directly as types in type position (TypeScript style):

```python
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE"
type StatusCode = 200 | 404 | 500
type Toggle = True | False
```

- **Type aliases:** `type UserId = int`, `type HttpMethod = "GET" | "POST"` (Python 3.12 `type` statement)
- **Optional types:** `str | None` -- no shorthand, just Python's union-with-None (Python 3.10+ syntax)
- `**Unknown` type:** safe top type -- accepts any value but must be narrowed (via `isinstance`, equality, etc.) before use. Unlike `Any` which opts out of type checking, `Unknown` forces the programmer to prove the type before operating on it
- **Type narrowing via control flow analysis:**
  - Truthiness checks: `if x:` narrows `x: str | None` to `x: str`
  - `isinstance()` checks: `if isinstance(x, int):` narrows union (Python built-in)
  - Equality checks: `if x == "GET":` narrows `x: str` to `x: "GET"` in the then-branch
  - `is None` / `is not None` checks (Python idiom)
  - `not` negation: else branches get the complement type
- **Type predicates:** user-defined narrowing via return type annotation (Python typing style):

```python
def is_string(x: int | str) -> TypeGuard[str]:
    return isinstance(x, str)

# Usage: if is_string(val): ... val is str here
```

- `**reveal_type()` built-in:** prints inferred type at compile time (same as mypy/pyright)
- `**never` exhaustiveness:** matching all union variants leaves `never` -- compiler error if not exhaustive
- **Intersection types:** internal to the narrowing engine only. No user-facing `A & B` syntax in M3. Exposed later when protocols land in M5

Note: **Discriminated unions** (union of structs with a shared tag field) are deferred to M5 when classes exist. M3 focuses on unions of primitive/literal types with narrowing via isinstance and equality.

### Compiler Architecture Changes

#### Type System Changes

Extend the `Type` enum in `crates/sifr_type_system/src/types.rs`:

```rust
enum Type {
    // ... existing types ...

    // Union: value is one of these types
    Union(Vec<Type>),

    // Intersection: value satisfies all of these (internal, for narrowing)
    Intersection(Vec<Type>),

    // Literal types: specific values as types
    LiteralInt(i64),
    LiteralStr(String),
    LiteralBool(bool),

    // Optional sugar: T | None
    Optional(Box<Type>),

    // Type alias reference (resolved during checking)
    Alias(String, Box<Type>),

    // Safe top type: must be narrowed before use (unlike Any which opts out)
    Unknown,
}
```

Key design decisions:

- `Optional(T)` is sugar that normalizes to `Union(vec![T, None])` internally
- Union types are **flattened** and **deduplicated** (no nested unions)
- Literal types **widen** to their base type at mutable assignment (like TypeScript's fresh literal behavior)
- `Union` maps to Rust `enum` in codegen (auto-generated discriminated enum)
- `Unknown` vs `Any`: `Any` disables type checking (escape hatch). `Unknown` accepts any value but requires narrowing before any operation -- it is the safe alternative. `Unknown` maps to `Box<dyn Any>` in Rust codegen but the compiler enforces narrowing at every use site.

#### Control Flow Graph (new module: `sifr_hir/src/cfg.rs`)

**Inspired by TypeScript's binder** (see `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/binder.md`):

Build a control flow graph during HIR lowering. Each statement/expression gets a `FlowNode` that points to its antecedents:

```rust
enum FlowNode {
    Start,
    Assignment { var: String, ty: Type, antecedent: FlowNodeId },
    Condition { expr: HirExprId, true_branch: FlowNodeId, false_branch: FlowNodeId },
    Label { antecedents: Vec<FlowNodeId> },  // join point
    Unreachable,
}
```

#### Narrowing Engine (new module: `sifr_type_system/src/narrow.rs`)

**Inspired by TypeScript's checker narrowing** (see `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-widening-narrowing.md`) and **ty's intersection-based narrowing**:

```rust
/// Narrow a type based on a condition being true/false.
fn narrow_type(ty: &Type, condition: &NarrowingCondition, is_true: bool) -> Type

enum NarrowingCondition {
    Truthiness(VarId),                          // if x:
    IsNone(VarId),                              // if x is None
    IsNotNone(VarId),                           // if x is not None
    IsInstance(VarId, Type),                     // if isinstance(x, int)
    Equality(VarId, LiteralValue),              // if x == "GET"
    TypePredicate(VarId, Type),                 // user-defined guard
    AttributeEquality(VarId, String, LiteralValue), // if x.tag == "circle"
    Not(Box<NarrowingCondition>),               // negation
    And(Vec<NarrowingCondition>),               // conjunction
    Or(Vec<NarrowingCondition>),                // disjunction
}
```

#### Scope Changes (update `sifr_hir/src/scope.rs`)

The scope must track **narrowed types** per variable at each point in the control flow:

```rust
struct VariableInfo {
    declared_type: Type,     // the annotation or inferred type
    narrowed_type: Type,     // current type after narrowing (starts = declared_type)
    is_moved: bool,
}
```

#### Codegen Changes (update `sifr_codegen/src/lib.rs`)

Union types map to Rust enums:

```python
# Sifr
x: int | str = 42
```

```rust
// Generated Rust
enum IntOrStr {
    Int(i64),
    Str(String),
}
let x: IntOrStr = IntOrStr::Int(42);
```

Narrowing maps to `match` or `if let`:

```python
# Sifr
def process(x: int | str):
    if isinstance(x, int):
        print(x + 1)     # x is int here
    else:
        print(x.upper())  # x is str here
```

```rust
// Generated Rust
fn process(x: IntOrStr) {
    match &x {
        IntOrStr::Int(x_val) => {
            println!("{}", x_val + 1);
        }
        IntOrStr::Str(x_val) => {
            println!("{}", x_val.to_uppercase());
        }
    }
}
```

### Example Programs (M3)

**Union types and narrowing:**

```python
type Shape = "circle" | "square"

def area(shape: Shape, size: float) -> float:
    if shape == "circle":
        return 3.14159 * size * size
    else:
        return size * size

def main():
    print(area("circle", 5.0))
    print(area("square", 4.0))
```

**Optional / None narrowing:**

```python
def find_user(name: str) -> str | None:
    if name == "alice":
        return "Alice Smith"
    return None

def main():
    user: str | None = find_user("alice")
    if user is not None:
        print(user.upper())   # narrowed to str
    else:
        print("not found")
```

**isinstance narrowing:**

```python
def describe(x: int | str) -> str:
    if isinstance(x, int):
        return f"number: {x + 1}"   # x is int here
    else:
        return f"text: {x.upper()}"  # x is str here

def main():
    print(describe(42))
    print(describe("hello"))
```

**Type predicates:**

```python
def is_nonempty(s: str | None) -> TypeGuard[str]:
    return s is not None and len(s) > 0

def main():
    name: str | None = "alice"
    if is_nonempty(name):
        print(name.upper())  # name narrowed to str
```

**Unknown type (safe top type):**

```python
def process(data: Unknown) -> str:
    if isinstance(data, str):
        return data.upper()       # narrowed to str
    if isinstance(data, int):
        return str(data)          # narrowed to int
    return "unknown"

def main():
    print(process("hello"))
    print(process(42))
```

### Files to Modify/Create for M3

**Modify:**

- `crates/sifr_type_system/src/types.rs` -- extend `Type` enum
- `crates/sifr_type_system/src/check.rs` -- type checking for unions
- `crates/sifr_type_system/src/infer.rs` -- inference with unions/literals
- `crates/sifr_hir/src/hir_nodes.rs` -- new HIR nodes for narrowing
- `crates/sifr_hir/src/lower.rs` -- lowering with CFG and narrowing
- `crates/sifr_hir/src/scope.rs` -- narrowed type tracking
- `crates/sifr_codegen/src/lib.rs` -- union -> enum codegen
- `crates/sifr_driver/src/lib.rs` -- pipeline updates

**Create:**

- `crates/sifr_type_system/src/narrow.rs` -- narrowing engine
- `crates/sifr_type_system/src/union.rs` -- union construction, normalization, simplification
- `crates/sifr_type_system/src/literal.rs` -- literal type handling, widening
- `crates/sifr_hir/src/cfg.rs` -- control flow graph
- E2E test files in `crates/sifr/tests/e2e/pass/` and `fail/`

---

## M4: Error Handling

**Goal:** Provide safe error handling that maps to Rust's `Result`/`Option` types rather than Python's exception model. Benefits from M3's union types -- `Result[T, E]` and `Option[T]` are union-based.

### Language Features

- `**Result[T, E]` type:** explicit error return type (replaces exceptions)
- `**Option[T]` type:** sugar for `T | None`, maps to Rust `Option<T>` (leverages M3's union types)
- `**try`/`except` syntax:** reinterpreted as pattern matching on `Result`
- `**?` operator:** early return on error (borrowed from Rust, new syntax for Sifr)
- `**raise` -> `Err()`:** raising maps to returning an error
- **Custom error types:** classes that implement an `Error` protocol
- `**assert` statement**

### Design Decision

Sifr does NOT use Python's exception model (stack unwinding). Instead, errors are values:

```python
def parse_int(s: str) -> Result[int, str]:
    # ...implementation...
    raise "not a number"   # becomes Err("not a number".to_string())

def main():
    result = parse_int("42")?   # early return on error
    print(result)
```

This maps cleanly to Rust's `Result<T, E>` and `?` operator.

---

## M5: Structs and Methods (OOP)

**Goal:** Support class-based programming that compiles to Rust structs with impl blocks. Benefits from M3's discriminated unions and narrowing.

### Design Decision: Nominal vs Structural Typing

Sifr uses **nominal typing by default** (like Rust) with **structural matching via protocols** (like TypeScript's interfaces):

- Two classes with identical fields are NOT automatically assignable to each other (nominal)
- A `Protocol` defines a structural contract -- any class that has the required fields/methods satisfies it (structural)
- This matches Rust's trait system: types are distinct, but traits provide shared interfaces

This is a deliberate middle ground between TypeScript (fully structural) and Rust (fully nominal). Protocols give the flexibility of structural typing where needed, while nominal classes prevent accidental type confusion.

### Language Features

- `**class` -> `struct` + `impl`:** class definitions become Rust structs
- `**__init__` -> `new()`:** constructor mapping
- **Methods:** `self` parameter maps to `&self` or `&mut self`
- **Properties:** `@property` maps to getter methods
- **Protocols/Interfaces:** `Protocol` classes map to Rust traits (structural matching -- any class with the right shape satisfies the protocol)
- `**isinstance` -> type narrowing:** compile-time type checking (leverages M3's narrowing)
- **Inheritance:** single inheritance via trait delegation (not Rust inheritance, which doesn't exist)
- **Operator overloading:** `__add__`, `__eq__`, etc. map to Rust trait impls (`Add`, `PartialEq`)
- **Discriminated unions:** classes with a shared literal-typed tag field, narrowed via attribute equality (leverages M3's narrowing engine):

```python
class Circle:
    tag: "circle" = "circle"
    radius: float

class Square:
    tag: "square" = "square"
    side: float

type Shape = Circle | Square

def area(shape: Shape) -> float:
    if shape.tag == "circle":
        return 3.14159 * shape.radius * shape.radius  # narrowed to Circle
    else:
        return shape.side * shape.side                  # narrowed to Square
```

- **Property existence narrowing (`in`):** `if "name" in obj:` narrows the type to one that has a `name` field (extends M3's narrowing to object properties)

### Example Program

```python
class Point:
    x: float
    y: float

    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y

    def distance(self, other: Point) -> float:
        dx: float = self.x - other.x
        dy: float = self.y - other.y
        return (dx * dx + dy * dy) ** 0.5

def main():
    p1 = Point(0.0, 0.0)
    p2 = Point(3.0, 4.0)
    print(p1.distance(p2))  # 5.0
```

### Generated Rust

```rust
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx: f64 = self.x - other.x;
        let dy: f64 = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}
```

---

## M6: Module System

**Goal:** Support multi-file projects with imports, enabling real application structure.

### Language Features

- `**import` / `from ... import`:** maps to Rust `mod` / `use`
- **Multi-file compilation:** compile a directory of `.sifr` files into one binary
- **Package structure:** `__init__.sifr` defines a package (like `mod.rs`)
- **Visibility:** `_private` prefix convention enforced as `pub`/non-`pub`
- `**sifr.toml`:** project manifest (like `Cargo.toml` + `pyproject.toml`)
- **Dependency management:** `sifr add <package>`, resolve from a registry or git

### Project Structure

```
my_app/
  sifr.toml
  src/
    main.sifr
    models/
      __init__.sifr
      user.sifr
    utils/
      __init__.sifr
      helpers.sifr
```

### Example

```python
# src/models/user.sifr
class User:
    name: str
    email: str

    def __init__(self, name: str, email: str):
        self.name = name
        self.email = email

# src/main.sifr
from models.user import User

def main():
    user = User("Alice", "alice@example.com")
    print(user.name)
```

---

## M7: Generics and Advanced Types

**Goal:** Support generic programming, closures, and higher-order functions. Union types and type aliases already exist from M3, so this focuses on parameterized types.

### Language Features

- **Generic functions:** `def first[T](items: list[T]) -> T` (Python 3.12 syntax)
- **Generic classes:** `class Stack[T]:` (Python 3.12 syntax)
- **Type bounds:** `def sort[T: Comparable](items: list[T])`
- **Closures / lambdas:** `lambda x: x + 1` maps to Rust closures
- **Contextual typing for lambdas:** lambda parameter types inferred from call-site context (e.g., `map_list(numbers, lambda x: x * 2)` infers `x: int` from `list[int]`)
- **Higher-order functions:** `map`, `filter`, `reduce` on collections
- **Iterators:** `__iter__` / `__next__` protocol maps to Rust `Iterator` trait
- **Utility types (TypeScript-inspired):** built-in type aliases for common transformations:
  - `Partial[T]` -- all fields optional (maps to `Option<field>` for each field)
  - `Readonly[T]` -- all fields immutable (maps to non-`mut` references)
  - `Pick[T, "field1", "field2"]` -- subset of fields
  - `Omit[T, "field1"]` -- all fields except specified
  - `Record[K, V]` -- sugar for `dict[K, V]`
- **Mapped/conditional types (stretch):** type-level programming

### Example Program

```python
def map_list[T, U](items: list[T], f: (T) -> U) -> list[U]:
    result: list[U] = []
    for item in items:
        result.append(f(item))
    return result

def main():
    numbers: list[int] = [1, 2, 3, 4, 5]
    doubled = map_list(numbers, lambda x: x * 2)
    print(doubled)
```

---

## M8: Standard Library

**Goal:** Provide essential built-in functionality for real programs.

### Modules

- `**sifr.io`:** file read/write, stdin/stdout, path operations
- `**sifr.json`:** JSON serialization/deserialization (wraps `serde_json`)
- `**sifr.env`:** environment variables
- `**sifr.fmt`:** string formatting, f-string internals
- `**sifr.math`:** math functions (sqrt, pow, abs, min, max, etc.)
- `**sifr.collections`:** `Set`, `OrderedDict`, `Deque`
- `**sifr.time`:** timestamps, durations, sleep
- `**sifr.random`:** random number generation
- `**sifr.os`:** process spawning, signals, exit codes
- `**sifr.re`:** regular expressions (wraps `regex` crate)

### Implementation Strategy

Each stdlib module is a thin Sifr wrapper around battle-tested Rust crates:

- `sifr.json` -> `serde` + `serde_json`
- `sifr.re` -> `regex`
- `sifr.time` -> `std::time` + `chrono`
- `sifr.io` -> `std::fs` + `std::io`

---

## M9: Async and Networking

**Goal:** Support async programming and HTTP, enabling web applications.

### Language Features

- `**async def` / `await`:** maps to Rust `async fn` / `.await`
- **Async runtime:** built on `tokio`
- `**sifr.http`:** HTTP client and server primitives
- `**sifr.net`:** TCP/UDP sockets
- `**sifr.web`:** minimal web framework (routing, request/response, middleware)

### Example: Web Application

```python
from sifr.web import App, Request, Response

app = App()

@app.route("/")
async def index(req: Request) -> Response:
    return Response.text("Hello, World!")

@app.route("/users/{id}")
async def get_user(req: Request) -> Response:
    user_id: str = req.params["id"]
    return Response.json({"id": user_id, "name": "Alice"})

def main():
    app.run(host="0.0.0.0", port=8000)
```

### Generated Rust (Conceptual)

The web framework layer wraps `axum` or `hyper`:

```rust
use axum::{Router, routing::get, response::IntoResponse};

async fn index() -> impl IntoResponse {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## M10: Metaprogramming

**Goal:** Support decorators and compile-time code generation.

### Language Features

- **Decorators:** `@decorator` maps to Rust attribute macros or code transforms
- `**@dataclass`:** auto-generate `__init__`, `__eq__`, `__repr__` (like Rust `#[derive]`)
- `**@property`:** getter/setter generation
- **Custom decorators:** user-defined compile-time transforms
- `***args` / `**kwargs`:** variadic arguments via macro expansion or trait objects
- **Compile-time evaluation:** `const` expressions evaluated at compile time

### Example

```python
@dataclass
class Config:
    host: str
    port: int
    debug: bool = False

# Auto-generates __init__, __eq__, __repr__, clone
# Maps to Rust #[derive(Debug, Clone, PartialEq)] struct
```

---

## M11: Production Readiness

**Goal:** Make Sifr a complete, usable language ecosystem.

### Tooling

- **LSP server (`sifr_lsp`):** autocomplete, go-to-definition, hover types, diagnostics
- **Formatter (`sifr fmt`):** opinionated code formatter (like `ruff format` / `rustfmt`)
- **Linter (`sifr lint`):** catch common mistakes beyond type errors
- **Package registry:** `sifr.dev` -- publish and install packages
- **Documentation generator:** `sifr doc` -- generate HTML docs from docstrings
- **REPL:** `sifr repl` -- interactive mode (compile-and-run snippets)

### Interop

- **Rust FFI:** call Rust crates directly from Sifr code
- **C FFI:** call C libraries via `unsafe` blocks
- **Python interop (stretch):** call Python libraries via PyO3 bindings

### Performance

- **Incremental compilation:** only recompile changed modules
- **Build caching:** cache generated Rust code and compiled artifacts
- **Parallel compilation:** compile independent modules in parallel

---

## Milestone Summary

```
M1:  Core Language (DONE)    -> "Hello World" compiles to native binary
M2:  Control Flow + Data (DONE) -> Process collections, loops, real algorithms
M3:  Advanced Type System    -> Union/intersection types, literal types, type narrowing
M4:  Error Handling          -> Safe error propagation via Result/Option (uses M3 unions)
M5:  Structs + Methods       -> OOP, data modeling, discriminated unions (uses M3 narrowing)
M6:  Module System           -> Multi-file projects, packages, dependencies
M7:  Generics + Closures     -> Generic programming, higher-order functions
M8:  Standard Library        -> File I/O, JSON, time, regex, OS operations
M9:  Async + Networking      -> Web servers, HTTP clients, async I/O
M10: Metaprogramming         -> Decorators, dataclasses, compile-time code gen
M11: Production Readiness    -> LSP, formatter, package registry, FFI
```

After M9, Sifr can build web applications. After M11, it is a complete language ecosystem.

---

## Type System Design

### Core Types (Full)

```rust
enum Type {
    // Primitives (Copy)
    Int,
    Float,
    Bool,
    Str,
    None,

    // Compound (Move)
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Set(Box<Type>),

    // Literal types (Copy) -- specific values as types (M3)
    LiteralInt(i64),
    LiteralStr(String),
    LiteralBool(bool),

    // Union / Intersection (M3)
    Union(Vec<Type>),           // int | str -- flattened, deduplicated
    Intersection(Vec<Type>),    // internal only, for narrowing engine

    // Type alias (M3)
    Alias(String, Box<Type>),   // type HttpMethod = "GET" | "POST"

    // Function
    Function(FunctionType),

    // Class instance (M5)
    Instance(ClassId),

    // Generics (M7)
    TypeVar(TypeVarId),
    GenericInstance(ClassId, Vec<Type>),

    // Result / Option (M4)
    Result(Box<Type>, Box<Type>),

    // Range (M2)
    Range,

    // Safe top type: must be narrowed before use (M3)
    Unknown,

    // Escape hatch: opts out of type checking
    Any,

    // Bottom
    Never,
}
```

### Literal Type Behavior (TypeScript-inspired)

Literal types represent specific values at the type level. In sifr, values are used directly as types in type position (TypeScript style), avoiding Python's verbose `Literal[...]` wrapper:

```python
type HttpMethod = "GET" | "POST" | "PUT"    # not Literal["GET"] | Literal["POST"] | ...
type StatusCode = 200 | 404 | 500
x: "hello" = "hello"                        # literal type annotation
```

Key behaviors:

- **Fresh literals widen at mutable locations:** `x = 42` infers `x: int` (widened), but `x: 42 = 42` preserves the literal type
- **Literal types are subtypes of their base type:** `42` is assignable to `int`, `"GET"` is assignable to `str`
- **Equality narrows to literals:** `if x == "GET":` narrows `x: str` to `x: "GET"` in the then-branch
- **Union of literals:** `"GET" | "POST"` is a valid type representing exactly two string values

### Union Type Behavior

- **Flattened:** `Union(vec![Union(vec![A, B]), C])` normalizes to `Union(vec![A, B, C])`
- **Deduplicated:** `Union(vec![Int, Int, Str])` normalizes to `Union(vec![Int, Str])`
- **Single-element unions collapse:** `Union(vec![Int])` becomes `Int`
- **Subtyping:** `A` is assignable to `A | B`; `A | B` is assignable to `C` only if both `A` and `C` and `B` and `C` are assignable
- **Codegen:** `int | str` generates a Rust enum `enum IntOrStr { Int(i64), Str(String) }`

### Type Narrowing (TypeScript-inspired, M3)

Narrowing refines a variable's type within a control flow branch:

- **Truthiness:** `if x:` removes `None` and falsy types from unions
- **isinstance:** `if isinstance(x, int):` narrows `x: int | str` to `x: int`
- **Equality:** `if x == "GET":` narrows to literal type
- **is None / is not None:** narrows optional types
- **Type predicates:** `def is_str(x: int | str) -> TypeGuard[str]:` enables user-defined narrowing
- **Assertion functions:** `def assert_int(x: int | str) -> AssertType[int]:` narrows after call
- **Exhaustiveness:** after narrowing all variants of a union, the remaining type is `Never` -- compiler error if not exhaustive

```

### Ownership Model

- All types are **move by default** (like Rust)
- Primitive types (`int`, `float`, `bool`) are `Copy` -- assignment copies
- Compound types (`str`, `list`, `dict`, classes) **move** on assignment
- Explicit `.clone()` for deep copy
- References via `ref` keyword (maps to `&T`)
- Mutable references via `mut ref` (maps to `&mut T`)
- Function arguments: move by default, use `ref` for borrowing

### Type Inference Strategy

- **Initializer inference:** `x = 42` infers `x: int` (literal widens to base type)
- **Return type inference:** analyze all return paths
- **Contextual typing (M7):** lambda/callback parameter types inferred from call-site context. E.g., `map_list(numbers, lambda x: x * 2)` infers `x: int` from the `list[int]` argument. Inspired by TypeScript's contextual typing which looks upward in the tree for type annotations.
- **Enforced annotations:** function parameters MUST have types (or be inferable from defaults)
- **Literal preservation:** `x: "GET" = "GET"` preserves the literal type; `x = "GET"` widens to `str`

---

## Test Suite Architecture

This compiler is built entirely by AI agents. The test suite is the contract that ensures correctness across all agents working on different parts of the compiler. It must be:

- **Deterministic:** same input always produces same output
- **Self-documenting:** test files are readable specifications of language behavior
- **Layered:** each compiler phase has its own test layer
- **Easy to extend:** adding a new language feature means adding test files, not modifying test infrastructure
- **Fast to run:** `cargo test` completes in seconds for the full suite

### Testing Strategy Overview

```mermaid
flowchart TD
    subgraph layer1 [Layer 1: Unit Tests]
        LexerUnit["Lexer unit tests\n(token output)"]
        ASTUnit["AST node tests\n(construction, size)"]
        TypeUnit["Type system tests\n(subtyping, inference)"]
    end
    subgraph layer2 [Layer 2: Snapshot Tests]
        ParseSnap["Parser snapshots\n(.sifr -> AST dump)"]
        TypeSnap["Type checker snapshots\n(inline assertions)"]
        CodegenSnap["Codegen snapshots\n(.sifr -> .rs output)"]
    end
    subgraph layer3 [Layer 3: End-to-End Tests]
        E2EPass["Compile + run tests\n(expected stdout)"]
        E2EFail["Compile-fail tests\n(expected errors)"]
        E2EOwnership["Ownership tests\n(move/borrow errors)"]
    end
    subgraph layer4 [Layer 4: Corpus Tests]
        Corpus["Corpus tests\n(no panics on large inputs)"]
    end
    layer1 --> layer2 --> layer3 --> layer4
```

### Layer 1: Unit Tests (per crate, `#[cfg(test)]`)

Standard Rust unit tests inside each crate. These test individual functions and data structures.

**Where:** `src/*.rs` in each crate, in `#[cfg(test)] mod tests { }` blocks.

**Examples:**

- Lexer: tokenize a string, assert token sequence
- AST: construct nodes, verify `Debug` output, check memory layout sizes
- Type system: `is_subtype(Int, Int) == true`, `is_subtype(Int, Str) == false`
- HIR: name resolution resolves `x` to the correct `DefId`

**Pattern (from ruff_python_ast):**

```rust
#[test]
fn size() {
    assert!(std::mem::size_of::<Stmt>() <= 120);
    assert_eq!(std::mem::size_of::<Expr>(), 64);
}
```

### Layer 2: Snapshot Tests (insta crate)

Snapshot testing using the `insta` crate. The compiler produces output that is compared against stored `.snap` files. When behavior changes intentionally, run `cargo insta review` to accept new baselines.

**Crate:** `insta` with `glob` feature.

#### 2a. Parser Snapshots

**Inspired by:** ruff_python_parser's fixture-driven snapshot tests.

**Directory structure:**

```
crates/sifr_python_parser/
  resources/
    valid/          # .sifr files that must parse successfully
      expressions/
        arithmetic.sifr
        boolean.sifr
        string.sifr
      statements/
        assignment.sifr
        if_else.sifr
        function_def.sifr
    invalid/        # .sifr files that must produce parse errors
      missing_colon.sifr
      bad_indent.sifr
      unterminated_string.sifr
  tests/
    snapshots/      # auto-generated .snap files
    fixtures.rs     # test harness
```

**Test harness (`fixtures.rs`):**

```rust
#[test]
fn test_valid_syntax() {
    insta::glob!("../resources/valid/**/*.sifr", |path| {
        let source = std::fs::read_to_string(path).unwrap();
        let parsed = parse_module(&source);
        assert!(parsed.is_valid(), "Parse errors: {:?}", parsed.errors());

        let mut output = String::new();
        writeln!(&mut output, "## AST\n\n```\n{:#?}\n```", parsed.syntax()).unwrap();

        insta::with_settings!({
            input_file => path,
            omit_expression => true,
        }, {
            insta::assert_snapshot!(output);
        });
    });
}

#[test]
fn test_invalid_syntax() {
    insta::glob!("../resources/invalid/**/*.sifr", |path| {
        let source = std::fs::read_to_string(path).unwrap();
        let parsed = parse_module(&source);
        assert!(!parsed.is_valid());

        let mut output = String::new();
        writeln!(&mut output, "## AST\n\n```\n{:#?}\n```", parsed.syntax()).unwrap();
        writeln!(&mut output, "\n## Errors\n").unwrap();
        for error in parsed.errors() {
            writeln!(&mut output, "  {}", error).unwrap();
        }

        insta::with_settings!({
            input_file => path,
        }, {
            insta::assert_snapshot!(output);
        });
    });
}
```

#### 2b. Type Checker Snapshots (Markdown Tests)

**Inspired by:** ty's mdtest framework -- Markdown files with inline assertions.

**Directory structure:**

```
crates/sifr_type_system/
  resources/
    mdtest/
      basics/
        literals.md
        variables.md
        arithmetic.md
      functions/
        parameters.md
        return_types.md
        inference.md
      ownership/
        move_semantics.md
        copy_types.md
        borrow.md
      errors/
        type_mismatch.md
        undefined_variable.md
  tests/
    mdtest.rs       # test harness using datatest-stable
```

**Markdown test format:**

```markdown
# Variable type inference

## Integer literal

`​`​`sifr
x = 42
reveal_type(x)  # revealed: int
`​`​`

## String literal

`​`​`sifr
name = "hello"
reveal_type(name)  # revealed: str
`​`​`

## Type mismatch

`​`​`sifr
x: int = "hello"  # error: [type-mismatch] expected `int`, got `str`
`​`​`

## Move semantics

`​`​`sifr
a: str = "hello"
b: str = a
print(a)  # error: [use-after-move] `a` was moved to `b`
`​`​`
```

**Assertion syntax:**

- `# revealed: <type>` -- assert inferred type (like ty)
- `# error: [rule-code] "optional message"` -- assert diagnostic
- `# error: <col> [rule-code]` -- assert diagnostic at specific column

#### 2c. Codegen Snapshots

**Inspired by:** TypeScript's `.js` baseline files.

**Directory structure:**

```
crates/sifr_codegen/
  resources/
    codegen/
      hello_world.sifr
      arithmetic.sifr
      functions.sifr
      if_else.sifr
      string_ops.sifr
  tests/
    snapshots/      # .snap files with expected Rust output
    codegen.rs      # test harness
```

**Test harness:**

```rust
#[test]
fn test_codegen() {
    insta::glob!("../resources/codegen/**/*.sifr", |path| {
        let source = std::fs::read_to_string(path).unwrap();
        let rust_output = compile_to_rust(&source).unwrap();

        insta::with_settings!({
            input_file => path,
        }, {
            insta::assert_snapshot!(rust_output);
        });
    });
}
```

**Snapshot content (e.g. `hello_world.sifr.snap`):**

```
---
source: crates/sifr_codegen/tests/codegen.rs
input_file: crates/sifr_codegen/resources/codegen/hello_world.sifr
---
fn main() {
    println!("{}", "Hello, World!");
}
```

### Layer 3: End-to-End Tests (Compile + Run)

**Inspired by:** Mojo's Lit + FileCheck pattern, adapted for Rust.

These tests compile `.sifr` files to binaries, run them, and check stdout/stderr.

**Directory structure:**

```
tests/
  e2e/
    pass/           # must compile and produce expected output
      hello_world.sifr
      factorial.sifr
      fibonacci.sifr
      arithmetic.sifr
      string_concat.sifr
      if_else.sifr
    fail/            # must fail to compile with expected errors
      type_mismatch.sifr
      undefined_var.sifr
      missing_return_type.sifr
      use_after_move.sifr
    ownership/       # ownership-specific compile failures
      move_on_assign.sifr
      double_move.sifr
      borrow_after_move.sifr
  e2e.rs             # test runner
```

**Test file format (pass tests) -- inline expected output:**

```python
# expect-stdout: 120
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    print(factorial(5))
```

**Test file format (fail tests) -- inline expected errors:**

```python
# expect-error: [type-mismatch]
def main():
    x: int = "hello"
```

**Test runner (`e2e.rs`):**

```rust
#[test]
fn test_e2e_pass() {
    for path in glob("tests/e2e/pass/**/*.sifr") {
        let source = fs::read_to_string(&path).unwrap();
        let expected_stdout = extract_expect_stdout(&source);

        // Compile to Rust, build, and run
        let output = compile_and_run(&path).unwrap();
        assert_eq!(output.stdout.trim(), expected_stdout,
            "Failed: {}", path.display());
    }
}

#[test]
fn test_e2e_fail() {
    for path in glob("tests/e2e/fail/**/*.sifr") {
        let source = fs::read_to_string(&path).unwrap();
        let expected_errors = extract_expect_errors(&source);

        let result = compile(&path);
        assert!(result.is_err());
        for expected in expected_errors {
            assert!(result.errors().any(|e| e.code() == expected));
        }
    }
}
```

### Layer 4: Corpus Tests (Robustness)

**Inspired by:** ty's corpus tests -- ensure the compiler doesn't panic on large/varied inputs.

**Purpose:** Run the parser and type checker on a large body of Python source code to catch panics, infinite loops, and crashes. These tests don't check correctness -- only that the compiler doesn't blow up.

**Sources:**

- Ruff's parser test fixtures (`/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_python_parser/resources/`)
- Python stdlib source files
- Any `.sifr` files in the test suite

```rust
#[test]
fn corpus_no_panics() {
    for path in glob("tests/corpus/**/*.sifr") {
        let source = fs::read_to_string(&path).unwrap();
        // Must not panic
        let _ = parse_module(&source);
    }
}
```

### Test Infrastructure Crate: `sifr_test_utils`

A shared crate providing test helpers used across all other crates:

```
crates/sifr_test_utils/
  src/
    lib.rs
    assertions.rs    # extract_expect_stdout, extract_expect_errors
    compile.rs       # compile_to_rust, compile_and_run helpers
    fixtures.rs      # fixture loading, glob helpers
    mdtest.rs        # markdown test parser (inline assertions)
```

**Key functions:**

- `extract_expect_stdout(source: &str) -> &str` -- parse `# expect-stdout:` header
- `extract_expect_errors(source: &str) -> Vec<&str>` -- parse `# expect-error:` comments
- `compile_to_rust(source: &str) -> Result<String, Vec<Diagnostic>>` -- full pipeline
- `compile_and_run(path: &Path) -> Result<Output, Error>` -- compile, build, execute
- `parse_mdtest(markdown: &str) -> Vec<TestCase>` -- parse markdown test files

### Test Commands

```bash
# Run all tests
cargo test

# Run specific layer
cargo test -p sifr_python_parser           # Parser snapshots
cargo test -p sifr_type_system -- mdtest    # Type checker markdown tests
cargo test -p sifr_codegen                  # Codegen snapshots
cargo test --test e2e                       # End-to-end tests

# Update snapshots after intentional changes
cargo insta review

# Run corpus tests (slower)
cargo test -- corpus --ignored
```

### Adding Tests for New Features (Agent Workflow)

When an AI agent adds a new language feature, it must:

1. **Parser:** Add `.sifr` fixture files in `resources/valid/` and `resources/invalid/`
2. **Type checker:** Add markdown test cases in `resources/mdtest/`
3. **Codegen:** Add `.sifr` fixture files in `resources/codegen/`
4. **E2E:** Add pass/fail test files in `tests/e2e/`
5. **Run `cargo insta review**` to accept new snapshots
6. **Run `cargo test**` to verify everything passes

This ensures every feature is tested at every layer of the compiler, and any agent can verify the full system by running `cargo test`.

---

## Design Note: Mojo Comparison

Mojo (`/Users/yaseralnajjar/work/sifr/modular/mojo`) was evaluated as a reference. Key findings:

- **No Rust code to reuse.** Mojo's compiler is proprietary, built on MLIR/LLVM (C++). The open-source repo only contains the stdlib, docs, and design proposals.
- **Ownership model difference:** Mojo chose **borrow-by-default** for function arguments. Sifr uses **move-by-default** (like Rust). This is a deliberate tradeoff -- Sifr prioritizes Rust-like safety and explicitness.
- **Useful design references:** `proposals/value-ownership.md` and `proposals/lifetimes-and-provenance.md` document tradeoffs between move/borrow defaults, ASAP destruction, and lifecycle methods.
- `**def` vs `fn` split:** Mojo uses `def` for dynamic and `fn` for strict. Sifr does not need this split since all code is strictly typed.

## Key Files to Reference During Implementation

### Ruff (parser, AST)

- **Ruff parser:** `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_python_parser/`
- **Ruff AST:** `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_python_ast/src/nodes.rs`

### ty (type checker)

- **ty type system:** `/Users/yaseralnajjar/work/sifr/ty/ruff/crates/ty_python_semantic/src/types.rs`

### TypeScript (type system design, narrowing, control flow analysis)

- **Checker architecture:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker.md`
- **Type narrowing and widening:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-widening-narrowing.md`
- **Type relations (subtyping, assignability):** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-relations.md`
- **Type inference:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-inference.md`
- **Binder (control flow graph):** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/binder.md`
- **Type definitions:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/types.md`
- **TypeScript wiki:** `/Users/yaseralnajjar/work/sifr/TypeScript.wiki/`

### Mojo (ownership model)

- **Mojo ownership design:** `/Users/yaseralnajjar/work/sifr/modular/mojo/proposals/value-ownership.md`
- **Mojo lifetimes design:** `/Users/yaseralnajjar/work/sifr/modular/mojo/proposals/lifetimes-and-provenance.md`

