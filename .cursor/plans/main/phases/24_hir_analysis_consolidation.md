# Phase 24: HIR Analysis Consolidation

## Objective
Consolidate HIR analysis into one canonical traversal/query architecture so emitters and lowering code do not carry ad-hoc recursive analysis logic.

## Depends on
- Phase 23

## Technical Context
- Canonical traversal/query helpers are maintained in `crates/sifr_codegen/src/helpers.rs`.
- Remaining emitter-local analysis logic currently lives in `crates/sifr_codegen/src/stmt_support_emitter.rs` (notably `body_always_exits_stmt`-style behavior).
- Phase 21 established traversal/control-flow correctness baseline; this phase consolidates analysis ownership so it no longer drifts across emitter-local implementations.
- Canonical vs ad-hoc criteria for this phase:
  - Canonical: analysis implemented in shared traversal/query modules with reusable interfaces.
  - Ad-hoc: recursive analysis logic embedded directly in emitter/lowering modules for local use only.

## Milestones

### milestone_24_1: Canonical Traversal Layer Contract
- Scope:
  - Establish one traversal layer as the only recursive descent over `HirStmt`/`HirExpr` for analysis use-cases.
  - Define and document traversal invariants and extension rules when HIR variants evolve.
- Definition of done:
  - Analysis recursion is centralized and versioned as a canonical contract.

### milestone_24_2: Semantic Query Layer Standardization
- Scope:
  - Build query APIs on top of the traversal layer for reusable analysis facts (for example: return/yield presence, function-call detection, defined-variable collection).
  - Enforce emitter/lowering consumers to call query APIs instead of implementing local recursive matching.
- Definition of done:
  - Semantic analyses are reusable queries, not duplicated emitter-local traversals.

### milestone_24_3: Control-Flow Effect Query Unification
- Scope:
  - Replace remaining ad-hoc `body_always_exits_stmt`-style logic with a shared control-flow effect query API.
  - Introduce a canonical effect model (for example: fallthrough, always returns, always raises) and use it consistently across affected call sites.
- Definition of done:
  - Exit-analysis behavior is computed through one shared query path with no local duplicates.

### milestone_24_4: Analysis/Emission Boundary Hardening
- Scope:
  - Define strict boundaries: analysis computes facts, emitters consume facts.
  - Remove analysis decisions embedded directly in emitter control flow where canonical queries exist.
- Definition of done:
  - Analysis and emission responsibilities are cleanly separated and regression-locked.

### milestone_24_5: Consolidation Regression Matrix
- Scope:
  - Add targeted regressions for prior drift-prone cases (nested conditionals, loop exits, early returns/raises, mixed block forms).
  - Add parity checks to ensure consolidated queries preserve intended semantics across existing constructs.
- Definition of done:
  - Traversal/query consolidation regressions are automatically detected before merge.

## Quality Contract
- Entry criteria: Phase 23 is completed and project graph/discovery behavior is stable.
- Exit criteria: HIR analysis is centralized behind canonical traversal/query APIs with no remaining ad-hoc emitter recursion for covered analyses.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_24_1` (Canonical Traversal Layer Contract): validation goals cover: Establish one traversal layer as the only recursive descent over `HirStmt`/`HirExpr` for analysis use-cases; Define and document traversal invariants and extension rules when HIR variants evolve. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_24_2` (Semantic Query Layer Standardization): validation goals cover: Build query APIs on top of the traversal layer for reusable analysis facts; Enforce emitter/lowering consumers to call query APIs instead of implementing local recursive matching. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_24_3` (Control-Flow Effect Query Unification): validation goals cover: Replace remaining ad-hoc `body_always_exits`-style logic with a shared control-flow effect query API; Introduce a canonical effect model and use it consistently across affected call sites. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_24_4` (Analysis/Emission Boundary Hardening): validation goals cover: Define strict boundaries where analysis computes facts and emitters consume facts; Remove analysis decisions embedded directly in emitter control flow where canonical queries exist. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_24_5` (Consolidation Regression Matrix): validation goals cover: Add targeted regressions for prior drift-prone cases and parity checks to ensure consolidated queries preserve intended semantics across existing constructs. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: HIR analysis is centralized behind canonical traversal/query APIs with no remaining ad-hoc emitter recursion for covered analyses.

## Exit Gate
- HIR analysis is centralized behind canonical traversal/query APIs with no remaining ad-hoc emitter recursion for covered analyses.
