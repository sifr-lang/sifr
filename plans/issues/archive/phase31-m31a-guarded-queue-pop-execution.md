# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 11

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_11_guarded_queue_pop_narrowing`

## Goal

Close remaining non-empty queue/pop optional leakage by narrowing safe guarded pop shapes beyond zero-arg `pop()`.

## Root Cause

- Guarded non-empty pop narrowing only covered zero-arg `pop()` / `popleft()`.
- Queue-style safe pop reads (`pop(0)` under non-empty guard, and deque `popleft()` under non-empty guard) still leaked `None`.
- This left `0127_word_ladder` with `None | T`-driven comparison/len failures even after slice 10.

## Implementation

- Extended guarded non-empty pop narrowing in HIR to safe additional shapes:
  - `list.pop(0)` under non-empty guard narrows `T | None` -> `T`
  - deque `pop`/`popleft` under non-empty guard now narrows `T | None` -> `T`
  - file: `crates/sifr_hir/src/lower/nonempty_method_narrowing.rs`
- Kept unsafe indexed pop shapes non-narrowed:
  - `list.pop(i)` for non-zero/non-literal indices remains optional
  - file: `crates/sifr_hir/src/lower/expressions_tests.rs`
- Extended codegen non-optional pop bridge to match the widened HIR domain:
  - applies invariant `let Some(...) = ... else { unreachable!(...) }` extraction for narrowed pop shapes
  - file: `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- Added codegen regression for guarded `pop(0)`:
  - file: `crates/sifr_codegen/src/lib_codegen_tests.rs`
- Expanded demo coverage to include both `pop()` and `pop(0)`:
  - `demos/phase31_pop_guard_narrowing_demo.sifr`

## Targeted Cases

- `0053`, `0127`, `0322`, `0502`, `0743`, `0746`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave11_guarded_queue_pop_results.json`

Status counts:

- `CHECK_ERROR=6` (count unchanged)

Meaningful movement:

- `0127_word_ladder` no longer reports optional pop leakage:
  - `cannot compare 'None | T' and 'str'` -> `cannot compare 'T' and 'str'`
  - `len(... got 'None | T')` -> `len(... got 'T')`

Remaining `0127` blockers are now canonical mutability plus generic/type precision follow-ons, not guarded queue-pop optional flow.

## Validation

Targeted validation:

- `cargo test -p sifr_hir test_guarded_list_pop_narrows_to_element_type`
- `cargo test -p sifr_hir test_guarded_zero_index_list_pop_narrows_to_element_type`
- `cargo test -p sifr_hir test_unguarded_zero_index_list_pop_stays_optional`
- `cargo test -p sifr_hir test_guarded_indexed_list_pop_stays_optional`
- `cargo test -p sifr_codegen test_generate_rust_guarded_list_pop_unwraps_compiler_verified_nonempty`
- `cargo test -p sifr_codegen test_generate_rust_guarded_list_pop_zero_unwraps_compiler_verified_nonempty`
- `cargo run -q -p sifr -- check audits/leetcode/0127_word_ladder.sifr`
- `cargo run -q -p sifr -- run demos/phase31_pop_guard_narrowing_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave11_guarded_queue_pop_results.json --case 0053 --case 0127 --case 0322 --case 0502 --case 0743 --case 0746`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 11 is complete because safe guarded queue-pop shapes (`pop(0)` and deque non-empty pop) are now narrowed end-to-end in HIR and codegen, and the seed-corpus reclassification confirms optional-pop leakage removal in `0127`.
