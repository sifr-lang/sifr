# Wave 2 Batch 28 Review Pass 1

## Scope

- `demos/borrow_by_default/idiomatic.rs`
- `demos/borrowed_builtins/idiomatic.rs`
- `demos/generic_cloning/idiomatic.rs`

## Review Method

- External per-file review using embedded paired Sifr and Rust sources.
- Reviewer prompts were constrained to actionable correctness, parity, and misleading-reference issues only.
- Per-file prompts were used because they have been more reliable than larger batch prompts in this workspace.

## Results

### `borrow_by_default`

- No accepted blockers.
- The pass-1 note asked to change `get_first_char` from `String` to `char`, but that was not accepted because this corpus consistently models Sifr `str` as Rust `String`, and the current companion already matches the paired demo's observable behavior exactly.

### `borrowed_builtins`

- No actionable issues found.

### `generic_cloning`

- No actionable issues found.

## Validation

- `rustfmt demos/borrow_by_default/idiomatic.rs demos/borrowed_builtins/idiomatic.rs demos/generic_cloning/idiomatic.rs`
- standalone `rustc` validation for all three companions
- targeted Sifr demo runs for all three demos
- `scripts/run_all_tests.sh`

## Status

- Pass 1 complete.
- No code changes were required.
- Batch advanced to pass 2 on the current validated code state.
