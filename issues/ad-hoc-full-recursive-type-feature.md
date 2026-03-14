# Ad Hoc Phase: Full Recursive Type Feature

Status: active on 2026-03-13

## Purpose

Make recursive types a first-class Sifr language feature rather than a narrow compatibility patch for `TreeNode`-style LeetCode cases.

This phase covers the full language/compiler contract for:

- self-referential classes,
- mutually recursive classes,
- recursive type aliases,
- generic recursive types,
- recursive types inside unions such as `T | None`,
- and stable Rust lowering for all supported recursive forms.

This is intentionally broader than the current Phase 31 carry-forward work. Phase 31 only needs a scoped tree-surface fix. This ad hoc phase defines what a production-grade, general recursive-type feature would require.

## Quality Contract

- Entry criteria: current class lowering, union lowering, generic type alias support, and borrow-by-default architecture remain green before this phase starts.
- Exit criteria: recursive types are production-grade, deterministic, regression-locked, and lowered through one coherent compiler architecture rather than special-case patches.

## Entry Baseline

Baseline checks that should be re-run and recorded in the execution issue before part 1 starts:

- `target/debug/sifr check crates/sifr/tests/e2e/pass/recursive_treenode.sifr` -> currently passes on `2026-03-13`
- `target/debug/sifr check crates/sifr/tests/e2e/pass/forward_ref_listnode.sifr` -> currently passes on `2026-03-13`
- `target/debug/sifr check audits/leetcode/0100_same_tree.sifr` -> currently fails on `2026-03-13` with:
  - `unknown type: 'TreeNode'`
  - `attribute access '.val' is not supported as an expression; use as a method call`
- `target/debug/sifr check audits/leetcode/0102_binary_tree_level_order_traversal.sifr` -> currently fails on `2026-03-13` with:
  - `unknown type: 'TreeNode'`
  - `attribute access '.left' is not supported as an expression; use as a method call`
  - `attribute access '.right' is not supported as an expression; use as a method call`

Interpretation:

- the compiler already has partial recursive-class and forward-reference capability in isolated fixtures,
- but recursive types are not yet implemented as a complete general feature,
- and the LeetCode tree surface still fails end-to-end because recursive-type resolution and attribute expression support are not unified under one architecture.

### Common quality controls

- No fallback or one-off `TreeNode`/`ListNode` special-casing is allowed as the phase architecture.
- No partial acceptance of recursive forms is allowed without explicit static rejection rules for unsupported forms.
- All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and stable diagnostics.
- Scope must stay centered on recursive type architecture, not incidental LeetCode rewrites.
- Validation evidence must be recorded in the execution issue before merge.
- Every part must include at least one positive-path and one negative-path validation case.
- No part is complete if recursive lowering still depends on ad hoc symbol-order accidents.
- Local validation gates pass before merge.
- Full local suite passes:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Milestone demo runs successfully before opening each part PR.
- PR is opened, externally reviewed, and merged before starting the next part.
- Roadmap/phase/issues docs are updated with latest status and merged PR links as each part closes.

### Non-regression obligations inherited from the shared quality bar

- No emitted data-dependent `.unwrap()` / `.expect()` / `panic!` is introduced on user-triggerable paths.
- Generated Rust for recursive values compiles cleanly with warnings denied where the existing phase gates require it.
- Behavior remains deterministic across repeated runs for the same source inputs.
- Every fixed recursive-type bug lands with permanent regression coverage.

## Problem Statement

Sifr already has partial signs of recursive-type intent:

- recursive class fixtures such as `crates/sifr/tests/e2e/pass/recursive_treenode.sifr`,
- forward-reference fixtures such as `crates/sifr/tests/e2e/pass/forward_ref_listnode.sifr`,
- generic type alias support from earlier type-system work,
- and a runtime type contract in `internal_docs/architecture.md` that already treats unions and optionals as first-class.

But the current language surface is not yet a full recursive-type feature. The remaining gaps are structural:

- forward references are not resolved as a general language rule,
- recursive class fields are not defined by one canonical lowering contract,
- recursive aliases do not have a complete well-formedness model,
- mutually recursive declarations are not modeled as a dependency graph,
- and LeetCode tree cases still fail because recursive-node field access and type resolution are not complete end-to-end.

