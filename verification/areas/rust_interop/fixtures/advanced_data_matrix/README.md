# Advanced Data Matrix Fixture

This fixture records contract-only passing coverage for advanced data metadata
across Arrow/dataframe and tensor/DLPack bridge categories. The focused unit coverage
checks schema identity, shared bridge crate boundaries, tensor dtype/rank/layout
metadata, tensor shape/strides metadata, CPU-only device metadata, and invalid
dtype/shape metadata rejection.

Runtime-observed certification for `datafusion`, `polars`, `ndarray`, and
`candle` is pending for ecosystem certification.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `advanced_data_metadata` runs `package_rust_interop_accepts_tensor_metadata_contract` in `crates/sifr_driver/src/build/rust_interop_advanced_data_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `dtype_shape_mismatch` runs `package_rust_interop_rejects_tensor_rank_shape_length_mismatch` in `crates/sifr_driver/src/build/rust_interop_advanced_data_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
