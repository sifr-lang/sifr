# Ad-hoc Phase: Surface Parity And API Cleanup (2026-04-07)

Status: done
Owner: phase_ad_hoc_surface_parity_and_api_cleanup
Source run artifact: `verification/leetcode/full_corpus_current_results_20260407_live_rerun2.json`
Source taxonomy artifact: `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun2.json`

## Reviewer Loop

Reviewer: agent

Artifacts:

- `reviews/surface-parity-api-cleanup-review-subgroup1.md`
- `reviews/surface-parity-api-cleanup-review-tuple-comparable.md`
- `reviews/surface-parity-api-cleanup-review-subgroup2b.md`
- `reviews/surface-parity-api-cleanup-review-codegen-group.md`
- `reviews/surface-parity-api-cleanup-review-readiness.md`
- `reviews/surface-parity-api-cleanup-review-pass2.md`

Reviewer conclusions applied:

- subgroup-1 classifications stand with no corrections
- tuple lexicographic ordering is a `compiler_feature`, not adaptation-only
- subgroup-2 ownership calls stand with no corrections
- run-stage scoped blockers belong in a compiler/codegen workstream
- readiness moved from `NOT_READY` to `ready` once the review loop was captured and the draft status cleared
- final pass-2 verdict: `READY`

## Scope Snapshot

Requested top-level categories from the latest full scan:

- `python_stdlib_and_builtin_parity_gap`: `10`
- `other_type_surface_and_api_mismatch`: `11`
- `callable_argument_contract_mismatch`: `0`
- `destructuring_and_assignment_target_surface_gap`: `1`

Important correction:

- the user-provided `callable_argument_contract_mismatch (1)` count is stale.
- latest live taxonomy has `callable_argument_contract_mismatch = 0`.

Current in-scope fixture total from the live taxonomy: `22`.

Scoped fixture list:

- `0012_integer_to_roman`
- `0072_edit_distance`
- `0130_surrounded_regions`
- `0150_evaluate_reverse_polish_notation`
- `0200_number_of_islands`
- `0212_word_search_ii`
- `0221_maximal_square`
- `0241_different_ways_to_add_parentheses`
- `0290_word_pattern`
- `0297_serialize_and_deserialize_binary_tree`
- `0682_baseball_game`
- `0853_car_fleet`
- `0994_rotting_oranges`
- `1029_two_city_scheduling`
- `1091_shortest_path_in_binary_matrix`
- `1260_shift_2d_grid`
- `1345_jump_game_iv`
- `1383_maximum_performance_of_a_team`
- `1498_number_of_subsequences_that_satisfy_the_given_sum_condition`
- `1834_single_threaded_cpu`
- `1851_minimum_interval_to_include_each_query`
- `2002_maximum_product_of_the_length_of_two_palindromic_subsequences`

Resolution-mode split for the current draft:

- `compiler`: `10`
- `adaptation`: `5`
- `both`: `7`

This split is based on actual fixture mechanics, not the trigger labels in the taxonomy.

## Root-Cause Breakdown

### SP-1 Builtin parity gaps in core collection helpers

Fixtures:

- `0072_edit_distance`
- `0221_maximal_square`
- `2002_maximum_product_of_the_length_of_two_palindromic_subsequences`

Observed diagnostics:

- `min() takes 1 or 2 arguments`
- `max() takes 1 or 2 arguments`

Root cause:

- Sifr currently lowers `min` and `max` only in 1-arg iterable and 2-arg scalar forms.
- Common Python algorithmic patterns use the variadic scalar form with 3+ positional operands.
- This is a genuine builtin parity hole, not a fixture abuse pattern.

Judgment:

- `compiler` for variadic builtin support.
- residual fixture cleanup may still be needed where the same fixture also mixes `float` sentinels or unchecked `Option` results.

Language-policy decision:

- support variadic `min(a, b, c, ...)` and `max(a, b, c, ...)`.
- this is consistent with Sifr’s core principles.
- do not weaken Optional safety just to make these fixtures pass.

### SP-2 Membership parity over `range` and compat container surfaces

Fixtures:

- `0130_surrounded_regions`
- `0200_number_of_islands`
- `0212_word_search_ii`
- `0994_rotting_oranges`
- `1345_jump_game_iv`

Observed diagnostics:

- `'in' operator not supported for type 'range'`
- `'not in' operator not supported for type 'range'`
- `'in' operator not supported for type '__compat_defaultdict_list'`

Root cause:

