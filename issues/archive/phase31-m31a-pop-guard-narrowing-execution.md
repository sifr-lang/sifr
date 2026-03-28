# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 10

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_10_guarded_pop_popleft_narrowing`

## Goal

Eliminate optional-flow leakage from `pop`/`popleft` when control flow already proves the sequence is non-empty.

## Root Cause

- Method typing returned `T | None` for `pop`/`popleft` unconditionally.
- Non-empty flow guards (`while seq:`, truthiness guards) were not applied to method-return refinement.
- This leaked `None` into downstream comparisons and `len(...)` calls in cases like `0127`.
- After narrowing, codegen still emitted raw `Vec::pop()`/deque pop calls (`Option<T>`), which caused Rust type mismatches in stdlib paths (for example `stdlib_configparser`).

## Implementation

- Added guarded refinement after method return-type resolution:
  - when receiver is a named sequence with active non-empty guard, `pop`/`popleft` narrows to non-optional `T`
  - file: `crates/sifr_hir/src/lower/expressions.rs`
- Restricted narrowing to the safe domain only:
  - receiver type must be `list`/deque storage (`Type::List(_)`)
  - method must be zero-arg `pop()` or `popleft()`
  - file: `crates/sifr_hir/src/lower/nonempty_method_narrowing.rs`
- Added codegen parity bridge for compiler-proven non-empty pop calls:
  - when HIR return type is non-optional, emit `let Some(...) = ... else { unreachable!(...) }` extraction
  - file: `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- Added regressions:
  - `test_guarded_list_pop_narrows_to_element_type`
  - `test_unguarded_list_pop_stays_optional`
  - `test_guarded_indexed_list_pop_stays_optional`
  - `test_guarded_dict_pop_stays_optional`
  - file: `crates/sifr_hir/src/lower/expressions_tests.rs`
- Added codegen regression:
  - `test_generate_rust_guarded_list_pop_unwraps_compiler_verified_nonempty`
  - file: `crates/sifr_codegen/src/lib_codegen_tests.rs`
- Added slice demo:
  - `demos/phase31_pop_guard_narrowing_demo.sifr`

## Targeted Cases

- `0053`, `0127`, `0322`, `0502`, `0743`, `0746`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave10_pop_guard_results.json`

Status counts:

- `CHECK_ERROR=6` (count unchanged)

Meaningful movement:

- `0127_word_ladder` no longer reports optional-pop leakage:
  - `cannot compare 'None | T' and 'str'` -> `cannot compare 'T' and 'str'`
  - `len(... got 'None | T')` -> `len(... got 'T')`

This confirms the optional-flow root cause for guarded pop/popleft is removed.

## Validation

Targeted validation:

- `cargo test -p sifr_hir test_guarded_list_pop_narrows_to_element_type`
- `cargo test -p sifr_hir test_unguarded_list_pop_stays_optional`
- `cargo test -p sifr_hir test_guarded_indexed_list_pop_stays_optional`
- `cargo test -p sifr_hir test_guarded_dict_pop_stays_optional`
- `cargo test -p sifr_codegen test_generate_rust_guarded_list_pop_unwraps_compiler_verified_nonempty`
- `cargo run -q -p sifr -- run demos/phase31_pop_guard_narrowing_demo.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0127_word_ladder.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave10_pop_guard_results.json --case 0053 --case 0127 --case 0322 --case 0502 --case 0743 --case 0746`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 10 is complete because guarded pop/popleft now composes with non-empty flow proofs and no longer leaks `None` in the targeted corpus. Remaining failures are narrower non-optional-flow follow-ons.
