# Codegen Architecture

**Why now:** The type system is complete (Phase 13), the stdlib is generic, and the language surface is stable. But the codegen — the single largest crate at 9,805 lines — is entirely string-based: every Rust construct is emitted via `self.write("...")` and `format!()` calls with no intermediate representation. This is the root cause of several systemic problems: no compile-time validation of generated code (typos in string templates only surface when `rustc` compiles the output), manual indentation tracking, heuristic clone insertion via temporal-coupling boolean flags (`suppress_field_clone`, `in_generator_closure`, `in_display_impl`), a string-parsing dead-code eliminator (`filter_rust_code_to_needed` that regex-parses generated Rust to build a dependency graph), and 34 Clippy suppressions that are direct consequences of the string-template approach. Every subsequent phase (async runtime, typed serde, web framework, FFI) will add hundreds of new intrinsics and codegen patterns. If those are added to the current string-template system, the codebase becomes unmaintainable. Introducing a structured Rust IR now means all future codegen is built on a sound foundation.

**Why this ordering within the phase:** Each milestone builds on the previous one. The IR type definitions must come first because everything else depends on them. The renderer comes second because it's the bridge between the new IR and the existing string output — once it exists, migration can begin. The preamble migration comes third because it's the safest, most self-contained conversion (static code, no user-input dependency) and proves the IR+renderer pipeline end-to-end. Statement and expression migration comes fourth because it's the bulk of the work and requires the IR, renderer, and preamble to already be proven. The intrinsic migration comes fifth because intrinsics are the most self-contained match arms and benefit from all prior infrastructure. Structural passes come last because they are new capabilities that require the full IR to be in place.

---

## milestone_rust_ir_types: Rust IR Type Definitions

status: pending

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

pub struct RustParam {
    pub name: String,
    pub ty: RustType,
    pub is_ref: bool,
    pub is_mut_ref: bool,
    pub is_self: bool,
    pub self_mutability: Option<bool>,
}

