# Direct Crate Negative Type Evidence

This fixture row is covered by focused driver diagnostics for unsupported
Sifr-facing Rust bridge type contracts.

- Diagnostic crate rationale: `regex` supplies the rejected unsupported
  direct-signature shape used only to exercise `SIFR-RUST-TYPE` diagnostics.
  The crate is not linked or executed by this fixture.
- Positive rejection coverage: `cargo test -p sifr_driver rust_interop` emits
  `SIFR-RUST-TYPE-0001` for unsupported bridge type contracts such as `set[int]`.
- Negative silent-compile coverage: the same test fails before direct Cargo
  probing or final generated binary materialization, so unsupported containers do
  not fall through to raw Rust build failures.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `unsupported_type_rejected` runs `direct_negative_type_reports_stable_unsupported_container_diagnostic` in `crates/sifr_driver/src/build/rust_interop_evidence_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `unsupported_type_cannot_compile_silently` runs `direct_negative_type_stops_before_cargo_probe_execution` in `crates/sifr_driver/src/build/rust_interop_evidence_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