Without a real recursive-type feature, the compiler risks accumulating narrow fixes for `TreeNode` while leaving the underlying language contract incomplete.

## Current State

What already works today:

- simple self-referential class fixtures can pass type checking
- at least one forward-reference `ListNode` fixture can pass type checking
- existing runtime/type contracts already support unions, `Option`, classes, and generic type aliases in isolation

What is not yet a complete feature:

- there is no documented general recursive-type well-formedness rule
- there is no explicit recursive declaration dependency-graph resolution contract
- there is no single documented lowering rule for when recursive fields become boxed/indirected in Rust
- recursive aliases are not yet defined as a supported general feature
- the LeetCode tree cases still fail because recursive-node resolution and attribute-expression lowering are not closed end-to-end

Practical consequence:

- the current compiler behavior is best described as partial recursive support, not a production-grade recursive-type feature
- this phase should promote that partial support into a fully specified and regression-locked language capability

## Product Decision

Sifr should support general recursive types as a first-class language feature.

Supported source forms should include:

```sifr
class TreeNode:
    val: int
    left: TreeNode | None
    right: TreeNode | None
```

```sifr
class A:
    b: B | None

class B:
    a: A | None
```

```sifr
type Json = None | bool | int | float | str | list[Json] | dict[str, Json]
```

```sifr
class Node[T]:
    value: T
    next: Node[T] | None
```

The feature must be explicit, statically checked, and lowered through a single recursive-type model. Unsupported recursive forms must fail with deterministic diagnostics rather than degrading to `Any` or relying on source-order accidents.

## Scope

In scope:

1. Predeclare type/class symbols so self-references and mutual recursion resolve deterministically.
2. Support recursive class fields through canonical compiler and Rust lowering rules.
3. Support mutually recursive classes.
4. Support recursive type aliases, including aliases that recurse through containers or unions.
5. Support generic recursive types.
6. Define and enforce well-formedness rules for legal vs illegal recursion.
7. Support attribute reads, constructor calls, method signatures, and local annotations involving recursive types.
8. Support recursive types in unions and optionals with normal narrowing behavior.
9. Document one canonical Rust representation strategy for all supported recursive forms.

Out of scope:

- cyclic runtime object graph analysis beyond normal ownership/borrowing rules,
- automatic shared mutability wrappers,
- protocol/object-safe recursive dynamic dispatch redesign,
- recursive types through unsupported future features not yet in Sifr,
- relaxing ownership, parse-safety, or type-safety rules to make recursive code compile.

## Root-Cause Fix

The root cause is that recursive types cannot be handled correctly by a single-pass annotation resolver or by type lowering that assumes every referenced type is already fully known.

The feature needs a real recursive-type pipeline:

1. **Predeclaration**
   - Register class and alias names before resolving their bodies.
   - This makes self-reference and mutual recursion legal at the symbol level.

2. **Dependency graph resolution**
   - Resolve recursive declarations as strongly connected components rather than one declaration at a time.
   - This is required for `A -> B -> A`, alias recursion, and generic recursive families.

3. **Well-formedness validation**
   - Reject infinite-size recursive forms that have no valid indirection boundary.
   - Accept recursive forms that recurse through a valid heap/container/boxed boundary.

4. **Canonical type representation**
   - Recursive types must map to one stable internal representation in the type system.
   - Recursive unions, aliases, and classes must compose with existing `Option`, enum-union, and generic machinery.

5. **Canonical Rust lowering**
   - Generated Rust must remain finite and valid.
   - Recursive class fields should lower through an explicit indirection boundary internally.
   - Alias recursion must lower only when the resolved form is well-founded.

Implementation note:

- do not solve this by hardcoding `TreeNode`, `ListNode`, or LeetCode-specific special cases
- do not solve this by silently rewriting recursive aliases into `Any`
- the architecture must treat recursive types as normal user-defined types with explicit static rules

## User-Facing Semantics

### Supported recursive classes

```sifr
class TreeNode:
    val: int
    left: TreeNode | None
    right: TreeNode | None
```

This should be legal in:

- field annotations,
- constructor signatures,
- local annotations,
- return types,
- and attribute reads:

