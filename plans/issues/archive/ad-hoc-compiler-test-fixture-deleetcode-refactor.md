# Ad Hoc Phase: Refactor LeetCode-Shaped Compiler Fixtures

Status: complete and merged on 2026-06-01
Context: planning phase for refactoring compiler tests, Sifr fixtures, and generated demo companions that currently read like LeetCode problem ports rather than focused compiler/language feature tests.

## Purpose

Remove LeetCode-style problem framing from the compiler test surface while preserving the coverage that those fixtures provide.

This phase does not ban standard data-structure vocabulary. `TreeNode`, `LinkedNode`, `Node`, `left`, `right`, and `next` are acceptable when they directly describe the compiler behavior under test. The target is problem-solution residue: challenge-style function names, helper module names, sample inputs, comments, and fixtures that read like copied benchmark/problem ports instead of minimal feature tests.

The phase is complete only when:

- compiler tests and fixtures focus on the language/codegen behavior being verified,
- problem-specific names and narratives are replaced with behavior-oriented names,
- meaningful coverage is preserved,
- generated demo companions are updated where demo source changes,
- validation manifests reference the renamed fixtures,
- `audits/leetcode/**` remains untouched.

## Source Inputs

- User request: refactor compiler tests that use LeetCode-style examples into focused language/compiler feature tests.
- Explicit exclusion: `audits/leetcode/**`.
- Explicit examples:
  - `BinaryBranch`
  - `ChainCell`
  - `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs`
  - `crates/sifr_codegen/src/lib_codegen_tests/performance_nested_mutation_codegen_tests.rs`
- Additional scan base: `8ef347e1e70207240b0a3db2bb991d6ece354b0a`.
- agent plan review: `reviews/compiler-test-leetcode-refactor-plan-review.md`.

## Non-Goals

- Do not modify `audits/leetcode/**`.
- Do not rewrite benchmark audit documents or historical review notes just because they mention LeetCode.
- Do not change compiler behavior except where a fixture rewrite exposes an existing bug.
- Do not remove coverage simply because the old fixture was algorithm-shaped.
- Do not add fallback code paths or broad refactors.
- Do not ban normal terms such as `tree`, `node`, `left`, `right`, `next`, `list`, or `dict` when those terms describe the language feature.

## Naming Rules

Use traditional, recognizable names for ordinary recursive structures:

| Old/problem-shaped name | Replacement rule |
| --- | --- |
| `ChainCell` | `LinkedNode` when a `next` field matters; otherwise `Node` |
| `BinaryBranch` | `TreeNode` when `left`/`right` fields matter |
| `ListNode` | Avoid in compiler fixtures; prefer `LinkedNode` |
| `helpers.list_node` | `helpers.nodes` |
| `nodeVal`, `nodeValue` | `node_value`, `read_value`, or `value_or_zero` |
| `treeToString` | `format_node` |
| `reverseInto`, `reverseList` | behavior names such as `move_next_into` |
| `swapPairs` | behavior names such as `rewire_first_child` |
| `combination_sum`, `subsets` | behavior names such as `collect_paths`, `collect_prefixes` |
| `collect_budget_routes` | `accumulate_items` |
| `longestCommonSubsequence` | behavior names such as `table_match_score` |
| `min_cost_climbing` | behavior names such as `neighbor_min_cost` |

Function and test names should describe compiler behavior:

- `owned_recursive_option_field_moves_without_tail_clone`
- `parent_remains_usable_after_child_take`
- `borrowed_recursive_field_clones_child`
- `recursive_option_requires_narrowing`
- `reverse_range_recurrence_reads_sized_table`
- `two_pointer_guard_reveals_element_type`
- `nested_helper_mutates_captured_collection`

## Scope Inventory

### Core Codegen Tests

Refactor inline Sifr source strings and assertions while preserving the emitted-code checks.

