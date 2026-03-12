# Phase 31 Ad Hoc Follow-up Milestones

Status: active follow-up plan on 2026-03-11
Source inputs:
- `verification/leetcode/phase31_scorecard.md`
- `verification/leetcode/phase31_failure_taxonomy.json`
- `verification/leetcode/phase31_remediation_backlog.json`

## Purpose

Convert the unresolved Phase 31 LeetCode compatibility backlog into a small set of execution-ready milestones with explicit sequencing, measurable success criteria, and validation gates.

Phase 31 itself is complete. This document is the carry-forward plan for the remaining compatibility work that Phase 31 surfaced.

## Current Remaining Surface

- Seed corpus size: `50`
- Current passes: `5`
- Remaining failing cases: `45`
- Supportable in current language/runtime direction: `44`
- Explicit intentional divergence: `1` (`ownership.borrowed_return_surface`, case `1299`)

## Planning Rules

- Fix root causes, not individual LeetCode programs.
- Prefer milestones that remove an entire failure bucket or unblock a dependent bucket.
- Do not add fallback semantics that weaken Sifr's ownership or type guarantees.
- Each milestone must end with:
  - updated regression coverage,
  - regenerated compatibility artifacts where counts change,
  - demo evidence for the milestone scope,
  - `scripts/run_all_tests.sh --profile quick`,
  - `scripts/run_all_tests.sh`.

## Execution Log

- `2026-03-11`: `m31_c_stdlib_module_parity` slice 1 completed local validation and targeted corpus rerun.
  - Execution report: `issues/phase31-m31c-stdlib-module-parity-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave1_results.json`
  - Targeted six-case status: `PASS=1`, `CHECK_ERROR=5`, `RUN_ERROR=0`
  - Confirmed pass: `0007_reverse_integer`
  - Confirmed reclassification signal: `0502_ipo` moved past missing-`heapq` failure into deeper typing/destructuring blockers
- `2026-03-11`: `m31_c_stdlib_module_parity` slice 2 completed local validation and targeted constructor-surface rerun.
  - Execution report: `issues/phase31-m31c-constructor-compatibility-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave2_results.json`
  - Targeted three-case status: `PASS=1`, `RUN_ERROR=1`, `CHECK_ERROR=1`
  - Confirmed pass: `0217_contains_duplicate`
  - Confirmed reclassification signal: `0127_word_ladder` moved past missing bare `deque(...)` into remaining `defaultdict` / `len(deque)` blockers
  - Confirmed deeper follow-on blocker: `0003_longest_substring_without_repeating_characters` moved past missing `set(...)` into a downstream codegen panic
- `2026-03-11`: `m31_c_stdlib_module_parity` slice 3 completed local validation for `defaultdict(...)` compatibility and `len(deque)`.
  - Execution report: `issues/phase31-m31c-defaultdict-len-compat-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave3_results.json`
  - Primary seeded-case status: `0127` remains `CHECK_ERROR`, but the prior stdlib blockers are removed
  - Confirmed parity pass: `0036_valid_sudoku` now checks and runs with `defaultdict(set)`
  - Confirmed reclassification signal: `0149_max_points_on_a_line` moved past `defaultdict(int)` surface failure into deeper optional/arithmetic typing gaps
- `2026-03-12`: `m31_c_stdlib_module_parity` slice 4 completed local validation for private `heapq` max-heap compatibility.
  - Execution report: `issues/phase31-m31c-private-heapq-max-compat-execution.md`
  - PR: `#1112`
  - Targeted result artifact: `verification/leetcode/phase31_m31c_wave4_results.json`
  - Targeted six-case status: `PASS=2`, `CHECK_ERROR=3`, `RUN_ERROR=1`
  - Confirmed reclassification signal: `1046_last_stone_weight` moved past missing private `heapq` symbols into deeper annotation / `Any` typing failures
  - Confirmed broader parity probe: `2971_find_polygon_with_the_largest_perimeter` now resolves private `heapq` helpers and fails only on downstream optional arithmetic
- `2026-03-12`: `m31_c_stdlib_module_parity` milestone closed.
  - Closure report: `issues/phase31-m31c-milestone-closure.md`
  - Closure PR: `#1112`
  - Closure basis: all remaining watched-case failures are now downstream codegen/type-system work rather than `stdlib.python_module_surface`
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 1 completed local validation for guarded sequence index narrowing.
  - Execution report: `issues/phase31-m31a-guarded-sequence-index-narrowing-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave1_results.json`
  - Targeted 10-case status: `PASS=3`, `CHECK_ERROR=7`, `RUN_ERROR=0`
  - Confirmed passes: `0014_longest_common_prefix`, `0198_house_robber`, `1768_merge_strings_alternately`
  - Confirmed reclassification signal: remaining watched failures are now narrower optional-flow shapes beyond the guarded-index root cause covered in slice 1
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 2 completed local validation for same-sequence two-pointer `while` guard narrowing.
  - Execution report: `issues/phase31-m31a-two-pointer-while-guard-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave2_results.json`
  - Targeted 10-case status: `PASS=4`, `CHECK_ERROR=6`, `RUN_ERROR=0`
  - Confirmed new pass: `0042_trapping_rain_water`
  - Confirmed reclassification signal: remaining watch-set failures are now concentrated in sliding-window left-pointer, reverse-range, and constructed-sequence proof gaps rather than the same-sequence two-pointer `while` shape
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 3 completed local validation for canonical sliding-window left-pointer narrowing.
  - Execution report: `issues/phase31-m31a-sliding-window-left-pointer-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave3_results.json`
  - Targeted three-case status: `PASS=2`, `CHECK_ERROR=1`, `RUN_ERROR=0`
  - Confirmed new passes: `0003_longest_substring_without_repeating_characters`, `1456_maximum_number_of_vowels_in_a_substring_of_given_length`
  - Confirmed reclassification signal: `0209_minimum_size_subarray_sum` moved past the prior left-pointer optional-index failure into the remaining `float('inf')` branch-type mismatch

