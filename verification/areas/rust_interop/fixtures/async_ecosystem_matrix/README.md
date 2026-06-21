# async_ecosystem_matrix

Cargo-probe fixture family for async Rust bridge signatures.

- Positive evidence: `current_thread_non_send_future` is represented by
  `package_rust_interop_async_current_thread_allows_non_send_future`, which
  permits a non-`Send` Rust future only when the declaration uses
  `thread_affinity=tokio_current_thread`.
- Negative evidence: `non_send_future_without_affinity` is represented by
  `package_rust_interop_async_requires_send_future_by_default`, which maps the
  default `Send` future obligation to `SIFR-RUST-ASYNC-0001`.
