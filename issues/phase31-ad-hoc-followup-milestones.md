# Phase 31 Ad Hoc Follow-up Milestones

Status: active follow-up plan on 2026-03-13
Source inputs:

- `verification/leetcode/phase31_current_full_results_after_m31a_wave5_rerun.json`
- `verification/leetcode/phase31_failure_taxonomy.json`
- `verification/leetcode/phase31_remediation_backlog.json`
- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`

## Purpose

Convert the remaining Phase 31 LeetCode failures into a complete carry-forward plan that:

- fixes root causes rather than patching individual problems,
- keeps every in-scope LeetCode problem solvable in Sifr,
- separates raw-source incompatibilities from true algorithm support,
- treats cross-cutting language-feature work as explicit prerequisites when those features are broader than the Phase 31 LeetCode closure itself.

Phase 31 itself is complete. This document is the carry-forward plan for the remaining compatibility work it surfaced.

## Current Remaining Surface

- Seed corpus size: `50`
- Current passes: `15`
- Remaining failing raw fixtures: `35`
- Problems expected to be solvable in Sifr after this carry-forward: `35`
- Known raw-source divergence requiring a canonical Sifr rewrite: `1` (`0043`)

## Planning Policy

- Fix root causes, not one-off fixtures.
- Every in-scope LeetCode problem must end up solvable in Sifr.
- If a required fix is already covered by a broader ad hoc language/compiler phase, make that phase a prerequisite and keep this Phase 31 plan focused on LeetCode closure after the prerequisite lands.
- A raw Python-shaped fixture may remain non-canonical only if it conflicts with an intentional Sifr language guarantee.
- If a raw fixture is non-canonical, add a canonical Sifr variant and count that as the pass target.
- Do not add fallback semantics that weaken ownership, type safety, or parse-safety guarantees.
- Each milestone must end with:
  - updated regression coverage,
  - regenerated compatibility artifacts where counts change,
  - demo evidence for the milestone scope,
  - `scripts/run_all_tests.sh --profile quick`,
  - `scripts/run_all_tests.sh`.

## Canonical Sifr Fixture Policy

- Every in-scope LeetCode problem must be solvable in Sifr.
- If a scraped Python fixture conflicts with an intentional Sifr language guarantee, do not weaken the language to accept it verbatim.
- Instead, create a canonical Sifr variant that preserves the same algorithm and changes only the minimum syntax or semantics required by Sifr's contracts.
- Prefer the nearest already-supported Sifr form over broader rewrites.
- Preserve algorithm shape, asymptotic complexity, and test expectations.
- Track the original raw fixture as a `raw-source divergence`, not as an unsupported problem.
- Count the canonical Sifr variant as the pass target for milestone closure.

### Rewrite Rules

- Keep supported constructs if Sifr already supports them.
- Replace only the conflicting surface.
- Prefer explicit safety over implicit fallback behavior.
- Prefer local helper extraction over whole-function rewrites.

### Milestone Planning Rule

- When a failure is caused by a policy mismatch rather than a missing compiler capability, the milestone must target the canonical Sifr form of that problem, and the plan must record the raw fixture as a source divergence rather than treating the problem itself as unsupported.

## Cross-Phase Prerequisites

These are broader feature phases that must land before the related Phase 31 LeetCode closure milestones start.

### `prereq_recursive_types`

- Source phase: `issues/ad-hoc-full-recursive-type-feature.md`
- Why it is a prerequisite:
  - Phase 31 tree cases need recursive forward references and recursive-node field access
  - the new ad hoc recursive-type phase intentionally owns the production-grade language feature and broader compiler architecture
- Phase 31 responsibility after prerequisite lands:
  - rerun the affected LeetCode cases
  - add any remaining corpus-specific regression coverage
  - close the related Phase 31 milestone only when the LeetCode cases are passing

### `prereq_own_mut`

- Source phase: `issues/ad-hoc-own-mut-parameter-convention.md`
- Why it is a prerequisite:
  - `1299` depends on `own mut` as a language feature, not a narrow LeetCode patch
  - the new ad hoc `own mut` phase intentionally owns parser/HIR/codegen/ownership contract work
- Phase 31 responsibility after prerequisite lands:
  - rewrite `1299` into canonical Sifr form
  - rerun the case and lock the corpus/demo coverage
  - close the related Phase 31 milestone only when the LeetCode case is passing

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
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 2 completed local validation for same-sequence two-pointer `while` guard narrowing.
  - Execution report: `issues/phase31-m31a-two-pointer-while-guard-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave2_results.json`
  - Targeted 10-case status: `PASS=4`, `CHECK_ERROR=6`, `RUN_ERROR=0`
  - Confirmed new pass: `0042_trapping_rain_water`
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 3 completed local validation for canonical sliding-window left-pointer narrowing.
  - Execution report: `issues/phase31-m31a-sliding-window-left-pointer-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave3_results.json`
  - Targeted three-case status: `PASS=2`, `CHECK_ERROR=1`, `RUN_ERROR=0`
  - Confirmed new passes: `0003_longest_substring_without_repeating_characters`, `1456_maximum_number_of_vowels_in_a_substring_of_given_length`
- `2026-03-12`: `m31_a_optional_narrowing_core` slice 4 completed local validation for sentinel-domain normalization on canonical infinity accumulators.
  - Execution report: `issues/phase31-m31a-sentinel-domain-normalization-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave4_results.json`
  - Targeted seeded-case status: `PASS=1`, `CHECK_ERROR=0`, `RUN_ERROR=0`
  - Confirmed new pass: `0209_minimum_size_subarray_sum`
- `2026-03-13`: `m31_a_optional_narrowing_core` slice 5 completed local validation for reverse-range recurrence narrowing over sized local constructions.
  - Execution report: `issues/phase31-m31a-reverse-range-recurrence-execution.md`
  - Targeted result artifact: `verification/leetcode/phase31_m31a_wave5_results.json`
  - Targeted four-case status: `PASS=1`, `CHECK_ERROR=3`, `RUN_ERROR=0`
  - Confirmed new pass: `1143_longest_common_subsequence`
  - Stable warmed full-corpus rerun: `verification/leetcode/phase31_current_full_results_after_m31a_wave5_rerun.json`
  - Full-corpus state after slice 5: `PASS=15`, `CHECK_ERROR=35`, `RUN_ERROR=0`

## Recommended Execution Order

1. `prereq_recursive_types`
2. `prereq_own_mut`
3. `m31_g_container_literal_specialization_and_state_tracking`
4. `m31_a_optional_flow_completion`
5. `m31_b_destructuring_and_composite_lvalues`
6. `m31_d_nested_function_pipeline_completion`
7. `m31_e_recursive_tree_surface_leetcode_closure`
8. `m31_h_local_name_binding_and_shadowing`
9. `m31_j_own_mut_leetcode_closure`
10. `m31_k_canonical_sifr_fixture_normalization`
11. `m31_i_corpus_fixture_canonicalization_for_multi_solution_files`

This order starts with the two broader feature prerequisites, then front-loads independent compiler wins, and keeps the related Phase 31 milestones focused on corpus closure rather than re-owning the prerequisite feature work.

## Milestones

### `m31_g_container_literal_specialization_and_state_tracking`

- Scope:
  - specialize empty container literals from later typed writes and reads
  - remove `Any` leakage through dictionary growth, `.get`, membership, and equality
- Implementation notes:
  - use first-write specialization for empty literals
  - propagate the specialized key/value shape through subsequent reads, `.get(...)`, membership checks, and equality
  - reject conflicting later writes with deterministic "empty literal type conflict" diagnostics
- Affected ids:
  - `0001`, `0242`, `0424`, `0523`, `0560`
- Definition of done:
  - these five cases move past `dict[Any, Any]` / `Any` arithmetic failures
  - regression coverage locks empty-literal specialization and conflicting-write diagnostics

### `m31_a_optional_flow_completion`

- Current execution status (`2026-03-13`):
  - guarded sequence indexing, two-pointer `while`, sliding-window left-pointer narrowing, sentinel normalization, and reverse-range recurrence narrowing are already landed
  - remaining optional-flow work is now the narrower closure set below
- Remaining root-cause scope:
  - fixed-index reads after length guards
  - non-empty queue/heap/list pop results under truthiness guards
  - subtractive/value-dependent recurrence indexing
- Implementation notes:
  - prefer a general forward-propagation rule for definite in-bounds access rather than adding more narrow special cases
  - track range/loop bounds, arithmetic offsets such as `i + 1` and `i + 2`, and first-element access after non-empty proofs
  - keep the existing no-implicit-unwrap rule outside proven-safe flow
- Affected ids:
  - `0053`, `0127`, `0238`, `0322`, `0502`, `0743`, `0746`
- Definition of done:
  - these seven cases move past `int | None`, `None | str`, and `None | tuple[...]` failures
  - regression coverage exists for guarded queue/heap pops and guarded recurrence indexing

### `m31_b_destructuring_and_composite_lvalues`

- Scope:
  - support fixed-shape destructuring into locals and attributes
  - support loop destructuring from known two-element items
  - support fixed-shape heterogeneous mutable cells used with subscript mutation
- Affected ids:
  - `0295`, `0703`, `0997`, `1209`
- Definition of done:
  - these four cases move past destructuring/composite-lvalue failures
  - regression coverage exists for attribute destructuring, loop tuple targets, and fixed-shape subscript augassign

### `m31_d_nested_function_pipeline_completion`

- Scope:
  - finish lowering for remaining nested function shapes, including `nonlocal`
  - infer nested helper params/returns for the supported corpus patterns
  - eliminate generic `Any` fallback leakage from nested helper bodies
- Implementation notes:
  - prefer usage-driven inference from nested helper call sites and captured-state operations rather than requiring manual annotations
  - flow argument and return expectations across recursive helpers, backtracking helpers, and captured mutable locals
  - keep this milestone corpus-driven rather than expanding into a broader nested-function feature redesign
- Affected ids:
  - `0017`, `0039`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912`
