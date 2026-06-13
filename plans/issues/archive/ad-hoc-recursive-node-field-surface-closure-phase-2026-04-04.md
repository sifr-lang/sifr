# Ad-hoc Phase: Recursive Node/Field Surface Closure (2026-04-04)

## Goal
Close the `recursive_node_and_field_expression_surface` bucket from the April 4 full-corpus rerun with a compiler-first strategy, then use minimal Sifr-canonical fixture adaptation only for residuals.

## Baseline
- Source run: `verification/leetcode/full_corpus_current_results_20260404_live_rerun1.json`
- Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260404_live_rerun1.json`
- Bucket size: `34` fixtures (`CHECK_ERROR` only)
- Per-fixture inventory: `verification/leetcode/recursive_node_field_surface_20260404_inventory.csv`
- Deep diagnostics: `tmp/recursive_node_field_34_diagnostics_20260404.txt`

Corrected ownership split after review:
- `both`: `27`
- `sifr_adaptation`: `5`
- `compiler_fix`: `2`

## Reviewer-validated corrections
- Review pass 1: `reviews/recursive-node-field-surface-review-pass1.md`
- Review pass 2: `reviews/recursive-node-field-surface-review-pass2.md`
- Review pass 3 (readiness): `reviews/recursive-node-field-surface-review-pass3.md` (`READY`)
- Corrected row: `0236_lowest_common_ancestor_of_a_binary_tree` moved `sifr_adaptation -> both`

## Root-cause decomposition
1. Node/class field expression coverage gap
- Evidence: repeated diagnostics like `attribute access '.next/.left/.right/.val/.children/.prev/.end' is not supported as an expression`
- Dominant share of bucket (`24` first-diagnostic rows), plus additional mixed rows
- Root cause is not “trees” specifically; it is expression-lowering and class-surface handling for node-style objects used by linked-list/tree/graph fixtures

2. Recursive nullable boundary contract gap
- Evidence: `expected 'TreeNode', got 'None | TreeNode'` and similar recursive helper call mismatches
- Affects recursive helpers and API boundaries where nullable traversal is intended

3. Nullable node container element refinement gap
- Evidence: queue/list tuple element mismatches like `tuple[None | TreeNode, int]` vs `tuple[TreeNode, int]`
- Needed for BFS patterns that carry nullable nodes

4. Residual fixture contract/canonicalization gap
- Evidence: duplicate signatures, quoted forward refs, return optionality mismatch
- This is residual after compiler closure; should not be first-line strategy

## Language/architecture decision
- Focus should be compiler feature closure first.
- We should not frame this lane as “tree special-casing”.
- Add general node/object field-expression and recursive nullable boundary support that helps all object-graph problems.
- Keep core principles unchanged:
  - no implicit unsafe unwrapping
  - no Python-exception semantics
  - no nonlocal mutable-capture broadening

## Workstreams

### workstream_rnfs_1_field_expression_surface (compiler)
Owner: compiler
Priority: P0

Scope:
- Support class/node field reads in expression position for object graphs used in algorithms
- Ensure field-access typing works consistently through class surfaces and recursive node types

Primary loci:
- `crates/sifr_hir/src/lower/expressions.rs` (attribute lowering path, current unsupported diagnostic)
- `crates/sifr_hir/src/lower/classes.rs` (class field surface collection/availability)
- `crates/sifr_hir/src/lower/typing_and_functions.rs` (class/recursive function boundary typing)

Definition of done:
- Field-read diagnostics for `.next/.left/.right/.val/.children/.prev/.end` disappear for compiler-owned fixtures in this bucket
- No regression in existing class-field and method-call tests
- New e2e fixtures added for linked-list/tree/graph field-read expressions

### workstream_rnfs_2_recursive_nullable_boundaries (compiler)
Owner: compiler
Priority: P0

Scope:
- Normalize recursive callable boundary checks for `T` vs `T | None` where nullable traversal is intended
- Ensure recursive helper signatures and calls are checked consistently

Primary loci:
- `crates/sifr_hir/src/lower/typing_and_functions.rs`
- `crates/sifr_type_system/src/check.rs`
- `crates/sifr_type_system/src/infer.rs`

Definition of done:
- Boundary mismatches in this lane move to pass or to non-recursive residual categories
- Diagnostics become precise when nullable intent is not declared

### workstream_rnfs_3_nullable_node_container_refinement (compiler)
Owner: compiler
Priority: P1

Scope:
- Refine element typing for queue/list/deque tuples carrying nullable nodes in BFS patterns
- Remove false incompatibility between proven-safe push/pop flows and declared container element types

Primary loci:
- `crates/sifr_hir/src/lower/container_literal_specialization.rs`
- `crates/sifr_hir/src/lower/empty_collection_refinement.rs`
- `crates/sifr_type_system/src/infer.rs`

Definition of done:
- `0513`/`0662` class of queue/container nullable-node element failures are compiler-closed

### workstream_rnfs_4_adaptation_residuals (Sifr canonicalization)
Owner: fixture adaptation
Priority: P2 (after ws1/ws2/ws3 rerun)

Adaptation-only fixture set (`5`):
- `0021_merge_two_sorted_lists`
- `0203_remove_linked_list_elements`
- `0606_construct_string_from_binary_tree`
- `0617_merge_two_binary_trees`
- `0894_all_possible_full_binary_trees`

Scope:
- Canonical signatures, explicit nullable contracts, and return annotation alignment
- No language broadening in this lane

Definition of done:
- These fixtures pass with canonical Sifr forms and no compiler hacks

## Sequencing
1. Land ws1 field-expression surface
2. Land ws2 recursive nullable boundaries
3. Land ws3 nullable-node container refinement
4. Full rerun and reclassify bucket
5. Apply ws4 adaptation only to residual adaptation-owned cases
6. Full rerun and reclassify again

## Validation protocol
Per workstream:
- Unit tests in owning crate(s)
- New non-LeetCode e2e fixtures for generalized behavior
- Targeted fixture reruns for owning subset

Phase gates:
- Gate 1: `recursive_node_and_field_expression_surface` drops materially after ws1+ws2+ws3
- Gate 2: adaptation-only set reduced to documented residuals
- Gate 3: post-ws4 full rerun with updated taxonomy and ownership matrix

## Execution artifacts to produce
- Updated full run JSON
- Updated failure taxonomy JSON
- Updated recursive node/field inventory CSV
- Short phase execution log documenting per-wave deltas

## Execution status (2026-04-04 wave-1)
- workstream focus: `workstream_rnfs_1_field_expression_surface` + `workstream_rnfs_3_nullable_node_container_refinement`
- compiler changes landed locally:
  - method-call receiver specialization now refines unresolved generic class instances from concrete method arguments (unblocks `deque`/node flows from staying as unresolved `T`)
  - non-empty `pop`/`popleft` narrowing now preserves element-level optionality (`list[T | None]` stays optional under non-empty guard instead of collapsing to `T`)
- targeted signal:
  - `audits/leetcode/0513_find_bottom_left_tree_value.sifr` no longer reports `deque.append(... expected T)` or node-field expression errors from unresolved `T`; residuals are now `while ... got 'deque'`, duplicate function definition, and return optionality
  - `audits/leetcode/0662_maximum_width_of_binary_tree.sifr` remains blocked by nullable tuple element refinement and index-shape typing (next wave)
- local validation:
  - `scripts/run_all_tests.sh --profile quick` passed

## Execution status (2026-04-04 wave-2)
- workstream focus: `workstream_rnfs_1_field_expression_surface` + `workstream_rnfs_3_nullable_node_container_refinement` + adaptation-owned residual lane
- compiler changes landed locally:
  - tuple subscript typing fixed (literal index -> exact element type, non-literal int index -> union of tuple element types)
  - class/protocol truthiness enabled in control-flow condition checks and bool/unary-not checks
  - constructor return specialization added for unresolved generic constructor returns
- adaptation-owned fixture status:
  - `0021`, `0203`, `0606`, `0617`, `0894` all check clean after canonicalization
- diagnostics artifacts:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave2_start.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave2_after_adapt.txt`

