# async_runtime_core

Contract-only fixture family for the stdlib-owned async declaration and
resource-lifecycle rules needed by native stdlib resource migrations.

- Positive evidence: `stdlib_async_resource_lifecycle` is represented by
  async declaration and lifecycle contract tests:
  `rust_interop_async_contract_tests::package_rust_interop_direct_probe_accepts_async_signature`,
  `rust_interop_async_contract_tests::package_rust_interop_async_probe_current_thread_allows_non_send_future`,
  `rust_interop_async_contract_tests::package_rust_interop_opaque_current_thread_clears_async_method_send_probe`,
  `rust_interop_contract_tests::package_rust_interop_opaque_async_close_policy_accepts_async_aclose_contract`,
  `rust_interop_contract_tests::package_rust_interop_opaque_async_close_policy_requires_async_aclose_contract`,
  `rust_interop_contract_tests::package_rust_interop_opaque_async_close_policy_rejects_sync_close_only_contract`,
  `async_task_runtime_codegen_tests::test_task_handle_join_lowers_to_task_result_observation`,
  `async_task_runtime_codegen_tests::test_await_task_handle_desugars_to_join_observation`,
  `async_task_runtime_codegen_tests::test_task_handle_cancel_borrows_handle_and_aborts_child`,
  `async_task_runtime_codegen_tests::test_task_timeout_handle_lowers_to_private_timeout_result`,
  and `sifr_runtime::interop::tests::async_handle_close_and_cancel_join_are_deterministic`.
- Negative evidence: `stdlib_async_hidden_blocking_rejected` is represented by
  `rust_interop_async_contract_tests::package_rust_interop_async_rejects_unsupported_thread_affinity`,
  `rust_interop_async_contract_tests::package_rust_interop_async_requires_send_future_by_default`,
  and `blocking_diagnostics` evidence that rejects hidden blocking/CPU-heavy
  effects on async Rust interop declarations.
- Panic and poisoning evidence is represented by
  `sifr_runtime::interop::tests::handle_poison_guard_marks_open_handle_when_rust_call_unwinds`,
  `sifr_runtime::interop::tests::poisoned_state_wins_over_closed_state`, and
  `sifr_runtime::interop::tests::catch_rust_panic_redacts_payload_details`.
- Scope note: this row certifies only compile-time stdlib-owned async
  declarations, async-close lifecycle contracts, current-thread affinity,
  panic-surface declarations, and hidden-blocking rejection. Runtime task
  cancellation/drop and loopback `tokio`/`reqwest` behavior remain tracked by
  runtime certification.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `stdlib_async_resource_lifecycle` runs `package_rust_interop_opaque_current_thread_clears_async_method_send_probe` in `crates/sifr_driver/src/build/rust_interop_async_contract_tests.rs` through the blocking `sifr_driver_lib` suite at the `create-pr` profile.
- Negative `stdlib_async_hidden_blocking_rejected` runs `rust_interop_hidden_blocking_async_resource_evidence_is_rejected` in `crates/sifr_lowering/src/lower/rust_interop_tests.rs` through the blocking `sifr_lowering` suite at the `create-pr` profile.
