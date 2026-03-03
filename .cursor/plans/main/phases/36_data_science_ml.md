# Phase 36: Data Science and ML

> Note: Needs more planning before execution (which data/ml subset to target, scope boundaries, dependencies, and acceptance gates are still draft-level).

## Objective
Add data science and ML capabilities after the web framework phase, while preserving existing reliability and diagnostics guarantees.

## Depends on
- Phase 35

## Milestones

### milestone_36_1: Data Processing
- Scope:
  - DataFrame workflows (CSV/Parquet I/O, transformations, aggregations).
  - Batch/lazy execution ergonomics suitable for Sifr data workloads.
- Definition of done:
  - Data processing workflows are stable and regression-covered.

### milestone_36_2: ML Inference
- Scope:
  - Model loading/inference runtime and typed input/output paths.
  - Tensor/array primitives needed for inference workloads.
- Definition of done:
  - ML inference paths are functional and test-covered.

## Exit Gate
- Data and ML workflows are usable end-to-end without regressing prior phase guarantees.