- Definition of done:
  - these nine cases move past nested-function and generic frontend failures
  - the generic frontend bucket reaches zero for the Phase 31 corpus

### `m31_e_recursive_tree_surface_leetcode_closure`

- Scope:
  - depends on `prereq_recursive_types`
  - verify that the recursive-type phase fully unblocks the tree LeetCode cases for this corpus
  - add any remaining corpus-specific regression coverage and demos needed for closure
- Affected ids:
  - `0100`, `0102`, `0110`, `0226`, `0235`
- Definition of done:
  - these five tree cases pass in the Phase 31 corpus after the recursive-type prerequisite lands
  - any residual tree-case failure is either fixed as a narrow LeetCode closure bug or sent back to the recursive-type phase with a concrete gap report
  - regression coverage exists for the corpus-facing recursive-node behavior exercised by these problems

### `m31_h_local_name_binding_and_shadowing`

- Scope:
  - make local assignment shadow the enclosing function symbol immediately and consistently
  - audit same-block reads/comparisons so they resolve to the local binding
- Affected ids:
  - `0015`
- Definition of done:
  - `0015` moves past the `function` vs `int` comparison failure
  - regression coverage locks same-name local shadowing behavior

### `m31_j_own_mut_leetcode_closure`

- Scope:
  - depends on `prereq_own_mut`
  - rewrite `1299` into canonical Sifr form using `own mut`
  - verify corpus/demo/regression closure for the LeetCode problem after the prerequisite lands
