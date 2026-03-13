# Ad Hoc Phase: Full Recursive Type Feature

Status: proposed on 2026-03-13

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
- recursive class fields lower through explicit indirection internally
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

## Implementation Plan

### 1. Symbol Predeclaration and Resolution Graph

- predeclare class and alias symbols before resolving their bodies
- add dependency graph resolution for recursive and mutually recursive declarations
- normalize declaration-order behavior so supported recursive forms are deterministic

### 2. Recursive Well-Formedness Rules

- define what counts as a valid recursive boundary
- reject naked infinite recursion with explicit diagnostics
- ensure recursive generic aliases are validated structurally rather than by ad hoc pattern lists

### 3. Type System Representation

- extend internal type representation to preserve recursive references safely
- ensure unions, optionals, aliases, and generics compose with recursive references
- keep narrowing behavior consistent for `RecursiveType | None`

### 4. HIR and Attribute Surface

- make recursive type annotations survive parser -> AST -> HIR consistently
- allow attribute access and local uses on resolved recursive-node values
- remove any remaining expression-shape rejections that only exist because recursive values are not fully typed

### 5. Codegen

- lower recursive classes to finite Rust representations
- lower recursive aliases only when well-formed
- keep generated Rust readable and deterministic

### 6. Tests and Demos

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

## Validation Gate

Before closing this phase:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Relationship to Phase 31

This phase is broader than the current Phase 31 carry-forward milestone `m31_e_recursive_tree_surface`.

- `m31_e` only needs enough recursive forward-reference and attribute support to make the current tree-domain LeetCode corpus pass.
- this ad hoc phase defines the full general feature that would make recursive types a normal Sifr capability beyond LeetCode.

Recommended planning relationship:

- keep `m31_e` narrow for current corpus closure
- use this ad hoc phase only if the project wants the complete recursive-type feature rather than the minimum tree-surface repair
