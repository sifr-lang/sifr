# Ad Hoc Phase: Optional/None and Narrowing Closure

Status: in progress (created 2026-03-29; execution started 2026-03-29)
Context: corrective compiler phase inserted after the full LeetCode corpus rerun and the Optional/None category breakdown
Execution readiness: implementation-ready in dependency order; narrowing facts must lead, but inference, container refinement, and recursive optional-boundary work are tracked as separate first-class lanes
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`

## Objective

Collapse the largest remaining LeetCode compiler-failure family by fixing the real owning compiler layers for Optional/None flow, rather than masking failures with fixture rewrites or Python-compatibility shortcuts.

This phase must preserve Sifr's core language contract:

- explicit `Option`/`Result` safety,
- explicit ownership and mutability,
- no hidden auto-unwrapping,
- no Python-truthiness-shaped narrowing,
- no hidden mutable closure state,
- and no corpus-shaped recognizers.

The goal is not to make `None` behave like Python at any cost.
The goal is to make intended Sifr Optional semantics precise enough that valid guarded programs type-check, lower, and run correctly.

## Source of Truth

- `verification/leetcode/full_corpus_current_results_20260329_live.json`
- `issues/optional-none-category-breakdown-2026-03-29.md`
- `reviews/optional-none-direct-pass1.md`
- `reviews/optional-none-category-implementation-readiness-claude.md`
- `internal_docs/architecture.md`

Implementation hotspots:

- `crates/sifr_hir/`
- `crates/sifr_type_system/`
- `crates/sifr_codegen/`
- `crates/sifr/tests/e2e/`
- `audits/leetcode/`

## Why This Needs Its Own Phase

This bucket is too large and too structurally important to treat as miscellaneous type-check cleanup.

It spans:

- control-flow fact propagation,
- type joins and `Unknown` stabilization,
- container element refinement,
- and recursive optional-boundary typing.

Those are shared compiler rules, not isolated fixture bugs.

If they are fixed ad hoc inside unrelated PRs, the likely result is:

- inconsistent narrowing semantics,
- duplicate logic across HIR and type-system layers,
- continued `Unknown | None` leakage,
- and repeated regressions across trees, graphs, DP caches, and index-heavy algorithms.

## Entry Baseline

Current live rerun baseline on 2026-03-29:

- full corpus result:
  - `PASS=97`
  - `CHECK_ERROR=290`
  - `RUN_ERROR=24`
- authoritative rerun artifact:
  - `verification/leetcode/full_corpus_current_results_20260329_live.json`
- non-failing set artifact:
  - `verification/leetcode/full_corpus_nonfailing_20260329_live.json`

Planning baseline for this phase:

- dominant family from the latest categorized analysis:
  - `Optional/None flow and narrowing gap`: `62` fixtures
- implementation-ready decomposition:
  - workstream 1: CFG/path-sensitive narrowing
  - workstream 2: inference cleanup and `Unknown | None` stabilization
  - workstream 3: container element refinement
  - workstream 4: recursive/graph/tree optional-boundary typing
  - workstream 5: residual canonicalization only after compiler closure

Important operational note:

- do not use `verification/leetcode/phase31_corpus_inventory.json` as the execution source for this phase without correcting its stale `audit/leetcode` paths
- authoritative corpus execution for this phase is the checked-in fixture tree under `audits/leetcode/`

## Current Execution Snapshot (2026-03-29)

- active lanes:
  - `workstream_1_cfg_optional_narrowing`
  - `workstream_2_unknown_optional_stabilization`
- local slice landed:
  - ternary (`if` expression) true-branch now consumes sequence guards during lowering
  - offset guard extraction now recognizes `i + k < len(seq)` and emits `IndexVarInRange` facts
  - `lower_if_expr` moved into `crates/sifr_hir/src/lower/if_expression.rs` to keep HIR module maintainability guardrails passing
  - dict key guard tokenization now supports tuple keys and boolean tuple members
  - dict subscript assignment now records key-presence sequence guards
  - exhaustive `if/else` merge now preserves branch-common sequence guards
  - singleton-row repeated matrix shapes now retain inner-length anchors for nested fixed-index reads
  - nested subscript assignment inference now refines dict/list element types
  - nested `max`/`min` inference now stabilizes return type from argument evidence
- validation evidence:
  - `cargo test -p sifr_hir lower::expressions_tests::test_if_expr_true_branch_sequence_guard_narrows_index -- --nocapture`
  - `cargo test -p sifr_hir lower::expressions_tests::test_if_expr_true_branch_sequence_guard_narrows_index_with_offset -- --nocapture`
  - `cargo test -q -p sifr_hir lower::guarded_index::tests::test_dict_key_presence_survives_exhaustive_if_branch_merge -- --nocapture`
  - `cargo test -q -p sifr_hir lower::guarded_index::tests::test_matrix_singleton_repeat_rows_allow_nested_fixed_index_reads -- --nocapture`
  - `cargo test -q -p sifr_hir lower::nested_function_tests::test_recursive_memoized_nested_helper_infers_deterministic_int_return -- --nocapture`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/optional_ifexpr_narrowing.sifr`
  - `cargo run -q -p sifr -- check audits/leetcode/0010_regular_expression_matching.sifr`
  - `cargo run -q -p sifr -- check audits/leetcode/0309_best_time_to_buy_and_sell_stock_with_cooldown.sifr`
  - `cargo run -q -p sifr -- check audits/leetcode/0494_target_sum.sifr`
  - `cargo run -q -p sifr -- check audits/leetcode/0518_coin_change_ii.sifr`
  - `scripts/run_all_tests.sh --profile quick`
