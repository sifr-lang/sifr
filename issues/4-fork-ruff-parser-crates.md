## Fork and Rename Ruff Parser Crates + Setup Cargo Workspace

#### **Current Situation**

- There is no compiler codebase yet -- the workspace is empty (no Cargo.toml, no Rust source files).
- The ruff project (MIT licensed) at `/Users/yaseralnajjar/work/sifr/ruff` contains battle-tested Python parser crates that we need as the foundation for Sifr's parser.
- The 6 crates we need are: `ruff_text_size`, `ruff_source_file`, `ruff_python_trivia`, `ruff_python_ast`, `ruff_python_parser`, `ruff_python_literal`.

#### **Desired Situation**

- A Cargo workspace is set up at the repository root with all 6 forked crates renamed with `sifr_` prefix.
- All crates compile successfully with `cargo build`.
- Parser tests pass with `cargo test -p sifr_python_parser`.
- The workspace is ready for adding new crates (type system, HIR, codegen, driver, CLI).

#### **Suggested Solution**

1. Create workspace `Cargo.toml` at repo root with `members = ["crates/*"]`.
2. Copy the 6 ruff crates from `/Users/yaseralnajjar/work/sifr/ruff/crates/` into `crates/`:
   - `ruff_text_size` -> `sifr_text_size`
   - `ruff_source_file` -> `sifr_source_file`
   - `ruff_python_trivia` -> `sifr_python_trivia`
   - `ruff_python_ast` -> `sifr_python_ast`
   - `ruff_python_parser` -> `sifr_python_parser`
   - `ruff_python_literal` -> `sifr_python_literal`
3. Rename all `ruff_` prefixes to `sifr_` in Cargo.toml files, source code imports, and module references.
4. Set up workspace-level dependency versions for shared external crates (bitflags, memchr, etc.).
5. Add MIT license attribution for the forked code.
6. Verify `cargo build` and `cargo test` pass for all forked crates.
7. Update `.gitignore` to include `/target/` and Cargo build artifacts.