## Recommended Execution Order

1. `m31_a_optional_narrowing_core`
2. `m31_b_destructuring_target_lowering`
3. `m31_c_stdlib_module_parity`
4. `m31_d_nested_function_pipeline`
5. `m31_e_tree_node_surface`
6. `m31_f_ownership_divergence_resolution`

This order is chosen to remove the largest independent blockers first, then clear dependency chains (`unsupported_ast_shape -> nested_function_annotation_support` and `recursive_node_forward_reference -> attribute_expression_support`).

## Milestones

### `m31_a_optional_narrowing_core`

- Current execution status (`2026-03-12`):
  - in progress
  - slice 1 completed the guarded sequence-index root cause for explicit `while i < len(seq)`, `for i in range(len(seq))`, and early-return non-empty guards
  - slice 2 completed same-sequence two-pointer `while left < right` narrowing for zero/end pointer construction plus the downstream production-path emit gaps that slice exposed
  - slice 3 completed canonical sliding-window left-pointer narrowing for direct `seq[l]` reads under a `for r in range(len(seq))` driver, including tuple-unpacked zero-based locals
  - validated with targeted HIR/e2e/demo coverage, `verification/leetcode/phase31_m31a_wave1_results.json`, `verification/leetcode/phase31_m31a_wave2_results.json`, and `verification/leetcode/phase31_m31a_wave3_results.json`
  - slice 3 targeted outcome: `PASS=2`, `CHECK_ERROR=1`, `RUN_ERROR=0` across `0003`, `0209`, and `1456`
  - remaining optional failures are now concentrated in constructed-sequence, recurrence, and broader arithmetic/branch-type proof gaps rather than guarded-index, same-sequence two-pointer, or canonical sliding-window left-pointer roots

- Scope:
  - resolve `type_system.optional_narrowing_and_union_ops`
  - current blocked cases: `16`
  - affected ids: `0014`, `0015`, `0042`, `0043`, `0053`, `0198`, `0209`, `0215`, `0238`, `0322`, `0424`, `0560`, `0746`, `1143`, `1456`, `1768`
- Why this is a standalone milestone:
  - it is the largest remaining bucket
  - it is independent of lowering/frontend enablement work
  - it affects DP, strings, heaps, and sliding-window patterns at once
- Definition of done:
  - optional/union arithmetic, indexing, comparisons, and return-flow patterns used by the corpus type-check successfully
  - narrowing behavior is deterministic across `if`, early-return, and local-rebinding paths
  - the bucket is either eliminated or reduced with every remaining case reclassified into a narrower root cause
- Required validation:
  - targeted type-system regression tests for optional narrowing and union operator compatibility
  - rerun the 16 affected corpus cases and regenerate the compatibility snapshot if counts change
  - full local suite
- Expected impact:
  - highest potential pass-rate improvement of any single milestone

### `m31_b_destructuring_target_lowering`

- Scope:
  - resolve `lowering.destructuring_target_support`
  - current blocked cases: `7`
  - affected ids: `0207`, `0295`, `0684`, `0703`, `0743`, `0997`, `1209`
- Why this is a standalone milestone:
  - it is a concentrated lowering limitation with clear syntax-shape boundaries
  - it blocks graph and heap solutions that otherwise already fit current language semantics
- Definition of done:
  - tuple/loop destructuring targets used in the corpus lower into stable HIR/codegen forms
  - reassignment, loop-target, and nested destructuring diagnostics remain deterministic for unsupported shapes
  - the affected cases move past the current lowering failure
- Required validation:
  - positive e2e coverage for supported destructuring assignments and loop targets
  - negative e2e coverage for still-unsupported destructuring forms, if any remain
  - rerun the 7 affected corpus cases and regenerate the compatibility snapshot if counts change
  - full local suite
- Expected impact:
  - medium pass-rate gain and lower friction for graph/heap workloads

### `m31_c_stdlib_module_parity`

