# async_runtime_core

Runtime-observed fixture family for the stdlib-owned async runtime core needed
by native stdlib resource migrations.

- Positive evidence: `stdlib_async_resource_lifecycle` is represented by
  executable async declaration and lifecycle tests:
  `rust_interop_async_contract_tests::package_rust_interop_direct_probe_accepts_async_signature`,
  `rust_interop_async_contract_tests::package_rust_interop_async_current_thread_allows_non_send_future`,
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
- Scope note: this row certifies only stdlib-owned async declarations,
  async-close lifecycle contracts, cancellation/drop task semantics, panic
  conversion through declared `RustPanicError` surfaces, and hidden-blocking
  rejection. Loopback `tokio`/`reqwest` behavior remains tracked by
  `async_runtime_reqwest`.
