# Ad-hoc Phase: Optional/Any Root-Cause Closure (2026-04-06)

## Snapshot

Source baseline:

- `verification/leetcode/full_corpus_current_results_20260406_live_rerun1.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260406_live_rerun1.json`
- `verification/leetcode/phase_apr06_on_au_root_cause_map.json`
- `verification/leetcode/phase_apr06_on_au_root_cause_map.csv`

Current category counts in scope:

- `optional_none_flow_and_narrowing_gap`: `30`
- `any_unknown_typing_and_container_specialization_gap`: `28`

Total in-scope fixtures for this phase: `58`

Resolution-mode split from root-cause map:

- `compiler`: `51`
- `both`: `6`
- `adaptation`: `1`

## Root-Cause Breakdown

### A) Optional/None flow and narrowing gap (`30`)

Sub-root causes:

- `ON-1-optional-arithmetic-operator-leak`: `15` -> `compiler`
- `ON-2-optional-container-boundary-leak`: `6` -> `compiler`
- `ON-3-optional-element-contamination`: `3` -> `compiler`
- `ON-4-optional-contract-and-return-closure`: `4` -> `both`
- `ON-5-optional-string-surface-guarding`: `2` -> `both`

Representative diagnostics:

- `unsupported operand type(s) for +/-//` with `int | None`
- `cannot iterate over type 'list[int] | None'`
- `type 'None | list[int]' has no method 'append'`
- `return type mismatch: expected 'int/bool', got 'None | int/bool'`
- `type 'None | str' has no method 'replace'`

Root cause summary:

- Optional unions are not consistently eliminated at dominated use sites.
- Optional container bindings are leaking into iteration/index/method paths.
- Element refinement after guarded population is incomplete.
- A small residual set mixes compiler closure with fixture-side explicit guard intent.

Decision:

- Compiler-first closure.
- Adaptation only after compiler lanes close and only for policy-consistent explicit guard canonicalization.

### B) Any/Unknown typing and container specialization gap (`28`)

Sub-root causes:

- `AU-1-heapq-unknown-container-shape`: `4` -> `compiler`
- `AU-2-any-unknown-flow-and-operator-leak`: `16` -> `compiler`
- `AU-3-any-unknown-optional-bridge`: `5` -> `compiler`
- `AU-4-unknown-stdlib-contract-surface`: `1` -> `compiler`
- `AU-5-signature-annotation-required`: `1` -> `adaptation`
- `AU-6-list-unknown-specialization`: `1` -> `compiler`

Representative diagnostics:

- `__compat_sifr_heapq_heapify`: expected `list[T]`, got `Unknown`
- `cannot index type 'Any' / 'Unknown'`
- `'in'/'not in' operator not supported for type 'Unknown'`
- `for-loop iterable must have a statically-known element type, got 'Unknown'`
- `return type mismatch: expected ..., got 'Any | None'`
- `parameter ... is missing a type annotation`

Root cause summary:

- Container specialization and join stabilization still leak `Any`/`Unknown` into operator/index/call boundaries.
- Optional bridge collapse for `Any | None` / `Unknown | None` is incomplete.
- Stdlib-compat constructors are receiving unresolved container/mapping types.
- One fixture requires canonical Sifr annotation adaptation.

Decision:

- Predominantly compiler closure.
- Keep the annotation case adaptation-only under current Sifr policy.

## Compiler vs Adaptation Decision Matrix

Compiler workstreams to implement:

1. `W1-ON-arithmetic-and-operator-narrowing` (`ON-1`)
2. `W2-ON-container-boundary-and-element-refinement` (`ON-2`, `ON-3`)
3. `W3-AU-flow-stabilization-and-operator-safety` (`AU-2`, `AU-3`)
4. `W4-AU-compat-container-contract-typing` (`AU-1`, `AU-4`, `AU-6`)
5. `W5-ON-contract-return-closure` compiler slice (`ON-4`, `ON-5` compiler side)

Adaptation workstreams:

1. `A1-signature-annotation-required` (`AU-5`)
2. `A2-explicit-guard-canonicalization` residual `ON-4`/`ON-5` only after `W5`

## Ready-to-Implement Phase Plan

Phase ID: `ad_hoc_optional_any_root_cause_closure`

### Workstream W1: Optional arithmetic/operator narrowing

Owner: compiler

Goal:

- eliminate `T | None` from arithmetic/operator positions once dominated by accepted guards.

