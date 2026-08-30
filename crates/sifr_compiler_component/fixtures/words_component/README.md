# Words compiler-component fixture

This crate builds the non-SQL qualification component. The guest parses the
compiler-owned JSON request. It derives the typed plan, source map, dependency,
diagnostic, and stable plan fingerprint inside the sandbox.

Use the locked toolchain to rebuild the artifact:

```bash
rustup target add wasm32-unknown-unknown
cargo build \
  --manifest-path crates/sifr_compiler_component/fixtures/words_component/Cargo.toml \
  --target wasm32-unknown-unknown \
  --release \
  --locked
cargo run \
  --manifest-path crates/sifr_compiler_component/fixtures/words_component/Cargo.toml \
  --features componentize \
  --bin componentize \
  --locked \
  -- \
  crates/sifr_compiler_component/fixtures/words_component/target/wasm32-unknown-unknown/release/sifr_component_fixture_words.wasm \
  crates/sifr_compiler_component/fixtures/words_component/words_component.wasm
```

Update the artifact digest in
`verification/areas/sql_platform/data/compiler_component_qualification.json`
only when the source or locked build tooling changes.
