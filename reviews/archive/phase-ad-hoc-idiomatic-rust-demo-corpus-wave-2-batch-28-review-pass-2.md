# Wave 2 Batch 28 Review Pass 2

## Scope

- `demos/borrow_by_default/idiomatic.rs`
- `demos/borrowed_builtins/idiomatic.rs`
- `demos/generic_cloning/idiomatic.rs`

## Review Method

- External per-file review using embedded paired Sifr and Rust sources.
- Reviewer prompts were constrained to actionable correctness, parity, and misleading-reference issues only.
- Per-file prompts were retained because larger combined prompts stalled repeatedly in this workspace during pass 2.

## Results

### `borrow_by_default`

- No accepted blockers.
- The pass-2 note again suggested changing `get_first_char` from `String` to `char`, but that was not accepted because this corpus consistently models Sifr `str` as Rust `String`, and the current companion already matches the paired demo's observable behavior exactly.

### `borrowed_builtins`

- No actionable issues found.

### `generic_cloning`

- No actionable issues found.

## Status

- Pass 2 complete.
- No additional code changes were required.
- Batch is ready for docs, commit, PR, and merge on the current validated code state.