## Execution status (2026-04-04 wave-3)
- workstream focus: `workstream_rnfs_2_recursive_nullable_boundaries` (generic inference closure for optional constructor parameters)
- compiler changes landed locally:
  - `infer_type_var_bindings` now handles union parameters/arguments with optional (`None`) branches to bind concrete non-`None` type variables
  - regressions added for union-based constructor inference in:
    - `crates/sifr_hir/src/lower/generic_inference.rs`
    - `crates/sifr_hir/src/lower/expressions_tests.rs`
- targeted signal:
  - `0199_binary_tree_right_side_view`: first diagnostic shifted from field-expression unsupported to deque nullable-element mismatch
  - 34-fixture first-diagnostic count for `attribute access ... unsupported` dropped `24 -> 23`
- diagnostics artifacts:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave3.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave4.txt`

## Execution status (2026-04-04 wave-4)
- workstream focus: `workstream_rnfs_4_adaptation_residuals` applied to selected `both` residuals
- fixture closures landed locally:
  - `0199_binary_tree_right_side_view` -> pass
  - `0513_find_bottom_left_tree_value` -> pass
- diagnostics artifact:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave5.txt`
- bucket signal:
  - failing-first-diagnostic fixture count: `29 -> 27`
  - first-diagnostic `attribute access ... unsupported` remained `23`

## Execution status (2026-04-04 wave-5)
- workstream focus: recursive nullable boundary canonicalization for remaining `both` residuals
- fixture closure landed locally:
  - `0124_binary_tree_maximum_path_sum` -> pass
- diagnostics artifact:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave6.txt`
- bucket signal:
  - failing-first-diagnostic fixture count: `27 -> 26`
  - first-diagnostic `attribute access ... unsupported`: `23` (next dominant lane)

## Execution status (2026-04-04 wave-6 to wave-11)
- workstream focus: closure of residual `both` fixtures after compiler-first deltas
- major fixture closure set landed locally across these waves:
  - `0094`, `0112`, `0572`, `0662`, `0729`, `0783`, `0297`, `0876`, `0083`, `0019`, `0061`, `0025`, `0092`, `0147`, `0148`, `0143`
  - final remaining closures in wave-10/wave-11: `0138`, `0146`, `0450`, `1609`, `1669`, `1721`, `2130`
- diagnostics artifacts:
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave7.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave8.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave9.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave10_valid.txt`
  - `tmp/recursive_node_field_34_diagnostics_20260404_wave11.txt`
- bucket signal:
  - wave-10-valid inventory sweep: `34 total, 27 pass, 7 fail`
  - wave-11 inventory sweep: `34 total, 34 pass, 0 fail`
- validation gate:
  - `scripts/run_all_tests.sh --profile quick` passed after wave-11 closure

## Phase closure verdict
Phase is closed for the tracked 34-fixture bucket (`recursive_node_and_field_expression_surface` inventory from 2026-04-04): all fixtures now check clean in wave-11 with required quick validation passing.
