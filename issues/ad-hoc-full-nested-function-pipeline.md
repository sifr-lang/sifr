# Ad Hoc Phase: Full Nested Function Pipeline

Status: completed on 2026-03-15

## Purpose

Make nested local functions a first-class, production-grade Sifr feature rather than a narrow LeetCode compatibility patch.

This phase covers the full compiler pipeline for:

- nested helper functions,
- recursive local helpers,
- captured local state,
- `nonlocal`-style mutation patterns,
- usage-driven parameter and return inference for supported local-helper patterns,
- and deterministic lowering/codegen for nested callables without degrading to `Any`.

## Execution Checklist

- [x] `milestone_nested_1`: nested symbol predeclaration and typed callable representation
- [x] `milestone_nested_2`: usage-driven inference and recursive local helper typing
- [x] `milestone_nested_3`: capture typing and `nonlocal`-style state updates
- [x] `milestone_nested_4`: codegen, diagnostics, and unsupported-shape boundaries
- [x] `milestone_nested_5`: regression corpus, demos, and full-corpus closure evidence
- [x] external review pass 1 completed and acted on
- [x] production-grade review pass completed and acted on

## Execution Log

- `2026-03-14`: `milestone_nested_1` completed.
  - Execution report: `issues/ad-hoc-full-nested-function-pipeline-part1-execution.md`
  - PR: `#1139`
  - Closure basis: nested helpers are now predeclared as typed local callables during HIR lowering, forward local helper references no longer fail due to statement-order registration, and missing helper names still fail explicitly.
- `2026-03-14`: `milestone_nested_2` completed.
  - Execution report: `issues/ad-hoc-full-nested-function-pipeline-part2-execution.md`
  - PR: `#1141`
  - Closure basis: supported recursive local helpers now infer parameters and returns deterministically, conflicting local-helper inference fails explicitly, and the watched corpus has moved beyond the original nested-annotation / `Any` failure mode.
- `2026-03-14`: `milestone_nested_3` completed.
  - Execution report: `issues/ad-hoc-full-nested-function-pipeline-part3-execution.md`
  - PR: `#1143`
  - Closure basis: supported non-recursive `nonlocal` capture updates now lower and run deterministically, recursive and tuple-unpack capture updates fail explicitly before codegen, and the watched corpus has moved off the generic unsupported-statement failure mode for captured-state updates.
- `2026-03-15`: `milestone_nested_4` completed.
  - Execution report: `issues/ad-hoc-full-nested-function-pipeline-part4-execution.md`
  - PR: `#1145`
  - Closure basis: structured codegen now lowers recursive nested helpers with typed capture parameters and mutable collection captures, supported backtracking helpers no longer fall through to production panics or `Any`-driven Rust mismatches, and immutable-parameter capture mutation now fails explicitly at the language boundary.
- `2026-03-15`: `milestone_nested_5` completed.
  - Execution report: `issues/ad-hoc-full-nested-function-pipeline-part5-execution.md`
  - PR: `#1146`
  - Closure basis: the phase now has authoritative demo coverage plus permanent pass/fail nested-function regressions, and the watched audit set is classified into supported passes, explicit ownership boundaries, and unrelated residual blockers outside this pipeline.
- `2026-03-15`: external review pass 1 completed.
  - Review file: `reviews/phase-nested-functions-review-pass-1.md`
  - Outcome: approved for the documented phase scope with no blocking findings and no required code changes.
- `2026-03-15`: production-grade review pass completed.
  - Review file: `reviews/phase-nested-functions-production-grade-review-pass-2a.md`
  - Outcome: approved as production-ready for the documented phase scope with no blocking findings and no required code changes.

## Entry Baseline Evidence (2026-03-14)

Positive-path checks:

- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/nested_function_basic.sifr` -> pass
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/nested_function_recursive.sifr` -> pass
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/nested_function_recursive_capture.sifr` -> pass

Known failing inference-driven checks:

- `cargo run -q -p sifr -- check audits/leetcode/0017_letter_combinations_of_a_phone_number.sifr` -> fails with missing nested parameter annotations plus downstream `Any` index/iteration fallout
- `cargo run -q -p sifr -- check audits/leetcode/0050_powx_n.sifr` -> fails with missing nested parameter annotations plus downstream `Any` arithmetic fallout

Typed-callable / unresolved-symbol baseline:

- forward local helper used as a callable value before its `def` currently fails as `undefined variable: 'helper'`

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
- Entry-baseline evidence is recorded in the execution issue before part 1 starts, including:
  - at least one passing nested-function capture/recursion fixture,
  - at least one failing inference-driven LeetCode case,
  - at least one failing unsupported-shape or `Any`-fallback case.

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

A broader LeetCode corpus review found:

- `114` still-failing fixtures containing nested local functions,
- `95` still-failing fixtures after excluding recursive-type and duplicate-top-level-definition families,
- repeated failure shapes across lowering, inference, capture typing, and downstream `Any` leakage.

This is broader than a Phase 31 closure milestone. It is a language/compiler feature surface.

## Current Readiness Note

This phase is intentionally **not** marked ready or in progress yet.

Validated current state:

- there is already partial nested-function infrastructure in the compiler,
- there are existing passing fixtures for basic nested functions, recursive nested functions, and recursive capture,
- but the broad feature is still incomplete enough that many real nested-helper programs degrade to missing annotations and `Any` fallout.

This phase exists to close that gap through one coherent architecture rather than treating the current partial support as sufficient.

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
- nested-function registration already exists in partial form in HIR lowering,
- top-level callable machinery already exists and can be extended rather than replaced,
- the compiler already has evidence that recursive and backtracking helper patterns are intended source forms.

What is still incomplete:

- usage-driven nested parameter/return inference is not implemented for the real LeetCode helper shapes that depend on it,
- nested defs are not treated as fully typed callables early enough,
- supported local-helper patterns still require annotations that should be inferable,
- capture typing is not preserved strongly enough through recursion and mutation,
- unsupported nested-function shapes do not yet fail through one clean boundary and still frequently degrade to `Any`,
- unsupported or partially supported shapes degrade to `Any` instead of failing explicitly,
- downstream closure/codegen behavior is not yet owned by one documented architecture.

### Entry baseline examples to record when execution starts

Positive-path baseline examples:

- `crates/sifr/tests/e2e/pass/nested_function_basic.sifr`
- `crates/sifr/tests/e2e/pass/nested_function_recursive.sifr`
- `crates/sifr/tests/e2e/pass/nested_function_recursive_capture.sifr`

Known failing inference-driven examples:

- `audits/leetcode/0017_letter_combinations_of_a_phone_number.sifr`
- `audits/leetcode/0039_combination_sum.sifr`
- `audits/leetcode/0050_powx_n.sifr`
- `audits/leetcode/0078_subsets.sifr`
- `audits/leetcode/0090_subsets_ii.sifr`
- `audits/leetcode/0912_sort_an_array.sifr`

Current representative failure shapes from those cases include:

- `parameter 'i' in function 'backtrack' is missing a type annotation`
- `parameter 'curStr' in function 'backtrack' is missing a type annotation`
- `parameter 'n' in function 'helper' is missing a type annotation`
- `function expects return type 'Any', but returns nothing`
- `cannot index type 'list[int]' with 'Any'`
- `unsupported operand type(s) for +: 'Any' and 'int'`

Important interpretation:

- milestone 1 is only **partially** present today because symbol registration exists but a complete typed nested-callable model does not.
- milestone 2 is the clearest unimplemented core gap.
- milestone 3 and milestone 4 are only partially present today.

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
  - this milestone closes the gap between the current partial nested symbol registration and a real typed nested-callable model,
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
  - unsupported nested-function shapes fail with explicit diagnostics instead of downstream `Any` arithmetic/indexing fallout,
  - generated Rust stays panic-free for supported nested-helper paths.

### milestone_nested_5: Regression Corpus, Demos, and Full-Corpus Closure Evidence

- Scope:
  - add pass/fail regression coverage for nested-function families,
  - add milestone demos or explicitly replace the current legacy nested-functions demo with phase-owned closure evidence,
  - rerun the watched LeetCode nested-helper set and record closure evidence.
- Definition of done:
  - nested-function regressions are permanently locked,
  - phase-owned demo evidence executes successfully and reflects this phase's supported boundaries,
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

- targeted baseline checks and reruns recorded in the execution issue,
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
