# local_bridge_blake3

Current evidence builds the checked-in package-local bridge scenario with its
locked Cargo graph. Broader runtime ecosystem certification remains separate.

- Positive evidence copies `local_blake3_bridge`, installs the canonical
  positive source, builds the generated package against its package-local
  bridge export, and runs the binary.
- Negative evidence installs the canonical missing-export source in the same
  Cargo scenario and observes `SIFR-RUST-RESOLVE-0001` for the local
  `bridge.blake3.missing_export` target.
- Archive evidence: Rust-backed local bridges require `Cargo.toml`, managed
  projection files, and user bridge files in package archives
  (`package_rust_bridge_archive_tests::*`).

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `local_bridge_hash_bytes` runs `test_build_local_bridge_blake3_positive_cargo_probe` in `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs` through the blocking `sifr_driver_generated_builds` suite at the `merge` profile.
- Negative `missing_local_bridge_export` runs `test_check_local_bridge_blake3_missing_export_cargo_probe` in `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs` through the blocking `sifr_driver_generated_builds` suite at the `merge` profile.