```sifr
def height(node: TreeNode | None) -> int:
    if node is None:
        return 0
    return 1 + max(height(node.left), height(node.right))
```

### Supported mutually recursive classes

```sifr
class Expr:
    term: Term

class Term:
    expr: Expr | None
```

### Supported recursive aliases

```sifr
type Json = None | bool | int | float | str | list[Json] | dict[str, Json]
```

### Rejected ill-formed recursion

These should fail with deterministic diagnostics:

```sifr
type Bad = Bad
```

```sifr
type AlsoBad[T] = tuple[AlsoBad[T], T]
```

unless the language explicitly defines that recursive tuples/classes are internally boxed by the type form itself. The default expectation for this phase should be:

- recursion is legal only when the recursive path crosses a valid indirection boundary
- container recursion such as `list[Json]` is valid
- naked alias recursion is invalid

## Rust Mapping Contract

This phase must define one coherent lowering strategy that composes with the existing runtime type contract in [architecture.md](/Users/yaseralnajjar/work/sifr/codebase/internal_docs/architecture.md).

Recommended contract:

- `T | None` continues to lower to `Option<T>`
- recursive class fields whose resolved field type is the containing class or another class in the same recursive SCC lower through explicit indirection internally
- direct recursive class field:
  - `next: Node | None` -> `Option<Box<Node>>`
  - `child: Node` -> `Box<Node>`
- container recursion does not add extra boxing beyond the container's own heap boundary:
  - `list[Json]`
  - `dict[str, Json]`
  - `set[NodeId]`
- recursive aliases lower only when the resolved recursive path is well-founded
- generic recursive types remain monomorphized like the rest of Sifr generics

Illustrative target shape:

```rust
struct TreeNode {
    val: i64,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}
```

This is a codegen consequence, not required user syntax.

## Diagnostic Contract

This phase should standardize diagnostics for ill-formed or unsupported recursive forms. Exact error codes can be assigned during implementation, but the message family should be fixed by the phase:

- naked alias recursion:
  - `type error: ill-formed recursive type alias 'Bad': recursion must cross an indirection boundary`
- recursive generic alias without a valid boundary:
  - `type error: ill-formed recursive generic alias 'AlsoBad[T]': recursion must cross an indirection boundary`
- unresolved recursive-name use outside supported declaration resolution:
  - `type error: unknown type: 'TreeNode'`
- recursive value field access before the recursive surface is supported:
  - `type error: attribute access '.left' is not supported as an expression; use as a method call`

Part completion is not allowed to introduce vague fallback diagnostics such as generic `Any`-driven errors when a specific recursive-type validation error is available.

## Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | self-referential class field annotations resolve without manual declaration reordering |
| AC-2 | mutually recursive classes resolve deterministically regardless of declaration order within the same SCC |
| AC-3 | recursive aliases that recurse through valid container/union boundaries type-check successfully |
| AC-4 | ill-formed recursive aliases fail with deterministic, specific diagnostics |
| AC-5 | generic recursive types preserve type arguments correctly through resolution and lowering |
| AC-6 | attribute reads and constructor signatures on recursive class values work end-to-end |
| AC-7 | `TreeNode`-style LeetCode cases move past unknown-type and attribute-expression failures without special casing |
| AC-8 | emitted Rust uses finite well-founded recursive representations |
| AC-9 | full local validation passes with no regressions in existing class/union/generic behavior |

## Execution Log

