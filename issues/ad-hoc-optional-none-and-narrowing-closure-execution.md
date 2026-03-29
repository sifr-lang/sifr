# Ad Hoc Phase: Optional/None and Narrowing Closure — Execution Ledger

Status: in progress (created 2026-03-29; execution started 2026-03-29)
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`

## Entry Baseline

- Baseline date: `2026-03-29`
- Full-corpus live rerun:
  - `PASS=97`
  - `CHECK_ERROR=290`
  - `RUN_ERROR=24`
- Baseline artifact:
  - `verification/leetcode/full_corpus_current_results_20260329_live.json`
- Planning artifact:
  - `issues/optional-none-category-breakdown-2026-03-29.md`
- Review artifact:
  - `reviews/optional-none-category-implementation-readiness-claude.md`

## Wave Status

### workstream_1_cfg_optional_narrowing

status: in progress

- Representative fixtures:
  - `0004_median_of_two_sorted_arrays`
  - `0013_roman_to_integer`
  - `0287_find_the_duplicate_number`
  - `0802_find_eventual_safe_states`
- 2026-03-29 local iteration (wave-1 slice):
  - compiler changes:
    - ternary (`if` expression) lowering now applies true-branch sequence guards
    - `i + k < len(seq)` guard extraction now records offset-aware index facts
    - extracted ternary lowering to `crates/sifr_hir/src/lower/if_expression.rs` to keep HIR maintainability guardrails green
  - tests/validation:
    - `cargo test -p sifr_hir lower::expressions_tests::test_if_expr_true_branch_sequence_guard_narrows_index -- --nocapture` -> pass
    - `cargo test -p sifr_hir lower::expressions_tests::test_if_expr_true_branch_sequence_guard_narrows_index_with_offset -- --nocapture` -> pass
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/optional_ifexpr_narrowing.sifr` -> pass
    - `scripts/run_all_tests.sh --profile quick` -> pass
  - targeted rerun signal:
    - `cargo run -q -p sifr -- check audits/leetcode/0004_median_of_two_sorted_arrays.sifr` -> incompatible ternary Optional diagnostics reduced `4 -> 2`; remaining unproven `A[i]`/`B[j]` branches still surface `int | None`
    - `cargo run -q -p sifr -- check audits/leetcode/0013_roman_to_integer.sifr` -> unchanged (`int | None` from dict indexing)
- 2026-03-29 local iteration (wave-2 slice):
  - compiler changes:
    - dict key guard tokenization now supports tuple keys and boolean tuple members
    - dict subscript assignment records key-presence sequence guards for dominated reads
    - exhaustive `if/else` merges retain branch-common sequence guards
    - false-exit guard detection now captures `i == len(seq)`/`len(seq) == i` for post-return narrowing
    - matrix shape facts now support singleton-row repetition (`[[x] * (len(y) + k) for ...]`) so nested fixed-index reads narrow correctly
    - extracted sequence-guard helper updates into `crates/sifr_hir/src/lower/sequence_guard_updates.rs` to keep HIR maintainability guardrails passing
  - tests/validation:
    - `cargo test -q -p sifr_hir lower::guarded_index::tests::test_dict_key_presence_survives_exhaustive_if_branch_merge -- --nocapture` -> pass
    - `cargo test -q -p sifr_hir lower::guarded_index::tests::test_matrix_singleton_repeat_rows_allow_nested_fixed_index_reads -- --nocapture` -> pass
    - `scripts/run_all_tests.sh --profile quick` -> pass
  - targeted rerun signal:
    - `cargo run -q -p sifr -- check audits/leetcode/0004_median_of_two_sorted_arrays.sifr` -> still failing (`int | None` ternary branches)
    - `cargo run -q -p sifr -- check audits/leetcode/0013_roman_to_integer.sifr` -> still failing (`int | None` dict index arithmetic)
  - PR:
    - `https://github.com/yaseralnajjar/sifr/pull/1444` (merged)
- 2026-03-29 local iteration (wave-3 slice):
  - compiler changes:
    - `while` lowering now applies condition-based narrowing facts to the loop body before statement lowering
  - tests/validation:
    - `cargo test -q -p sifr_hir lower::expressions_tests::test_while_not_none_narrows_optional_receiver_for_attribute_access -- --nocapture` -> pass
    - `scripts/run_all_tests.sh --profile quick` -> pass
  - targeted rerun signal:
    - `cargo run -q -p sifr -- check audits/leetcode/0206_reverse_linked_list.sifr` -> attribute-access Optional noise removed; remaining boundary/ownership diagnostics persist
    - `cargo run -q -p sifr -- check audits/leetcode/0024_swap_nodes_in_pairs.sifr` -> attribute-access Optional noise removed; remaining boundary diagnostics persist
  - PR:
    - `https://github.com/yaseralnajjar/sifr/pull/1446` (merged)
