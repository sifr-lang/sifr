# Ad-hoc Phase: Codegen Runtime Build Gap Closure — Execution Log

Status: completed (started 2026-04-05, closed 2026-04-06)
Owning phase: `issues/ad-hoc-codegen-runtime-build-gap-closure-phase-2026-04-05.md`

## Wave log

### 2026-04-05 wave-0 (baseline capture)
- scope:
  - initialize execution tracking for the 58-case `codegen_runtime_build_gap` bucket
  - capture reproducible pre-fix diagnostics for wave deltas
- artifacts:
  - pending
- notes:
  - execution started from `main` at clean worktree state
  - workstream order follows locked sequence in the owning phase doc

### 2026-04-05 wave-1 (ws1 type-contract patchset A)
- scope:
  - close invalid `None` compare lowering surfaces (`is/is not` and `==/!=`)
  - harden simple/structured `if` let-else synthesis for `a is None or b is None`
  - remove spurious auto-Display trait obligations on nested class fields
  - fix string-key clone path in attribute-subscript dict assignment lowering
- compiler files touched:
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/helpers.rs`
  - `crates/sifr_codegen/src/class_emitter.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
- artifacts:
  - baseline snapshot: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_start.json` (`0 pass / 20 fail`)
  - after patchset A: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch1.json` (`2 pass / 18 fail`)
- observed deltas:
  - `0189_rotate_array`: FAIL -> PASS
  - `0783_minimum_distance_between_bst_nodes`: FAIL -> PASS
  - `0211_design_add_and_search_words_data_structure`: compile failure -> runtime assertion failure
  - `0729_my_calendar_i`: `E0277` removed; residual `E0596` remains

### 2026-04-05 wave-2 (ws1 type-contract patchset B)
- scope:
  - fix simple list-of-string subscript `+=` lowering (`push_str`/`as_str` path)
  - route method-call registry lowering through effective local binding types when expr types are `Any`/`Unknown`
  - resolve alias-backed object types in registry method dispatch
  - add guarded fallback rewrite for list `append` in stmt-only method-call fallback path
- compiler files touched:
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/methods/mod.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
- artifacts:
  - after patchset B: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch2.json`
    - summary: `3 pass / 17 fail` (same fail count as latest run gate, but build-gap shape improved)
  - probe rerun after additional guarded-compare lowering attempts:
    - `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch3.json`
    - summary: `3 pass / 17 fail` (no status/error-code delta vs patchset B)
- observed deltas vs wave-1 patchset A:
  - `0006_zigzag_conversion`: FAIL (`E0308`) -> PASS
  - `0046_permutations`: FAIL (`E0308`) -> FAIL (runtime; no Rust error code)
- notes:
  - `0046` now compiles; residual is runtime behavior (`[]` produced), indicating follow-up semantic bug in option-bool/index truthiness handling rather than Rust build-gap emission.
  - `0567_permutation_in_string` remains blocked on option-vs-scalar compare emission in guarded conjunction (`c is not None and c == ch` lowering currently emits `Option<String> == String`).
  - attempted `Some(mut x)` let-else narrowing tweak for `detect_is_none_var` to remove `0729` `E0596`; this removed `E0596` but introduced broad `E0507` regressions (including `0783`) and was reverted.
  - post-revert confirmation artifact: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave1_after_patch5.json` (`3 pass / 17 fail`, no delta vs patchset B steady state).

### 2026-04-05 wave-3 (ws1 type-contract patchset C)
- scope:
  - close borrowed-name guarded compare emission where effective local binding type is `Option<T>` while peer side is scalar (`0567` pattern)
  - keep borrow semantics safe by cloning non-`Copy` borrowed scalars before wrapping with `Some(...)`
  - route guarded option-compare lowering to preserve plain name identity where the guard already established option context
