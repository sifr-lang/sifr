# Tensor and DLPack Fixture

This fixture records contract-only passing coverage for tensor and DLPack view
metadata. The driver validates `dtype=`, `shape=`, `rank=`,
`layout=`, `strides=`, `device=`, and `ownership=` for tensor views, requires
CPU-only device metadata for this verification surface, and rejects DLPack handoff unless
`ownership=transfer`, an owned owner parameter, and `protocol=` are explicit.

Runtime-observed `ndarray` and DLPack crate exchange is pending for the
ecosystem certification fixture pass.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `explicit_tensor_ownership` runs `package_rust_interop_accepts_explicit_dlpack_transfer_contract` in `crates/sifr_driver/src/build/rust_interop_advanced_data_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `implicit_dlpack_ownership_rejected` runs `package_rust_interop_rejects_implicit_dlpack_ownership` in `crates/sifr_driver/src/build/rust_interop_advanced_data_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