Primary loci:

- `crates/sifr_type_system/src/narrow.rs`
- `crates/sifr_type_system/src/check.rs`
- `crates/sifr_hir/src/lower/function_flow.rs`
- `crates/sifr_hir/src/lower/expressions.rs`

Acceptance:

- all `ON-1` signatures removed from focused fixture rerun.
- no regression in optional narrowing e2e fixtures.

### Workstream W2: Optional container boundary + element refinement

Owner: compiler

Goal:

- stop `None | container` leaks at iteration/index/method sites.
- refine element domains after guarded writes/build patterns.

Primary loci:

- `crates/sifr_hir/src/lower/container_literal_specialization.rs`
- `crates/sifr_hir/src/lower/guarded_index.rs`
- `crates/sifr_type_system/src/infer.rs`
- `crates/sifr_type_system/src/union.rs`

Acceptance:

- `ON-2` and `ON-3` signatures removed from focused rerun.
- no regressions in list/dict specialization tests.

### Workstream W3: Any/Unknown flow stabilization and operator safety

Owner: compiler

Goal:

- prevent `Any`/`Unknown` escape at joins and downstream operator/index/call usage.
- stabilize `Any|None` and `Unknown|None` before boundary checks.

Primary loci:

- `crates/sifr_type_system/src/infer.rs`
- `crates/sifr_type_system/src/union.rs`
- `crates/sifr_type_system/src/check.rs`

Acceptance:

- `AU-2` and `AU-3` signatures removed from focused rerun.
- no new `Any`/`Unknown` regressions in existing focus4 tests.

### Workstream W4: Compat container contract typing

Owner: compiler

Goal:

- ensure heap/defaultdict/typed-list compat entry points receive stabilized concrete container types.

Primary loci:

- `crates/sifr_hir/src/lower/container_literal_specialization.rs`
- `crates/sifr_codegen/src` compat lowering for heap/defaultdict call paths
- `crates/sifr_type_system/src/check.rs`

Acceptance:

- `AU-1`, `AU-4`, and `AU-6` signatures removed from focused rerun.

### Workstream W5: Optional contract/return closure (compiler slice)

Owner: compiler

Goal:

- close false optional return/argument unions where control-flow is semantically complete.

Primary loci:

- `crates/sifr_hir/src/cfg.rs`
- `crates/sifr_hir/src/lower/function_flow.rs`
- `crates/sifr_type_system/src/check.rs`

Acceptance:

- compiler-owned part of `ON-4` and `ON-5` removed.
- remaining residuals are explicit adaptation candidates only.

### Adaptation A1: Required signature annotation

Owner: fixture canonicalization

Goal:

- canonicalize the explicit annotation-required case (`AU-5`) without changing compiler policy.

Acceptance:

- fixture compiles under current annotation rules.

### Adaptation A2: Explicit guard canonicalization (residual)

Owner: fixture canonicalization

Goal:

- apply explicit guard rewrites only to residual `ON-4`/`ON-5` cases that remain after `W5`.

Acceptance:

- adaptation set is small, auditable, and does not broaden language semantics.

## Phase Exit Gates

1. Root-cause presence gate:

- `ON-1..ON-5` and `AU-1..AU-6` targeted signatures removed from focused rerun or explicitly transferred to approved adaptation list.

2. Full-corpus gate:

- new full-corpus rerun artifact generated.
- taxonomy regenerated.
- no net regressions outside approved adaptation transitions.
- the `53` non-targeted fixtures across all other taxonomy categories must not change status (any change is a regression requiring investigation).

3. Policy gate:

- no weakening of ownership/mutability, parse safety, or unsupported `nonlocal` mutable capture policy.

## Validation Commands

- `cargo build --release -p sifr`
- focused fixture rerun for all `58` mapped fixtures
- full rerun:
  - `python3 /tmp/sifr_full_leetcode_scan_<date>.py`
- taxonomy regeneration for the new full-corpus artifact
- `scripts/run_all_tests.sh --profile quick`

## Expected Outcome

- Close the compiler-owned majority (`51/58`) directly.
- Minimize adaptation to explicit policy-consistent cases (`<=7`, currently projected `1` mandatory + residual guard cases only if still needed).
- Produce a measurable reduction in both `optional_none_flow_and_narrowing_gap` and `any_unknown_typing_and_container_specialization_gap` in the next full run.
