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

## Post-Closure Full Corpus Audit (2026-05-14)

The follow-up emitted-code audit checked all non-negative demos and all LeetCode
fixtures one by one with generated Rust build, forbidden construct scan, official
`cargo fmt`, `cargo fmt --check`, and the generated-code clippy profile.

- Demos full sweep:
  `target/full_emitted_quality/demos-full-final3-1778757911/report.jsonl`.
- Demos failed-subset recheck:
  `target/full_emitted_quality/demos-failed-subset-final-1778759486/report.jsonl`.
- LeetCode full sweep:
  `target/full_emitted_quality/leetcode-1778753354/report.jsonl`.
- LeetCode failed-subset recheck:
  `target/full_emitted_quality/leetcode-failed-subset-final-1778756628/report.jsonl`.
- Fresh generated clippy gate:
  `target/sifr_generated_code_quality/evidence/clippy-1778760229-33909.json`.

Result:

- Demos: 257 entries currently reach generated Rust and pass the emitted-code
  quality gates. The 15 remaining failures stop before emitted-code quality due
  to frontend/type/demo-contract gaps.
- LeetCode: 363 entries currently reach generated Rust and pass the emitted-code
  quality gates. The 48 remaining failures stop before emitted-code quality due
  to frontend/type/lowering compatibility gaps.
- Review rounds are recorded in
  `reviews/phase34-demo-leetcode-emitted-audit-review-1.md`,
  `reviews/phase34-demo-leetcode-emitted-audit-review-2.md`, and
  `reviews/phase34-demo-leetcode-emitted-audit-review-3.md`.
