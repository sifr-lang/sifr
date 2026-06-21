# bridge_version_mismatch

Bridge-version `1` is the generated bridge namespace schema for current
package-local bridge projection evidence.

- Positive evidence: bridge-version `1` packages record generated bridge
  namespace paths and continue through Rust interop metadata resolution
  (`rust_interop_tests::package_rust_interop_resolves_bridge_root`).
- Negative evidence: unsupported bridge versions are rejected as Sifr Rust
  interop metadata errors before probe/build execution
  (`rust_interop_tests::package_rust_interop_rejects_unsupported_bridge_version`).
