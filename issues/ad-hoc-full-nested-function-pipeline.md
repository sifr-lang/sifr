# Ad Hoc Phase: Full Nested Function Pipeline

Status: proposed on 2026-03-14

## Purpose

Make nested local functions a first-class, production-grade Sifr feature rather than a narrow LeetCode compatibility patch.

This phase covers the full compiler pipeline for:

- nested helper functions,
- recursive local helpers,
- captured local state,
- `nonlocal`-style mutation patterns,
- usage-driven parameter and return inference for supported local-helper patterns,
- and deterministic lowering/codegen for nested callables without degrading to `Any`.

## Quality Contract

### Entry criteria

- The current callable/type-checking baseline is green before this phase starts.
- Phase 27 non-regression invariants remain green at phase start:
  - no user-triggerable panic paths,
  - no emitted data-dependent `.unwrap()` / `.expect()` / `panic!` in user runtime paths,
  - stable diagnostic contract,
  - deterministic recovery ordering,
  - stable exit-code and CLI behavior.
- Phase 29 verification baseline remains green at phase start:
  - local-first validation remains authoritative,
  - regression corpus discipline remains enforced,
  - deterministic artifacts remain reviewable and reproducible.

### Exit criteria

- Nested local functions are production-grade, deterministic, regression-locked, and implemented through one coherent compiler architecture rather than ad hoc special cases.
- Supported nested-helper patterns no longer degrade to `Any`.
- Unsupported nested-function shapes fail with explicit, deterministic diagnostics.

### Common quality controls

- No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
- No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires structural rework.
- All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards.
- Scope must remain centered on nested-function architecture, not incidental LeetCode rewrites.
- Validation evidence must be recorded in the execution issue before merge.
- Every milestone must include at least one positive-path and one negative-path validation case.
- No milestone is complete if supported nested-helper behavior still degrades to `Any`.
- No milestone is complete if outputs are not reviewable and reproducible locally.
- Local validation gates pass before merge.
- Full local suite passes:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Milestone demo runs successfully before opening each part PR.
- PR is opened, externally reviewed, and merged before starting the next part.
- Roadmap/phase/issues docs are updated with latest status and merged PR links as each part closes.

### Phase-wide invariants

- No user-triggerable panic paths are introduced.
- No data-dependent emitted `.unwrap()` / `.expect()` / `panic!` is introduced in user runtime paths.
- Supported nested helper signatures, captures, and returns never silently fall back to `Any`.
- Diagnostics for unsupported nested shapes remain explicit and stable.
- Nested-function lowering remains deterministic across repeated runs.
- Generated Rust for supported nested-helper programs compiles cleanly and preserves source semantics.

## Why This Needs Its Own Phase

The full LeetCode corpus review in `issues/full-leetcode-corpus-strategy-review.md` found:

- `114` still-failing fixtures containing nested local functions,
- `95` still-failing fixtures after excluding recursive-type and duplicate-top-level-definition families,
- repeated failure shapes across lowering, inference, capture typing, and downstream `Any` leakage.

This is broader than a Phase 31 closure milestone. It is a language/compiler feature surface.

## Non-goals

- Broad redesign of top-level function inference.
- Dynamic closure objects beyond what current Sifr callable architecture supports.
- Relaxing ownership, parse-safety, or type-safety rules.
- Fixture rewrites as a substitute for compiler support when the source shape should be supported.
- General-purpose higher-order function redesign beyond what is required for nested local functions.

## Problem Statement

Today, nested local functions are only partially supported. The current pipeline still breaks on important supported-looking shapes:

- missing nested parameter annotations when types should be inferable from call sites,
- recursive helper return typing,
- captured local-state typing,
- `nonlocal`-style state updates,
- nested helper bodies degrading to `Any`,
- downstream failures such as:
  - `'<` not supported between instances of 'Any' and 'int'`
  - `bad operand type for unary not: 'Any'`
  - `cannot index type 'Any' with 'int'`
  - `function expects return type 'Any', but returns nothing`

Without a coherent feature phase, the compiler risks accumulating narrow fixes for a few LeetCode shapes while leaving the language surface incomplete.

## Current State

What partially works today:

- some nested local helpers parse and lower far enough to run when fully annotated and minimally captured,
- top-level callable machinery already exists and can be extended rather than replaced,
- the compiler already has evidence that recursive and backtracking helper patterns are intended source forms.

What is still incomplete:

- nested defs are not treated as fully typed callables early enough,
- supported local-helper patterns still require annotations that should be inferable,
- capture typing is not preserved strongly enough through recursion and mutation,
- unsupported or partially supported shapes degrade to `Any` instead of failing explicitly,
- downstream closure/codegen behavior is not yet owned by one documented architecture.

## Product Decision

Sifr should support nested local functions as a first-class language feature for statically analyzable patterns such as:

- recursive DFS/BFS helpers,
- backtracking helpers,
- local memoization/search helpers,
- local helpers capturing immutable state,
- local helpers updating captured state through explicit supported rules.

The feature must be explicit, statically checked, and lowered through one canonical nested-function model. Unsupported shapes must fail with deterministic diagnostics rather than degrading to `Any`.

## Scope

In scope:

1. Nested local function lowering as a stable source-language feature.
2. Usage-driven inference for nested helper parameters and returns in supported corpus patterns.
3. Recursive local helper typing.
4. Captured local-state typing and mutation tracking.
5. `nonlocal`-style update semantics for supported shapes.
6. Deterministic closure/capture lowering without `Any` fallback.
7. Stable diagnostics for unsupported nested-function shapes.
8. Regression coverage for recursive helpers, backtracking helpers, DFS/BFS helpers, and captured mutable state.

Out of scope:

- lambda/general callable redesign outside nested local functions,
- arbitrary escaping closures as a separate feature line,
- dynamic environment objects or runtime reflection over closures,
- fixture-only rewrites when the language/compiler should support the source pattern directly.

## Root-Cause Fix

The root cause is that nested functions currently fall through multiple partial systems instead of one coherent pipeline:

1. nested defs are not fully modeled as typed callables early enough,
2. call-site and capture information is not propagated strongly enough into nested signatures,
3. captured locals do not retain a stable typed representation through recursive or mutating helper bodies,
4. unsupported or partially supported nested shapes degrade to `Any`, which then causes unrelated-looking downstream errors.

The feature needs one coherent architecture:

- predeclare nested helper symbols inside the enclosing scope,
- infer supported nested signatures from call sites and captured-state usage,
- carry capture typing through recursive helper analysis,
- lower nested helpers through a deterministic callable representation,
- model supported captured mutation and `nonlocal`-style updates explicitly,
- reject unsupported nested shapes explicitly instead of silently degrading to `Any`.

## Representative Corpus Cases

- seed-corpus cases already tracked in `m31_d`:
  - `0017`, `0039`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912`
