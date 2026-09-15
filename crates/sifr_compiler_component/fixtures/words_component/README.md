# Words compiler-component fixture

This crate builds the non-SQL qualification component. The guest parses the
compiler-owned JSON request and derives the typed plan, source map, dependency,
diagnostic and stable plan fingerprint inside the sandbox.

Install the `wasm32-unknown-unknown` target for the repository's pinned Rust
toolchain, then regenerate the artifact and its actual producer receipt:

```bash
python3 verification/areas/sql_platform/tools/build_words_component.py
```

The build uses locked offline dependencies, wit-bindgen 0.61.1 with explicit
macros/realloc/std features for the closed synchronous WIT interface, and
wit-component 0.258.0 with output validation. `component-artifacts.json` records
source, compiler, target-library, componentizer and output digests. The builder
updates the qualification artifact digest only after a successful rebuild.
See [compiler component builds](../../../../internal_docs/compiler_component_builds.md)
for the shared source and tool provenance contract.
