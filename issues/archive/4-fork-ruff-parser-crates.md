## Fork and Rename Ruff Parser Crates + Setup Cargo Workspace

> **Update:** This task was originally completed by forking all 6 crates. It has since been refactored to a **hybrid approach**: 4 infrastructure crates (`ruff_text_size`, `ruff_source_file`, `ruff_python_trivia`, `ruff_python_literal`) are now git dependencies from ruff v0.4.10, while 2 crates (`sifr_python_ast`, `sifr_python_parser`) remain as vendored forks for future modification. See `replace_forked_ruff_crates_33379730.plan.md` for details.

#### **Current Situation**

- There is no compiler codebase yet -- the workspace is empty (no Cargo.toml, no Rust source files).
- The ruff project (MIT licensed) at `/Users/yaseralnajjar/work/sifr/ruff` contains battle-tested Python parser crates that we need as the foundation for Sifr's parser.
- The 6 crates we need are: `ruff_text_size`, `ruff_source_file`, `ruff_python_trivia`, `ruff_python_ast`, `ruff_python_parser`, `ruff_python_literal`.

#### **Desired Situation**

- A Cargo workspace is set up at the repository root with the 2 vendored fork crates and git dependencies for the 4 infrastructure crates.
- All crates compile successfully with `cargo build`.
- Parser tests pass with `cargo test -p sifr_python_parser`.
- The workspace is ready for adding new crates (type system, HIR, codegen, driver, CLI).

#### **Suggested Solution**

1. Create workspace `Cargo.toml` at repo root with explicit workspace members.
2. Fork 2 parser/AST crates from `/Users/yaseralnajjar/work/sifr/ruff/crates/` into `crates/`:
   - `ruff_python_ast` -> `sifr_python_ast`
   - `ruff_python_parser` -> `sifr_python_parser`
3. Reference 4 infrastructure crates as git dependencies from ruff v0.4.10:
   - `ruff_text_size`, `ruff_source_file`, `ruff_python_trivia`, `ruff_python_literal`
4. Rename `ruff_` prefixes to `sifr_` only in the vendored fork crates' Cargo.toml files and source code.
5. Set up workspace-level dependency versions for shared external crates (bitflags, memchr, etc.).
6. Add MIT license attribution for the forked code.
7. Verify `cargo build` and `cargo test` pass for all crates.
8. Update `.gitignore` to include `/target/` and Cargo build artifacts.
