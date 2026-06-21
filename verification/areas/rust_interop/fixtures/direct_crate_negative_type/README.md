# Direct Crate Negative Type Evidence

This fixture row is covered by focused driver diagnostics for unsupported
Sifr-facing Rust bridge type contracts.

- Positive rejection coverage: `cargo test -p sifr_driver rust_interop` emits
  `SIFR-RUST-TYPE-0001` for unsupported bridge type contracts such as `set[int]`.
- Negative silent-compile coverage: the same test fails before direct Cargo
  probing or final generated binary materialization, so unsupported containers do
  not fall through to raw Rust build failures.
