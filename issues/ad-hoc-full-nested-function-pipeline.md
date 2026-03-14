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

## Why This Needs Its Own Phase

The full LeetCode corpus review in `issues/full-leetcode-corpus-strategy-review.md` found:

- `114` still-failing fixtures containing nested local functions,
- `95` still-failing fixtures after excluding recursive-type and duplicate-top-level-definition families,
- repeated failure shapes across lowering, inference, capture typing, and downstream `Any` leakage.

This is broader than a Phase 31 closure milestone. It is a language/compiler feature surface.

## Problem Statement

Today, nested local functions are only partially supported. The current pipeline still breaks on important supported-looking shapes:

- missing nested parameter annotations when types should be inferable from call sites,
- recursive helper return typing,
- captured local-state typing,
- nested helper bodies degrading to `Any`,
- downstream failures such as:
  - `'<` not supported between instances of 'Any' and 'int'`
  - `bad operand type for unary not: 'Any'`
  - `cannot index type 'Any' with 'int'`
  - `function expects return type 'Any', but returns nothing`

Without a coherent feature phase, the compiler risks accumulating ad hoc fixes for a few LeetCode shapes while leaving the language surface incomplete.

## Scope

In scope:

1. Nested local function lowering as a stable source-language feature.
2. Usage-driven inference for nested helper parameters and returns in supported corpus patterns.
3. Recursive local helper typing.
4. Captured local-state typing and mutation tracking.
5. `nonlocal`-style update semantics for supported shapes.
6. Deterministic closure/capture lowering without `Any` fallback.
7. Regression coverage for recursive helpers, backtracking helpers, DFS/BFS helpers, and captured mutable state.

Out of scope:

- broad redesign of top-level function inference,
- dynamic closure objects beyond what current Sifr callable architecture supports,
- relaxing ownership or type-safety rules,
- fixture rewrites as a substitute for compiler support when the source shape should be supported.

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
- reject unsupported nested shapes explicitly instead of silently degrading to `Any`.

## Representative Corpus Cases

- seed-corpus cases already tracked in `m31_d`:
  - `0017`, `0039`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912`
- broader full-corpus cases:
  - `0010`, `0079`, `0091`, `0208`, `0211`, `0212`, `0269`, `0309`, `0410`, `0540`, `0673`, `0745`, `0981`, `1049`, `1397`, `2101`, `2616`

## Acceptance Criteria

- nested helper parameters are inferred for supported call-site-driven patterns without degrading to `Any`
- recursive local helpers type-check deterministically
- captured mutable and immutable locals retain stable types inside nested helpers
- supported `nonlocal`-style updates check and lower correctly
- unsupported nested shapes fail with explicit diagnostics rather than `Any` fallback
- the LeetCode nested-helper family moves past the current inference/capture blockers

## Relationship to Phase 31

This phase is broader than Phase 31. Phase 31 or later LeetCode closure work should only own:

- rerunning the affected fixtures after this phase lands,
- fixing any remaining narrow corpus bugs,
- and locking regression coverage for the corpus-facing behavior.