- `contains_element_type()` only recognizes `list`, `set`, `dict`, `str`, and `bytes`.
- `range` is intentionally a first-class type in Sifr but is not wired into membership typing.
- compat mapping surfaces such as `defaultdict(list)` are not exposing membership semantics cleanly enough to the checker.

Judgment:

- `compiler` for `range` and compat-container membership support.
- some affected fixtures also contain separate adaptation or out-of-scope blockers.

Language-policy decision:

- add membership support for `range` as `int in range(...) -> bool`.
- add membership support for compat mapping wrappers where the underlying key type is statically known.
- do not add generic dynamic membership on unknown shapes.

### SP-3 Iterator/list consumer interoperability is incomplete

Fixtures:

- `0853_car_fleet`
- `1834_single_threaded_cpu`

Observed diagnostics:

- `cannot iterate over type 'Iterator[tuple[int, int]]'`
- `cannot iterate over type 'Iterator[tuple[int, list[int]]]'`
- `for-loop iterable must have a statically-known element type, got 'Unknown'`

Root cause:

- the language already intends `zip`, `enumerate`, `reversed`, and `sorted` to interoperate with iterator consumers.
- current failures suggest type stabilization is being lost across list-comprehension or `sorted(...)` boundaries and downstream `for` loops.
- `1834` also reveals heap element typing instability after `heappop`.

Judgment:

- `compiler`.

Language-policy decision:

- no language adaptation required here.
- this is intended parity already documented in `internal_docs/phases/02_type_system_power.md`.

### SP-4 Empty-container specialization and mapping key/value inference drift

Fixtures:

- `0290_word_pattern`
- `1345_jump_game_iv`

Observed diagnostics:

- `'in' operator: element type 'str' is not compatible with collection element type 'int'`
- `dict subscript assignment key type 'str' is not compatible with dict key type 'int'`
- `subscript assignment is not supported for type 'Unknown'`

Root cause:

- empty `{}` and compat-container initialization are still over-specializing or specializing to the wrong key/value types under first-write patterns.
- this produces downstream membership and subscript-assignment errors that are not caused by the fixture itself.

Judgment:

- `compiler`.

Language-policy decision:

- strengthen empty-dict and compat-container specialization.
- do not relax static typing by defaulting to dynamic maps.

### SP-5 Heap tuple ordering and structured-comparable surface

Fixtures:

- `1851_minimum_interval_to_include_each_query`
- secondary dependency for `1834_single_threaded_cpu`

Observed diagnostics:

- `type 'tuple[int, int]' does not implement protocol 'Comparable' required by type parameter 'T'`

Root cause:

- `Comparable` currently only recognizes primitive types directly in `type_bounds.rs`.
- heap algorithms regularly use tuples of comparable primitives for lexicographic ordering.
- Python and Rust both support lexicographic tuple ordering when element types are comparable.

Judgment:

- `compiler`, unless architecture explicitly rejects tuple comparability.
- current architecture does not document tuple ordering as forbidden.

Language-policy decision:

- add lexicographic `Comparable` for tuples when all elements satisfy `Comparable`.
- this is a clean, static, principled extension.

### SP-6 Parse-safety and dynamic-shape canonicalization remains adaptation-only

Fixtures:

- `0241_different_ways_to_add_parentheses`
- `0682_baseball_game`

Observed diagnostics:

- `cannot iterate over type 'bool'`
- `return type mismatch: expected 'list[int]', got 'bool'`
- `list.append() argument type 'Result[int, ParseError]' is not compatible with list element type 'int'`

Root cause:

- `0241` relies on Python truthiness with `res or [int(s)]`, which is not canonical Sifr.
- `0682` relies on `int(str)` behaving like Python exception flow rather than Sifr parse-safe `Result[int, ParseError]`.
- both also use unchecked optional list indexing results in arithmetic.

Judgment:

- `adaptation`.

Language-policy decision:

- keep `int(str) -> Result[int, ParseError]`.
- keep explicit bool conditions instead of container truthiness shortcuts.
- do not add exception-like fallback coercions to satisfy these fixtures.

### SP-7 Canonical structured-data shape adaptation

Fixtures:

- `0012_integer_to_roman`
- `1029_two_city_scheduling`
- `1091_shortest_path_in_binary_matrix`

Observed diagnostics:

- `list element type mismatch: expected 'str', got 'int'`
- `for loop tuple target expects iterable elements of tuple type, got 'list[int]'`
- `'not in' operator: element type 'tuple[int, int]' is not compatible with collection element type 'int'`

Root cause:

- `0012` uses heterogeneous list rows (`["I", 1]`) where canonical Sifr wants tuples or named structure.
- `1029` destructures `list[int]` rows as tuple targets; that restriction is already an intentional Sifr policy.
- `1091` uses `set((0, 0))`, which is ambiguous for Sifr and specializes incorrectly; canonical Sifr should use a tuple element set shape explicitly.

