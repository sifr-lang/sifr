# bridge_version_mismatch

Bridge-version `1` is the generated bridge namespace schema for current
package-local bridge projection evidence.

- Positive evidence: bridge-version `1` packages record generated bridge
  namespace paths and continue through Rust interop metadata resolution
  (`rust_interop_tests::package_rust_interop_resolves_bridge_root`).
- Negative evidence: unsupported bridge versions are rejected as Sifr Rust
  interop metadata errors before probe/build execution
  (`rust_interop_tests::package_rust_interop_rejects_unsupported_bridge_version`).

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `bridge_version_1_accepted` runs `package_rust_interop_resolves_bridge_root` in `crates/sifr_driver/src/build/rust_interop_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `unsupported_bridge_version_rejected` runs `package_rust_interop_rejects_unsupported_bridge_version` in `crates/sifr_driver/src/build/rust_interop_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
