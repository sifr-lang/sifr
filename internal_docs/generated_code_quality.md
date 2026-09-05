# Generated Code Quality

generated-code quality gate treats generated Rust as a product artifact. The generated-code quality
corpus is defined by
`verification/areas/generated_code_quality/data/corpus_manifest.json` and run
through the `generated_code_quality` verification area at
`verification/areas/generated_code_quality/`.

## Quality Gates

- `uv run --project verification --locked python -m sifr_verify areas run --area generated_code_quality --suite corpus`
  validates the corpus manifest, materializes each selected entry as an
  isolated Cargo crate under `target/sifr_generated_code_quality/<run-id>/`,
  and runs `cargo check`.
- `uv run --project verification --locked python -m sifr_verify areas run --area generated_code_quality --suite panic-scan`
  rejects generated `.rs` files that contain `.unwrap(`, `.expect(`, `panic!`,
  `todo!`, `unimplemented!`, `unsafe`, or gate-suppressing `#[allow(...)]`
  attributes.
- `uv run --project verification --locked python -m sifr_verify areas run --area generated_code_quality --suite rustfmt`
  proves the format negative seed fails, formats transient generated crates,
  and verifies `cargo fmt -- --check`.
- `uv run --project verification --locked python -m sifr_verify areas run --area generated_code_quality --suite clippy`
  proves the lint negative seed fails and runs `cargo clippy -- -D warnings`
  with a narrow generated-code allowlist for known style debt (for example
  Python-compatible lowercase type names and whole-stdlib dead code) on
  transient generated crates after formatting.
- `uv run --project verification --locked python -m sifr_verify areas run --area generated_code_quality --suite determinism`
  verifies byte-stable repeated `emit` output for every positive manifest entry
  and proves a seeded mismatch fails.
- `uv run --project verification --locked python -m sifr_verify areas run --area generated_code_quality --suite demos`
  runs the required generated-code quality gate demo entries.

Successful runs write evidence JSON to
`target/sifr_generated_code_quality/evidence/`. Failed runs preserve their
transient generated project roots for inspection.

## PR Lane Integration

`scripts/run_all_tests.sh --profile merge` includes a clearly named
`Generated Code Quality Checks` step that runs all generated-code quality
checks through `sifr_verify areas run --area generated_code_quality --suite
representative`. The same area runner is used locally and in CI.
