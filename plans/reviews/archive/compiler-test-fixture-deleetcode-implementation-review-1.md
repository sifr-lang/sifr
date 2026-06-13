## Review: `ad-hoc-compiler-test-fixture-deleetcode-refactor.md`

Residue scan and focused validations match what you reported. The renamed pass fixtures, fail fixture, and touched demos all run. Manifests (`verification/validation_lanes/{pr,quick}_e2e_manifest.json`, `verification/generated_code_quality/manifest.json`) are updated and point at the new files. `internal_docs/architecture.md` and `internal_docs/sifr_workspace_design.md` are consistent with the `helpers.nodes` rename. All LeetCode-style function names (`chain_cell`, `list_node`, `combination_sum`, `subsets`, `longestCommonSubsequence`, `min_cost_climbing`, `letter_combinations`, `total_n_queens`, `redundant_connection`, `treeToString`, `nodeVal`, `nodeValue`, `reverseInto`, `swapPairs`, `envelope_adjustment`, `longest_distinct_badge_run`, `collect_budget_routes`, `mirrored_sum`, `same_shape_and_sum`, `demo_letter_combinations`, `demo_total_n_queens`, `demo_redundant_connection`, `chain_cell`, `binary_branch`, `chaincell`, `binarybranch`) are gone from the in-scope tree.

There are no actionable blockers for the residue goal. The items below are inconsistencies worth flagging; none of them re-introduce LeetCode-shaped problem residue.

### Findings (ordered by severity)

1. **M0 locked-naming deviation (medium, non-blocking for the residue goal).** The phase's M0 closeout explicitly locks `TreeNode` for `left/right` recursive children, but the unit tests, HIR test, and three e2e fixtures use `BinaryNode` for `left/right` classes while the demos (`demos/recursive_types/`, `recursive_type_part4/5/6/`) use `TreeNode`. Per M0 the implementation should use one name. `BinaryNode` is a traditional, non-LeetCode name, so it is acceptable, but it is inconsistent with the demo set and with M0.
   - `crates/sifr_codegen/src/lib_codegen_tests/recursive_node_codegen_tests.rs:77, 112`
   - `crates/sifr_codegen/src/lib_codegen_tests/async_control_codegen_tests.rs:320`
   - `crates/sifr_hir/src/lower/expressions_tests/control_flow_and_strings.rs:666`
   - `crates/sifr/tests/e2e/pass/recursive_tree_node.sifr:3` and `crates/sifr/tests/e2e/pass/recursive_tree_narrowing_runtime.sifr:3`
   - `crates/sifr/tests/e2e/fail/recursive_tree_attribute_without_narrowing.sifr:3`
   - `crates/sifr_codegen/src/lower_expr/comprehension_and_misc_tests.rs:228` uses `TreeNode` (the other test files split the opposite way).

2. **Stale doc reference (low).** `internal_docs/diagnostic_emission_inventory.md:227` still lists `recursive_branch_attribute_without_narrowing.sifr`. The fixture was renamed to `recursive_tree_attribute_without_narrowing.sifr` in this phase. This is a real doc that is in the implementation diff scope and should be updated to match.

3. **Stale test function name (low).** `crates/sifr_codegen/src/lib_codegen_tests/async_control_codegen_tests.rs:318` is `test_generate_rust_recursive_branch_traversal_uses_option_let_else_and_cloned_box_reads` — the test body and fixture class were refactored, but the test function still carries the old `branch_traversal` token. Not in the phase's residue-scan regex, but it is the same `branch` residue that the phase is removing from filenames and other surface.

4. **Naming drift between demo and corresponding e2e/unit (low).** Demos use `tree_sum`; the corresponding e2e fixture and unit tests use `tree_value_sum`. Both reach the same behavior assertions, but the mismatch is the kind of thing the phase says to keep tight (`function and test names should describe compiler behavior`). Visible at:
   - `demos/recursive_type_part4/main.sifr:13` vs `crates/sifr/tests/e2e/pass/recursive_tree_narrowing_runtime.sifr:13` and `crates/sifr_codegen/src/lib_codegen_tests/async_control_codegen_tests.rs:320`.

### Non-blocking cleanup / validation caveats

- `internal_docs/generated_code_quality.md:160, 165` still mentions `treeToString` in a NeetCode-group audit section. The phase says "do not rewrite benchmark audit documents or historical review notes just because they mention LeetCode", so this is acceptable; flagging only because it appeared in the broader scan.
- `internal_docs/phases/34_generated_code_quality_and_production_readiness.md:351-360` is intentionally excluded from the phase's residue scan; same `treeToString` references live there.
- The codegen-lowering changes (`crates/sifr_codegen/src/lower_stmt/condition_type_and_expr_helpers.rs`, `crates/sifr_codegen/src/stmt_support_emitter/{await_and_async_comprehension,string_assignment}.rs`, and the `performance_codegen_tests.rs` push→push_str test changes) are pre-existing dirty work that you flagged as out of scope for this phase. The full `cargo test -p sifr_codegen` failure you described is consistent with those, not with this phase's diff. The `nested_function_recursive_collection_backtracking.sifr` rename and demo companion regeneration are part of the phase and tested independently.
- Project verification baselines under `crates/sifr/tests/verification/project/**/baselines/` are pre-existing dirty work; out of scope.
- `verification/generated_code_quality/generated_code_quality.py`, `scripts/build_full_corpus_failure_taxonomy.py` (`"unknown type: 'ListNode'"` → `"LinkedNode"`), and `audits/leetcode` are pre-existing dirty work; out of scope.
- The `pr_e2e_manifest.json`/`quick_e2e_manifest.json` were not updated to add the rewritten in-place fixtures (`reverse_range_narrowing`, `sliding_window_narrowing`, `two_pointer_guard_narrowing`, `nested_function_recursive_collection_backtracking`, `recursive_tree_node`, `recursive_tree_attribute_without_narrowing`). They were not in the pre-PR manifests either, so this is consistent with the phase's manifest-update list — flagging only for completeness.

### Verdict

No actionable blockers for the residue-removal goal. If you want strict M0 compliance, address items 1-2 (and optionally 3-4) in a follow-up; the renames in items 3-4 are surface polish, not residue.