pub struct RustMatchArm {
    pub pattern: String,
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

2. **Match arm patterns are `String`, not a structured pattern type.** Rust match patterns are complex (nested destructuring, `ref`, `@` bindings, range patterns). Modeling them fully would add 100+ lines of types for marginal benefit. Since Sifr generates a limited set of patterns, keeping them as strings is pragmatic. The renderer emits them verbatim.

3. **No `syn`/`quote` dependency.** These crates are designed for proc-macros and model all of Rust syntax (~50k lines). Sifr only emits ~50 distinct constructs. A purpose-built IR of ~300 lines is the right size.

4. **`FormatMacro` is separate from `MacroCall`.** Format macros (`format!`, `write!`, `println!`, `panic!`) have a format string + args pattern that deserves its own node for clarity and potential future optimization (e.g., folding adjacent string literals).

5. **`RustParam` handles all parameter forms.** Regular params (`name: Type`), `&self`, `&mut self`, `self` are all represented by the same struct with boolean flags. This avoids a separate enum for receiver types.

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

status: pending

**Goal:** Write a renderer that takes `RustFile` (or any IR node) and produces correctly formatted, indented Rust source code as a `String`. This is the single point where indentation logic lives — the codegen never thinks about whitespace again. The renderer must produce output that is byte-for-byte identical to what the current string-based codegen produces for any IR node that has been converted (so that snapshot tests don't break during migration).

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

status: pending

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

Currently: Boolean flags (`needs_hashmap`, `needs_hashset`, `needs_vecdeque`, `needs_bigint`) checked individually to emit `use` statements.

Target: A `Vec<RustItem::Use>` collected during emission, deduplicated, and prepended to the file. The boolean flags remain for now (they're set during HIR traversal) but the emission path uses IR.

#### 5. `is_builtin_error_referenced` elimination

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

### Definition of Done (milestone_codegen_preamble_migration)

- All error type definitions emitted via IR (`error_type_items` function)
- `FileHandle` struct and all 10 methods emitted via structured IR (no more 500+ char string templates)
- `SifrFileHandle` enum and `__SIFR_FILE_HANDLES` static emitted via IR
- Logging infrastructure emitted via IR
- Import collection uses `RustItem::Use` instead of direct string concatenation
- `is_builtin_error_referenced` string-scanning function eliminated or reduced to a thin compatibility shim
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- At least 5 Clippy suppressions from the file header can be removed (specifically `format_push_string` which is the most directly caused by string templates)
- Generated Rust code compiles and produces identical runtime behavior for all test cases

---

## milestone_codegen_stmt_expr_migration: Statement and Expression Migration

status: pending

**Goal:** Migrate the core codegen functions — `emit_stmt` (line 3047, ~1,400 lines), `emit_expr` (line 5509, ~1,400 lines), `emit_method_call` (line 4888, ~570 lines), and `emit_function`/`emit_class` — from writing directly to `self.output` to building IR nodes that are rendered at the end. This is the bulk of the migration and converts the codegen from an imperative string-writing model to a functional IR-building model.

**Depends on:** milestone_codegen_preamble_migration (the preamble migration proves the IR+renderer pipeline works end-to-end on real output)

### Migration Strategy

The migration uses a **dual-path approach** during the transition:

1. **New `lower_*` methods** return IR nodes: `fn lower_stmt(&self, stmt: &HirStmt) -> Vec<RustStmt>`, `fn lower_expr(&self, expr: &HirExpr) -> RustExpr`, etc.
2. **Old `emit_*` methods** become thin wrappers that call the new `lower_*` methods and render the result into `self.output`.
3. **Unconverted match arms** in the new `lower_*` methods return `RawCode(...)` by capturing the old string output via a temporary buffer.

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

#### Phase B: Compound expressions (recurse into sub-expressions)

7. `HirExpr::BinOp` → `RustExpr::BinOp` (with special cases for string concat, list concat, floor div, power)
8. `HirExpr::UnaryOp` → `RustExpr::UnaryOp`
9. `HirExpr::Compare` → `RustExpr::BinOp` chains
10. `HirExpr::BoolOp` → `RustExpr::BinOp` chains with `&&`/`||`
11. `HirExpr::IfExpr` → `RustExpr::If`
12. `HirExpr::Index` → `RustExpr::MethodCall` (`.get()`) or `RustExpr::Index`
13. `HirExpr::FieldAccess` → `RustExpr::Field`
14. `HirExpr::MethodCall` → `RustExpr::MethodCall` (delegates to `lower_method_call`)
15. `HirExpr::Call` → `RustExpr::FnCall` (with intrinsic dispatch)
16. `HirExpr::FString` → `RustExpr::FormatMacro`
17. `HirExpr::ListLiteral` / `SetLiteral` / `DictLiteral` / `TupleLiteral` → `RustExpr::Vec` / macro calls / `RustExpr::Tuple`
18. `HirExpr::Lambda` → `RustExpr::Closure`
19. `HirExpr::ListComp` / `DictComp` / `SetComp` → iterator chain expressions
20. `HirExpr::ConstructorCall` → `RustExpr::StructInit` or `RustExpr::FnCall`
21. `HirExpr::QuestionMark` → `RustExpr::Try`
22. `HirExpr::OkWrap` / `ErrWrap` → `RustExpr::FnCall` wrapping
23. `HirExpr::Slice` → `lower_list_slice` / `lower_string_slice`
24. `HirExpr::Match` → `RustExpr::Match`

#### Phase C: Statements

25. `HirStmt::Let` → `RustStmt::Let`
26. `HirStmt::Assign` → `RustStmt::Assign`
27. `HirStmt::AugAssign` → `RustStmt::AugAssign` or method call
28. `HirStmt::Return` → `RustStmt::Return` (with context-dependent wrapping for Display impls, generators, try blocks)
29. `HirStmt::Expr` → `RustStmt::Expr`
30. `HirStmt::If` → `RustStmt::If`
31. `HirStmt::While` → `RustStmt::While`
32. `HirStmt::For` → `RustStmt::For` (with iterator adaptation)
33. `HirStmt::Break` / `Continue` → `RustStmt::Break` / `RustStmt::Continue`
34. `HirStmt::Match` → `RustStmt::Match`
35. `HirStmt::TryExcept` → `RustStmt::Match` on Result
36. `HirStmt::Raise` → `RustStmt::Return` with `Err(...)`
37. `HirStmt::Assert` → `RustStmt::If` + `RustStmt::RawCode("panic!(...)")`
38. `HirStmt::FieldAssign` / `SubscriptAssign` → `RustStmt::Assign`

#### Phase D: Top-level items

39. `emit_function` → builds `RustItem::Fn`
40. `emit_class` → builds `RustItem::Struct` + `RustItem::Impl`
41. `emit_protocol_trait` → builds `RustItem::Trait`
42. `emit_enum_class` → builds `RustItem::Enum` + `RustItem::Impl`
43. `emit_operator_impls` → builds `RustItem::Impl` for trait impls
44. `emit_module` → builds `RustFile`

### Temporal Coupling Flags

As each codegen path is converted to IR, the temporal coupling flags can be eliminated:

- **`suppress_field_clone`**: Instead of setting a flag before a method call and clearing it after, the `lower_method_call` function inspects the receiver expression and decides whether to wrap it in `RustExpr::Clone` or not. The decision is local to the function, not a global flag.
- **`in_generator_closure`**: Instead of a flag, `lower_stmt` for `Return` checks whether the enclosing function context is a generator and wraps accordingly. Pass the context as a parameter, not a mutable field.
- **`in_display_impl`**: Same approach — pass the context as a parameter to `lower_stmt`.
- **`in_loop_with_else`**: Pass loop context as a parameter.

These flags are NOT removed in this milestone if doing so would risk regressions. They are removed only when the corresponding codegen path is fully converted and tested. Some flags may persist as parameters to `lower_*` methods rather than mutable struct fields — this is still an improvement (explicit parameter vs hidden mutable state).

### Definition of Done (milestone_codegen_stmt_expr_migration)

- `lower_expr` handles all `HirExpr` variants (some may use `RawCode` for complex cases)
- `lower_stmt` handles all `HirStmt` variants (some may use `RawCode` for complex cases)
- `emit_function` and `emit_class` build IR items
- `emit_module` builds a `RustFile` and renders it
- At least 80% of match arms in `lower_expr` and `lower_stmt` produce structured IR (not `RawCode`)
- The remaining `RawCode` usages are documented with `// TODO: convert to structured IR` comments
- At least 3 temporal coupling flags (`suppress_field_clone`, `in_generator_closure`, `in_display_impl`) are eliminated or converted to explicit parameters
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- At least 10 additional Clippy suppressions from the file header can be removed
- The old `emit_stmt` and `emit_expr` methods are thin wrappers around `lower_*` + render

---

## milestone_codegen_intrinsic_migration: Intrinsic Call Migration

status: pending

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

### Registry Pattern (Optional Improvement)

Instead of a single 1,300-line match statement, intrinsics can be registered in a `HashMap<&str, fn(&RustEmitter, &[HirExpr]) -> RustExpr>` at initialization time. This:
- Makes adding new intrinsics a one-line registration instead of editing a giant match
- Enables intrinsic discovery (list all registered intrinsics)
- Reduces the size of any single function

This is an optional improvement — the match statement works fine functionally. The registry is better for maintainability but adds indirection.

### Definition of Done (milestone_codegen_intrinsic_migration)

- `lower_intrinsic_call` handles all ~80 intrinsic function match arms via structured IR
- `lower_method_call` handles all ~50 method call match arms via structured IR
- No intrinsic match arm contains a `self.write(...)` call longer than 100 characters (the current worst case is 1,500+ characters)
- The `builtin_open` intrinsic is a readable, structured function body (not a single string literal)
- All `RawCode` usages in intrinsics are eliminated (intrinsics are fully converted)
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- The total line count of `sifr_codegen/src/lib.rs` is reduced by at least 500 lines (string templates replaced by more compact IR construction)

---

## milestone_codegen_structural_passes: Structural IR Passes

status: pending

**Goal:** Now that the entire codegen produces structured IR, add optimization and validation passes that operate on the IR before rendering. These passes are impossible with string-based codegen — they require the ability to inspect and transform the output structurally. This milestone delivers the payoff of the IR migration.

**Depends on:** milestone_codegen_intrinsic_migration (the full IR must be in place before structural passes can operate on it)

### Pass 1: Import Collection

Currently: Boolean flags (`needs_hashmap`, `needs_hashset`, `needs_vecdeque`, `needs_bigint`) are set during HIR traversal and checked during preamble generation.

New approach: Walk the `RustFile` IR and collect all `RustType` nodes. If any node contains `RustType::HashMap`, add `use std::collections::HashMap;`. If any contains `RustType::HashSet`, add `use std::collections::HashSet;`. Etc.

This eliminates the boolean flags and makes import collection automatic and correct — if a codegen path forgets to set a flag, the import is still collected.

### Pass 2: Dead Code Elimination (replace `filter_rust_code_to_needed`)

Currently: `filter_rust_code_to_needed` (lines 218-349) parses generated Rust source code as text, extracts named blocks via regex-like heuristics, builds a dependency graph by string-matching function names, computes a transitive closure, and emits only needed blocks.

New approach: Walk the `RustFile` IR. For each `RustItem`, extract its name. Build a dependency graph by walking the item's body and collecting all `RustExpr::Ident` and `RustExpr::Path` references. Compute the transitive closure from the set of imported names. Filter the `RustFile.items` to only include needed items.

This is structurally correct (no false positives from substring matches), handles all edge cases (nested references, type references, trait impl references), and is ~50 lines of code instead of ~130 lines of string parsing.

### Pass 3: Clone Optimization

Currently: `.clone()` is inserted heuristically at 20+ locations based on contextual flags. Some clones are unnecessary (e.g., cloning a `Copy` type, cloning a value that is used only once).

New approach: Walk the IR and identify `RustExpr::Clone(inner)` nodes where:
- `inner` is a literal (no clone needed)
- `inner` is a `Copy` type based on the Sifr type information (no clone needed)
- `inner` is used exactly once in the enclosing scope (move instead of clone)

Remove unnecessary clones. This is a conservative pass — it only removes clones that are provably unnecessary, never adds them.

### Pass 4: IR Validation

Walk the IR and check for structural issues that would cause `rustc` errors:
- Unbalanced braces (impossible with IR, but verify `RawCode` nodes are balanced)
- Empty function bodies (must have at least `()` or a statement)
- Duplicate field names in struct definitions
- `return` outside of a function body

This catches codegen bugs at Sifr compile time rather than at Rust compile time.

### Definition of Done (milestone_codegen_structural_passes)

- Import collection pass eliminates `needs_hashmap`, `needs_hashset`, `needs_vecdeque`, `needs_bigint` boolean flags
- Dead code elimination pass replaces `filter_rust_code_to_needed` (the old string-parsing function is deleted)
- Clone optimization pass removes at least 10% of `.clone()` calls in generated code (measured across the E2E test suite)
- IR validation pass catches at least 3 categories of structural issues
- All existing E2E tests still pass (zero regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes
- The `parse_rust_blocks`, `extract_top_level_item_name`, and `count_braces` helper functions are deleted
- At least 20 of the 34 Clippy suppressions in the file header can be removed
- Binary size of generated programs does not increase (clone optimization should decrease it slightly)

---

## Milestone Ordering

- **milestone_rust_ir_types first:** Everything depends on the IR type definitions. No existing code changes — pure addition.
- **milestone_rust_ir_renderer second:** The renderer is needed before any migration can begin. Also pure addition — no existing code changes.
- **milestone_codegen_preamble_migration third:** The safest migration target (static code, no user-input dependency). Proves the pipeline end-to-end. Eliminates the worst string templates.
- **milestone_codegen_stmt_expr_migration fourth:** The bulk of the migration. Requires the preamble to be proven. Converts the core codegen loop.
- **milestone_codegen_intrinsic_migration fifth:** Intrinsics are the most self-contained. Requires `lower_expr` to be in place so intrinsic bodies can reference it.
- **milestone_codegen_structural_passes last:** New capabilities that require the full IR. The payoff milestone — this is where the architectural investment delivers measurable improvements.
