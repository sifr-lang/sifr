# Ad-hoc Phase: Recursive Node/Field Surface Closure — Execution Log

Status: completed (execution started and closed on 2026-04-04)
Owning phase: `issues/ad-hoc-recursive-node-field-surface-closure-phase-2026-04-04.md`

## Wave log

### 2026-04-04 wave-1 (compiler)
- scope:
  - generic class receiver specialization on method calls
  - non-empty pop narrowing fix for optional element preservation
- compiler changes:
  - added `crates/sifr_hir/src/lower/generic_receiver_specialization.rs`
  - method calls now refine unresolved class type variables from concrete argument types on receiver-bound methods
  - `nonempty_method_narrowing` now derives element type from the receiver (`list`/`deque._data`) so `T | None` element optionality is preserved under non-empty guards
- tests:
  - `cargo test -p sifr_hir generic_class_receiver -- --nocapture` -> pass
  - `cargo test -p sifr_hir guarded_list_pop -- --nocapture` -> pass
  - `scripts/run_all_tests.sh --profile quick` -> pass
- targeted fixture checks:
  - `cargo run -q -p sifr -- check audits/leetcode/0513_find_bottom_left_tree_value.sifr`
    - removed: unresolved-`T` queue append and node field-expression failures
    - residual: `while condition ... got 'deque'`, duplicate function definition, return optionality mismatch
  - `cargo run -q -p sifr -- check audits/leetcode/0662_maximum_width_of_binary_tree.sifr`
    - residual unchanged: nullable tuple element append mismatch (`tuple[None | TreeNode, int]`) and downstream index arithmetic typing

### 2026-04-04 wave-2 (compiler + targeted adaptation)
- scope:
  - tuple subscript result typing + class/protocol truthiness follow-up
  - constructor specialization fallback for unresolved generic returns
  - adaptation-owned residual canonicalization set
- compiler changes:
  - added `crates/sifr_hir/src/lower/subscript_type.rs`
  - tuple index lowering now uses exact element type for literal indices and union-of-elements for non-literal int indices
  - enabled class/protocol truthiness in control-flow condition validation and bool/unary-not checks
  - added `crates/sifr_hir/src/lower/generic_constructor_specialization.rs` and wired constructor return refinement
- adaptation lane:
  - updated adaptation-owned fixtures:
    - `audits/leetcode/0021_merge_two_sorted_lists.sifr`
    - `audits/leetcode/0203_remove_linked_list_elements.sifr`
    - `audits/leetcode/0606_construct_string_from_binary_tree.sifr`
    - `audits/leetcode/0617_merge_two_binary_trees.sifr`
    - `audits/leetcode/0894_all_possible_full_binary_trees.sifr`
  - all five adaptation-owned fixtures check clean
- diagnostics artifacts:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave2_start.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave2_after_adapt.txt`

### 2026-04-04 wave-3 (compiler)
- scope:
  - unresolved generic `T` closure for optional constructor-parameter signatures (`list[T] | None` shape)
- compiler changes:
  - updated `crates/sifr_hir/src/lower/generic_inference.rs`
    - type-variable binding now handles union parameters/arguments with explicit optional (`None`) branch behavior
  - added regressions:
    - `crates/sifr_hir/src/lower/generic_inference.rs`
      - `infers_typevar_from_optional_union_parameter_non_none_branch`
      - `optional_union_parameter_does_not_bind_typevar_from_none_argument`
    - `crates/sifr_hir/src/lower/expressions_tests.rs`
      - `test_generic_constructor_infers_typevar_from_optional_union_param`
- targeted checks:
  - `cargo run -q -p sifr -- check audits/leetcode/0199_binary_tree_right_side_view.sifr`
    - first diagnostic moved from field-expression unsupported to container element mismatch:
      - from: `attribute access '.left' is not supported as an expression`
      - to: `deque.append expected 'TreeNode', got 'None | TreeNode'`
  - `cargo run -q -p sifr -- check audits/leetcode/0101_symmetric_tree.sifr`
    - still blocked by field-expression unsupported + unresolved `T` in queue branch
- diagnostics artifacts:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave3.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave4.txt`
  - first-diagnostic `attribute access ... unsupported` count: `24 -> 23`

