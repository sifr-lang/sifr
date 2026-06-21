# direct_crate_crc32

This fixture row covers a direct Cargo dependency binding with no
package-local bridge module.

- Positive evidence: `package_project_build_check::test_build_cached_package_project_links_direct_rust_interop_dependency`
  builds a Sifr package that declares `@rust(crc32fast.hash,
  panic=trusted_no_panic)`, links a direct `crc32fast` Cargo dependency through
  the generated binary project, runs the binary, and observes the Rust function
  result.
- Negative evidence: `rust_interop_contract_tests::package_rust_interop_direct_non_result_requires_panic_policy`
  rejects a direct non-`Result` Rust binding unless the declaration supplies an
  explicit panic policy and matching trust evidence.
- Manual smoke evidence for the registry crate: `cargo run -q -p sifr --
  run src/main.sifr` from a temporary package using `crc32fast = "1.5.0"`
  printed `3421780262` for `b"123456789"` with explicit
  `rust-build-scripts = ["crc32fast"]` and
  `rust-no-panic = ["crc32fast.hash"]` trust entries.
