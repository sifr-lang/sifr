# shared_bridge_crate

Shared bridge crates are ordinary direct Cargo dependencies with a
package-boundary restriction: shared crates must not import package-specific
`crate::__sifr_bridge` modules.

- Positive `cargo-probe` evidence:
  `test_build_shared_bridge_crate_positive_cargo_probe` consumes the checked-in
  Cargo workspace, compiles and runs both the positive evidence source and the
  scenario source, links the shared crate, and observes `bytes` and `str`
  values crossing the boundary.
- Negative `cargo-probe` evidence:
  `test_check_shared_bridge_crate_negative_cargo_probe` checks the checked-in
  `negative/package_generated_type_import_rejected.sifr` source, negative-only
  trust manifest, and rejected shared-crate Rust source; it observes
  `SIFR-RUST-RESOLVE-0001` for
  `crate::__sifr_bridge::*` before Cargo execution.

Both tests belong to the blocking `sifr_driver_generated_builds` crate-test
suite selected by the merge, nightly, and release profiles in full mode.
Reproduce either side with:

```bash
cargo test -p sifr_driver --lib test_build_shared_bridge_crate_positive_cargo_probe -- --ignored --test-threads=1
cargo test -p sifr_driver --lib test_check_shared_bridge_crate_negative_cargo_probe -- --ignored --test-threads=1
```

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `stable_runtime_types_only` runs `test_build_shared_bridge_crate_positive_cargo_probe` in `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs` through the blocking `sifr_driver_generated_builds` suite at the `merge` profile.
- Negative `package_generated_type_import_rejected` runs `test_check_shared_bridge_crate_negative_cargo_probe` in `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs` through the blocking `sifr_driver_generated_builds` suite at the `merge` profile.
