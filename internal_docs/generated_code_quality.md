# Generated Code Quality

Phase 34 treats generated Rust as a product artifact. The generated-code quality
gate is defined by `verification/generated_code_quality/manifest.json` and run
through the scripts in `verification/generated_code_quality/`.

## Quality Gates

- `generated_code_quality_corpus.sh` validates the manifest, materializes each
  selected entry as an isolated Cargo crate under
  `target/sifr_generated_code_quality/<run-id>/`, and runs `cargo check`.
- `generated_code_quality_panic_scan.sh` rejects generated `.rs` files that
  contain `.unwrap(`, `.expect(`, `panic!`, `todo!`, `unimplemented!`, `unsafe`,
  or gate-suppressing `#[allow(...)]` attributes.
- `generated_code_quality_rustfmt.sh` proves the format negative seed fails,
  formats transient generated crates, and verifies `cargo fmt -- --check`.
- `generated_code_quality_clippy.sh` proves the lint negative seed fails and
  runs `cargo clippy -- -D warnings` with a narrow generated-code allowlist for
  known style debt (for example Python-compatible lowercase type names and
  whole-stdlib dead code) on transient generated crates after formatting.
- `generated_code_quality_determinism.sh` verifies byte-stable repeated `emit`
  output for every positive manifest entry and proves a seeded mismatch fails.
- `generated_code_quality_demos.sh` runs the required Phase 34 demo entries.

Successful runs write evidence JSON to
`target/sifr_generated_code_quality/evidence/`. Failed runs preserve their
transient generated project roots for inspection.

## PR Lane Integration

`scripts/run_all_tests.sh --profile pr` includes a clearly named
`Generated Code Quality Checks` step that runs all generated-code quality
scripts. The same scripts are used locally and in CI.

## Phase 34 Closure Evidence

Latest local evidence recorded during closure:

- Corpus: `target/sifr_generated_code_quality/evidence/corpus-1778726430-5910.json`
- Panic scan: `target/sifr_generated_code_quality/evidence/panic-scan-1778726771-44667.json`
- Rustfmt: `target/sifr_generated_code_quality/evidence/rustfmt-1778727026-70597.json`
- Clippy: `target/sifr_generated_code_quality/evidence/clippy-1778727293-1964.json`
- Determinism: `target/sifr_generated_code_quality/evidence/determinism-1778727645-43530.json`
- Demos: `target/sifr_generated_code_quality/evidence/corpus-1778727829-54351.json`
