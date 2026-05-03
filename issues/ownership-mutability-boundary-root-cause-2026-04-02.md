# Ownership/Mutability Boundary Root-Cause Analysis

Date: 2026-04-02
Source run: `verification/leetcode/full_corpus_current_results_20260402_live.json`
Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260402_live.json`
Breakdown artifact: `verification/leetcode/ownership_mutability_boundary_breakdown_20260402_live.json`

## Scope

Current bucket:

- `48` fixtures in `ownership_and_mutability_boundary`

## Current Decomposition

1. `31` `immutable_parameter_mutation`
   - diagnostic shape: `cannot mutate through immutable parameter ... add mut`
2. `11` `immutable_parameter_reassignment`
   - diagnostic shape: `cannot reassign immutable parameter ... add mut`
3. `4` `borrowed_parameter_escape_store`
   - diagnostic shape: `cannot store borrowed parameter ... add own or clone`
4. `2` `borrowed_parameter_escape_return`
   - diagnostic shape: `cannot return borrowed parameter ... add own or clone`

Note on overlap: sub-bucket assignment is based on the first emitted diagnostic. A subset of fixtures (estimated `3-5`, including `0075_sort_colors`, `0669_trim_a_binary_search_tree`, `0701_insert_into_a_binary_search_tree`) likely need compound boundary annotations (`own mut`) spanning both mutability and ownership dimensions.

High-frequency parameter names in this bucket:

- `nums` (`11`)
- `node` (`4`)
- `s`, `matrix`, `nums1`, `n`, `intervals`, `flowerbed`, `root`, `grid` (`2` each)

## Architectural Ground Truth

This bucket aligns directly with documented language rules:

- `internal_docs/architecture.md:149`
  - parameter reassignment/mutation is explicit only (`mut` / `own mut`)
- `internal_docs/architecture.md:308-324`
  - borrow-by-default parameter model; ownership and mutability are explicit axes
- `internal_docs/architecture.md:338`
  - borrowed move-type params cannot escape by return/store unless explicitly owned or cloned

## Root Cause

The dominant root cause is not compiler unsoundness. It is **surface mismatch between Python-style LeetCode code and explicit Sifr ownership/mutability contracts**.

### root_cause_a_explicit_mutability_not_declared (`31 + 11 = 42`)

The source mutates or rebinds parameters without `mut`.

Typical patterns:

- in-place list/matrix mutation on input params
- `sort()`/write-through operations on input collections
- scalar traversal using parameter rebinding (`n`, `k`, etc.)

Why it happens:

- LeetCode Python baselines treat parameter rebinding/mutation as implicit
- Sifr intentionally requires explicit mutability at the boundary

Decision:

- **adapt fixtures** (or source) to explicit `mut`/`own mut` or local-copy style
- **do not** loosen language semantics

### root_cause_b_borrowed_escape_requires_ownership (`4 + 2 = 6`)

The source tries to store or return borrowed move-type parameters.

Typical patterns:

- storing node parameters in state/containers
- returning borrowed parameter values directly

Why it happens:

- Python references alias freely; Sifr enforces borrow/ownership boundaries

Decision:

- **adapt fixtures** to `own`/`own mut` or explicit `.clone()` based on caller contract:
  - use `own`/`own mut` when caller relinquishes the value (expected default for most LeetCode fixtures)
  - use `.clone()` when caller must retain independent access after the call
- **do not** add implicit cloning or hidden ownership transfer

## Language-Level Judgment

For this bucket, the right policy is:

1. Keep explicit ownership/mutability as-is.
2. Treat almost all current failures as adaptation-required by design.
3. Improve diagnostics and migration ergonomics, not semantics.

This bucket should **not** drive language weakening.

## Compiler Work That Is Still Justified

The following are quality improvements, not semantic relaxations:

1. Better primary diagnostics
   - when a parameter needs `own mut` (not just `mut`) due to both mutation and escape, suggest the precise convention
2. Better fixer-oriented guidance
   - suggest local-copy rewrite for copy-type scalar rebinding when cleaner than mutating parameter contracts
3. Reduced cascades
   - once a boundary mutability/ownership error is emitted, suppress secondary noise from the same root cause

## Execution-Ready Remediation Strategy

1. Batch-adapt this bucket by subcategory:
   - mutation/reassignment first (`42`)
   - then escape-by-store/return (`6`)
2. For each fixture:
   - choose boundary annotation by contract:
     - use `mut` for in-place mutation where no ownership escape is required
     - use `own mut` when both mutation and ownership escape/return are required
     - use `.clone()` only when preserving caller-side availability is required
   - apply a concrete mutability rule:
     - for copy-type scalar rebinding (`n`, `k`, `left`, `columnNumber`, `speed`, `s`), prefer `let mut local = param`
     - for collection/object in-place edits (`nums`, `intervals`, `tokens`, `matrix`, tree roots), prefer explicit parameter `mut` (or `own mut` when escaping)
3. Rerun full corpus and reclassify any residuals as true secondary defects.
   - expectation: at least `6` fixtures with `node`/`root`-style surfaces may unmask secondary categories (for example, recursive node/field-expression surfaces) after ownership/mutability fixes.
4. Escape (`6`) and mutation/reassignment (`42`) workstreams are independent and can be adapted in parallel.

## Bottom Line

`48/48` in this bucket are consistent with Sifr core principles.
Primary action is fixture/source adaptation plus diagnostic-quality improvements.
No language-semantics broadening is warranted.
