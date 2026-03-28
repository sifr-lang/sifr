## Build sifr_codegen Crate

#### **Current Situation**

- The HIR contains typed, name-resolved, ownership-checked intermediate representation.
- There is no code generation step to produce Rust source code from the HIR.
- The goal is to emit valid Rust that can be compiled by `rustc` via `cargo build`.

#### **Desired Situation**

- A `sifr_codegen` crate exists that walks the typed HIR and emits Rust source code.
- Generated Rust code compiles with `cargo build` without errors.
- Type mapping is correct: `int`->`i64`, `float`->`f64`, `bool`->`bool`, `str`->`String`, `None`->`()`.
- `print()` maps to `println!` macro.
- Function definitions, if/else, assignments, and expressions all generate correct Rust.
- Codegen snapshot tests verify the generated Rust output.

#### **Suggested Solution**

1. Create `crates/sifr_codegen/` with a `RustEmitter` struct that:
   - Maintains an output buffer with indentation tracking
   - Walks HIR nodes and emits corresponding Rust code
   - Handles type mapping (Sifr types -> Rust types)
2. Implement code generation for:
   - Function definitions: `def foo(x: int) -> int:` -> `fn foo(x: i64) -> i64 {`
   - Variable declarations: `x: int = 5` -> `let x: i64 = 5;`
   - Inferred variables: `x = 5` -> `let x = 5_i64;`
   - If/elif/else: Python `if`/`elif`/`else` -> Rust `if`/`else if`/`else`
   - Return statements: `return x` -> `return x;` (or expression position)
   - Binary operations: same syntax in Rust for arithmetic/comparison
   - Boolean operations: `and`/`or`/`not` -> `&&`/`||`/`!`
   - Floor division: `//` -> custom implementation or `/` for integers
   - Print: `print(x)` -> `println!("{}", x);` (with Display formatting)
   - String concatenation: `+` on strings -> `format!("{}{}", a, b)` or similar
3. Generate a complete Rust project:
   - `Cargo.toml` with `[package]` metadata
   - `src/main.rs` with the generated code
   - `main()` function as entry point
4. Add codegen snapshot tests:
   - `.sifr` input files in `resources/codegen/`
   - `.snap` files with expected Rust output
   - Test that generated Rust compiles with `rustc`
