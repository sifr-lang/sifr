## Build sifr_driver Crate

#### **Current Situation**

- The individual compiler phases exist (parser, type system, HIR, codegen) but there is no orchestration layer.
- Errors from different phases need to be collected and presented uniformly.
- There is no unified API for "compile this source file."

#### **Desired Situation**

- A `sifr_driver` crate exists that orchestrates the full compilation pipeline.
- A single `compile()` function takes source code and produces either generated Rust code or a list of diagnostics.
- Error reporting uses source spans with nice formatting (file:line:col, code snippets, colored output).
- The driver is the API that the CLI binary calls.

#### **Suggested Solution**

1. Create `crates/sifr_driver/` with:
   - `compile(source: &str, filename: &str) -> CompileResult` function
   - `CompileResult` containing either generated Rust code or diagnostics
   - `check(source: &str, filename: &str) -> Vec<Diagnostic>` for type-check only
2. Implement the pipeline orchestration:
   - Parse source -> AST (collect parse errors)
   - Lower AST -> HIR (collect type errors, name errors, ownership errors)
   - Generate Rust code from HIR (collect codegen errors)
   - Return all diagnostics if any errors, or generated code if clean
3. Implement diagnostic formatting:
   - Use `miette` or `ariadne` crate for pretty error output
   - Include source file name, line number, column number
   - Show code snippet with error location highlighted
   - Categorize: error, warning, note
4. Implement the Rust project generation and build:
   - Write generated Rust code to a temp directory
   - Generate `Cargo.toml` for the output project
   - Invoke `cargo build` and capture output
   - Return the path to the compiled binary
5. Add integration tests that verify the full pipeline:
   - Valid source -> successful compilation
   - Invalid source -> correct diagnostics
