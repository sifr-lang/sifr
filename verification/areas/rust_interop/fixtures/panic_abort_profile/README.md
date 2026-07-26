# Rust Interop Panic Abort Profile Fixtures

This fixture family tracks explicit abort-profile evidence for Rust interop panic policy.

Current passing coverage is source-driven driver validation:

- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs::package_rust_interop_abort_policy_requires_trust`
- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs::package_rust_interop_abort_policy_requires_abort_strategy_after_trust`
- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs::package_rust_interop_abort_policy_accepts_trust_and_abort_strategy`

The policy is accepted only when package trust and the selected Cargo panic strategy both opt into `abort`.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `explicit_abort_trust_and_strategy_declared` runs `package_rust_interop_abort_policy_accepts_trust_and_abort_strategy` in `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `recoverable_abort_profile_rejected` runs `package_rust_interop_abort_policy_requires_abort_strategy_after_trust` in `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