| File | Current issue | Required change |
| --- | --- | --- |
| `crates/sifr_codegen/src/lib_codegen_tests/recursive_node_codegen_tests.rs` | Dense cluster of `ChainCell`, `BinaryBranch`, `reverseInto`, `swapPairs`, queue traversal, and `nodeValue`. | Rewrite to minimal `LinkedNode`/`TreeNode` fixtures with behavior-oriented function names. Preserve `.take()` vs clone assertions, parent reuse assertion, mutable local binding assertion, borrowed wrapper clone assertion, and borrowed field clone assertion. |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs` | Uses `BinaryBranch` only to test non-option local widening to `Option`. | Replace with a tiny neutral class such as `Payload`. |
| `crates/sifr_codegen/src/lib_codegen_tests/async_control_codegen_tests.rs` | Recursive branch traversal fixture uses `BinaryBranch`. | Rename to `TreeNode` and behavior-oriented sum/narrowing helpers. Preserve option let-else and cloned boxed read checks. |
| `crates/sifr_codegen/src/lower_expr/comprehension_and_misc_tests.rs` | Uses `BinaryBranch` and `treeToString`. | Rename to `TreeNode` and `format_node`; preserve optional-widening closure assertion. |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs` | Mostly feature-focused, but has some algorithm-flavored locals. File is close to 900-line cap. | Apply small net-zero naming cleanup only. Do not add tests or increase size materially. |
| `crates/sifr_codegen/src/lib_codegen_tests/performance_nested_mutation_codegen_tests.rs` | Already behavior-focused; only minor names may read algorithmic. | Rename only if it improves clarity without changing behavior. |

### HIR Tests

Refactor inline Sifr source strings and any expected names.

| File | Current issue | Required change |
| --- | --- | --- |
| `crates/sifr_hir/src/lower/expressions_tests/control_flow_and_strings.rs` | `BinaryBranch` and `mirrored_sum` fixture tests recursive option narrowing. | Rename to `TreeNode` and a behavior-oriented combined-value helper. Preserve `if not p or not q` early-return narrowing coverage. |
| `crates/sifr_hir/src/lower/expressions_tests/callable_and_builtin_diagnostics.rs` | `ChainCell` nested attribute assignment fixtures. | Rename to `LinkedNode` and neutral wrapper names. Preserve nested self-field and optional-field assignment coverage. |
| `crates/sifr_hir/src/lower/nested_function_tests.rs` | `collect_budget_routes` still mirrors combination-sum structure and sample data. | Rewrite to a smaller captured-collection mutation fixture with neutral data. Keep aligned with the matching e2e fixture. |
| `crates/sifr_hir/src/lower/mod_impl.rs` | Comment mentions `ChainCell`, `BinaryBranch`, `Node`. | Update comment examples to `LinkedNode`, `TreeNode`, `Node`. |

### Driver And Workspace Tests

Refactor dotted-module examples consistently.

| File | Current issue | Required change |
| --- | --- | --- |
| `crates/sifr_driver/src/tests/project_build_check.rs` | Uses `helpers.list_node`, `ChainCell`, `Bag`, `nodeVal`. | Rename to `helpers.nodes`, `LinkedNode`, `NodeBag`, and `node_value`/`nodeValue` consistently with Sifr naming support. |
| `crates/sifr_driver/src/tests/discovery_and_workspace.rs` | Uses `helpers.list_node` and `ChainCell`. | Rename dotted module and class examples. |
| `crates/sifr_driver/src/tests/project_graph.rs` | Uses `helpers.list_node` and `ChainCell`. | Rename dotted module and class examples. |
| `crates/sifr_driver/src/tests/test_runner.rs` | Uses `helpers.list_node` as generic helper module. | Rename to the same neutral helper module if project module examples are changed. |
| `crates/sifr_driver/src/tests/diagnostics.rs` | Diagnostic text uses `helpers.list_node`. | Update expected text if the canonical example changes. |
| `crates/sifr_driver/src/project/rust_module_layout.rs` | Unit examples use `helpers.list_node`. | Rename examples to avoid preserving old helper residue. |

### E2E Fixtures

Rename fixture files when the filename itself carries old residue. Update validation manifests and generated-code-quality manifest entries.

| Current fixture | Planned fixture | Required change |
| --- | --- | --- |
| `crates/sifr/tests/e2e/pass/recursive_linked_node.sifr` | `recursive_linked_node.sifr` | Rename file and content to `LinkedNode`; update generated-code-quality manifest. |
| `crates/sifr/tests/e2e/pass/forward_ref_linked_node.sifr` | `forward_ref_linked_node.sifr` | Rename file and content; update PR e2e manifest. |
| `crates/sifr/tests/e2e/pass/recursive_tree_node.sifr` | `recursive_tree_node.sifr` | Rename file and content to `TreeNode`. |
| `crates/sifr/tests/e2e/pass/recursive_tree_narrowing_runtime.sifr` | `recursive_tree_narrowing_runtime.sifr` | Rename file and content; update quick and PR e2e manifests. |
| `crates/sifr/tests/e2e/fail/recursive_tree_attribute_without_narrowing.sifr` | `recursive_tree_attribute_without_narrowing.sifr` | Rename file and content to `TreeNode`. |
| `crates/sifr/tests/e2e/pass/reverse_range_narrowing.sifr` | Same file | Replace `longestCommonSubsequence` with neutral 2D recurrence/table scoring. |
| `crates/sifr/tests/e2e/pass/sliding_window_narrowing.sifr` | Same file | Rename `longest_distinct_badge_run` and remove challenge-style sample data. |
| `crates/sifr/tests/e2e/pass/two_pointer_guard_narrowing.sifr` | Same file | Replace trapping-water-shaped sample with minimal two-pointer guard fixture. |
| `crates/sifr/tests/e2e/pass/nested_function_recursive_collection_backtracking.sifr` | Same file | Rewrite `collect_budget_routes` to `accumulate_items`; preserve `copy`, `append`, `pop`, and recursive inference coverage. |
| `crates/sifr/tests/e2e/pass/nested_function_recursive_prefix_paths.sifr` | Same file | Review for naming consistency; keep if it already reads as a feature test. |

