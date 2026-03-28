# Phase 31 Strategy Synthesis Review

Status: current assessment with production-grade review confirmation on 2026-03-26

Inputs reviewed:
- `verification/leetcode/phase31_current_full_results_20260321.json`
- `verification/leetcode/phase31_review_pass1_full_results_v2.json`
- `issues/phase31-ad-hoc-followup-milestones.md`
- `reviews/phase31-ad-hoc-followup-milestones-review-pass-1.md`
- `reviews/phase31-ad-hoc-followup-milestones-review-pass-2.md`
- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`
- `issues/ad-hoc-full-nested-function-pipeline.md`
- `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
- `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
- `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`

## Question

For the current remaining Phase 31 seed-corpus failures, decide which work still belongs to:

1. a broader prerequisite phase,
2. canonical Sifr fixture adaptation,
3. ordinary compiler/runtime closure,
4. a targeted compiler feature,
5. or a combination of those.

## Current Measured State

Fresh current seed-corpus rerun:

- `PASS=50`
- `CHECK_ERROR=0`
- `RUN_ERROR=0`

Current passing cases:

- all 50 seed-corpus cases are currently passing in manifest mode (`embedded_asserts`)

Execution delta (`2026-03-26`, `m31_a` slice 15):

- `0502_ipo` and `0743_network_delay_time` were canonicalized to encoded-int heap forms and now run green as `NO_ORACLE` in targeted reruns.
- `m31_a_optional_flow_completion` is closed for its owner scope (`0127`, `0322`, `0502`, `0743`).

Execution delta (`2026-03-26`, `m31_b` slice 1):

- tuple-unpack lowering now supports attribute targets (`obj.a, obj.b = ...`) in HIR/codegen.
- canonical closure moved `0295`, `0703`, `0997`, and `1209` to green targeted statuses (`NO_ORACLE`/`PASS`).
- `0226` is now isolated to a run-stage boxed optional-tree lowering gap.

Execution delta (`2026-03-26`, `m31_b` slice 2):

- recursive optional-field assignment boxing is now handled in codegen.
- `0226_invert_binary_tree` moved from `RUN_ERROR` to `NO_ORACLE` in targeted reruns.
- `m31_b_destructuring_and_composite_lvalues` owner scope is now closed.

Execution delta (`2026-03-26`, `m31_d` slice 1):

- canonical nested-helper closure landed across all eight owner cases (`0017`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912`).
- `0052` used the documented canonical workaround route for recursive `nonlocal` mutation instead of scope-expanding nested recursive `nonlocal`.
- targeted status moved to `PASS=6`, `NO_ORACLE=2` and `m31_d_nested_function_pipeline_completion` owner scope is now closed.

Execution delta (`2026-03-26`, `m31_e` slice 1):

- canonical recursive-tree closure landed across `0100`, `0102`, and `0235`.
- targeted status moved to `NO_ORACLE=3` and `m31_e_recursive_tree_surface_leetcode_closure` owner scope is now closed.

Execution delta (`2026-03-26`, `m31_l` slice 1):

- canonical tree local-state closure landed for `0110`.
- targeted status moved to `NO_ORACLE=1` and `m31_l_tree_local_state_follow_on_closure` owner scope is now closed.

Execution delta (`2026-03-26`, `m31_h` slice 1):

- canonical local-binding/shadowing closure landed for `0015` and `0424`.
- targeted status moved to `PASS=2` and `m31_h_local_name_binding_and_shadowing` owner scope is now closed.

Execution delta (`2026-03-26`, `m31_j` slice 1):

- canonical `own mut` closure landed for `1299`.
- targeted status moved to `PASS=1` and `m31_j_own_mut_leetcode_closure` owner scope is now closed.

Execution delta (`2026-03-26`, `m31_k` slice 1):

- canonical parse-safe fixture normalization landed for `0043`.
- targeted status moved to `PASS=1` and `m31_k_canonical_sifr_fixture_normalization` owner scope is now closed.

Execution delta (`2026-03-26`, `m31_i` slice 1):

- canonical one-solution fixture normalization landed for `0215` and `1046`.
- targeted status moved to `NO_ORACLE=2` and `m31_i_corpus_fixture_canonicalization_for_multi_solution_files` owner scope is now closed.

Execution delta (`2026-03-26`, external review pass 1 hardening):

- upgraded 14 `no_oracle` seed entries with embedded assertions to `embedded_asserts` and revalidated (`PASS=14`).
- closed regression triplet `0007`, `0009`, and `0151` via canonical explicit-mut adaptation (`PASS=3`).
- closed residual pair `0001`, `0242` and reran full seed corpus to green (`PASS=50`).

## Current Conclusion