- targeted fixture signal:
  - `0004_median_of_two_sorted_arrays` Optional ternary failures reduced from `4` to `2` incompatible-branch diagnostics (remaining `A[i]`/`B[j]` paths still carry `| None`)
  - `0013_roman_to_integer` remains open (`dict` index Optional contamination)
  - `0010_regular_expression_matching`, `0309_best_time_to_buy_and_sell_stock_with_cooldown`, `0494_target_sum`, and `0518_coin_change_ii` now type-check cleanly

## Core Contract and Guardrails

### What this phase must preserve

- `Option[T]` stays explicit; there is no implicit `Option[T] -> T` coercion
- guarded code should narrow precisely when the program proves non-`None`
- unguarded code should continue failing with clear diagnostics
- Python `if x:` truthiness must not become hidden Optional narrowing
- `global` / `nonlocal` unsupported-by-design remains unchanged
- recursive optional fields must be modeled through typed language rules, not lowered through unsafe shortcuts

### What this phase must not do

- no hidden auto-unwrap at call, return, index, arithmetic, or iteration sites
- no weakening of comparison rules to let `Any | None` silently flow through
- no fixture-specific narrowing exceptions
- no Python-compatibility behavior that bypasses Sifr's explicit safety model
- no residual-lane growth that turns compiler bugs into fixture rewrites

## Workstream Decomposition

### workstream_1_cfg_optional_narrowing

Status: in progress
Complexity: large

Owns:

- path-sensitive narrowing after `is None`, `is not None`, equivalent early-return guards, and branch merges
- use-site narrowing for arithmetic, comparison, indexing, calling, iteration, and return-path validation after guards

Representative failures:

- `0004_median_of_two_sorted_arrays`
- `0013_roman_to_integer`
- `0287_find_the_duplicate_number`
- `0802_find_eventual_safe_states`

Technical approach:

- make Optional facts a first-class per-program-point analysis output
- carry narrowing facts through branch entry, early exit, and merge blocks
- ensure downstream expression/type checking consumes narrowed facts instead of original binding types
- primary implementation loci:
  - `crates/sifr_type_system/src/narrow.rs`
  - `crates/sifr_hir/src/cfg.rs`
  - `crates/sifr_hir/src/lower/function_flow.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`

Definition of done:

- after an accepted non-`None` guard, the checked expression type at dominated use sites is `T`, not `T | None`
- branch joins only retain `| None` when one control-flow path genuinely allows absence
- targeted fixtures in this lane move out of Optional diagnostics without introducing truthiness-based narrowing
- before/after contract:
  - before: the checker still reports `int | None`, `list[T] | None`, or similar at a dominated use site
  - after: the same dominated use site is typed as concrete `T` or concrete collection type and any remaining failure is a different real bucket

Validation:

- focused unit tests for branch domination and merge behavior
- e2e fixtures covering arithmetic, index, call, iteration, and return-on-guard shapes
- targeted LeetCode reruns for lane owner fixtures
- existing test anchors to extend:
  - `crates/sifr_hir/src/lower/expressions_tests.rs`
  - `crates/sifr/tests/e2e/pass/optional_narrowing.sifr`
  - `crates/sifr/tests/e2e/pass/optional_arithmetic_narrowing_complex_flow.sifr`
  - `crates/sifr/tests/e2e/fail/optional_arithmetic_without_narrowing.sifr`
  - `crates/sifr/tests/e2e/fail/optional_arithmetic_reachable_after_partial_narrowing.sifr`

### workstream_2_unknown_optional_stabilization

