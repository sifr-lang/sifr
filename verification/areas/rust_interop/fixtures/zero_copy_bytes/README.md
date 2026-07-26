# Rust Interop Zero-Copy Bytes Fixtures

This fixture family tracks the initial compile-time zero-copy bytes view contract surface.

Current evidence:

- positive `borrowed_bytes_view`: `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs::package_rust_interop_zero_copy_accepts_borrowed_bytes_view_contract`
- negative `copy_fallback_rejected`: `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs::package_rust_interop_rejects_zero_copy_copy_fallback`

Runtime-observed crate-backed bytes view certification is pending behind the same fixture family.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `borrowed_bytes_view` runs `package_rust_interop_zero_copy_accepts_borrowed_bytes_view_contract` in `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `copy_fallback_rejected` runs `package_rust_interop_rejects_zero_copy_copy_fallback` in `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