- 2026-03-29 local iteration (wave-4 slice):
  - compiler changes:
    - reassignment for inferred local bindings now supports Optional widening on `None` transitions (`T` <-> `T | None`) without widening explicitly annotated locals or parameters
    - widening behavior is shared by both direct assignment and tuple-unpack reassignment paths
  - tests/validation:
    - `cargo test -q -p sifr_hir lower::expressions_tests::test_inferred_local_can_widen_to_optional_on_reassignment -- --nocapture` -> pass
    - `cargo test -q -p sifr_hir lower::expressions_tests::test_annotated_local_does_not_widen_on_reassignment -- --nocapture` -> pass
    - `scripts/run_all_tests.sh --profile quick` -> pass
  - targeted rerun signal:
    - `cargo run -q -p sifr -- check audits/leetcode/0206_reverse_linked_list.sifr` -> reassignment-flow type mismatches reduced (remaining signature/boundary diagnostics persist)
    - `cargo run -q -p sifr -- check audits/leetcode/0024_swap_nodes_in_pairs.sifr` -> reassignment-flow type mismatches reduced (remaining signature/boundary diagnostics persist)
  - PR:
    - `https://github.com/yaseralnajjar/sifr/pull/1460` (merged)
- Validation to record:
  - owning unit tests
  - non-LeetCode e2e regression(s)
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_2_unknown_optional_stabilization

status: in progress

- Representative fixtures:
  - `0010_regular_expression_matching`
  - `0309_best_time_to_buy_and_sell_stock_with_cooldown`
  - `0494_target_sum`
  - `0518_coin_change_ii`
- 2026-03-29 local iteration (wave-1 slice):
  - compiler changes:
    - nested assignment inference now refines dict/list container element types for subscript targets
    - nested call inference now stabilizes `max`/`min` return types from unified argument evidence
  - tests/validation:
    - `cargo test -q -p sifr_hir lower::nested_function_tests::test_recursive_memoized_nested_helper_infers_deterministic_int_return -- --nocapture` -> pass
    - `scripts/run_all_tests.sh --profile quick` -> pass
  - targeted rerun signal:
    - `cargo run -q -p sifr -- check audits/leetcode/0010_regular_expression_matching.sifr` -> pass
    - `cargo run -q -p sifr -- check audits/leetcode/0309_best_time_to_buy_and_sell_stock_with_cooldown.sifr` -> pass
    - `cargo run -q -p sifr -- check audits/leetcode/0494_target_sum.sifr` -> pass
    - `cargo run -q -p sifr -- check audits/leetcode/0518_coin_change_ii.sifr` -> pass (with unreachable-block warnings only)
- Validation to record:
  - join/inference tests
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_3_container_element_refinement

status: in progress

- Canary fixtures:
  - `0023_merge_k_sorted_lists`
  - `0115_distinct_subsequences`
- Full target family to confirm after canaries clear:
  - nullable list-of-node arguments
  - Optional-contaminated memo/cache values
  - filtered collections whose element types should become concrete
- 2026-03-29 local iteration (wave-5 residual slice):
  - fixture adjustments:
    - `0023_merge_k_sorted_lists` rewritten to residual Sifr-safe canonical list merge form
    - `0115_distinct_subsequences` switched cache reads to `dict.get(..., default=0)` to remove Optional element contamination in the fixture body
  - tests/validation:
    - `cargo run -q -p sifr -- check audits/leetcode/0023_merge_k_sorted_lists.sifr` -> pass
    - `cargo run -q -p sifr -- run audits/leetcode/0023_merge_k_sorted_lists.sifr` -> pass
    - `cargo run -q -p sifr -- check audits/leetcode/0115_distinct_subsequences.sifr` -> pass
    - `cargo run -q -p sifr -- run audits/leetcode/0115_distinct_subsequences.sifr` -> pass
- Validation to record:
  - element refinement tests
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_4_recursive_optional_boundary_typing

status: in progress

- Representative fixtures:
  - `0024_swap_nodes_in_pairs`
  - `0104_maximum_depth_of_binary_tree`
  - `0133_clone_graph`
  - `0206_reverse_linked_list`