Judgment:

- `adaptation`.

Language-policy decision:

- keep tuple-target destructuring restricted to tuple-shaped elements.
- keep explicit container element shapes.
- do not broaden list rows into tuple-compatible destructuring implicitly.

### SP-8 Cross-layer compiler/codegen defects surfaced inside this scope

Fixtures:

- `0150_evaluate_reverse_polish_notation`
- `0297_serialize_and_deserialize_binary_tree`
- `1260_shift_2d_grid`
- `1383_maximum_performance_of_a_team`
- `1498_number_of_subsequences_that_satisfy_the_given_sum_condition`

Observed diagnostics:

- generated Rust compares `String` against `Option<String>`
- generated Rust uses `Option<i64>` in index normalization arithmetic
- generated Rust emits reserved identifier `mod`
- run-stage warning lines about int-overflow risk appear before the actual build failure

Root cause:

- these fixtures are currently bucketed under top-level surface/API categories because of trigger diagnostics, but the pass blockers are codegen defects.
- `0150` and `0297` share an Option-comparison lowering bug around narrowed character access.
- `1260` exposes incorrect index normalization lowering for values coming from list-returning helpers.
- `1383` and `1498` expose missing Rust-keyword escaping for local identifiers.

Judgment:

- `compiler`.

Language-policy decision:

- fix codegen.
- do not reinterpret safety warnings as the real blocking issue when build failure is the actual blocker.

### SP-9 Mixed fixtures where parity closure still needs policy-consistent adaptation

Fixtures:

- `0072_edit_distance`
- `0130_surrounded_regions`
- `0200_number_of_islands`
- `0212_word_search_ii`
- `0221_maximal_square`
- `0994_rotting_oranges`
- `1851_minimum_interval_to_include_each_query`

Why they are mixed:

- `0072` still uses `float("inf")` sentinel shape in an `int` algorithm.
- `0130` and `0994` mutate parameters that are not declared `mut`.
- `0200` carries duplicate solution definitions and list-row destructuring in alternate variants.
- `0212` still has untyped trie helpers and additional field-expression issues beyond this phase.
- `0221` still depends on a total `int` result after `max(cache.values())`.
- `1851` uses `list[list[int]]` interval rows plus heap tuple ordering; one half is parity, one half is canonical shape.

Judgment:

- `both`.

Phase policy:

- close compiler-owned parity first.
- then canonicalize the residual fixture shapes without weakening Sifr rules.

## Fixture-by-Fixture Resolution Matrix

| Fixture | Primary root cause | Resolution mode | Notes |
|---|---|---|---|
| `0012_integer_to_roman` | heterogeneous row shape (`list[list[mixed]]`) | `adaptation` | rewrite to tuple or named structure |
| `0072_edit_distance` | variadic `min` + float sentinel drift | `both` | builtin parity plus canonical int-only DP shape |
| `0130_surrounded_regions` | `range` membership + parameter mutability + side-effect tuple-return style | `both` | compiler plus canonical rewrite |
| `0150_evaluate_reverse_polish_notation` | codegen bug on narrowed char/Option compare | `compiler` | overflow warning is secondary |
| `0200_number_of_islands` | `range` membership + duplicate defs + list-row destructure | `both` | compiler plus canonicalization |
| `0212_word_search_ii` | `range` membership plus additional trie-surface blockers | `both` | has out-of-scope secondary dependencies |
| `0221_maximal_square` | variadic `min` + residual optional/totality closure | `both` | compiler first |
| `0241_different_ways_to_add_parentheses` | Pythonic `or` truthiness + parse-safety mismatch | `adaptation` | keep explicit control flow |
| `0290_word_pattern` | empty-dict specialization drift | `compiler` | canonical Sifr code is already reasonable |
| `0297_serialize_and_deserialize_binary_tree` | codegen bug on narrowed char/Option compare | `compiler` | same family as `0150` |
| `0682_baseball_game` | `int(str)` Result boundary + unchecked optional list access | `adaptation` | keep parse-safe semantics |
| `0853_car_fleet` | iterator/list consumer parity + `sort(reverse=...)` stabilization | `compiler` | should already be supported surface |
| `0994_rotting_oranges` | `range` membership + parameter mutability | `both` | compiler plus explicit `mut` |
| `1029_two_city_scheduling` | list-row tuple destructuring restriction | `adaptation` | intentional Sifr policy |
| `1091_shortest_path_in_binary_matrix` | ambiguous `set((0, 0))` shape | `adaptation` | canonicalize tuple-set construction |
| `1260_shift_2d_grid` | codegen bug in index normalization | `compiler` | warning is secondary |
| `1345_jump_game_iv` | compat-membership + specialization drift | `compiler` | may still depend on separate return-path closure |
| `1383_maximum_performance_of_a_team` | reserved-identifier escaping in codegen | `compiler` | warning is secondary |
| `1498_number_of_subsequences_that_satisfy_the_given_sum_condition` | reserved-identifier escaping in codegen | `compiler` | warning is secondary |
| `1834_single_threaded_cpu` | iterator consumer parity + tuple-heap typing | `compiler` | tuple comparability likely secondary dependency |
| `1851_minimum_interval_to_include_each_query` | tuple heap comparability + row shape/optional extraction | `both` | compiler plus canonical data shape |
| `2002_maximum_product_of_the_length_of_two_palindromic_subsequences` | variadic `max` + secondary optional/shift typing | `compiler` | may expose cross-bucket residual after max parity |

