# Rust Interop Zero-Copy Bytes Fixtures

This fixture family tracks the initial compile-time zero-copy bytes view contract surface.

Current evidence:

- positive `borrowed_bytes_view`: `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs::package_rust_interop_zero_copy_accepts_borrowed_bytes_view_contract`
- negative `copy_fallback_rejected`: `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs::package_rust_interop_rejects_zero_copy_copy_fallback`

Runtime-observed crate-backed bytes view certification is pending behind the same fixture family.
