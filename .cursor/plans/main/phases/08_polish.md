# Polish

This phase adds the final features that turn Sifr from a functional language into a complete ecosystem: compile-time metaprogramming, FFI for accessing the Rust/C ecosystem, package management with dependency resolution, developer tooling (LSP, formatter, linter), and a package registry with incremental compilation and REPL.

---

## milestone_metaprogramming: Metaprogramming

`status: pending`

**Goal:** Support compile-time code generation and advanced decorators. **Note:** basic function decorators (runtime wrapping) are already available from milestone_decorators. This milestone adds compile-time AST transformation decorators.

### Language Features

- **Compile-time decorators:** `@decorator` maps to Rust attribute macros or AST transforms (extends milestone_decorators's runtime decorators with compile-time power)
- `**@dataclass`:** auto-generate `__init__`, `__eq__`, `__repr__` (like Rust `#[derive]`)
- **Custom decorators:** user-defined compile-time transforms (note: basic `@property` getter/setter is delivered in milestone_inheritance; this milestone extends it with compile-time computed/cached property variants if needed)
- `***args` / `**kwargs`:** delivered in milestone_decorators (needed for generic decorators). Available here for use in compile-time decorator transforms.
- **Compile-time evaluation:** `const` expressions evaluated at compile time

### Example

```python
@dataclass
class Config:
    host: str
    port: int
    debug: bool = False

# Auto-generates __init__, __eq__, __repr__, clone
# Maps to Rust #[derive(Debug, Clone, PartialEq)] struct
```

### Security Boundary for Compile-time Evaluation

Compile-time evaluation (`const` expressions, custom decorators) runs during compilation. To prevent supply-chain attacks via malicious packages:

- **No I/O at compile time:** compile-time evaluation cannot read files, make network requests, or access environment variables. It is a pure computation sandbox.
- **No arbitrary code execution:** custom decorators are limited to AST transformations (adding/removing/modifying fields and methods). They cannot execute arbitrary Rust code or shell commands.
- **Deterministic:** compile-time evaluation must produce the same output for the same input, regardless of the host system.

### Definition of Done (milestone_metaprogramming)

- `@dataclass` generates `__init__`, `__eq__`, `__repr__`, `clone` methods
- Custom decorators can transform class definitions (add/remove fields and methods)
- `*args` / `**kwargs` (delivered in milestone_decorators) work within compile-time decorator transforms
- Positional-only parameters (`def f(x, /, y)`) work
- `const` expressions evaluated at compile time
- Compile-time sandbox enforced (no I/O, no side effects)
- Deterministic compile-time expansion: same source always produces same output (important for caching in milestone_ecosystem)
- E2E pass tests: dataclass_basic, property_decorator, custom_decorator, const_eval, positional_only_params
- Milestone demo in `./demos/milestone_metaprogramming_demo.sifr`

---

## milestone_ffi: FFI and Interop

`status: pending`

**Goal:** Give Sifr access to the entire Rust and C ecosystem via foreign function interfaces. This is the escape hatch that makes Sifr practical before every Rust crate has a Sifr wrapper -- users can call any Rust crate directly.

### Language Features

- **Rust FFI:** call Rust crates directly from Sifr code using `extern` blocks
- **C FFI:** call C libraries via `unsafe` blocks (maps to Rust's `extern "C"`)
- `**unsafe` keyword:** required for any FFI call. The compiler emits a warning for any `unsafe` usage, encouraging safe wrappers
- **Python interop (stretch):** call Python libraries via PyO3 bindings

### FFI Syntax

```python
# Declare an external Rust crate dependency
extern crate uuid

# Use it in Sifr code
from uuid import Uuid

def main():
    id: str = unsafe { Uuid.new_v4().to_string() }
    print(f"Generated UUID: {id}")
```

### FFI Security Boundary

FFI introduces unsafe code into the Sifr ecosystem. The following policies apply:

- `**unsafe` keyword required:** any FFI call must be wrapped in an `unsafe` block
- **Panic boundary (Rust FFI):** Rust FFI entry points are wrapped in `catch_unwind`. Unwinding panics from Rust libraries are caught and converted to `Result::Err`. Note: if the Rust library is compiled with `panic=abort`, the process will abort instead of unwinding -- this is a known limitation documented in the FFI guide.
- **Crash boundary (C FFI):** C library crashes (segfault, `abort()`, stack overflow) are **not recoverable** -- the process terminates. Safe wrappers must validate inputs before calling C functions. The compiler emits a warning when `extern "C"` functions are called without a safe wrapper.
- **Non-recoverable cases:** stack overflow, double panic, `abort()`, and C-level undefined behavior always terminate the process. These are explicitly documented as non-catchable.
- **No implicit `unsafe`:** stdlib wrappers (milestone_protocols-milestone_data_processing) encapsulate all `unsafe` internally. User code never needs `unsafe` unless calling raw FFI
- **Type mapping:** the compiler maps Sifr types to Rust types at FFI boundaries. Mismatches are compile-time errors

### Codegen

- `extern crate` declarations add the crate to the generated `Cargo.toml` dependencies
- `unsafe { ... }` blocks generate Rust `unsafe { ... }` blocks
- FFI function calls generate direct Rust function calls with type-mapped arguments
- Rust FFI return values are wrapped in `Result` when `catch_unwind` is applied
- C FFI return values are passed through directly (no automatic wrapping)

### Definition of Done (milestone_ffi)

- `extern crate` adds Rust crate dependencies to generated Cargo.toml
- Rust FFI calls compile and execute correctly
- `unsafe` blocks required and enforced by the compiler
- Rust FFI panic boundary (`catch_unwind`) wraps entry points and converts panics to `Result::Err`
- C FFI via `extern "C"` works for basic function calls
- C FFI non-recoverability is documented; compiler warns on unwrapped `extern "C"` calls
- Type mapping between Sifr and Rust types at FFI boundaries
- Rustc-to-Sifr error span translation: errors from FFI-generated code map back to the Sifr source location
- E2E pass tests: ffi_rust_crate, ffi_c_function, unsafe_block, ffi_rust_panic_caught
- E2E fail tests: missing_unsafe, ffi_type_mismatch
- Milestone demo in `./demos/milestone_ffi_demo.sifr` (calling a Rust crate from Sifr)

---

## milestone_package_mgmt: Package Management

`status: pending`

**Goal:** Add the package management infrastructure that was deferred from milestone_imports. Now that the language is mature and a registry is about to be built (milestone_ecosystem), it's time to add dependency resolution, lockfiles, and the `sifr add` command.

### Language Features

- `**sifr.toml`:** project manifest with `[dependencies]` section. Version ranges use semver (e.g., `requests = "^1.2"`).
- `**sifr.lock`:** auto-generated lockfile with exact resolved versions, content hashes (SHA-256), and source URLs. Must be committed to version control for reproducible builds.
- **Version solver:** PubGrub-based algorithm (same as Cargo and uv). Resolves the full dependency graph with conflict detection and clear error messages.
- **Dependency sources:** git repositories and local paths. Registry support (`sifr.dev`) added in milestone_ecosystem.
- `**sifr add <package>`:** adds a dependency to `sifr.toml` and resolves the lockfile.
- `**sifr remove <package>`:** removes a dependency.

### Definition of Done (milestone_package_mgmt)

- `sifr.toml` parsed and used for project configuration and dependencies
- `sifr.lock` generated with exact versions and content hashes
- `sifr add` resolves and updates lockfile
- `sifr remove` removes dependencies cleanly
- PubGrub solver handles version conflicts with clear diagnostics
- Git and local path dependencies work
- E2E pass tests: add_dependency, remove_dependency, lockfile_generation, version_conflict_resolution
- Milestone demo in `./demos/milestone_package_mgmt_demo.sifr`

---

## milestone_dev_tooling: Developer Tooling

`status: pending`

**Goal:** Provide the developer experience tools that make Sifr productive for daily use: IDE support, code formatting, linting, and documentation generation. These tools are what make a language feel "real" to developers.

### LSP Server (`sifr_lsp`)

A Language Server Protocol implementation that provides IDE features:

- **Autocomplete:** suggest variables, functions, methods, and types based on scope and type information
- **Go-to-definition:** jump to the definition of any symbol
- **Hover types:** show the inferred type of any expression on hover
- **Diagnostics:** show type errors, unused variables, and linter warnings in real-time
- **Rename refactor:** rename a symbol across all files in the project
- **Find references:** find all usages of a symbol

**Implementation:** built as a new `sifr_lsp` crate using the `tower-lsp` Rust crate. Reuses the existing parser, type checker, and HIR infrastructure. The LSP server runs the compiler pipeline incrementally on file changes.

### Formatter (`sifr fmt`)

An opinionated code formatter that enforces consistent style:

- **Indentation:** 4 spaces (like Python/ruff)
- **Line length:** 88 characters (like Black/ruff)
- **String quotes:** double quotes by default
- **Trailing commas:** always in multi-line constructs
- **Import sorting:** alphabetical, grouped by stdlib/third-party/local

**Implementation:** built as a new `sifr_fmt` crate. Can reuse ruff's formatting infrastructure as a reference. Operates on the AST (parse -> format -> emit), preserving comments.

### Linter (`sifr lint`)

A linter that catches common mistakes beyond type errors:

- **Unused variables/imports:** warn when a variable or import is never used
- **Unreachable code:** warn when code follows a `return` or `raise`
- **Shadowed variables:** warn when a variable shadows an outer scope variable
- **Style violations:** enforce naming conventions (snake_case for functions/variables, PascalCase for classes)
- **Complexity warnings:** warn when functions exceed cyclomatic complexity thresholds

**Implementation:** built as a new `sifr_lint` crate. Operates on the HIR (after type checking), so it has full type information available.

### Documentation Generator (`sifr doc`)

Generate HTML documentation from docstrings:

- **Docstring format:** triple-quoted strings at the top of functions/classes/modules
- **Output:** static HTML site (like Rust's `rustdoc`)
- **Cross-references:** link to other symbols in the documentation
- **Type signatures:** automatically include type annotations in the docs

### Definition of Done (milestone_dev_tooling)

- LSP server provides autocomplete, go-to-definition, hover types, and real-time diagnostics
- LSP works with VS Code (via extension) and any LSP-compatible editor
- `sifr fmt` formats all valid Sifr code consistently and idempotently
- `sifr lint` detects unused variables, unreachable code, and style violations
- `sifr doc` generates browsable HTML documentation from docstrings
- E2E tests: LSP responds correctly to completion/hover/definition requests
- Formatter round-trip test: `format(format(code)) == format(code)`
- Milestone demo in `./demos/milestone_dev_tooling_demo.sifr` (project with LSP, formatted code, and generated docs)

---

## milestone_ecosystem: Package Ecosystem

`status: pending`

**Goal:** Build the infrastructure for sharing and reusing Sifr code: a package registry, incremental compilation for fast iteration, and a REPL for interactive exploration. This is the milestone that turns Sifr from a language into an ecosystem.

### Package Registry (`sifr.dev`)

A package registry for publishing and installing Sifr packages:

- **Publish:** `sifr publish` uploads a package to `sifr.dev`
- **Install:** `sifr add <package>` resolves from the registry (extends milestone_package_mgmt's git/path-only support)
- **Versioning:** semver with the PubGrub solver (from milestone_package_mgmt)
- **Trust model:** packages with `unsafe` usage are flagged and require explicit opt-in by the consumer (`allow_unsafe = true` in `sifr.toml`)
- **Package metadata:** name, version, description, license, repository URL, dependencies
- **Search:** `sifr search <query>` searches the registry

### Incremental Compilation

Optimize the compiler for fast iteration during development:

- **Module-level caching:** only recompile modules whose source (or dependencies) changed
- **Generated Rust caching:** cache the generated `.rs` files and skip codegen for unchanged modules
- **Cargo build caching:** leverage Cargo's built-in incremental compilation for the Rust compilation step
- **File watcher mode:** `sifr watch` recompiles on file changes (like `cargo watch`)

**Cache key and invalidation contract:**

- **Cache key:** content hash (SHA-256) of the source file combined with the public API signature hash of all transitive dependencies. Two compilations with the same cache key produce identical output.
- **Public API signature hash:** a hash of the module's exported symbols (function signatures, type definitions, re-exports). If only the implementation body changes but the public API is identical, dependents are NOT recompiled.
- **Transitive invalidation:** if module A depends on module B, and B's public API hash changes, A is recompiled. If B's API hash is unchanged (implementation-only change), A is skipped.
- **Decorator/macro expansion:** expansion output is included in the content hash. A decorator that changes its output invalidates the module even if the source text is unchanged.
- **Detailed design deferred:** the full cache storage format, eviction policy, and cross-machine sharing strategy will be designed during milestone_ecosystem implementation.

### REPL (`sifr repl`)

An interactive mode for quick experimentation:

- **Expression evaluation:** type an expression, see the result immediately
- **Type display:** show the inferred type of each expression
- **Multi-line input:** support for function definitions and control flow
- **History:** up/down arrow for command history

**Implementation:** compile each REPL input as a small Sifr program, run it, and display the result. Use `rustyline` for line editing.

### Definition of Done (milestone_ecosystem)

- `sifr publish` uploads packages to `sifr.dev`
- `sifr add <package>` resolves and installs from the registry
- Package trust model enforced (unsafe flagging, opt-in)
- Incremental compilation skips unchanged modules
- `sifr watch` recompiles on file changes
- `sifr repl` provides interactive expression evaluation with type display
- Fuzz testing for parser and type checker integrated into CI
- Benchmark suite with regression thresholds for compile time and binary size
- Milestone demo: a complete web application built entirely in Sifr, published as a package

---

## Milestone Ordering

- **milestone_metaprogramming-milestone_ecosystem last:** Metaprogramming, FFI, package management, tooling, and ecosystem polish come after the language is functional for real-world use
- **milestone_ffi before milestone_package_mgmt:** FFI unlocks access to the full Rust crate ecosystem; package management benefits from a stable language surface
- **milestone_package_mgmt before milestone_dev_tooling:** Package management infrastructure needed before developer tooling
- **milestone_dev_tooling before milestone_ecosystem:** LSP and formatter should exist before the package registry launches, so published packages have consistent quality