### 2026-04-04 wave-4 (fixture adaptation on `both` residuals)
- scope:
  - canonicalize high-signal residuals after compiler-first deltas
- fixture changes:
  - `audits/leetcode/0199_binary_tree_right_side_view.sifr`
    - rewrote solution to nullable-safe recursive right-first DFS form
    - removed deque optional-element + move-surface residuals
  - `audits/leetcode/0513_find_bottom_left_tree_value.sifr`
    - removed duplicate implementation and replaced nonlocal-based recursion with canonical nullable-safe BFS helper flow
    - resolved duplicate signature / nonlocal mutation / optional return-path instability
- targeted checks:
  - `0199` -> `no errors found`
  - `0513` -> `no errors found`
- diagnostics artifact:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave5.txt`
  - fail count delta: `29 -> 27`

### 2026-04-04 wave-5 (fixture adaptation on `both` residuals)
- scope:
  - recursive nullable helper boundary canonicalization
- fixture changes:
  - `audits/leetcode/0124_binary_tree_maximum_path_sum.sifr`
    - normalized signature to nullable root
    - typed nested helper boundary as `TreeNode | None -> int`
    - added explicit optional handling for list-index reads (`res[0]`)
- targeted checks:
  - `0124` -> `no errors found`
  - regression confirm: `0199` and `0513` still clean
- diagnostics artifact:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave6.txt`
  - fail count delta: `27 -> 26`
  - first-diagnostic `attribute access ... unsupported` remains `23`

### 2026-04-04 wave-6 to wave-9 (fixture adaptation closure waves)
- scope:
  - close remaining `both` fixtures with canonical nullable/ownership-safe forms after compiler deltas
- fixture closures landed during these waves:
  - `0094_binary_tree_inorder_traversal`
  - `0112_path_sum`
  - `0572_subtree_of_another_tree`
  - `0662_maximum_width_of_binary_tree`
  - `0729_my_calendar_i`
  - `0783_minimum_distance_between_bst_nodes`
  - `0297_serialize_and_deserialize_binary_tree`
  - `0876_middle_of_the_linked_list`
  - `0083_remove_duplicates_from_sorted_list`
  - `0019_remove_nth_node_from_end_of_list`
  - `0061_rotate_list`
  - `0025_reverse_nodes_in_k_group`
  - `0092_reverse_linked_list_ii`
  - `0147_insertion_sort_list`
  - `0148_sort_list`
  - `0143_reorder_list`
- diagnostics artifacts:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave7.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave8.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave9.txt`
- signal:
  - first-diagnostic failing fixture count reached `13` by wave-9

### 2026-04-04 wave-10 (fresh inventory sweep and residual isolation)
- scope:
  - regenerate full 34-fixture diagnostics after latest adaptations
- diagnostics artifact:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave10_valid.txt`
- result:
  - `34 total, 27 pass, 7 fail`
  - remaining fixtures:
    - `0138_copy_list_with_random_pointer`
    - `0146_lru_cache`
    - `0450_delete_node_in_a_bst`
    - `1609_even_odd_tree`
    - `1669_merge_in_between_linked_lists`
    - `1721_swapping_nodes_in_a_linked_list`
    - `2130_maximum_twin_sum_of_a_linked_list`

### 2026-04-04 wave-11 (final closure)
- scope:
  - close final 7 residual fixtures and rerun full inventory
- fixture closures landed:
  - `0138_copy_list_with_random_pointer`
  - `0146_lru_cache`
  - `0450_delete_node_in_a_bst`
  - `1609_even_odd_tree`
  - `1669_merge_in_between_linked_lists`
  - `1721_swapping_nodes_in_a_linked_list`
  - `2130_maximum_twin_sum_of_a_linked_list`
- diagnostics artifact:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave11.txt`
- result:
  - `34 total, 34 pass, 0 fail`
- validation:
  - `scripts/run_all_tests.sh --profile quick` -> pass

## Closure summary
- Phase bucket inventory (`34` fixtures) is fully closed in local checks.
- Quick validation lane passed post-closure.
