# direct_crate_matrix

This fixture family tracks direct bindings for representative dependency roots:
`blake3`, `sha2`, `uuid`, and `regex`.

- Positive evidence: `compatible_direct_signatures` passes through the direct
  Cargo dependency probe path for compatible Rust item shapes.
- Negative evidence: `incompatible_direct_signatures` passes by mapping
  incompatible or unsupported public Rust signatures to stable
  `SIFR-RUST-TYPE-*` or `SIFR-RUST-RESOLVE-*` diagnostics.
- Compatibility category: `supported`. Direct bindings are verified only for bridge-compatible signatures; adapters
  still require explicit bridges.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `compatible_direct_signatures` runs `package_rust_interop_direct_probe_accepts_bridge_signature` in `crates/sifr_driver/src/build/rust_interop_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `incompatible_direct_signatures` runs `package_rust_interop_direct_probe_checks_signature_shape` in `crates/sifr_driver/src/build/rust_interop_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