The current strategy is:

- no broad prerequisite phase remains open for the current seed corpus
- canonical fixture adaptation and follow-on closure work has been completed for current seed scope
- external review pass 1 findings were resolved with root-cause fixes and verification-policy alignment
- no active failure bucket remains in the current seed corpus (`PASS=50`)

## Phases Already Consumed

These should no longer be described as pending prerequisites in the Phase 31 strategy:

### Recursive Types

- `issues/ad-hoc-full-recursive-type-feature.md` has already landed its scoped feature work
- Phase 31 impact:
  - `0100`, `0102`, and `0235` are now `m31_e` closure work, not blockers on a future prerequisite
  - any residual recursive-tree failure should be treated as a concrete closure gap or sent back to the recursive-type phase with a specific gap report

### `own mut`

- `issues/ad-hoc-own-mut-parameter-convention.md` is complete
- Phase 31 impact:
  - `1299` is no longer blocked on missing language support
  - it is now blocked on canonical fixture adaptation to explicit `own mut`

### Nested Functions

- `issues/ad-hoc-full-nested-function-pipeline.md` is complete
- Phase 31 impact:
  - remaining nested-helper cases are no longer evidence that the broad nested-function phase is missing
  - they are now residual follow-on bugs, unsupported subshapes, or downstream closure issues

## Current Open Buckets

Post-review status: no active open bucket remains for the current Phase 31 seed corpus. The bucket breakdown below is retained as historical planning context.

### Snapshot Regression Note

Compared with the warmed `2026-03-13` rerun (`PASS=15`, `CHECK_ERROR=35`, `RUN_ERROR=0`), the current `2026-03-21` snapshot regressed to `PASS=13`, `CHECK_ERROR=36`, `RUN_ERROR=1`.

Concrete status changes:

- `0007`: `PASS -> CHECK_ERROR`
- `0009`: `PASS -> CHECK_ERROR`
- `0039`: `CHECK_ERROR -> PASS`
- `0078`: `CHECK_ERROR -> RUN_ERROR`
- `0151`: `PASS -> CHECK_ERROR`

Interpretation:

- the current plan should treat this regression as real current scope rather than assuming the older `PASS=15` state is still authoritative
- `0078` remains the highest-priority correctness regression because it is now the only run-time failure in the seed corpus

### 1. Canonical Sifr mutability / ownership adaptation

These fixtures now need explicit canonical Sifr parameter mutability or ownership at the source boundary:

- `0007`
- `0009`
- `0015`
- `0090`
- `0127`
- `0151`
- `0226`
- `0746`
- `0912`
- `1299`

Representative current failure shapes:

- `cannot reassign immutable parameter ... add mut to the parameter declaration`
- `cannot mutate through immutable parameter ... add mut to the parameter declaration`
- `cannot return borrowed parameter ... add own at the signature boundary or return clone()`

Interpretation:

- this is now a real current bucket in the seed corpus
- it is mostly canonical source adaptation work, not a missing feature phase

### 2. Canonical fixture normalization plus residual closure

These still need canonicalization before their remaining compiler bugs can be judged cleanly:

- none (`m31_i` owner scope closed in slice 1)

Current status:

- no active residual remains in this bucket after canonical fixture normalization

### 3. Container-literal / `Any` specialization targeted compiler feature

This bucket still clearly stands:

- `0001`
- `0242`
- `0424`
- `0523`
- `0560`

Current representative failures:

- `cannot index type 'dict[Any, Any]' with ...`
- `unsupported operand type(s) for +: 'int' and 'Any'`
- `unsupported operand type(s) for -: 'int' and 'Any'`

Interpretation:

- this is not just ordinary closure work
- it is a targeted compiler/type-inference feature that should still be delivered inside the Phase 31 carry-forward rather than spun into a separate broad prerequisite

### 4. Optional-flow / arithmetic proof closure

This bucket still stands, although the exact cases have shifted:

- `0053`
- `0238`
- `0322`
- parts of `0015`
- parts of `0746`

Current representative failures:

- `type mismatch: expected 'int', got 'int | None'`
- `unsupported operand type(s) for *: 'int | None' and 'int'`
- `return type mismatch: expected 'int', got 'int | None'`

### 5. Residual nested-function follow-ons and explicit boundary decisions

These are no longer active residuals:

- none (m31_d owner scope closed in slice 1)

Current representative failures:

- none remaining in this bucket

Interpretation:

- no active nested-function residual currently remains in this bucket

### 6. Destructuring / class-surface follow-on closure

These are still active residuals:

- none (m31_b owner scope closed in slice 2)

Current representative failures:

- none remaining in this bucket

Interpretation:

- no active destructuring/class-surface residual currently remains in this bucket

