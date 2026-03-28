## Update Codegen for Union Types and Narrowing

#### **Current Situation**

- The codegen (`crates/sifr_codegen/src/lib.rs`) maps sifr types 1:1 to Rust types (Int -> i64, Str -> String, etc.).
- There is no concept of generating Rust enums for union types.
- There is no concept of generating `match` expressions for type narrowing.
- Function parameters and return types only handle simple types.

#### **Desired Situation**

- Union types generate Rust enums: `int | str` -> `enum IntOrStr { Int(i64), Str(String) }`
- Enum definitions are emitted once at the top of the generated Rust file
- Function parameters/returns with union types use the generated enum name
- Narrowing (isinstance checks) generates `match` expressions that destructure the enum
- Literal types map to their base Rust type with value checking
- Optional types (`str | None`) generate `Option<String>` in Rust (special case)
- Unknown type generates `Box<dyn std::any::Any>` with downcast in narrowing

#### **Suggested Solution**

**Modified files:**
- `crates/sifr_codegen/src/lib.rs`:
  - Add `UnionRegistry`: tracks all union types encountered, generates unique enum names, deduplicates
  - Add `emit_union_enum_definitions()`: emits all union enum `enum X { ... }` definitions at top of file
  - Add `emit_union_construction()`: wraps a value in its enum variant (e.g., `IntOrStr::Int(42)`)
  - Add `emit_narrowing_match()`: generates `match x { EnumName::Variant(val) => { ... } }` for isinstance narrowing
  - Update `emit_function()`: handle union parameter types and return types
  - Update `emit_expr()`: handle union value construction
  - Update `emit_if()`: detect narrowing and emit match instead of if/else when appropriate
  - Special case: `T | None` -> `Option<T>` (Rust idiomatic), narrowing -> `if let Some(val) = x { ... }`
  - Handle `reveal_type()`: emit a comment in generated Rust showing the type

**Key design decisions:**
- Enum naming: concatenate type names (e.g., `IntOrStr`, `GetOrPostOrPutOrDelete`)
- For literal unions of same base type (e.g., `"GET" | "POST"`), use the base type with value validation at construction
- `None` in unions uses `Option<T>` when there are exactly 2 variants (T and None)
- For 3+ variant unions including None, use a full enum with a `None` variant

**Unit tests:**
- Generate enum for `int | str` and verify valid Rust
- Generate `Option<String>` for `str | None`
- Generate match expression for isinstance narrowing
- Verify generated Rust compiles with `rustc`
