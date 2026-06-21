# same_workspace_crate

Same-workspace crates are ordinary Cargo dependencies. The compiler does
not invent workspace fallback resolution.

- Positive evidence: direct dependency metadata with a local manifest path is
  probed successfully through the same direct Cargo dependency machinery as
  registry crates
  (`rust_interop_tests::package_rust_interop_resolves_same_workspace_path_dependency`).
- Negative evidence: undeclared roots continue to produce
  `SIFR-RUST-RESOLVE-0001`
  (`rust_interop_tests::package_rust_interop_rejects_unknown_target_root`).
