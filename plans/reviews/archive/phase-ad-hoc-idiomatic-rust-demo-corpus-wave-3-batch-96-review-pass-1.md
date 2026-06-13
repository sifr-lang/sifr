# Wave 3 Batch 96 Review Pass 1

- `demos/import_forms/negative_cases/idiomatic.rs`
  - OK: scaffold correctly records the unsupported bare-relative, unsupported plain `import`, and unsupported level-2 relative import contracts, including their reachable unresolved-name follow-on diagnostics.
- `demos/resolver_triggers/negative_cases/idiomatic.rs`
  - OK: scaffold correctly documents that the same unsupported import forms must not silently trigger project-mode resolution.
- `demos/stdlib_modules/negative_cases/idiomatic.rs`
  - OK: scaffold correctly documents the `_sifr.*` intrinsic-import ban and the reachable undefined `sqrt` follow-on diagnostic.

Result: `OK` for all three files. No blockers.
