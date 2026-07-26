# Opaque Handle Tokenizer Evidence

This fixture row tracks the first opaque-handle contract gates for
`@rust.opaque(...)` declarations.

- Positive coverage: `cargo test -p sifr_driver rust_interop` includes
  `package_rust_interop_opaque_probe_accepts_declared_send_sync_copy`, which
  resolves a direct Cargo opaque type and records `Send`/`Sync` probe
  obligations.
- Negative coverage: the same test filter includes
  `package_rust_interop_opaque_probe_rejects_unsatisfied_send_obligation` and
  `package_rust_interop_opaque_probe_rejects_unsatisfied_copy_clone_policy`,
  which report `SIFR-RUST-TYPE-0001` when Cargo probing proves that a Rust
  target does not satisfy the declared `Send` or `Copy` obligation.
- Contract validation coverage: `package_rust_interop_opaque_rejects_unknown_contract_key`
  rejects keys outside the fixed opaque decorator surface.

Full generated opaque resource wrappers and method lowering remain tracked by
`opaque_resource_matrix`.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `declared_send_sync_copy_handle` runs `package_rust_interop_opaque_probe_accepts_declared_send_sync_copy` in `crates/sifr_driver/src/build/rust_interop_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `unsatisfied_send_or_copy_rejected` runs `package_rust_interop_opaque_probe_rejects_unsatisfied_send_obligation` in `crates/sifr_driver/src/build/rust_interop_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
