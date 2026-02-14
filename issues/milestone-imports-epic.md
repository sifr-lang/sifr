# milestone_imports: Multi-file Compilation and Imports

## Product Requirements

### Objective

Support multi-file projects with imports, enabling real application structure. Each `.sifr` file is a module. Functions, classes, and type aliases can be imported across modules.

### Scope

#### Features In

1. `from module import name` - import specific names from another module
2. Multi-file compilation - compile a directory of `.sifr` files into one binary
3. Module resolution - find `.sifr` files relative to the project root
4. `_private` prefix convention - names starting with `_` are not importable
5. Circular import detection with clear diagnostics

#### Features Out

| Feature | Reason |
|---------|--------|
| `import module` (bare import) | Deferred -- requires module-as-namespace support |
| `__init__.sifr` packages | Deferred -- needs package semantics |
| Relative imports (`from .utils import`) | Deferred -- needs package structure |
| Re-exports | Deferred -- needs package semantics |

## Solution Design

### Architecture

```
sifr_driver   (module resolution, dependency graph, compilation order)
       ↓
sifr_hir      (import statement lowering, cross-module name resolution)
       ↓
sifr_codegen  (multi-module Rust codegen with mod/use)
       ↓
sifr (tests)  (E2E multi-file tests)
```

### Key Design Decisions

1. **Module = File**: Each `.sifr` file is a module. The module name is the filename without extension.
2. **Flat module structure**: For now, modules are in the same directory as `main.sifr`.
3. **Compilation**: The driver compiles all `.sifr` files in the project directory, resolving imports.
4. **Codegen**: Each module becomes a Rust `mod` block. `from module import name` becomes `use module::name`.

### Task Breakdown

**Task 1: Import Statement Lowering**
- Parse `from module import name1, name2` statements
- Add `HirImport` node to HIR
- Resolve imported names against the target module's exports

**Task 2: Multi-file Driver**
- Update the driver to discover and compile multiple `.sifr` files
- Build a dependency graph from import statements
- Topological sort for compilation order
- Detect circular imports

**Task 3: Multi-module Codegen**
- Generate Rust `mod` blocks for each module
- Generate `use` statements for imports
- Handle visibility (`pub` for exported names, non-`pub` for `_private`)

**Task 4: E2E Tests & Demo**
- Pass tests: multi_file_basic, import_function, import_class
- Fail tests: circular_import, missing_module, private_access
- Milestone demo

### Testing Strategy

| Test | Layer | Check |
|------|-------|-------|
| multi_file_basic | E2E pass | Import function from another module |
| import_class | E2E pass | Import class from another module |
| circular_import | E2E fail | Circular import detection |
| missing_module | E2E fail | Import from non-existent module |
