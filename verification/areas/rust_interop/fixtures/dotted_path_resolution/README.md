# dotted_path_resolution

This fixture family tracks Rust interop target parsing and root resolution.

- Positive evidence: `valid_structured_paths` passes for structured dotted
  decorator paths such as dependency roots, `bridge.*`, shared bridge roots, and
  valid `Self.*` method targets.
- Negative evidence: `string_and_reserved_root_rejection` passes for string
  targets, legacy `crate=`/`path=` syntax, and reserved-root misuse.
- Compatibility category: `supported`. Rust interop target paths use AST-owned dotted paths, not strings.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `valid_structured_paths` runs `rust_interop_lowers_function_decorators_into_hir` in `crates/sifr_lowering/src/lower/rust_interop_tests.rs` through the blocking `sifr_lowering` suite at the `create-pr` profile.
- Negative `string_and_reserved_root_rejection` runs `rust_interop_rejects_string_target` in `crates/sifr_lowering/src/lower/rust_interop_tests.rs` through the blocking `sifr_lowering` suite at the `create-pr` profile.
