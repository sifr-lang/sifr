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
  which reject incompatible Rust handle targets before generated application
  build.
- Contract validation coverage: `package_rust_interop_opaque_rejects_unknown_contract_key`
  rejects keys outside the fixed opaque decorator surface.

Full generated opaque resource wrappers and method lowering remain tracked by
`opaque_resource_matrix`.
