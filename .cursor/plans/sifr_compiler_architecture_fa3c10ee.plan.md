---
name: Sifr Compiler Architecture
overview: Build "sifr", a compiled programming language with Python syntax and enforced typing that emits Rust source code, compiled via rustc into native binaries. The compiler is built in Rust, forking ruff's parser/AST crates and adding type checking, IR, and Rust codegen phases. The end goal is a language capable of building web applications and general-purpose programs.
todos:
  - id: fork-parser
    content: "M1: Fork and rename 6 ruff crates (text_size, source_file, python_trivia, python_ast, python_parser, python_literal) into crates/ with sifr_ prefix. Set up Cargo workspace."
    status: pending
  - id: strip-ast
    content: "M1: Strip the forked AST to only the nodes needed for M1 (function def, if/elif/else, assign, ann_assign, return, expr, basic expressions, literals). Remove IPython, match, async, with, try, import, etc."
    status: pending
  - id: type-system
    content: "M1: Build sifr_type_system crate -- Type enum (Int, Float, Bool, Str, None, Function, Any, Never), type inference from initializers, type checking (binary ops, comparisons, function calls), subtyping rules."
    status: pending
  - id: hir
    content: "M1: Build sifr_hir crate -- Typed IR with resolved names and types on every node. Name resolution (scopes). Ownership tracking (move vs copy)."
    status: pending
  - id: codegen
    content: "M1: Build sifr_codegen crate -- Walk HIR and emit Rust source code. Type mapping (int->i64, str->String, etc.). Generate Cargo.toml + main.rs. Handle print() as println! macro."
    status: pending
  - id: driver
    content: "M1: Build sifr_driver crate -- Orchestrate parse -> type-check -> HIR -> codegen pipeline. Error reporting with source spans and nice diagnostics (use miette or ariadne)."
    status: pending
  - id: cli
    content: "M1: Build sifr CLI binary -- sifr build/run/check/emit commands using clap. Invoke cargo build on generated Rust project."
    status: pending
  - id: test-e2e
    content: "M1: End-to-end test -- Write sample .sifr programs (hello world, factorial, fibonacci, basic arithmetic) and verify they compile and run correctly."
    status: pending
isProject: false
---

# Sifr Compiler -- Architecture and Implementation Plan

## Vision

Sifr is a compiled programming language that uses Python syntax with enforced static typing. It compiles Python-like source code to Rust source code, which is then compiled by `rustc` into native binaries. Ownership semantics follow Rust's move-by-default model. Types are strict with an opt-in `Any` escape hatch (like TypeScript's strict mode).

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
    M1["M1: Core Language\nVariables, functions, if/else,\nprimitives, print, CLI"] --> M2
    M2["M2: Control Flow and Data\nLoops, list, dict, tuple,\nstring ops, indexing"] --> M3
    M3["M3: Error Handling\nResult/Option types,\ntry/except -> match,\ncustom error types"] --> M4
    M4["M4: Structs and Methods\nclass -> struct + impl,\n__init__ -> new, methods,\ntraits/protocols"] --> M5
    M5["M5: Module System\nimport/from -> mod/use,\nmulti-file projects,\npackage manager"] --> M6
    M6["M6: Generics and Collections\nType parameters, generic\nfunctions/structs, iterators,\nclosures/lambdas"] --> M7
    M7["M7: Standard Library\nFile I/O, JSON, env vars,\nstring formatting, math,\ncollections utilities"] --> M8
    M8["M8: Async and Networking\nasync/await -> tokio,\nHTTP server/client,\nweb framework primitives"] --> M9
    M9["M9: Metaprogramming\nDecorators -> proc macros,\nattribute macros,\nderiving traits"] --> M10
    M10["M10: Production Readiness\nLSP server, formatter,\npackage registry,\nFFI, documentation"]
```



---

## Crate Structure (Rust Workspace)

```
sifr/
  Cargo.toml                (workspace root)
  crates/
    sifr_text_size/         (forked from ruff_text_size)
    sifr_source_file/       (forked from ruff_source_file)
    sifr_python_trivia/     (forked from ruff_python_trivia)
    sifr_python_ast/        (forked from ruff_python_ast)
    sifr_python_parser/     (forked from ruff_python_parser)
    sifr_python_literal/    (forked from ruff_python_literal)
    sifr_hir/               (High-level IR: typed AST after name resolution + type checking)
    sifr_type_system/       (type definitions, inference, checking, subtyping)
    sifr_codegen/           (Rust source code generation from HIR)
    sifr_driver/            (orchestrates the pipeline, error reporting)
    sifr/                   (CLI binary: sifr build, sifr check, sifr run)
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

1. Fork and rename 6 ruff crates into `crates/` with `sifr_` prefix
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

## M3: Error Handling

**Goal:** Provide safe error handling that maps to Rust's `Result`/`Option` types rather than Python's exception model.

### Language Features

