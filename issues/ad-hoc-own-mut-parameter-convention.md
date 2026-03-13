# Ad Hoc Phase: `own mut` Parameter Convention

Status: proposed on 2026-03-13

## Purpose

Add first-class support for parameters that are both:

- ownership-transferring, and
- locally mutable

This closes the gap between Sifr's current three parameter modes:

- default borrow: `x: T`
- mutable borrow: `mut x: T`
- owned immutable: `own x: T`

and the fourth semantically valid mode that Sifr currently cannot express:

- owned mutable: `own mut x: T`

This phase is motivated by in-place transform APIs such as LeetCode `1299`, where the function must mutate a list and then return that same owned list.

## Problem Statement

Today, Sifr's ownership model treats mutability and ownership as separate concepts, but the parameter surface only exposes three combinations. That leaves one real hole:

- `mut x: list[int]` can model `&mut Vec<i64>` semantics
- `own x: list[int]` can model `Vec<i64>` semantics
- there is no parameter form that models `mut arr: Vec<i64>` in Rust terms

As a result, the language cannot express the natural contract for consume-and-mutate APIs:

```sifr
def replaceElements(own mut arr: list[int]) -> list[int]:
    ...
    return arr
```

Without this capability:

- `mut` alone solves mutation but not ownership-return
- `own` alone solves ownership-return but not mutation
- `1299` remains a documented divergence even though the underlying ownership model is conceptually close to Rust

## Product Decision

Sifr should support `own mut` as a first-class parameter form.

Canonical source syntax:

```sifr
def f(own mut x: list[int]) -> list[int]:
    ...
```

Parsing policy:

- accept `own mut x: T`
- accept `mut own x: T` for tolerance during rollout
- normalize docs, demos, diagnostics, and snapshots to `own mut`

Language meaning:

- `own` means the callee receives the argument by value
- `mut` means the local binding may be mutated
- `own mut` means the callee owns the value and may mutate it, matching Rust `mut x: T`

## Scope

In scope:

1. Extend the parameter syntax to allow combined `own` + `mut`.
2. Make combined parameters survive parser -> AST -> HIR -> codegen without fallback behavior.
3. Emit correct Rust for all four valid parameter modes:
   - `x: T` -> `x: &T` for move types
   - `mut x: T` -> `x: &mut T`
   - `own x: T` -> `x: T`
   - `own mut x: T` -> `mut x: T`
4. Make `return x` legal when `x` is an owned parameter, including `own mut`.
5. Preserve current escape-analysis errors for borrowed parameters.
6. Add regression coverage for consuming mutable APIs, including a direct `1299`-style case.

Out of scope:

- expression-level `own` syntax such as `return own x`
- implicit ownership escalation from borrowed parameters
- automatic cloning or silent fallback semantics
- changing default borrow-by-default rules
- method receiver redesign beyond what is required to keep parameter lowering consistent

## Root-Cause Fix

The root cause is that the current parameter representation folds ownership and mutability into a single three-state convention. Supporting `own mut` cleanly requires making the model represent both dimensions.

Recommended internal model:

- ownership axis:
  - `Borrow`
  - `Own`
- mutability axis:
  - `Immutable`
  - `Mutable`

Valid parameter combinations:

| Source form | Ownership axis | Mutability axis | Rust shape |
| --- | --- | --- | --- |
| `x: T` | `Borrow` | `Immutable` | `&T` |
| `mut x: T` | `Borrow` | `Mutable` | `&mut T` |
| `own x: T` | `Own` | `Immutable` | `T` |
| `own mut x: T` | `Own` | `Mutable` | `mut x: T` |

Implementation note:

- do not paper over this by adding ad hoc codegen branches only
- either replace `ParamConvention` with an orthogonal representation, or evolve it to carry both dimensions explicitly while preserving helper APIs
- all borrow checking, move tracking, and codegen decisions should derive from those two dimensions rather than string or syntax checks

