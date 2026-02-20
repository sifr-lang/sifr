# Developer Tools

**Why now:** Developer tooling is how most users interact with the language daily. An LSP with autocomplete, go-to-definition, and hover types dramatically improves the experience for anyone writing Sifr code. With the language feature-complete, the type system stable, and package management in place (so the LSP understands project structure), this is the right time to invest in IDE support, code formatting, linting, and documentation generation.

---

## milestone_dev_tooling: Developer Tooling

status: pending

**Goal:** Provide a complete developer experience with IDE support, code formatting, linting, and documentation generation.

**Depends on:** milestone_package_mgmt (package management needed so the LSP understands project structure from `sifr.toml`)

### Work Items

#### LSP Server (`sifr_lsp` crate)

- Autocomplete: suggest variable names, function names, class members, module exports
- Go-to-definition: jump to function/class/variable definitions across files
- Hover types: show inferred types on hover for variables, expressions, function signatures
- Real-time diagnostics: show type errors, unused variables, and import errors as the user types
- Signature help: show function parameter names and types while typing arguments

#### Formatter (`sifr fmt`)

- Format all valid Sifr code consistently and idempotently
- Configurable style options (indent width, line length) via `sifr.toml`
- Integration with LSP for format-on-save

#### Linter (`sifr lint`)

- Detect unused variables, unreachable code, and style violations
- Configurable rules via `sifr.toml`
- Auto-fix for simple issues (unused imports, trailing whitespace)

#### Documentation Generator (`sifr doc`)

- Generate browsable HTML documentation from docstrings
- Cross-reference links between modules, classes, and functions
- Type signatures displayed in documentation

### Definition of Done (milestone_dev_tooling)

- LSP server provides autocomplete, go-to-definition, hover types, and real-time diagnostics
- `sifr fmt` formats all valid Sifr code consistently and idempotently
- `sifr lint` detects unused variables, unreachable code, and style violations
- `sifr doc` generates browsable HTML documentation from docstrings
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E tests: LSP responds correctly to completion/hover/definition requests
- Milestone demo: a Sifr project with LSP integration, formatted code, and generated docs
