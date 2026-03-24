# Phase 31 Strategy Synthesis Review

Status: current assessment on 2026-03-24

Inputs reviewed:
- `verification/leetcode/phase31_current_full_results_20260321.json`
- `issues/phase31-ad-hoc-followup-milestones.md`
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
4. or a combination of those.

## Current Measured State

Fresh current seed-corpus rerun:

- `PASS=13`
- `CHECK_ERROR=36`
- `RUN_ERROR=1`

Current passing cases:

- `0003`
- `0014`
- `0039`
- `0042`
- `0069`
- `0070`
- `0198`
- `0209`
- `0217`
- `1143`
- `1456`
- `1768`
- `2235`

## Current Conclusion

The current strategy is:

- the only clearly remaining broad prerequisite for the Phase 31 seed corpus is the recursive-type phase
- `own mut` is no longer a pending prerequisite because that phase is complete
- the nested-function phase is no longer a pending prerequisite because that phase is complete
- most of the remaining failures are now:
  - canonical Sifr source adaptation work,
  - ordinary residual compiler/runtime closure,
  - or narrow follow-on bugs that remain after broader prerequisite phases already landed

## Broad Prerequisite Still Relevant

### `prereq_recursive_types`

This still stands as the main remaining cross-phase dependency for the seed corpus.

Current clearly recursive/tree-driven cases:

- `0100`
- `0102`
- `0235`

Current failure shapes still point at recursive-type follow-on closure:

- attribute-expression rejection on recursive nodes
- `TreeNode` boundary/type resolution mismatches
- tree helper signatures still not aligning cleanly with recursive node usage

## Phases Already Consumed

These should no longer be described as pending prerequisites in the Phase 31 strategy:

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

### 1. Canonical Sifr mutability / ownership adaptation

These fixtures now need explicit canonical Sifr parameter mutability or ownership at the source boundary:

- `0007`
- `0009`
- `0015`
- `0043`
- `0090`
- `0127`
- `0151`
- `0215`
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

- `0043`
- `0215`
- `1046`

Current status:

- `0043` still mixes raw parse-safety mismatch with downstream compiler errors
- `0215` still contains multiple top-level solution definitions and also hits mutability / return-typing follow-ons
- `1046` still needs canonicalization and still degrades into `Any`-driven heap/math follow-ons

### 3. Container-literal / `Any` specialization closure

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

### 5. Residual nested-function follow-on bugs

These are no longer blocked on the broad nested-function phase itself, but they still need cleanup after that phase:

- `0017`
- `0052`
- `0078`
- `0207`
- `0684`

Current representative failures:

- `dict[str, str]` indexed by `str | None`
- recursive nested `nonlocal` mutation still rejected explicitly
- `Unknown`/`Any` flow into indexing
- one run-time assertion failure:
  - `0078`

### 6. Destructuring / class-surface follow-on closure

These are still active residuals:

- `0226`
- `0295`
- `0703`
- `0743`
- `0997`
- `1209`

Current representative failures:

- `tuple unpacking target must be a simple name`
- `for loop tuple target expects iterable elements of tuple type, got 'list[int]'`
- class-field follow-ons like missing `large` / `minHeap`
- composite mutation target failures

### 7. Iterator / comparability / concrete iterable follow-on closure

These cases still fail, but they should not currently be explained as missing the completed iteration-closure phases:

- `0017`
- `0207`
- `0502`
- `0743`
- `1046`

Current representative failures:

- `for-loop iterable must have a statically-known element type`
- `cannot iterate over type 'Iterator[tuple[int, int]]'`
- tuple comparability / heap constraints not closing
- `Any`-typed heap data still leaking into math and indexing

### 8. Single-case residuals that have moved buckets

- `0050`
  - now a concrete float/int comparison closure bug, not primarily a nested-helper inference problem
- `0110`
  - now a bool/list/local-state closure bug, not primarily a recursive-attribute blocker

## Current Case-by-Case Classification

| ID | Current classification | Current primary owner |
| --- | --- | --- |
| `0001` | normal closure | container specialization |
| `0007` | canonical fixture adaptation | explicit `mut` |
| `0009` | canonical fixture adaptation | explicit `mut` |
| `0015` | canonical fixture adaptation + closure | `mut` + local binding / optional-flow |
| `0017` | normal closure | residual nested/iterable typing |
| `0043` | canonical fixture adaptation + closure | canonical rewrite + mutability/typing cleanup |
| `0050` | normal closure | numeric/typing cleanup |
| `0052` | normal closure | residual nested unsupported subshape |
| `0053` | normal closure | optional-flow |
| `0078` | normal closure | run-time regression |
| `0090` | canonical fixture adaptation + closure | explicit `mut` + list typing |
| `0100` | prerequisite + closure | recursive types |
| `0102` | prerequisite + closure | recursive types |
| `0110` | normal closure | bool/local-state follow-on |
| `0127` | canonical fixture adaptation + closure | `mut` + residual optional/iterable typing |
| `0151` | canonical fixture adaptation | explicit `mut` |
| `0207` | normal closure | residual nested/destructuring/iterable typing |
| `0215` | canonical fixture adaptation + closure | multi-solution canonicalization + `mut` + return typing |
| `0226` | canonical fixture adaptation + closure | ownership + destructuring follow-on |
| `0235` | prerequisite + closure | recursive types |
| `0238` | normal closure | optional-flow |
| `0242` | normal closure | container specialization |
| `0295` | normal closure | destructuring/class-surface follow-on |
| `0322` | normal closure | optional-flow |
| `0424` | normal closure | container specialization |
| `0502` | normal closure | iterator/comparable residuals |
| `0523` | normal closure | container specialization |
| `0560` | normal closure | container specialization |
| `0684` | normal closure | residual nested/index typing |
| `0703` | normal closure | destructuring/class-surface follow-on |
| `0743` | normal closure | destructuring/iterator/comparable residuals |
| `0746` | canonical fixture adaptation + closure | `mut` + optional-flow |
| `0912` | canonical fixture adaptation + closure | `mut` |
| `0997` | normal closure | destructuring |
| `1046` | canonical fixture adaptation + closure | multi-solution canonicalization + `Any`/heap residuals |
| `1209` | normal closure | composite mutation / destructuring follow-on |
| `1299` | canonical fixture adaptation | explicit `own mut` rewrite |

## Bottom Line

The currently valid Phase 31 strategy is:

- keep recursive types as the main remaining broader prerequisite
- treat `own mut` and nested functions as already-landed dependencies, not future blockers
- treat a significant part of the remaining seed corpus as canonical Sifr fixture adaptation, especially around explicit `mut` / `own mut`
- keep the rest in ordinary closure buckets: container specialization, optional-flow, destructuring/class follow-ons, iterator/comparability follow-ons, and one run-time regression
