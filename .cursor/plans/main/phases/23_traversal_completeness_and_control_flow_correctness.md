# Phase 22: Traversal Completeness and Control-Flow Correctness

## Objective
Guarantee walkers and control-flow analyses cover all supported constructs correctly.

## Depends on
- Phase 21

## Milestones

### milestone_22_1: Canonical Walker Coverage
- Scope:
  - Standardize recursive traversal across statement/expression variants.
  - Remove partial traversal blind spots.
- Definition of done:
  - Traversal completeness matrix is satisfied for supported nodes.

### milestone_22_2: `while ... else` End-to-End Support
- Scope:
  - Implement intended Python-like `while ... else` semantics through HIR and codegen.
- Definition of done:
  - `while ... else` behavior matches language intent with regression tests.

### milestone_22_3: Yield and Exception-Path Coverage
- Scope:
  - Fix generator/yield detection across nested constructs.
  - Ensure try/except analysis includes loop-else and other missed paths.
- Definition of done:
  - No known missed traversal paths in generator/error analysis.

## Exit Gate
- Control-flow lowering/analysis is complete for supported syntax and semantics.