- broader full-corpus cases:
  - `0010`, `0079`, `0091`, `0208`, `0211`, `0212`, `0269`, `0309`, `0410`, `0540`, `0673`, `0745`, `0981`, `1049`, `1397`, `2101`, `2616`

## Milestones

### milestone_nested_1: Nested Symbol Predeclaration and Typed Callable Representation

- Scope:
  - predeclare nested helper symbols in the enclosing scope,
  - represent nested defs as typed callables in HIR/type checking,
  - remove early name-resolution holes that currently produce undefined helper/function fallout.
- Definition of done:
  - nested helper symbols resolve deterministically,
  - supported local calls no longer fail due to missing helper symbol registration,
  - unsupported shapes fail explicitly rather than degrading to unresolved names or `Any`.

### milestone_nested_2: Usage-Driven Inference and Recursive Local Helper Typing

- Scope:
  - infer nested helper parameters/returns from supported call-site and body patterns,
  - support recursive local helper typing,
  - remove the current annotation burden where inference should be possible.
- Definition of done:
  - supported recursive helpers and backtracking helpers infer deterministically,
  - recursive local calls no longer require unnecessary manual annotations,
  - nested helper return typing no longer degrades to `Any`.

### milestone_nested_3: Capture Typing and `nonlocal`-Style State Updates

- Scope:
  - preserve typed captures through nested helper bodies,
  - support explicit captured-state update patterns,
  - model supported `nonlocal`-style mutation without lossy fallback.
- Definition of done:
  - captured locals retain stable types inside nested helpers,
  - supported captured mutation checks and lowers correctly,
  - unsupported captured-mutation shapes fail with deterministic diagnostics.

### milestone_nested_4: Codegen, Diagnostics, and Unsupported-Shape Boundaries

- Scope:
  - lower supported nested helpers through deterministic codegen,
  - eliminate remaining supported-path `Any` fallback,
  - stabilize diagnostics for unsupported nested-function shapes.
- Definition of done:
  - supported nested-helper programs check, emit, and run cleanly,
  - unsupported shapes fail with explicit diagnostics instead of downstream `Any` errors,
  - generated Rust stays panic-free for supported nested-helper paths.

### milestone_nested_5: Regression Corpus, Demos, and Full-Corpus Closure Evidence

- Scope:
  - add pass/fail regression coverage for nested-function families,
  - add milestone demos,
  - rerun the watched LeetCode nested-helper set and record closure evidence.
- Definition of done:
  - nested-function regressions are permanently locked,
  - milestone demos execute successfully,
  - full-corpus or targeted closure artifacts show the nested-helper family moving past current blockers.

## Validation Planning Goals

- `milestone_nested_1` goals cover:
  - deterministic nested symbol predeclaration,
  - typed callable registration for nested defs,
  - explicit diagnostics for unsupported unresolved nested shapes.
- `milestone_nested_2` goals cover:
  - usage-driven inference for supported local-helper signatures,
  - recursive helper typing,
  - elimination of annotation-only blockers in supported patterns.
- `milestone_nested_3` goals cover:
  - captured immutable and mutable state typing,
  - supported `nonlocal`-style updates,
  - explicit rejection of unsupported capture/update forms.
- `milestone_nested_4` goals cover:
  - deterministic lowering/codegen for supported nested helpers,
  - stable diagnostics for unsupported shapes,
  - no supported-path degradation to `Any`.
- `milestone_nested_5` goals cover:
  - regression corpus permanence,
  - milestone demo reproducibility,
  - corpus-facing evidence that the nested-helper family is materially unblocked.
- Exit-gate evidence explicitly demonstrates:
  - nested local functions are a concrete language feature with coherent typing/lowering semantics rather than an ad hoc collection of special cases.

## Local Validation Commands

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Exit Gate

Nested local functions are first-class, deterministic, panic-safe, and production-ready across parser, HIR, type checking, codegen, diagnostics, and regression coverage with no supported-path fallback to `Any`.

Phase 27 and Phase 29 non-regression invariants remain green:

- no user-triggerable panic paths,
- no emitted data-dependent unwrap/expect/panic in user runtime paths,
- stable diagnostics and deterministic validation artifacts.

## Relationship to Phase 31

This phase is broader than Phase 31. Phase 31 or later LeetCode closure work should only own:

- rerunning the affected fixtures after this phase lands,
- fixing any remaining narrow corpus bugs,
- and locking regression coverage for the corpus-facing behavior.
