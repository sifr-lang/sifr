# callback_subscription_core

This fixture family tracks the stdlib-owned callback subscription declaration
policy needed by signal-style subscriptions.

- Positive evidence: `signal_subscription_cancel_shutdown` validates a
  thread-safe callback declaration that returns an async-close opaque handle
  and carries bounded cancellation/shutdown policy.
- Negative evidence: `invalid_subscription_callback_policy_rejected` proves
  subscription callbacks must declare bounded backpressure, overflow, and
  shutdown policy before they can cross the callback boundary.
- Compatibility category: `supported` for the stated contract-only scope.
  Runtime subscription lifecycle and ecosystem crates remain in runtime
  certification and are not claimed by this row.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `signal_subscription_cancel_shutdown` runs `package_rust_interop_accepts_callback_policy_contract` in `crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `invalid_subscription_callback_policy_rejected` runs `package_rust_interop_rejects_callback_missing_shutdown` in `crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
