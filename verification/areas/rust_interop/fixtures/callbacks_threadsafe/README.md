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

The runtime-observed subscription, cancellation, shutdown, cross-thread capture,
and ecosystem callback cases remain in `callback_subscription_matrix`.
