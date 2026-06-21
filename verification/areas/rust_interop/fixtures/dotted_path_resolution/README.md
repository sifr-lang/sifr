# dotted_path_resolution

This fixture family tracks Rust interop target parsing and root resolution.

- Positive evidence: `valid_structured_paths` passes for structured dotted
  decorator paths such as dependency roots, `bridge.*`, shared bridge roots, and
  valid `Self.*` method targets.
- Negative evidence: `string_and_reserved_root_rejection` passes for string
  targets, legacy `crate=`/`path=` syntax, and reserved-root misuse.
- Compatibility category: `supported`. Rust interop target paths use AST-owned dotted paths, not strings.