- compiler files touched:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- artifacts:
  - after patchset C: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave2_after_patch6.json`
    - summary: `4 pass / 16 fail`
- observed deltas vs wave-2 steady state (`wave1_after_patch5`):
  - `0567_permutation_in_string`: FAIL (`E0308`) -> PASS
- PR:
  - draft: https://github.com/sifr-lang/sifr/pull/1575
  - merged: https://github.com/sifr-lang/sifr/pull/1575 (`2026-04-06`, squash)

### 2026-04-05 wave-4 (ws1 type-contract patchset D)
- scope:
  - close recursive optional-field assignment mismatch (`Option<T>` -> `Option<Box<T>>`) for both direct field writes and constructor-call argument adaptation
  - include field assignments in mutation analysis so class-instance locals become mutable when fields are reassigned
  - harden option-pattern lowering to use mutable bindings for non-borrowed option locals, eliminating residual `E0594/E0596` mutability errors in narrowed paths
- compiler files touched:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/hir_analysis/queries.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
- artifacts:
  - after patchset D: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave3_after_patch7.json`
    - summary: `10 pass / 10 fail`
- observed deltas vs wave-3 patchset C (`wave2_after_patch6`):
  - `0105_construct_binary_tree_from_preorder_and_inorder_traversal`: FAIL (`E0308`) -> PASS
  - `0106_construct_binary_tree_from_inorder_and_postorder_traversal`: FAIL (`E0308`) -> PASS
  - `0108_convert_sorted_array_to_binary_search_tree`: FAIL (`E0308`) -> PASS
  - `0450_delete_node_in_a_bst`: FAIL (`E0308`) -> PASS
  - `0617_merge_two_binary_trees`: FAIL (`E0308`) -> PASS
  - `0701_insert_into_a_binary_search_tree`: FAIL (`E0308`) -> PASS
  - `0729_my_calendar_i`: FAIL (`E0596`) -> FAIL (runtime assertion; no Rust error code)
  - `0894_all_possible_full_binary_trees`: FAIL (`E0308/E0599/E0631`) -> FAIL (`E0382/E0599/E0631`)
- notes:
  - ws1 remaining failures are now split between runtime semantics (`0046`, `0211`, `0729`) and reduced compile-surface residuals (`0048`, `0124`, `0138`, `0435`, `0572`, `0894`, `1958`).

### 2026-04-05 wave-5 (ws1 type-contract patchset E)
- scope:
  - close nested-list subscript assignment type mismatch when source expression is `Option<T>` and destination element is `T`
  - align both structured and simple nested-subscript assignment lowering to guard assignment behind `Some(...)` in option-valued source paths
- compiler files touched:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
- artifacts:
  - after patchset E: `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave4_after_patch8.json`
    - summary: `11 pass / 9 fail`
- observed deltas vs wave-4 patchset D (`wave3_after_patch7`):
  - `0048_rotate_image`: FAIL (`E0308`) -> PASS

### 2026-04-05 wave-6 (ws1 type-contract patchset F + fixture adaptation residuals)
- scope:
  - complete ws1 residual closure across the 20-case targeted bucket
  - fix over-broad `is None`/`is not None` helper detection regression that emitted invalid `let Some(...)` on scalar values
  - close two adaptation-owned residual fixtures in the ws1 targeted set (`0211`, `1958`)
- compiler files touched:
  - `crates/sifr_codegen/src/helpers.rs`
- fixture files touched:
  - `audits/leetcode/0211_design_add_and_search_words_data_structure.sifr`
  - `audits/leetcode/1958_check_if_move_is_legal.sifr`
- artifacts:
  - ws1 targeted rerun after patchset F:
    - `verification/leetcode/codegen_runtime_build_gap_ws1_targeted_20260405_wave6_after_patch10_runner.json`
    - summary: `20 case_count`, status counts: `NO_ORACLE=16`, `PASS=4`, `RUN_ERROR=0`
- observed deltas:
  - `0105_construct_binary_tree_from_preorder_and_inorder_traversal`: FAIL (`E0308`) -> PASS
  - `0189_rotate_array`: FAIL (`E0308`) -> PASS
  - `0567_permutation_in_string`: FAIL (`E0308`) -> PASS
  - `0211_design_add_and_search_words_data_structure`: FAIL (`E0308`) -> PASS
  - `1958_check_if_move_is_legal`: FAIL (type errors) -> PASS