- Affected ids:
  - `1299`
- Definition of done:
  - `1299` is no longer treated as a permanent divergence in the Phase 31 corpus
  - canonical `1299` Sifr source using `own mut` checks, emits, and runs successfully
  - any residual failure is either fixed as a narrow LeetCode closure bug or sent back to the `own mut` phase with a concrete gap report

### `m31_k_canonical_sifr_fixture_normalization`

- Scope:
  - define the corpus rule for raw-source policy mismatches
  - keep the problem in scope while replacing the pass target with a canonical Sifr fixture
  - do not weaken core Sifr guarantees just to accept the raw Python-shaped syntax verbatim
- Initial affected ids:
  - `0043`
- Definition of done:
  - canonical Sifr version of `0043` exists and is counted as the pass target
  - corpus docs clearly separate “problem supported” from “raw fixture source-compatible”

### `m31_i_corpus_fixture_canonicalization_for_multi_solution_files`

- Scope:
  - normalize scraped fixtures that contain multiple alternative top-level solutions
  - prefer one canonical typed / lowest-dependency solution
  - do not treat duplicate top-level solution blocks as a language feature requirement
- Affected ids:
  - `0215`, `1046`
- Definition of done:
  - each file is reduced to one canonical solution
  - any remaining failure is reclassified into a real compiler/runtime bucket

### `m31_c_stdlib_module_parity`

- Status:
  - complete
  - leave closed unless later milestones expose a real new stdlib blocker rather than a corpus artifact or deeper compiler failure

## Raw-Source Divergence List

These are not unsupported LeetCode problems. They are raw source shapes we do not plan to support verbatim if doing so would weaken intentional Sifr guarantees.

### `0043_multiply_strings`

- Why it is a raw-source divergence:
  - the scraped Python solution relies on unchecked `int(str)` conversion
  - Sifr intentionally keeps parse safety: `int(str)` is `Result[int, ParseError]`
  - weakening that behavior would change the language’s error model, not just fix a compiler bug
- Carry-forward policy:
  - keep the problem in scope
  - add a canonical Sifr rewrite and count that as the pass target
  - rewrite only the conflicting parse-safety surface and keep the rest of the algorithm as close as possible to the existing supported form
  - document the raw-source incompatibility as a corpus divergence

## Exit Conditions For The Carry-forward Plan

- Every remaining failing problem is assigned to exactly one milestone or to the raw-source divergence list.
- Every in-scope LeetCode problem ends up solvable in Sifr, even if the raw scraped Python source is non-canonical.
- The recursive-type phase and `own mut` phase land before their dependent Phase 31 closure milestones begin.
- `1299` is closed in the corpus after the `own mut` prerequisite, rather than left as a permanent unsupported case.
- Dependency-bearing milestones are sequenced before their dependents.
- Each milestone can be executed as its own PR loop: plan -> implement -> validate -> demo -> PR -> review -> merge.