Status: in progress
Complexity: medium

Owns:

- `Unknown | None` accumulation
- optional-aware join/meet behavior when one side is unresolved and the other is concretely optional or non-optional

Representative failures:

- `0010_regular_expression_matching`
- `0309_best_time_to_buy_and_sell_stock_with_cooldown`
- `0494_target_sum`
- `0518_coin_change_ii`

Technical approach:

- tighten join behavior between `Unknown`, `T`, and `None`
- ensure post-narrowing inference re-evaluates deferred result types rather than freezing `Unknown | None`
- prevent empty/deferred inference from poisoning final return types when later evidence is concrete
- primary implementation loci:
  - `crates/sifr_type_system/src/infer.rs`
  - `crates/sifr_type_system/src/union.rs`
  - `crates/sifr_type_system/src/check.rs`
  - `crates/sifr_hir/src/lower/generic_inference.rs`

Definition of done:

- `Unknown | None` does not survive to a final diagnostic when enough local evidence exists to resolve `T`
- return, assignment, and cache-value inference stabilize to concrete `T` or concrete `T | None` intentionally
- workstream-1 fixes do not regress into unresolved `Unknown | None` shadows
- before/after contract:
  - before: final diagnostics still expose `Unknown | None`
  - after: the same sites resolve to concrete `T`, concrete `T | None`, or a different explicit mismatch without `Unknown`

Validation:

- unit tests for joins involving `Unknown`, `None`, and concrete types
- e2e fixtures with recursive memoization and deferred cache initialization
- existing test anchors to extend:
  - `crates/sifr_type_system/src/infer.rs`
  - `crates/sifr_type_system/src/union.rs`
  - `crates/sifr_hir/src/lower/generic_inference.rs`

### workstream_3_container_element_refinement

Status: pending
Complexity: medium

Owns:

- refinement of `list[T | None]`, `dict[K, V | None]`, and empty-literal-backed containers after filtering/building
- element-type cleanup after guarded insertion, filtering, or cache population

Representative failures:

- `0023_merge_k_sorted_lists`
- `0115_distinct_subsequences`

Technical approach:

- separate binding-type narrowing from element-type refinement
- add explicit container refinement rules for known filtering/building patterns
- stabilize empty-literal inference so one optional write does not permanently poison the container when later writes prove concrete element type
- primary implementation loci:
  - `crates/sifr_hir/src/lower/container_literal_specialization.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_type_system/src/infer.rs`
  - `crates/sifr_type_system/src/union.rs`

Definition of done:

- filtered or proven-present element collections can be typed as `list[T]` where the program has removed `None`
- empty container inference does not retain a spurious `| None` element/value type after concrete population
- before/after contract:
  - before: filtered/populated containers still expose `list[T | None]` or `dict[K, V | None]`
  - after: the same containers refine to concrete element/value types when the program proves absence has been removed

Validation:

- unit tests for list/dict element refinement after guard-based filtering and cache updates
- targeted LeetCode reruns for list-of-node and memo/cache families
- existing test anchors to extend:
  - `crates/sifr_hir/src/lower/container_literal_specialization.rs`
  - `crates/sifr/tests/e2e/pass/list_pop_option.sifr`
  - `crates/sifr/tests/e2e/pass/forward_ref_listnode.sifr`

### workstream_4_recursive_optional_boundary_typing

Status: pending
Complexity: medium-large

Owns:

- recursive node, graph, and tree function boundaries where base-case absence and present-node types are inconsistently modeled

Representative failures:

- `0024_swap_nodes_in_pairs`
- `0104_maximum_depth_of_binary_tree`
- `0133_clone_graph`
- `0206_reverse_linked_list`

Technical approach:

- standardize the intended recursive contract for nullable node parameters and returns
- ensure function-call checking, recursive self-calls, and optional recursive fields use the same boundary rules
- keep this separate from generic narrowing so recursive structures do not need local pattern-specific hacks
- primary implementation loci:
  - `crates/sifr_hir/src/lower/typing_and_functions.rs`
  - `crates/sifr_hir/src/lower/classes.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_type_system/src/check.rs`

Definition of done:

- recursive APIs consistently accept `T | None` or `T` according to declared intent
- base-case `None` handling no longer leaks as an invalid argument where the language surface intends nullable recursive traversal
- recursive fixtures leave this bucket without broadening language rules beyond intended recursive optional support
- before/after contract:
  - before: recursive calls or constructor/call boundaries reject `None` or `T | None` where the declared recursive contract intends nullable traversal
  - after: those boundaries accept the intended nullable contract, and remaining failures are either non-optional or true surface mismatches

