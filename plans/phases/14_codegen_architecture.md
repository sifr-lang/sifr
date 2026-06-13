# Codegen Architecture

**Why now:** The type system is complete (Phase 13), the stdlib is generic, and the language surface is stable. But the codegen — the single largest crate at 9,805 lines — is entirely string-based: every Rust construct is emitted via `self.write("...")` and `format!()` calls with no intermediate representation. This is the root cause of several systemic problems: no compile-time validation of generated code (typos in string templates only surface when `rustc` compiles the output), manual indentation tracking, heuristic clone insertion via temporal-coupling boolean flags (`suppress_field_clone`, `in_generator_closure`, `in_display_impl`), a string-parsing dead-code eliminator (`filter_rust_code_to_needed` that regex-parses generated Rust to build a dependency graph), and 34 Clippy suppressions that are direct consequences of the string-template approach. Every subsequent phase (async runtime, typed serde, web framework, FFI) will add hundreds of new intrinsics and codegen patterns. If those are added to the current string-template system, the codebase becomes unmaintainable. Introducing a structured Rust IR now means all future codegen is built on a sound foundation.

**Why this ordering within the phase:** Each milestone builds on the previous one. The IR type definitions must come first because everything else depends on them. The renderer comes second because it's the bridge between the new IR and the existing string output — once it exists, migration can begin. The preamble migration comes third because it's the safest, most self-contained conversion (static code, no user-input dependency) and proves the IR+renderer pipeline end-to-end. Statement and expression migration comes fourth because it's the bulk of the work and requires the IR, renderer, and preamble to already be proven. The intrinsic migration comes fifth because intrinsics are the most self-contained match arms and benefit from all prior infrastructure. Structural passes come last because they are new capabilities that require the full IR to be in place.

---

## milestone_rust_ir_types: Rust IR Type Definitions

status: done

**Goal:** Define a purpose-built intermediate representation for the subset of Rust that Sifr actually generates. This is NOT a general-purpose Rust AST (no `syn`, no `quote`, no `proc-macro2`). It is a ~300-line set of enum/struct types covering the ~50 distinct Rust constructs the codegen emits: struct, enum, impl, fn, let, if, match, for, while, loop, return, break, continue, closures, method calls, field access, indexing, binary/unary ops, macros, attributes, and type references. The IR includes a `RawCode(String)` escape hatch on every node type to enable incremental migration — unconverted codegen paths can emit raw strings through the IR without blocking the migration.

**Depends on:** milestone_stdlib_generic_rewrite (Phase 13 must be complete — the language surface must be stable before restructuring the codegen)

### New File: `crates/sifr_codegen/src/rust_ir.rs`

#### Top-level items

```rust
pub struct RustFile {
    pub items: Vec<RustItem>,
}

pub enum RustItem {
    Use(Vec<String>),
    Struct {
        name: String,
        visibility: Visibility,
        derives: Vec<String>,
        fields: Vec<(String, RustType)>,
    },
    TupleStruct {
        name: String,
        visibility: Visibility,
        derives: Vec<String>,
        inner: RustType,
    },
    Enum {
        name: String,
        visibility: Visibility,
        derives: Vec<String>,
        repr: Option<String>,
        variants: Vec<RustEnumVariant>,
    },
    Trait {
        name: String,
        visibility: Visibility,
        supertraits: Vec<String>,
        methods: Vec<RustItem>,
    },
    Impl {
        target: String,
        type_params: Vec<RustTypeParam>,
        trait_: Option<String>,
        items: Vec<RustItem>,
    },
    Fn {
        name: String,
        visibility: Visibility,
        type_params: Vec<RustTypeParam>,
        params: Vec<RustParam>,
        ret: Option<RustType>,
        body: Vec<RustStmt>,
        is_async: bool,
    },
    Const {
        name: String,
        visibility: Visibility,
        ty: RustType,
        value: RustExpr,
    },
    Static {
        name: String,
        visibility: Visibility,
        ty: RustType,
        value: RustExpr,
    },
    Attr(String),
    RawCode(String),
}
```

#### Statements

```rust
pub enum RustStmt {
    Let {
        mutable: bool,
        name: String,
        ty: Option<RustType>,
        value: RustExpr,
    },
    Assign {
        target: RustExpr,
        value: RustExpr,
    },
    AugAssign {
        target: RustExpr,
        op: String,
        value: RustExpr,
    },
    Expr(RustExpr),
    Return(Option<RustExpr>),
    If {
        cond: RustExpr,
        then_body: Vec<RustStmt>,
        else_body: Option<Vec<RustStmt>>,
    },
    IfLet {
        pattern: String,
        expr: RustExpr,
        then_body: Vec<RustStmt>,
        else_body: Option<Vec<RustStmt>>,
    },
    Match {
        expr: RustExpr,
        arms: Vec<RustMatchArm>,
    },
    For {
        var: String,
        iter: RustExpr,
        body: Vec<RustStmt>,
    },
    While {
        cond: RustExpr,
        body: Vec<RustStmt>,
    },
    Loop {
        body: Vec<RustStmt>,
    },
    Break,
    Continue,
    Block(Vec<RustStmt>),
    RawCode(String),
}
```

#### Expressions

```rust
pub enum RustExpr {
    Literal(RustLiteral),
    Ident(String),
    Path(Vec<String>),
    MethodCall {
        receiver: Box<RustExpr>,
        method: String,
        args: Vec<RustExpr>,
    },
    FnCall {
        func: Box<RustExpr>,
        args: Vec<RustExpr>,
    },
    MacroCall {
        name: String,
        args: Vec<RustExpr>,
    },
    FormatMacro {
        name: String,
        format_str: String,
        args: Vec<RustExpr>,
    },
    BinOp {
        left: Box<RustExpr>,
        op: String,
        right: Box<RustExpr>,
    },
    UnaryOp {
        op: String,
        operand: Box<RustExpr>,
    },
    Field {
        expr: Box<RustExpr>,
        field: String,
    },
    Index {
        expr: Box<RustExpr>,
        index: Box<RustExpr>,
    },
    Ref {
        mutable: bool,
        expr: Box<RustExpr>,
    },
    Deref(Box<RustExpr>),
    Clone(Box<RustExpr>),
    Cast {
        expr: Box<RustExpr>,
        ty: RustType,
    },
    Block {
        stmts: Vec<RustStmt>,
        expr: Option<Box<RustExpr>>,
    },
    If {
        cond: Box<RustExpr>,
        then_expr: Box<RustExpr>,
        else_expr: Option<Box<RustExpr>>,
    },
    Match {
        expr: Box<RustExpr>,
        arms: Vec<RustMatchArm>,
    },
    Closure {
        params: Vec<RustParam>,
        body: Box<RustExpr>,
        is_move: bool,
    },
    ClosureBlock {
        params: Vec<RustParam>,
        body: Vec<RustStmt>,
        is_move: bool,
    },
    StructInit {
        name: String,
        fields: Vec<(String, RustExpr)>,
    },
    Tuple(Vec<RustExpr>),
    Vec(Vec<RustExpr>),
    Try(Box<RustExpr>),
    Await(Box<RustExpr>),
    Range {
        start: Box<RustExpr>,
        end: Box<RustExpr>,
    },
    RawCode(String),
}
```

