# Rust Interop Zero-Copy View Matrix Fixtures

This fixture family tracks the initial compile-time owner/lifetime/mutability view contract surface.

Current evidence:

- positive `owner_lifetime_views`: `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs::package_rust_interop_zero_copy_accepts_borrowed_bytes_view_contract`
- negative `mutable_alias_rejected`: `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs::package_rust_interop_rejects_mutable_view_from_shared_borrow_owner`

Runtime-observed `memmap2`, `bytemuck`, and `zerocopy` certification is pending behind this fixture family.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `owner_lifetime_views` runs `package_rust_interop_view_send_sync_metadata_reaches_probe_plan` in `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `mutable_alias_rejected` runs `package_rust_interop_rejects_mutable_view_from_shared_borrow_owner` in `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