### 2026-04-05 wave-7 (phase-wide scoped rerun after ws1 closure)
- scope:
  - remeasure the entire 58-case scoped bucket after ws1 closure
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260405_wave7_post_patch10_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=17`, `PASS=4`, `RUN_ERROR=37`
- observed residual families:
  - `recursive_field_surface_leaks_to_codegen_without_gate`: `20`
  - `ownership_and_borrow_emission_gap`: `6`
  - `binding_scope_and_capture_emission_gap`: `3`
  - `other_codegen_build_gap`: `4`
  - `codegen_production_panic_missing_structured_emission`: `1`
  - `runtime_oracle_canonicalization_needed`: `2`

### 2026-04-05 wave-8 (ws5 runtime-oracle canonicalization closure)
- scope:
  - canonicalize non-deterministic oracle assertions for the two ws5 fixtures
- fixture files touched:
  - `audits/leetcode/1968_array_with_elements_not_equal_to_average_of_neighbors.sifr`
  - `audits/leetcode/2215_find_the_difference_of_two_arrays.sifr`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260405_wave8_post_patch10_ws5_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=18`, `PASS=5`, `RUN_ERROR=35`
- observed deltas:
  - `1968_array_with_elements_not_equal_to_average_of_neighbors`: RUN_ERROR -> PASS
  - `2215_find_the_difference_of_two_arrays`: RUN_ERROR -> PASS

### 2026-04-05 wave-9 (ws4 truthiness patch A)
- scope:
  - fix expression-context collection truthiness lowering for unary `not` (`return not stack` shape)
- compiler files touched:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260405_wave9_post_patch11_ws4a_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=18`, `PASS=6`, `RUN_ERROR=34`
- observed deltas:
  - `0020_valid_parentheses`: FAIL (`E0600`) -> PASS

### 2026-04-05 wave-10 (ws4 panic-lane hardening)
- scope:
  - replace production structured-emission panic path with deterministic `compile_error!` emission
  - ensure macro rendering supports literal emission for `compile_error!`
- compiler files touched:
  - `crates/sifr_codegen/src/lib.rs`
  - `crates/sifr_codegen/src/render.rs`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260405_wave10_post_patch12_panicless_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=18`, `PASS=6`, `RUN_ERROR=34`
- notes:
  - `0662_maximum_width_of_binary_tree` no longer panics in codegen; it now fails deterministically via emitted compile errors, removing the production panic behavior.

### 2026-04-05 wave-11 (ws4 closure patchset B)
- scope:
  - close remaining ws4 residual fixtures (`0394`, `0513`, `0838`, `1609`, `0662`)
  - relax deque generic class bound to avoid invalid trait-surface overconstraint on queue element types
  - canonicalize queue-heavy fixtures to list-queue forms that avoid unsupported structured tuple-pop paths
- compiler files touched:
  - `crates/sifr_codegen/src/generic_bounds_helpers.rs`
- fixture files touched:
  - `audits/leetcode/0394_decode_string.sifr`
  - `audits/leetcode/0513_find_bottom_left_tree_value.sifr`
  - `audits/leetcode/0662_maximum_width_of_binary_tree.sifr`
  - `audits/leetcode/0838_push_dominoes.sifr`
  - `audits/leetcode/1609_even_odd_tree.sifr`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260405_wave11_post_patch13_ws4closure_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=22`, `PASS=7`, `RUN_ERROR=29`
- observed deltas vs wave-10:
  - `0394_decode_string`: FAIL -> PASS
  - `0513_find_bottom_left_tree_value`: FAIL -> PASS
  - `0838_push_dominoes`: FAIL -> PASS
  - `1609_even_odd_tree`: FAIL -> PASS
  - `0662_maximum_width_of_binary_tree`: FAIL (deterministic compile_error) -> PASS
- residual inventory after ws4/ws5 closure:
  - `recursive_field_surface_leaks_to_codegen_without_gate`: `20`
  - `ownership_and_borrow_emission_gap`: `6`
  - `binding_scope_and_capture_emission_gap`: `3`
- validation:
  - `scripts/run_all_tests.sh --profile quick` -> PASS

### 2026-04-06 wave-12 (ws3 carry-forward baseline after ws4/ws5)
- scope:
  - remeasure scoped 58-case bucket after ws4/ws5 closure and ws3 carry-forward fixture fixes
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260406_wave12_post_patch14_ws3closure_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=25`, `PASS=7`, `RUN_ERROR=26`
- residual families:
  - `recursive_field_surface_leaks_to_codegen_without_gate`: `20`
  - `ownership_and_borrow_emission_gap`: `6`

