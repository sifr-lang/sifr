# Ad Hoc Recursive Type Feature: Part 4 Execution

Status: complete
Started: 2026-03-13
Completed: 2026-03-13
Part: `recursive_hir_surface_and_attribute_access`
PR: `#1125`

## Goal

Make resolved recursive-node values behave like normal typed class values in HIR so guarded attribute access and local uses stop failing at type-check time.

This slice is specifically about the recursive HIR surface. It needs to preserve the recursive types from earlier parts through control-flow narrowing and expression lowering, without falling back to `Any` or treating recursive nodes as a one-off special case.

## Root Cause

After part 3, recursive class annotations and recursive aliases could survive type resolution, but a LeetCode-style guard such as:

```sifr
if not p or not q:
    return False
```

still left the post-guard path unnarrowed for `p` and `q`.

The immediate reason was that HIR lowering recognized `and`-based narrowing conditions but did not build `Or(...)` conditions from boolean `or` expressions. That meant the false branch of `if not p or not q:` never applied the inner negations, so later attribute reads like `p.val` still saw `TreeNode | None` and failed with the generic attribute-expression rejection.

## Implementation

- Extend boolean-condition narrowing detection so `a or b` produces `NarrowingCondition::Or(...)` in the same structured way `a and b` already produced `And(...)`.
- Reuse the existing false-branch narrowing logic so `if not p or not q: return ...` narrows the following path to truthy `p` and `q`.
- Add a HIR regression test for a self-contained recursive `TreeNode` function that reads recursive attributes after an `if not p or not q:` early-return guard.
- Add a type-system regression test proving `Or(...)` false-branch narrowing applies each inner negation deterministically.
- Add a negative e2e check fixture showing that recursive-node attribute access still fails when `None` has not been narrowed away.
- Add a part 4 demo that exercises recursive-node attribute reads and local bindings under guard-based narrowing.

## Validation

Targeted validation:

- `cargo test -p sifr_type_system narrow -- --nocapture`
- `cargo test -p sifr_hir test_recursive_tree_attributes_narrow_after_truthiness_or_guard -- --nocapture`
- `cargo run -q -p sifr -- check demos/ad_hoc_recursive_type_part4_demo.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/recursive_tree_attribute_without_narrowing.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0100_same_tree.sifr`
  - still fails on `2026-03-13` with:
    - `unknown type: 'TreeNode'`
    - `attribute access '.val' is not supported as an expression; use as a method call`

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Coverage Added

- `crates/sifr_hir/src/lower/expressions.rs`
- `crates/sifr_type_system/src/narrow.rs`
- `crates/sifr/tests/e2e/fail/recursive_tree_attribute_without_narrowing.sifr`
- `demos/ad_hoc_recursive_type_part4_demo.sifr`

## Closure Decision

Part 4 is complete for the intended compiler slice because resolved recursive-node values now narrow correctly across `or`-guard early returns in HIR, and guarded recursive attribute access/type-checking behaves like normal class access instead of failing with the generic attribute-expression rejection.

The raw LeetCode audit files that mention bare `TreeNode` without defining or importing it are not blocked by recursive HIR anymore; they are blocked by the absence of any ambient/external class declaration for that source surface. That remaining corpus-facing gap is explicitly deferred to the later closure work instead of being papered over with a `TreeNode` special case in this phase.