### Demos And Companions

Every changed demo source requires regenerated `emitted.rs` and hand-maintained `idiomatic.rs` updates.

| Demo | Current issue | Required change |
| --- | --- | --- |
| `demos/recursive_types/` | `ChainCell`, `BinaryBranch`, linked-list/tree comments. | Rename to `LinkedNode` and `TreeNode`; regenerate `emitted.rs`; update `idiomatic.rs`. |
| `demos/recursive_type_part4/` | `BinaryBranch` recursive traversal fixture. | Rename to `TreeNode`; regenerate and update companion Rust files. |
| `demos/recursive_type_part5/` | Same `BinaryBranch` traversal fixture. | Rename and regenerate/update companions. |
| `demos/recursive_type_part6/` | `BinaryBranch` plus recursive alias demo. | Rename recursive class; keep alias coverage; regenerate/update companions. |
| `demos/fixed_indexing/` | `min_cost_climbing` challenge-shaped DP. | Replace with neutral fixed-index recurrence preserving `len` guard and `i + 1`/`i + 2` reads; regenerate/update companions. |
| `demos/nested_function_part4/` | `combination_sum` and `subsets`. | Replace with nested-helper feature examples preserving captured collection mutation, recursion, `copy`, `append`, and `pop`; regenerate/update companions. |
| `demos/nested_function_part5/` | Same challenge-shaped helpers. | Replace with neutral examples while preserving closure/callable/nonlocal coverage; regenerate/update companions. |
| `demos/nested_helpers/` | Letter combinations, N-Queens, redundant connection examples. | Replace with neutral nested-helper demos preserving dict-key guarded expansion, recursive set constraint counting, and relation/cycle helper coverage; regenerate/update companions. |
| `demos/nested_helpers/main.sifr` | `demo_letter_combinations`, `demo_total_n_queens`, `demo_redundant_connection`. | Rename to behavior-oriented helpers such as `expand_keyed_strings`, `count_configurations`, and `detect_first_cycle`; regenerate/update companions. |

### Docs And Manifests

Update docs only when they preserve a renamed helper/module example used by tests.

| File | Required change |
| --- | --- |
| `verification/validation_lanes/quick_e2e_manifest.json` | Update renamed e2e fixture IDs. |
| `verification/validation_lanes/pr_e2e_manifest.json` | Update renamed e2e fixture IDs. |
| `verification/generated_code_quality/manifest.json` | Update renamed source path and ID. |
| `internal_docs/architecture.md` | Update `helpers.list_node` example if driver examples change. |
| `internal_docs/sifr_workspace_design.md` | Update `helpers.list_node` example if driver examples change. |

Concrete manifest changes:

| Manifest | Entry | From | To |
| --- | --- | --- | --- |
| `verification/generated_code_quality/manifest.json` | `id` | `e2e-048-recursive-linked-node` | `e2e-048-recursive-linked-node` |
| `verification/generated_code_quality/manifest.json` | `source_path` | `crates/sifr/tests/e2e/pass/recursive_linked_node.sifr` | `crates/sifr/tests/e2e/pass/recursive_linked_node.sifr` |
| `verification/validation_lanes/quick_e2e_manifest.json` | fixture ID | `recursive_tree_narrowing_runtime` | `recursive_tree_narrowing_runtime` |
| `verification/validation_lanes/pr_e2e_manifest.json` | fixture ID | `recursive_tree_narrowing_runtime` | `recursive_tree_narrowing_runtime` |
| `verification/validation_lanes/pr_e2e_manifest.json` | fixture ID | `forward_ref_linked_node` | `forward_ref_linked_node` |
| `verification/validation_lanes/pr_e2e_manifest.json` | fixture ID | `recursive_linked_node` | `recursive_linked_node` |

