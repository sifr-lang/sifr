# shared_bridge_crate

Shared bridge crates are ordinary direct Cargo dependencies with a
package-boundary restriction: shared crates must not import package-specific
`crate::__sifr_bridge` modules.

- Positive evidence: comments, strings, and related identifiers mentioning
  `__sifr_bridge` do not violate the boundary
  (`rust_interop_tests::package_rust_interop_allows_shared_bridge_comments_about_generated_bridge_types`).
- Negative evidence: a local shared dependency source file importing
  `crate::__sifr_bridge::*` is rejected before Cargo execution
  (`rust_interop_tests::package_rust_interop_rejects_shared_bridge_crate_importing_generated_bridge_types`).
