# Optional/None Closure Follow-up: Root Cause Plan (Waves 7-9)

Date: 2026-03-29
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Status: reviewer-approved planning artifact (pass-2 ready verdict on 2026-03-29)

## Current Snapshot

- Baseline rerun: `PASS=97`, `CHECK_ERROR=290`, `RUN_ERROR=24`
- Latest rerun (after wave-6): `PASS=112`, `CHECK_ERROR=275`, `RUN_ERROR=24`
- Artifacts:
  - `verification/leetcode/full_corpus_current_results_20260329_live_after_optional_wave6.json`
  - `verification/leetcode/full_corpus_nonfailing_20260329_live_after_optional_wave6.json`
- Taxonomy signal:
  - Optional bucket (heuristic) remains `75` (from `84` baseline), still unresolved at phase-close level

## Root Cause Clusters (Remaining Optional Bucket)

Derived from the wave-6 full-corpus artifact (`CHECK_ERROR` fixtures in Optional bucket).

- `arith_optional_operand` (`45` fixtures):
  - arithmetic/comparison receives `T | None` where dominated control flow should prove concrete `T`
  - representative fixtures: `0016`, `0062`, `0063`, `0105`, `0122`, `0135`, `0300`
- `assignment_optional_leak` (`19` fixtures):
  - annotated locals still receive `T | None` despite nearby guards proving concrete value
  - representative fixtures: `0121`, `0152`, `0169`
- `return_optional_leak` (`15` fixtures):
  - function returns retain `T | None` after guarded logic
  - representative fixtures: `0062`, `0120`, `0377`, `0540`
- `index_optional` (`8` fixtures):
  - index target or index operand remains optional after loop/guard context
  - representative fixtures: `0287`, `0456`, `0739`, `1397`
- secondary clusters:
  - `container_elem_optional` (`4`)
  - `compare/equality optional operand` (`7` combined)
  - `membership/dict-key/stdlib-arg optional` (`6` combined)

## Principle Constraints (Non-negotiable)

All follow-up waves must preserve Sifr design:

- no implicit `Option[T] -> T` coercion
- no hidden unwrap insertion at arithmetic/call/index/return sites
- no truthiness-based Optional coercion beyond explicit, audited narrowing rules
- no fixture-specific recognizers
- residual fixture rewrites only after compiler-rule closure

## Reviewer Pass 1 Summary (Pre-Implementation Gate)

Reviewer verdict on initial draft: **not ready**.

Blocking corrections required by review:

- wave ownership needed to move to actual implementation loci:
  - assignment/flow fixes belong in `assignment_widening.rs` + `statements.rs` (+ `tuple_unpack.rs` path), not `function_flow.rs` / `check.rs`
  - call-boundary fixes belong in `method_call_args.rs` + call lowering in `expressions.rs`
  - container/key guard refinement belongs in `sequence_guard_detection.rs` + `guarded_index.rs`
- plan had to explicitly forbid operator-level Optional stripping as a “shortcut” (would violate explicit Option semantics)
- truthiness-derived sequence guards must remain sequence-specific and type-gated (no global Optional coercion)

## Proposed New Waves (Revised, Pre-Implementation)

### Wave 7: Sequence-Guard Dominance and Guarded Index Consumption

Goal:
- eliminate `T | None` leakage for guarded list/str/dict subscripts and related expression uses

Focus:
- strengthen sequence/dict guard propagation in boolean short-circuit and loop-dominated contexts
- cover shapes like:
  - `if i < len(seq) and ... seq[i] ...`
  - `if not stack or stack[-1] ...` / `if stack and ... stack[-1] ...`
  - range-derived index safety where bounds come from aliases or sequence-shape facts
- ensure guarded subscripts are consumed as concrete element types at dominated use sites only

Likely loci:
- `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
- `crates/sifr_hir/src/lower/guarded_index.rs`
- `crates/sifr_hir/src/lower/sequence_guards.rs`
- `crates/sifr_hir/src/lower/expressions.rs`
- tests: `guarded_index.rs`, `expressions_tests.rs`

Representative canaries:
- `0062_unique_paths`
- `0121_best_time_to_buy_and_sell_stock`
- `0287_find_the_duplicate_number`
- `0020_valid_parentheses`
- `0904_fruit_into_baskets`

### Wave 8: Optional Join Stabilization in Value Flow

Goal:
- prevent sticky `T | None` in assignment and return joins when all live dominated paths are concrete

Focus:
- improve reassignment/flow behavior around loops/branches so Optional widening is explicit and bounded to true nullable paths
- reduce downstream arithmetic/comparison failures by proving concrete values before operator checking
- do **not** alter operator typing to accept `T | None` directly

Likely loci:
- `crates/sifr_hir/src/lower/assignment_widening.rs`
- `crates/sifr_hir/src/lower/statements.rs`
- `crates/sifr_hir/src/lower/tuple_unpack.rs`
- (only if required by evidence) targeted follow-up in `crates/sifr_type_system/src/infer.rs`
  - evidence threshold: after wave-8 HIR-lowering changes, canary fixtures still show `Unknown | None` in final diagnostics where assignment-flow evidence is already concrete

Representative canaries:
- `0063_unique_paths_ii`
- `0120_triangle`
- `0377_combination_sum_iv`
- `0540_single_element_in_a_sorted_array`

### Wave 9: Call-Boundary and Container Refinement Closure

Goal:
- close callable-boundary Optional leaks and element/key refinement leaks without semantics weakening

Focus:
- call-boundary consistency where declarations intentionally accept nullable traversal; reject nullable calls where declarations do not
- refine container element/key types after proven guard/membership paths and non-empty proofs
- stabilize non-empty pop/peek flows without introducing implicit unwrap behavior

Likely loci:
- `crates/sifr_hir/src/lower/method_call_args.rs`
- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_hir/src/lower/nonempty_method_narrowing.rs`
- `crates/sifr_hir/src/lower/sequence_guard_detection.rs`
- `crates/sifr_hir/src/lower/guarded_index.rs`

Representative canaries:
- `0124_binary_tree_maximum_path_sum`
- `0108_convert_sorted_array_to_binary_search_tree`
- `0338_counting_bits`
- `1968_array_with_elements_not_equal_to_average_of_neighbors`

## Reviewer Gate (Required Before Code Changes)

Implementation on waves 7-9 is blocked until reviewer sign-off confirms:

- proposed fixes stay inside Sifr principles
- no hidden unwrap/truthiness broadening is introduced
- wave boundaries are coherent and independently testable

## Additional Test Guardrails (Required in Each Wave)

- add negative tests proving Optional remains rejected when no dominating proof exists
- avoid truthiness widening outside sequence-specific guarded contexts
- run full-corpus rerun after each wave and record pass/check/run deltas before moving to the next wave

Per-wave negative matrix minimum:

- wave-7:
  - unguarded index arithmetic/comparison still fails with Optional diagnostics
  - guarded index dominance succeeds only under explicit bounds/non-empty proof
- wave-8:
  - reassignment/join paths that are truly nullable remain `T | None`
  - no operator typing path accepts `T | None` directly
- wave-9:
  - nullable actual arguments remain rejected at non-nullable call boundaries
  - container/key refinement occurs only after proven guard/membership/non-empty facts
