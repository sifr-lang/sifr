# Wave 2 Batch 22 Review Pass 1

## Scope

- `demos/iteration_basics/idiomatic.rs`
- `demos/iterator_builtins/idiomatic.rs`
- `demos/iterators_and_comprehensions/idiomatic.rs`

## Review Method

- External per-file review using embedded paired Sifr and Rust sources.
- Reviewer prompts were constrained to actionable correctness, parity, and misleading-reference issues only.
- Per-file prompts were used because recent full-batch prompts have been less reliable in this workspace.

## Results

### `iteration_basics`

- No actionable issues found.

### `iterator_builtins`

- Accepted a reference-quality follow-up: repeated traversals now use borrowed iterators and the helper was reshaped from a one-off descending helper into `sorted(values, reverse)` so the Rust companion mirrors the Sifr `sorted(..., reverse=True)` surface more directly.
- A claimed move error on `nums.into_iter()` was not accepted as the root issue because the original file had already compiled successfully in this workspace; the accepted change was taken anyway because it clarifies the intended non-consuming iterator behavior.

### `iterators_and_comprehensions`

- Accepted a reference-quality follow-up: repeated passes over `nums`, `unsorted`, `letters`, and `bools` now use borrowed iterators so the Rust companion makes the same reusable-sequence semantics explicit as the Sifr source.
- The reviewer framed this as a move/use-after-consume problem, but the accepted fix was applied for parity clarity rather than because the original array-based code was invalid.

## Validation After Accepted Changes

- `rustfmt demos/iteration_basics/idiomatic.rs demos/iterator_builtins/idiomatic.rs demos/iterators_and_comprehensions/idiomatic.rs`
- Standalone `rustc` validation for `demos/iterator_builtins/idiomatic.rs`
- Standalone `rustc` validation for `demos/iterators_and_comprehensions/idiomatic.rs`
- `cargo run -q -p sifr -- run demos/iterator_builtins/main.sifr`
- `cargo run -q -p sifr -- run demos/iterators_and_comprehensions/main.sifr`
- `scripts/run_all_tests.sh`

## Status

- Pass 1 complete.
- Accepted notes were applied and revalidated.
- Batch advanced to pass 2 on the updated code state.