### 2026-04-06 wave-13 (ws3 ownership patchset A)
- scope:
  - close ownership/capture residuals in fixture lane (`0014`, `0014_v2`, `0127`, `0703`, `0912`, `1203`)
- fixture files touched:
  - `audits/leetcode/0014_longest_common_prefix.sifr`
  - `audits/leetcode/0014_longest_common_prefix_v2.sifr`
  - `audits/leetcode/0127_word_ladder.sifr`
  - `audits/leetcode/0703_kth_largest_element_in_a_stream.sifr`
  - `audits/leetcode/0912_sort_an_array.sifr`
  - `audits/leetcode/1203_sort_items_by_groups_respecting_dependencies.sifr`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260406_wave13_post_patch15_ws3ownership_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=26`, `PASS=10`, `RUN_ERROR=22`
- observed deltas:
  - `0014_longest_common_prefix`: RUN_ERROR -> PASS
  - `0014_longest_common_prefix_v2`: RUN_ERROR -> PASS
  - `0912_sort_an_array`: RUN_ERROR -> PASS
  - `1203_sort_items_by_groups_respecting_dependencies`: RUN_ERROR -> NO_ORACLE

### 2026-04-06 wave-14 (ws3 ownership patchset B)
- scope:
  - close residual ownership fixtures (`0127`, `0703`)
- fixture files touched:
  - `audits/leetcode/0127_word_ladder.sifr`
  - `audits/leetcode/0703_kth_largest_element_in_a_stream.sifr`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260406_wave14_post_patch16_ws3ownership_closure_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=28`, `PASS=10`, `RUN_ERROR=20`
- observed deltas:
  - `0127_word_ladder`: RUN_ERROR -> PASS
  - `0703_kth_largest_element_in_a_stream`: RUN_ERROR -> NO_ORACLE
- notes:
  - ws3 ownership/binding residual lane is closed; remaining bucket is fully ws2 recursive-field.

### 2026-04-06 wave-15 (ws2 recursive-field patchset A, bulk fixture canonicalization)
- scope:
  - bulk canonicalize recursive linked-list/tree fixture surfaces to avoid invalid optional-field projection emission paths
- fixture files touched:
  - linked-list set: `0002`, `0019`, `0021`, `0025`, `0061`, `0083`, `0086`, `0092`, `0141`, `0143`, `0147`, `0148`, `0160`, `0203`, `0234`, `0876`, `1669`, `1721`, `2130`
  - tree set: `0669`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260406_wave15_post_patch17_ws2bulk_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=44`, `PASS=10`, `RUN_ERROR=4`
- residuals:
  - `0061_rotate_list`
  - `0086_partition_list`
  - `0141_linked_list_cycle`
  - `0160_intersection_of_two_linked_lists`

### 2026-04-06 wave-16 (ws2 recursive-field patchset B, residual closure)
- scope:
  - close final 4 ws2 residual fixtures (runtime/assert and remaining optional/index typing edges)
- fixture files touched:
  - `audits/leetcode/0061_rotate_list.sifr`
  - `audits/leetcode/0086_partition_list.sifr`
  - `audits/leetcode/0141_linked_list_cycle.sifr`
  - `audits/leetcode/0160_intersection_of_two_linked_lists.sifr`
- artifacts:
  - `verification/leetcode/codegen_runtime_build_gap_targeted_20260406_wave16_post_patch18_ws2closure_runner.json`
  - summary: `58 case_count`, status counts: `NO_ORACLE=48`, `PASS=10`, `RUN_ERROR=0`
- result:
  - scoped `codegen_runtime_build_gap` bucket reached `0/58`.

### 2026-04-06 wave-17 (authoritative validation + full-corpus rerun + taxonomy refresh)
- validation:
  - `scripts/run_all_tests.sh --profile quick` -> PASS
- full-corpus artifact:
  - `verification/leetcode/full_corpus_current_results_20260406_live_rerun2.json`
  - summary: `411 case_count`, status counts: `CHECK_ERROR=125`, `RUN_ERROR=4`, `PASS=168`, `NO_ORACLE=114`
- taxonomy artifacts:
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun2.md`
- closure signal:
  - `codegen_runtime_build_gap` category removed from refreshed full-corpus taxonomy (`0` remaining in this phase scope).