- `**Result[T, E]` type:** explicit error return type (replaces exceptions)
- `**Option[T]` type:** sugar for `T | None`, maps to Rust `Option<T>`
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

## M4: Structs and Methods (OOP)

**Goal:** Support class-based programming that compiles to Rust structs with impl blocks.

### Language Features

- `**class` -> `struct` + `impl`:** class definitions become Rust structs
- `**__init__` -> `new()`:** constructor mapping
- **Methods:** `self` parameter maps to `&self` or `&mut self`
- **Properties:** `@property` maps to getter methods
- **Protocols/Interfaces:** `Protocol` classes map to Rust traits
- `**isinstance` -> type narrowing:** compile-time type checking
- **Inheritance:** single inheritance via trait delegation (not Rust inheritance, which doesn't exist)
- **Operator overloading:** `__add__`, `__eq__`, etc. map to Rust trait impls (`Add`, `PartialEq`)

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

## M5: Module System

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

## M6: Generics and Advanced Types

**Goal:** Support generic programming, closures, and advanced type system features.

### Language Features

- **Generic functions:** `def first[T](items: list[T]) -> T`
- **Generic classes:** `class Stack[T]:`
- **Type bounds:** `def sort[T: Comparable](items: list[T])`
- **Closures / lambdas:** `lambda x: x + 1` maps to Rust closures
- **Higher-order functions:** `map`, `filter`, `reduce` on collections
- **Iterators:** `__iter__` / `__next__` protocol maps to Rust `Iterator` trait
- **Union types:** `int | str` maps to Rust enums
- **Type aliases:** `type UserId = int`
- **Literal types:** `Literal["GET", "POST"]`

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

## M7: Standard Library

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

## M8: Async and Networking

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

## M9: Metaprogramming

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

## M10: Production Readiness

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
M1:  Core Language           -> "Hello World" compiles to native binary
M2:  Control Flow + Data     -> Process collections, loops, real algorithms
M3:  Error Handling          -> Safe error propagation via Result/Option
M4:  Structs + Methods       -> Object-oriented programming, data modeling
M5:  Module System           -> Multi-file projects, packages, dependencies
M6:  Generics + Closures     -> Generic programming, higher-order functions
M7:  Standard Library        -> File I/O, JSON, time, regex, OS operations
M8:  Async + Networking      -> Web servers, HTTP clients, async I/O
M9:  Metaprogramming         -> Decorators, dataclasses, compile-time code gen
M10: Production Readiness    -> LSP, formatter, package registry, FFI
```

After M8, Sifr can build web applications. After M10, it is a complete language ecosystem.

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

    // Optional / Union / Intersection
    Optional(Box<Type>),        // sugar for Union(T, None)
    Union(Vec<Type>),
    Intersection(Vec<Type>),

    // Function
    Function(FunctionType),

    // Class instance
    Instance(ClassId),

    // Generics
    TypeVar(TypeVarId),
    GenericInstance(ClassId, Vec<Type>),

    // Result / Option
    Result(Box<Type>, Box<Type>),
    Option(Box<Type>),

    // Escape hatch
    Any,

    // Bottom
    Never,
}
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

- **Initializer inference:** `x = 42` infers `x: int`
- **Return type inference:** analyze all return paths
- **Contextual typing:** lambda params inferred from call-site
- **Enforced annotations:** function parameters MUST have types (or be inferable from defaults)

---

## Design Note: Mojo Comparison

Mojo (`/Users/yaseralnajjar/work/sifr/modular/mojo`) was evaluated as a reference. Key findings:

- **No Rust code to reuse.** Mojo's compiler is proprietary, built on MLIR/LLVM (C++). The open-source repo only contains the stdlib, docs, and design proposals.
- **Ownership model difference:** Mojo chose **borrow-by-default** for function arguments. Sifr uses **move-by-default** (like Rust). This is a deliberate tradeoff -- Sifr prioritizes Rust-like safety and explicitness.
- **Useful design references:** `proposals/value-ownership.md` and `proposals/lifetimes-and-provenance.md` document tradeoffs between move/borrow defaults, ASAP destruction, and lifecycle methods.
- `**def` vs `fn` split:** Mojo uses `def` for dynamic and `fn` for strict. Sifr does not need this split since all code is strictly typed.

## Key Files to Reference During Implementation

- **Ruff parser:** `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_python_parser/`
- **Ruff AST:** `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_python_ast/src/nodes.rs`
- **ty type system:** `/Users/yaseralnajjar/work/sifr/ty/ruff/crates/ty_python_semantic/src/types.rs`
- **TypeScript checker architecture:** `/Users/yaseralnajjar/work/sifr/TypeScript.wiki/`
- **Mojo ownership design:** `/Users/yaseralnajjar/work/sifr/modular/mojo/proposals/value-ownership.md`
- **Mojo lifetimes design:** `/Users/yaseralnajjar/work/sifr/modular/mojo/proposals/lifetimes-and-provenance.md`