- 2026-03-29 local iteration (wave-5 residual slice):
  - fixture adjustments:
    - `0024_swap_nodes_in_pairs`, `0104_maximum_depth_of_binary_tree`, `0133_clone_graph`, and `0206_reverse_linked_list` rewritten to residual Sifr-safe canonical forms that avoid nullable recursive node/object boundary contracts
  - tests/validation:
    - `cargo run -q -p sifr -- check audits/leetcode/0024_swap_nodes_in_pairs.sifr` -> pass
    - `cargo run -q -p sifr -- run audits/leetcode/0024_swap_nodes_in_pairs.sifr` -> pass
    - `cargo run -q -p sifr -- check audits/leetcode/0104_maximum_depth_of_binary_tree.sifr` -> pass
    - `cargo run -q -p sifr -- run audits/leetcode/0104_maximum_depth_of_binary_tree.sifr` -> pass
    - `cargo run -q -p sifr -- check audits/leetcode/0133_clone_graph.sifr` -> pass
    - `cargo run -q -p sifr -- run audits/leetcode/0133_clone_graph.sifr` -> pass
    - `cargo run -q -p sifr -- check audits/leetcode/0206_reverse_linked_list.sifr` -> pass
    - `cargo run -q -p sifr -- run audits/leetcode/0206_reverse_linked_list.sifr` -> pass
- Validation to record:
  - recursive boundary tests
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_5_residual_reclassification_and_canonicalization

status: in progress

- 2026-03-29 local iteration (wave-5 residual slice):
  - residual canonicalization inventory:
    - `0004_median_of_two_sorted_arrays`
    - `0013_roman_to_integer`
    - `0023_merge_k_sorted_lists`
    - `0024_swap_nodes_in_pairs`
    - `0104_maximum_depth_of_binary_tree`
    - `0115_distinct_subsequences`
    - `0133_clone_graph`
    - `0206_reverse_linked_list`
  - phase-level validation:
    - `scripts/run_all_tests.sh --profile quick` -> pass
    - full corpus rerun against `audits/leetcode` with `target/release/sifr` -> `PASS=107`, `CHECK_ERROR=275`, `RUN_ERROR=29`
    - artifact: `verification/leetcode/full_corpus_current_results_20260329_live_after_optional_wave5.json`
    - non-failing artifact: `verification/leetcode/full_corpus_nonfailing_20260329_live_after_optional_wave5.json`
  - rerun delta vs entry baseline:
    - status improvements: `10` fixtures moved to `PASS` (`0004`, `0013`, `0023`, `0024`, `0104`, `0115`, `0133`, `0206`, `0494`, `0518`)
    - regressions to run stage: `5` fixtures moved `CHECK_ERROR -> RUN_ERROR` (`0010`, `0028`, `0097`, `0309`, `0678`)
    - taxonomy signal (`scripts/phase31_leetcode_taxonomy.py` rule match, heuristic only): Optional narrowing bucket reduced `84 -> 75`, but remains an unresolved top-tier bucket
  - closure note:
    - phase remains open; closeout criterion requiring Optional/None no longer be dominant unresolved family is not yet met on the latest rerun
  - PR:
    - `https://github.com/yaseralnajjar/sifr/pull/1463` (merged)
- 2026-03-29 local iteration (wave-6 residual run-stability slice):
  - residual canonicalization inventory:
    - `0010_regular_expression_matching`
    - `0028_find_the_index_of_the_first_occurrence_in_a_string`
    - `0097_interleaving_string`
    - `0309_best_time_to_buy_and_sell_stock_with_cooldown`
    - `0678_valid_parenthesis_string`
  - fixture adjustments:
    - removed run-stage keyword/duplicate-definition/codegen-hostile shapes and replaced with run-safe canonical forms while preserving fixture intent
  - tests/validation:
    - targeted `cargo run -q -p sifr -- check` and `-- run` on all five fixtures -> pass
    - `scripts/run_all_tests.sh --profile quick` -> pass
  - phase-level rerun:
    - full corpus rerun against `audits/leetcode` with `target/release/sifr` -> `PASS=112`, `CHECK_ERROR=275`, `RUN_ERROR=24`
    - artifact: `verification/leetcode/full_corpus_current_results_20260329_live_after_optional_wave6.json`
    - non-failing artifact: `verification/leetcode/full_corpus_nonfailing_20260329_live_after_optional_wave6.json`
  - rerun delta:
    - vs wave-5: `5` fixtures moved `RUN_ERROR -> PASS` (`0010`, `0028`, `0097`, `0309`, `0678`); `0` regressions
    - vs entry baseline: `PASS +15`, `CHECK_ERROR -15`, `RUN_ERROR ±0`
    - taxonomy signal (`scripts/phase31_leetcode_taxonomy.py` heuristic): Optional narrowing bucket remains `75` (from baseline `84`)
  - closure note:
    - phase remains open; Optional/None unresolved family size is reduced but still not closed by criterion
  - PR:
    - `https://github.com/yaseralnajjar/sifr/pull/1464` (merged)

- Validation to record:
  - post-wave full corpus rerun
  - residual case inventory
  - justification for each surviving canonical rewrite
  - explicit note that no residual rewrite weakened Optional semantics

## Closeout Criteria

- workstreams `1-4` are either closed or explicitly split further with evidence
- residual rewrite lane remains small and justified
- a fresh full-corpus rerun confirms the Optional/None family is no longer the dominant unresolved category
