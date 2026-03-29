# Wave 2 Batch 31 Review Pass 1

## Scope

- `demos/optional_indexing/idiomatic.rs`
- `demos/optional_arithmetic/idiomatic.rs`
- `demos/return_type_inference/idiomatic.rs`

## Review Method

- External per-file review using embedded paired Sifr and Rust sources.
- Reviewer prompts were constrained to actionable correctness, parity, and misleading-reference issues only.
- Per-file prompts were used because they have been more reliable than larger batch prompts in this workspace.

## Results

### `optional_indexing`

- No actionable issues found.

### `optional_arithmetic`

- No actionable issues found.

### `return_type_inference`

- No accepted blockers.
- The pass-1 note claimed `greet("sifr")` should use `{}` instead of `{:?}`, but that was not accepted because the paired Sifr demo in this workspace prints `"hello sifr"` with quotes, so the Rust debug-format output is the actual parity-preserving choice for this companion.

## Validation

- `rustfmt demos/optional_indexing/idiomatic.rs demos/optional_arithmetic/idiomatic.rs demos/return_type_inference/idiomatic.rs`
- standalone `rustc` validation for all three companions
- targeted Sifr demo runs for all three demos
- `scripts/run_all_tests.sh`

## Status

- Pass 1 complete.
- No code changes were required.
- Batch advanced to pass 2 on the current validated code state.