#### Supporting types

```rust
pub enum RustLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Unit,
    None,
}

pub enum RustType {
    I64,
    F64,
    Bool,
    String_,
    Unit,
    Vec(Box<RustType>),
    HashMap(Box<RustType>, Box<RustType>),
    HashSet(Box<RustType>),
    VecDeque(Box<RustType>),
    Option(Box<RustType>),
    Result(Box<RustType>, Box<RustType>),
    Tuple(Vec<RustType>),
    Ref {
        mutable: bool,
        inner: Box<RustType>,
    },
    Named(String),
    Generic {
        base: String,
        params: Vec<RustType>,
    },
    Fn {
        params: Vec<RustType>,
        ret: Box<RustType>,
    },
    DynTrait(String),
    Impl(String),
    RawCode(String),
}

pub enum RustParam {
    SelfParam { mutable: bool },
    Named { name: String, ty: RustType },
}

pub struct RustMatchArm {
    pub pattern: String,
    pub bindings: Vec<String>,
    pub guard: Option<RustExpr>,
    pub body: Vec<RustStmt>,
}

pub struct RustEnumVariant {
    pub name: String,
    pub fields: Vec<(String, RustType)>,
    pub value: Option<RustExpr>,
}

pub struct RustTypeParam {
    pub name: String,
    pub bounds: Vec<String>,
}

pub enum Visibility {
    Private,
    Pub,
}
```

### Design Decisions

1. **`RawCode(String)` on every node type.** This is the critical migration enabler. Any codegen path that hasn't been converted yet can emit `RawCode("the old string output")`. The renderer passes it through verbatim. This means every intermediate state compiles and works correctly — there is no big-bang rewrite.

2. **Match arm patterns are `String`, not a structured pattern type.** Rust match patterns are complex (nested destructuring, `ref`, `@` bindings, range patterns). Modeling them fully would add 100+ lines of types for marginal benefit. Since Sifr generates a limited set of patterns, keeping them as strings is pragmatic. The renderer emits them verbatim. However, each `RustMatchArm` carries a `bindings: Vec<String>` field that lists the variable names introduced by the pattern. This enables the DCE pass to know which identifiers are defined (not just referenced) in match arms, without requiring a full pattern AST.

3. **No `syn`/`quote` dependency.** These crates are designed for proc-macros and model all of Rust syntax (~50k lines). Sifr only emits ~50 distinct constructs. A purpose-built IR of ~300 lines is the right size.

4. **`FormatMacro` is separate from `MacroCall`.** Format macros (`format!`, `write!`, `println!`, `panic!`) have a format string + args pattern that deserves its own node for clarity and potential future optimization (e.g., folding adjacent string literals).

5. **`RustParam` is an enum, not a struct with boolean flags.** `SelfParam { mutable }` handles `&self`/`&mut self` receivers. `Named { name, ty }` handles regular parameters where reference-ness is expressed through `RustType::Ref` on the type itself (e.g., `&str` is `Named { name: "s", ty: RustType::Ref { mutable: false, inner: Box::new(RustType::String_) } }`). This avoids invalid states where boolean flags (`is_ref`, `is_mut_ref`) could contradict the type.

### Integration Point

The new module is added to `crates/sifr_codegen/src/lib.rs` as `mod rust_ir;` and `pub use rust_ir::*;`. No existing code changes in this milestone — the types are defined but not yet used by the emitter.

### Definition of Done (milestone_rust_ir_types)

- New file `crates/sifr_codegen/src/rust_ir.rs` with all types defined above
- All types derive `Debug`, `Clone`
- `mod rust_ir; pub use rust_ir::*;` added to `lib.rs`
- No existing codegen behavior changes — this milestone only adds types
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- Unit tests in `rust_ir.rs` that construct representative IR nodes for: a simple struct with derives, an impl block with a method, a function with let/if/return, a match expression, a closure, a `RawCode` fallback

---

## milestone_rust_ir_renderer: Rust IR Pretty-Printer

status: done

**Goal:** Write a renderer that takes `RustFile` (or any IR node) and produces correctly formatted, indented Rust source code as a `String`. This is the single point where indentation logic lives — the codegen never thinks about whitespace again. The renderer must produce output that is **semantically identical** to what the current string-based codegen produces — the generated Rust must compile and produce the same runtime behavior. Whitespace and formatting differences are acceptable (the IR renderer may produce cleaner formatting than the manual `push_str` chains). The verification contract is: **same compilation success, same runtime output** — not byte-for-byte textual identity.

**Depends on:** milestone_rust_ir_types (the IR types must exist before the renderer can consume them)

### New File: `crates/sifr_codegen/src/render.rs`

#### Core Renderer

```rust
pub struct Renderer {
    output: String,
    indent: usize,
}

impl Renderer {
    pub fn new() -> Self { ... }
    pub fn render_file(&mut self, file: &RustFile) -> String { ... }
    pub fn render_item(&mut self, item: &RustItem) { ... }
    pub fn render_stmt(&mut self, stmt: &RustStmt) { ... }
    pub fn render_expr(&mut self, expr: &RustExpr) { ... }
    pub fn render_type(&mut self, ty: &RustType) { ... }
    fn write(&mut self, s: &str) { ... }
    fn writeln(&mut self, s: &str) { ... }
    fn write_indent(&mut self) { ... }
    fn indent(&mut self) { ... }
    fn dedent(&mut self) { ... }
}
```

#### Rendering Rules

Each IR node type has exactly one rendering path:

- **`RustItem::Struct`**: Emit `#[derive(...)]`, optional `pub`, `struct Name { fields }` with one field per line, indented.
- **`RustItem::Enum`**: Emit `#[derive(...)]`, optional `#[repr(...)]`, `enum Name { variants }`.
- **`RustItem::Impl`**: Emit `impl<params> Trait for Target { items }`.
- **`RustItem::Fn`**: Emit `fn name<params>(args) -> ret { body }` with proper indentation.
- **`RustStmt::Let`**: Emit `let [mut] name[: ty] = expr;`.
- **`RustStmt::If`**: Emit `if cond { body } [else { body }]` with indentation.
- **`RustStmt::Match`**: Emit `match expr { arms }` with each arm indented.
- **`RustStmt::For`**: Emit `for var in iter { body }`.
- **`RustExpr::MethodCall`**: Emit `receiver.method(args)`.
- **`RustExpr::FnCall`**: Emit `func(args)`.
- **`RustExpr::FormatMacro`**: Emit `name!("fmt", args)`.
- **`RustExpr::BinOp`**: Emit `left op right` (with parentheses when needed for precedence).
- **`RustExpr::Clone`**: Emit `expr.clone()`.
- **`RustExpr::Try`**: Emit `expr?`.
- **`RustExpr::RawCode`**: Emit the string verbatim (no indentation adjustment).
- **`RustStmt::RawCode`**: Emit the string verbatim with current indentation prefix.
- **`RustItem::RawCode`**: Emit the string verbatim at top level.

