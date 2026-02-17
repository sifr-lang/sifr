# Polish and Tooling

This phase adds the final features that turn Sifr from a functional language into a complete ecosystem: compile-time metaprogramming, FFI for accessing the Rust/C ecosystem, package management with dependency resolution, developer tooling (LSP, formatter, linter), and a package registry with incremental compilation and REPL.

---

## milestone_metaprogramming: Compile-Time Decorators

status: pending

**Goal:** Support compile-time code generation and advanced decorators.

**Depends on:** milestone_data_processing (the full language and web stack should be complete)

### Work Items

- `@dataclass`: auto-generate `__init__`, `__eq__`, `__repr__`
- Custom decorators: user-defined compile-time AST transforms
- Positional-only parameters (`def f(x, /, y)`)

### Definition of Done (milestone_metaprogramming)

- `@dataclass` generates `__init__`, `__eq__`, `__repr__`, `clone` methods
- Custom decorators can transform class definitions
- Positional-only parameters work
- E2E pass tests: dataclass_basic, custom_decorator, positional_only_params
- Milestone demo in `./demos/milestone_metaprogramming_demo.sifr`

---

## milestone_ffi: Foreign Function Interface

status: pending

**Goal:** Give Sifr access to the entire Rust and C ecosystem via foreign function interfaces.

**Depends on:** milestone_metaprogramming (language features should be complete)

### Work Items

- Rust FFI, C FFI, `unsafe` keyword

### Definition of Done (milestone_ffi)

- `extern crate` adds Rust crate dependencies to generated Cargo.toml
- Rust FFI calls compile and execute correctly
- `unsafe` blocks required and enforced by the compiler
- C FFI via `extern "C"` works for basic function calls
- E2E pass tests: ffi_rust_crate, ffi_c_function, unsafe_block
- E2E fail tests: missing_unsafe, ffi_type_mismatch
- Milestone demo in `./demos/milestone_ffi_demo.sifr`

---

## milestone_package_mgmt: Package Management

status: pending

**Goal:** Add package management infrastructure.

**Depends on:** milestone_ffi (FFI unlocks the Rust crate ecosystem; package management benefits from a stable language surface)

### Work Items

- `sifr.toml`, `sifr.lock`, dependency resolution, `sifr add`/`remove`

### Definition of Done (milestone_package_mgmt)

- `sifr.toml` parsed and used for project configuration and dependencies
- `sifr.lock` generated with exact versions and content hashes
- `sifr add` / `sifr remove` work
- PubGrub solver handles version conflicts with clear diagnostics
- E2E pass tests: add_dependency, remove_dependency, lockfile_generation
- Milestone demo in `./demos/milestone_package_mgmt_demo.sifr`

---

## milestone_dev_tooling: Developer Tooling

status: pending

**Goal:** Provide developer experience tools.

**Depends on:** milestone_package_mgmt (package management needed before developer tooling)

### Work Items

- LSP server, formatter (`sifr fmt`), linter (`sifr lint`), doc generator (`sifr doc`)

### Definition of Done (milestone_dev_tooling)

- LSP server provides autocomplete, go-to-definition, hover types, and real-time diagnostics
- `sifr fmt` formats all valid Sifr code consistently and idempotently
- `sifr lint` detects unused variables, unreachable code, and style violations
- `sifr doc` generates browsable HTML documentation from docstrings
- E2E tests: LSP responds correctly to completion/hover/definition requests

---

## milestone_ecosystem: Package Ecosystem

status: pending

**Goal:** Build the infrastructure for sharing and reusing Sifr code.

**Depends on:** milestone_dev_tooling (LSP and formatter should exist before the registry launches)

### Work Items

- Package registry (`sifr.dev`), incremental compilation, REPL

### Definition of Done (milestone_ecosystem)

- `sifr publish` uploads packages to `sifr.dev`
- `sifr add <package>` resolves and installs from the registry
- Incremental compilation skips unchanged modules
- `sifr repl` provides interactive expression evaluation with type display
- Milestone demo: a complete web application built entirely in Sifr, published as a package

---

## Milestone Ordering

- **milestone_metaprogramming first:** Compile-time decorators complete the language feature set.
- **milestone_ffi second:** FFI unlocks the Rust/C ecosystem.
- **milestone_package_mgmt third:** Package management infrastructure needed before tooling.
- **milestone_dev_tooling fourth:** LSP and formatter should exist before the registry launches.
- **milestone_ecosystem last:** The registry and REPL are the final ecosystem pieces.