- `2026-03-13`: part 1 `recursive_symbol_predeclaration_and_alias_order_resolution` completed local validation.
  - Execution report: `issues/ad-hoc-full-recursive-type-feature-part1-execution.md`
  - PR: `#1122`
  - Demo: `demos/ad_hoc_recursive_type_part1_demo.sifr`
  - Added regression coverage:
    - `crates/sifr/tests/e2e/pass/recursive_type_alias_symbol_predeclaration.sifr`
    - `crates/sifr/tests/e2e/fail/type_alias_missing_dependency.sifr`
    - `crates/sifr_hir/src/lower/type_alias_tests.rs`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-13`: part 2 `recursive_well_formedness_validation` completed local validation.
  - Execution report: `issues/ad-hoc-full-recursive-type-feature-part2-execution.md`
  - PR: `#1123`
  - Demo: `demos/ad_hoc_recursive_type_part2_demo.sifr`
  - Added regression coverage:
    - `crates/sifr/tests/e2e/pass/recursive_type_alias_well_formed.sifr`
    - `crates/sifr/tests/e2e/fail/recursive_type_alias_missing_boundary.sifr`
    - `crates/sifr_hir/src/lower/type_alias_tests.rs`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-13`: part 3 `recursive_type_representation_in_the_type_system` completed local validation.
  - Execution report: `issues/ad-hoc-full-recursive-type-feature-part3-execution.md`
  - PR: `#1124`
  - Demo: `demos/ad_hoc_recursive_type_part3_demo.sifr`
  - Added regression coverage:
    - `crates/sifr/tests/e2e/pass/recursive_generic_type_alias_representation.sifr`
    - `crates/sifr/tests/e2e/fail/recursive_generic_type_alias_wrong_arity.sifr`
    - `crates/sifr_hir/src/lower/type_alias_tests.rs`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-13`: part 4 `recursive_hir_surface_and_attribute_access` completed local validation.
  - Execution report: `issues/ad-hoc-full-recursive-type-feature-part4-execution.md`
  - PR: `#1125`
  - Demo: `demos/ad_hoc_recursive_type_part4_demo.sifr`
  - Added regression coverage:
    - `crates/sifr_hir/src/lower/expressions.rs`
    - `crates/sifr_type_system/src/narrow.rs`
    - `crates/sifr/tests/e2e/fail/recursive_tree_attribute_without_narrowing.sifr`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-13`: part 5 `recursive_rust_lowering_and_codegen` completed local validation.
  - Execution report: `issues/ad-hoc-full-recursive-type-feature-part5-execution.md`
  - PR: `#1126`
  - Demo: `demos/ad_hoc_recursive_type_part5_demo.sifr`
  - Added regression coverage:
    - `crates/sifr_codegen/src/lib_codegen_tests.rs`
    - `crates/sifr/tests/e2e/pass/recursive_tree_traversal_runtime.sifr`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-13`: part 6 `recursive_corpus_closure_tests_and_demo` completed local validation.
  - Execution report: `issues/ad-hoc-full-recursive-type-feature-part6-execution.md`
  - PR: `#1127`
  - Demo: `demos/ad_hoc_recursive_type_part6_demo.sifr`
  - Added regression coverage:
    - `crates/sifr_codegen/src/lib_codegen_tests.rs`
    - `crates/sifr_hir/src/lower/type_alias_tests.rs`
    - `crates/sifr/tests/e2e/pass/recursive_mutual_classes_runtime.sifr`
    - `crates/sifr/tests/e2e/pass/recursive_generic_node_runtime.sifr`
    - `crates/sifr/tests/e2e/fail/recursive_mutual_type_alias_missing_boundary.sifr`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
  - External review follow-up:
    - review pass 1 found no actionable issues
    - production-grade review pass 2 was invalidated against stale local `main`; merged `origin/main` already contains the claimed fixes and fixtures
    - production-grade re-review pass 4 on merged `origin/main` confirmed the phase is ready for production with no further in-scope changes required

## Implementation Parts and PR Breakdown

This phase should not be executed as one large PR. The recommended breakdown is:

### Part 1. Recursive symbol predeclaration and SCC resolution

- PR scope:
  - predeclare class and alias symbols before resolving their bodies
  - add dependency-graph resolution for recursive and mutually recursive declarations
  - normalize declaration-order behavior so supported recursive forms are deterministic
- Primary compiler areas:
  - parser/AST-to-HIR type resolution path
  - symbol registration and annotation resolution
- Part definition of done:
  - self-recursive and mutually recursive declarations resolve deterministically at the symbol/type-name level
  - declaration order inside one SCC no longer changes supported outcomes
- Required validation:
  - positive tests for self recursion and mutual recursion
  - negative tests for unresolved names outside the supported resolution model

### Part 2. Recursive well-formedness validation

- Depends on: part 1
- PR scope:
  - define the exact valid indirection boundaries
  - reject naked infinite recursion and structurally invalid recursive aliases
  - stabilize recursive-type diagnostics