#### `RawCode` Handling

The `RawCode` variant is the migration bridge. The renderer emits it verbatim:
- `RustExpr::RawCode(s)` → writes `s` directly (inline, no newline)
- `RustStmt::RawCode(s)` → writes indent + `s` + newline
- `RustItem::RawCode(s)` → writes `s` directly (may contain multiple lines, already formatted)

This means during migration, unconverted codegen can capture its old string output into `RawCode` and the renderer passes it through unchanged.

### Convenience Functions

```rust
pub fn render_items(items: &[RustItem]) -> String { ... }
pub fn render_stmts(stmts: &[RustStmt]) -> String { ... }
pub fn render_expr(expr: &RustExpr) -> String { ... }
```

These are standalone functions that create a `Renderer`, render the input, and return the string. Used for unit testing and for incremental migration (where a single function's output is rendered via IR while the rest uses the old path).

### Testing Strategy

The renderer must be tested against known-good Rust output. For each IR node type:

1. Construct the IR node programmatically
2. Render it to a string
3. Assert the string matches the expected Rust source (snapshot test via `insta`)

Example test cases:
- A struct with `#[derive(Debug, Clone, PartialEq)]` and 3 fields
- An `impl` block with 2 methods (one `&self`, one `&mut self`)
- A function with `let`, `if`/`else`, `match`, and `return`
- A `for` loop with a method call chain inside
- A `RawCode` item that passes through verbatim
- Nested indentation: function inside impl inside module
- A `FormatMacro` with multiple arguments

### Definition of Done (milestone_rust_ir_renderer)

- New file `crates/sifr_codegen/src/render.rs` with the `Renderer` struct and all rendering methods
- `mod render; pub use render::*;` added to `lib.rs`
- Every IR node type has a rendering path (no unhandled variants)
- `RawCode` variants pass through verbatim at all levels (item, stmt, expr)
- Indentation is correct for all nesting levels
- No existing codegen behavior changes — this milestone only adds the renderer
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- Snapshot tests (via `insta`) for each IR node type covering: struct, enum, trait, impl, fn, let, if, match, for, while, loop, closure, method call, format macro, raw code passthrough
- Unit test proving that `render_items(&[RustItem::RawCode(s)])` returns `s` unchanged

---

## milestone_codegen_preamble_migration: Preamble Migration to IR

status: done

**Goal:** Migrate the preamble generation (error types, file handle infrastructure, logging globals, import collection) from string templates to structured IR. This is the safest first migration target because the preamble is static Rust code — it does not depend on user input or HIR traversal. It proves the IR+renderer pipeline end-to-end on real output and eliminates the worst string templates in the codebase (the `FileHandle` methods at 500-650 characters per `push_str` call).

**Depends on:** milestone_rust_ir_renderer (the renderer must exist to serialize IR nodes to strings)

### Migration Scope

#### 1. Error type definitions (lines 774-855 of current `lib.rs`)

Currently: ~80 lines of `push_str` calls building error structs, `new()` constructors, `Display` impls, and `Error` trait impls for 20+ error types.

Target: A function `fn error_type_items(name: &str, extra_fields: &[(&str, RustType)]) -> Vec<RustItem>` that returns:
- `RustItem::Struct` with `derives: ["Debug", "Clone"]`, fields `message: String` + any extra fields
- `RustItem::Impl` with a `new()` method
- `RustItem::Impl` for `std::fmt::Display` with `write!(f, "{}", self.message)`
- `RustItem::Impl` for `std::error::Error` (empty impl)

The 20+ error types are generated by calling this function with different names and extra fields:
- `IOError` → extra field `kind: String`, plus `__io_err` helper function
- `JSONDecodeError` / `TOMLDecodeError` → extra fields `line: i64`, `column: i64`
- `RegexError` → extra field `detail: String`
- All others → no extra fields

#### 2. File handle infrastructure (lines 857-895)

Currently: The `SifrFileHandle` enum, `__SIFR_FILE_HANDLES` static, and `FileHandle` struct with 10 methods are emitted as single-line `push_str` calls (500-650 chars each).

Target: Structured IR items:
- `RustItem::Enum` for `SifrFileHandle` with 4 variants
- `RustItem::Static` for `__SIFR_FILE_HANDLES`
- `RustItem::Struct` for `FileHandle` with `_handle: i64` and `_mode: String`
- `RustItem::Impl` for `FileHandle` with 10 methods, each built as `RustItem::Fn` with proper `RustStmt` bodies

The `FileHandle` methods are the single biggest win: each 500+ character string template becomes a structured function body with `let`, `match`, method calls, and error returns — all type-checked by the Rust compiler at Sifr build time.

#### 3. Logging infrastructure (lines 897-907)

Currently: Conditional `push_str` for `__SIFR_GLOBAL_LOG_LEVEL` static.

Target: `RustItem::Static` with `RustType::Named("std::sync::LazyLock<Mutex<i64>>")`.

#### 4. Import collection (lines 754-768)

Current bootstrap notes referred to per-feature boolean import flags such as `needs_hashmap`, `needs_hashset`, `needs_vecdeque`, and the historical `needs_bigint`. The canonical integer model routes exact integer support through `sifr_runtime::SifrInt`; future import/dependency collection should operate on structured runtime needs rather than a public `bigint` flag.

Target: A `Vec<RustItem::Use>` collected during emission, deduplicated, and prepended to the file. The boolean flags remain for now (they're set during HIR traversal) but the emission path uses IR.

#### 5. `Type::rust_type()` → `RustType` mapping

Currently: `Type::rust_type()` in `sifr_type_system/src/types.rs` returns a `String` (e.g., `"Vec<i64>"`, `"HashMap<String, f64>"`). The codegen interpolates this string directly into templates.

Target: Add a helper function `fn sifr_type_to_rust_type(ty: &Type) -> RustType` in `sifr_codegen` that maps Sifr types to structured `RustType` nodes. This function lives in the codegen crate (not the type system crate) because `RustType` is a codegen concept. The existing `Type::rust_type() -> String` method remains unchanged for backward compatibility — the new function is used by the IR-building `lower_*` methods, while the old method continues to serve any remaining string-based paths during migration.

#### 6. `is_builtin_error_referenced` elimination

Currently: The function at line 74 scans the generated Rust source code as a string to determine which error types are actually used, using word-boundary matching. This is a consequence of string-based codegen — you can't query the output structurally.

Target: During IR-based preamble generation, track which error types are referenced by the user's code (already known from HIR analysis). Only emit error type items for referenced types. The string-scanning function becomes unnecessary.

### Integration Approach

The `generate_rust_with_stdlib` function (line 547) currently builds the preamble as a `String` via `push_str` chains, then concatenates with `emitter.output`. After this milestone:

1. Build the preamble as `Vec<RustItem>` using the new IR-based functions
2. Render the preamble items to a `String` using the renderer
3. Concatenate with `emitter.output` (unchanged) as before

The user-code codegen (`emitter.output`) is NOT changed in this milestone. Only the preamble switches to IR.

### Verification

The generated Rust output must be functionally identical before and after migration. Strategy:

1. For each E2E test that uses error types or file handles, capture the generated Rust source before migration
2. After migration, capture the generated Rust source again
3. Both must compile and produce identical runtime behavior
4. Whitespace differences are acceptable (the IR renderer may format differently than the manual `push_str` chains) as long as the code compiles and behaves identically

### Differential Testing Harness

Starting with this milestone and continuing through milestones 4-5, use a differential testing approach: for the full E2E test corpus, run both the old codegen path and the new IR path, compile both outputs, and compare runtime results. This catches semantic regressions that unit tests might miss. The harness is a test-time utility, not a production feature — it can be a `#[cfg(test)]` function that generates Rust via both paths and asserts identical output/behavior.

### Definition of Done (milestone_codegen_preamble_migration)

- All error type definitions emitted via IR (`error_type_items` function)
- `FileHandle` struct and all 10 methods emitted via structured IR (no more 500+ char string templates)
- `SifrFileHandle` enum and `__SIFR_FILE_HANDLES` static emitted via IR
- Logging infrastructure emitted via IR
- Import collection uses `RustItem::Use` instead of direct string concatenation
- `is_builtin_error_referenced` string-scanning function eliminated or reduced to a thin compatibility shim
- `sifr_type_to_rust_type` helper function maps all Sifr `Type` variants to `RustType` nodes
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- At least 5 Clippy suppressions from the file header can be removed (specifically `format_push_string` which is the most directly caused by string templates)
- Generated Rust code compiles and produces identical runtime behavior for all test cases

---

## milestone_codegen_stmt_expr_migration: Statement and Expression Migration

status: done

**Goal:** Migrate the core codegen functions — `emit_stmt` (line 3047, ~1,400 lines), `emit_expr` (line 5509, ~1,400 lines), `emit_method_call` (line 4888, ~570 lines), and `emit_function`/`emit_class` — from writing directly to `self.output` to building IR nodes that are rendered at the end. This is the bulk of the migration and converts the codegen from an imperative string-writing model to a functional IR-building model.

**Depends on:** milestone_codegen_preamble_migration (the preamble migration proves the IR+renderer pipeline works end-to-end on real output)

### `RustEmitter` State Decomposition

The current `RustEmitter` is a god-object with 33 mutable fields mixing output accumulation, contextual flags, and type metadata. As part of this migration, decompose its state into focused contexts:

- **`CodegenContext`**: Carries immutable compilation context (type information, stdlib symbols, compiler options). Passed by reference to all `lower_*` methods.
- **`ScopeContext`**: Carries mutable scope-local state (current function return type, whether inside a generator/display impl/loop-with-else). Passed as a parameter, not stored as mutable struct fields. This replaces the temporal coupling flags.
- **`RustEmitter`** retains only the output accumulation (`output: String`, `indent: usize`) and delegates to `lower_*` methods that accept contexts explicitly.

This decomposition happens incrementally alongside the `lower_*` migration — each converted codegen path receives explicit context parameters instead of reading mutable struct fields.

### Design Principle: `lower_*` Methods Return `Result`

All new `lower_*` methods return `Result<RustExpr, CodegenError>` / `Result<Vec<RustStmt>, CodegenError>` instead of panicking on unexpected input. Define a `CodegenError` type in `sifr_codegen` for structured error reporting. `CodegenError` must carry the source `Span` (line/column) from the `HirExpr` or `HirStmt` being lowered, so the driver can report the exact user source location when codegen fails on an unsupported construct. This is adopted incrementally — the initial `lower_*` implementations may use `.expect()` for cases that are genuinely unreachable, but new code defaults to `Result` propagation.

### Migration Strategy

The migration uses a **dual-path approach** during the transition:

1. **New `lower_*` methods** return IR nodes: `fn lower_stmt(&self, stmt: &HirStmt) -> Result<Vec<RustStmt>, CodegenError>`, `fn lower_expr(&self, expr: &HirExpr) -> Result<RustExpr, CodegenError>`, etc.
2. **Old `emit_*` methods** become thin wrappers that call the new `lower_*` methods and render the result into `self.output`.
3. **Unconverted match arms** in the new `lower_*` methods return `Ok(RawCode(...))` by capturing the old string output via a temporary buffer.

This means:
- Every intermediate state compiles and works correctly
- Each match arm can be converted independently
- Tests pass after every individual conversion
- The old `emit_*` methods are deleted only after all arms are converted

### Conversion Order (within this milestone)

Convert in order of decreasing isolation (most self-contained first):

#### Phase A: Leaf expressions (no recursion into sub-expressions)

1. `HirExpr::IntLiteral` → `RustExpr::Literal(RustLiteral::Int(v))`
2. `HirExpr::FloatLiteral` → `RustExpr::Literal(RustLiteral::Float(v))`
3. `HirExpr::StringLiteral` → `RustExpr::FormatMacro` or `RustExpr::Literal`
4. `HirExpr::BoolLiteral` → `RustExpr::Literal(RustLiteral::Bool(v))`
5. `HirExpr::NoneLiteral` → `RustExpr::Literal(RustLiteral::None)`
6. `HirExpr::Name` → `RustExpr::Ident(name)` or stdlib constant lookup
7. `HirExpr::EnumVariant` → `RustExpr::Path(vec![enum_name, variant])` (e.g., `Color::RED`)

#### Phase B: Compound expressions (recurse into sub-expressions)

8. `HirExpr::BinOp` → `RustExpr::BinOp` (with special cases for string concat, list concat, floor div, power)
9. `HirExpr::UnaryOp` → `RustExpr::UnaryOp`
10. `HirExpr::Compare` → `RustExpr::BinOp` chains
11. `HirExpr::BoolOp` → `RustExpr::BinOp` chains with `&&`/`||`
12. `HirExpr::ContainsOp` → `RustExpr::MethodCall` (`.contains()`) on collection
13. `HirExpr::IfExpr` → `RustExpr::If`
14. `HirExpr::Index` → `RustExpr::MethodCall` (`.get()`) or `RustExpr::Index`
15. `HirExpr::FieldAccess` → `RustExpr::Field`
16. `HirExpr::MethodCall` → `RustExpr::MethodCall` (delegates to `lower_method_call`)
17. `HirExpr::Call` → `RustExpr::FnCall` (with intrinsic dispatch)
18. `HirExpr::FString` → `RustExpr::FormatMacro`
19. `HirExpr::ListLiteral` / `SetLiteral` / `DictLiteral` / `TupleLiteral` → `RustExpr::Vec` / macro calls / `RustExpr::Tuple`
20. `HirExpr::RangeLiteral` → `RustExpr::Range` (with step handling via `.step_by()`)
21. `HirExpr::Lambda` → `RustExpr::Closure`
22. `HirExpr::ListComp` / `DictComp` / `SetComp` → iterator chain expressions
23. `HirExpr::GeneratorExpr` → lazy iterator chain (similar to comprehensions but without `.collect()`)
24. `HirExpr::ConstructorCall` → `RustExpr::StructInit` or `RustExpr::FnCall`
25. `HirExpr::SuperCall` → `RustExpr::FnCall` to `ParentType::new(args)`
26. `HirExpr::WalrusExpr` → `RustExpr::Block` with let-binding and trailing expression
27. `HirExpr::QuestionMark` → `RustExpr::Try`
28. `HirExpr::OkWrap` / `ErrWrap` → `RustExpr::FnCall` wrapping
29. `HirExpr::Slice` → `lower_list_slice` / `lower_string_slice`
30. `HirExpr::Match` → `RustExpr::Match`

#### Phase C: Statements

31. `HirStmt::Let` → `RustStmt::Let`
32. `HirStmt::Assign` → `RustStmt::Assign`
33. `HirStmt::AugAssign` → `RustStmt::AugAssign` or method call
34. `HirStmt::Return` → `RustStmt::Return` (with context-dependent wrapping for Display impls, generators, try blocks)
35. `HirStmt::Expr` → `RustStmt::Expr`
36. `HirStmt::Pass` → empty `Vec<RustStmt>` (no-op, produces no output)
37. `HirStmt::If` → `RustStmt::If` (**semantic transformation**: `elif_clauses` become nested `else { if ... }` chains — see note below)
38. `HirStmt::While` → `RustStmt::While` (**semantic transformation**: `else_body` requires flag variable — see note below)
39. `HirStmt::For` → `RustStmt::For` (with type-driven iterator adaptation — see note below; **semantic transformation** for `else_body`)
40. `HirStmt::Break` / `Continue` → `RustStmt::Break` / `RustStmt::Continue`
41. `HirStmt::Match` → `RustStmt::Match`
42. `HirStmt::TryExcept` → `RustStmt::Match` on Result
43. `HirStmt::Raise` → `RustStmt::Return` with `Err(...)`
44. `HirStmt::Assert` → `RustStmt::If` + `RustStmt::RawCode("panic!(...)")`
45. `HirStmt::FieldAssign` → `RustStmt::Assign` targeting `RustExpr::Field`
46. `HirStmt::SubscriptAssign` → `RustStmt::Expr` with method call (e.g., `.insert()`, index assign)
47. `HirStmt::NestedSubscriptAssign` → `RustStmt::Expr` with chained index access + assignment
48. `HirStmt::SubscriptAugAssign` → index access + augmented assignment
49. `HirStmt::AttributeAugAssign` → field access + augmented assignment
50. `HirStmt::AttributeSubscriptAssign` → field access + index assignment
51. `HirStmt::TupleUnpack` → multiple `RustStmt::Let` bindings from destructured tuple
52. `HirStmt::StarUnpack` → `RustStmt::Let` bindings with slice operations for `*rest`
53. `HirStmt::Delete` → `RustStmt::Expr` with `.remove()` method call
54. `HirStmt::Yield` → `RustStmt::Expr` with generator yield (context-dependent on generator type)
55. `HirStmt::With` → `RustStmt::Block` with resource acquisition and drop semantics
56. `HirStmt::NestedFunction` → `RustStmt::Expr` containing a closure `let` binding (or `RustItem::Fn` if non-capturing)

#### Phase D: Top-level items

57. `emit_function` → builds `RustItem::Fn`
58. `emit_class` → builds `RustItem::Struct` + `RustItem::Impl`
59. `emit_protocol_trait` → builds `RustItem::Trait`
60. `emit_enum_class` → builds `RustItem::Enum` + `RustItem::Impl`
61. `emit_operator_impls` → builds `RustItem::Impl` for trait impls
62. `collect_union_types` + `generate_enum_definitions` → builds `Vec<RustItem::Enum>` for union type enums (these are currently a pre-pass that scans the module for union types and prepends enum definitions to the output; after migration they become `RustItem::Enum` nodes inserted into `RustFile.items`)
63. `emit_module` → builds `RustFile` (orchestrates all of the above, including the union enum pre-pass)
64. `generate_rust_test` → builds `RustFile` in test mode (sets `Visibility::Pub` appropriately, adds `#[test]` attributes via `RustItem::Attr`). After migration, this entry point shares the same `lower_*` pipeline as `generate_rust_with_stdlib`, differing only in the `CodegenContext` configuration.
65. `generate_rust_for_modules` → builds one `RustFile` per module for multi-file projects. Each module uses `Visibility::Pub` for non-main modules. After migration, this creates a `CodegenContext` per module and calls the shared `lower_module` pipeline.

### Temporal Coupling Flags

As each codegen path is converted to IR, the temporal coupling flags can be eliminated:

- **`suppress_field_clone`**: Instead of setting a flag before a method call and clearing it after, the `lower_method_call` function inspects the receiver expression and decides whether to wrap it in `RustExpr::Clone` or not. The decision is local to the function, not a global flag.
- **`in_generator_closure`**: Instead of a flag, `lower_stmt` for `Return` checks whether the enclosing function context is a generator and wraps accordingly. Pass the context as a parameter, not a mutable field.
- **`in_display_impl`**: Same approach — pass the context as a parameter to `lower_stmt`.
- **`in_loop_with_else`**: Pass loop context as a parameter. Replaced by the `for/else` semantic transformation (see below).
- **`pub_mode`**: Currently checked at ~15 locations to decide whether to emit `pub`. With the IR, this maps directly to `visibility: Visibility::Pub` when building IR items. The flag becomes unnecessary — `CodegenContext` carries whether the current module is a non-main module, and item-building code sets `Visibility` accordingly.

These flags are NOT removed in this milestone if doing so would risk regressions. They are removed only when the corresponding codegen path is fully converted and tested. Some flags may persist as parameters to `lower_*` methods rather than mutable struct fields — this is still an improvement (explicit parameter vs hidden mutable state).

### Semantic Transformations

Several HIR→IR lowerings are not simple syntax mappings — they require non-trivial semantic transformations. These are where bugs will hide during migration and deserve explicit attention:

**`elif` chains → nested `if/else`:** The HIR's `If` statement has `elif_clauses: Vec<(HirExpr, Vec<HirStmt>)>` — a flat list of elif branches. Rust has no `elif`. The lowering must convert this to nested `else { if ... }` chains: the first elif becomes the `else_body` containing a new `RustStmt::If`, whose `else_body` contains the next elif, and so on. The final `else_body` (if any) goes in the innermost `else`. Getting the nesting wrong produces incorrect control flow.

**`for/else` and `while/else`:** The HIR's `For` and `While` have `else_body: Option<Vec<HirStmt>>`. Rust has no `for/else` or `while/else`. The lowering must introduce a flag variable: `{ let mut __loop_completed = true; for ... { if <break condition> { __loop_completed = false; break; } } if __loop_completed { <else_body> } }`. The `lower_for` and `lower_while` methods must detect the presence of `else_body` and wrap accordingly. This replaces the current `in_loop_with_else` temporal coupling flag.

**Type-driven iterator adaptation:** The HIR's `For` has `target_ty: Type` which the codegen uses to determine the correct iterator method: `.iter()` for borrowed iteration, `.into_iter()` for owned, `.chars()` for string iteration, `.drain(..)` for consuming, etc. The `lower_for` method must have access to the HIR's type information (via `CodegenContext`) to produce the correct `iter` expression in the IR. The type information is consumed during lowering — the resulting `RustStmt::For` only has the final `iter: RustExpr`.

**`expr_to_string` elimination:** The current codegen has a hack at lines 4709-4718 that saves the output buffer, emits an expression to a fresh buffer, captures the result as a string, and restores the original buffer. This is used ~10 times for match guards and other contexts where a sub-expression's string output is needed inline. With the IR, this hack disappears naturally — `lower_expr` returns a `RustExpr` that can be placed anywhere in the IR tree. All `expr_to_string` call sites must be identified and replaced with direct `lower_expr` calls during migration.

### Module Decomposition

As part of this milestone, break up the monolithic `lib.rs` (9,805 lines) into focused modules:

- `crates/sifr_codegen/src/rust_ir.rs` — IR type definitions (already created in milestone 1)
- `crates/sifr_codegen/src/render.rs` — Renderer (already created in milestone 2)
- `crates/sifr_codegen/src/lower_expr.rs` — `lower_expr` and all expression-lowering helpers
- `crates/sifr_codegen/src/lower_stmt.rs` — `lower_stmt` and all statement-lowering helpers
- `crates/sifr_codegen/src/lower_item.rs` — `lower_function`, `lower_class`, `lower_enum_class`, `lower_protocol_trait`, `lower_module`
- `crates/sifr_codegen/src/context.rs` — `CodegenContext`, `ScopeContext`, `CodegenError` definitions
- `crates/sifr_codegen/src/preamble.rs` — Preamble generation functions (moved from milestone 3 additions in `lib.rs`)
- `crates/sifr_codegen/src/lib.rs` — Retains `RustEmitter`, `generate_rust_with_stdlib`, and module declarations; becomes a thin orchestration layer

The decomposition happens as each `lower_*` function is written — new code goes directly into the new module files. The old `emit_*` methods in `lib.rs` become thin wrappers that delegate to the new modules.

### Definition of Done (milestone_codegen_stmt_expr_migration)

- `lower_expr` handles all 30 `HirExpr` variants (some may use `RawCode` for complex cases)
- `lower_stmt` handles all 26 `HirStmt` variants (some may use `RawCode` for complex cases)
- `emit_function` and `emit_class` build IR items
- `emit_module` builds a `RustFile` and renders it
- `generate_rust_test` and `generate_rust_for_modules` use the shared `lower_module` pipeline
- Union enum generation (`collect_union_types` + `generate_enum_definitions`) produces `RustItem::Enum` nodes
- All new `lower_*` methods return `Result<_, CodegenError>` (not panicking)
- `CodegenContext` and `ScopeContext` structs are defined and used by all `lower_*` methods
- At least 80% of match arms in `lower_expr` and `lower_stmt` produce structured IR (not `RawCode`)
- The remaining `RawCode` usages are documented with `// TODO: convert to structured IR` comments
- At least 4 temporal coupling flags (`suppress_field_clone`, `in_generator_closure`, `in_display_impl`, `pub_mode`) are eliminated or converted to explicit `ScopeContext` parameters
- All `expr_to_string` call sites replaced with direct `lower_expr` calls
- Semantic transformations for `elif` chains, `for/else`, `while/else` are implemented and tested
- Module decomposition complete: `lower_expr.rs`, `lower_stmt.rs`, `lower_item.rs`, `context.rs`, `preamble.rs` exist as separate files; `lib.rs` is reduced to orchestration
- Differential test harness verifies semantic parity between old and new codegen paths across the full E2E test corpus
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- At least 10 additional Clippy suppressions from the file header can be removed
- The old `emit_stmt` and `emit_expr` methods are thin wrappers around `lower_*` + render

---

## milestone_codegen_intrinsic_migration: Intrinsic Call Migration

status: done

**Goal:** Migrate `emit_intrinsic_call` (line 6930, ~1,300 lines matching on intrinsic function name strings) and `emit_method_call` (line 4888, ~570 lines matching on type+method pairs) from string-template emission to IR construction. These are the most self-contained codegen functions — each match arm is independent and can be converted one-by-one.

**Depends on:** milestone_codegen_stmt_expr_migration (the core `lower_expr`/`lower_stmt` must be in place so intrinsic bodies can use structured IR)

### Migration Scope

#### `emit_intrinsic_call` → `lower_intrinsic_call`

The current function has ~80 match arms covering:
- `sifr.io`: `read_text`, `write_text`, `exists`, `read_lines`, `append_text`, `getcwd`, `listdir`, `mkdir`, `rmdir`, `remove_file`, `rename`, `is_file`, `is_dir`, `copy_file`, `walk_dir`, `rmdir_all`, `gettempdir`, `makedirs`
- `sifr.json`: `json_loads`, `json_dumps`
- `sifr.env`: `env_get`, `env_set`, `env_unset`, `env_keys`, `env_values`, `env_items`
- `sifr.os`: `run_command`, `get_args`
- `sifr.math`: `sqrt`, `floor`, `ceil`, `abs_val`, `log`, `cbrt`, `exp2`, `sin`, `cos`, `tan`, `pow_val`, `round_val`, `min_val`, `max_val`, `gamma`, `lgamma`, etc.
- `sifr.time`: `time_now`, `sleep`, `perf_counter`, `monotonic`
- `sifr.random`: `random_float`, `random_int`, `random_seed`
- `sifr.re`: `regex_match`, `regex_search`, `regex_findall`, `regex_sub`, `regex_split`
- `sifr.hashlib`: `sha256`, `md5`, `sha1`, `sha512`
- `sifr.base64`: `b64encode`, `b64decode`
- `sifr.crypto`: `random_bytes`, `random_hex`
- File handle intrinsics: `builtin_open`, `open_file`, `file_read`, `file_write`, `file_readline`, `file_readlines`, `file_close`, `file_read_bytes`, `file_write_bytes`

Each arm becomes a function `fn lower_intrinsic_{name}(&self, args: &[HirExpr]) -> RustExpr` that returns a structured IR expression.

Simple intrinsics (e.g., `sqrt`, `floor`, `ceil`) are one-liners:

```rust
fn lower_intrinsic_sqrt(&self, args: &[HirExpr]) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(self.lower_expr(&args[0])),
        method: "sqrt".to_string(),
        args: vec![],
    }
}
```

Complex intrinsics (e.g., `builtin_open`, `walk_dir`, `disk_usage`) produce `RustExpr::Block` with multiple statements — replacing the current 1,500+ character string templates with structured, readable IR.

#### `emit_method_call` → `lower_method_call`

The current function matches on (object type, method name) pairs for ~50 methods across `str`, `list`, `dict`, `set`, `tuple`, `deque`, `FileHandle`, etc. Each arm becomes a structured IR expression.

### Intrinsic Registry (Mandatory)

Instead of a single 1,300-line match statement, intrinsics **must** be registered in a registry at initialization time. Each registration includes:
- The intrinsic name (e.g., `"sha256"`)
- The lowering function: `fn(&CodegenContext, &[HirExpr]) -> Result<RustExpr, CodegenError>`
- The required Cargo crate dependency, if any (e.g., `Some("sha2")` for `sha256`, `None` for `sqrt`)

This:
- Makes adding new intrinsics a one-line registration instead of editing a giant match
- Enables intrinsic discovery (list all registered intrinsics)
- Reduces the size of any single function
- Is essential for maintainability as future phases (async, typed serde, web framework, FFI) will add hundreds of new intrinsics
- **Eliminates the driver's string-scanning hack** for Cargo dependency detection. Historical code scanned generated Rust for crate-specific strings such as `num_bigint::BigInt`; the canonical integer model now depends on the shared runtime crate for exact integer support. With the registry, codegen collects all required crate names into a `HashSet<String>` during lowering and returns them alongside the generated code. The driver uses this set directly — no string scanning needed. This is not a full package manager (deferred to Phase 18) but it removes the most fragile part of dependency detection.

The registry is built once at codegen initialization. Each intrinsic function lives in a domain-specific module:

- `crates/sifr_codegen/src/intrinsics/mod.rs` — Registry construction and dispatch
- `crates/sifr_codegen/src/intrinsics/io.rs` — `sifr.io` intrinsics (~18 functions)
- `crates/sifr_codegen/src/intrinsics/math.rs` — `sifr.math` intrinsics (~20 functions)
- `crates/sifr_codegen/src/intrinsics/json.rs` — `sifr.json` intrinsics
- `crates/sifr_codegen/src/intrinsics/env.rs` — `sifr.env` intrinsics
- `crates/sifr_codegen/src/intrinsics/os.rs` — `sifr.os` intrinsics
- `crates/sifr_codegen/src/intrinsics/time.rs` — `sifr.time` intrinsics
- `crates/sifr_codegen/src/intrinsics/random.rs` — `sifr.random` intrinsics
- `crates/sifr_codegen/src/intrinsics/re.rs` — `sifr.re` intrinsics
- `crates/sifr_codegen/src/intrinsics/hashlib.rs` — `sifr.hashlib` intrinsics
- `crates/sifr_codegen/src/intrinsics/base64.rs` — `sifr.base64` intrinsics
- `crates/sifr_codegen/src/intrinsics/crypto.rs` — `sifr.crypto` intrinsics
- `crates/sifr_codegen/src/intrinsics/file_handle.rs` — File handle intrinsics

Similarly, method calls are organized by type in a `methods/` directory:

- `crates/sifr_codegen/src/methods/mod.rs` — Method registry construction and dispatch
- `crates/sifr_codegen/src/methods/str_methods.rs` — String method lowering
- `crates/sifr_codegen/src/methods/list_methods.rs` — List method lowering
- `crates/sifr_codegen/src/methods/dict_methods.rs` — Dict method lowering
- `crates/sifr_codegen/src/methods/set_methods.rs` — Set method lowering
- etc.

### Definition of Done (milestone_codegen_intrinsic_migration)

- Intrinsic registry is implemented and dispatches all ~80 intrinsic functions, each declaring its required Cargo crate (if any)
- Method registry dispatches all ~50 method calls organized by receiver type
- Intrinsics are decomposed into domain-specific modules under `intrinsics/` directory
- Methods are decomposed into type-specific modules under `methods/` directory
- Codegen returns a `HashSet<String>` of required Cargo crates alongside the generated code; the driver's string-scanning dependency detection is deleted
- No intrinsic or method lowering function contains a `self.write(...)` call longer than 100 characters (the current worst case is 1,500+ characters)
- The `builtin_open` intrinsic is a readable, structured function body (not a single string literal)
- All `RawCode` usages in intrinsics and methods are eliminated (fully converted)
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- The total line count of `sifr_codegen/src/lib.rs` is reduced by at least 2,000 lines (intrinsics + methods moved to separate modules)

---

## milestone_codegen_structural_passes: Structural IR Passes

status: done

**Goal:** Now that the entire codegen produces structured IR, add optimization and validation passes that operate on the IR before rendering. These passes are impossible with string-based codegen — they require the ability to inspect and transform the output structurally. This milestone delivers the payoff of the IR migration.

**Depends on:** milestone_codegen_intrinsic_migration (the full IR must be in place before structural passes can operate on it)

### Pass 1: Import Collection

Currently: Boolean flags for collection/runtime needs are set during HIR traversal and checked during preamble generation. Historical notes named `needs_bigint`; exact integer support should instead be represented as a `sifr_runtime`/`SifrInt` runtime need.

New approach: Walk the entire `RustFile` IR tree and collect all `RustType` nodes. If any node contains `RustType::HashMap`, add `use std::collections::HashMap;`. If any contains `RustType::HashSet`, add `use std::collections::HashSet;`. Etc. The walk must visit all IR nodes recursively — items, statements, expressions, types, and nested bodies.

**`RawCode` handling:** `RawCode` nodes are opaque strings and cannot be inspected structurally. If any `RawCode` nodes remain in the IR at this point, the import collection pass falls back to the existing boolean flags for those paths. The goal is to have zero `RawCode` by the time this pass runs (see structural passes DoD), but the fallback ensures correctness during any intermediate state.

This eliminates the boolean flags and makes import collection automatic and correct — if a codegen path forgets to set a flag, the import is still collected.

### Pass 2: Dead Code Elimination (replace `filter_rust_code_to_needed`)

Currently: `filter_rust_code_to_needed` (lines 218-349) parses generated Rust source code as text, extracts named blocks via regex-like heuristics, builds a dependency graph by string-matching function names, computes a transitive closure, and emits only needed blocks.

New approach: Walk the `RustFile` IR. For each `RustItem`, extract its name. Build a dependency graph by walking the item's body and collecting all `RustExpr::Ident` and `RustExpr::Path` references. Compute the transitive closure from the set of imported names. Filter the `RustFile.items` to only include needed items.

**`RawCode` gate:** This pass requires zero `RawCode` nodes in the IR to be fully correct. `RawCode` nodes are opaque — the pass cannot extract identifiers or references from them, which means the dependency graph would be incomplete. The prerequisite for enabling this pass is that all core codegen paths (milestones 3-5) have eliminated `RawCode`. If any `RawCode` remains in the stdlib preamble, the DCE pass must conservatively mark those items as always-needed (roots of the dependency graph).

**Scope:** DCE operates on the full `RustFile` — both user code items and stdlib preamble items. The entry points (roots) are the user's `main` function and any `#[test]` functions.

This is structurally correct (no false positives from substring matches), handles all edge cases (nested references, type references, trait impl references), and is ~50 lines of code instead of ~130 lines of string parsing.

### Pass 3: Clone Optimization (Conservative)

Currently: `.clone()` is inserted heuristically at 20+ locations based on contextual flags. Some clones are unnecessary (e.g., cloning a `Copy` type, cloning a literal).

New approach: Walk the IR and identify `RustExpr::Clone(inner)` nodes where the clone is **trivially provable** as unnecessary:
- `inner` is a literal (no clone needed — literals are always fresh values)
- `inner` is a `Copy` type based on the Sifr type information (no clone needed — `i64`, `f64`, `bool`, `char`)

**Explicitly out of scope for this phase:** "Used exactly once" analysis. Determining whether a value is used exactly once requires full dataflow/ownership analysis across the enclosing scope, which is a complex problem that belongs in `sifr_lowering` as a proper ownership analysis pass (a future phase concern). This phase only removes clones that are provably unnecessary from local type information alone.

Remove unnecessary clones. This is a conservative pass — it only removes clones that are provably unnecessary, never adds them.

### Pass 4: IR Validation

Walk the IR and check for structural issues that would cause `rustc` errors:
- Unbalanced braces (impossible with IR, but verify `RawCode` nodes are balanced)
- Empty function bodies (must have at least `()` or a statement)
- Duplicate field names in struct definitions
- `return` outside of a function body

This catches codegen bugs at Sifr compile time rather than at Rust compile time.

### Definition of Done (milestone_codegen_structural_passes)

- **`RawCode`-zero gate met:** Zero `RawCode` nodes in all core codegen paths (user code expressions, statements, items, intrinsics, methods). The only acceptable `RawCode` remaining is in the stdlib preamble if any edge cases resist conversion — these must be explicitly documented and counted (target: zero, hard maximum: 5)
- Import collection pass eliminates ad hoc collection/runtime boolean flags and replaces the historical `needs_bigint` path with structured `sifr_runtime` dependency tracking for exact integers.
- Dead code elimination pass replaces `filter_rust_code_to_needed` (the old string-parsing function is deleted)
- Clone optimization pass removes clones on literals and `Copy` types (conservative scope — no ownership analysis)
- IR validation pass catches at least 3 categories of structural issues
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- The `parse_rust_blocks`, `extract_top_level_item_name`, and `count_braces` helper functions are deleted
- At least 20 of the 34 Clippy suppressions in the file header can be removed
- Binary size of generated programs does not increase (clone optimization should decrease it slightly)
- Stdlib code goes through the same IR pipeline as user code — the stdlib preamble is built as `Vec<RustItem>` and rendered, not special-cased as raw strings

---

## Explicit Scope Exclusions

The following are **not** addressed in this phase:

- **`Cargo.toml` structured representation.** The generated Rust code's `Cargo.toml` is currently built by string concatenation in the driver. Migrating this to a structured data model is deferred to the package management phase (Phase 18). However, the driver's fragile string-scanning hack for detecting which crates are needed **is** eliminated in this phase — the intrinsic registry declares crate dependencies as metadata, and the codegen returns a `HashSet<String>` of required crates alongside the generated code.

- **HIR-level ownership analysis.** The root cause of many unnecessary `.clone()` calls is that the HIR does not track ownership/borrowing semantics. A proper ownership analysis pass in `sifr_lowering` would allow the codegen to emit moves instead of clones in many cases. This is a future phase concern — this phase only removes clones that are trivially provable as unnecessary from type information alone.

- **Full pattern AST for match arms.** Match arm patterns remain as `String` with a `bindings` annotation. A full structured pattern type is not justified by the current set of patterns Sifr generates.

## Stdlib Pipeline Clarification

Stdlib code (compiled from `stdlib.sifr`) goes through the **same IR pipeline** as user code. The stdlib is compiled by `sifr_driver` using the same `RustEmitter` → IR → Renderer path. The only special handling is the **preamble** (error types, `FileHandle`, logging globals, imports) which is generated programmatically (not from `.sifr` source) — this preamble is also built as `Vec<RustItem>` and rendered through the IR pipeline, not emitted as raw strings.

---

## Milestone Ordering

- **milestone_rust_ir_types first:** Everything depends on the IR type definitions. No existing code changes — pure addition.
- **milestone_rust_ir_renderer second:** The renderer is needed before any migration can begin. Also pure addition — no existing code changes.
- **milestone_codegen_preamble_migration third:** The safest migration target (static code, no user-input dependency). Proves the pipeline end-to-end. Eliminates the worst string templates.
- **milestone_codegen_stmt_expr_migration fourth:** The bulk of the migration. Requires the preamble to be proven. Converts the core codegen loop. Includes module decomposition of `lib.rs`.
- **milestone_codegen_intrinsic_migration fifth:** Intrinsics are the most self-contained. Requires `lower_expr` to be in place so intrinsic bodies can reference it. Includes mandatory registry pattern and domain-specific module decomposition.
- **milestone_codegen_structural_passes last:** New capabilities that require the full IR. The payoff milestone — this is where the architectural investment delivers measurable improvements. Requires `RawCode`-zero gate to be met.
