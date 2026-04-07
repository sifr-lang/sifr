# Ad-hoc Phase Execution: Surface Parity And API Cleanup (2026-04-07)

Status: in_progress
Parent phase: `issues/ad-hoc-surface-parity-and-api-cleanup-2026-04-07.md`

## Scope Gate

Latest scoped categories from `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun2.json`:

- `python_stdlib_and_builtin_parity_gap`: `10`
- `other_type_surface_and_api_mismatch`: `11`
- `callable_argument_contract_mismatch`: `0`
- `destructuring_and_assignment_target_surface_gap`: `1`

Current scoped fixture total: `22`

## Workstream Order

1. `WS1` variadic builtin parity for `min` / `max`
2. `WS2` membership parity for `range` and compat mapping wrappers
3. `WS4` empty-container specialization repair
4. `WS3` iterator consumer stabilization and tuple heap comparability
5. `WS5` scoped codegen defect closure
6. `WS6` canonical Sifr adaptation sweep

## Workstream Exit Criteria

### WS1

- no scoped `min()/max() takes 1 or 2 arguments` diagnostics remain
- new targeted tests cover 3-arg and 4-arg scalar `min`/`max`

### WS2

- no scoped `range` membership diagnostics remain
- no scoped compat-map membership diagnostics remain
- targeted tests cover `in` and `not in` over `range`

### WS4

- `0290_word_pattern` no longer fails from dict specialization drift
- `1345_jump_game_iv` no longer fails from compat specialization drift

### WS3

- `0853_car_fleet` and `1834_single_threaded_cpu` no longer fail on iterator consumer parity
- compiler-owned part of `1851_minimum_interval_to_include_each_query` is closed
- tuple comparability tests exist for heap ordering

### WS5

- `0150`, `0297`, `1260`, `1383`, and `1498` no longer fail from generated Rust build/runtime breakage
- scoped run-stage failures are not being mislabeled as policy issues

### WS6

- adaptation-owned fixtures are canonicalized without language-policy relaxation
- remaining scoped failures, if any, are explicitly attributable to cross-bucket dependencies only

## Reviewer Artifacts

- `reviews/surface-parity-api-cleanup-review-subgroup1.md`
- `reviews/surface-parity-api-cleanup-review-tuple-comparable.md`
- `reviews/surface-parity-api-cleanup-review-subgroup2b.md`
- `reviews/surface-parity-api-cleanup-review-codegen-group.md`
- `reviews/surface-parity-api-cleanup-review-readiness.md`
- `reviews/surface-parity-api-cleanup-review-pass2.md`

## Execution Notes

- `0212_word_search_ii` stays in phase with dependency flags; do not count it as a clean closure until its secondary blockers are addressed.
- `1345_jump_game_iv`, `1851_minimum_interval_to_include_each_query`, and `2002_maximum_product_of_the_length_of_two_palindromic_subsequences` may still expose secondary categories after their in-phase blockers close.
- `callable_argument_contract_mismatch` is currently `0`; keep no dedicated workstream for it unless it reappears in the next full rerun.

## Work Log

- 2026-04-07: `WS1` implemented, validated, reviewed, and merged via PR `#1596`.

## Wave Log

- Wave WS1 (variadic builtin parity for `min` / `max`)
  - Compiler changes:
    - variadic scalar lowering for `min`/`max` in HIR (3+ operands)
    - pairwise variadic operand validation retained for optional/comparability checks
    - codegen lowering updated to emit nested Rust comparisons for variadic scalar forms
  - Tests:
    - `crates/sifr_hir/src/lower/expressions_tests.rs`
      - `test_min_max_accept_variadic_scalar_inputs`
    - `crates/sifr_codegen/src/lib_codegen_tests.rs`
      - `test_variadic_min_max_lower_to_nested_calls`
  - Validation:
    - `cargo test -p sifr_hir test_min_max_accept_variadic_scalar_inputs -- --nocapture`
    - `cargo test -p sifr_hir test_max_two_arg_rejects_optional_operand -- --nocapture`
    - `cargo test -p sifr_codegen test_variadic_min_max_lower_to_nested_calls -- --nocapture`
    - `cargo run -q -p sifr -- check audits/leetcode/0072_edit_distance.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/0221_maximal_square.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/2002_maximum_product_of_the_length_of_two_palindromic_subsequences.sifr`
  - Scope delta:
    - scoped fixtures no longer emit `min()/max() takes 1 or 2 arguments`; residual diagnostics are secondary non-WS1 blockers
  - PR:
    - `https://github.com/yaseralnajjar/sifr/pull/1596` (merged)