### 7. Iterator / comparability / concrete iterable follow-on closure

These cases still fail, but they should not currently be explained as missing the completed iteration-closure phases:

- none (`m31_d` and `m31_i` owner scopes closed)

Current representative failures:

- none remaining in this bucket

Interpretation:

- no active iterator/comparability residual currently remains in this bucket

### 8. Single-case residuals and small current families

- `0050`
  - now a concrete float/int comparison closure bug, not primarily a nested-helper inference problem
- `0110`
  - now a bool/list/local-state closure bug, not primarily a recursive-attribute blocker

## Historical Case-by-Case Classification

Post-review current state superseding the historical table:

- all 50 seed-corpus ids are currently `PASS` in `verification/leetcode/phase31_review_pass1_full_results_v2.json`

| ID | Current classification | Current primary owner |
| --- | --- | --- |
| `0001` | targeted compiler feature | container specialization |
| `0007` | canonical fixture adaptation | explicit `mut` |
| `0009` | canonical fixture adaptation | explicit `mut` |
| `0015` | closed in `m31_h` slice 1 | canonical local binding/shadowing closure (`PASS`) |
| `0017` | closed in `m31_d` slice 1 | canonical nested-helper closure (`PASS`) |
| `0043` | closed in `m31_k` slice 1 | canonical parse-safe fixture normalization (`PASS`) |
| `0050` | closed in `m31_d` slice 1 | canonical nested-helper closure (`PASS`) |
| `0052` | closed in `m31_d` slice 1 | canonical workaround closure for recursive `nonlocal` (`PASS`) |
| `0053` | normal closure | optional-flow |
| `0078` | closed in `m31_d` slice 1 | canonical assertion-order closure (`PASS`) |
| `0090` | closed in `m31_d` slice 1 | canonical nested-helper + mutability closure (`PASS`) |
| `0100` | closed in `m31_e` slice 1 | canonical recursive-tree closure (`NO_ORACLE`) |
| `0102` | closed in `m31_e` slice 1 | canonical recursive-tree closure (`NO_ORACLE`) |
| `0110` | closed in `m31_l` slice 1 | canonical tree local-state closure (`NO_ORACLE`) |
| `0127` | canonical fixture adaptation + closure | `mut` + residual optional/iterable typing |
| `0151` | canonical fixture adaptation | explicit `mut` |
| `0207` | closed in `m31_d` slice 1 | canonical nested-helper closure (`NO_ORACLE`) |
| `0215` | closed in `m31_i` slice 1 | canonical one-solution fixture normalization (`NO_ORACLE`) |
| `0226` | closed in `m31_b` slice 2 | recursive optional-field boxing closure (`NO_ORACLE`) |
| `0235` | closed in `m31_e` slice 1 | canonical recursive-tree closure (`NO_ORACLE`) |
| `0238` | normal closure | optional-flow |
| `0242` | targeted compiler feature | container specialization |
| `0295` | closed in `m31_b` slice 1 | canonical sorted-surface implementation (`NO_ORACLE`) |
| `0322` | normal closure | optional-flow |
| `0424` | closed in `m31_h` slice 1 | canonical local binding/shadowing closure (`PASS`) |
| `0502` | closed in `m31_a` slice 15 | canonical encoded-heap form (`NO_ORACLE`) |
| `0523` | targeted compiler feature | container specialization |
| `0560` | targeted compiler feature | container specialization |
| `0684` | closed in `m31_d` slice 1 | canonical DSU helper closure (`NO_ORACLE`) |
| `0703` | closed in `m31_b` slice 1 | canonical sorted-surface implementation (`NO_ORACLE`) |
| `0743` | closed in `m31_a` slice 15 | canonical encoded-heap form (`NO_ORACLE`) |
| `0746` | canonical fixture adaptation + closure | `mut` + optional-flow |
| `0912` | closed in `m31_d` slice 1 | canonical nested-helper closure (`PASS`) |
| `0997` | closed in `m31_b` slice 1 | canonical guarded dict-surface implementation (`PASS`) |
| `1046` | closed in `m31_i` slice 1 | canonical one-solution fixture normalization (`NO_ORACLE`) |
| `1209` | closed in `m31_b` slice 1 | canonical string-run reduction implementation (`PASS`) |
| `1299` | closed in `m31_j` slice 1 | canonical `own mut` closure (`PASS`) |

## Bottom Line

The currently valid Phase 31 strategy is:

- all current seed-corpus cases are now closed and assertion-verified (`PASS=50`)
- external review pass 1 findings were resolved with root-cause fixes (oracle-mode alignment + residual fixture closure)
- no active failure bucket remains for the current Phase 31 seed corpus