Historical `issues/` and `reviews/leetcode-*` files are out of implementation scope unless a later request explicitly asks to rewrite historical planning artifacts.

## Implementation Milestones

This is one implementation change set. Milestones define ordering and validation checkpoints, not separate PRs.

### M0: Lock Names And Manifests

- Choose final structure names:
  - `LinkedNode` for one optional `next` child.
  - `TreeNode` for `left`/`right` recursive children.
  - `Node` only for generic recursive examples with no linked/tree semantics.
- Choose final dotted module example:
  - `helpers.nodes`.
- Identify every manifest entry that references renamed e2e fixtures.

Closeout:

- `rg` for old names has only intentional historical docs or out-of-scope files.
- Manifest updates are included in the implementation patch.

### M1: Core Compiler Unit Tests

- Refactor codegen unit tests.
- Refactor HIR unit tests.
- Refactor driver/workspace unit tests.
- Keep assertions behavior-equivalent.

Closeout:

- Focused `cargo test` slices pass for touched unit tests.
- `collections_and_stdlib_codegen_tests.rs` remains under 900 lines.

### M2: E2E Fixture Rename And Rewrite

- Rename recursive e2e files with old residue.
- Rewrite contents to use `LinkedNode`/`TreeNode` and behavior-oriented functions.
- Rewrite algorithm-shaped narrowing/backtracking fixtures to neutral examples.
- Update validation manifests.

Closeout:

- Targeted `cargo run -q -p sifr -- check/run <fixture>` passes where appropriate.
- Quick/PR manifest references resolve to existing files.

### M3: Demo Rewrite And Companion Regeneration

- Rewrite demo `main.sifr` files.
- Regenerate `emitted.rs` with:

```bash
cargo run -q -p sifr -- emit demos/<demo>/main.sifr > demos/<demo>/emitted.rs
```

- Update `idiomatic.rs` manually so it remains human-readable Rust with equivalent behavior.

Closeout:

- Changed demos run or build locally.
- Generated and idiomatic companions no longer preserve old problem names.

### M4: Residue Scan And Local Validation

- Run residue scans outside `audits/leetcode/**`.
- Run targeted tests.
- Run generated-code-quality checks for changed e2e/demo fixtures.
- Run required local validation.

Closeout:

- No in-scope `ChainCell`, `BinaryBranch`, `ListNode`, `helpers.list_node`, or challenge function names remain.
- `scripts/run_all_tests.sh --profile quick` passes.
- `python3 scripts/check_hir_maintainability_guardrails.py` passes.

## Validation Plan

Run focused checks first:

```bash
cargo test -p sifr_codegen recursive_node_codegen_tests
cargo test -p sifr_codegen classes_and_basics_codegen_tests
cargo test -p sifr_codegen async_control_codegen_tests
cargo test -p sifr_codegen collections_and_stdlib_codegen_tests
cargo test -p sifr_codegen performance_nested_mutation_codegen_tests
cargo test -p sifr_hir recursive_tree_attributes_narrow_after_truthiness_or_guard
cargo test -p sifr_hir nested_attribute_assignment
cargo test -p sifr_hir nested_function
cargo test -p sifr_driver project_build_check
cargo test -p sifr_driver discovery_and_workspace
cargo test -p sifr_driver project_graph
```

Run changed e2e/demo fixtures directly as appropriate:

```bash
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/<renamed-fail-fixture>.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/<renamed-pass-fixture>.sifr
cargo run -q -p sifr -- run demos/<demo>/main.sifr
```

Verify manifest entries resolve after e2e renames and run generated-code quality checks if manifests or demo/e2e fixtures change:

```bash
python3 verification/generated_code_quality/generated_code_quality.py e2e
python3 verification/generated_code_quality/generated_code_quality.py demos
```

Run required local validation:

```bash
python3 scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh --profile quick
```

Also run file-size and whitespace checks:

```bash
wc -l crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs
git diff --check
```

## Residue Scan

After implementation, run scans excluding `audits/leetcode/**`, `target/**`, `third_party/**`, and historical planning/review files unless those are intentionally in scope:

```bash
rg -n --glob '!audits/leetcode/**' --glob '!target/**' --glob '!third_party/**' \
  'ChainCell|BinaryBranch|ListNode|helpers\.list_node|reverseInto|swapPairs|reverseList|combination_sum|combinationSum|subsets|longestCommonSubsequence|min_cost_climbing|letter_combinations|total_n_queens|redundant_connection'
```

Expected remaining matches:

