# Rust Interop Panic Wrapper Emission Fixtures

This fixture family certifies generated wrapper behavior that is not covered by
the contract-only panic boundary evidence.

Passing coverage:

- emitted wrappers catch recoverable Rust panics and surface the declared Sifr error channel,
- `panic=map_error(path)` adapters are signature-checked,
- mapper panics fall back to the original redacted `RustPanicError`,
- invalid mapper signatures fail before final generated binary build, and
- `panic=map_error` declarations without a `RustPanicError` fallback are
  rejected before target execution.

Panic-channel validation is structural and nominal: aliases of
`PanicMapped | RustPanicError` are accepted, while `RustPanicErrorish`,
wrapper-only channels, and async mapper declarations are rejected.

The locked `panic_wrapper_runtime` scenario executes success, ordinary bridge
error, mapped panic, and mapper-panic fallback paths. Positive
`generated_wrapper_maps_panic_to_declared_error` runs
`test_build_panic_boundary_wrapper_runtime`; negative
`invalid_map_error_signature_rejected` runs
`test_check_panic_boundary_invalid_mapper_signature`. The same panic-contract
unit suite directly rejects an unrepresentable mapper fallback, a wrapper-only
error channel, a similarly named non-panic error, and async mapper use. Both
fixture-bound tests live in
`crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs` and run
through the blocking `sifr_driver_generated_builds` suite at the `merge`
profile.