## Cross-Bucket Dependencies

These fixtures are in the requested scope but will not necessarily pass after this phase alone unless their secondary blockers are also closed.

| Fixture | In-phase blocker | Secondary blocker |
|---|---|---|
| `0212_word_search_ii` | `range` membership parity | recursive node / field-expression surface, helper annotations |
| `1345_jump_game_iv` | compat membership parity | return-path / specialization closure if `seen` remains unresolved |
| `1851_minimum_interval_to_include_each_query` | tuple comparability | residual optional row extraction unless fixture is canonicalized |
| `2002_maximum_product_of_the_length_of_two_palindromic_subsequences` | variadic `max` | residual optional shift/index typing if still present |

## Compiler vs Adaptation Judgment

Summary:

- this phase is mostly compiler work.
- but a meaningful minority is intentional Sifr adaptation and must stay adaptation.

Compiler-owned closures:

1. variadic `min` / `max`
2. membership support for `range`
3. membership support for compat mapping wrappers
4. iterator/list consumer stabilization for `zip`/`sorted`/`enumerate` outputs
5. empty-dict and compat-container specialization repair
6. tuple lexicographic comparability for heap use
7. scoped codegen defects:
   - narrowed value vs `Option` compare lowering
   - index normalization with `Option`-tainted temporaries
   - Rust keyword escaping for local identifiers

Adaptation-owned closures:

1. explicit parse-safe handling of `int(str)` results
2. explicit bool control flow instead of Python truthiness shortcuts
3. canonical tuple/named-structure rows instead of heterogeneous list rows
4. tuple-target destructuring only on tuple-shaped elements
5. explicit `mut` on mutated parameters
6. explicit set/tuple element construction where the current syntax is ambiguous

## Ready-to-Implement Workstreams

### WS1: Builtin parity closure for variadic `min` / `max`

Owner: compiler

Goal:

- support `min(a, b, c, ...)` and `max(a, b, c, ...)` for 3+ scalar operands when all operands satisfy the same comparison contract.

Primary loci:

- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/min_max_validation.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`

Acceptance:

- no remaining `min()/max() takes 1 or 2 arguments` diagnostics in the scoped rerun.
- explicit tests for 3-arg and 4-arg scalar calls.

### WS2: Membership parity for `range` and compat mapping wrappers

Owner: compiler

Goal:

- make `in` / `not in` work on `range` and statically-typed compat mappings.

Primary loci:

- `crates/sifr_type_system/src/types.rs`
- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/compat_imports.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`

Acceptance:

- no scoped `range` membership diagnostics remain.
- no scoped `__compat_defaultdict_list` membership diagnostics remain.
- dedicated tests for `int in range(...)`, `int not in range(...)`, and typed compat-map membership.

### WS3: Iterator consumer stabilization and tuple heap comparability

Owner: compiler

Goal:

- preserve concrete tuple element types through iterator/list consumers.
- allow heap tuples when every tuple element is `Comparable`.

Primary loci:

- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/statements.rs`
- `crates/sifr_hir/src/lower/type_bounds.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`
- `crates/sifr/tests/e2e/pass/iterator_basics.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_heapq_consolidated.sifr`

Acceptance:

- `0853`, `1834`, and compiler-owned part of `1851` stop failing in focused reruns.
- tuple `Comparable` tests exist for heap use and lexicographic ordering.

### WS4: Empty-container specialization repair

Owner: compiler

Goal:

- stop `{}` and compat container builders from specializing to the wrong key/value surface under first-write and membership patterns.

Primary loci:

- `crates/sifr_hir/src/lower/container_literal_specialization.rs`
- `crates/sifr_hir/src/lower/nested_function_tests.rs`
- `crates/sifr_hir/src/lower/statements.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`

Acceptance:

- `0290` stops failing.
- `1345` loses the container-specialization blocker.
- new tests cover paired forward/reverse writes and membership checks after empty initialization.

### WS5: Scoped codegen defect closure

Owner: compiler

Goal:

- eliminate the run-stage Rust build defects currently surfacing inside this phase bucket.

Primary loci:

- `crates/sifr_codegen/src`
- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr/tests/e2e/pass`

Sub-lanes:

1. narrowed-value comparison lowering (`0150`, `0297`)
2. index-normalization lowering with list-returned positions (`1260`)
3. Rust keyword escaping for local identifiers (`1383`, `1498`)

Acceptance:

- focused run reruns for those fixtures produce no Rust build error.
- warnings may remain only if they are intentional safety diagnostics and the fixture still compiles and runs.

### WS6: Canonical Sifr adaptation sweep

Owner: fixture canonicalization

Goal:

- rewrite the adaptation-owned fixtures into canonical Sifr without relaxing language policy.

Target fixtures:

- `0012`
- `0241`
- `0682`
- `1029`
- `1091`
- residual adaptation parts of `0072`, `0130`, `0200`, `0221`, `0994`, `1851`

Acceptance:

- no adaptation-owned diagnostics remain in the focused rerun.
- rewrites preserve algorithmic intent and stay within current Sifr rules.

## Execution Order

1. `WS1` builtin parity
2. `WS2` membership parity
3. `WS4` empty-container specialization
4. `WS3` iterator stabilization and tuple comparability
5. `WS5` scoped codegen defects
6. `WS6` adaptation sweep

Reasoning:

- `WS1` and `WS2` remove the most obvious false blockers.
- `WS4` must land before several iterator and compat cases stop degenerating into `Unknown`.
- `WS3` depends on concrete stabilized element types.
- `WS5` is isolated by layer and should run after checker-side type surfaces are cleaner.
- `WS6` comes last so fixture rewrites are not compensating for compiler defects.

## Implementation-Readiness Checklist

- [x] each workstream has explicit compiler loci
- [x] each adaptation lane is policy-aligned
- [x] trigger-label vs actual-root-cause mismatches are documented
- [x] cross-bucket blockers are called out so exit expectations stay realistic
- [x] agent review loop completed

## Execution Progress

- [x] WS1: Builtin parity closure for variadic `min` / `max`
- [x] WS2: Membership parity for `range` and compat mapping wrappers
- [x] WS3: Iterator consumer stabilization and tuple heap comparability
- [x] WS4: Empty-container specialization repair
- [x] WS5: Scoped codegen defect closure
- [x] WS6: Canonical Sifr adaptation sweep

## Wave/PR Ledger (Merged)

- WS1: [#1596](https://github.com/sifr-lang/sifr/pull/1596)
- WS2: [#1597](https://github.com/sifr-lang/sifr/pull/1597)
- WS4: [#1598](https://github.com/sifr-lang/sifr/pull/1598)
- WS3: [#1599](https://github.com/sifr-lang/sifr/pull/1599)
- WS5 + WS6 + phase closure validation: [#1600](https://github.com/sifr-lang/sifr/pull/1600)

## Closure Validation (2026-04-07)

Scoped fixture rerun (`22` fixtures listed in this phase): `22/22` pass.

- `0012_integer_to_roman`
- `0072_edit_distance`
- `0130_surrounded_regions`
- `0150_evaluate_reverse_polish_notation`
- `0200_number_of_islands`
- `0212_word_search_ii`
- `0221_maximal_square`
- `0241_different_ways_to_add_parentheses`
- `0290_word_pattern`
- `0297_serialize_and_deserialize_binary_tree`
- `0682_baseball_game`
- `0853_car_fleet`
- `0994_rotting_oranges`
- `1029_two_city_scheduling`
- `1091_shortest_path_in_binary_matrix`
- `1260_shift_2d_grid`
- `1345_jump_game_iv`
- `1383_maximum_performance_of_a_team`
- `1498_number_of_subsequences_that_satisfy_the_given_sum_condition`
- `1834_single_threaded_cpu`
- `1851_minimum_interval_to_include_each_query`
- `2002_maximum_product_of_the_length_of_two_palindromic_subsequences`

Project validation gates:

- `scripts/run_all_tests.sh --profile quick`: pass
- `scripts/run_all_tests.sh` (profile `pr`): pass
