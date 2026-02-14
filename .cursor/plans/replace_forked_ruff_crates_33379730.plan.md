---
name: Replace forked ruff crates
overview: Hybrid approach -- use git dependencies for 4 pure-infrastructure ruff crates that will never change, keep 2 crates (parser and AST) as vendored forks since future milestones may require modifying them.
todos:
  - id: update-workspace-toml
    content: "Update workspace Cargo.toml: remove 4 infrastructure crates from members, add git deps for ruff_text_size/source_file/python_trivia/python_literal; keep sifr_python_ast and sifr_python_parser as workspace members"
    status: completed
  - id: update-crate-tomls
    content: "Update Cargo.toml in all crates: switch infrastructure deps from sifr_* to ruff_*, keep sifr_python_ast and sifr_python_parser references"
    status: completed
  - id: rename-imports
    content: Rename only the 4 infrastructure imports (sifr_text_size -> ruff_text_size, etc.) in source files; leave sifr_python_ast and sifr_python_parser imports unchanged
    status: completed
  - id: update-forked-crates
    content: Update sifr_python_ast and sifr_python_parser Cargo.toml to depend on ruff_text_size/ruff_source_file/ruff_python_trivia via git instead of sifr_* path deps
    status: completed
  - id: delete-infra-crates
    content: Delete the 4 infrastructure crate directories (sifr_text_size, sifr_source_file, sifr_python_trivia, sifr_python_literal)
    status: completed
  - id: build-and-test
    content: Run cargo build, cargo test, and verify demos still work
    status: completed
  - id: update-docs
    content: Update plan file and any comments referencing the crate structure
    status: completed
isProject: false
---

# Hybrid Approach: Git Deps for Infrastructure, Vendored Fork for Parser/AST

## Context

The 6 forked crates from ruff v0.4.10 are currently unmodified copies with only a name rename. However, not all 6 are equal in terms of future modification risk:

**Pure infrastructure (will never change)** -- these do generic text/source handling with no language semantics:

- `sifr_text_size` -- text span/range utilities
- `sifr_source_file` -- source file representation, line indexing
- `sifr_python_trivia` -- whitespace/comment handling
- `sifr_python_literal` -- literal parsing (string escapes, number formats)

**May need modification in future milestones** -- these define the language's syntax:

- `sifr_python_ast` -- AST node definitions. May need new node types if sifr introduces syntax beyond Python (e.g., `Result[T, E]` as a first-class type, pattern matching extensions).
- `sifr_python_parser` -- the parser itself. Will need modification if sifr adds non-Python syntax like the `?` operator (M3) or custom syntax sugar.

## Strategy

- **Replace 4 infrastructure crates** with git dependencies pointing to ruff v0.4.10 (`ruff_text_size`, `ruff_source_file`, `ruff_python_trivia`, `ruff_python_literal`)
- **Keep 2 crates as vendored forks** (`sifr_python_ast`, `sifr_python_parser`) since they may need modification starting from M3
- Update the kept forks to depend on the git-referenced `ruff_*` infrastructure crates instead of the deleted `sifr_*` path deps

## Steps

### 1. Update workspace `Cargo.toml`

Remove the 4 infrastructure crates from `[workspace.members]`. Add git dependencies for them. Keep `sifr_python_ast` and `sifr_python_parser` as workspace members:

```toml
[workspace]
members = [
    "crates/sifr_python_ast",
    "crates/sifr_python_parser",
    "crates/sifr_hir",
    "crates/sifr_type_system",
    "crates/sifr_codegen",
    "crates/sifr_driver",
    "crates/sifr",
]

[workspace.dependencies]
# Infrastructure crates -- unmodified, referenced from ruff v0.4.10
ruff_text_size = { git = "https://github.com/astral-sh/ruff", tag = "v0.4.10" }
ruff_source_file = { git = "https://github.com/astral-sh/ruff", tag = "v0.4.10" }
ruff_python_trivia = { git = "https://github.com/astral-sh/ruff", tag = "v0.4.10" }
ruff_python_literal = { git = "https://github.com/astral-sh/ruff", tag = "v0.4.10" }

# Vendored forks -- kept for future modification
sifr_python_ast = { path = "crates/sifr_python_ast" }
sifr_python_parser = { path = "crates/sifr_python_parser" }
```

### 2. Update vendored fork `Cargo.toml` files

Update `sifr_python_ast/Cargo.toml` and `sifr_python_parser/Cargo.toml` to depend on `ruff_text_size`, `ruff_source_file`, `ruff_python_trivia` (git deps) instead of the deleted `sifr_*` path deps.

### 3. Update custom crate `Cargo.toml` files

In `sifr_hir`, `sifr_type_system`, `sifr_codegen`, `sifr_driver`:

- Change infrastructure deps from `sifr_text_size` to `ruff_text_size`, etc.
- Keep `sifr_python_ast` and `sifr_python_parser` references as-is

### 4. Rename only infrastructure imports in source code

Search-and-replace only the 4 infrastructure crate names:

- `sifr_text_size` -> `ruff_text_size`
- `sifr_source_file` -> `ruff_source_file`
- `sifr_python_trivia` -> `ruff_python_trivia`
- `sifr_python_literal` -> `ruff_python_literal`

Leave all `sifr_python_ast` and `sifr_python_parser` imports unchanged.

Also update these names inside the vendored `sifr_python_ast` and `sifr_python_parser` source files (they import the infrastructure crates internally).

### 5. Delete the 4 infrastructure crate directories

- `crates/sifr_text_size/`
- `crates/sifr_source_file/`
- `crates/sifr_python_trivia/`
- `crates/sifr_python_literal/`

### 6. Build and test

- `cargo build` -- verify compilation
- `cargo test` -- verify all tests pass
- `sifr run tmp/m1_demo.sifr` and `sifr run tmp/m2_demo.sifr` -- verify demos work

### 7. Update documentation

- Update the architecture plan's crate structure section
- Update comments referencing "forked from ruff" to clarify the hybrid approach

## Risks and future considerations

### ruff workspace dependency resolution

The ruff crates use `[workspace.dependencies]` internally. When pulled as git dependencies, Cargo resolves these from the ruff workspace's `Cargo.toml`, so they should work. If any fail, we may need to add those external deps explicitly.

### Renaming within vendored forks

The vendored `sifr_python_ast` and `sifr_python_parser` currently import `sifr_text_size`, `sifr_source_file`, etc. internally. These imports must be updated to `ruff_text_size`, `ruff_source_file`, etc. to match the git dependencies. This is a one-time rename within those two crates.

### Future modifications to parser/AST

The vendored `sifr_python_ast` and `sifr_python_parser` are kept specifically because future milestones may require changes:

- **M3**: The `?` operator requires parser changes (new syntax not in Python)
- **M3**: `Result[T, E]` / `Option[T]` may benefit from dedicated AST nodes
- **M4+**: Custom syntax sugar or language extensions

When those milestones arrive, the vendored forks are ready to modify directly.