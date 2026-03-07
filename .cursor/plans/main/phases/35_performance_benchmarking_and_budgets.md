# Phase 35: Performance Benchmarking, Shared Analysis Query Architecture, and Budgets

## Objective
Establish and enforce compiler-focused performance budgets (compile-time, compiler memory, and check/build latency), while defining the canonical reusable analysis/query architecture and cache-invalidation contracts consumed by both CLI and future tooling.

## Depends on
- Phase 34

## Milestones

### milestone_35_1: Baseline Benchmark Suite
- Scope:
  - Define compiler benchmark suites for `check`, `build`, and incremental local loops.
- Definition of done:
  - Baselines are versioned and reproducible locally.

### milestone_35_2: Budget and Threshold Policy
- Scope:
  - Set compiler regression thresholds and waiver process.
- Definition of done:
  - Performance budget policy is documented and testable.

### milestone_35_3: Enforcement Integration
- Scope:
  - Add local and CI gates for benchmark regressions.
- Definition of done:
  - Regressions fail gates unless approved waiver exists.

### milestone_35_4: Shared Analysis Query Architecture and Cache Contracts
- Scope:
  - Introduce the canonical reusable frontend analysis/query API for parse/lower/type-check/diagnostics.
  - Define the minimum required API surface explicitly:
    - create/load one project or compilation context
    - parse one module or project input set
    - lower parsed modules through the canonical frontend pipeline
    - type-check and collect canonical diagnostics
    - inspect module/project graph state needed by CLI and adapter consumers
    - request per-module and whole-project analysis results without reimplementing semantics
  - Define dependency-tracked query/cache architecture at module granularity.
  - Define deterministic invalidation rules and cache-consistency guarantees for local loops.
  - Make the compiler CLI consume the same analysis/query ownership model that future tooling must use.
- Definition of done:
  - Shared analysis/query design and cache contracts are explicit, deterministic, and regression-covered.
  - The anti-split-brain foundation is in place before standalone tooling surfaces begin.
  - The minimum API surface is documented clearly enough that Phase 36 can consume it without inventing new semantics-bearing entrypoints.

## Quality Contract
- Entry criteria: Phase 34 is completed and generated-code quality gates are enforced.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Performance regressions are systematically detected and controlled, and the canonical shared analysis/query foundation is established.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_35_1` (Baseline Benchmark Suite): validation goals cover: Define compiler benchmark suites for `check`, `build`, and incremental local loops. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_35_2` (Budget and Threshold Policy): validation goals cover: Set compiler regression thresholds and waiver process. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_35_3` (Enforcement Integration): validation goals cover: Add local and CI gates for benchmark regressions. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_35_4` (Shared Analysis Query Architecture and Cache Contracts): validation goals cover: Introduce the canonical reusable frontend analysis/query API for parse/lower/type-check/diagnostics; Define the minimum required API surface for project context creation, parse, lower, type-check, diagnostics, graph inspection, and analysis queries; Define dependency-tracked query/cache architecture at module granularity; Define deterministic invalidation rules and cache-consistency guarantees for local loops; Make the compiler CLI consume the same analysis/query ownership model that future tooling must use. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Performance regressions are systematically detected and controlled, and the canonical shared analysis/query foundation is established.

## Exit Gate
- Performance regressions are systematically detected and controlled, and the canonical shared analysis/query foundation is established.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