- Minimum rule set for this phase:
  - legal recursion must cross a valid indirection boundary
  - valid boundaries are:
    - recursive class fields lowered through internal boxing
    - heap-owning containers such as `list[...]`, `dict[..., ...]`, and `set[...]`
  - naked alias recursion like `type Bad = Bad` is illegal
  - tuple-only recursive aliases are illegal in this phase unless a future phase explicitly adds tuple-level indirection semantics
- Part definition of done:
  - supported recursive forms are accepted for principled reasons
  - unsupported recursive forms fail with deterministic specific diagnostics

### Part 3. Recursive type representation in the type system

- Depends on: parts 1 and 2
- PR scope:
  - extend internal type representation to preserve recursive references safely
  - ensure unions, optionals, aliases, and generics compose with recursive references
  - keep narrowing behavior consistent for `RecursiveType | None`
- Primary compiler areas:
  - type representation and annotation resolution
  - normalization / equality / substitution for recursive generic forms
- Part definition of done:
  - recursive references survive type resolution without degrading to `Any`
  - generic recursive types preserve type arguments end-to-end

### Part 4. Recursive HIR surface and attribute access

- Depends on: parts 1 through 3
- PR scope:
  - make recursive annotations survive parser -> AST -> HIR consistently
  - allow attribute access and local uses on resolved recursive-node values
  - close the LeetCode-facing tree surface that still fails on `unknown type` and attribute-expression rejection
- Primary compiler areas:
  - HIR lowering
  - attribute-expression typing/lowering
- Part definition of done:
  - current LeetCode tree failures move past unknown-type and attribute-expression blockers without special casing
  - recursive-node values behave like normal typed class values in expressions

### Part 5. Recursive Rust lowering and codegen

- Depends on: parts 1 through 4
- PR scope:
  - lower recursive classes to finite Rust representations
  - lower recursive aliases only when well-formed
  - keep generated Rust readable and deterministic
- Explicit lowering rules for this phase:
  - recursive class fields to same-SCC classes are boxed at the field boundary
  - `T | None` around a recursive class field remains `Option<Box<T>>`
  - container recursion uses the container's existing heap representation and does not add extra boxing just because the contained type is recursive
- Part definition of done:
  - generated Rust for supported recursive forms is finite, valid, and stable
  - no special-case `TreeNode`/`ListNode` lowering exists

### Part 6. Corpus closure, tests, and demo

- Depends on: parts 1 through 5
- PR scope:
  - add the final regression matrix
  - add demo coverage
  - verify the handoff to the Phase 31 tree closure milestone
- Required coverage:
  - parser/unit coverage for recursive alias syntax and forward refs
  - type-system coverage for:
    - self recursion
    - mutual recursion
    - recursive generics
    - accepted container-recursive aliases
    - rejected naked recursion
  - codegen coverage for recursive class lowering
  - e2e pass fixtures for:
    - recursive class fields
    - mutually recursive classes
    - recursive alias through containers
    - recursive generic node types
  - e2e fail fixtures for ill-formed recursion
  - demo showing recursive tree traversal and a non-tree recursive alias
- Part definition of done:
  - the full recursive-type feature is regression-locked
  - the prerequisite is ready for `m31_e_recursive_tree_surface_leetcode_closure`

## Validation Gate

Before closing this phase:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Relationship to Phase 31

This phase is broader than the current Phase 31 carry-forward milestones `prereq_recursive_types` and `m31_e_recursive_tree_surface_leetcode_closure`.

- `prereq_recursive_types` should point at this ad hoc phase and use it as the implementation source of truth for the broad feature.
- `m31_e_recursive_tree_surface_leetcode_closure` should stay narrow and corpus-focused:
  - verify that this prerequisite fully unblocks `0100`, `0102`, `0110`, `0226`, and `0235`
  - add only the remaining LeetCode-specific regression/demo closure work
- this ad hoc phase does not supersede the Phase 31 milestone tracker; it supplies the prerequisite feature work that the tracker depends on.

Recommended planning relationship:

- execute this ad hoc phase as the broad recursive-type prerequisite
- keep `m31_e_recursive_tree_surface_leetcode_closure` narrow for current corpus closure after the prerequisite lands
