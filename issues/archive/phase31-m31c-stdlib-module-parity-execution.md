# Phase 31 Follow-up: `m31_c_stdlib_module_parity`

Status: in_progress
Started: 2026-03-11
Current slice: `m31_c_slice_1_python_module_attr_and_truthiness`

## Goal

Remove the first root-cause blockers inside `stdlib.python_module_surface` without introducing fallback semantics or weakening type/runtime guarantees.

## Targeted Cases

- `0003` `longest_substring_without_repeating_characters`
- `0007` `reverse_integer`
- `0127` `word_ladder`
- `0217` `contains_duplicate`
- `0502` `ipo`
- `1046` `last_stone_weight`

## Slice 1 Scope

- Python-style stdlib module/member compatibility for direct calls such as `math.fmod(...)`
- Synthetic stdlib imports wired into lowering and codegen without dead-code-eliminating required runtime items
- Numeric truthiness condition lowering (`if x`, `while x`, `if not x`) for integer/float values
- Runtime parity for `math.fmod` sign behavior on negative dividends

## Implemented Root-cause Fixes

- Added synthetic compatibility imports during lowering for supported Python-style stdlib attribute calls.
- Registered synthetic import aliases in the lowering/type environment so compatibility calls type-check as normal function/class references.
- Fixed stdlib prescan filtering so compatibility aliases do not cause required stdlib Rust items to be dropped.
- Kept `__compat_*` calls off the leaf-expression fast path so they lower through the structured intrinsic path.
- Canonicalized math compatibility aliases back to their intrinsic export names during intrinsic lowering.
- Corrected `math.fmod` lowering to use remainder semantics instead of `rem_euclid`, matching Python `math.fmod`.
- Added numeric truthiness lowering for condition positions so `while x` and `if x` on numeric values emit valid Rust comparisons.
- Relaxed `int -> float` assignability and boolean-operator truthiness enough to support the compatibility slice without adding fallback behavior.

## Regression Coverage

- `crates/sifr/tests/e2e/pass/phase31_python_module_attr_compat.sifr`
- `crates/sifr_codegen/src/lower_expr.rs` test: `compat_stdlib_alias_calls_stay_off_plain_call_fast_path`
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs` test: `canonicalizes_math_compat_intrinsic_aliases`
- `crates/sifr_codegen/src/lower_stmt.rs` numeric-truthiness regression coverage

## Validation Evidence

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase31_python_module_attr_compat.sifr`
- `cargo run -q -p sifr -- run audit/leetcode/0007_reverse_integer.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31c_wave1_results.json --case 0003 --case 0007 --case 0127 --case 0217 --case 0502 --case 1046`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Current Measured Outcome

Artifact:
- `verification/leetcode/phase31_m31c_wave1_results.json`

Status counts for the targeted six-case slice after this work:
- `PASS=1`
- `CHECK_ERROR=5`
- `RUN_ERROR=0`

Observed movement:
- `0007` is now a full `PASS`.
- `0502` no longer fails on missing `heapq`; it now fails on deeper type/destructuring issues, which means the stdlib surface blocker has been peeled back.

Remaining slice-local blockers:
- `0003`, `0217`: missing `set(...)` constructor compatibility
- `0127`: missing `deque(...)` / `collections` constructor compatibility plus follow-on local typing cleanup
- `1046`: remaining `heapq`/`Any` surface cleanup and annotation-driven type recovery
- `0502`: reclassified downstream blockers outside pure stdlib surface work

## Next Slice Candidates

1. Add constructor compatibility for `set(...)` and `collections.deque(...)`.
2. Finish heap-oriented module/member parity for `heapq` call patterns that still surface as undefined symbols.
3. Re-run the targeted six-case slice and update this report with the next delta before opening the next PR.

## External Review Follow-up

- `2026-03-11` review pass 1 (`reviews/phase-31-m31c-slice1-review-pass-1.md`) was validated against the merged code.
- Accepted finding:
  - add `BigInt` truthiness parity to condition lowering and boolean-operator type checking
- Deferred as next functional slice, not review-fix scope:
  - broader numeric-expression truthiness beyond simple names (`call`, `binop`, `field access`)
