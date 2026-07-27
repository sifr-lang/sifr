# Bridge Type Matrix Evidence

This fixture row certifies bridge type generation and conversion through a
generated package binary. The positive scenario exercises nested Serde/JSON
values, a display-mapped `thiserror` value, `bytes::Bytes`, and
recursive `indexmap::IndexMap` conversion through dictionaries and
list-of-dictionary values at the full Rust bridge boundary. Sifr dict key
iteration order is not preserved by the internal `HashMap` conversion.

- Supporting compiler coverage: `cargo test -p sifr_codegen rust_interop`
  records `RustBridgeSignatureContract` entries for bytes parameters, `Result`
  returns, generated record bridge types, generated error bridge types, and
  cross-module generated bridge field paths.
- Negative coverage: `cargo test -p sifr_driver rust_interop` rejects an
  incompatible borrowed-bytes Rust signature and rejects unsupported container
  contracts before final binary build.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Negative `unsupported_container_rejections` runs `package_rust_interop_rejects_unsupported_bridge_type_contract` in `crates/sifr_driver/src/build/rust_interop_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Positive `supported_type_roundtrips` runs `test_build_bridge_type_matrix_positive_cargo_probe` in `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs` through the blocking `sifr_driver_generated_builds` suite at the `merge` profile.