- Current execution status (`2026-03-12`):
  - complete
  - landed root-cause fixes for Python-style stdlib attribute compatibility, numeric truthiness lowering, `math.fmod` runtime parity, native `set(...)` constructor lowering, bare `deque(...)` compatibility resolution, imported callable default propagation, `defaultdict(...)` builtin compatibility, `len(deque)` sized-class handling, and private `heapq` max-heap compatibility
  - validated with `scripts/run_all_tests.sh --profile quick` and `scripts/run_all_tests.sh`
  - targeted rerun outcomes are recorded in `verification/leetcode/phase31_m31c_wave1_results.json`, `verification/leetcode/phase31_m31c_wave2_results.json`, `verification/leetcode/phase31_m31c_wave3_results.json`, and `verification/leetcode/phase31_m31c_wave4_results.json`
  - milestone closure report: `issues/phase31-m31c-milestone-closure.md`
  - remaining watched-case blockers are downstream `0003` codegen work plus deeper non-stdlib issues in `0127`, `0502`, and `1046`

- Scope:
  - resolve `stdlib.python_module_surface`
  - current blocked cases: `6`
  - affected ids: `0003`, `0007`, `0127`, `0217`, `0502`, `1046`
- Why this is a standalone milestone:
  - this is runtime/API surface work rather than core compiler work
  - it can be validated with focused stdlib parity tests and corpus reruns
- Definition of done:
  - corpus usages of `set`, `defaultdict`, `deque`, `heapq`, and equivalent module aliases resolve and behave according to documented Sifr semantics
  - any deliberate Python parity differences are documented explicitly instead of surfacing as undefined-symbol errors
  - each newly added API has regression coverage in stdlib/runtime tests
- Required validation:
  - focused stdlib parity tests per newly exposed symbol/module surface
  - rerun the 6 affected corpus cases and regenerate the compatibility snapshot if counts change
  - demo showing at least one graph case and one heap case now working
  - full local suite
- Expected impact:
  - moderate pass-rate gain across unrelated algorithm families

### `m31_d_nested_function_pipeline`

- Scope:
  - resolve `lowering.unsupported_ast_shape`
  - resolve `frontend.nested_function_annotation_support`
  - resolve `frontend.generic_check_failure`
  - current blocked cases: `10`
  - affected ids:
    - `0052`
    - `0017`, `0039`, `0050`, `0078`, `0090`, `0912`
    - `0001`, `0242`, `0523`
- Why this is a single milestone:
  - the nested-helper failures are pipeline-related
  - enabling the AST shape before inference cleanup matches the documented dependency chain
  - the generic frontend bucket should be reclassified or eliminated once nested helper handling is repaired
- Definition of done:
  - previously unsupported nested function statement shapes lower successfully
  - nested helper parameter/return inference works for the covered corpus patterns without requiring ad hoc annotations
  - the generic frontend bucket reaches zero, either by fixes or by reclassification into tighter buckets
- Required validation:
  - targeted lowering tests for nested helper shapes
  - targeted frontend/type-check tests for nested helper inference and generic-bucket regressions
  - rerun the 10 affected corpus cases and regenerate the compatibility snapshot if counts change
  - full local suite
- Expected impact:
  - unblocks backtracking-heavy problems and removes a residual generic failure bucket

### `m31_e_tree_node_surface`

- Scope:
  - resolve `type_system.recursive_node_forward_reference`
  - resolve `lowering.attribute_expression_support`
  - current blocked cases: `5`
  - affected ids: `0100`, `0102`, `0110`, `0226`, `0235`
- Why this is a single milestone:
  - the attribute-expression blocker depends on recursive node reference support
  - both buckets are tree-domain specific and benefit from being validated together
- Definition of done:
  - `TreeNode`/`ListNode`-style recursive forward references resolve in signatures and local usage without manual reordering
  - attribute reads on supported recursive node values lower successfully
  - tree cases move past both the unknown-type and unsupported-expression diagnostics
- Required validation:
  - targeted type-system tests for recursive forward references
  - targeted lowering tests for field/attribute access on recursive node values
  - rerun the 5 affected corpus cases and regenerate the compatibility snapshot if counts change
  - full local suite
- Expected impact:
  - closes the remaining tree-structure enablement gap for the seed corpus

### `m31_f_ownership_divergence_resolution`

- Scope:
  - resolve planning status for `ownership.borrowed_return_surface`
  - current blocked cases: `1`
  - affected id: `1299`
- Why this is a milestone:
  - this is not a normal bug bucket; it is the only explicitly documented language-design divergence
  - the project still needs a crisp product decision and user-facing documentation
- Definition of done:
  - either:
    - the divergence remains intentional and the required `own`/clone-style escape hatch is documented in compiler/language docs and compatibility reporting, or
    - product direction changes and a real implementation milestone is created with ownership-sound acceptance criteria
  - the status is no longer ambiguous in planning artifacts
- Required validation:
  - documentation update and compatibility artifact alignment
  - if implementation is chosen, add focused ownership tests and rerun the affected corpus case
- Expected impact:
  - removes ambiguity from the only remaining unsupported-by-design case

## Exit Conditions For The Carry-forward Plan

- Every supportable remaining case is assigned to exactly one milestone above.
- Dependency-bearing milestones are sequenced before their dependents.
- The intentional divergence is either documented as a stable policy or promoted into a normal implementation milestone.
- Each milestone can be executed as its own PR loop: plan -> implement -> validate -> demo -> PR -> review -> merge.
