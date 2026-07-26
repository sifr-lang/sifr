# Rust Interop Thread-Safe Callback Policy

This fixture family tracks the contract-only thread-safe callback policy surface
for Rust interop callback contracts.

Current passing evidence:

- `bounded_callback_policy`: covered by
  `crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs::package_rust_interop_accepts_callback_policy_contract`
  and
  `crates/sifr_codegen/src/rust_interop_plan_tests.rs::interop_bridge_callable_params_require_callback_contract`.
- `missing_backpressure_rejected`: covered by
  `crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs::package_rust_interop_rejects_callback_missing_backpressure`.

Signal-style stdlib subscriptions are tracked by `callback_subscription_core`.
Runtime-observed ecosystem subscription, cancellation, shutdown, and
cross-thread capture cases remain in `callback_subscription_ecosystem`.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `bounded_callback_policy` runs `package_rust_interop_accepts_direct_callback_backpressure` in `crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `missing_backpressure_rejected` runs `package_rust_interop_rejects_callback_missing_backpressure` in `crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
