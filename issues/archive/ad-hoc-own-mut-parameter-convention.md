# Ad Hoc Phase: `own mut` Parameter Convention

Status: complete on 2026-03-14

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

## Quality Contract

- Entry criteria: borrow-by-default parameter conventions, escape analysis, and current `mut` / `own` lowering remain green before this phase starts.
- Exit criteria: `own mut` is a production-grade, deterministic, regression-locked parameter mode with no fallback semantics and no ownership-safety regressions.

### Common quality controls

- No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
- No lazy or partial fixes are allowed; each part must resolve the root cause completely, even when that requires structural rework.
- All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
- Scope must remain constrained to the current part definition-of-done.
- Validation evidence must be recorded in the execution issue before merge.
- Every part must include at least one positive-path and one negative-path validation case.
- No part is complete if its outputs are not reviewable and reproducible locally.
- Local validation gates pass before merge.
- Full local suite passes:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Milestone demo runs successfully before opening each part PR.
- PR is opened, externally reviewed, and merged before starting the next part.
- Roadmap/phase/issues docs are updated with latest status and merged PR links as each part closes.

### Non-regression obligations inherited from the shared quality bar

- No emitted data-dependent `.unwrap()` / `.expect()` / `panic!` is introduced on user-triggerable paths.
- Generated Rust for the new ownership surface compiles cleanly with warnings denied where the existing phase gates require it.
- Behavior remains deterministic across repeated runs for the same source inputs.
- Every fixed bug in scope lands with permanent regression coverage.

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
7. Update `internal_docs/architecture.md` so the canonical ownership model documents `own mut` and the orthogonal ownership/mutability interpretation.

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

## Part Breakdown

- [x] Part 1 `orthogonal_parameter_convention_model_and_frontend_normalization`
  - parser accepts `own mut` and `mut own`
  - AST/HIR/type signatures carry orthogonal ownership + mutability structurally
  - normalization and duplicate-modifier regressions are locked
  - runnable demo proves the new syntax survives frontend lowering without fallback behavior
- [x] Part 2 `borrow_checking_and_escape_semantics_for_four_parameter_modes`
  - borrow/exclusivity checks derive from the orthogonal model
  - owned parameters, including `own mut`, remain returnable
  - borrowed parameters keep deterministic escape diagnostics
- [x] Part 3 `codegen_and_runtime_semantics_for_owned_mutable_parameters`
  - emitted Rust distinguishes all four parameter modes
  - `own mut` lowers to `mut x: T`
  - direct runtime coverage includes a `1299`-style consuming mutable transform
- [x] Part 4 `phase_closure_review_cycles_and_documentation`
  - architecture docs describe the orthogonal parameter model canonically
  - full validation, external review loops, and closure evidence are recorded

## Execution Log

- `2026-03-14`: part 1 `orthogonal_parameter_convention_model_and_frontend_normalization` completed and merged.
  - Execution report: `issues/ad-hoc-own-mut-parameter-convention-part1-execution.md`
  - PR: `#1130`
  - Demo: `demos/ad_hoc_own_mut_parameter_convention_part1_demo.sifr`
  - Added regression coverage:
    - `crates/sifr_python_parser/src/parser/tests.rs`
    - `crates/sifr_hir/src/lower/own_mut_param_tests.rs`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-14`: part 2 `borrow_checking_and_escape_semantics_for_four_parameter_modes` completed local validation and opened PR `#1132`.
  - Execution report: `issues/ad-hoc-own-mut-parameter-convention-part2-execution.md`
  - PR: `#1132`
  - Demo: `demos/ad_hoc_own_mut_parameter_convention_part2_demo.sifr`
  - Added regression coverage:
    - `crates/sifr_hir/src/lower/own_mut_semantics_tests.rs`
    - `crates/sifr/tests/e2e/pass/own_mut_parameter_semantics.sifr`
    - `crates/sifr/tests/e2e/fail/borrowed_mut_parameter_return_escape.sifr`
    - `crates/sifr/tests/e2e/fail/own_parameter_mutation_requires_mut.sifr`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-14`: part 3 `codegen_and_runtime_semantics_for_owned_mutable_parameters` completed local validation and opened PR `#1133`.
  - Execution report: `issues/ad-hoc-own-mut-parameter-convention-part3-execution.md`
  - PR: `#1133`
  - Demo: `demos/ad_hoc_own_mut_parameter_convention_part3_demo.sifr`
  - Added regression coverage:
    - `crates/sifr_codegen/src/lib_codegen_tests.rs`
    - `crates/sifr_hir/src/lower/own_mut_semantics_tests.rs`
    - `crates/sifr/tests/e2e/pass/own_mut_replace_elements_1299.sifr`
    - `crates/sifr/tests/e2e/fail/own_parameter_method_mutation_requires_mut.sifr`
  - Full local validation:
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
- `2026-03-14`: part 4 `phase_closure_review_cycles_and_documentation` first external review pass completed.
  - Execution report: `issues/ad-hoc-own-mut-parameter-convention-part4-execution.md`
  - Review file: `reviews/phase-own-mut-review-pass-1.md`
  - Result: `APPROVED - Ready for production`
  - Action taken: reviewer notes validated; no implementation changes were required
- `2026-03-14`: part 4 `phase_closure_review_cycles_and_documentation` second production-grade review pass completed.
  - Execution report: `issues/ad-hoc-own-mut-parameter-convention-part4-execution.md`
  - Review file: `reviews/phase-own-mut-production-grade-review-pass-2.md`
  - Result: `APPROVED - Production Ready`
  - Action taken: reviewer notes validated; no implementation changes were required

## Closure

This ad hoc phase is complete. All four parts have been merged, authoritative validation passed locally, the first review pass was approved, and the second production-grade review pass was approved.

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
- architecture documentation updated alongside the implementation change, not after milestone closure

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