## User-Facing Semantics

### Rust Mapping

This Sifr function:

```sifr
def replaceElements(own mut arr: list[int]) -> list[int]:
    rightMax = -1
    for i in range(len(arr) - 1, -1, -1):
        newMax = max(rightMax, arr[i])
        arr[i] = rightMax
        rightMax = newMax
    return arr
```

should lower to Rust with the same ownership semantics:

```rust
fn replace_elements(mut arr: Vec<i64>) -> Vec<i64> {
    let mut right_max = -1;
    for i in (0..arr.len()).rev() {
        let new_max = right_max.max(arr[i]);
        arr[i] = right_max;
        right_max = new_max;
    }
    arr
}
```

### Return Semantics

- `return x` is valid when `x` is owned by the function
- `return x` remains invalid when `x` is a borrowed parameter of a move type
- diagnostics for borrowed-parameter escape should keep recommending explicit ownership at the signature boundary rather than hidden clones

## Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | `def f(own mut items: list[int]) -> int` parses successfully and preserves both modifiers through AST and HIR |
| AC-2 | emitted Rust for `own mut items: list[int]` uses `mut items: Vec<i64>` |
| AC-3 | local element mutation on an `own mut` parameter compiles and runs correctly |
| AC-4 | returning an `own mut` parameter compiles and runs correctly without clone insertion |
| AC-5 | `mut`-only parameters still lower to `&mut T` and cannot be returned by value |
| AC-6 | `own`-only parameters still lower to `T` and remain immutable unless explicitly marked `mut` |
| AC-7 | borrowed parameters still fail escape analysis with deterministic diagnostics |
| AC-8 | a `1299`-style fixture written with `own mut` checks, emits, and runs successfully |
| AC-9 | full local validation passes with no regressions in existing borrow-by-default coverage |

## Implementation Plan

### 1. Parser and AST

- allow two soft keywords before a parameter name instead of at most one
- parse both `own mut` and `mut own`
- store the normalized ownership/mutability information structurally, not as source-order text

### 2. Type System and HIR

- replace the single convention decision with a representation that can express both axes
- update `FunctionType`, callable signatures, and HIR parameters accordingly
- keep Copy types passing by value as they do today; `mut` on Copy parameters should still only affect local mutability, not borrow emission

### 3. Borrow Checking and Escape Analysis

- treat `own mut` parameters as owned locals for move/return analysis
- keep exclusivity checks for borrowed mutable parameters unchanged
- ensure reassignment and mutation logic treats owned mutable parameters the same way as mutable local bindings

### 4. Codegen

- emit `&T`, `&mut T`, `T`, or `mut x: T` based on the normalized parameter model
- update regular-call, method-call, callable, and intrinsic-call emission to preserve existing semantics for the three supported modes
- ensure `own mut` does not accidentally emit `&mut T` or require clone insertion on return

### 5. Tests and Demos

- parser/unit coverage for both accepted modifier orders
- HIR lowering coverage for the four valid parameter combinations
- codegen snapshot coverage for Rust signature emission
- e2e pass fixture for owned mutable parameter mutation and return
- e2e fail fixtures confirming:
  - borrowed parameter return still fails
  - `own`-only parameter mutation fails if local mutability is required
  - `mut`-only parameter return still fails
- demo file showing `1299`-style consume-mutate-return behavior

## Validation Gate

Before closing this phase:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Relationship to Phase 31

This phase provides the language feature needed to make the current `1299` ownership divergence supportable in idiomatic Sifr source.

It does not, by itself, make the raw LeetCode Python-shaped fixture pass unchanged. The explicit Sifr form is expected to be:

```sifr
def replaceElements(own mut arr: list[int]) -> list[int]:
    ...
    return arr
```

If this phase lands, `phase31` ownership follow-up should be updated so `1299` is no longer treated as a permanent divergence, but as a language-surface gap resolved by explicit `own mut`.