Validation:

- targeted unit and e2e tests across linked-list, tree, and graph recursive helpers
- reruns for representative recursive LeetCode fixtures
- existing test anchors to extend:
  - `crates/sifr/tests/e2e/pass/recursive_listnode.sifr`
  - `crates/sifr/tests/e2e/pass/recursive_treenode.sifr`
  - `crates/sifr/tests/e2e/pass/recursive_tree_traversal_runtime.sifr`
  - `crates/sifr/tests/e2e/fail/recursive_tree_attribute_without_narrowing.sifr`

### workstream_5_residual_canonicalization

Status: pending
Complexity: small

Owns:

- the residual cases left after compiler closure where the raw fixture truly encodes semantics Sifr intentionally rejects

Technical approach:

- only after workstreams 1-4 are rerun and reclassified
- adapt fixture shape to explicit Sifr semantics instead of broadening the compiler

Definition of done:

- each residual rewrite is justified as policy-preserving, not compiler-defect avoidance
- the residual lane stays small and auditable

Validation:

- explicit before/after classification note in the execution ledger

## Cross-Stream Dependencies

| Workstream | Depends on | Why |
| --- | --- | --- |
| `1` CFG narrowing | none | foundational fact propagation lane |
| `2` Unknown stabilization | `1` should lead | narrowing produces the concrete facts that deferred joins must consume |
| `3` container refinement | `1` strongly informs; `2` may influence | element refinement is unreliable if variable facts and joins are still unstable |
| `4` recursive optional boundaries | can run in parallel after boundary contract is confirmed | distinct surface contract, but reclassification improves after `1` |
| `5` residual canonicalization | `1-4` complete for current rerun | must only own true residual policy cases |

## Validation Strategy

Per workstream:

- add narrow unit tests at the owning compiler layer
- add at least one non-LeetCode e2e regression fixture for the generalized rule
- rerun the representative LeetCode fixtures for that workstream

Phase integration loop:

1. land one workstream root cause
2. rerun its representative fixtures
3. rerun the full LeetCode corpus against `audits/leetcode`
4. regenerate category notes in the execution ledger
5. reclassify moved failures before starting the next residual lane

The phase is not closed until:

- the Optional/None lane no longer dominates the corpus
- each moved fixture is either reclassified into a different legitimate bucket or cleared
- no workstream closure depended on hidden coercions or semantics weakening

Testing split requirement:

- compiler-rule changes must land with unit coverage in the owning crate before full-corpus claims are updated
- e2e additions must prove the generalized rule outside the LeetCode corpus
- full-corpus reruns are phase-level evidence, not the only test signal

## Scope

This phase owns:

- Optional/None narrowing facts
- `Unknown | None` stabilization
- container element refinement for Optional-heavy flows
- recursive nullable node boundary closure
- residual canonicalization only after compiler closure

This phase does not own:

- truthiness redesign
- `nonlocal` mutable capture support
- broad fixture rewriting as a first-pass strategy
- unrelated run-stage Rust build failures
- unrelated ownership-only or parse-safety-only categories

## Execution Order

### workstream_1_cfg_optional_narrowing

status: in progress

Owner: compiler/type-analysis

Acceptance target:

- guarded dominated use sites are checked as concrete `T`
- merge blocks retain `| None` only when one live path still permits absence
- representative diagnostics change away from Optional-leak shapes rather than silently weakening checks

### workstream_2_unknown_optional_stabilization

status: in progress

Owner: type inference / join logic

Acceptance target:

- representative diagnostics no longer expose `Unknown | None`
- deferred joins stabilize to concrete `T` or intentional `T | None`
- narrowing fixes from wave 1 do not regress into unresolved `Unknown` joins

### workstream_3_container_element_refinement

status: pending

Owner: type-system/container inference

Acceptance target:

- canary fixtures refine `list[T | None]` to `list[T]` after proven filtering/population
- empty-literal-backed caches stabilize to concrete value types after population
- representative container diagnostics move out of Optional-element contamination

### workstream_4_recursive_optional_boundary_typing

status: pending

Owner: recursive type surface / call checking

Acceptance target:

- representative recursive boundaries accept intended nullable traversal contracts
- base-case `None` handling no longer appears as an invalid argument where nullable recursion is intended
- recursive fixtures leave the Optional bucket without introducing truthiness or hidden unwrap semantics

### workstream_5_residual_reclassification_and_canonicalization

status: pending

Owner: corpus closure / policy review

Acceptance target:

- remaining cases are either cleared or explicitly justified as residual policy-preserving rewrites
