# Phase 31 Strategy Synthesis Review

Status: current assessment on 2026-03-26

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
4. a targeted compiler feature,
5. or a combination of those.

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

- no broad prerequisite phase remains open for the current seed corpus
- `own mut` is no longer a pending prerequisite because that phase is complete
- the nested-function phase is no longer a pending prerequisite because that phase is complete
- container-literal specialization should be treated as a targeted compiler/type-inference feature inside the carry-forward plan, not as a trivial cleanup item
- most of the remaining failures are now:
  - canonical Sifr source adaptation work,
  - ordinary residual compiler/runtime closure,
  - explicit unsupported-shape boundaries that need a product decision or canonical workaround,
  - or narrow follow-on bugs that remain after broader prerequisite phases already landed

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
- `0043`
- `0090`
- `0127`
- `0151`
- `0215`
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

- `0043`
- `0215`
- `1046`

Current status:

- `0043` still mixes raw parse-safety mismatch with downstream compiler errors
- `0215` still contains multiple top-level solution definitions and also hits mutability / return-typing follow-ons
- `1046` still needs canonicalization and still degrades into `Any`-driven heap/math follow-ons

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

These are no longer blocked on the broad nested-function phase itself, but they split into residual closure work and one explicit language-boundary decision:

- `0017`
- `0052`
- `0078`
- `0207`
- `0684`

Current representative failures:

- `dict[str, str]` indexed by `str | None`
- recursive nested `nonlocal` mutation still rejected explicitly at the current language boundary
- `Unknown`/`Any` flow into indexing
- one run-time assertion failure:
  - `0078`

Interpretation:

- `0017`, `0078`, `0207`, and `0684` remain residual compiler/runtime closure work
- `0052` is not routine cleanup; it needs an explicit decision between extending recursive nested `nonlocal` support and adopting a canonical Sifr workaround pattern

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

Interpretation:

- `0226` is a tree-shaped case, but its current compiler blocker is attribute destructuring after canonical `own` adaptation, so the closure owner is `m31_b` rather than `m31_e`

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

Interpretation:

- `0502` is best treated as an iterator/comparability case with downstream `Any` leakage that may shrink once the container-specialization feature lands

### 8. Single-case residuals and small current families

- `0050`
  - now a concrete float/int comparison closure bug, not primarily a nested-helper inference problem
- `0295`
  - also includes a float/int comparison mismatch, so mixed numeric comparison is a small current family to watch rather than a one-off
- `0110`
  - now a bool/list/local-state closure bug, not primarily a recursive-attribute blocker

## Current Case-by-Case Classification

| ID | Current classification | Current primary owner |
| --- | --- | --- |
| `0001` | targeted compiler feature | container specialization |
| `0007` | canonical fixture adaptation | explicit `mut` |
| `0009` | canonical fixture adaptation | explicit `mut` |
| `0015` | canonical fixture adaptation + closure | `mut` + local binding / optional-flow |
| `0017` | normal closure | residual nested/iterable typing |
| `0043` | canonical fixture adaptation + closure | canonical rewrite + mutability/typing cleanup |
| `0050` | normal closure | numeric/typing cleanup |
| `0052` | explicit language-boundary decision | nested recursive `nonlocal` support or canonical workaround |
| `0053` | normal closure | optional-flow |
| `0078` | normal closure | run-time regression |
| `0090` | canonical fixture adaptation + closure | explicit `mut` + list typing |
| `0100` | normal closure | recursive-tree residual under `m31_e` |
| `0102` | normal closure | recursive-tree residual under `m31_e` |
| `0110` | normal closure | bool/local-state follow-on |
| `0127` | canonical fixture adaptation + closure | `mut` + residual optional/iterable typing |
| `0151` | canonical fixture adaptation | explicit `mut` |
| `0207` | normal closure | residual nested/destructuring/iterable typing |
| `0215` | canonical fixture adaptation + closure | multi-solution canonicalization + `mut` + return typing |
| `0226` | canonical fixture adaptation + closure | `own` adaptation + attribute destructuring under `m31_b` |
| `0235` | normal closure | recursive-tree residual under `m31_e` |
| `0238` | normal closure | optional-flow |
| `0242` | targeted compiler feature | container specialization |
| `0295` | normal closure | numeric comparison + destructuring/class-surface follow-on |
| `0322` | normal closure | optional-flow |
| `0424` | targeted compiler feature + closure | container specialization + local name binding follow-on |
| `0502` | normal closure | iterator/comparable residuals with downstream `Any` leakage |
| `0523` | targeted compiler feature | container specialization |
| `0560` | targeted compiler feature | container specialization |
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

- no broad prerequisite phase remains open for the current seed corpus
- treat `own mut` and nested functions as already-landed dependencies, not future blockers
- treat recursive-tree failures as current `m31_e` closure work rather than waiting on another prerequisite
- treat a significant part of the remaining seed corpus as canonical Sifr fixture adaptation, especially around explicit `mut` / `own mut`
- treat container-literal specialization as a targeted compiler feature-sized item inside the carry-forward plan
- treat `0052` as an explicit nested-function boundary decision rather than routine cleanup
- keep the rest in ordinary closure buckets: optional-flow, destructuring/class follow-ons, iterator/comparability follow-ons, numeric follow-ons, and one run-time regression
