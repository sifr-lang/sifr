# Arrow Record Batch Fixture

This fixture records contract-only passing coverage for advanced data views.
The driver validates `@rust.view(..., data=arrow_record_batch, ...)`
metadata, requires explicit schema identity through `schema=`, requires explicit
borrowed or owned view ownership, and enforces the `sifr_arrow_bridge` shared
bridge crate boundary.

Runtime-observed `arrow` crate record batch exchange is pending for the
ecosystem certification fixture pass.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `arrow_schema_identity` runs `package_rust_interop_accepts_arrow_record_batch_metadata_contract` in `crates/sifr_driver/src/build/rust_interop_advanced_data_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `invalid_arrow_metadata` runs `package_rust_interop_rejects_arrow_view_without_schema` in `crates/sifr_driver/src/build/rust_interop_advanced_data_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
