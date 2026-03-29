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
- Validation to record:
  - owning unit tests
  - non-LeetCode e2e regression(s)
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_2_unknown_optional_stabilization

status: pending

- Representative fixtures:
  - `0010_regular_expression_matching`
  - `0309_best_time_to_buy_and_sell_stock_with_cooldown`
  - `0494_target_sum`
  - `0518_coin_change_ii`
- Validation to record:
  - join/inference tests
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_3_container_element_refinement

status: pending

- Canary fixtures:
  - `0023_merge_k_sorted_lists`
  - `0115_distinct_subsequences`
- Full target family to confirm after canaries clear:
  - nullable list-of-node arguments
  - Optional-contaminated memo/cache values
  - filtered collections whose element types should become concrete
- Validation to record:
  - element refinement tests
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_4_recursive_optional_boundary_typing

status: pending

- Representative fixtures:
  - `0024_swap_nodes_in_pairs`
  - `0104_maximum_depth_of_binary_tree`
  - `0133_clone_graph`
  - `0206_reverse_linked_list`
- Validation to record:
  - recursive boundary tests
  - targeted fixture rerun results
  - full-corpus delta after reclassification

### workstream_5_residual_reclassification_and_canonicalization

status: pending

- Validation to record:
  - post-wave full corpus rerun
  - residual case inventory
  - justification for each surviving canonical rewrite
  - explicit note that no residual rewrite weakened Optional semantics

## Closeout Criteria

- workstreams `1-4` are either closed or explicitly split further with evidence
- residual rewrite lane remains small and justified
- a fresh full-corpus rerun confirms the Optional/None family is no longer the dominant unresolved category
