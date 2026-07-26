# Rust Interop Panic Boundary Fixtures

This fixture family tracks Rust panic-to-error contract evidence.

Current passing coverage is source-driven driver/runtime validation:

- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs::package_rust_interop_result_requires_panic_surface`
- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs::package_rust_interop_result_accepts_rust_panic_error_surface`
- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs::package_rust_interop_result_accepts_map_error_surface`
- `crates/sifr_runtime/src/interop.rs::tests::catch_rust_panic_redacts_payload_details`

Generated wrapper emission fixtures remain tracked separately from this contract-only evidence.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `result_declares_rust_panic_error_or_map_error` runs `package_rust_interop_result_accepts_map_error_surface` in `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `panic_payload_not_exposed` runs `package_rust_interop_result_requires_panic_surface` in `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