- `audits/leetcode/**` only, excluded by the command.
- Historical issue/review documents only if the implementation intentionally leaves them untouched.
- Standard library API names such as `combinations`, `permutations`, `zip_longest`, and `rotate` are allowed when they test stdlib parity rather than problem solutions.

## Review Log

- `2026-06-01`: Initial discovery completed against current tree and commit `8ef347e1e70207240b0a3db2bb991d6ece354b0a`.
- `2026-06-01`: agent plan review recorded in `reviews/compiler-test-leetcode-refactor-plan-review.md`; accepted recommendations to include e2e file renames, manifest updates, and demo companion regeneration.
- `2026-06-01`: User naming discussion resolved: use traditional data-structure names where appropriate, but avoid LeetCode-specific helper conventions and problem function names.
- `2026-06-01`: agent implementation-readiness review recorded in `reviews/compiler-test-fixture-deleetcode-phase-readiness-review.md`; accepted recommendations to lock `LinkedNode`/`TreeNode`/`helpers.nodes`, enumerate manifest renames, and make `collect_budget_routes` replacement explicit.
- `2026-06-01`: agent implementation review round 1 recorded in `reviews/compiler-test-fixture-deleetcode-implementation-review-1.md`; addressed the remaining `BinaryNode`, stale diagnostic inventory, stale branch traversal test name, and `tree_sum` naming findings.
- `2026-06-01`: agent implementation review round 2 recorded in `reviews/compiler-test-fixture-deleetcode-implementation-review-2.md`; accepted the finding that `two_pointer_guard_narrowing.sifr` needed to restore post-move index reads.
- `2026-06-01`: agent implementation review round 3 recorded in `reviews/compiler-test-fixture-deleetcode-implementation-review-3.md`; reviewer verdict: no further review iteration required.

## Closure Notes

- M0 complete: final names are `LinkedNode`, `TreeNode`, and `helpers.nodes`; quick/PR/generated-code manifests reference the renamed fixtures.
- M1 complete: codegen, HIR, and driver/workspace fixtures were rewritten with behavior-oriented names while preserving the original assertions.
- M2 complete: recursive e2e files were renamed, algorithm-shaped narrowing/backtracking fixtures were rewritten to neutral examples, and the two-pointer post-move read coverage was restored after review.
- M3 complete: changed demo sources were regenerated into `emitted.rs` and paired with updated `idiomatic.rs` companions.
- M4 complete: residue scan is clean for in-scope compiler/tests/demos/manifests/docs; remaining historical mentions are outside implementation scope.

Validation completed:

- `CARGO_INCREMENTAL=0 cargo test -p sifr_codegen recursive_node_codegen_tests`
- `CARGO_INCREMENTAL=0 cargo test -p sifr_codegen test_generate_rust_recursive_tree_traversal_uses_option_let_else_and_cloned_box_reads`
- `CARGO_INCREMENTAL=0 cargo test -p sifr_codegen test_non_option_local_widened_to_option_when_reassigned_none`
- `CARGO_INCREMENTAL=0 cargo test -p sifr_codegen lowers_map_named_callable_with_optional_widening_closure`
- `CARGO_INCREMENTAL=0 cargo test -p sifr_hir recursive_tree_attributes_narrow_after_truthiness_or_guard`
- `CARGO_INCREMENTAL=0 cargo test -p sifr_hir nested_attribute_assignment`
- `CARGO_INCREMENTAL=0 cargo test -p sifr_hir test_recursive_nested_helper_infers_mutable_collection_param_from_usage`
- focused `sifr_driver` tests covering the renamed `helpers.nodes` examples
- direct `target/debug/sifr run` checks for renamed e2e pass fixtures, rewritten narrowing/backtracking fixtures, and changed demos
- direct `target/debug/sifr check` for `recursive_tree_attribute_without_narrowing.sifr`, with the expected type error
- `python3 verification/generated_code_quality/generated_code_quality.py demos`
- generated-code and e2e manifest path integrity check
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `git diff --cached --check`

- `scripts/run_all_tests.sh --profile quick` from a clean detached worktree at commit `409a5536b`
  - First attempt initialized submodules after a package-manager guardrail missing-file failure, then hit one transient `LSP protocol stress` process exit with empty stderr.
  - Immediate direct rerun of `python3 verification/tooling/lsp_protocol_stress.py` passed.
  - Full quick-profile rerun passed: `wall_time=287.88s`, `cpu=355.97s`, `67/67` quick e2e pass fixtures passed, advisory only for e2e group skew.

## PR Log

- Implementation PR merged: https://github.com/sifr-lang/sifr/pull/2226
- Merge commit: `0a616e13b56e03e39d21c23972cc2d5e2e1a1ec1`
