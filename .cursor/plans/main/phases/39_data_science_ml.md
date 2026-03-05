# Phase 38: Data Science and ML

> Note: Needs more planning before execution (which data/ml subset to target, scope boundaries, dependencies, and acceptance gates are still draft-level).

## Objective
Add data science and ML capabilities after the web framework phase, while preserving existing reliability and diagnostics guarantees.

## Depends on
- Phase 37

## Milestones

### milestone_38_1: Data Processing
- Scope:
  - DataFrame workflows (CSV/Parquet I/O, transformations, aggregations).
  - Batch/lazy execution ergonomics suitable for Sifr data workloads.
- Definition of done:
  - Data processing workflows are stable and regression-covered.

### milestone_38_2: ML Inference
- Scope:
  - Model loading/inference runtime and typed input/output paths.
  - Tensor/array primitives needed for inference workloads.
- Definition of done:
  - ML inference paths are functional and test-covered.

## Quality Contract
- Entry criteria: Phase 37 is completed and prior reliability/diagnostics guarantees remain green.
- Exit criteria: Data and ML workflows are usable end-to-end without regressing prior phase guarantees.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_38_1` (Data Processing): validation goals cover: DataFrame workflows (CSV/Parquet I/O, transformations, aggregations); Batch/lazy execution ergonomics suitable for Sifr data workloads. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_38_2` (ML Inference): validation goals cover: Model loading/inference runtime and typed input/output paths; Tensor/array primitives needed for inference workloads. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Data and ML workflows are usable end-to-end without regressing prior phase guarantees.

## Exit Gate
- Data and ML workflows are usable end-to-end without regressing prior phase guarantees.
