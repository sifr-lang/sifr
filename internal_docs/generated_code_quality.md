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

`scripts/run_all_tests.sh --profile merge` includes a clearly named
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

## Post-Closure Audit Wave 2 (2026-05-14)

The second emitted-code audit wave rechecked all non-negative demos and all
top-level LeetCode fixtures after removing additional generated-code artifacts:

- `while true` now optimizes to `loop`.
- `.skip(0)` iterator calls are removed from generated IR.
- `print("")` and empty generated `println` format macros render as
  `println!()`.
- Fallible bytes constructors now emit typed `Ok::<Vec<u8>, ...>(...)` results
  so standalone ignored results do not leave the Rust error type ambiguous.
- The generated clippy profile no longer allows `while_true`,
  `clippy::iter_skip_zero`, or `clippy::println_empty_string`.

Evidence:

- Demos post-patch sweep:
  `target/full_emitted_quality/demos-wave2-postpatch-1778765101/report.jsonl`.
- Demos failed-subset recheck after `pure_stdlib` demo cleanup:
  `target/full_emitted_quality/demos-wave2-failed-subset-after-pure-1778768309/report.jsonl`.
- Demos failed-subset recheck after bytes constructor typing:
  `target/full_emitted_quality/demos-wave2-failed-subset-after-bytes-1778769453/report.jsonl`.
- LeetCode post-patch sweep:
  `target/full_emitted_quality/leetcode-wave2-postpatch-1778766274/report.jsonl`.
- Reduced-allowlist generated clippy gate:
  `target/sifr_generated_code_quality/evidence/clippy-1778769689-83126.json`.
- Review rounds:
  `reviews/phase34-demo-leetcode-emitted-audit-wave2-review-1.md`,
  `reviews/phase34-demo-leetcode-emitted-audit-wave2-review-2.md`, and
  `reviews/phase34-demo-leetcode-emitted-audit-wave2-review-3.md`.

Result:

- Demos: 259 entries reach generated Rust and pass build, forbidden scan,
  `cargo fmt`, `cargo fmt --check`, and generated clippy. The remaining 13
  entries fail before emitted-code quality due to frontend/type/demo-contract
  gaps.
- LeetCode: 377 entries reach generated Rust and pass the emitted-code quality
  gates. The remaining 34 fail before emitted-code quality due to
  frontend/type/lowering compatibility gaps.

## Post-Closure Audit Wave 3 (2026-05-14)

The third emitted-code audit wave expanded the demo sweep to every
`demos/**/main.sifr` entry, including negative demo cases, and rechecked all
top-level LeetCode fixtures. It removed two additional generated Rust artifacts:

- Known `Decimal::checked_div(...).map_or_else(default, |value| value)` calls
  now optimize to `unwrap_or_else(default)`, removing a generated clippy failure
  in the decimal division negative demo without rewriting unknown receivers.
- Boolean literal comparisons now simplify during IR optimization, for example
  `flag == false` becomes `!flag` and `flag != false` becomes `flag`.
- The generated clippy profile no longer allows `clippy::bool_comparison`.

Evidence:

- All-demo pre-patch sweep:
  `target/full_emitted_quality/demos-wave3-all-1778776830/report.jsonl`.
- Demo failed-subset recheck after optimizer cleanup:
  `target/full_emitted_quality/demos-wave3-failed-subset-post-bool-map-1778779394/report.jsonl`.
- LeetCode full sweep:
  `target/full_emitted_quality/leetcode-wave3-all-1778778208/report.jsonl`.
- LeetCode boolean-comparison subset recheck:
  `target/full_emitted_quality/leetcode-wave3-bool-subset-post-bool-map-1778779466/report.jsonl`.
- Reduced-allowlist generated clippy gate:
  `target/sifr_generated_code_quality/evidence/clippy-1778780702-5147.json`.

Result:

- Demos: 261 entries reach generated Rust and pass build, forbidden scan,
  `cargo fmt`, `cargo fmt --check`, and generated clippy. The remaining 49
  entries fail before emitted-code quality due to expected negative demo
  diagnostics or frontend/type/demo-contract gaps.
- LeetCode: 377 entries reach generated Rust and pass the emitted-code quality
  gates. The 29 fixtures that previously emitted boolean literal comparisons
  were rechecked after the optimizer cleanup and now pass with zero remaining
  `== true`, `== false`, `!= true`, or `!= false` occurrences.

## NeetCode Group Audit Wave

The NeetCode-oriented audit reviewed fixtures by README problem group, then
reran the full demo and LeetCode emitted-code corpora. One generated Rust
blocker was found in the Trees group and fixed in compiler codegen:

- `map(treeToString, nodes)` now emits a closure that applies optional widening
  and borrowing for typed callable arguments instead of a direct function
  pointer call when the iterable element is `T` and the callable parameter is
  `T | None`.
- The fixed Trees fixture now emits
  `.map(|__sifr_map_item| treeToString(&Some(__sifr_map_item)))`.

Evidence:

- Trees post-fix group rerun:
  `target/neetcode_group_quality/trees-post-map-1778787311/report.jsonl`.
- Final demos full scan:
  `target/full_emitted_quality/demos-neetcode-final-1778787559/report.jsonl`.
- Final LeetCode full scan:
  `target/full_emitted_quality/leetcode-neetcode-final-1778788537/report.jsonl`.
- Claude review artifacts:
  `reviews/phase34-neetcode-group-01-arrays-hashing-review.md`,
  `reviews/phase34-neetcode-group-02-two-pointers-review.md`,
  `reviews/phase34-neetcode-groups-03-through-18-review.md`, and
  `reviews/phase34-neetcode-trees-map-fix-review.md`.
- Final closing review:
  `reviews/phase34-neetcode-final-review.md`.

Result:

- Trees group: 32/32 mapped fixtures reach generated Rust and pass build,
  forbidden scan, `cargo fmt`, `cargo fmt --check`, generated clippy, and
  fixed-pattern regression scans.
- Demos: 261 entries pass the emitted-code gate; 49 fail before emitted Rust
  quality due to expected negative diagnostics or frontend/type/demo-contract
  gaps.
- LeetCode: 378 fixtures pass the emitted-code gate; 33 remain pre-emission
  frontend/type/lowering failures.
- Final fixed-pattern scans report zero boolean literal comparisons, zero
  identity `map_or_else`, zero `while true`, zero `.skip(0)`, and zero
  `println!("")`.
