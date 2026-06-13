# Ad-hoc Phase Execution: Surface Parity And API Cleanup (2026-04-07)

Status: done
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
- 2026-04-07: `WS2` implemented, validated, reviewed, and merged via PR `#1597`.
- 2026-04-07: `WS4` implemented, validated, reviewed, and merged via PR `#1598`.
- 2026-04-07: `WS3` implemented, validated, reviewed, and merged via PR `#1599`.
- 2026-04-07: `WS5` and `WS6` implemented, validated, reviewed, and merged via PR `#1600`.
- 2026-04-07: scoped phase fixtures reached full pass (`22/22`) after WS5/WS6 closure.
- 2026-04-07: project validation gates passed:
  - `scripts/run_all_tests.sh --profile quick`
  - `scripts/run_all_tests.sh` (profile `pr`)
- 2026-04-07: phase closure doc updated with merged PR ledger links via PR `#1602`.

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
    - `https://github.com/sifr-lang/sifr/pull/1596` (merged)

- Wave WS2 (membership parity for `range` and compat mapping wrappers)
  - Compiler changes:
    - `Type::contains_element_type()` now supports `range`, compat `defaultdict` aliases, and nullable-container unions
    - leaf/structured codegen `ContainsOp` lowering now supports `range` collections
  - Tests:
    - `crates/sifr_type_system/src/types.rs`
      - `test_contains_element_type_range_and_compat_defaultdict`
    - `crates/sifr_hir/src/lower/expressions_tests.rs`
      - `test_range_membership_checks_lower`
      - `test_defaultdict_membership_checks_lower`
    - `crates/sifr_codegen/src/lower_expr.rs`
      - `lowers_contains_for_range_collection`
  - Validation:
    - `cargo test -p sifr_type_system test_contains_element_type_range_and_compat_defaultdict -- --nocapture`
    - `cargo test -p sifr_hir test_range_membership_checks_lower -- --nocapture`
    - `cargo test -p sifr_hir test_defaultdict_membership_checks_lower -- --nocapture`
    - `cargo test -p sifr_codegen lowers_contains_for_range_collection -- --nocapture`
    - `cargo run -q -p sifr -- check audits/leetcode/0130_surrounded_regions.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/0200_number_of_islands.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/0212_word_search_ii.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/0994_rotting_oranges.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/1345_jump_game_iv.sifr`
  - Scope delta:
    - scoped fixtures no longer emit `range` membership diagnostics
    - compat mapping membership diagnostic (`__compat_defaultdict_list`) is removed
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1597` (merged)

- Wave WS4 (empty-container specialization repair)
  - Compiler changes:
    - nested inference now recognizes `str.split(...) -> list[str]` (instead of `list[Unknown]`)
    - empty-dict specialization no longer drifts to incompatible key/value types in split+zip dict patterns
  - Tests:
    - `crates/sifr_hir/src/lower/expressions_tests.rs`
      - `test_empty_dict_specialization_with_split_zip_word_pattern_shape`
  - Validation:
    - `cargo test -p sifr_hir test_empty_dict_specialization_with_split_zip_word_pattern_shape -- --nocapture`
    - `cargo test -p sifr_hir test_tuple_for_target_inference_specializes_empty_dict_for_membership_index_pattern -- --nocapture`
    - `cargo run -q -p sifr -- check audits/leetcode/0290_word_pattern.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/1345_jump_game_iv.sifr`
  - Scope delta:
    - `0290_word_pattern` now passes; scoped dict-specialization drift is removed
    - `1345_jump_game_iv` still has residual non-WS4 blockers
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1598` (merged)

- Wave WS3 (iterator consumer stabilization and tuple heap comparability)
  - Compiler changes:
    - list/set/dict comprehension lowering now accepts iterator-protocol inputs via canonical iterable element discovery
    - tuple lexicographic `Comparable` bound support added when all tuple elements satisfy `Comparable`
    - `list.sort(reverse=...)` now accepted in HIR method-call normalization/type-checking
    - list method codegen now lowers `sort(reverse=expr)` to conditional in-place reverse ordering
  - Tests:
    - `crates/sifr_hir/src/lower/expressions_tests.rs`
      - `test_comprehensions_accept_iterator_inputs`
      - `test_list_sort_accepts_reverse_keyword`
      - `test_list_sort_rejects_non_bool_reverse_keyword`
      - `test_comparable_bound_accepts_homogeneous_tuples`
    - `crates/sifr_codegen/src/methods/mod.rs`
      - `lower_method_supports_list_sort_with_reverse_flag`
  - Validation:
    - `cargo test -p sifr_hir test_comprehensions_accept_iterator_inputs -- --nocapture`
    - `cargo test -p sifr_hir test_list_sort_rejects_non_bool_reverse_keyword -- --nocapture`
    - `cargo test -p sifr_hir test_comparable_bound_accepts_homogeneous_tuples -- --nocapture`
    - `cargo test -p sifr_codegen lower_method_supports_list_sort_with_reverse_flag -- --nocapture`
    - `cargo run -q -p sifr -- check audits/leetcode/0853_car_fleet.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/1834_single_threaded_cpu.sifr`
    - `cargo run -q -p sifr -- check audits/leetcode/1851_minimum_interval_to_include_each_query.sifr`
    - `scripts/run_all_tests.sh --profile quick`
  - Scope delta:
    - `0853_car_fleet` now passes (iterator comprehension + `sort(reverse=True)` parity closed)
    - `1834_single_threaded_cpu` no longer fails on iterator-consumer parity; residual issues are secondary (`mut` param and `heapq` optional/Any typing flow)
    - `1851_minimum_interval_to_include_each_query` no longer fails on tuple comparability; residual issue is optional arithmetic narrowing
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1599` (merged)

- Wave WS5 + WS6 (codegen defect closure + canonical adaptation sweep)
  - Compiler/codegen closure:
    - narrowed value vs `Option` compare lowering corrected
    - index normalization lowering corrected for optional-tainted arithmetic
    - Rust keyword escaping for locals hardened in renderer/codegen paths
    - option-widening flow and scoped widened-binding usage stabilized
  - Adaptation closure:
    - canonicalized adaptation-owned and mixed fixtures without policy relaxation
    - explicit mutability, explicit option/result handling, canonical tuple/list shapes
  - Validation:
    - scoped `22`-fixture rerun: `22/22` pass
    - `scripts/run_all_tests.sh --profile quick` pass
    - `scripts/run_all_tests.sh` (profile `pr`) pass
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1600` (merged)

- Wave WS7 (documentation closure consistency)
  - Change:
    - parent phase doc updated with merged PR ledger section for WS1-WS6
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1602` (merged)
